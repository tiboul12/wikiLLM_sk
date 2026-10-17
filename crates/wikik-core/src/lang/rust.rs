//! Extraction Rust avec tree-sitter : symboles, blocs de doc, balises, hash, imports et sites d'appel.
//!
//! Parcours manuel de l'arbre (et non requêtes `.scm`) pour suivre la portée : modules en ligne,
//! impl, traits (SPEC-03). Règles de hash et d'identifiants validées par SPIKE-01.
//!
//! @req EF-001, EF-006, EF-007, ENF-006
//! @adr 2

use super::Langage;
use crate::hash::{empreinte, empreinte_morceaux};
use crate::modele::{AppelBrut, Balise, ExtraitFichier, Import, Symbole};
use std::collections::HashMap;
use tree_sitter::{Node, Parser};

/// Balises reconnues par SPEC-01 ; les autres sont gardées mais signalées par `check`.
pub const BALISES_CONNUES: &[&str] = &["adr", "req", "topic", "pitfall", "relu"];

/// Langage Rust.
pub struct Rust;

impl Langage for Rust {
    fn nom(&self) -> &'static str {
        "rust"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn extraire(&self, chemin: &str, source: &[u8]) -> ExtraitFichier {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("grammaire tree-sitter-rust incompatible");
        let Some(arbre) = parser.parse(source, None) else {
            return ExtraitFichier {
                lignes: compter_lignes(source),
                erreurs_parse: 1,
                ..Default::default()
            };
        };
        let racine = arbre.root_node();
        let mut ctx = Ctx {
            src: source,
            chemin,
            symboles: vec![],
        };
        parcourir_items(racine, &Portee::default(), &mut ctx);
        let mut symboles = ctx.symboles;
        completer_visibilite_impl_trait(&mut symboles);
        desambiguiser(&mut symboles);
        symboles.sort_by(|a, b| a.ligne_debut.cmp(&b.ligne_debut).then(a.id.cmp(&b.id)));

        let mut imports = vec![];
        let mut c = racine.walk();
        for enfant in racine.children(&mut c) {
            if enfant.kind() == "use_declaration"
                && let Some(arg) = enfant.child_by_field_name("argument")
            {
                aplatir_use(arg, "", source, &mut imports);
            }
        }
        imports.sort_by(|a, b| a.alias.cmp(&b.alias).then(a.chemin.cmp(&b.chemin)));
        imports.dedup();

        ExtraitFichier {
            entete: entete(racine, source),
            symboles,
            imports,
            lignes: compter_lignes(source),
            lignes_macros: lignes_macros(racine),
            erreurs_parse: compter_erreurs(racine),
        }
    }
}

struct Ctx<'a> {
    src: &'a [u8],
    chemin: &'a str,
    symboles: Vec<Symbole>,
}

/// Portée courante pendant le parcours.
#[derive(Default, Clone)]
struct Portee {
    /// Préfixe du nom qualifié (`interne`, `Point`, `<Point as Display>`).
    prefixe: String,
    /// Type de l'impl englobant, pour `type_parent`.
    type_impl: Option<String>,
    /// `Some(trait)` dans un `impl Trait for Type`.
    trait_impl: Option<String>,
    /// Visibilité imposée (méthodes d'un trait : celle du trait).
    visibilite_heritee: Option<String>,
    dans_impl_ou_trait: bool,
    est_test: bool,
}

fn txt<'a>(n: Node, src: &'a [u8]) -> &'a str {
    n.utf8_text(src).unwrap_or("")
}

fn compter_lignes(src: &[u8]) -> usize {
    if src.is_empty() {
        return 0;
    }
    src.iter().filter(|b| **b == b'\n').count() + usize::from(*src.last().unwrap() != b'\n')
}

fn qualifier(prefixe: &str, nom: &str) -> String {
    if prefixe.is_empty() {
        nom.to_string()
    } else {
        format!("{prefixe}::{nom}")
    }
}

/// `Foo<T>` → `Foo`, `&mut Foo` → `Foo`, `std::fmt::Display` → `Display` (SPEC-02).
fn nom_type(n: Node, src: &[u8]) -> String {
    let t = txt(n, src).trim();
    let t = t.trim_start_matches('&').trim_start();
    let t = t.strip_prefix("mut ").unwrap_or(t).trim();
    let sans_generiques = t.split('<').next().unwrap_or(t).trim();
    sans_generiques
        .rsplit("::")
        .next()
        .unwrap_or(sans_generiques)
        .trim()
        .to_string()
}

fn visibilite(item: Node, src: &[u8]) -> Option<String> {
    let mut c = item.walk();
    for enfant in item.children(&mut c) {
        if enfant.kind() == "visibility_modifier" {
            let t: String = txt(enfant, src)
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            return Some(if t == "pub" {
                "pub".into()
            } else {
                "pub-crate".into()
            });
        }
    }
    None
}

/// Doc et attributs : frères précédents contigus de l'item (SPIKE-01 S3).
struct Prelude {
    lignes_doc: Vec<String>,
    doc_ligne_debut: Option<usize>,
    doc_ligne_fin: Option<usize>,
    attributs: Vec<String>,
}

fn prelude(item: Node, src: &[u8]) -> Prelude {
    let mut docs: Vec<(usize, usize, Vec<String>)> = vec![];
    let mut attributs = vec![];
    let mut p = item.prev_sibling();
    while let Some(s) = p {
        match s.kind() {
            "line_comment" => {
                let t = txt(s, src);
                if t.starts_with("///") && !t.starts_with("////") {
                    let l = t.trim_end_matches(['\n', '\r']);
                    let corps = l.strip_prefix("///").unwrap_or(l);
                    let corps = corps.strip_prefix(' ').unwrap_or(corps);
                    let ligne = s.start_position().row + 1;
                    docs.push((ligne, ligne, vec![corps.trim_end().to_string()]));
                } else {
                    break;
                }
            }
            "block_comment" => {
                let t = txt(s, src);
                if t.starts_with("/**") && !t.starts_with("/***") && t != "/**/" {
                    let interieur = t.trim_start_matches("/**").trim_end_matches("*/");
                    let lignes = interieur
                        .lines()
                        .map(|l| {
                            let l = l.trim();
                            l.strip_prefix("* ")
                                .or_else(|| l.strip_prefix('*'))
                                .unwrap_or(l)
                                .to_string()
                        })
                        .collect::<Vec<_>>();
                    docs.push((s.start_position().row + 1, s.end_position().row + 1, lignes));
                } else {
                    break;
                }
            }
            "attribute_item" => attributs.push(txt(s, src).to_string()),
            _ => break,
        }
        p = s.prev_sibling();
    }
    docs.reverse();
    attributs.reverse();
    let doc_ligne_debut = docs.first().map(|d| d.0);
    let doc_ligne_fin = docs.last().map(|d| d.1);
    let mut lignes_doc: Vec<String> = docs.into_iter().flat_map(|d| d.2).collect();
    while lignes_doc.first().is_some_and(|l| l.trim().is_empty()) {
        lignes_doc.remove(0);
    }
    while lignes_doc.last().is_some_and(|l| l.trim().is_empty()) {
        lignes_doc.pop();
    }
    Prelude {
        lignes_doc,
        doc_ligne_debut,
        doc_ligne_fin,
        attributs,
    }
}

/// Première phrase du premier paragraphe, hors balises (SPEC-01 R2).
pub fn resume(lignes: &[String]) -> Option<String> {
    let mut para = vec![];
    for l in lignes {
        let t = l.trim();
        if t.is_empty() {
            if para.is_empty() {
                continue;
            }
            break;
        }
        if est_ligne_balise(t) {
            break;
        }
        para.push(t);
    }
    if para.is_empty() {
        return None;
    }
    let texte = para.join(" ");
    let mut fin = texte.len();
    for (i, _) in texte.match_indices(". ") {
        // Ignore les abréviations courtes (« ex. », « cf. », « p. ex. »).
        let mot = texte[..i].rsplit(' ').next().unwrap_or("");
        if mot.chars().count() > 3 {
            fin = i + 1;
            break;
        }
    }
    let phrase = texte[..fin].trim().to_string();
    if phrase.chars().count() > 200 {
        let coupe: String = phrase.chars().take(197).collect();
        Some(format!("{}…", coupe.trim_end()))
    } else {
        Some(phrase)
    }
}

fn est_ligne_balise(t: &str) -> bool {
    let mut it = t.chars();
    it.next() == Some('@') && it.next().is_some_and(|c| c.is_ascii_lowercase())
}

fn balises(lignes: &[String], ligne_debut: Option<usize>) -> Vec<Balise> {
    let mut v = vec![];
    for (i, l) in lignes.iter().enumerate() {
        let t = l.trim();
        if !est_ligne_balise(t) {
            continue;
        }
        let corps = &t[1..];
        let (nom, valeur) = corps.split_once(char::is_whitespace).unwrap_or((corps, ""));
        v.push(Balise {
            nom: nom.to_string(),
            valeur: valeur.trim().to_string(),
            ligne: ligne_debut.map_or(0, |d| d + i),
        });
    }
    v
}

/// Hash normalisé (SPIKE-01 S2) : feuilles hors commentaires, sans virgule finale avant un
/// délimiteur fermant, sans accolades d'un bloc à expression unique sous `match`/closure.
pub fn hash_normalise(n: Node, src: &[u8], sauter: Option<usize>) -> String {
    let mut feuilles: Vec<(usize, &str)> = vec![];
    let mut pile = vec![n];
    while let Some(x) = pile.pop() {
        if Some(x.id()) == sauter || x.kind() == "line_comment" || x.kind() == "block_comment" {
            continue;
        }
        if x.child_count() == 0 {
            if x.kind() == ","
                && x.next_sibling()
                    .is_some_and(|s| matches!(s.kind(), ")" | "]" | "}" | ">"))
            {
                continue;
            }
            if matches!(x.kind(), "{" | "}") && accolade_facultative(x) {
                continue;
            }
            feuilles.push((x.start_byte(), txt(x, src)));
        } else {
            let mut c = x.walk();
            for enfant in x.children(&mut c) {
                pile.push(enfant);
            }
        }
    }
    feuilles.sort_by_key(|f| f.0);
    empreinte_morceaux(feuilles.into_iter().map(|f| f.1))
}

fn accolade_facultative(accolade: Node) -> bool {
    let Some(bloc) = accolade.parent() else {
        return false;
    };
    if bloc.kind() != "block" || bloc.named_child_count() != 1 {
        return false;
    }
    let Some(seul) = bloc.named_child(0) else {
        return false;
    };
    let instruction = seul.kind() == "let_declaration"
        || (seul.kind() == "expression_statement"
            && seul
                .child(seul.child_count().saturating_sub(1))
                .is_some_and(|l| l.kind() == ";"));
    !instruction
        && bloc
            .parent()
            .is_some_and(|p| matches!(p.kind(), "match_arm" | "closure_expression"))
}

/// Texte de l'item jusqu'à son corps, blancs compactés : ce qu'affichent les requêtes.
fn texte_signature(item: Node, corps: Option<Node>, src: &[u8]) -> String {
    let fin = corps.map_or(item.end_byte(), |c| c.start_byte());
    let brut = std::str::from_utf8(&src[item.start_byte()..fin]).unwrap_or("");
    let compact = brut.split_whitespace().collect::<Vec<_>>().join(" ");
    let compact = compact.trim_end_matches(['{', ';']).trim().to_string();
    if compact.chars().count() > 300 {
        let c: String = compact.chars().take(297).collect();
        format!("{c}…")
    } else {
        compact
    }
}

fn est_attribut_test(attrs: &[String]) -> bool {
    attrs.iter().any(|a| {
        let a: String = a.chars().filter(|c| !c.is_whitespace()).collect();
        a == "#[test]" || a.ends_with("::test]") || a == "#[cfg(test)]"
    })
}

fn predicat_cfg(attrs: &[String]) -> Option<String> {
    attrs.iter().find_map(|a| {
        let a: String = a.chars().filter(|c| !c.is_whitespace()).collect();
        a.strip_prefix("#[cfg(")
            .and_then(|r| r.strip_suffix(")]"))
            .map(str::to_string)
    })
}

fn parcourir_items(noeud: Node, portee: &Portee, ctx: &mut Ctx) {
    let mut c = noeud.walk();
    let enfants: Vec<Node> = noeud.children(&mut c).collect();
    for item in enfants {
        let src = ctx.src;
        match item.kind() {
            "function_item" | "function_signature_item" => ajouter_fonction(item, portee, ctx),
            "struct_item" | "enum_item" | "union_item" | "trait_item" | "type_item"
            | "const_item" | "static_item" | "macro_definition" => {
                let nom = item
                    .child_by_field_name("name")
                    .map(|n| txt(n, src).to_string())
                    .unwrap_or_default();
                let genre = match item.kind() {
                    "struct_item" => "struct",
                    "enum_item" => "enum",
                    "union_item" => "union",
                    "trait_item" => "trait",
                    "type_item" => "type",
                    "const_item" => "const",
                    "static_item" => "static",
                    _ => "macro",
                };
                let p = prelude(item, src);
                let vis = if genre == "macro" {
                    if p.attributs.iter().any(|a| a.contains("macro_export")) {
                        "pub".into()
                    } else {
                        "prive".into()
                    }
                } else {
                    portee
                        .visibilite_heritee
                        .clone()
                        .or_else(|| visibilite(item, src))
                        .unwrap_or_else(|| "prive".into())
                };
                let corps = item.child_by_field_name("body");
                let sig = if matches!(genre, "struct" | "enum" | "union" | "trait") {
                    texte_signature(item, corps, src)
                } else {
                    texte_signature(item, None, src)
                };
                let nq = qualifier(&portee.prefixe, &nom);
                let est_test = portee.est_test || est_attribut_test(&p.attributs);
                ctx.symboles.push(nouveau_symbole(
                    ctx.chemin,
                    &nom,
                    nq.clone(),
                    genre,
                    vis.clone(),
                    item,
                    &p,
                    sig,
                    hash_normalise(item, src, None),
                    String::new(),
                    est_test,
                    portee.type_impl.clone(),
                ));
                if genre == "trait"
                    && let Some(b) = corps
                {
                    let sous = Portee {
                        prefixe: nq,
                        type_impl: Some(nom.clone()),
                        trait_impl: None,
                        visibilite_heritee: Some(vis),
                        dans_impl_ou_trait: true,
                        est_test,
                    };
                    parcourir_items(b, &sous, ctx);
                }
            }
            "impl_item" => {
                let ty = item
                    .child_by_field_name("type")
                    .map(|t| nom_type(t, src))
                    .unwrap_or_default();
                let tr = item.child_by_field_name("trait").map(|t| nom_type(t, src));
                let prefixe = match &tr {
                    Some(t) => qualifier(&portee.prefixe, &format!("<{ty} as {t}>")),
                    None => qualifier(&portee.prefixe, &ty),
                };
                let p = prelude(item, src);
                if let Some(b) = item.child_by_field_name("body") {
                    let sous = Portee {
                        prefixe,
                        type_impl: Some(ty),
                        trait_impl: tr,
                        visibilite_heritee: None,
                        dans_impl_ou_trait: true,
                        est_test: portee.est_test || est_attribut_test(&p.attributs),
                    };
                    parcourir_items(b, &sous, ctx);
                }
            }
            "mod_item" => {
                let Some(corps) = item.child_by_field_name("body") else {
                    continue;
                };
                let nom = item
                    .child_by_field_name("name")
                    .map(|n| txt(n, src).to_string())
                    .unwrap_or_default();
                let p = prelude(item, src);
                let nq = qualifier(&portee.prefixe, &nom);
                let est_test = portee.est_test || est_attribut_test(&p.attributs);
                let vis = visibilite(item, src).unwrap_or_else(|| "prive".into());
                let sig = texte_signature(item, Some(corps), src);
                ctx.symboles.push(nouveau_symbole(
                    ctx.chemin,
                    &nom,
                    nq.clone(),
                    "mod",
                    vis,
                    item,
                    &p,
                    sig,
                    hash_normalise(item, src, None),
                    String::new(),
                    est_test,
                    None,
                ));
                let sous = Portee {
                    prefixe: nq,
                    est_test,
                    ..Default::default()
                };
                parcourir_items(corps, &sous, ctx);
            }
            _ => {}
        }
    }
}

fn ajouter_fonction(item: Node, portee: &Portee, ctx: &mut Ctx) {
    let src = ctx.src;
    let nom = item
        .child_by_field_name("name")
        .map(|n| txt(n, src).to_string())
        .unwrap_or_default();
    let p = prelude(item, src);
    let corps = item.child_by_field_name("body");
    let genre = if p.attributs.iter().any(|a| a.contains("component")) {
        "composant"
    } else if portee.dans_impl_ou_trait {
        "methode"
    } else {
        "fn"
    };
    let vis = portee
        .visibilite_heritee
        .clone()
        .or_else(|| visibilite(item, src))
        // Méthode d'impl de trait : complétée après coup par la visibilité du type.
        .unwrap_or_else(|| {
            if portee.trait_impl.is_some() {
                "?".into()
            } else {
                "prive".into()
            }
        });
    let hash_corps = corps.map_or_else(
        || hash_normalise(item, src, None),
        |b| hash_normalise(b, src, None),
    );
    let hash_signature = hash_normalise(item, src, corps.map(|b| b.id()));
    let mut s = nouveau_symbole(
        ctx.chemin,
        &nom,
        qualifier(&portee.prefixe, &nom),
        genre,
        vis,
        item,
        &p,
        texte_signature(item, corps, src),
        hash_corps,
        hash_signature,
        portee.est_test || est_attribut_test(&p.attributs),
        portee.type_impl.clone(),
    );
    if let Some(b) = corps {
        collecter_appels(b, src, &mut s.appels);
    }
    ctx.symboles.push(s);
}

#[allow(clippy::too_many_arguments)]
fn nouveau_symbole(
    chemin: &str,
    nom: &str,
    nom_qualifie: String,
    genre: &str,
    visibilite: String,
    item: Node,
    p: &Prelude,
    signature: String,
    hash_corps: String,
    hash_signature: String,
    est_test: bool,
    type_parent: Option<String>,
) -> Symbole {
    let a_doc = !p.lignes_doc.is_empty();
    let doc_normalisee = p
        .lignes_doc
        .iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n");
    Symbole {
        id: format!("{chemin}::{nom_qualifie}"),
        nom: nom.to_string(),
        nom_qualifie,
        genre: genre.to_string(),
        visibilite,
        ligne_debut: item.start_position().row + 1,
        ligne_fin: item.end_position().row + 1,
        doc_ligne_debut: p.doc_ligne_debut,
        doc_ligne_fin: p.doc_ligne_fin,
        signature,
        resume: resume(&p.lignes_doc),
        doc: a_doc.then(|| p.lignes_doc.join("\n")),
        hash_corps,
        hash_signature,
        hash_doc: a_doc.then(|| empreinte(doc_normalisee.as_bytes())),
        est_test,
        type_parent,
        balises: balises(&p.lignes_doc, p.doc_ligne_debut),
        cfg: predicat_cfg(&p.attributs),
        appels: vec![],
    }
}

/// Méthode d'un `impl Trait for Type` : visibilité du type s'il est défini dans le fichier (SPEC-03).
fn completer_visibilite_impl_trait(symboles: &mut [Symbole]) {
    let types: HashMap<String, String> = symboles
        .iter()
        .filter(|s| {
            matches!(
                s.genre.as_str(),
                "struct" | "enum" | "union" | "type" | "trait"
            )
        })
        .map(|s| (s.nom.clone(), s.visibilite.clone()))
        .collect();
    for s in symboles.iter_mut().filter(|s| s.visibilite == "?") {
        s.visibilite = s
            .type_parent
            .as_ref()
            .and_then(|t| types.get(t).cloned())
            .unwrap_or_else(|| "prive".into());
    }
}

/// Suffixe `@cfg(..)` puis `#n` pour les identifiants en collision dans le fichier (SPEC-02).
fn desambiguiser(symboles: &mut [Symbole]) {
    let mut compte: HashMap<String, usize> = HashMap::new();
    for s in symboles.iter() {
        *compte.entry(s.id.clone()).or_default() += 1;
    }
    for s in symboles.iter_mut() {
        if compte[&s.id] > 1
            && let Some(cfg) = &s.cfg
        {
            s.id = format!("{}@cfg({cfg})", s.id);
        }
    }
    let mut compte: HashMap<String, usize> = HashMap::new();
    for s in symboles.iter() {
        *compte.entry(s.id.clone()).or_default() += 1;
    }
    let mut vus: HashMap<String, usize> = HashMap::new();
    for s in symboles.iter_mut() {
        if compte[&s.id] > 1 {
            let n = vus.entry(s.id.clone()).or_default();
            *n += 1;
            if *n > 1 {
                s.id = format!("{}#{}", s.id, n);
            }
        }
    }
}

fn collecter_appels(corps: Node, src: &[u8], sortie: &mut Vec<AppelBrut>) {
    let mut pile = vec![corps];
    while let Some(x) = pile.pop() {
        match x.kind() {
            "call_expression" => {
                if let Some(f) = x.child_by_field_name("function")
                    && let Some(a) = appel_de(f, src, x.start_position().row + 1)
                {
                    sortie.push(a);
                }
            }
            "macro_invocation" => {
                let nom = x
                    .child_by_field_name("macro")
                    .map(|m| txt(m, src))
                    .unwrap_or("");
                if nom == "view" {
                    composants_rendus(x, src, sortie);
                }
                continue;
            }
            _ => {}
        }
        let mut c = x.walk();
        for enfant in x.children(&mut c) {
            pile.push(enfant);
        }
    }
    sortie.sort_by(|a, b| a.ligne.cmp(&b.ligne).then(a.cible.cmp(&b.cible)));
}

fn sans_turbofish(chemin: &str) -> String {
    let mut sortie = String::new();
    let mut profondeur = 0usize;
    for ch in chemin.chars() {
        match ch {
            '<' => profondeur += 1,
            '>' => profondeur = profondeur.saturating_sub(1),
            _ if profondeur == 0 => sortie.push(ch),
            _ => {}
        }
    }
    sortie
        .replace("::::", "::")
        .trim_end_matches("::")
        .to_string()
}

fn appel_de(f: Node, src: &[u8], ligne: usize) -> Option<AppelBrut> {
    match f.kind() {
        "identifier" => Some(AppelBrut {
            forme: "direct".into(),
            cible: txt(f, src).into(),
            qualificatif: None,
            ligne,
        }),
        "scoped_identifier" => {
            let c = sans_turbofish(txt(f, src));
            let c: String = c.chars().filter(|c| !c.is_whitespace()).collect();
            Some(AppelBrut {
                forme: "chemin".into(),
                cible: c,
                qualificatif: None,
                ligne,
            })
        }
        "field_expression" => {
            let champ = f.child_by_field_name("field")?;
            let receveur = f.child_by_field_name("value");
            let q = receveur
                .filter(|r| r.kind() == "self")
                .map(|_| "self".to_string());
            Some(AppelBrut {
                forme: "methode".into(),
                cible: txt(champ, src).into(),
                qualificatif: q,
                ligne,
            })
        }
        "generic_function" => appel_de(f.child_by_field_name("function")?, src, ligne),
        _ => None,
    }
}

/// `<Composant ...>` dans `view!` : arbre de tokens non analysé, heuristique de SPIKE-01 S5.
fn composants_rendus(macro_view: Node, src: &[u8], sortie: &mut Vec<AppelBrut>) {
    let mut pile = vec![macro_view];
    while let Some(y) = pile.pop() {
        if y.kind() == "<"
            && let Some(nx) = y.next_sibling()
        {
            let t = txt(nx, src);
            if nx.kind() == "identifier" && t.chars().next().is_some_and(|c| c.is_uppercase()) {
                sortie.push(AppelBrut {
                    forme: "rend".into(),
                    cible: t.into(),
                    qualificatif: None,
                    ligne: nx.start_position().row + 1,
                });
            }
        }
        let mut c = y.walk();
        for enfant in y.children(&mut c) {
            pile.push(enfant);
        }
    }
}

fn aplatir_use(n: Node, prefixe: &str, src: &[u8], sortie: &mut Vec<Import>) {
    let joindre = |a: &str, b: &str| {
        if a.is_empty() {
            b.to_string()
        } else {
            format!("{a}::{b}")
        }
    };
    match n.kind() {
        "identifier" | "scoped_identifier" | "crate" | "super" | "self" => {
            let chemin = joindre(prefixe, txt(n, src));
            let alias = chemin.rsplit("::").next().unwrap_or(&chemin).to_string();
            let (alias, chemin) = if alias == "self" {
                let parent = chemin.trim_end_matches("::self").to_string();
                (
                    parent.rsplit("::").next().unwrap_or(&parent).to_string(),
                    parent,
                )
            } else {
                (alias, chemin)
            };
            sortie.push(Import { alias, chemin });
        }
        "use_as_clause" => {
            if let (Some(p), Some(a)) = (
                n.child_by_field_name("path"),
                n.child_by_field_name("alias"),
            ) {
                sortie.push(Import {
                    alias: txt(a, src).into(),
                    chemin: joindre(prefixe, txt(p, src)),
                });
            }
        }
        "scoped_use_list" => {
            let base = n
                .child_by_field_name("path")
                .map(|p| joindre(prefixe, txt(p, src)))
                .unwrap_or_else(|| prefixe.to_string());
            if let Some(liste) = n.child_by_field_name("list") {
                aplatir_use(liste, &base, src, sortie);
            }
        }
        "use_list" => {
            let mut c = n.walk();
            for enfant in n.named_children(&mut c) {
                aplatir_use(enfant, prefixe, src, sortie);
            }
        }
        _ => {}
    }
}

fn entete(racine: Node, src: &[u8]) -> Option<String> {
    let mut lignes = vec![];
    let mut c = racine.walk();
    for enfant in racine.children(&mut c) {
        if enfant.kind() != "line_comment" {
            break;
        }
        let t = txt(enfant, src).trim_end_matches(['\n', '\r']);
        match t.strip_prefix("//!") {
            Some(r) => lignes.push(r.strip_prefix(' ').unwrap_or(r).trim_end().to_string()),
            // Commentaire ordinaire (licence…) avant l'en-tête : ignoré ; après : fin de l'en-tête.
            None if lignes.is_empty() => continue,
            None => break,
        }
    }
    while lignes.last().is_some_and(|l| l.is_empty()) {
        lignes.pop();
    }
    (!lignes.is_empty()).then(|| lignes.join("\n"))
}

fn lignes_macros(racine: Node) -> usize {
    let mut total = 0;
    let mut pile = vec![racine];
    while let Some(x) = pile.pop() {
        if x.kind() == "macro_invocation" {
            total += x.end_position().row - x.start_position().row + 1;
            continue;
        }
        let mut c = x.walk();
        for enfant in x.children(&mut c) {
            pile.push(enfant);
        }
    }
    total
}

fn compter_erreurs(racine: Node) -> usize {
    if !racine.has_error() {
        return 0;
    }
    let mut n = 0;
    let mut pile = vec![racine];
    while let Some(x) = pile.pop() {
        if x.is_error() || x.is_missing() {
            n += 1;
            continue;
        }
        if x.has_error() {
            let mut c = x.walk();
            for enfant in x.children(&mut c) {
                pile.push(enfant);
            }
        }
    }
    n
}

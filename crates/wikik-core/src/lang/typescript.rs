//! Extraction TypeScript avec tree-sitter : symboles, JSDoc, balises, hash, imports, appels (SPEC-15).
//!
//! Même modèle que l'extraction Rust : identifiants `chemin::Classe::methode`, trois hash, sites
//! d'appel bruts résolus globalement. Les imports relatifs sont résolus en chemins de fichiers
//! dès l'extraction, pour que la résolution n'ait qu'à comparer des chemins.
//!
//! @req EF-070, EF-071, EF-072, EF-073
//! @adr 2

use super::Langage;
use super::rust::{balises, compter_erreurs, compter_lignes, desambiguiser, resume};
use crate::hash::{empreinte, empreinte_morceaux};
use crate::modele::{AppelBrut, Balise, ExtraitFichier, Import, Symbole};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

/// Balises JSDoc courantes : légitimes dans un bloc TypeScript, jamais signalées par `check` (SPEC-15 R6).
pub const BALISES_JSDOC: &[&str] = &[
    "param",
    "returns",
    "return",
    "throws",
    "example",
    "deprecated",
    "see",
    "link",
    "template",
    "type",
    "typedef",
    "default",
    "remarks",
    "since",
    "internal",
    "private",
    "public",
    "override",
    "inheritdoc",
    "file",
    "fileoverview",
    "module",
];

/// Langage TypeScript (et TSX).
pub struct TypeScript;

impl Langage for TypeScript {
    fn nom(&self) -> &'static str {
        "typescript"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx", "mts", "cts"]
    }

    fn extraire(&self, chemin: &str, source: &[u8]) -> ExtraitFichier {
        if chemin.ends_with(".d.ts") {
            return ExtraitFichier {
                lignes: compter_lignes(source),
                ..Default::default()
            };
        }
        let grammaire = if chemin.ends_with(".tsx") {
            tree_sitter_typescript::LANGUAGE_TSX
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT
        };
        extraire_ts(chemin, source, &grammaire.into())
    }
}

/// Langage JavaScript : même extraction que TypeScript, avec la grammaire TSX (qui accepte le JSX).
pub struct JavaScript;

impl Langage for JavaScript {
    fn nom(&self) -> &'static str {
        "javascript"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["js", "jsx", "mjs", "cjs"]
    }

    fn extraire(&self, chemin: &str, source: &[u8]) -> ExtraitFichier {
        extraire_ts(chemin, source, &tree_sitter_typescript::LANGUAGE_TSX.into())
    }
}

/// Extraction commune à TypeScript, JavaScript et aux scripts Svelte.
pub(crate) fn extraire_ts(
    chemin: &str,
    source: &[u8],
    grammaire: &tree_sitter::Language,
) -> ExtraitFichier {
    let mut parser = Parser::new();
    parser
        .set_language(grammaire)
        .expect("grammaire tree-sitter-typescript incompatible");
    let Some(arbre) = parser.parse(source, None) else {
        return ExtraitFichier {
            lignes: compter_lignes(source),
            erreurs_parse: 1,
            ..Default::default()
        };
    };
    let racine = arbre.root_node();
    let (entete, balises_entete, commentaires_entete) = entete(racine, source);
    let mut ctx = Ctx {
        src: source,
        chemin,
        symboles: vec![],
        exclus: commentaires_entete,
        fichier_de_test: est_fichier_de_test(chemin),
    };
    parcourir(racine, &Portee::default(), &mut ctx);
    let mut symboles = ctx.symboles;
    desambiguiser(&mut symboles);
    symboles.sort_by(|a, b| a.ligne_debut.cmp(&b.ligne_debut).then(a.id.cmp(&b.id)));

    let mut imports = vec![];
    let mut c = racine.walk();
    for enfant in racine.children(&mut c) {
        if enfant.kind() == "import_statement" {
            lire_import(enfant, source, chemin, &mut imports);
        }
    }
    imports.sort_by(|a, b| a.alias.cmp(&b.alias).then(a.chemin.cmp(&b.chemin)));
    imports.dedup();

    ExtraitFichier {
        entete,
        balises_entete,
        symboles,
        imports,
        lignes: compter_lignes(source),
        lignes_macros: 0,
        erreurs_parse: compter_erreurs(racine),
    }
}

/// Appels faits par les instructions de premier niveau (hors déclarations) : la logique d'un
/// composant Svelte qui s'exécute à son montage.
pub(crate) fn appels_niveau_module(
    source: &[u8],
    grammaire: &tree_sitter::Language,
) -> Vec<AppelBrut> {
    let mut parser = Parser::new();
    if parser.set_language(grammaire).is_err() {
        return vec![];
    }
    let Some(arbre) = parser.parse(source, None) else {
        return vec![];
    };
    let racine = arbre.root_node();
    let mut appels = vec![];
    let mut c = racine.walk();
    for n in racine.named_children(&mut c) {
        if n.kind() == "expression_statement" {
            collecter_appels(n, source, &mut appels);
        }
    }
    appels
}

fn est_fichier_de_test(chemin: &str) -> bool {
    [
        ".spec.ts",
        ".test.ts",
        ".spec.tsx",
        ".test.tsx",
        ".spec.mts",
        ".test.mts",
    ]
    .iter()
    .any(|s| chemin.ends_with(s))
}

struct Ctx<'a> {
    src: &'a [u8],
    chemin: &'a str,
    symboles: Vec<Symbole>,
    /// Commentaires d'en-tête : ils ne sont la doc d'aucune déclaration.
    exclus: HashSet<usize>,
    fichier_de_test: bool,
}

#[derive(Default, Clone)]
struct Portee {
    prefixe: String,
    classe: Option<String>,
    visibilite_classe: Option<String>,
}

fn txt<'a>(n: Node, src: &'a [u8]) -> &'a str {
    n.utf8_text(src).unwrap_or("")
}

fn qualifier(prefixe: &str, nom: &str) -> String {
    if prefixe.is_empty() {
        nom.to_string()
    } else {
        format!("{prefixe}::{nom}")
    }
}

/// Lignes d'un commentaire `/** … */` sans les marqueurs ni les `*` de début de ligne.
pub(crate) fn lignes_jsdoc(t: &str) -> Vec<String> {
    let interieur = t.trim_start_matches("/**").trim_end_matches("*/");
    let mut lignes: Vec<String> = interieur
        .lines()
        .map(|l| {
            let l = l.trim();
            l.strip_prefix("* ")
                .or_else(|| l.strip_prefix('*'))
                .unwrap_or(l)
                .trim_end()
                .to_string()
        })
        .collect();
    while lignes.first().is_some_and(|l| l.is_empty()) {
        lignes.remove(0);
    }
    while lignes.last().is_some_and(|l| l.is_empty()) {
        lignes.pop();
    }
    lignes
}

/// En-tête : commentaires `//` de tête, ou `/** */` de tête marqué `@file`/`@module` ou suivi d'une ligne vide.
pub(crate) fn entete(racine: Node, src: &[u8]) -> (Option<String>, Vec<Balise>, HashSet<usize>) {
    let mut lignes = vec![];
    let mut ids = HashSet::new();
    let mut premiere = None;
    let mut c = racine.walk();
    let enfants: Vec<Node> = racine.children(&mut c).collect();
    if let Some(&premier) = enfants.first()
        && premier.kind() == "comment"
    {
        let t = txt(premier, src);
        if t.starts_with("//") {
            for n in enfants
                .iter()
                .take_while(|n| n.kind() == "comment" && txt(**n, src).starts_with("//"))
            {
                premiere.get_or_insert(n.start_position().row + 1);
                let l = txt(*n, src).trim_start_matches('/');
                lignes.push(l.strip_prefix(' ').unwrap_or(l).trim_end().to_string());
                ids.insert(n.id());
            }
        } else if t.starts_with("/**") {
            let marque = ["@file", "@fileoverview", "@module"]
                .iter()
                .any(|m| t.contains(m));
            let detache = enfants
                .get(1)
                .is_none_or(|s| s.start_position().row > premier.end_position().row + 1);
            if marque || detache {
                premiere = Some(premier.start_position().row + 1);
                lignes = lignes_jsdoc(t);
                ids.insert(premier.id());
            }
        }
    }
    while lignes.last().is_some_and(|l| l.is_empty()) {
        lignes.pop();
    }
    let b = balises(&lignes, premiere);
    ((!lignes.is_empty()).then(|| lignes.join("\n")), b, ids)
}

/// Doc d'une déclaration : le `/** */` qui précède la déclaration ou son `export`, décorateurs sautés.
struct Doc {
    lignes: Vec<String>,
    debut: Option<usize>,
    fin: Option<usize>,
    decorateurs: Vec<String>,
}

fn doc_de(cible: Node, src: &[u8], exclus: &HashSet<usize>) -> Doc {
    let mut decorateurs = vec![];
    let mut c = cible.walk();
    for e in cible.children(&mut c) {
        if e.kind() == "decorator" {
            decorateurs.push(txt(e, src).to_string());
        }
    }
    let mut p = cible.prev_sibling();
    while let Some(s) = p {
        if s.kind() == "decorator" {
            decorateurs.push(txt(s, src).to_string());
            p = s.prev_sibling();
            continue;
        }
        break;
    }
    match p {
        Some(s) if s.kind() == "comment" && !exclus.contains(&s.id()) => {
            let t = txt(s, src);
            if t.starts_with("/**") && t != "/**/" {
                return Doc {
                    lignes: lignes_jsdoc(t),
                    debut: Some(s.start_position().row + 1),
                    fin: Some(s.end_position().row + 1),
                    decorateurs,
                };
            }
            Doc {
                lignes: vec![],
                debut: None,
                fin: None,
                decorateurs,
            }
        }
        _ => Doc {
            lignes: vec![],
            debut: None,
            fin: None,
            decorateurs,
        },
    }
}

/// Hash normalisé (SPEC-15 R7) : hors commentaires, `;`, guillemets, virgules finales,
/// modificateurs d'accessibilité et parenthèses d'un paramètre unique de fonction fléchée.
pub fn hash_normalise(n: Node, src: &[u8], sauter: Option<usize>) -> String {
    let mut feuilles: Vec<(usize, &str)> = vec![];
    let mut pile = vec![n];
    while let Some(x) = pile.pop() {
        if Some(x.id()) == sauter || matches!(x.kind(), "comment" | "accessibility_modifier") {
            continue;
        }
        if x.child_count() == 0 {
            let k = x.kind();
            if k == ";" || k == "\"" || k == "'" {
                continue;
            }
            if k == ","
                && x.next_sibling()
                    .is_some_and(|s| matches!(s.kind(), ")" | "]" | "}" | ">"))
            {
                continue;
            }
            if matches!(k, "(" | ")")
                && x.parent().is_some_and(|p| {
                    p.kind() == "formal_parameters"
                        && p.named_child_count() == 1
                        && p.parent().is_some_and(|f| f.kind() == "arrow_function")
                })
            {
                continue;
            }
            feuilles.push((x.start_byte(), txt(x, src)));
        } else {
            let mut c = x.walk();
            for e in x.children(&mut c) {
                pile.push(e);
            }
        }
    }
    feuilles.sort_by_key(|f| f.0);
    empreinte_morceaux(feuilles.into_iter().map(|f| f.1))
}

fn texte_signature(cible: Node, corps: Option<Node>, src: &[u8]) -> String {
    let debut = {
        // Les décorateurs ne font pas partie de la signature affichée.
        let mut c = cible.walk();
        cible
            .children(&mut c)
            .find(|e| e.kind() != "decorator" && e.kind() != "comment")
            .map_or(cible.start_byte(), |e| e.start_byte())
    };
    let fin = corps
        .map_or(cible.end_byte(), |c| c.start_byte())
        .max(debut);
    let brut = std::str::from_utf8(&src[debut..fin]).unwrap_or("");
    let compact = brut.split_whitespace().collect::<Vec<_>>().join(" ");
    let compact = compact
        .trim_end_matches(['{', ';', '='])
        .trim_end_matches("=>")
        .trim()
        .to_string();
    if compact.chars().count() > 300 {
        format!("{}…", compact.chars().take(297).collect::<String>())
    } else {
        compact
    }
}

#[allow(clippy::too_many_arguments)]
fn ajouter(
    ctx: &mut Ctx,
    nom: &str,
    genre: &str,
    visibilite: &str,
    cible: Node,
    corps: Option<Node>,
    portee: &Portee,
    hash_entier: bool,
) {
    let src = ctx.src;
    let doc = doc_de(cible, src, &ctx.exclus);
    let genre = if genre == "classe" && doc.decorateurs.iter().any(|d| d.starts_with("@Component"))
    {
        "composant"
    } else {
        genre
    };
    let nq = qualifier(&portee.prefixe, nom);
    let a_doc = !doc.lignes.is_empty();
    let doc_normalisee = doc
        .lignes
        .iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n");
    let (hash_corps, hash_signature) = if hash_entier {
        (hash_normalise(cible, src, None), String::new())
    } else {
        (
            corps.map_or_else(
                || hash_normalise(cible, src, None),
                |b| hash_normalise(b, src, None),
            ),
            hash_normalise(cible, src, corps.map(|b| b.id())),
        )
    };
    let mut appels = vec![];
    if let Some(b) = corps
        && !hash_entier
    {
        collecter_appels(b, src, &mut appels);
    }
    ctx.symboles.push(Symbole {
        id: format!("{}::{nq}", ctx.chemin),
        nom: nom.to_string(),
        nom_qualifie: nq,
        genre: genre.to_string(),
        visibilite: visibilite.to_string(),
        ligne_debut: cible.start_position().row + 1,
        ligne_fin: cible.end_position().row + 1,
        doc_ligne_debut: doc.debut,
        doc_ligne_fin: doc.fin,
        signature: texte_signature(cible, corps, src),
        resume: resume(&doc.lignes),
        doc: a_doc.then(|| doc.lignes.join("\n")),
        hash_corps,
        hash_signature,
        hash_doc: a_doc.then(|| empreinte(doc_normalisee.as_bytes())),
        est_test: ctx.fichier_de_test,
        type_parent: portee.classe.clone(),
        balises: balises(&doc.lignes, doc.debut),
        cfg: None,
        appels,
    });
}

fn nom_de(n: Node, src: &[u8]) -> Option<String> {
    n.child_by_field_name("name")
        .map(|x| txt(x, src).to_string())
}

fn parcourir(noeud: Node, portee: &Portee, ctx: &mut Ctx) {
    let mut c = noeud.walk();
    let enfants: Vec<Node> = noeud.named_children(&mut c).collect();
    for n in enfants {
        match n.kind() {
            "export_statement" => {
                if let Some(d) = n.child_by_field_name("declaration") {
                    declaration(d, n, "pub", portee, ctx);
                }
            }
            "expression_statement" => {
                let mut c2 = n.walk();
                for e in n.named_children(&mut c2) {
                    if matches!(e.kind(), "internal_module" | "module") {
                        declaration(e, e, "prive", portee, ctx);
                    }
                }
            }
            _ => declaration(n, n, "prive", portee, ctx),
        }
    }
}

/// `d` : la déclaration ; `cible` : le nœud qui porte doc et décorateurs (l'`export` s'il existe).
/// @relu 2026-10-17 0a363323
fn declaration(d: Node, cible: Node, visibilite: &str, portee: &Portee, ctx: &mut Ctx) {
    let src = ctx.src;
    match d.kind() {
        "function_declaration" | "generator_function_declaration" => {
            if let Some(nom) = nom_de(d, src) {
                ajouter(
                    ctx,
                    &nom,
                    "fn",
                    visibilite,
                    cible,
                    d.child_by_field_name("body"),
                    portee,
                    false,
                );
            }
        }
        "class_declaration" | "abstract_class_declaration" => {
            let Some(nom) = nom_de(d, src) else { return };
            let corps = d.child_by_field_name("body");
            ajouter(ctx, &nom, "classe", visibilite, cible, corps, portee, true);
            if let Some(b) = corps {
                let sous = Portee {
                    prefixe: qualifier(&portee.prefixe, &nom),
                    classe: Some(nom),
                    visibilite_classe: Some(visibilite.to_string()),
                };
                membres(b, &sous, ctx);
            }
        }
        "interface_declaration" | "type_alias_declaration" | "enum_declaration" => {
            if let Some(nom) = nom_de(d, src) {
                let genre = match d.kind() {
                    "interface_declaration" => "interface",
                    "type_alias_declaration" => "type",
                    _ => "enum",
                };
                ajouter(ctx, &nom, genre, visibilite, cible, None, portee, true);
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            let mut c = d.walk();
            let declarateurs: Vec<Node> = d
                .named_children(&mut c)
                .filter(|x| x.kind() == "variable_declarator")
                .collect();
            let seul = declarateurs.len() == 1;
            for v in declarateurs {
                let Some(nom_n) = v.child_by_field_name("name") else {
                    continue;
                };
                if nom_n.kind() != "identifier" {
                    continue;
                }
                let nom = txt(nom_n, src).to_string();
                let valeur = v.child_by_field_name("value");
                // La doc et l'export portent sur la déclaration entière quand elle n'a qu'un déclarateur.
                let porteur = if seul { cible } else { v };
                match valeur {
                    Some(f)
                        if matches!(
                            f.kind(),
                            "arrow_function" | "function_expression" | "function"
                        ) =>
                    {
                        ajouter(
                            ctx,
                            &nom,
                            "fn",
                            visibilite,
                            porteur,
                            f.child_by_field_name("body"),
                            portee,
                            false,
                        )
                    }
                    _ => {
                        ajouter(ctx, &nom, "const", visibilite, porteur, None, portee, true);
                        // `const api = { projet: () => lire(…) }` : les appels de la valeur sont ceux de la constante.
                        if let Some(val) = valeur
                            && let Some(dernier) = ctx.symboles.last_mut()
                        {
                            collecter_appels(val, src, &mut dernier.appels);
                        }
                    }
                }
            }
        }
        "internal_module" | "module" => {
            let Some(nom) = nom_de(d, src) else { return };
            let corps = d.child_by_field_name("body");
            ajouter(ctx, &nom, "mod", visibilite, cible, corps, portee, true);
            if let Some(b) = corps {
                let sous = Portee {
                    prefixe: qualifier(&portee.prefixe, &nom),
                    ..Default::default()
                };
                parcourir(b, &sous, ctx);
            }
        }
        _ => {}
    }
}

fn membres(corps: Node, portee: &Portee, ctx: &mut Ctx) {
    let src = ctx.src;
    let mut c = corps.walk();
    let enfants: Vec<Node> = corps.named_children(&mut c).collect();
    for m in enfants {
        let (genre, fonction) = match m.kind() {
            "method_definition" => ("methode", Some(m)),
            "abstract_method_signature" => ("methode", None),
            "public_field_definition" => match m.child_by_field_name("value") {
                Some(f) if matches!(f.kind(), "arrow_function" | "function_expression") => {
                    ("methode", Some(f))
                }
                _ => ("champ", None),
            },
            _ => continue,
        };
        let Some(nom_n) = m.child_by_field_name("name") else {
            continue;
        };
        let nom = txt(nom_n, src).to_string();
        let mut c2 = m.walk();
        let restreint = m.children(&mut c2).any(|e| {
            e.kind() == "accessibility_modifier" && matches!(txt(e, src), "private" | "protected")
        }) || nom.starts_with('#');
        let vis = if restreint {
            "prive"
        } else {
            portee.visibilite_classe.as_deref().unwrap_or("prive")
        };
        let corps_m = fonction.and_then(|f| f.child_by_field_name("body"));
        ajouter(ctx, &nom, genre, vis, m, corps_m, portee, genre == "champ");
        // Initialiseur d'un champ (`api = inject(Api)`, `total = computed(() => …)`) : ses appels
        // sont ceux du champ.
        if genre == "champ"
            && let Some(v) = m.child_by_field_name("value")
            && let Some(dernier) = ctx.symboles.last_mut()
        {
            collecter_appels(v, src, &mut dernier.appels);
        }
    }
}

pub(crate) fn collecter_appels(corps: Node, src: &[u8], sortie: &mut Vec<AppelBrut>) {
    let mut pile = vec![corps];
    while let Some(x) = pile.pop() {
        let ligne = x.start_position().row + 1;
        match x.kind() {
            "call_expression" => {
                if let Some(f) = x.child_by_field_name("function") {
                    match f.kind() {
                        "identifier" => {
                            sortie.push(AppelBrut {
                                forme: "direct".into(),
                                cible: txt(f, src).into(),
                                qualificatif: None,
                                ligne,
                            });
                            // Injection de dépendance Angular : `inject(Service)` dépend de `Service`.
                            if txt(f, src) == "inject"
                                && let Some(arg) = x
                                    .child_by_field_name("arguments")
                                    .and_then(|a| a.named_child(0))
                                && arg.kind() == "identifier"
                            {
                                sortie.push(AppelBrut {
                                    forme: "direct".into(),
                                    cible: txt(arg, src).into(),
                                    qualificatif: None,
                                    ligne,
                                });
                            }
                        }
                        "member_expression" => {
                            if let Some(p) = f.child_by_field_name("property") {
                                let objet = f.child_by_field_name("object");
                                let q = match objet.map(|o| o.kind()) {
                                    Some("this") => Some("self".to_string()),
                                    Some("identifier") => {
                                        objet.map(|o| format!("obj:{}", txt(o, src)))
                                    }
                                    _ => None,
                                };
                                sortie.push(AppelBrut {
                                    forme: "methode".into(),
                                    cible: txt(p, src).into(),
                                    qualificatif: q,
                                    ligne,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            "new_expression" => {
                if let Some(k) = x.child_by_field_name("constructor")
                    && k.kind() == "identifier"
                {
                    sortie.push(AppelBrut {
                        forme: "direct".into(),
                        cible: txt(k, src).into(),
                        qualificatif: None,
                        ligne,
                    });
                }
            }
            _ => {}
        }
        let mut c = x.walk();
        for e in x.children(&mut c) {
            pile.push(e);
        }
    }
    sortie.sort_by(|a, b| a.ligne.cmp(&b.ligne).then(a.cible.cmp(&b.cible)));
}

/// `./x` depuis `front/src/a.ts` → `front/src/x` (sans extension) ; un paquet reste tel quel.
pub fn resoudre_module(depuis: &str, specificateur: &str) -> String {
    if !specificateur.starts_with('.') {
        return specificateur.to_string();
    }
    let mut parties: Vec<&str> = depuis.split('/').collect();
    parties.pop();
    for s in specificateur.split('/') {
        match s {
            "." | "" => {}
            ".." => {
                parties.pop();
            }
            autre => parties.push(autre),
        }
    }
    let joint = parties.join("/");
    for ext in [".js", ".mjs", ".ts", ".tsx", ".mts", ".jsx"] {
        if let Some(sans) = joint.strip_suffix(ext) {
            return sans.to_string();
        }
    }
    joint
}

fn lire_import(n: Node, src: &[u8], chemin: &str, sortie: &mut Vec<Import>) {
    let Some(source) = n.child_by_field_name("source") else {
        return;
    };
    let spec = txt(source, src).trim_matches(['"', '\'', '`']);
    // `import Cellule from "./Cellule.svelte"` : le composant porte le nom de son fichier.
    if let Some(sans) = spec.strip_suffix(".svelte") {
        let module = resoudre_module(chemin, sans);
        let tige = module.rsplit('/').next().unwrap_or(&module).to_string();
        let mut c = n.walk();
        for clause in n.children(&mut c).filter(|e| e.kind() == "import_clause") {
            let mut c2 = clause.walk();
            if let Some(id) = clause
                .named_children(&mut c2)
                .find(|e| e.kind() == "identifier")
            {
                sortie.push(Import {
                    alias: txt(id, src).into(),
                    chemin: format!("{module}::{tige}"),
                });
            }
        }
        return;
    }
    let module = resoudre_module(chemin, spec);
    let mut c = n.walk();
    for clause in n.children(&mut c).filter(|e| e.kind() == "import_clause") {
        let mut c2 = clause.walk();
        for e in clause.named_children(&mut c2) {
            match e.kind() {
                "identifier" => sortie.push(Import {
                    alias: txt(e, src).into(),
                    chemin: format!("{module}::default"),
                }),
                "namespace_import" => {
                    let mut c3 = e.walk();
                    if let Some(id) = e.named_children(&mut c3).find(|x| x.kind() == "identifier") {
                        sortie.push(Import {
                            alias: txt(id, src).into(),
                            chemin: format!("{module}::*"),
                        });
                    }
                }
                "named_imports" => {
                    let mut c3 = e.walk();
                    for s in e
                        .named_children(&mut c3)
                        .filter(|x| x.kind() == "import_specifier")
                    {
                        let Some(nom) = s.child_by_field_name("name") else {
                            continue;
                        };
                        let alias = s.child_by_field_name("alias").unwrap_or(nom);
                        sortie.push(Import {
                            alias: txt(alias, src).into(),
                            chemin: format!("{module}::{}", txt(nom, src)),
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resoudre_module;

    #[test]
    fn modules() {
        assert_eq!(
            resoudre_module("front/src/pages/A.ts", "../lib/api"),
            "front/src/lib/api"
        );
        assert_eq!(
            resoudre_module("front/src/main.ts", "./x.js"),
            "front/src/x"
        );
        assert_eq!(resoudre_module("a.ts", "@angular/core"), "@angular/core");
    }
}

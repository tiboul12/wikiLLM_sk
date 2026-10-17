//! `check` : doc périmée, blocs manquants, références cassées, exigences non couvertes (SPEC-04).
//!
//! La doc d'un symbole est périmée si sa signature ou son corps (hash normalisés) diffèrent de
//! leur état au dernier commit qui a touché son bloc (ADR-0006). Hors git, repli sur un état local.
//!
//! @req EF-010, EF-011, EF-012, EF-013, EF-014, EF-015
//! @adr 6

use crate::git::{Depot, NON_COMMITE};
use crate::index::{DOSSIER, Index};
use crate::lang::{Langage, rust};
use anyhow::Result;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Un problème trouvé par `check`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Diagnostic {
    pub fichier: String,
    pub ligne: usize,
    pub code: String,
    pub symbole: Option<String>,
    pub severite: String,
    pub message: String,
}

/// Résultat complet, sérialisé tel quel par `--format json`.
#[derive(Debug, Clone, Serialize)]
pub struct RapportCheck {
    pub version: u32,
    /// `git` ou `local` (repli hors dépôt git, moins fiable).
    pub mode_reference: String,
    pub diagnostics: Vec<Diagnostic>,
    pub resume: Resume,
}

/// Nombre de diagnostics par sévérité.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Resume {
    pub erreurs: usize,
    pub avertissements: usize,
    pub infos: usize,
}

impl RapportCheck {
    /// Code de retour de la CLI : 1 s'il y a une erreur (ou un avertissement en mode strict).
    pub fn code_retour(&self, strict: bool) -> i32 {
        i32::from(self.resume.erreurs > 0 || (strict && self.resume.avertissements > 0))
    }
}

/// Sévérité par défaut de chaque code (SPEC-04) ; `wikik.toml` peut la changer.
pub fn severite_par_defaut(code: &str) -> &'static str {
    match code {
        "ref-adr" | "ref-req" | "lien-casse" | "frontmatter-invalide" | "id-duplique" => "erreur",
        "perime-signature"
        | "perime-corps"
        | "manquant"
        | "exigence-non-couverte"
        | "balise-inconnue"
        | "parse-partiel" => "avertissement",
        _ => "info",
    }
}

/// Mots sans valeur pour l'heuristique `paraphrase` (SPEC-01).
const MOTS_VIDES: &[&str] = &[
    "le", "la", "les", "l", "un", "une", "des", "de", "du", "d", "à", "a", "au", "aux", "et", "ou",
    "en", "pour", "par", "sur", "dans", "avec", "renvoie", "retourne", "donne", "the", "an", "of",
    "to", "for", "and", "or", "in", "on", "with", "returns", "return", "gets", "get", "sets",
    "set", "is",
];

struct Collecteur<'a> {
    severites: &'a BTreeMap<String, String>,
    diagnostics: Vec<Diagnostic>,
}

impl Collecteur<'_> {
    fn ajouter(
        &mut self,
        code: &str,
        fichier: &str,
        ligne: usize,
        symbole: Option<&str>,
        message: String,
    ) {
        let severite = self
            .severites
            .get(code)
            .map(String::as_str)
            .unwrap_or_else(|| severite_par_defaut(code));
        if severite == "ignore" {
            return;
        }
        self.diagnostics.push(Diagnostic {
            fichier: fichier.to_string(),
            ligne,
            code: code.to_string(),
            symbole: symbole.map(str::to_string),
            severite: severite.to_string(),
            message,
        });
    }
}

struct SymDoc {
    id: String,
    fichier: String,
    nom: String,
    nom_qualifie: String,
    genre: String,
    visibilite: String,
    ligne_debut: usize,
    doc_ligne_debut: Option<usize>,
    doc_ligne_fin: Option<usize>,
    signature: String,
    resume: Option<String>,
    doc: Option<String>,
    hash_corps: String,
    hash_signature: String,
    hash_doc: Option<String>,
    est_test: bool,
}

/// Lance toutes les vérifications sur un index à jour.
/// @relu 2026-10-17 a91daaee
pub fn verifier(index: &Index) -> Result<RapportCheck> {
    let c = &index.conn;
    let mut col = Collecteur {
        severites: &index.config.check.severites,
        diagnostics: vec![],
    };

    let symboles = charger_symboles(c, None)?;

    // Blocs manquants (SPEC-01 R5) et heuristiques de rédaction.
    let seuil = index.config.check.seuil_visibilite.as_str();
    for s in &symboles {
        if s.est_test {
            continue;
        }
        let exige = match seuil {
            "tout" => true,
            "pub-crate" => s.visibilite != "prive",
            _ => s.visibilite == "pub",
        };
        // Les méthodes d'un impl de trait héritent de la doc du trait ; un module en ligne ou un champ
        // de classe (TypeScript) n'est pas exigé.
        let exempte = s.nom_qualifie.contains('<') || matches!(s.genre.as_str(), "mod" | "champ");
        match &s.doc {
            None if exige && !exempte => col.ajouter(
                "manquant",
                &s.fichier,
                s.ligne_debut,
                Some(&s.id),
                format!("{} `{}` sans bloc de doc", s.genre, s.nom_qualifie),
            ),
            Some(doc) => {
                let ligne = s.doc_ligne_debut.unwrap_or(s.ligne_debut);
                if doc.lines().count() > 25 {
                    col.ajouter(
                        "bloc-long",
                        &s.fichier,
                        ligne,
                        Some(&s.id),
                        "bloc de plus de 25 lignes".into(),
                    );
                }
                if let Some(r) = &s.resume
                    && paraphrase(r, &s.nom, &s.signature)
                {
                    col.ajouter(
                        "paraphrase",
                        &s.fichier,
                        ligne,
                        Some(&s.id),
                        format!("le résumé « {r} » ne dit rien de plus que la signature"),
                    );
                }
            }
            None => {}
        }
    }

    // Fichiers : en-tête et erreurs de parse.
    {
        let mut st = c.prepare(
            "SELECT chemin, entete IS NULL, erreurs_parse FROM fichiers ORDER BY chemin",
        )?;
        let lignes: Vec<(String, bool, i64)> = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (chemin, sans_entete, erreurs) in lignes {
            if sans_entete {
                col.ajouter(
                    "manquant-entete",
                    &chemin,
                    1,
                    None,
                    "fichier sans bloc d'en-tête `//!`".into(),
                );
            }
            if erreurs > 0 {
                col.ajouter(
                    "parse-partiel",
                    &chemin,
                    1,
                    None,
                    format!("{erreurs} erreur(s) de syntaxe : fichier indexé partiellement"),
                );
            }
        }
    }

    // Balises inconnues.
    {
        let mut st = c.prepare(
            "SELECT b.symbole, s.fichier, b.nom, b.ligne FROM balises b JOIN symboles s ON s.id = b.symbole
             UNION ALL SELECT fichier, fichier, nom, ligne FROM balises_fichier ORDER BY 1, 4",
        )?;
        let lignes: Vec<(String, String, String, i64)> = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (sym, fichier, nom, ligne) in lignes {
            if !rust::BALISES_CONNUES.contains(&nom.as_str())
                && !crate::lang::typescript::BALISES_JSDOC.contains(&nom.as_str())
            {
                col.ajouter(
                    "balise-inconnue",
                    &fichier,
                    ligne as usize,
                    (sym != fichier).then_some(sym.as_str()),
                    format!(
                        "balise `@{nom}` inconnue (attendu : {})",
                        rust::BALISES_CONNUES.join(", ")
                    ),
                );
            }
        }
    }

    // Références @adr / @req vers des documents absents.
    let ids_docs: HashSet<String> = {
        let mut st = c.prepare("SELECT id FROM documents")?;
        st.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    {
        let mut st = c.prepare(
            "SELECT a.source, coalesce(s.fichier, f.chemin), a.cible, a.genre, a.ligne FROM aretes a
             LEFT JOIN symboles s ON s.id = a.source LEFT JOIN fichiers f ON f.chemin = a.source
             WHERE a.genre IN ('adr', 'req') AND (s.id IS NOT NULL OR f.chemin IS NOT NULL) ORDER BY 1, 3",
        )?;
        let lignes: Vec<(String, String, String, String, Option<i64>)> = st
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        for (sym, fichier, cible, genre, ligne) in lignes {
            if !ids_docs.contains(&cible) {
                let quoi = if genre == "adr" { "ADR" } else { "exigence" };
                col.ajouter(
                    &format!("ref-{genre}"),
                    &fichier,
                    ligne.unwrap_or(0) as usize,
                    (sym != fichier).then_some(sym.as_str()),
                    format!("`@{genre}` vers {quoi} inexistant(e) : {cible}"),
                );
            }
        }
    }

    // Documents : liens cassés, frontmatter invalide, doublons, exigences non couvertes.
    {
        let mut st = c.prepare(
            "SELECT d.chemin, l.cible_brute, l.ligne FROM liens_doc l JOIN documents d ON d.id = l.source
             WHERE l.cible IS NULL ORDER BY 1, 3, 2",
        )?;
        let lignes: Vec<(String, String, i64)> = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (chemin, brute, ligne) in lignes {
            col.ajouter(
                "lien-casse",
                &chemin,
                ligne as usize,
                None,
                format!("lien vers « {brute} » introuvable"),
            );
        }
        let mut st =
            c.prepare("SELECT id, chemin, frontmatter_invalide FROM documents ORDER BY chemin")?;
        let docs: Vec<(String, String, Option<String>)> = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (id, chemin, invalide) in docs {
            if let Some(e) = invalide {
                col.ajouter(
                    "frontmatter-invalide",
                    &chemin,
                    1,
                    None,
                    format!("frontmatter illisible : {e}"),
                );
            }
            if let Some((original, _)) = id.split_once('@') {
                col.ajouter(
                    "id-duplique",
                    &chemin,
                    1,
                    None,
                    format!("identifiant {original} déjà utilisé par un autre document"),
                );
            }
        }
        let mut st = c.prepare(
            "SELECT d.id, d.chemin FROM documents d
             WHERE d.genre IN ('exigence', 'exigence-nf') AND d.priorite = 'must'
               AND d.statut IN ('validé', 'implémenté')
               AND NOT EXISTS (SELECT 1 FROM aretes a WHERE a.cible = d.id AND a.genre = 'req')
             ORDER BY d.id",
        )?;
        let lignes: Vec<(String, String)> = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        for (id, chemin) in lignes {
            col.ajouter(
                "exigence-non-couverte",
                &chemin,
                1,
                None,
                format!("{id} (must) n'est citée par aucun `@req`"),
            );
        }
    }

    // Doc périmée.
    let mode = match Depot::trouver(&index.racine) {
        Some(depot) => {
            perimes_git(index, &depot, &symboles, &mut col)?;
            "git"
        }
        None => {
            perimes_local(index, &symboles, &mut col)?;
            "local"
        }
    };

    col.diagnostics.sort();
    let mut resume = Resume::default();
    for d in &col.diagnostics {
        match d.severite.as_str() {
            "erreur" => resume.erreurs += 1,
            "avertissement" => resume.avertissements += 1,
            _ => resume.infos += 1,
        }
    }
    Ok(RapportCheck {
        version: 1,
        mode_reference: mode.into(),
        diagnostics: col.diagnostics,
        resume,
    })
}

fn charger_symboles(c: &rusqlite::Connection, fichiers: Option<&[String]>) -> Result<Vec<SymDoc>> {
    // Filtre passé en une seule chaîne `\nf1\nf2\n` pour rester dans une requête préparée.
    let filtre: Option<String> = fichiers.map(|f| format!("\n{}\n", f.join("\n")));
    let mut st = c.prepare(
            "SELECT id, fichier, nom, nom_qualifie, genre, visibilite, ligne_debut, doc_ligne_debut, doc_ligne_fin,
                    signature, resume, doc, hash_corps, hash_signature, hash_doc, est_test FROM symboles
             WHERE ?1 IS NULL OR instr(?1, char(10) || fichier || char(10)) > 0 ORDER BY id",
        )?;
    let v = st
        .query_map([filtre], |r| {
            Ok(SymDoc {
                id: r.get(0)?,
                fichier: r.get(1)?,
                nom: r.get(2)?,
                nom_qualifie: r.get(3)?,
                genre: r.get(4)?,
                visibilite: r.get(5)?,
                ligne_debut: r.get::<_, i64>(6)? as usize,
                doc_ligne_debut: r.get::<_, Option<i64>>(7)?.map(|v| v as usize),
                doc_ligne_fin: r.get::<_, Option<i64>>(8)?.map(|v| v as usize),
                signature: r.get(9)?,
                resume: r.get(10)?,
                doc: r.get(11)?,
                hash_corps: r.get(12)?,
                hash_signature: r.get(13)?,
                hash_doc: r.get(14)?,
                est_test: r.get::<_, i64>(15)? != 0,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Statut de doc des symboles des fichiers donnés : `à jour`, `périmé` ou `sans doc` (SPEC-07).
pub fn statuts(index: &Index, fichiers: &[String]) -> Result<HashMap<String, String>> {
    let symboles = charger_symboles(&index.conn, Some(fichiers))?;
    let vide = BTreeMap::new();
    let mut col = Collecteur {
        severites: &vide,
        diagnostics: vec![],
    };
    match Depot::trouver(&index.racine) {
        Some(depot) => perimes_git(index, &depot, &symboles, &mut col)?,
        None => perimes_local(index, &symboles, &mut col)?,
    }
    let perimes: HashSet<String> = col
        .diagnostics
        .into_iter()
        .filter(|d| d.code == "perime-corps" || d.code == "perime-signature")
        .filter_map(|d| d.symbole)
        .collect();
    Ok(symboles
        .into_iter()
        .map(|s| {
            let statut = if s.doc.is_none() {
                "sans doc"
            } else if perimes.contains(&s.id) {
                "périmé"
            } else {
                "à jour"
            };
            (s.id, statut.to_string())
        })
        .collect())
}

/// Le résumé n'utilise que des mots du nom et de la signature (SPEC-01, indice et non règle).
fn paraphrase(resume: &str, nom: &str, signature: &str) -> bool {
    let mots = |t: &str| -> Vec<String> {
        let mut v = vec![];
        for brut in t.split(|c: char| !c.is_alphanumeric()) {
            let mut courant = String::new();
            for ch in brut.chars() {
                if ch.is_uppercase() && !courant.is_empty() {
                    v.push(std::mem::take(&mut courant));
                }
                courant.extend(ch.to_lowercase());
            }
            if !courant.is_empty() {
                v.push(courant);
            }
        }
        v
    };
    let identifiants: HashSet<String> = mots(nom).into_iter().chain(mots(signature)).collect();
    let significatifs: Vec<String> = mots(resume)
        .into_iter()
        .filter(|m| m.chars().count() > 1 && !MOTS_VIDES.contains(&m.as_str()))
        .collect();
    !significatifs.is_empty() && significatifs.iter().all(|m| identifiants.contains(m))
}

/// Symboles dont la dernière balise `@relu` porte le jeton du code actuel : relus, donc frais.
fn relus_a_jour(index: &Index, symboles: &[SymDoc]) -> Result<HashSet<String>> {
    let mut st = index
        .conn
        .prepare("SELECT symbole, valeur FROM balises WHERE nom = 'relu'")?;
    let jetons: Vec<(String, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let par_id: HashMap<&str, &SymDoc> = symboles.iter().map(|s| (s.id.as_str(), s)).collect();
    Ok(jetons
        .into_iter()
        .filter(|(id, valeur)| {
            par_id.get(id.as_str()).is_some_and(|s| {
                valeur.split_whitespace().nth(1)
                    == Some(crate::relu::jeton(&s.hash_signature, &s.hash_corps).as_str())
            })
        })
        .map(|(id, _)| id)
        .collect())
}

fn perimes_git(
    index: &Index,
    depot: &Depot,
    symboles: &[SymDoc],
    col: &mut Collecteur,
) -> Result<()> {
    let relus = relus_a_jour(index, symboles)?;
    let langages = crate::lang::actives(&index.config.index.langages);
    let mut par_fichier: BTreeMap<&str, Vec<&SymDoc>> = BTreeMap::new();
    for s in symboles
        .iter()
        .filter(|s| s.doc_ligne_debut.is_some() && !s.est_test)
    {
        par_fichier.entry(s.fichier.as_str()).or_default().push(s);
    }
    let cache_dir = index.racine.join(DOSSIER).join("cache-git");
    let mut anciens: HashMap<(String, String), HashMap<String, (String, String)>> = HashMap::new();
    for (fichier, syms) in par_fichier {
        // Fichier non suivi : toutes ses lignes sont « non commitées », donc fraîches.
        let Some(blame) = depot.blame(fichier) else {
            continue;
        };
        for s in syms.into_iter().filter(|s| !relus.contains(&s.id)) {
            let (Some(d), Some(f)) = (s.doc_ligne_debut, s.doc_ligne_fin) else {
                continue;
            };
            let lignes: Vec<_> = (d..=f).filter_map(|n| blame.get(n - 1)).collect();
            if lignes.is_empty() || lignes.iter().any(|l| l.commit == NON_COMMITE) {
                continue;
            }
            let reference = lignes
                .iter()
                .max_by(|a, b| a.date.cmp(&b.date).then(a.commit.cmp(&b.commit)))
                .unwrap();
            let cle = (reference.commit.clone(), reference.chemin.clone());
            if !anciens.contains_key(&cle) {
                // L'état passé se relit avec l'extracteur du langage du fichier.
                let ext = fichier.rsplit('.').next().unwrap_or("");
                let Some(langage) = langages.iter().find(|l| l.extensions().contains(&ext)) else {
                    continue;
                };
                let table =
                    charger_ancien(depot, langage.as_ref(), fichier, &cle.0, &cle.1, &cache_dir);
                anciens.insert(cle.clone(), table);
            }
            let court = &reference.commit[..7];
            match anciens[&cle].get(&s.id) {
                None => col.ajouter(
                    "perime-inconnu",
                    &s.fichier,
                    s.ligne_debut,
                    Some(&s.id),
                    format!("symbole introuvable au commit {court} où son bloc a été écrit (déplacé ou renommé ?)"),
                ),
                Some((_, sig)) if *sig != s.hash_signature => col.ajouter(
                    "perime-signature",
                    &s.fichier,
                    s.ligne_debut,
                    Some(&s.id),
                    format!("la signature a changé depuis la dernière modification du bloc (commit {court})"),
                ),
                Some((corps, _)) if *corps != s.hash_corps => col.ajouter(
                    "perime-corps",
                    &s.fichier,
                    s.ligne_debut,
                    Some(&s.id),
                    format!("le corps a changé depuis la dernière modification du bloc (commit {court})"),
                ),
                _ => {}
            }
        }
    }
    Ok(())
}

/// Hash (corps, signature) des symboles d'un fichier à un commit, relus avec l'extracteur de son langage ;
/// cache disque par (version des règles, langage, commit, chemin).
fn charger_ancien(
    depot: &Depot,
    langage: &dyn Langage,
    fichier_projet: &str,
    commit: &str,
    chemin_depot: &str,
    cache_dir: &std::path::Path,
) -> HashMap<String, (String, String)> {
    // Clé : version des règles, langage, commit, chemins ; un changement de l'un invalide l'entrée.
    let nom_cache = format!(
        "v{}-{}-{commit}-{}.json",
        crate::VERSION_EXTRACTION,
        langage.nom(),
        crate::hash::empreinte(format!("{chemin_depot}\0{fichier_projet}").as_bytes())
    );
    let chemin_cache = cache_dir.join(nom_cache);
    if let Ok(t) = std::fs::read_to_string(&chemin_cache)
        && let Ok(m) = serde_json::from_str(&t)
    {
        return m;
    }
    let Some(octets) = depot.contenu(commit, chemin_depot) else {
        return HashMap::new();
    };
    // Même chemin que l'actuel : les identifiants se comparent directement, même après renommage du fichier.
    let extrait = langage.extraire(fichier_projet, &octets);
    let table: HashMap<String, (String, String)> = extrait
        .symboles
        .into_iter()
        .map(|s| (s.id, (s.hash_corps, s.hash_signature)))
        .collect();
    let _ = std::fs::create_dir_all(cache_dir);
    if let Ok(t) = serde_json::to_string(&table) {
        let _ = std::fs::write(&chemin_cache, t);
    }
    table
}

/// Repli hors git : référence = état vu quand le bloc est apparu ou a changé, gardé dans `.wikik/`.
/// Comme en mode git, un bloc dont le jeton `@relu` correspond au code actuel est frais.
fn perimes_local(index: &Index, symboles: &[SymDoc], col: &mut Collecteur) -> Result<()> {
    let chemin = index.racine.join(DOSSIER).join("reference.json");
    let mut reference: BTreeMap<String, (String, String, String)> =
        std::fs::read_to_string(&chemin)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
    let relus = relus_a_jour(index, symboles)?;
    for s in symboles
        .iter()
        .filter(|s| !s.est_test && !relus.contains(&s.id))
    {
        let Some(hd) = &s.hash_doc else { continue };
        match reference.get(&s.id) {
            Some((doc, corps, sig)) if doc == hd => {
                if *sig != s.hash_signature {
                    col.ajouter(
                        "perime-signature",
                        &s.fichier,
                        s.ligne_debut,
                        Some(&s.id),
                        "la signature a changé depuis que le bloc a été vu (référence locale)"
                            .into(),
                    );
                } else if *corps != s.hash_corps {
                    col.ajouter(
                        "perime-corps",
                        &s.fichier,
                        s.ligne_debut,
                        Some(&s.id),
                        "le corps a changé depuis que le bloc a été vu (référence locale)".into(),
                    );
                }
            }
            _ => {
                reference.insert(
                    s.id.clone(),
                    (hd.clone(), s.hash_corps.clone(), s.hash_signature.clone()),
                );
            }
        }
    }
    std::fs::write(&chemin, serde_json::to_string_pretty(&reference)?)?;
    Ok(())
}

/// Rapport lisible, groupé par fichier.
pub fn formater_texte(r: &RapportCheck) -> String {
    let mut s = String::new();
    if r.mode_reference == "local" {
        s.push_str("note : hors dépôt git, la détection de doc périmée utilise une référence locale (.wikik/reference.json)\n\n");
    }
    let mut courant = "";
    for d in &r.diagnostics {
        if d.fichier != courant {
            if !courant.is_empty() {
                s.push('\n');
            }
            s.push_str(&d.fichier);
            s.push('\n');
            courant = &d.fichier;
        }
        let sym = d
            .symbole
            .as_deref()
            .map(|x| format!(" [{}]", x.rsplit_once("::").map_or(x, |p| p.1)))
            .unwrap_or_default();
        s.push_str(&format!(
            "  {:>4}  {:<13} {:<22} {}{}\n",
            d.ligne, d.severite, d.code, d.message, sym
        ));
    }
    if !r.diagnostics.is_empty() {
        s.push('\n');
    }
    s.push_str(&format!(
        "{} erreur(s), {} avertissement(s), {} info(s)\n",
        r.resume.erreurs, r.resume.avertissements, r.resume.infos
    ));
    s
}

#[cfg(test)]
mod tests {
    use super::paraphrase;

    #[test]
    fn heuristique_paraphrase() {
        assert!(paraphrase(
            "Returns the user name.",
            "user_name",
            "pub fn user_name(&self) -> String"
        ));
        assert!(!paraphrase(
            "Distance à l'origine.",
            "norme",
            "pub fn norme(&self) -> f64"
        ));
    }
}

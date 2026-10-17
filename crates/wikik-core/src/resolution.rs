//! Résolution globale des arêtes : appels, composants rendus, balises et liens de documents.
//!
//! Refaite entièrement à chaque mise à jour qui change un fichier (SPEC-03 R3) : la cible d'un
//! appel peut dépendre d'un autre fichier que celui qui a changé. Sans analyse de types, chaque
//! arête dit honnêtement si elle est `certain` ou `probable`.
//!
//! @req EF-001, EF-006, EF-007
//! @pitfall Les appels de méthode sur un receveur autre que `self` ne sont résolus que si le nom est unique dans le projet.

use crate::projet::normaliser_adr;
use anyhow::Result;
use rusqlite::{Transaction, params};
use std::collections::{BTreeMap, HashMap};

/// Noms de méthodes trop courants dans la bibliothèque standard pour être devinés (SPEC-03).
const METHODES_COURANTES: &[&str] = &[
    "as_ref",
    "as_str",
    "borrow",
    "clone",
    "collect",
    "contains",
    "default",
    "drop",
    "eq",
    "extend",
    "filter",
    "find",
    "fmt",
    "from",
    "get",
    "get_mut",
    "insert",
    "into",
    "into_iter",
    "is_empty",
    "is_some",
    "is_none",
    "iter",
    "iter_mut",
    "join",
    "len",
    "lock",
    "map",
    "new",
    "next",
    "ok",
    "parse",
    "pop",
    "push",
    "read",
    "remove",
    "send",
    "set",
    "split",
    "to_owned",
    "to_string",
    "trim",
    "unwrap",
    "unwrap_or",
    "update",
    "with",
    "write",
];

struct Sym {
    id: String,
    fichier: String,
    nom: String,
    genre: String,
    type_parent: Option<String>,
    segments: Vec<String>,
}

/// Arêtes à écrire, dédoublonnées : (source, cible, genre) → (confiance, ligne).
type Aretes = BTreeMap<(String, String, String), (String, Option<i64>)>;

fn ajouter(
    aretes: &mut Aretes,
    source: &str,
    cible: &str,
    genre: &str,
    confiance: &str,
    ligne: Option<i64>,
) {
    let cle = (source.to_string(), cible.to_string(), genre.to_string());
    match aretes.get_mut(&cle) {
        Some((c, l)) => {
            if confiance == "certain" {
                *c = "certain".into();
            }
            *l = match (*l, ligne) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
        }
        None => {
            aretes.insert(cle, (confiance.to_string(), ligne));
        }
    }
}

/// Chemin de module déduit du chemin de fichier : `src/a/b.rs` → `[a, b]`, `src/lib.rs` → `[]`.
pub fn module_du_fichier(chemin: &str) -> Vec<String> {
    let parties: Vec<&str> = chemin.split('/').collect();
    let debut = parties
        .iter()
        .rposition(|p| *p == "src")
        .map_or(0, |i| i + 1);
    let mut segs: Vec<String> = parties[debut..].iter().map(|s| s.to_string()).collect();
    if let Some(dernier) = segs.pop() {
        let tige = dernier.strip_suffix(".rs").unwrap_or(&dernier).to_string();
        if !matches!(tige.as_str(), "lib" | "main" | "mod") {
            segs.push(tige);
        }
    }
    segs
}

fn segments_chemin(chemin: &str) -> Vec<String> {
    chemin
        .split("::")
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn se_termine_par(segs: &[String], suffixe: &[String]) -> bool {
    suffixe.len() <= segs.len() && segs[segs.len() - suffixe.len()..] == *suffixe
}

/// Recalcule toutes les arêtes et la cible des liens de documents.
pub fn recalculer(tx: &Transaction) -> Result<()> {
    let symboles: Vec<Sym> = {
        let mut st = tx.prepare(
            "SELECT id, fichier, nom, nom_qualifie, genre, type_parent FROM symboles ORDER BY id",
        )?;
        st.query_map([], |r| {
            let fichier: String = r.get(1)?;
            let nq: String = r.get(3)?;
            let mut segments = module_du_fichier(&fichier);
            segments.extend(segments_chemin(&nq));
            Ok(Sym {
                id: r.get(0)?,
                fichier,
                nom: r.get(2)?,
                genre: r.get(4)?,
                type_parent: r.get(5)?,
                segments,
            })
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    let par_id: HashMap<&str, &Sym> = symboles.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut par_nom: HashMap<&str, Vec<&Sym>> = HashMap::new();
    for s in &symboles {
        par_nom.entry(s.nom.as_str()).or_default().push(s);
    }
    let imports: HashMap<(String, String), String> = {
        let mut st = tx.prepare("SELECT fichier, alias, chemin_import FROM imports")?;
        st.query_map([], |r| Ok(((r.get(0)?, r.get(1)?), r.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?
    };

    let mut aretes: Aretes = BTreeMap::new();

    // Appels et composants rendus.
    let appels: Vec<(String, String, String, Option<String>, i64)> = {
        let mut st = tx.prepare("SELECT symbole, forme, cible, qualificatif, ligne FROM appels_bruts ORDER BY symbole, ligne, cible")?;
        st.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };
    for (source, forme, cible, qualif, ligne) in &appels {
        let Some(src) = par_id.get(source.as_str()) else {
            continue;
        };
        let genre = if forme == "rend" { "rend" } else { "appel" };
        for (id, confiance) in resoudre(src, forme, cible, qualif.as_deref(), &par_nom, &imports) {
            ajouter(&mut aretes, source, &id, genre, confiance, Some(*ligne));
        }
    }

    // Balises.
    let balises: Vec<(String, String, String, i64)> = {
        let mut st =
            tx.prepare("SELECT symbole, nom, valeur, ligne FROM balises ORDER BY symbole, ligne")?;
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<rusqlite::Result<_>>()?
    };
    for (source, nom, valeur, ligne) in &balises {
        let valeurs = valeur
            .split([',', ' '])
            .map(str::trim)
            .filter(|v| !v.is_empty());
        match nom.as_str() {
            "adr" => {
                for v in valeurs {
                    let cible = normaliser_adr(v).unwrap_or_else(|| v.to_string());
                    ajouter(&mut aretes, source, &cible, "adr", "certain", Some(*ligne));
                }
            }
            "req" => {
                for v in valeurs {
                    ajouter(
                        &mut aretes,
                        source,
                        &v.to_ascii_uppercase(),
                        "req",
                        "certain",
                        Some(*ligne),
                    );
                }
            }
            "topic" => {
                for v in valeurs {
                    ajouter(
                        &mut aretes,
                        source,
                        &format!("topic:{}", v.to_lowercase()),
                        "topic",
                        "certain",
                        Some(*ligne),
                    );
                }
            }
            _ => {}
        }
    }

    // Liens de documents.
    let docs: Vec<(String, String)> = {
        let mut st = tx.prepare("SELECT id, chemin FROM documents ORDER BY id")?;
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?
    };
    let fichiers: Vec<String> = {
        let mut st = tx.prepare("SELECT chemin FROM fichiers")?;
        st.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let liens: Vec<(i64, String, String, i64)> = {
        let mut st = tx.prepare("SELECT rowid, source, cible_brute, ligne FROM liens_doc")?;
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<rusqlite::Result<_>>()?
    };
    for (rowid, source, brute, ligne) in &liens {
        let cible = resoudre_lien(brute, &docs, &par_id, &fichiers);
        tx.execute(
            "UPDATE liens_doc SET cible = ?1 WHERE rowid = ?2",
            params![cible, rowid],
        )?;
        if let Some(c) = cible {
            ajouter(&mut aretes, source, &c, "lien", "certain", Some(*ligne));
        }
    }

    tx.execute("DELETE FROM aretes", [])?;
    let mut ins = tx.prepare_cached("INSERT INTO aretes VALUES (?1, ?2, ?3, ?4, ?5)")?;
    for ((s, c, g), (conf, l)) in &aretes {
        ins.execute(params![s, c, g, conf, l])?;
    }
    Ok(())
}

fn resoudre(
    src: &Sym,
    forme: &str,
    cible: &str,
    qualif: Option<&str>,
    par_nom: &HashMap<&str, Vec<&Sym>>,
    imports: &HashMap<(String, String), String>,
) -> Vec<(String, &'static str)> {
    let vide = vec![];
    let fonctions = |nom: &str| -> Vec<&Sym> {
        par_nom
            .get(nom)
            .unwrap_or(&vide)
            .iter()
            .filter(|s| matches!(s.genre.as_str(), "fn" | "composant"))
            .copied()
            .collect()
    };
    match forme {
        "direct" => {
            // Fonctions visibles depuis la portée de l'appelant dans le même fichier ; plusieurs
            // résultats = variantes `cfg` d'une même fonction, toutes appelées.
            let locales: Vec<&Sym> = fonctions(cible)
                .into_iter()
                .filter(|s| {
                    s.fichier == src.fichier
                        && s.type_parent.is_none()
                        && src
                            .segments
                            .starts_with(&s.segments[..s.segments.len() - 1])
                })
                .collect();
            if !locales.is_empty() {
                return locales.iter().map(|s| (s.id.clone(), "certain")).collect();
            }
            if let Some(chemin) = imports.get(&(src.fichier.clone(), cible.to_string())) {
                let r = par_suffixe(&normaliser_chemin(chemin, src, imports), par_nom);
                if r.len() == 1 {
                    return vec![(r[0].id.clone(), "certain")];
                }
            }
            let globales = fonctions(cible);
            if globales.len() == 1 {
                return vec![(globales[0].id.clone(), "probable")];
            }
            vec![]
        }
        "chemin" => {
            let segs = normaliser_chemin(cible, src, imports);
            if segs.len() == 2 && src.type_parent.as_deref() == Some(segs[0].as_str()) {
                let meme_fichier: Vec<&Sym> = par_nom
                    .get(segs[1].as_str())
                    .unwrap_or(&vide)
                    .iter()
                    .filter(|s| {
                        s.fichier == src.fichier
                            && s.type_parent.as_deref() == Some(segs[0].as_str())
                    })
                    .copied()
                    .collect();
                if let Some(s) = meme_fichier.first() {
                    return vec![(s.id.clone(), "certain")];
                }
            }
            let r = par_suffixe(&segs, par_nom);
            match r.len() {
                1 => vec![(r[0].id.clone(), "certain")],
                2..=3 => r.iter().map(|s| (s.id.clone(), "probable")).collect(),
                _ => vec![],
            }
        }
        "methode" => {
            if qualif == Some("self")
                && let Some(t) = &src.type_parent
            {
                let r: Vec<&Sym> = par_nom
                    .get(cible)
                    .unwrap_or(&vide)
                    .iter()
                    .filter(|s| {
                        s.type_parent.as_deref() == Some(t.as_str()) && s.genre == "methode"
                    })
                    .copied()
                    .collect();
                if let Some(s) = r.iter().find(|s| s.fichier == src.fichier).or(r.first()) {
                    return vec![(s.id.clone(), "certain")];
                }
            }
            if METHODES_COURANTES.contains(&cible) {
                return vec![];
            }
            let r: Vec<&Sym> = par_nom
                .get(cible)
                .unwrap_or(&vide)
                .iter()
                .filter(|s| s.genre == "methode")
                .copied()
                .collect();
            if r.len() == 1 {
                vec![(r[0].id.clone(), "probable")]
            } else {
                vec![]
            }
        }
        "rend" => {
            let r = fonctions(cible);
            if r.len() == 1 {
                vec![(r[0].id.clone(), "probable")]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// `crate::a::f` → `[a, f]` ; `Self::f` → `[Type, f]` ; premier segment importé → chemin de l'import.
fn normaliser_chemin(
    chemin: &str,
    src: &Sym,
    imports: &HashMap<(String, String), String>,
) -> Vec<String> {
    let mut segs = segments_chemin(chemin);
    if segs.first().is_some_and(|s| s == "Self")
        && let Some(t) = &src.type_parent
    {
        segs[0] = t.clone();
    }
    if segs.len() > 1
        && let Some(imp) = imports.get(&(src.fichier.clone(), segs[0].clone()))
    {
        let mut nouveaux = segments_chemin(imp);
        nouveaux.extend(segs.drain(1..));
        segs = nouveaux;
    }
    while segs
        .first()
        .is_some_and(|s| matches!(s.as_str(), "crate" | "self" | "super"))
    {
        segs.remove(0);
    }
    segs
}

fn par_suffixe<'a>(segs: &[String], par_nom: &HashMap<&str, Vec<&'a Sym>>) -> Vec<&'a Sym> {
    let Some(dernier) = segs.last() else {
        return vec![];
    };
    par_nom.get(dernier.as_str()).map_or_else(Vec::new, |v| {
        v.iter()
            .filter(|s| se_termine_par(&s.segments, segs))
            .copied()
            .collect()
    })
}

fn resoudre_lien(
    brute: &str,
    docs: &[(String, String)],
    par_id: &HashMap<&str, &Sym>,
    fichiers: &[String],
) -> Option<String> {
    let b = brute.trim();
    if let Some((id, _)) = docs.iter().find(|(id, _)| id == b) {
        return Some(id.clone());
    }
    if let Some((id, _)) = docs.iter().find(|(id, _)| id.eq_ignore_ascii_case(b)) {
        return Some(id.clone());
    }
    if let Some(adr) = normaliser_adr(b)
        && let Some((id, _)) = docs.iter().find(|(id, _)| *id == adr)
    {
        return Some(id.clone());
    }
    let avec_md = if b.ends_with(".md") {
        b.to_string()
    } else {
        format!("{b}.md")
    };
    let suffixe = format!("/{avec_md}");
    let r: Vec<&(String, String)> = docs
        .iter()
        .filter(|(_, c)| *c == avec_md || c.ends_with(&suffixe))
        .collect();
    if r.len() == 1 {
        return Some(r[0].0.clone());
    }
    if par_id.contains_key(b) || fichiers.iter().any(|f| f == b) {
        return Some(b.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modules() {
        assert_eq!(module_du_fichier("src/lib.rs"), Vec::<String>::new());
        assert_eq!(module_du_fichier("src-tauri/src/photo.rs"), vec!["photo"]);
        assert_eq!(module_du_fichier("src/a/mod.rs"), vec!["a"]);
        assert_eq!(module_du_fichier("src/a/b.rs"), vec!["a", "b"]);
        assert_eq!(module_du_fichier("build.rs"), vec!["build"]);
    }
}

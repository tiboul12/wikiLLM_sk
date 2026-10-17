//! Requêtes pour l'agent : réponses compactes et ciblées, sans code par défaut (SPEC-07).
//!
//! Chaque requête renvoie une [`Sortie`] : un texte pensé pour un prompt (première ligne `#` avec
//! le coût estimé en tokens) et la même information en JSON.
//!
//! @req EF-020, EF-021, EF-022, EF-023, EF-024, EF-025, EF-026, EF-027, EF-028, ENF-001

use crate::check;
use crate::index::{Index, requete_json};
use crate::lang::rust::resume;
use crate::projet::normaliser_adr;
use anyhow::{Result, bail};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

fn filtre<'a>(v: &'a [(String, Voisin)], g: &str) -> Vec<&'a Voisin> {
    v.iter().filter(|(x, _)| x == g).map(|(_, y)| y).collect()
}

/// Résultat d'une requête.
#[derive(Debug, Clone)]
pub struct Sortie {
    pub texte: String,
    pub json: Value,
    /// Vrai si la demande ne désignait pas un seul élément (la CLI renvoie alors le code 1).
    pub ambigu: bool,
}

/// Coût estimé d'un texte : octets / 4, l'ordre de grandeur des tokenizers courants.
pub fn tokens_estimes(texte: &str) -> usize {
    texte.len().div_ceil(4)
}

/// Ajoute la ligne d'en-tête `# titre · ~N tokens` (R3).
fn finaliser(titre: &str, corps: String, json: Value) -> Sortie {
    let provisoire = format!("# {titre} · ~0000 tokens\n{corps}");
    let texte = format!(
        "# {titre} · ~{} tokens\n{corps}",
        tokens_estimes(&provisoire)
    );
    Sortie {
        texte,
        json,
        ambigu: false,
    }
}

fn une_ligne(t: &str) -> String {
    resume(&t.lines().map(str::to_string).collect::<Vec<_>>()).unwrap_or_default()
}

fn titres_documents(index: &Index) -> Result<HashMap<String, String>> {
    let mut st = index.conn.prepare("SELECT id, titre FROM documents")?;
    let v = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Résout un identifiant de symbole complet ou partiel (R5). `Err(candidats)` si ambigu ou inconnu.
pub fn resoudre_symbole(
    index: &Index,
    demande: &str,
) -> Result<std::result::Result<String, Vec<String>>> {
    let c = &index.conn;
    let exact: Option<String> = c
        .query_row("SELECT id FROM symboles WHERE id = ?1", [demande], |r| {
            r.get(0)
        })
        .optional()?;
    if let Some(id) = exact {
        return Ok(Ok(id));
    }
    let mut st = c.prepare(
        "SELECT id FROM symboles WHERE nom_qualifie = ?1 OR nom = ?1 OR id LIKE '%::' || ?1
         OR id LIKE ?1 || '@cfg(%' OR id LIKE '%::' || ?1 || '@cfg(%' ORDER BY id",
    )?;
    let candidats: Vec<String> = st
        .query_map([demande], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let candidats: Vec<String> = candidats
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if candidats.len() == 1 {
        return Ok(Ok(candidats[0].clone()));
    }
    Ok(Err(candidats))
}

fn sortie_ambigue(demande: &str, candidats: &[String]) -> Sortie {
    let corps = if candidats.is_empty() {
        "aucun symbole ne correspond ; essayer `wikik query cherche`\n".to_string()
    } else {
        candidats.iter().map(|c| format!("{c}\n")).collect()
    };
    let mut s = finaliser(
        &format!("« {demande} » : {} candidat(s)", candidats.len()),
        corps,
        json!({ "demande": demande, "candidats": candidats }),
    );
    s.ambigu = true;
    s
}

/// `carte` : vue d'ensemble du projet, un fichier par ligne (EF-020). Les champs de classe ne sont
/// pas comptés comme symboles : aucun bloc n'est exigé sur eux.
pub fn carte(index: &Index, complet: bool) -> Result<Sortie> {
    let c = &index.conn;
    let fichiers = requete_json(
        c,
        "SELECT f.chemin, f.entete, f.lignes,
                (SELECT COUNT(*) FROM symboles s WHERE s.fichier = f.chemin AND s.est_test = 0 AND s.genre <> 'champ'),
                (SELECT COUNT(*) FROM symboles s WHERE s.fichier = f.chemin AND s.est_test = 0 AND s.genre <> 'champ' AND s.doc IS NOT NULL)
         FROM fichiers f ORDER BY f.chemin",
        &["chemin", "entete", "lignes", "symboles", "documentes"],
    )?;
    let total_l: i64 = fichiers
        .iter()
        .map(|f| f["lignes"].as_i64().unwrap_or(0))
        .sum();
    let total_s: i64 = fichiers
        .iter()
        .map(|f| f["symboles"].as_i64().unwrap_or(0))
        .sum();
    let total_d: i64 = fichiers
        .iter()
        .map(|f| f["documentes"].as_i64().unwrap_or(0))
        .sum();
    let mut par_dossier: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for f in &fichiers {
        let ch = f["chemin"].as_str().unwrap_or_default();
        let dossier = ch.rsplit_once('/').map_or("", |p| p.0).to_string();
        par_dossier.entry(dossier).or_default().push(f);
    }
    let mut corps = String::new();
    let resume_seul = fichiers.len() > 150 && !complet;
    for (dossier, fs) in &par_dossier {
        let nom = if dossier.is_empty() {
            "./".to_string()
        } else {
            format!("{dossier}/")
        };
        if resume_seul {
            let s: i64 = fs.iter().map(|f| f["symboles"].as_i64().unwrap_or(0)).sum();
            corps.push_str(&format!("{nom} : {} fichiers, {s} symboles\n", fs.len()));
            continue;
        }
        corps.push_str(&format!("{nom}\n"));
        for f in fs {
            let ch = f["chemin"].as_str().unwrap_or_default();
            let court = ch.rsplit('/').next().unwrap_or(ch);
            let role = f["entete"]
                .as_str()
                .map(une_ligne)
                .unwrap_or_else(|| "(sans en-tête)".into());
            corps.push_str(&format!(
                "  {court} — {role} ({} sym, {} doc)\n",
                f["symboles"], f["documentes"]
            ));
        }
    }
    if resume_seul {
        corps.push_str("(plus de 150 fichiers : `--complet` pour le détail, ou `wikik query fichier <chemin>`)\n");
    }

    let docs = requete_json(
        c,
        "SELECT id, genre, titre, statut FROM documents ORDER BY id",
        &["id", "genre", "titre", "statut"],
    )?;
    if !docs.is_empty() {
        let mut par_genre: BTreeMap<String, usize> = BTreeMap::new();
        for d in &docs {
            *par_genre
                .entry(d["genre"].as_str().unwrap_or("page").to_string())
                .or_default() += 1;
        }
        let liste: Vec<String> = par_genre.iter().map(|(g, n)| format!("{g} {n}")).collect();
        corps.push_str(&format!(
            "\nzone Projet : {} documents ({})\n",
            docs.len(),
            liste.join(", ")
        ));
        for d in docs.iter().filter(|d| {
            matches!(
                d["genre"].as_str(),
                Some("adr" | "spec" | "plan" | "bdr-definition")
            )
        }) {
            corps.push_str(&format!(
                "  {} — {} [{}]\n",
                d["id"].as_str().unwrap_or_default(),
                d["titre"].as_str().unwrap_or_default(),
                d["statut"].as_str().unwrap_or("?")
            ));
        }
    }
    corps.push_str("\nsuite : `query fichier <chemin>`, `query symbole <id>`, `query impact <id>`, `query perimes`\n");
    let taux = if total_s > 0 {
        total_d * 100 / total_s
    } else {
        0
    };
    let titre = format!(
        "carte · {} fichiers · {total_l} lignes · {total_s} symboles · doc {taux} %",
        fichiers.len()
    );
    Ok(finaliser(
        &titre,
        corps,
        json!({ "fichiers": fichiers, "documents": docs }),
    ))
}

/// `fichier` : en-tête, imports et symboles d'un fichier (EF-021).
///
/// Au-delà de 40 symboles, les champs, les privés et les publics sans doc sont listés par leur nom
/// seulement, pour tenir le budget de tokens (ENF-001) même sur un composant de plusieurs milliers de lignes.
pub fn fichier(index: &Index, demande: &str) -> Result<Sortie> {
    let c = &index.conn;
    let mut st = c.prepare(
        "SELECT chemin FROM fichiers WHERE chemin = ?1 OR chemin LIKE '%/' || ?1 ORDER BY chemin",
    )?;
    let trouves: Vec<String> = st
        .query_map([demande], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let chemin = match trouves.as_slice() {
        [un] => un.clone(),
        _ => return Ok(sortie_ambigue(demande, &trouves)),
    };
    let (entete, lignes, macros): (Option<String>, i64, i64) = c.query_row(
        "SELECT entete, lignes, lignes_macros FROM fichiers WHERE chemin = ?1",
        [&chemin],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let symboles = requete_json(
        c,
        &format!(
            "SELECT id, nom_qualifie, genre, visibilite, ligne_debut, resume, est_test FROM symboles
             WHERE fichier = '{}' ORDER BY ligne_debut, id",
            chemin.replace('\'', "''")
        ),
        &[
            "id",
            "nom_qualifie",
            "genre",
            "visibilite",
            "ligne_debut",
            "resume",
            "est_test",
        ],
    )?;
    let statuts = check::statuts(index, std::slice::from_ref(&chemin))?;
    let imports = requete_json(
        c,
        &format!(
            "SELECT alias, chemin_import FROM imports WHERE fichier = '{}' ORDER BY alias",
            chemin.replace('\'', "''")
        ),
        &["alias", "chemin"],
    )?;

    let mut corps = String::new();
    if let Some(e) = &entete {
        corps.push_str(e);
        corps.push_str("\n\n");
    }
    if !imports.is_empty() {
        // Regroupés par module : `module::{a, b as c}` (plus court qu'un chemin par import).
        let mut par_module: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for i in &imports {
            let a = i["alias"].as_str().unwrap_or_default();
            let ch = i["chemin"].as_str().unwrap_or_default();
            let (module, nom) = ch.rsplit_once("::").unwrap_or(("", ch));
            let entree = if nom == a {
                nom.to_string()
            } else {
                format!("{nom} as {a}")
            };
            par_module
                .entry(module.to_string())
                .or_default()
                .push(entree);
        }
        let l: Vec<String> = par_module
            .into_iter()
            .map(|(m, noms)| {
                if m.is_empty() {
                    noms.join(", ")
                } else if noms.len() == 1 {
                    format!("{m}::{}", noms[0])
                } else {
                    format!("{m}::{{{}}}", noms.join(", "))
                }
            })
            .collect();
        corps.push_str(&format!("imports : {}\n", l.join(" ; ")));
    }
    let mut documentes = 0;
    let mut tests = 0;
    // Gros fichier : résumé pour ce qui est visible hors du module, noms seuls pour le reste (ENF-001).
    let compact = symboles.len() > 40;
    let mut prives = vec![];
    let mut champs = vec![];
    let mut sans_doc = vec![];
    for s in &symboles {
        if s["est_test"] == 1 {
            tests += 1;
            continue;
        }
        let id = s["id"].as_str().unwrap_or_default();
        let statut = statuts.get(id).map(String::as_str).unwrap_or("à jour");
        if statut != "sans doc" {
            documentes += 1;
        }
        let marque = if statut == "périmé" {
            " [doc périmée]"
        } else {
            ""
        };
        let nq = s["nom_qualifie"].as_str().unwrap_or_default();
        if compact && s["visibilite"] == "prive" {
            prives.push(format!("{nq}{}", if marque.is_empty() { "" } else { "*" }));
            continue;
        }
        if compact && s["genre"] == "champ" {
            champs.push(nq.rsplit("::").next().unwrap_or(nq).to_string());
            continue;
        }
        if compact
            && s["resume"].is_null()
            && !matches!(
                s["genre"].as_str(),
                Some("classe" | "composant" | "struct" | "trait")
            )
        {
            sans_doc.push(nq.rsplit("::").next().unwrap_or(nq).to_string());
            continue;
        }
        let resume = s["resume"]
            .as_str()
            .map_or("(sans doc)".to_string(), |r| tronquer(r, 100));
        corps.push_str(&format!(
            "L{:<5} {} {} {nq} — {resume}{marque}\n",
            s["ligne_debut"].as_i64().unwrap_or(0),
            s["visibilite"].as_str().unwrap_or_default(),
            s["genre"].as_str().unwrap_or_default(),
        ));
    }
    if !sans_doc.is_empty() {
        corps.push_str(&format!(
            "publics sans doc ({}) : {}\n",
            sans_doc.len(),
            sans_doc.join(", ")
        ));
    }
    if !champs.is_empty() {
        corps.push_str(&format!(
            "champs publics ({}) : {}\n",
            champs.len(),
            champs.join(", ")
        ));
    }
    if !prives.is_empty() {
        corps.push_str(&format!(
            "privés ({}, * = doc périmée) : {}\n",
            prives.len(),
            prives.join(", ")
        ));
    }
    if tests > 0 {
        corps.push_str(&format!("+ {tests} symbole(s) de test\n"));
    }
    if macros > 0 {
        corps.push_str(&format!(
            "code en macros : {macros} lignes ({} %), appels non analysés sauf composants `view!`\n",
            macros * 100 / lignes.max(1)
        ));
    }
    let n = symboles
        .iter()
        .filter(|s| s["est_test"] != 1 && s["genre"] != "champ")
        .count();
    let titre =
        format!("fichier {chemin} · {lignes} lignes · {n} symboles ({documentes} documentés)");
    Ok(finaliser(
        &titre,
        corps,
        json!({ "chemin": chemin, "entete": entete, "imports": imports, "symboles": symboles, "statuts": statuts }),
    ))
}

struct Voisin {
    id: String,
    confiance: String,
    resume: Option<String>,
}

fn voisins(
    index: &Index,
    id: &str,
    entrants: bool,
    genres: &[&str],
) -> Result<Vec<(String, Voisin)>> {
    let (moi, autre) = if entrants {
        ("cible", "source")
    } else {
        ("source", "cible")
    };
    let liste = genres
        .iter()
        .map(|g| format!("'{g}'"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT a.genre, a.{autre}, a.confiance, s.resume FROM aretes a LEFT JOIN symboles s ON s.id = a.{autre}
         WHERE a.{moi} = ?1 AND a.genre IN ({liste}) ORDER BY a.genre, a.{autre}"
    );
    let mut st = index.conn.prepare(&sql)?;
    let v = st
        .query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                Voisin {
                    id: r.get(1)?,
                    confiance: r.get(2)?,
                    resume: r.get(3)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

fn marque_confiance(c: &str) -> &'static str {
    if c == "certain" { "" } else { " (probable)" }
}

/// `symbole` : tout ce qu'il faut savoir d'un symbole sans ouvrir son fichier (EF-022).
pub fn symbole(index: &Index, demande: &str, avec_code: bool) -> Result<Sortie> {
    let id = match resoudre_symbole(index, demande)? {
        Ok(id) => id,
        Err(c) => return Ok(sortie_ambigue(demande, &c)),
    };
    let c = &index.conn;
    let s = &requete_json(
        c,
        &format!(
            "SELECT id, fichier, genre, visibilite, ligne_debut, ligne_fin, signature, doc FROM symboles WHERE id = '{}'",
            id.replace('\'', "''")
        ),
        &[
            "id",
            "fichier",
            "genre",
            "visibilite",
            "ligne_debut",
            "ligne_fin",
            "signature",
            "doc",
        ],
    )?[0];
    let fichier_s = s["fichier"].as_str().unwrap_or_default().to_string();
    let statut = check::statuts(index, std::slice::from_ref(&fichier_s))?
        .remove(&id)
        .unwrap_or_default();
    let titres = titres_documents(index)?;

    let mut corps = format!("{}\n", s["signature"].as_str().unwrap_or_default());
    if let Some(doc) = s["doc"].as_str() {
        corps.push_str("---\n");
        corps.push_str(doc);
        corps.push_str("\n---\n");
    }
    let sortants = voisins(index, &id, false, &["appel", "rend", "adr", "req", "topic"])?;
    let entrants = voisins(index, &id, true, &["appel", "rend"])?;
    let lister = |titre: &str, v: &[&Voisin]| -> String {
        if v.is_empty() {
            return String::new();
        }
        let l: Vec<String> = v
            .iter()
            .map(|x| format!("{}{}", x.id, marque_confiance(&x.confiance)))
            .collect();
        format!("{titre} : {}\n", l.join(", "))
    };
    corps.push_str(&lister("appelle", &filtre(&sortants, "appel")));
    corps.push_str(&lister("rend", &filtre(&sortants, "rend")));
    corps.push_str(&lister("appelé par", &filtre(&entrants, "appel")));
    corps.push_str(&lister("rendu par", &filtre(&entrants, "rend")));
    for (g, nom) in [("adr", "ADR"), ("req", "exigences")] {
        let v = filtre(&sortants, g);
        if !v.is_empty() {
            let l: Vec<String> = v
                .iter()
                .map(|x| {
                    format!(
                        "{} {}",
                        x.id,
                        titres.get(&x.id).map_or("(introuvable)", String::as_str)
                    )
                })
                .collect();
            corps.push_str(&format!("{nom} : {}\n", l.join(" ; ")));
        }
    }
    let topics = filtre(&sortants, "topic");
    if !topics.is_empty() {
        let l: Vec<&str> = topics
            .iter()
            .map(|x| x.id.trim_start_matches("topic:"))
            .collect();
        corps.push_str(&format!("topics : {}\n", l.join(", ")));
    }
    let mut code = None;
    if avec_code {
        let texte = std::fs::read_to_string(index.racine.join(&fichier_s)).unwrap_or_default();
        let (d, f) = (
            s["ligne_debut"].as_u64().unwrap_or(1) as usize,
            s["ligne_fin"].as_u64().unwrap_or(1) as usize,
        );
        let extrait: Vec<&str> = texte.lines().skip(d - 1).take(f + 1 - d).collect();
        let extrait = extrait.join("\n");
        corps.push_str(&format!("```rust\n{extrait}\n```\n"));
        code = Some(extrait);
    }
    let titre = format!(
        "symbole {id} · {} {} · L{}-L{} · doc {statut}",
        s["visibilite"].as_str().unwrap_or_default(),
        s["genre"].as_str().unwrap_or_default(),
        s["ligne_debut"],
        s["ligne_fin"]
    );
    let json = json!({
        "symbole": s, "statut_doc": statut, "code": code,
        "sortants": sortants.iter().map(|(g, v)| json!({"genre": g, "cible": v.id, "confiance": v.confiance})).collect::<Vec<_>>(),
        "entrants": entrants.iter().map(|(g, v)| json!({"genre": g, "source": v.id, "confiance": v.confiance})).collect::<Vec<_>>(),
    });
    Ok(finaliser(&titre, corps, json))
}

/// Appelant atteint par `impact` : (distance, chemin entièrement certain, résumé).
type Atteint = (usize, bool, Option<String>);

/// `impact` : appelants transitifs, ADR, exigences et pièges à relire avant de modifier (EF-023).
pub fn impact(index: &Index, demande: &str, profondeur: usize) -> Result<Sortie> {
    let id = match resoudre_symbole(index, demande)? {
        Ok(id) => id,
        Err(c) => return Ok(sortie_ambigue(demande, &c)),
    };
    // Parcours en largeur des arêtes entrantes ; une arête probable rend tout le chemin probable.
    let mut vus: BTreeMap<String, Atteint> = BTreeMap::new();
    let mut file = VecDeque::from([(id.clone(), 0usize, true)]);
    while let Some((courant, d, sur)) = file.pop_front() {
        if d >= profondeur {
            continue;
        }
        for (_, v) in voisins(index, &courant, true, &["appel", "rend"])? {
            if v.id == id || vus.contains_key(&v.id) {
                continue;
            }
            let sur_v = sur && v.confiance == "certain";
            vus.insert(v.id.clone(), (d + 1, sur_v, v.resume.clone()));
            file.push_back((v.id, d + 1, sur_v));
        }
    }
    let titres = titres_documents(index)?;
    let mut concernes: Vec<String> = vec![id.clone()];
    concernes.extend(vus.keys().cloned());
    let mut liens: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut pieges = vec![];
    let mut tests = vec![];
    for s in &concernes {
        for (g, v) in voisins(index, s, false, &["adr", "req"])? {
            liens
                .entry((g, v.id))
                .or_default()
                .insert(s.rsplit("::").next().unwrap_or(s).to_string());
        }
        let mut st = index
            .conn
            .prepare("SELECT valeur FROM balises WHERE symbole = ?1 AND nom = 'pitfall'")?;
        for p in st.query_map([s], |r| r.get::<_, String>(0))? {
            pieges.push((s.clone(), p?));
        }
        let est_test: bool =
            index
                .conn
                .query_row("SELECT est_test FROM symboles WHERE id = ?1", [s], |r| {
                    r.get(0)
                })?;
        if est_test {
            tests.push(s.clone());
        }
    }

    let mut par_distance: Vec<(&String, &Atteint)> = vus.iter().collect();
    par_distance.sort_by(|a, b| a.1.0.cmp(&b.1.0).then(a.0.cmp(b.0)));
    let limite = 60;
    let mut corps = String::new();
    for (s, (d, sur, r)) in par_distance.iter().take(limite) {
        corps.push_str(&format!(
            "{d}  {s}{} — {}\n",
            if *sur { "" } else { " (probable)" },
            r.as_deref().unwrap_or("(sans doc)")
        ));
    }
    if vus.len() > limite {
        corps.push_str(&format!(
            "… et {} autres (réduire `--profondeur`)\n",
            vus.len() - limite
        ));
    }
    if vus.is_empty() {
        corps.push_str("aucun appelant connu (attention : appels de méthode et macros partiellement analysés)\n");
    }
    for (g, nom) in [("adr", "ADR"), ("req", "exigences")] {
        let l: Vec<String> = liens
            .iter()
            .filter(|((gg, _), _)| gg == g)
            .map(|((_, cible), via)| {
                format!(
                    "{cible} {} (via {})",
                    titres.get(cible).map_or("(introuvable)", String::as_str),
                    via.iter().cloned().collect::<Vec<_>>().join(", ")
                )
            })
            .collect();
        if !l.is_empty() {
            corps.push_str(&format!("{nom} : {}\n", l.join(" ; ")));
        }
    }
    for (s, p) in &pieges {
        corps.push_str(&format!(
            "piège ({}) : {p}\n",
            s.rsplit("::").next().unwrap_or(s)
        ));
    }
    if !tests.is_empty() {
        corps.push_str(&format!("tests concernés : {}\n", tests.join(", ")));
    }
    let titre = format!(
        "impact de {id} · {} symbole(s) · profondeur {profondeur}",
        vus.len()
    );
    let json = json!({
        "symbole": id,
        "appelants": par_distance.iter().map(|(s, (d, sur, _))| json!({"id": s, "distance": d, "confiance": if *sur {"certain"} else {"probable"}})).collect::<Vec<_>>(),
        "liens": liens.iter().map(|((g, c), via)| json!({"genre": g, "cible": c, "via": via})).collect::<Vec<_>>(),
        "pieges": pieges.iter().map(|(s, p)| json!({"symbole": s, "piege": p})).collect::<Vec<_>>(),
    });
    Ok(finaliser(&titre, corps, json))
}

/// `perimes` : doc périmée ou manquante, une ligne par symbole (EF-024).
pub fn perimes(index: &Index) -> Result<Sortie> {
    let r = check::verifier(index)?;
    let retenus: Vec<&check::Diagnostic> = r
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("perime-") || d.code == "manquant")
        .collect();
    let mut corps = String::new();
    for d in &retenus {
        corps.push_str(&format!(
            "{}:{} {} {}\n",
            d.fichier,
            d.ligne,
            d.code,
            d.symbole
                .as_deref()
                .and_then(|s| s.split_once("::").map(|p| p.1))
                .unwrap_or("")
        ));
    }
    if retenus.is_empty() {
        corps.push_str("aucune doc périmée ni manquante\n");
    }
    let titre = format!(
        "perimes · {} symbole(s) · référence {}",
        retenus.len(),
        r.mode_reference
    );
    Ok(finaliser(&titre, corps, json!({ "diagnostics": retenus })))
}

/// `cherche` : recherche plein texte (FTS5, classement bm25) dans les blocs et les documents (EF-025).
pub fn cherche(index: &Index, texte: &str) -> Result<Sortie> {
    let mots: Vec<String> = texte
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|m| !m.is_empty())
        .map(|m| format!("\"{}\"*", m.replace('"', "")))
        .collect();
    if mots.is_empty() {
        bail!("recherche vide");
    }
    let sql = "SELECT cle, genre, titre, snippet(recherche, 3, '[', ']', '…', 12) FROM recherche
               WHERE recherche MATCH ?1 ORDER BY rank LIMIT 15";
    let executer = |expr: &str| -> Result<Vec<(String, String, String, String)>> {
        let mut st = index.conn.prepare(sql)?;
        let v = st
            .query_map([expr], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(v)
    };
    let mut resultats = executer(&mots.join(" "))?;
    let mut mode = "tous les mots";
    if resultats.is_empty() && mots.len() > 1 {
        resultats = executer(&mots.join(" OR "))?;
        mode = "au moins un mot";
    }
    let mut corps = String::new();
    for (cle, genre, titre, extrait) in &resultats {
        let extrait = extrait.split_whitespace().collect::<Vec<_>>().join(" ");
        if genre == "symbole" {
            let r: Option<String> = index
                .conn
                .query_row("SELECT resume FROM symboles WHERE id = ?1", [cle], |r| {
                    r.get(0)
                })
                .optional()?
                .flatten();
            corps.push_str(&format!("symbole {cle} — {}\n", r.unwrap_or(extrait)));
        } else {
            corps.push_str(&format!("document {cle} — {titre} : {extrait}\n"));
        }
    }
    if resultats.is_empty() {
        corps.push_str("aucun résultat\n");
    }
    let titre = format!(
        "cherche « {texte} » · {} résultat(s) · {mode}",
        resultats.len()
    );
    let json = json!({ "resultats": resultats.iter().map(|(c, g, t, e)| json!({"cle": c, "genre": g, "titre": t, "extrait": e})).collect::<Vec<_>>() });
    Ok(finaliser(&titre, corps, json))
}

fn document(index: &Index, id: &str) -> Result<Option<Value>> {
    let v = requete_json(
        &index.conn,
        &format!(
            "SELECT id, chemin, genre, titre, statut, version, priorite, frontmatter, corps FROM documents WHERE id = '{}'",
            id.replace('\'', "''")
        ),
        &[
            "id",
            "chemin",
            "genre",
            "titre",
            "statut",
            "version",
            "priorite",
            "frontmatter",
            "corps",
        ],
    )?;
    Ok(v.into_iter().next())
}

/// Texte d'une section `## titre` (jusqu'à la suivante), tronqué.
fn section(corps: &str, titre: &str, max: usize) -> Option<String> {
    let mut dedans = false;
    let mut lignes = vec![];
    for l in corps.lines() {
        if let Some(t) = l.strip_prefix("## ") {
            if dedans {
                break;
            }
            dedans = t.trim().eq_ignore_ascii_case(titre);
            continue;
        }
        if dedans {
            lignes.push(l);
        }
    }
    let t = lignes.join("\n").trim().to_string();
    if t.is_empty() {
        return None;
    }
    Some(if t.chars().count() > max {
        format!("{}…", t.chars().take(max).collect::<String>())
    } else {
        t
    })
}

/// Symboles et fichiers (balise d'en-tête) qui citent `cible` : (source, résumé, est un test).
fn citants(index: &Index, cible: &str, genre: &str) -> Result<Vec<(String, Option<String>, bool)>> {
    let mut st = index.conn.prepare(
        "SELECT a.source, s.resume, f.entete, coalesce(s.est_test, 0) FROM aretes a
         LEFT JOIN symboles s ON s.id = a.source LEFT JOIN fichiers f ON f.chemin = a.source
         WHERE a.cible = ?1 AND a.genre = ?2 ORDER BY a.source",
    )?;
    let v = st
        .query_map(params![cible, genre], |r| {
            let entete: Option<String> = r.get(2)?;
            let resume: Option<String> = r.get(1)?;
            Ok((
                r.get(0)?,
                resume.or_else(|| entete.map(|e| format!("fichier : {}", une_ligne(&e)))),
                r.get(3)?,
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// `adr` : l'ADR résumé et ce qui le cite (EF-026).
pub fn adr(index: &Index, demande: &str) -> Result<Sortie> {
    let id = normaliser_adr(demande).unwrap_or_else(|| demande.to_string());
    let Some(d) = document(index, &id)? else {
        return Ok(sortie_ambigue(demande, &[]));
    };
    let corps_doc = d["corps"].as_str().unwrap_or_default();
    let mut corps = String::new();
    if let Some(dec) = section(corps_doc, "Décision", 600) {
        corps.push_str(&format!("décision : {dec}\n"));
    }
    let syms = citants(index, &id, "adr")?;
    for (s, r, _) in &syms {
        corps.push_str(&format!(
            "cité par {s} — {}\n",
            r.as_deref().unwrap_or("(sans doc)")
        ));
    }
    let docs = citants(index, &id, "lien")?;
    if !docs.is_empty() {
        corps.push_str(&format!(
            "documents liés : {}\n",
            docs.iter()
                .map(|x| x.0.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let titre = format!(
        "adr {id} · {} · {} · cité par {} symbole(s)",
        d["titre"].as_str().unwrap_or_default(),
        d["statut"].as_str().unwrap_or("?"),
        syms.len()
    );
    Ok(finaliser(
        &titre,
        corps,
        json!({ "document": d, "symboles": syms.iter().map(|x| &x.0).collect::<Vec<_>>() }),
    ))
}

fn ligne_champ(corps: &str, champ: &str) -> Option<String> {
    corps.lines().find_map(|l| {
        let l = l.trim();
        let reste = l
            .strip_prefix(&format!("**{champ}**"))
            .or_else(|| l.strip_prefix(&format!("{champ} ")))?;
        Some(reste.trim_start_matches([' ', ':']).trim().to_string())
    })
}

/// `exigence` : énoncé, besoins, et ce qui l'implémente ou la teste (EF-027).
pub fn exigence(index: &Index, demande: &str) -> Result<Sortie> {
    let id = demande.trim().to_ascii_uppercase();
    let Some(d) = document(index, &id)? else {
        return Ok(sortie_ambigue(demande, &[]));
    };
    let titres = titres_documents(index)?;
    let corps_doc = d["corps"].as_str().unwrap_or_default();
    let fm: Value =
        serde_json::from_str(d["frontmatter"].as_str().unwrap_or("{}")).unwrap_or_default();
    let mut corps = String::new();
    if let Some(b) = fm.get("besoins").and_then(Value::as_array) {
        let l: Vec<String> = b
            .iter()
            .filter_map(Value::as_str)
            .map(|x| {
                format!(
                    "{x} {}",
                    titres.get(x).map_or("(introuvable)", String::as_str)
                )
            })
            .collect();
        corps.push_str(&format!("besoins : {}\n", l.join(" ; ")));
    }
    for (champ, nom) in [("Énoncé", "énoncé"), ("Critère d'acceptation", "critère")] {
        if let Some(v) = ligne_champ(corps_doc, champ) {
            corps.push_str(&format!("{nom} : {v}\n"));
        }
    }
    let syms = citants(index, &id, "req")?;
    let (tests, code): (Vec<_>, Vec<_>) = syms.iter().partition(|x| x.2);
    for (s, r, _) in &code {
        corps.push_str(&format!(
            "implémentée par {s} — {}\n",
            r.as_deref().unwrap_or("(sans doc)")
        ));
    }
    for (s, _, _) in &tests {
        corps.push_str(&format!("testée par {s}\n"));
    }
    if syms.is_empty() {
        corps.push_str("aucun symbole ne la cite (`@req`)\n");
    }
    let docs = citants(index, &id, "lien")?;
    if !docs.is_empty() {
        corps.push_str(&format!(
            "documents liés : {}\n",
            docs.iter()
                .map(|x| x.0.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let titre = format!(
        "exigence {id} · {} · {} · {}",
        d["titre"].as_str().unwrap_or_default(),
        d["priorite"].as_str().unwrap_or("?"),
        d["statut"].as_str().unwrap_or("?")
    );
    let json = json!({ "document": d, "symboles": code.iter().map(|x| &x.0).collect::<Vec<_>>(), "tests": tests.iter().map(|x| &x.0).collect::<Vec<_>>() });
    Ok(finaliser(&titre, corps, json))
}

/// `ref` : résout une référence `@wiki:` copiée depuis le wiki (EF-028).
///
/// Une ancre `#section` désigne la section dont le titre a cet identifiant, calculé comme le wiki ([`ancre`]).
pub fn reference(index: &Index, brute: &str) -> Result<Sortie> {
    let r = brute.trim().trim_start_matches("@wiki:");
    if let Some((chemin, sym)) = r.split_once("::") {
        let chemin = retirer_projet(index, chemin)?;
        return symbole(index, &format!("{chemin}::{sym}"), false);
    }
    let (cible, ancre) = r.split_once('#').map_or((r, None), |(a, b)| (a, Some(b)));
    let cible = retirer_projet(index, cible)?;
    let est_code: bool = index.conn.query_row(
        "SELECT COUNT(*) > 0 FROM fichiers WHERE chemin = ?1",
        [&cible],
        |x| x.get(0),
    )?;
    if est_code {
        return fichier(index, &cible);
    }
    let mut st = index.conn.prepare("SELECT id, chemin FROM documents")?;
    let docs: Vec<(String, String)> = st
        .query_map([], |x| Ok((x.get(0)?, x.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let sans_md = cible.trim_end_matches(".md");
    let trouve = docs.iter().find(|(id, ch)| {
        id == sans_md
            || ch.trim_end_matches(".md") == sans_md
            || ch.trim_end_matches(".md").ends_with(&format!("/{sans_md}"))
    });
    let Some((id, _)) = trouve else {
        return Ok(sortie_ambigue(brute, &[]));
    };
    let d = document(index, id)?.unwrap_or_default();
    let corps_doc = d["corps"].as_str().unwrap_or_default();
    let texte = match ancre {
        Some(a) => {
            section_par_ancre(corps_doc, a, 1500).unwrap_or_else(|| tronquer(corps_doc, 1500))
        }
        None => tronquer(corps_doc, 1500),
    };
    let titre = format!(
        "document {id} · {}",
        d["titre"].as_str().unwrap_or_default()
    );
    Ok(finaliser(
        &titre,
        format!("{texte}\n"),
        json!({ "document": d, "ancre": ancre }),
    ))
}

/// Identifiant d'ancre d'un titre, comme le calcule le wiki (`ancre()` dans `front/src/lib/markdown.ts`) :
/// minuscules, accents retirés, tout le reste devient `-`.
pub fn ancre(titre: &str) -> String {
    let mut s = String::new();
    for c in titre.to_lowercase().chars() {
        let base = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            c if c.is_ascii_alphanumeric() => c,
            _ => '-',
        };
        if base != '-' || !s.ends_with('-') {
            s.push(base);
        }
    }
    s.trim_matches('-').to_string()
}

/// Section dont le titre (tout niveau) a pour ancre `a`, jusqu'au prochain titre de niveau égal ou supérieur.
fn section_par_ancre(corps: &str, a: &str, max: usize) -> Option<String> {
    let mut niveau = None;
    let mut lignes = vec![];
    for l in corps.lines() {
        let n = l.chars().take_while(|c| *c == '#').count();
        let est_titre = n > 0 && l[n..].starts_with(' ');
        match niveau {
            None if est_titre && ancre(&l[n..]) == a => {
                niveau = Some(n);
                lignes.push(l);
            }
            Some(n0) if est_titre && n <= n0 => break,
            Some(_) => lignes.push(l),
            None => {}
        }
    }
    (!lignes.is_empty()).then(|| tronquer(&lignes.join("\n"), max))
}

fn tronquer(t: &str, max: usize) -> String {
    let t = t.trim();
    if t.chars().count() > max {
        format!("{}…", t.chars().take(max).collect::<String>())
    } else {
        t.to_string()
    }
}

/// `@wiki:projet/chemin` : retire le nom de projet s'il ne fait pas partie d'un chemin connu.
fn retirer_projet(index: &Index, chemin: &str) -> Result<String> {
    let connu = |c: &str| -> Result<bool> {
        Ok(index.conn.query_row(
            "SELECT (SELECT COUNT(*) FROM fichiers WHERE chemin = ?1) + (SELECT COUNT(*) FROM documents WHERE chemin LIKE ?1 || '%' OR id = ?1)",
            [c],
            |r| r.get::<_, i64>(0),
        )? > 0)
    };
    if connu(chemin)? {
        return Ok(chemin.to_string());
    }
    match chemin.split_once('/') {
        Some((_, reste)) if connu(reste)? => Ok(reste.to_string()),
        _ => Ok(chemin.to_string()),
    }
}

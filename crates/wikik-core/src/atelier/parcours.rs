//! Moteur de parcours de l'atelier (SPEC-18) : état des étapes, consignes, propositions en attente,
//! décisions (garder, corriger, rejeter), saisies, sélections et synthèses.
//!
//! Chaque élément gardé devient un fichier (SPEC-17) qui cite ses sources réelles : un identifiant que le
//! LLM aurait inventé est écarté. L'appel au LLM se fait hors de tout verrou : [`preparer`] construit la
//! demande, [`enregistrer`] range la réponse.
//!
//! @req EF-101, EF-103, EF-105, EF-107
//! @adr 10

use super::artefacts;
use super::fiches::{self, Fiche, SousEtape};
use super::fournisseur::{Consommation, Reponse};
use crate::index::{DOSSIER, Index};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use ts_rs::TS;

/// Artefact affiché dans une étape.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ArtefactResume {
    pub id: String,
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub genre: String,
    pub titre: String,
    pub statut: String,
    pub chemin: String,
    pub sources: Vec<String>,
}

/// Une proposition du LLM et la décision prise.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Proposition {
    #[ts(type = "Record<string, unknown>")]
    pub donnees: Value,
    /// `en_attente`, `garde` ou `rejete`.
    pub etat: String,
    pub artefact: Option<String>,
}

/// Lot de propositions d'une sous-étape, pour une cible éventuelle.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Lot {
    pub etape: String,
    pub sous_etape: String,
    pub cible: Option<String>,
    pub elements: Vec<Proposition>,
    /// Propositions écartées automatiquement (verbatim introuvable, par exemple), pour information.
    pub ecartees: Vec<String>,
}

/// État d'une étape.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct EtatEtape {
    pub fiche: Fiche,
    /// `a_faire`, `en_cours`, `faite` ou `sautee`.
    pub etat: String,
    pub mode: String,
    pub artefacts: Vec<ArtefactResume>,
    /// Sélection par sous-étape.
    pub selections: BTreeMap<String, Vec<String>>,
    pub lots: Vec<Lot>,
}

/// État complet de l'atelier, servi à l'interface.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct EtatAtelier {
    /// Titre du projet (pitch), s'il existe.
    pub projet: Option<String>,
    pub etapes: Vec<EtatEtape>,
    pub clarifications: Vec<ArtefactResume>,
    pub fournisseur: String,
    pub consommation: Option<Consommation>,
    pub url_wiki: String,
}

/// Demande prête à envoyer au LLM, construite sous verrou puis exécutée sans.
#[derive(Debug, Clone)]
pub struct Preparation {
    pub etape: String,
    pub sous_etape: String,
    pub cible: Option<String>,
    pub consigne: String,
    pub schema: Value,
    pub exemple: Option<Value>,
    /// Texte dont chaque `verbatim` proposé doit être extrait mot pour mot (free writing).
    pub texte_source: Option<String>,
    /// Texte saisi qui accompagne la demande (description libre du pitch).
    pub entree: Option<String>,
}

// ---------------------------------------------------------------- état persistant

fn dossier_travail(index: &Index) -> PathBuf {
    travail(&index.racine)
}

/// Dossier de travail non versionné de l'atelier (`.wikik/atelier/`).
pub fn travail(racine: &Path) -> PathBuf {
    racine.join(DOSSIER).join("atelier")
}

fn chemin_parcours(index: &Index) -> String {
    artefacts::chemin_relatif(
        &index.config,
        artefacts::type_de("parcours").unwrap(),
        "PARCOURS",
    )
}

/// État du parcours (`docs/atelier/parcours.md`), créé vide au besoin.
fn charger_parcours(index: &Index) -> Map<String, Value> {
    match artefacts::lire(&index.racine, &chemin_parcours(index)) {
        Ok((fm, _)) if !fm.is_empty() => fm,
        _ => json!({
            "id": "PARCOURS", "type": "parcours", "titre": "Parcours de l'atelier", "statut": "brouillon",
            "mode": "express", "etapes": {}, "selections": {}, "liens_carte": [], "horizons": null,
        })
        .as_object()
        .unwrap()
        .clone(),
    }
}

fn sauver_parcours(index: &Index, p: &Map<String, Value>) -> Result<()> {
    let relatif = chemin_parcours(index);
    std::fs::create_dir_all(index.racine.join(&relatif).parent().unwrap())?;
    let mut lignes =
        String::from("# Parcours de l'atelier\n\n| Étape | État | Mode |\n|---|---|---|\n");
    for f in fiches::fiches().iter().filter(|f| !f.hors_parcours) {
        let e = p.get("etapes").and_then(|e| e.get(&f.id));
        let etat = e
            .and_then(|e| e.get("etat"))
            .and_then(Value::as_str)
            .unwrap_or("a_faire");
        let mode = e
            .and_then(|e| e.get("mode"))
            .and_then(Value::as_str)
            .unwrap_or("express");
        lignes.push_str(&format!("| {} | {etat} | {mode} |\n", f.titre));
    }
    artefacts::reecrire(&index.racine, &relatif, p, &lignes)
}

fn mode_de(p: &Map<String, Value>, etape: &str) -> String {
    p.get("etapes")
        .and_then(|e| e.get(etape))
        .and_then(|e| e.get("mode"))
        .and_then(Value::as_str)
        .or_else(|| p.get("mode").and_then(Value::as_str))
        .unwrap_or("express")
        .to_string()
}

fn marquer(p: &mut Map<String, Value>, etape: &str, etat: &str) {
    let etapes = p.entry("etapes").or_insert_with(|| json!({}));
    let e = etapes
        .as_object_mut()
        .unwrap()
        .entry(etape.to_string())
        .or_insert_with(|| json!({}));
    e["etat"] = Value::String(etat.into());
}

fn cle_lot(etape: &str, sous: &str, cible: Option<&str>) -> String {
    format!(
        "{etape}__{sous}__{}",
        cible.unwrap_or("_").replace(['/', ':'], "_")
    )
}

fn chemin_lot(index: &Index, cle: &str) -> PathBuf {
    dossier_travail(index)
        .join("propositions")
        .join(format!("{cle}.json"))
}

fn charger_lot(index: &Index, etape: &str, sous: &str, cible: Option<&str>) -> Lot {
    std::fs::read_to_string(chemin_lot(index, &cle_lot(etape, sous, cible)))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| Lot {
            etape: etape.into(),
            sous_etape: sous.into(),
            cible: cible.map(str::to_string),
            elements: vec![],
            ecartees: vec![],
        })
}

fn sauver_lot(index: &Index, lot: &Lot) -> Result<()> {
    let chemin = chemin_lot(
        index,
        &cle_lot(&lot.etape, &lot.sous_etape, lot.cible.as_deref()),
    );
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(chemin, serde_json::to_string_pretty(lot)?)?;
    Ok(())
}

// ---------------------------------------------------------------- lecture de l'index

struct Doc {
    id: String,
    genre: String,
    titre: String,
    statut: String,
    chemin: String,
    fm: Value,
}

fn documents(index: &Index) -> Result<Vec<Doc>> {
    let mut st = index.conn.prepare("SELECT id, genre, titre, coalesce(statut, ''), chemin, frontmatter FROM documents ORDER BY id")?;
    let v = st
        .query_map([], |r| {
            let fm: String = r.get(5)?;
            Ok(Doc {
                id: r.get(0)?,
                genre: r.get(1)?,
                titre: r.get(2)?,
                statut: r.get(3)?,
                chemin: r.get(4)?,
                fm: serde_json::from_str(&fm).unwrap_or_default(),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

fn resume(d: &Doc) -> ArtefactResume {
    ArtefactResume {
        id: d.id.clone(),
        genre: d.genre.clone(),
        titre: d.titre.clone(),
        statut: d.statut.clone(),
        chemin: d.chemin.clone(),
        sources: chaines(d.fm.get("sources")),
    }
}

fn chaines(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn texte(v: &Value, cle: &str) -> String {
    v.get(cle)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Sélection effective d'une sous-étape : choix enregistré, sinon présélection de la fiche.
fn selection(p: &Map<String, Value>, docs: &[Doc], fiche: &Fiche, se: &SousEtape) -> Vec<String> {
    let cle = format!("{}/{}", fiche.id, se.id);
    let choisie = chaines(p.get("selections").and_then(|s| s.get(&cle)));
    if !choisie.is_empty() {
        return choisie;
    }
    match se.preselection.as_deref() {
        Some("tous") => docs
            .iter()
            .filter(|d| Some(d.genre.as_str()) == se.parmi.as_deref())
            .map(|d| d.id.clone())
            .collect(),
        Some(autre) => fiches::fiche(autre)
            .ok()
            .and_then(|f| {
                f.sous_etapes
                    .iter()
                    .find(|s| s.genre == "selection")
                    .map(|s| selection(p, docs, f, s))
            })
            .unwrap_or_default(),
        None => vec![],
    }
}

// ---------------------------------------------------------------- état servi à l'interface

/// État de l'atelier pour l'interface.
pub fn etat(index: &Index, fournisseur: &str) -> Result<EtatAtelier> {
    let p = charger_parcours(index);
    let docs = documents(index)?;
    let mut etapes = vec![];
    for f in fiches::fiches().iter().filter(|f| !f.hors_parcours) {
        let artefacts = docs
            .iter()
            .filter(|d| {
                d.fm.get("technique").and_then(Value::as_str) == Some(f.id.as_str())
                    || (f.id == "pitch" && d.genre == "pitch")
            })
            .map(resume)
            .collect();
        let mut selections = BTreeMap::new();
        let mut lots = vec![];
        for se in &f.sous_etapes {
            if se.genre == "selection" {
                selections.insert(se.id.clone(), selection(&p, &docs, f, se));
            }
            if se.genre == "propositions" || se.brouillon {
                let prefixe = format!("{}__{}__", f.id, se.id);
                if let Ok(entrees) = std::fs::read_dir(dossier_travail(index).join("propositions"))
                {
                    let mut noms: Vec<String> = entrees
                        .flatten()
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    noms.sort();
                    for n in noms.into_iter().filter(|n| n.starts_with(&prefixe)) {
                        if let Some(lot) = std::fs::read_to_string(
                            dossier_travail(index).join("propositions").join(&n),
                        )
                        .ok()
                        .and_then(|t| serde_json::from_str::<Lot>(&t).ok())
                        {
                            lots.push(lot);
                        }
                    }
                }
            }
        }
        let e = p.get("etapes").and_then(|e| e.get(&f.id));
        etapes.push(EtatEtape {
            fiche: f.clone(),
            etat: e
                .and_then(|e| e.get("etat"))
                .and_then(Value::as_str)
                .unwrap_or("a_faire")
                .to_string(),
            mode: mode_de(&p, &f.id),
            artefacts,
            selections,
            lots,
        });
    }
    let projet = docs
        .iter()
        .find(|d| d.genre == "pitch")
        .map(|d| d.titre.clone());
    let clarifications = docs
        .iter()
        .filter(|d| d.genre == "clarification")
        .map(resume)
        .collect();
    Ok(EtatAtelier {
        projet,
        etapes,
        clarifications,
        fournisseur: fournisseur.into(),
        consommation: derniere_consommation(index),
        url_wiki: index.config.atelier.url_wiki.clone(),
    })
}

fn derniere_consommation(index: &Index) -> Option<Consommation> {
    let t = std::fs::read_to_string(dossier_travail(index).join("journal.jsonl")).ok()?;
    t.lines()
        .rev()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find_map(|v| {
            let c = v.get("consommation")?;
            Some(Consommation {
                cinq_heures: c.get("cinq_heures")?.as_f64()?,
                sept_jours: c.get("sept_jours")?.as_f64()?,
            })
        })
}

// ---------------------------------------------------------------- pilotage des étapes

/// `ouvrir`, `terminer`, `sauter` (étape optionnelle) ou `rouvrir` une étape ; changer son mode.
pub fn piloter(index: &Index, etape: &str, action: &str, mode: Option<&str>) -> Result<()> {
    let fiche = fiches::fiche(etape)?;
    let mut p = charger_parcours(index);
    match action {
        "ouvrir" | "rouvrir" => marquer(&mut p, etape, "en_cours"),
        "terminer" => marquer(&mut p, etape, "faite"),
        "sauter" if fiche.optionnelle => marquer(&mut p, etape, "sautee"),
        "sauter" => bail!("{} n'est pas optionnelle", fiche.titre),
        "mode" => {}
        autre => bail!("action inconnue : {autre}"),
    }
    if let Some(m) = mode {
        if !matches!(m, "express" | "complet") {
            bail!("mode inconnu : {m}");
        }
        let etapes = p.entry("etapes").or_insert_with(|| json!({}));
        let e = etapes
            .as_object_mut()
            .unwrap()
            .entry(etape.to_string())
            .or_insert_with(|| json!({"etat": "en_cours"}));
        e["mode"] = Value::String(m.into());
    }
    sauver_parcours(index, &p)
}

/// Enregistre la sélection d'une sous-étape `selection`.
pub fn selectionner(index: &Index, etape: &str, sous: &str, ids: &[String]) -> Result<()> {
    let fiche = fiches::fiche(etape)?;
    let se = fiche.sous_etape(sous)?;
    if se.genre != "selection" {
        bail!("{etape}/{sous} n'est pas une sélection");
    }
    let docs = documents(index)?;
    for id in ids {
        let d = docs
            .iter()
            .find(|d| &d.id == id)
            .with_context(|| format!("identifiant inconnu : {id}"))?;
        if Some(d.genre.as_str()) != se.parmi.as_deref() {
            bail!(
                "{id} est un(e) {}, pas un(e) {}",
                d.genre,
                se.parmi.as_deref().unwrap_or("?")
            );
        }
    }
    let mut p = charger_parcours(index);
    let sel = p.entry("selections").or_insert_with(|| json!({}));
    sel[format!("{etape}/{sous}")] = json!(ids);
    marquer(&mut p, etape, "en_cours");
    sauver_parcours(index, &p)
}

// ---------------------------------------------------------------- propositions

/// Vérifie la cible d'une sous-étape répétée (`pour_chaque`) et renvoie son document.
fn verifier_cible<'d>(
    index: &Index,
    docs: &'d [Doc],
    fiche: &Fiche,
    se: &SousEtape,
    cible: Option<&str>,
) -> Result<Option<&'d Doc>> {
    let Some(type_cible) = se
        .pour_chaque
        .as_deref()
        .or(se.complete.as_deref().filter(|c| *c == "clarification"))
    else {
        return Ok(None);
    };
    let id = cible.with_context(|| {
        format!(
            "{}/{} se fait pour chaque {type_cible} : cible attendue",
            fiche.id, se.id
        )
    })?;
    let d = docs
        .iter()
        .find(|d| d.id == id)
        .with_context(|| format!("cible inconnue : {id}"))?;
    if d.genre != type_cible {
        bail!(
            "{id} est un(e) {}, un(e) {type_cible} est attendu(e)",
            d.genre
        );
    }
    // Si l'étape sélectionne ses sujets, la cible doit en faire partie.
    if let Some(sel) = fiche
        .sous_etapes
        .iter()
        .find(|s| s.genre == "selection" && s.parmi.as_deref() == Some(type_cible))
    {
        let choix = selection(&charger_parcours(index), docs, fiche, sel);
        if !choix.is_empty() && !choix.iter().any(|c| c == id) {
            bail!("{id} ne fait pas partie de la sélection de {}", fiche.titre);
        }
    }
    Ok(Some(d))
}

/// Construit la demande au LLM pour une sous-étape (sous verrou ; l'appel se fait ensuite sans).
pub fn preparer(
    index: &Index,
    etape: &str,
    sous: &str,
    cible: Option<&str>,
    entree: Option<&str>,
) -> Result<Preparation> {
    let fiche = fiches::fiche(etape)?;
    let se = fiche.sous_etape(sous)?;
    if se.genre != "propositions" && !se.brouillon {
        bail!("{etape}/{sous} n'appelle pas le LLM");
    }
    let docs = documents(index)?;
    let doc_cible = verifier_cible(index, &docs, fiche, se, cible)?;
    let p = charger_parcours(index);
    let mode = mode_de(&p, etape);
    let n = se.variantes.map_or(1, |v| v.pour(&mode));
    let lot = charger_lot(index, etape, sous, cible);
    let deja_vus: Vec<String> = lot
        .elements
        .iter()
        .map(|e| {
            ["titre", "verbatim", "piste", "nom", "domaine"]
                .iter()
                .map(|k| texte(&e.donnees, k))
                .find(|t| !t.is_empty())
                .unwrap_or_default()
        })
        .filter(|t| !t.is_empty())
        .collect();
    let pitch = docs.iter().find(|d| d.genre == "pitch");
    let mut c = String::new();
    c.push_str(&format!("Tu animes l'étape « {} » d'un atelier de définition du besoin (framework Fast Design Thinking).\n", fiche.titre));
    c.push_str("Réponds en français. Tu lis le projet avec les outils wikik (atelier, document, cherche) ; tu n'écris rien toi-même.\n\n");
    if let Some(pt) = pitch {
        c.push_str(&format!(
            "Projet : {} — {}\n",
            pt.titre,
            texte(&pt.fm, "probleme")
        ));
    } else if etape != "pitch" {
        c.push_str("Le pitch n'est pas encore écrit.\n");
    }
    if let Some(d) = doc_cible {
        c.push_str(&format!(
            "Élément visé : {} « {} » ({}) : lis-le avec l'outil document.\n",
            d.id, d.titre, d.genre
        ));
    }
    if etape == "pitch" {
        let description = entree
            .map(str::to_string)
            .or_else(|| pitch.map(|pt| texte(&pt.fm, "description")))
            .unwrap_or_default();
        if description.trim().is_empty() {
            bail!("écris d'abord une description libre du projet");
        }
        c.push_str(&format!(
            "Description libre écrite par l'utilisateur :\n{description}\n"
        ));
    }
    c.push_str(&format!(
        "\nConsigne :\n{}\n",
        se.consigne.as_deref().unwrap_or("")
    ));
    c.push_str(&format!(
        "\nMode {mode} : produis exactement {n} élément(s) dans `elements`.\n"
    ));
    if !deja_vus.is_empty() {
        c.push_str(&format!(
            "Déjà proposé, ne le répète pas : {}.\n",
            deja_vus.join(" ; ")
        ));
    }
    c.push_str("Cite dans les champs d'identifiants (`sources`, `parent`, `besoins`) uniquement des identifiants qui existent dans l'atelier ou le projet.\n");
    let texte_source = (etape == "free-writing" && sous == "recolte")
        .then(|| doc_cible.and_then(|d| std::fs::read_to_string(index.racine.join(&d.chemin)).ok()))
        .flatten();
    Ok(Preparation {
        etape: etape.into(),
        sous_etape: sous.into(),
        cible: cible.map(str::to_string),
        consigne: c,
        schema: se.schema.clone().context("sous-étape sans schéma")?,
        exemple: fiche.exemple.get(&se.id).cloned(),
        texte_source,
        entree: entree.map(str::to_string),
    })
}

fn compacter(t: &str) -> String {
    t.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Range la réponse du LLM dans le lot de la sous-étape et tient le journal des appels.
pub fn enregistrer(
    index: &Index,
    prep: &Preparation,
    reponse: &Reponse,
    fournisseur: &str,
) -> Result<Lot> {
    let mut lot = charger_lot(index, &prep.etape, &prep.sous_etape, prep.cible.as_deref());
    let source = prep.texte_source.as_deref().map(compacter);
    for e in &reponse.elements {
        // Principe de non-altération (P.A.R.S.) : un verbatim doit figurer mot pour mot dans le texte libre.
        if let (Some(src), Some(v)) = (&source, e.get("verbatim").and_then(Value::as_str))
            && !src.contains(&compacter(v))
        {
            lot.ecartees
                .push(format!("verbatim introuvable dans le texte : « {v} »"));
            continue;
        }
        let mut donnees = e.clone();
        // Le brouillon de pitch garde la description d'origine.
        if let (Some(t), Some(o)) = (&prep.entree, donnees.as_object_mut()) {
            o.entry("description")
                .or_insert_with(|| Value::String(t.clone()));
        }
        lot.elements.push(Proposition {
            donnees,
            etat: "en_attente".into(),
            artefact: None,
        });
    }
    sauver_lot(index, &lot)?;
    let mut p = charger_parcours(index);
    marquer(&mut p, &prep.etape, "en_cours");
    sauver_parcours(index, &p)?;
    journaliser(&index.racine, prep, fournisseur, Ok(reponse))?;
    Ok(lot)
}

/// Ajoute une ligne au journal des appels (SPEC-19 R6), réussis ou non.
pub fn journaliser(
    racine: &Path,
    prep: &Preparation,
    fournisseur: &str,
    resultat: std::result::Result<&Reponse, &str>,
) -> Result<()> {
    let secondes = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut ligne = json!({
        "date": crate::relu::aujourdhui(), "horodatage": secondes, "etape": prep.etape, "sous_etape": prep.sous_etape,
        "cible": prep.cible, "fournisseur": fournisseur, "succes": resultat.is_ok(),
    });
    match resultat {
        Ok(r) => {
            ligne["elements"] = json!(r.elements.len());
            ligne["duree_ms"] = json!(r.duree_ms as u64);
            ligne["consommation"] = json!(r.consommation);
        }
        Err(e) => ligne["erreur"] = json!(e),
    }
    let journal = travail(racine).join("journal.jsonl");
    std::fs::create_dir_all(journal.parent().unwrap())?;
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal)?;
    writeln!(f, "{ligne}")?;
    Ok(())
}

/// Garde (éventuellement corrigée) ou rejette la proposition `i` d'un lot ; renvoie l'artefact créé ou complété.
pub fn decider(
    index: &Index,
    etape: &str,
    sous: &str,
    cible: Option<&str>,
    i: usize,
    decision: &str,
    corrigee: Option<Value>,
) -> Result<Option<String>> {
    let fiche = fiches::fiche(etape)?;
    let se = fiche.sous_etape(sous)?;
    let mut lot = charger_lot(index, etape, sous, cible);
    let element = lot
        .elements
        .get_mut(i)
        .with_context(|| format!("proposition {i} inconnue"))?;
    if element.etat != "en_attente" {
        bail!("proposition {i} déjà décidée ({})", element.etat);
    }
    let resultat = match decision {
        "rejeter" => {
            element.etat = "rejete".into();
            None
        }
        "garder" => {
            let donnees = corrigee.unwrap_or_else(|| element.donnees.clone());
            if let Some(schema_el) = se
                .schema
                .as_ref()
                .and_then(|s| s.pointer("/properties/elements/items"))
            {
                super::fiches::valider(schema_el, &donnees, "proposition")?;
            }
            let id = materialiser(index, fiche, se, cible, &donnees)?;
            let element = lot.elements.get_mut(i).unwrap();
            element.donnees = donnees;
            element.etat = "garde".into();
            element.artefact = id.clone();
            id
        }
        autre => bail!("décision inconnue : {autre} (garder ou rejeter)"),
    };
    sauver_lot(index, &lot)?;
    Ok(resultat)
}

// ---------------------------------------------------------------- matérialisation des artefacts

/// Identifiants connus de l'index : seuls ceux-là sont gardés dans `sources` et les champs liés.
fn connus(index: &Index) -> Result<BTreeSet<String>> {
    let mut st = index
        .conn
        .prepare("SELECT id FROM documents UNION SELECT id FROM symboles")?;
    let v = st
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

fn ids_reels(connus: &BTreeSet<String>, v: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut vus = BTreeSet::new();
    v.into_iter()
        .filter(|x| connus.contains(x) && vus.insert(x.clone()))
        .collect()
}

fn objet(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

fn tronquer(t: &str, n: usize) -> String {
    let t = t.trim();
    if t.chars().count() > n {
        format!("{}…", t.chars().take(n).collect::<String>().trim_end())
    } else {
        t.to_string()
    }
}

/// Crée ou complète l'artefact d'un élément gardé (ou saisi) ; renvoie son identifiant.
fn materialiser(
    index: &Index,
    fiche: &Fiche,
    se: &SousEtape,
    cible: Option<&str>,
    d: &Value,
) -> Result<Option<String>> {
    let connus = connus(index)?;
    let racine = &index.racine;
    let config = &index.config;
    let sources = |extra: &[&str]| -> Vec<String> {
        let mut v: Vec<String> = cible.map(str::to_string).into_iter().collect();
        v.extend(extra.iter().map(|s| texte(d, s)).filter(|s| !s.is_empty()));
        v.extend(chaines(d.get("sources")));
        ids_reels(&connus, v)
    };
    let base = |titre: &str| -> Map<String, Value> {
        objet(json!({ "titre": titre, "statut": "retenu", "technique": fiche.id }))
    };
    if let Some(complete) = se.complete.as_deref() {
        let id = cible.context("cible attendue")?;
        return completer(index, complete, id, &se.id, d).map(Some);
    }
    if se.ajoute_a.as_deref() == Some("carte") {
        let mut p = charger_parcours(index);
        let liens = p.entry("liens_carte").or_insert_with(|| json!([]));
        if let Some(a) = liens.as_array_mut() {
            a.push(json!({ "de": texte(d, "de"), "vers": texte(d, "vers"), "raison": texte(d, "raison") }));
        }
        sauver_parcours(index, &p)?;
        return Ok(None);
    }
    if fiche.id == "roadmap" && se.id == "horizons" {
        let mut p = charger_parcours(index);
        p.insert("horizons".into(), d.clone());
        sauver_parcours(index, &p)?;
        return Ok(None);
    }
    let Some(ty) = se.cree.as_deref() else {
        return Ok(None);
    };
    let (id, _) = match ty {
        "axe" => artefacts::creer(
            racine,
            config,
            "axe",
            base(&texte(d, "titre")),
            &format!("# {}\n\n{}", texte(d, "titre"), texte(d, "description")),
        )?,
        "unite" => {
            let mut fm = base(&tronquer(&texte(d, "verbatim"), 60));
            fm.insert("qualification".into(), json!(texte(d, "qualification")));
            let axe = cible
                .and_then(|c| documents(index).ok()?.into_iter().find(|x| x.id == c))
                .map(|e| texte(&e.fm, "axe"))
                .unwrap_or_default();
            if connus.contains(&axe) {
                fm.insert("axe".into(), json!(axe));
            }
            fm.insert("sources".into(), json!(sources(&[])));
            artefacts::creer(racine, config, "unite", fm, &texte(d, "verbatim"))?
        }
        "idee" => {
            let mut fm = base(&texte(d, "titre"));
            for k in ["levier", "principe"] {
                if !texte(d, k).is_empty() {
                    fm.insert(k.into(), json!(texte(d, k)));
                }
            }
            if connus.contains(&texte(d, "parent")) {
                fm.insert("parent".into(), json!(texte(d, "parent")));
            }
            fm.insert("sources".into(), json!(sources(&["parent"])));
            artefacts::creer(
                racine,
                config,
                "idee",
                fm,
                &format!("# {}\n\n{}", texte(d, "titre"), texte(d, "description")),
            )?
        }
        "probleme" => {
            let mut fm = base(&texte(d, "titre"));
            fm.insert("sources".into(), json!(sources(&[])));
            artefacts::creer(
                racine,
                config,
                "probleme",
                fm,
                &format!("# {}\n\n{}", texte(d, "titre"), texte(d, "description")),
            )?
        }
        "analogie" => {
            let mut fm = base(&texte(d, "analogie"));
            fm.insert("domaine".into(), json!(texte(d, "domaine")));
            fm.insert(
                "principes".into(),
                d.get("principes").cloned().unwrap_or(json!([])),
            );
            if let Some(c) = cible {
                fm.insert("probleme".into(), json!(c));
            }
            fm.insert("sources".into(), json!(sources(&[])));
            let principes: Vec<String> = chaines(d.get("principes"))
                .iter()
                .map(|p| format!("- {p}"))
                .collect();
            let corps = format!(
                "# {}\n\n**Domaine** : {}\n\n## Principes\n\n{}",
                texte(d, "analogie"),
                texte(d, "domaine"),
                principes.join("\n")
            );
            artefacts::creer(racine, config, "analogie", fm, &corps)?
        }
        "analyse" => {
            let methode = se.methode.clone().unwrap_or_default();
            let sujet = cible.unwrap_or_default();
            let titre_sujet = documents(index)?
                .into_iter()
                .find(|x| x.id == sujet)
                .map(|x| x.titre)
                .unwrap_or_default();
            let (nom, cles): (&str, &[(&str, &str)]) = if methode == "cubing" {
                (
                    "Cubing",
                    &[
                        ("decrire", "Décrire"),
                        ("comparer", "Comparer"),
                        ("associer", "Associer"),
                        ("analyser", "Analyser"),
                        ("appliquer", "Appliquer"),
                        ("argumenter", "Argumenter"),
                    ],
                )
            } else {
                (
                    "5W1H",
                    &[
                        ("quoi", "Quoi ?"),
                        ("qui", "Qui ?"),
                        ("ou", "Où ?"),
                        ("quand", "Quand ?"),
                        ("pourquoi", "Pourquoi ?"),
                        ("comment", "Comment ?"),
                    ],
                )
            };
            let mut fm = base(&format!("{nom} : {titre_sujet}"));
            fm.insert("methode".into(), json!(methode));
            fm.insert("sujet".into(), json!(sujet));
            fm.insert("sources".into(), json!(sources(&[])));
            let mut corps = format!("# {nom} : {titre_sujet}\n\n**Sujet** : {sujet}\n");
            for (k, t) in cles {
                corps.push_str(&format!("\n## {t}\n\n{}\n", texte(d, k)));
            }
            artefacts::creer(racine, config, "analyse", fm, &corps)?
        }
        "storyboard" => {
            let mut fm = base(&texte(d, "titre"));
            fm.insert("statut".into(), json!("brouillon"));
            fm.insert("scenario".into(), json!(texte(d, "scenario")));
            fm.insert("vignettes".into(), json!([]));
            fm.insert("artefacts".into(), json!([]));
            fm.insert("sources".into(), json!(sources(&[])));
            artefacts::creer(
                racine,
                config,
                "storyboard",
                fm,
                &format!(
                    "# {}\n\n## Scénario\n\n{}",
                    texte(d, "titre"),
                    texte(d, "scenario")
                ),
            )?
        }
        "besoin" => {
            let srcs = sources(&[]);
            let version = config.atelier.version.clone();
            artefacts::creer_avec(racine, config, "besoin", |id| {
                let fm = objet(json!({
                    "titre": texte(d, "titre"), "statut": "proposé", "version": version, "technique": fiche.id,
                    "categorie": texte(d, "categorie"), "priorite": texte(d, "priorite"), "sources": srcs,
                }));
                (
                    fm,
                    format!(
                        "# {id} : {}\n\n**Besoin** : {}\n",
                        texte(d, "titre"),
                        texte(d, "description")
                    ),
                )
            })?
        }
        "exigence" => {
            let srcs = sources(&[]);
            let besoins: Vec<String> = cible
                .map(str::to_string)
                .into_iter()
                .filter(|c| connus.contains(c))
                .collect();
            let version = config.atelier.version.clone();
            artefacts::creer_avec(racine, config, "exigence", |id| {
                let fm = objet(json!({
                    "titre": texte(d, "titre"), "statut": "proposé", "priorite": texte(d, "priorite"), "version": version,
                    "technique": fiche.id, "besoins": besoins, "sources": srcs,
                }));
                (
                    fm,
                    format!(
                        "# {id} : {}\n\n**Énoncé** : {}\n\n**Critère d'acceptation** : {}\n",
                        texte(d, "titre"),
                        texte(d, "enonce"),
                        texte(d, "critere")
                    ),
                )
            })?
        }
        "ecrit" => {
            let mut fm = base(&format!("Écriture libre : {}", cible.unwrap_or("axe")));
            if let Some(c) = cible {
                fm.insert("axe".into(), json!(c));
                fm.insert("sources".into(), json!([c]));
            }
            fm.insert(
                "duree_minutes".into(),
                d.get("duree_minutes").cloned().unwrap_or(json!(10)),
            );
            artefacts::creer(racine, config, "ecrit", fm, &texte(d, "texte"))?
        }
        "pitch" => {
            let mut fm = base(&texte(d, "titre"));
            for k in [
                "description",
                "probleme",
                "cible",
                "valeur",
                "contraintes",
                "criteres_succes",
                "hors_sujet",
            ] {
                fm.insert(k.into(), json!(texte(d, k)));
            }
            let sections = [
                ("Problème", "probleme"),
                ("Pour qui", "cible"),
                ("Valeur", "valeur"),
                ("Contraintes", "contraintes"),
                ("Critères de succès", "criteres_succes"),
                ("Hors sujet", "hors_sujet"),
            ];
            let mut corps = format!("# {}\n", texte(d, "titre"));
            for (t, k) in sections {
                corps.push_str(&format!("\n## {t}\n\n{}\n", texte(d, k)));
            }
            artefacts::creer(racine, config, "pitch", fm, &corps)?
        }
        autre => bail!("type d'artefact non géré par l'atelier : {autre}"),
    };
    Ok(Some(id))
}

/// Complète un artefact existant : cadre, vignettes et artefacts d'un storyboard ; pistes d'une clarification.
fn completer(index: &Index, ty: &str, id: &str, sous: &str, d: &Value) -> Result<String> {
    let doc = documents(index)?
        .into_iter()
        .find(|x| x.id == id)
        .with_context(|| format!("{id} introuvable"))?;
    if doc.genre != ty {
        bail!("{id} n'est pas un(e) {ty}");
    }
    let (mut fm, corps) = artefacts::lire(&index.racine, &doc.chemin)?;
    match (ty, sous) {
        ("storyboard", "cadre") => {
            for k in ["personnage", "scene", "intrigue"] {
                fm.insert(k.into(), json!(texte(d, k)));
            }
        }
        ("storyboard", "vignettes") => {
            let v = fm.entry("vignettes").or_insert_with(|| json!([]));
            let n = v.as_array().map_or(0, Vec::len) + 1;
            v.as_array_mut().unwrap().push(json!({ "id": format!("V{n}"), "titre": texte(d, "titre"), "description": texte(d, "description"), "prompt": texte(d, "prompt") }));
        }
        ("storyboard", "artefacts") => {
            let v = fm.entry("artefacts").or_insert_with(|| json!([]));
            let n = v.as_array().map_or(0, Vec::len) + 1;
            v.as_array_mut().unwrap().push(json!({ "id": format!("{id}-A{n}"), "nom": texte(d, "nom"), "genre": texte(d, "genre"), "vignette": texte(d, "vignette"), "description": texte(d, "description") }));
        }
        ("clarification", "reponse") => {
            let v = fm.entry("pistes").or_insert_with(|| json!([]));
            v.as_array_mut().unwrap().push(json!({ "piste": texte(d, "piste"), "justification": texte(d, "justification"), "sources": chaines(d.get("sources")) }));
        }
        _ => bail!("complément inconnu : {ty}/{sous}"),
    }
    let corps = if ty == "storyboard" {
        corps_storyboard(&fm)
    } else {
        corps_clarification(&fm, &corps)
    };
    artefacts::reecrire(&index.racine, &doc.chemin, &fm, &corps)?;
    Ok(id.to_string())
}

fn corps_storyboard(fm: &Map<String, Value>) -> String {
    let t = |k: &str| {
        fm.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let mut c = format!("# {}\n\n## Scénario\n\n{}\n", t("titre"), t("scenario"));
    if !t("personnage").is_empty() {
        c.push_str(&format!("\n## Personnage, scène, intrigue\n\n- **Personnage** : {}\n- **Scène** : {}\n- **Intrigue** : {}\n", t("personnage"), t("scene"), t("intrigue")));
    }
    let vignettes = fm
        .get("vignettes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !vignettes.is_empty() {
        c.push_str("\n## Vignettes\n\n| # | Vignette | Ce qui se passe | Prompt d'illustration |\n|---|---|---|---|\n");
        for v in &vignettes {
            c.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                texte(v, "id"),
                texte(v, "titre"),
                texte(v, "description"),
                texte(v, "prompt")
            ));
        }
        c.push_str("\n## Récit\n\n");
        for v in &vignettes {
            c.push_str(&format!(
                "- **{}** : {}\n",
                texte(v, "titre"),
                texte(v, "description")
            ));
        }
    }
    let art = fm
        .get("artefacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !art.is_empty() {
        c.push_str("\n## Artefacts à produire\n\n| Id | Artefact | Genre | Vignette source | Description |\n|---|---|---|---|---|\n");
        for a in &art {
            c.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                texte(a, "id"),
                texte(a, "nom"),
                texte(a, "genre"),
                texte(a, "vignette"),
                texte(a, "description")
            ));
        }
    }
    c
}

fn corps_clarification(fm: &Map<String, Value>, ancien: &str) -> String {
    let entete = ancien
        .split("\n## Pistes")
        .next()
        .unwrap_or(ancien)
        .trim_end()
        .to_string();
    let mut c = entete;
    let pistes = fm
        .get("pistes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if !pistes.is_empty() {
        c.push_str("\n\n## Pistes\n");
        for p in &pistes {
            c.push_str(&format!(
                "\n- **{}** — {} (sources : {})\n",
                texte(p, "piste"),
                texte(p, "justification"),
                chaines(p.get("sources")).join(", ")
            ));
        }
    }
    if let Some(conclusion) = fm.get("conclusion").and_then(Value::as_str) {
        c.push_str(&format!("\n## Conclusion\n\n{conclusion}\n"));
    }
    c
}

// ---------------------------------------------------------------- saisies

/// Enregistre une saisie de Tibo : pitch, texte libre, tri des besoins, conclusion d'une clarification.
pub fn saisir(
    index: &Index,
    etape: &str,
    sous: &str,
    cible: Option<&str>,
    d: &Value,
) -> Result<Option<String>> {
    let fiche = fiches::fiche(etape)?;
    let se = fiche.sous_etape(sous)?;
    if se.genre != "saisie" {
        bail!("{etape}/{sous} n'est pas une saisie");
    }
    let docs = documents(index)?;
    verifier_cible(index, &docs, fiche, se, cible)?;
    let id = match (etape, sous) {
        ("pitch", "pitch") => materialiser(index, fiche, se, None, d)?,
        ("free-writing", "ecriture") => {
            if texte(d, "texte").is_empty() {
                bail!("le texte libre est vide");
            }
            materialiser(index, fiche, se, cible, d)?
        }
        ("bilan", "tri") => {
            for b in d
                .get("besoins")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let id = texte(b, "id");
                let doc = docs
                    .iter()
                    .find(|x| x.id == id && x.genre == "besoin")
                    .with_context(|| format!("besoin inconnu : {id}"))?;
                let (mut fm, corps) = artefacts::lire(&index.racine, &doc.chemin)?;
                for k in ["categorie", "priorite"] {
                    if !texte(b, k).is_empty() {
                        fm.insert(k.into(), json!(texte(b, k)));
                    }
                }
                if let Some(o) = b.get("ordre") {
                    fm.insert("ordre".into(), o.clone());
                }
                artefacts::reecrire(&index.racine, &doc.chemin, &fm, &corps)?;
            }
            None
        }
        ("clarification", "conclusion") => {
            let id = cible.context("clarification attendue")?;
            let doc = docs
                .iter()
                .find(|x| x.id == id)
                .context("clarification introuvable")?;
            let (mut fm, corps) = artefacts::lire(&index.racine, &doc.chemin)?;
            fm.insert("conclusion".into(), json!(texte(d, "conclusion")));
            fm.insert("etat".into(), json!("close"));
            fm.insert("statut".into(), json!("retenu"));
            let corps = corps_clarification(&fm, &corps);
            artefacts::reecrire(&index.racine, &doc.chemin, &fm, &corps)?;
            Some(id.to_string())
        }
        _ => bail!("saisie non gérée : {etape}/{sous}"),
    };
    let mut p = charger_parcours(index);
    if fiche.hors_parcours {
        return Ok(id);
    }
    marquer(&mut p, etape, "en_cours");
    sauver_parcours(index, &p)?;
    Ok(id)
}

// ---------------------------------------------------------------- synthèses

fn ecrire_genere(index: &Index, relatif: &str, fm: Map<String, Value>, corps: &str) -> Result<()> {
    let chemin = index.racine.join(relatif);
    if chemin.exists() {
        let (ancien, _) = artefacts::lire(&index.racine, relatif)?;
        if ancien.get("genere_par").and_then(Value::as_str) != Some("atelier") {
            bail!("{relatif} existe et n'a pas été produit par l'atelier : il n'est pas réécrit");
        }
    }
    let mut fm = fm;
    fm.insert("genere_par".into(), json!("atelier"));
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(chemin, artefacts::formater(&fm, corps))?;
    Ok(())
}

/// Assemble l'artefact de synthèse d'une sous-étape ; renvoie les fichiers écrits.
pub fn synthetiser(index: &Index, etape: &str, sous: &str) -> Result<Vec<String>> {
    let fiche = fiches::fiche(etape)?;
    let se = fiche.sous_etape(sous)?;
    if se.genre != "synthese" {
        bail!("{etape}/{sous} n'est pas une synthèse");
    }
    let docs = documents(index)?;
    let p = charger_parcours(index);
    let de = |genre: &str| {
        docs.iter()
            .filter(move |d| d.genre == genre && d.statut != "ecarte")
            .collect::<Vec<_>>()
    };
    let titre_projet = docs
        .iter()
        .find(|d| d.genre == "pitch")
        .map(|d| d.titre.clone())
        .unwrap_or_else(|| "Projet".into());
    let version = index.config.atelier.version.clone();
    let majeure = format!(
        "{}/majeures/{version}",
        index.config.projet.dossier.trim_end_matches('/')
    );
    let mut ecrits = vec![];
    match se.cree.as_deref() {
        Some("carte") => {
            let net = |t: &str| t.replace(['(', ')', '[', ']', '{', '}'], " ");
            let mut m = format!("```mermaid\nmindmap\n  root(({}))\n", net(&titre_projet));
            let mut noeuds = vec![];
            for axe in de("axe") {
                m.push_str(&format!("    {}\n", net(&axe.titre)));
                noeuds.push(axe.id.clone());
                for u in de("unite")
                    .into_iter()
                    .filter(|u| texte(&u.fm, "axe") == axe.id)
                {
                    m.push_str(&format!("      {}\n", net(&u.titre)));
                    noeuds.push(u.id.clone());
                    for i in de("idee")
                        .into_iter()
                        .filter(|i| texte(&i.fm, "parent") == u.id)
                    {
                        m.push_str(&format!("        {}\n", net(&i.titre)));
                        noeuds.push(i.id.clone());
                    }
                }
            }
            m.push_str("```\n");
            let liens = p
                .get("liens_carte")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if !liens.is_empty() {
                m.push_str("\n## Liens transversaux\n\n");
                for l in &liens {
                    m.push_str(&format!(
                        "- {} ↔ {} : {}\n",
                        texte(l, "de"),
                        texte(l, "vers"),
                        texte(l, "raison")
                    ));
                }
            }
            let fm = objet(
                json!({ "id": "CARTE", "type": "carte", "titre": format!("Carte mentale : {titre_projet}"), "statut": "retenu", "technique": fiche.id, "sources": noeuds }),
            );
            let relatif =
                artefacts::chemin_relatif(&index.config, artefacts::type_de("carte")?, "CARTE");
            std::fs::create_dir_all(index.racine.join(&relatif).parent().unwrap())?;
            artefacts::reecrire(
                &index.racine,
                &relatif,
                &fm,
                &format!("# Carte mentale : {titre_projet}\n\n{m}"),
            )?;
            ecrits.push(relatif);
        }
        Some("scamper") => {
            let sel = fiche
                .sous_etapes
                .iter()
                .find(|s| s.genre == "selection")
                .map(|s| selection(&p, &docs, fiche, s))
                .unwrap_or_default();
            for sujet in sel {
                let titre_sujet = docs
                    .iter()
                    .find(|d| d.id == sujet)
                    .map(|d| d.titre.clone())
                    .unwrap_or_default();
                let idees: Vec<&Doc> = de("idee")
                    .into_iter()
                    .filter(|i| {
                        texte(&i.fm, "technique") == "scamper"
                            && chaines(i.fm.get("sources")).contains(&sujet)
                    })
                    .collect();
                let mut corps =
                    format!("# Analyse SCAMPER : {titre_sujet}\n\n**Sujet** : {sujet}\n");
                for (l, nom) in [
                    ("S", "Substituer"),
                    ("C", "Combiner"),
                    ("A", "Adapter"),
                    ("M", "Modifier"),
                    ("P", "Proposer un autre usage"),
                    ("E", "Éliminer"),
                    ("R", "Renverser / Réorganiser"),
                ] {
                    corps.push_str(&format!("\n## {l} - {nom}\n\n"));
                    for i in idees.iter().filter(|i| texte(&i.fm, "levier") == l) {
                        corps.push_str(&format!("- [[{}]] {}\n", i.id, i.titre));
                    }
                }
                let mut ids: Vec<String> = vec![sujet.clone()];
                ids.extend(idees.iter().map(|i| i.id.clone()));
                let existant = de("scamper")
                    .into_iter()
                    .find(|s| texte(&s.fm, "sujet") == sujet)
                    .map(|s| s.chemin.clone());
                let fm = objet(
                    json!({ "titre": format!("SCAMPER : {titre_sujet}"), "statut": "retenu", "technique": fiche.id, "sujet": sujet, "sources": ids }),
                );
                let relatif = match existant {
                    Some(c) => {
                        let (ancien, _) = artefacts::lire(&index.racine, &c)?;
                        let mut fm2 = fm.clone();
                        fm2.insert("id".into(), ancien.get("id").cloned().unwrap_or_default());
                        fm2.insert("type".into(), json!("scamper"));
                        artefacts::reecrire(&index.racine, &c, &fm2, &corps)?;
                        c
                    }
                    None => {
                        artefacts::creer(&index.racine, &index.config, "scamper", fm, &corps)?.1
                    }
                };
                ecrits.push(relatif);
            }
        }
        Some("storyboard") => {
            for sb in de("storyboard") {
                let (mut fm, _) = artefacts::lire(&index.racine, &sb.chemin)?;
                fm.insert("statut".into(), json!("retenu"));
                let corps = corps_storyboard(&fm);
                artefacts::reecrire(&index.racine, &sb.chemin, &fm, &corps)?;
                ecrits.push(sb.chemin.clone());
            }
        }
        Some("bdr") => {
            let pitch = docs
                .iter()
                .find(|d| d.genre == "pitch")
                .context("le pitch n'est pas écrit")?;
            let t = |k: &str| texte(&pitch.fm, k);
            let bdr1 = format!(
                "# BDR {version}, partie 1 : analyse du besoin\n\n> Produite par l'atelier à partir du pitch ([[PITCH]]).\n\n## Contexte\n\n{}\n\n## Personnes concernées\n\n{}\n\n## Situation actuelle et irritants\n\n{}\n\n## Contraintes\n\n{}\n\n## Hors sujet\n\n{}\n",
                t("valeur"),
                t("cible"),
                t("probleme"),
                t("contraintes"),
                t("hors_sujet")
            );
            let r1 = format!("{majeure}/bdr-1-analyse.md");
            ecrire_genere(
                index,
                &r1,
                objet(
                    json!({ "id": format!("BDR-{version}-1"), "type": "bdr-analyse", "titre": format!("Analyse du besoin {version} : {titre_projet}"), "statut": "proposé", "version": version, "sources": ["PITCH"] }),
                ),
                &bdr1,
            )?;
            let mut besoins: Vec<&Doc> = de("besoin");
            let rang = |b: &Doc| {
                ["must", "should", "could", "wont"]
                    .iter()
                    .position(|x| *x == texte(&b.fm, "priorite"))
                    .unwrap_or(4)
            };
            besoins.sort_by_key(|b| {
                (
                    rang(b),
                    b.fm.get("ordre")
                        .and_then(Value::as_i64)
                        .unwrap_or(i64::MAX),
                    b.id.clone(),
                )
            });
            let mut table =
                String::from("| ID | Besoin | Catégorie | Priorité |\n|---|---|---|---|\n");
            for b in &besoins {
                table.push_str(&format!(
                    "| [[{}]] | {} | {} | {} |\n",
                    b.id,
                    b.titre,
                    texte(&b.fm, "categorie"),
                    texte(&b.fm, "priorite")
                ));
            }
            let bdr2 = format!(
                "# BDR {version}, partie 2 : définition du besoin\n\n## Objectifs mesurables\n\n{}\n\n## Besoins\n\n{table}\n## Décision de lancement\n\nÀ prendre par le responsable produit.\n",
                t("criteres_succes")
            );
            let r2 = format!("{majeure}/bdr-2-definition.md");
            ecrire_genere(
                index,
                &r2,
                objet(
                    json!({ "id": format!("BDR-{version}-2"), "type": "bdr-definition", "titre": format!("Définition du besoin {version} : {titre_projet}"), "statut": "proposé", "version": version, "sources": [format!("BDR-{version}-1")] }),
                ),
                &bdr2,
            )?;
            ecrits.extend([r1, r2]);
        }
        Some("cahier-des-charges") => {
            let mut table =
                String::from("| ID | Exigence | Priorité | Besoins |\n|---|---|---|---|\n");
            for e in de("exigence")
                .into_iter()
                .filter(|e| texte(&e.fm, "technique") == "exigences")
            {
                table.push_str(&format!(
                    "| [[{}]] | {} | {} | {} |\n",
                    e.id,
                    e.titre,
                    texte(&e.fm, "priorite"),
                    chaines(e.fm.get("besoins")).join(", ")
                ));
            }
            let r = format!("{majeure}/cahier-des-charges.md");
            let corps = format!(
                "# Cahier des charges {version} : {titre_projet}\n\n> Produit par l'atelier. Chaque exigence a son fichier dans `docs/projet/exigences/`.\n\n## Exigences fonctionnelles\n\n{table}"
            );
            ecrire_genere(
                index,
                &r,
                objet(
                    json!({ "id": format!("CDC-{version}"), "type": "cahier-des-charges", "titre": format!("Cahier des charges {version} : {titre_projet}"), "statut": "proposé", "version": version, "sources": [format!("BDR-{version}-2")] }),
                ),
                &corps,
            )?;
            ecrits.push(r);
        }
        Some("roadmap") => {
            let h = p
                .get("horizons")
                .filter(|v| !v.is_null())
                .context("aucune répartition en horizons n'a été gardée")?;
            let titres: BTreeMap<String, String> = de("besoin")
                .into_iter()
                .map(|b| (b.id.clone(), b.titre.clone()))
                .collect();
            let mut corps = format!("# Roadmap {version} : {titre_projet}\n");
            let mut tous = vec![];
            for (cle, nom) in [
                (
                    "maintenant",
                    "Horizon 1 : Maintenant (MVP / prochain cycle)",
                ),
                ("ensuite", "Horizon 2 : Ensuite (après le lancement)"),
                ("plus_tard", "Horizon 3 : Plus tard (backlog stratégique)"),
            ] {
                let hz = h.get(cle).cloned().unwrap_or_default();
                corps.push_str(&format!(
                    "\n## {nom}\n\n**Objectif stratégique** : {}\n\n{}\n\n",
                    texte(&hz, "objectif"),
                    texte(&hz, "recit")
                ));
                for b in chaines(hz.get("besoins"))
                    .into_iter()
                    .filter(|b| titres.contains_key(b))
                {
                    corps.push_str(&format!("- [[{b}]] {}\n", titres[&b]));
                    tous.push(b);
                }
            }
            let r = format!("{majeure}/roadmap.md");
            ecrire_genere(
                index,
                &r,
                objet(
                    json!({ "id": format!("ROADMAP-{version}"), "type": "roadmap", "titre": format!("Roadmap {version} : {titre_projet}"), "statut": "proposé", "version": version, "besoins": tous }),
                ),
                &corps,
            )?;
            ecrits.push(r);
        }
        autre => bail!("synthèse non gérée : {}", autre.unwrap_or("aucune")),
    }
    Ok(ecrits)
}

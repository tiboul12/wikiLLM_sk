//! Atelier de définition du besoin : artefacts en fichiers, fiches de technique, parcours, fournisseur LLM.
//!
//! Tout ce qui est produit est un fichier du dépôt (ADR-0010) ; l'atelier n'a pas de base à lui.
//!
//! @req EF-105, EF-110
//! @adr 10

pub mod artefacts;
pub mod connecteurs;
pub mod fiches;
pub mod fournisseur;
pub mod parcours;

use crate::index::Index;
use anyhow::{Context, Result, bail};
use rusqlite::OptionalExtension;
use serde_json::json;

/// Ouvre une clarification sur un identifiant existant (besoin, exigence, artefact ou symbole) et renvoie
/// (identifiant de la clarification, adresse de son écran) (SPEC-21 R3). Quand le serveur écoute sur le
/// réseau, l'adresse porte l'IP de la machine : Tibo l'ouvre depuis son poste ; elle porte le port
/// réellement pris par le serveur du projet (plusieurs projets ouverts).
///
/// Sans question (bouton « clarifier » d'un document, EF-144), la question par défaut porte sur ce qui
/// reste flou dans le sujet, et une clarification encore ouverte sur le même sujet est reprise au lieu
/// d'en créer une autre.
///
/// @req EF-110, EF-144
/// @relu 2026-10-22 956e2351
pub fn rappel(index: &Index, sujet: &str, question: &str) -> Result<(String, String)> {
    let c = &index.conn;
    let existe: bool = c
        .query_row(
            "SELECT 1 FROM documents WHERE id = ?1 UNION SELECT 1 FROM symboles WHERE id = ?1",
            [sujet],
            |_| Ok(true),
        )
        .optional()?
        .unwrap_or(false);
    if !existe {
        bail!(
            "sujet inconnu : {sujet} (identifiant de besoin, d'exigence, d'artefact ou de symbole attendu)"
        );
    }
    let adresse = |id: &str| {
        let serveur =
            crate::reseau::pour_le_reseau(&crate::reseau::adresse_serveur(index), &index.config);
        format!("{}/clarifications/{id}", serveur.trim_end_matches('/'))
    };
    let question = if question.trim().is_empty() {
        if let Some(id) = clarifications(index)?
            .into_iter()
            .find(|c| c.sujet == sujet && c.etat != "close")
            .map(|c| c.id)
        {
            let url = adresse(&id);
            return Ok((id, url));
        }
        let titre: String = c
            .query_row("SELECT titre FROM documents WHERE id = ?1", [sujet], |r| {
                r.get(0)
            })
            .optional()?
            .unwrap_or_default();
        format!("Qu'est-ce qui reste flou, incomplet ou contradictoire dans {sujet} « {titre} » ?")
    } else {
        question.trim().to_string()
    };
    let question = question.as_str();
    let fm = json!({
        "titre": format!("Clarification de {sujet}"),
        "statut": "brouillon",
        "etat": "ouverte",
        "sujet": sujet,
        "question": question.trim(),
        "sources": [sujet],
    });
    let (id, _) = artefacts::creer(
        &index.racine,
        &index.config,
        "clarification",
        fm.as_object().unwrap().clone(),
        &format!(
            "# Clarification de {sujet}\n\n**Question** : {}\n",
            question.trim()
        ),
    )?;
    let url = adresse(&id);
    Ok((id, url))
}

/// Une clarification, pour la regrouper par sujet.
struct Clar {
    id: String,
    sujet: String,
    etat: String,
    chemin: String,
}

/// Clarifications du projet, par identifiant, frontmatter décodé.
fn clarifications(index: &Index) -> Result<Vec<Clar>> {
    let mut st = index.conn.prepare(
        "SELECT id, chemin, frontmatter FROM documents WHERE genre = 'clarification' ORDER BY id",
    )?;
    let v = st
        .query_map([], |r| {
            let fm: String = r.get(2)?;
            let fm: serde_json::Value = serde_json::from_str(&fm).unwrap_or_default();
            let champ = |k: &str| {
                fm.get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string()
            };
            Ok(Clar {
                id: r.get(0)?,
                chemin: r.get(1)?,
                sujet: champ("sujet"),
                etat: champ("etat"),
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(v)
}

/// Clarification par identifiant ; erreur claire si elle n'existe pas.
fn clarification(index: &Index, id: &str) -> Result<Clar> {
    clarifications(index)?
        .into_iter()
        .find(|c| c.id == id)
        .with_context(|| format!("clarification inconnue : {id}"))
}

/// Déclare une clarification finie (EF-144) : les pistes gardées (corrigées au besoin) font la réponse,
/// écrite en conclusion ; `etat: close`. Sans piste gardée, rien n'est clos.
pub fn clore(index: &Index, id: &str) -> Result<()> {
    let c = clarification(index, id)?;
    let (mut fm, corps) = artefacts::lire(&index.racine, &c.chemin)?;
    let pistes: Vec<String> = fm
        .get("pistes")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .filter_map(|p| p.get("piste").and_then(|v| v.as_str()))
        .map(|p| format!("- {p}"))
        .collect();
    if pistes.is_empty() {
        bail!("garde ou corrige au moins une piste : c'est elle qui fait la réponse");
    }
    fm.insert("conclusion".into(), json!(pistes.join("\n")));
    fm.insert("etat".into(), json!("close"));
    fm.insert("statut".into(), json!("retenu"));
    let corps = parcours::corps_clarification(&fm, &corps);
    artefacts::reecrire(&index.racine, &c.chemin, &fm, &corps)
}

/// Rouvre une clarification close : la conclusion reste jusqu'à la prochaine clôture.
pub fn rouvrir(index: &Index, id: &str) -> Result<()> {
    let c = clarification(index, id)?;
    let (mut fm, corps) = artefacts::lire(&index.racine, &c.chemin)?;
    fm.insert("etat".into(), json!("ouverte"));
    fm.insert("statut".into(), json!("brouillon"));
    artefacts::reecrire(&index.racine, &c.chemin, &fm, &corps)
}

/// Supprime une clarification (REC-032) : son fichier et ses propositions en attente.
/// @relu 2026-10-22 d7754c38
pub fn supprimer(index: &Index, id: &str) -> Result<()> {
    let c = clarification(index, id)?;
    crate::journal::supprimer(index.racine.join(&c.chemin))?;
    let lot = parcours::travail(&index.racine)
        .join("propositions")
        .join(format!("clarification__reponse__{id}.json"));
    if lot.exists() {
        crate::journal::supprimer(lot)?;
    }
    Ok(())
}

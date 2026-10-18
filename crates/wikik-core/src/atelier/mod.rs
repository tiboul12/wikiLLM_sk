//! Atelier de définition du besoin : artefacts en fichiers, fiches de technique, parcours, fournisseur LLM.
//!
//! Tout ce qui est produit est un fichier du dépôt (ADR-0010) ; l'atelier n'a pas de base à lui.
//!
//! @req EF-105, EF-110
//! @adr 10

pub mod artefacts;
pub mod fiches;
pub mod fournisseur;
pub mod parcours;

use crate::index::Index;
use anyhow::{Result, bail};
use rusqlite::OptionalExtension;
use serde_json::json;

/// Ouvre une clarification sur un identifiant existant (besoin, exigence, artefact ou symbole) et renvoie
/// (identifiant de la clarification, adresse de son écran dans l'atelier) (SPEC-21 R3). Quand les serveurs
/// écoutent sur le réseau, l'adresse porte l'IP de la machine : Tibo l'ouvre depuis son poste.
///
/// @req EF-110
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
    if question.trim().is_empty() {
        bail!("la question est vide");
    }
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
    let atelier = crate::reseau::pour_le_reseau(&index.config.atelier.url, &index.config);
    let url = format!("{}/clarification/{id}", atelier.trim_end_matches('/'));
    Ok((id, url))
}

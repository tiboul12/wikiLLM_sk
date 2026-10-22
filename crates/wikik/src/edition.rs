//! Routes d'édition (ADR-0016) : documents modifiés dans le navigateur (EF-158), annuler et rétablir
//! (EF-159), favoris (EF-157). L'édition et l'annulation écrivent dans le projet : elles sont réservées au
//! propriétaire (ADR-0013). Une couche ouvre une action du journal à chaque requête d'écriture, pour que
//! Ctrl+Z défasse exactement ce geste.
//!
//! @req EF-157, EF-158, EF-159
//! @adr 13
//! @adr 16

use crate::acces;
use crate::atelier::resultat;
use crate::serve::Partage;
use anyhow::Result;
use axum::extract::{Query, Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wikik_core::{edition, journal};

/// Routes d'édition ; tout ce qui écrit dans le dépôt passe par la garde du propriétaire.
pub(crate) fn routes(etat: Partage) -> Router<Partage> {
    let reservees = Router::new()
        .route("/api/document/source", get(api_source))
        .route("/api/document/ecrire", post(api_ecrire))
        .route("/api/document/statut", post(api_statut))
        .route("/api/annuler", post(api_annuler))
        .route("/api/retablir", post(api_retablir))
        .route_layer(axum::middleware::from_fn_with_state(etat, acces::reserve));
    Router::new()
        .route("/api/journal", get(api_journal))
        .route("/api/favoris", get(api_favoris).post(api_ecrire_favoris))
        .merge(reservees)
}

/// Ouvre une action du journal pour chaque requête d'écriture (sauf annuler, rétablir et ce qui n'écrit
/// pas dans le projet : terminal, arrêt du LLM ou du serveur, test de connexion).
pub(crate) async fn action(requete: Request, suite: Next) -> Response {
    let chemin = requete.uri().path().to_string();
    let hors_journal = [
        "/api/annuler",
        "/api/retablir",
        "/api/llm/annuler",
        "/api/terminal/envoyer",
        "/api/serveur/arreter",
        "/api/parametres/tester",
        "/api/recherche/ia",
        "/api/favoris",
    ];
    if requete.method() == Method::POST
        && chemin.starts_with("/api/")
        && !hors_journal.contains(&chemin.as_str())
    {
        journal::nouvelle_action(libelle(&chemin));
    }
    suite.run(requete).await
}

/// Nom d'un geste, montré après Ctrl+Z (« Annulé : décision sur une proposition »).
fn libelle(chemin: &str) -> &'static str {
    match chemin {
        "/api/document/ecrire" => "modification du document",
        "/api/document/statut" => "changement de statut",
        "/api/atelier/decider" => "décision sur une proposition",
        "/api/atelier/saisir" => "saisie de l'atelier",
        "/api/atelier/selection" => "sélection",
        "/api/atelier/synthetiser" => "synthèse",
        "/api/atelier/proposer" => "propositions du LLM",
        "/api/atelier/parcours" => "parcours",
        "/api/atelier/clarification" => "clarification",
        "/api/atelier/piloter" => "étape de l'atelier",
        "/api/reception/remarque" => "remarque",
        "/api/reception/traiter" => "traitement d'une remarque",
        "/api/reception/supprimer" => "suppression d'une remarque",
        "/api/reception/page" => "carte des pages",
        "/api/parametres" => "paramètres",
        "/api/skills" | "/api/skills/retablir" | "/api/skills/installer" => "skill",
        _ => "modification",
    }
}

#[derive(Deserialize)]
struct ParamId {
    id: String,
}

/// Texte complet d'un document, pour l'éditeur.
async fn api_source(State(e): State<Partage>, Query(p): Query<ParamId>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Value> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        i.mettre_a_jour()?;
        let (chemin, texte) = edition::source(&i, &p.id)?;
        Ok(json!({ "chemin": chemin, "texte": texte }))
    })
    .await;
    resultat(r)
}

#[derive(Deserialize)]
struct CorpsEcrire {
    id: String,
    texte: String,
}

async fn api_ecrire(State(e): State<Partage>, Json(c): Json<CorpsEcrire>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        i.mettre_a_jour()?;
        let chemin = edition::ecrire(&i, &c.id, &c.texte)?;
        i.mettre_a_jour()?;
        Ok(chemin)
    })
    .await;
    resultat(r)
}

#[derive(Deserialize)]
struct CorpsStatut {
    id: String,
    statut: String,
}

async fn api_statut(State(e): State<Partage>, Json(c): Json<CorpsStatut>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        i.mettre_a_jour()?;
        let chemin = edition::changer_statut(&i, &c.id, &c.statut)?;
        i.mettre_a_jour()?;
        Ok(chemin)
    })
    .await;
    resultat(r)
}

/// Annule la dernière action (Ctrl+Z) ; `null` s'il n'y a rien à annuler.
async fn api_annuler(State(e): State<Partage>, Json(_): Json<Value>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Option<journal::Bilan>> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        let b = journal::annuler(&i.racine)?;
        i.mettre_a_jour()?;
        Ok(b)
    })
    .await;
    resultat(r)
}

/// Rejoue la dernière action annulée (Ctrl+Y) ; `null` s'il n'y a rien à rétablir.
async fn api_retablir(State(e): State<Partage>, Json(_): Json<Value>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Option<journal::Bilan>> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        let b = journal::retablir(&i.racine)?;
        i.mettre_a_jour()?;
        Ok(b)
    })
    .await;
    resultat(r)
}

async fn api_journal() -> Json<Value> {
    let (a, r) = journal::etat();
    Json(json!({ "annulables": a, "retablissables": r }))
}

/// Une page mise en favori (EF-157) : son adresse et son titre.
#[derive(Serialize, Deserialize)]
struct Favori {
    url: String,
    titre: String,
}

fn fichier_favoris(e: &Partage) -> std::path::PathBuf {
    let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
    i.racine.join(".wikik").join("favoris.json")
}

/// Favoris de cette machine (`.wikik/favoris.json`, pas versionné).
async fn api_favoris(State(e): State<Partage>) -> Json<Vec<Favori>> {
    let f = fichier_favoris(&e);
    Json(
        std::fs::read_to_string(f)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default(),
    )
}

async fn api_ecrire_favoris(State(e): State<Partage>, Json(liste): Json<Vec<Favori>>) -> Response {
    let f = fichier_favoris(&e);
    let r = tokio::task::spawn_blocking(move || -> Result<bool> {
        std::fs::create_dir_all(f.parent().expect("dossier .wikik"))?;
        std::fs::write(f, serde_json::to_string_pretty(&liste)?)?;
        Ok(true)
    })
    .await;
    resultat(r)
}

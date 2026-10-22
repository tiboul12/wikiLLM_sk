//! Routes d'édition (ADR-0016) : documents modifiés dans le navigateur (EF-158), annuler et rétablir
//! (EF-159), favoris (EF-157), tests (EF-160) et dessins de l'atelier (EF-167). L'édition et l'annulation écrivent dans le projet : elles sont réservées au
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
/// @relu 2026-10-22 7747fa71
pub(crate) fn routes(etat: Partage) -> Router<Partage> {
    let reservees = Router::new()
        .route("/api/document/source", get(api_source))
        .route("/api/document/ecrire", post(api_ecrire))
        .route("/api/document/statut", post(api_statut))
        .route("/api/document/valider-test", post(api_valider_test))
        .route("/api/document/nouveau", post(api_nouveau))
        .route("/api/dessin", post(api_ecrire_dessin))
        .route("/api/annuler", post(api_annuler))
        .route("/api/retablir", post(api_retablir))
        .route_layer(axum::middleware::from_fn_with_state(etat, acces::reserve));
    Router::new()
        .route("/api/journal", get(api_journal))
        .route("/api/dessins", get(api_dessins))
        .route("/api/dessin/lire", get(api_lire_dessin))
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
/// @relu 2026-10-22 2b07bbc1
fn libelle(chemin: &str) -> &'static str {
    match chemin {
        "/api/document/ecrire" => "modification du document",
        "/api/document/statut" => "changement de statut",
        "/api/document/valider-test" => "validation d'un test",
        "/api/document/nouveau" => "nouveau document",
        "/api/dessin" => "dessin",
        "/api/atelier/decider" => "décision sur une proposition",
        "/api/atelier/saisir" => "saisie de l'atelier",
        "/api/atelier/selection" => "sélection",
        "/api/atelier/synthetiser" => "synthèse",
        "/api/atelier/proposer" => "propositions du LLM",
        "/api/atelier/parcours" => "parcours",
        "/api/atelier/parcours/modifier" => "atelier modifié",
        "/api/atelier/clarification" => "clarification",
        "/api/atelier/piloter" => "étape de l'atelier",
        "/api/reception/remarque" => "remarque",
        "/api/reception/traiter" => "traitement d'une remarque",
        "/api/reception/valider" => "validation d'une remarque",
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

#[derive(Deserialize)]
struct CorpsNouveau {
    genre: String,
    titre: String,
    /// Exigences vérifiées (un test créé depuis la page d'une exigence).
    #[serde(default)]
    exigences: Vec<String>,
}

/// Crée un document au prochain identifiant libre (`wikik nouveau`), éventuellement relié à des
/// exigences ; renvoie son identifiant.
async fn api_nouveau(State(e): State<Partage>, Json(c): Json<CorpsNouveau>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        let rel = wikik_core::init::nouveau(&i.racine, &i.config, &c.genre, c.titre.trim())?;
        let abs = i.racine.join(&rel);
        let mut texte = std::fs::read_to_string(&abs)?;
        if let Some(premiere) = c.exigences.first() {
            texte = edition::avec_champ(
                &texte,
                "exigences",
                &format!("[{}]", c.exigences.join(", ")),
            );
            // Le test prend la version de l'exigence qu'il vérifie.
            let version: Option<String> = i
                .conn
                .query_row(
                    "SELECT version FROM documents WHERE id = ?1",
                    [premiere],
                    |r| r.get(0),
                )
                .ok()
                .flatten();
            if let Some(v) = version {
                texte = edition::avec_champ(&texte, "version", &v);
            }
            journal::ecrire(&abs, &texte)?;
        }
        i.mettre_a_jour()?;
        let id = texte
            .lines()
            .find_map(|l| l.strip_prefix("id:"))
            .map(|x| x.split('#').next().unwrap_or("").trim().to_string())
            .unwrap_or_default();
        Ok(id)
    })
    .await;
    resultat(r)
}

#[derive(Deserialize)]
struct CorpsVerdict {
    id: String,
    verdict: String,
}

/// Ticket de validation d'un test (EF-160) : validé par l'utilisateur, en échec, ou rouvert.
async fn api_valider_test(State(e): State<Partage>, Json(c): Json<CorpsVerdict>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        i.mettre_a_jour()?;
        let chemin = edition::valider_test(&i, &c.id, &c.verdict)?;
        i.mettre_a_jour()?;
        Ok(chemin)
    })
    .await;
    resultat(r)
}

#[derive(Deserialize)]
struct CorpsDessin {
    nom: String,
    #[serde(default)]
    svg: String,
}

/// Enregistre un dessin de l'atelier (EF-167).
async fn api_ecrire_dessin(State(e): State<Partage>, Json(c): Json<CorpsDessin>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<String> {
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        edition::ecrire_dessin(&i, &c.nom, &c.svg)
    })
    .await;
    resultat(r)
}

#[derive(Deserialize)]
struct ParamDessin {
    nom: String,
}

async fn api_lire_dessin(State(e): State<Partage>, Query(p): Query<ParamDessin>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Option<String>> {
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        edition::lire_dessin(&i, &p.nom)
    })
    .await;
    resultat(r)
}

async fn api_dessins(State(e): State<Partage>) -> Json<Vec<String>> {
    let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
    Json(edition::lister_dessins(&i))
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

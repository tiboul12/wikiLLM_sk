//! Routes de l'atelier de définition du besoin (SPEC-18 à SPEC-20), section du serveur du projet.
//!
//! Les routes d'écriture passent par le moteur de parcours, qui n'écrit que dans `docs/atelier/`,
//! `docs/projet/` et `.wikik/atelier/`. Elles exigent un corps JSON et, si le navigateur l'envoie, une
//! origine identique à l'hôte : une page tierce ne peut pas écrire dans le projet. L'appel au LLM se fait
//! hors du verrou de l'index : le wiki reste lisible pendant qu'il réfléchit.
//!
//! @req EF-100, EF-104, ENF-007, ENF-008
//! @adr 10

use crate::serve::{Partage, repondre};
use anyhow::Result;
use axum::extract::{Query, Request, State};
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use wikik_core::Index;
use wikik_core::atelier::fournisseur::Demande;
use wikik_core::atelier::parcours;

/// Routes de l'atelier (état, pilotage, propositions, décisions, saisies, synthèses, rappel, parcours,
/// clarifications). Chaque écriture d'un parcours porte `parcours` (`PARCOURS` par défaut, EF-145).
pub(crate) fn routes() -> Router<Partage> {
    Router::new()
        .route("/api/atelier/etat", get(api_etat))
        .route("/api/atelier/parcours", post(api_parcours))
        .route("/api/atelier/clarification", post(api_clarification))
        .route("/api/atelier/piloter", post(api_piloter))
        .route("/api/atelier/selection", post(api_selection))
        .route("/api/atelier/proposer", post(api_proposer))
        .route("/api/atelier/decider", post(api_decider))
        .route("/api/atelier/saisir", post(api_saisir))
        .route("/api/atelier/synthetiser", post(api_synthetiser))
        .route("/api/atelier/rappel", post(api_rappel))
}

/// Une écriture venue d'une autre origine (page tierce ouverte dans le navigateur) est refusée.
pub(crate) async fn meme_origine(requete: Request, suite: Next) -> Response {
    if requete.method() != Method::GET && requete.method() != Method::HEAD {
        let hote = requete
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();
        if let Some(origine) = requete.headers().get(header::ORIGIN) {
            let origine = origine.to_str().unwrap_or_default();
            if origine.split("://").nth(1) != Some(hote) {
                return (StatusCode::FORBIDDEN, "origine refusée").into_response();
            }
        }
    }
    suite.run(requete).await
}

fn defaut() -> String {
    parcours::DEFAUT.to_string()
}

#[derive(Deserialize)]
struct ParamEtat {
    #[serde(default = "defaut")]
    parcours: String,
}

async fn api_etat(State(e): State<Partage>, Query(q): Query<ParamEtat>) -> Response {
    let nom = e.fournisseur.nom();
    repondre(e, move |i| parcours::etat(i, &q.parcours, &nom)).await
}

#[derive(Deserialize)]
struct NouveauParcours {
    titre: String,
    #[serde(default)]
    version: String,
}

async fn api_parcours(State(e): State<Partage>, Json(p): Json<NouveauParcours>) -> Response {
    ecrire(e, move |i| {
        parcours::creer_parcours(i, &p.titre, &p.version)
    })
    .await
}

#[derive(Deserialize)]
struct ActionClarification {
    id: String,
    /// `clore`, `rouvrir` ou `supprimer`.
    action: String,
}

async fn api_clarification(
    State(e): State<Partage>,
    Json(p): Json<ActionClarification>,
) -> Response {
    ecrire(e, move |i| match p.action.as_str() {
        "clore" => wikik_core::atelier::clore(i, &p.id),
        "rouvrir" => wikik_core::atelier::rouvrir(i, &p.id),
        "supprimer" => wikik_core::atelier::supprimer(i, &p.id),
        autre => anyhow::bail!("action inconnue : {autre}"),
    })
    .await
}

#[derive(Deserialize)]
struct Pilotage {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    action: String,
    mode: Option<String>,
}

async fn api_piloter(State(e): State<Partage>, Json(p): Json<Pilotage>) -> Response {
    ecrire(e, move |i| {
        parcours::piloter(i, &p.parcours, &p.etape, &p.action, p.mode.as_deref())
    })
    .await
}

#[derive(Deserialize)]
struct Selection {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    ids: Vec<String>,
}

async fn api_selection(State(e): State<Partage>, Json(p): Json<Selection>) -> Response {
    ecrire(e, move |i| {
        parcours::selectionner(i, &p.parcours, &p.etape, &p.sous_etape, &p.ids)
    })
    .await
}

#[derive(Deserialize)]
struct Proposer {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    entree: Option<String>,
}

/// Prépare sous verrou, appelle le LLM sans, range la réponse sous verrou, dans le parcours demandé.
/// @relu 2026-10-19 a34aad18
async fn api_proposer(State(e): State<Partage>, Json(p): Json<Proposer>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<parcours::Lot> {
        let prep = {
            let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
            i.mettre_a_jour()?;
            parcours::preparer(
                &i,
                &p.parcours,
                &p.etape,
                &p.sous_etape,
                p.cible.as_deref(),
                p.entree.as_deref(),
            )?
        };
        let nom = e.fournisseur.nom();
        let reponse = e.fournisseur.proposer(&Demande {
            consigne: &prep.consigne,
            schema: &prep.schema,
            exemple: prep.exemple.as_ref(),
        });
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        match reponse {
            Ok(r) => parcours::enregistrer(&i, &prep, &r, &nom),
            Err(err) => {
                let _ = parcours::journaliser(&i.racine, &prep, &nom, Err(&format!("{err:#}")));
                Err(err)
            }
        }
    })
    .await;
    resultat(r)
}

#[derive(Deserialize)]
struct Decision {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    indice: usize,
    decision: String,
    donnees: Option<Value>,
}

async fn api_decider(State(e): State<Partage>, Json(p): Json<Decision>) -> Response {
    ecrire(e, move |i| {
        parcours::decider(
            i,
            &p.parcours,
            &p.etape,
            &p.sous_etape,
            p.cible.as_deref(),
            p.indice,
            &p.decision,
            p.donnees,
        )
    })
    .await
}

#[derive(Deserialize)]
struct Saisie {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    donnees: Value,
}

async fn api_saisir(State(e): State<Partage>, Json(p): Json<Saisie>) -> Response {
    ecrire(e, move |i| {
        parcours::saisir(
            i,
            &p.parcours,
            &p.etape,
            &p.sous_etape,
            p.cible.as_deref(),
            &p.donnees,
        )
    })
    .await
}

#[derive(Deserialize)]
struct Synthese {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
}

async fn api_synthetiser(State(e): State<Partage>, Json(p): Json<Synthese>) -> Response {
    ecrire(e, move |i| {
        parcours::synthetiser(i, &p.parcours, &p.etape, &p.sous_etape)
    })
    .await
}

#[derive(Deserialize)]
struct Rappel {
    sujet: String,
    #[serde(default)]
    question: String,
}

async fn api_rappel(State(e): State<Partage>, Json(p): Json<Rappel>) -> Response {
    ecrire(e, move |i| {
        wikik_core::atelier::rappel(i, &p.sujet, &p.question)
    })
    .await
}

/// Écriture sous verrou ; l'index est remis à jour après, pour que la lecture suivante voie le résultat.
pub(crate) async fn ecrire<T, F>(e: Partage, f: F) -> Response
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce(&Index) -> Result<T> + Send + 'static,
{
    let r = tokio::task::spawn_blocking(move || {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        i.mettre_a_jour()?;
        let v = f(&i)?;
        i.mettre_a_jour()?;
        Ok(v)
    })
    .await;
    resultat(r)
}

pub(crate) fn resultat<T: serde::Serialize>(
    r: std::result::Result<Result<T>, tokio::task::JoinError>,
) -> Response {
    match r {
        Ok(Ok(v)) => Json(v).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "erreur": format!("{e:#}") })),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

//! `wikik atelier` : l'atelier de définition du besoin (SPEC-18 à SPEC-20), servi avec l'API de lecture du
//! wiki et le même front.
//!
//! Les routes d'écriture passent par le moteur de parcours, qui n'écrit que dans `docs/atelier/`,
//! `docs/projet/` et `.wikik/atelier/`. Elles exigent un corps JSON et, si le navigateur l'envoie, une
//! origine identique à l'hôte : une page tierce ne peut pas écrire dans le projet. L'appel au LLM se fait
//! hors du verrou de l'index : le wiki reste lisible pendant qu'il réfléchit.
//!
//! @req EF-100, EF-104, ENF-007, ENF-008
//! @adr 10

use crate::serve::{Etat, Partage, lancer, repondre, routes_wiki, verifier_adresse};
use anyhow::Result;
use axum::Json;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wikik_core::Index;
use wikik_core::atelier::fournisseur::{self, Demande, Fournisseur};
use wikik_core::atelier::parcours;

/// Fournisseur LLM de l'atelier.
pub(crate) struct Atelier {
    pub(crate) fournisseur: Box<dyn Fournisseur>,
}

/// Démarre l'atelier sur `port` (ou le suivant libre) et bloque jusqu'à Ctrl+C.
pub fn servir(
    racine: &Path,
    adresse: &str,
    port: crate::serve::Port,
    autoriser_reseau: bool,
    factice: bool,
) -> Result<()> {
    let ip = verifier_adresse(adresse, autoriser_reseau)?;
    let mut index = Index::ouvrir(racine)?;
    index.mettre_a_jour()?;
    let mut config = index.config.atelier.llm.clone();
    if factice {
        config.fournisseur = "factice".into();
    }
    let fournisseur =
        fournisseur::depuis_config(&config, &index.racine, &parcours::travail(&index.racine))?;
    eprintln!("wikik atelier : fournisseur {}", fournisseur.nom());
    let racine = index.racine.clone();
    let etat: Partage = Arc::new(Etat {
        index: Mutex::new(index),
        atelier: Some(Atelier { fournisseur }),
    });
    lancer(routes(etat), ip, port, &racine, "atelier")
}

/// Routes du wiki et de l'atelier.
pub(crate) fn routes(etat: Partage) -> axum::Router {
    routes_wiki()
        .route("/api/atelier/etat", get(api_etat))
        .route("/api/atelier/piloter", post(api_piloter))
        .route("/api/atelier/selection", post(api_selection))
        .route("/api/atelier/proposer", post(api_proposer))
        .route("/api/atelier/decider", post(api_decider))
        .route("/api/atelier/saisir", post(api_saisir))
        .route("/api/atelier/synthetiser", post(api_synthetiser))
        .route("/api/atelier/rappel", post(api_rappel))
        .layer(axum::middleware::from_fn(meme_origine))
        .with_state(etat)
}

/// Une écriture venue d'une autre origine (page tierce ouverte dans le navigateur) est refusée.
async fn meme_origine(requete: Request, suite: Next) -> Response {
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

fn nom_fournisseur(e: &Etat) -> String {
    e.atelier
        .as_ref()
        .map_or_else(|| "aucun".into(), |a| a.fournisseur.nom())
}

async fn api_etat(State(e): State<Partage>) -> Response {
    let nom = nom_fournisseur(&e);
    repondre(e, move |i| parcours::etat(i, &nom)).await
}

#[derive(Deserialize)]
struct Pilotage {
    etape: String,
    action: String,
    mode: Option<String>,
}

async fn api_piloter(State(e): State<Partage>, Json(p): Json<Pilotage>) -> Response {
    ecrire(e, move |i| {
        parcours::piloter(i, &p.etape, &p.action, p.mode.as_deref())
    })
    .await
}

#[derive(Deserialize)]
struct Selection {
    etape: String,
    sous_etape: String,
    ids: Vec<String>,
}

async fn api_selection(State(e): State<Partage>, Json(p): Json<Selection>) -> Response {
    ecrire(e, move |i| {
        parcours::selectionner(i, &p.etape, &p.sous_etape, &p.ids)
    })
    .await
}

#[derive(Deserialize)]
struct Proposer {
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    entree: Option<String>,
}

/// Prépare sous verrou, appelle le LLM sans, range la réponse sous verrou.
async fn api_proposer(State(e): State<Partage>, Json(p): Json<Proposer>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<parcours::Lot> {
        let prep = {
            let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
            i.mettre_a_jour()?;
            parcours::preparer(
                &i,
                &p.etape,
                &p.sous_etape,
                p.cible.as_deref(),
                p.entree.as_deref(),
            )?
        };
        let atelier = e
            .atelier
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("pas de fournisseur"))?;
        let nom = atelier.fournisseur.nom();
        let reponse = atelier.fournisseur.proposer(&Demande {
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
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    donnees: Value,
}

async fn api_saisir(State(e): State<Partage>, Json(p): Json<Saisie>) -> Response {
    ecrire(e, move |i| {
        parcours::saisir(i, &p.etape, &p.sous_etape, p.cible.as_deref(), &p.donnees)
    })
    .await
}

#[derive(Deserialize)]
struct Synthese {
    etape: String,
    sous_etape: String,
}

async fn api_synthetiser(State(e): State<Partage>, Json(p): Json<Synthese>) -> Response {
    ecrire(e, move |i| {
        parcours::synthetiser(i, &p.etape, &p.sous_etape)
    })
    .await
}

#[derive(Deserialize)]
struct Rappel {
    sujet: String,
    question: String,
}

async fn api_rappel(State(e): State<Partage>, Json(p): Json<Rappel>) -> Response {
    ecrire(e, move |i| {
        wikik_core::atelier::rappel(i, &p.sujet, &p.question)
    })
    .await
}

/// Écriture sous verrou ; l'index est remis à jour après, pour que la lecture suivante voie le résultat.
async fn ecrire<T, F>(e: Partage, f: F) -> Response
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

fn resultat<T: serde::Serialize>(
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

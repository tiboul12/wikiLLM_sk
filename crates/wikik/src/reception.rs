//! Routes du cahier de réception (SPEC-22, SPEC-23), section du serveur du projet.
//!
//! Les écritures ne touchent que `docs/reception/` ; mêmes gardes que l'atelier (corps JSON, origine
//! identique). Le corps d'une requête peut atteindre 25 Mo : les captures collées voyagent en base64.
//!
//! @req EF-120, EF-121, EF-122, EF-123, EF-124

use crate::atelier::{ecrire, resultat};
use crate::serve::{Partage, repondre};
use anyhow::Result;
use axum::extract::State;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use wikik_core::Index;
use wikik_core::atelier::fournisseur::Demande;
use wikik_core::reception::{carte, remarques};

/// Routes du cahier de réception.
pub(crate) fn routes() -> Router<Partage> {
    Router::new()
        .route("/api/reception/etat", get(api_etat))
        .route("/api/reception/scanner", post(api_scanner))
        .route("/api/reception/page", post(api_page))
        .route("/api/reception/remarque", post(api_remarque))
        .route("/api/reception/traiter", post(api_traiter))
        .route("/api/reception/supprimer", post(api_supprimer))
        .route("/api/reception/application", post(api_application))
        .route("/api/reception/completer", post(api_completer))
}

/// La carte est enregistrée au premier geste qui en a besoin (remarque, correction, adresse) : ouvrir
/// le cahier n'écrit rien dans le dépôt (ADR-0012).
fn assurer_carte(i: &Index) -> Result<()> {
    if carte::lire(i).is_empty() {
        carte::scanner(i)?;
    }
    Ok(())
}

async fn api_etat(State(e): State<Partage>) -> Response {
    let nom = e.fournisseur.nom();
    repondre(e, move |i| wikik_core::reception::etat(i, &nom)).await
}

async fn api_scanner(State(e): State<Partage>) -> Response {
    ecrire(e, carte::scanner).await
}

#[derive(Deserialize)]
struct Correction {
    cle: String,
    nom: Option<String>,
    masque: Option<bool>,
    groupe: Option<String>,
    route: Option<String>,
}

async fn api_page(State(e): State<Partage>, Json(p): Json<Correction>) -> Response {
    ecrire(e, move |i| {
        assurer_carte(i)?;
        carte::corriger(
            i,
            &p.cle,
            p.nom.as_deref(),
            p.masque,
            p.groupe.as_deref(),
            p.route.as_deref(),
        )
    })
    .await
}

#[derive(Deserialize)]
struct Application {
    url: String,
}

async fn api_application(State(e): State<Partage>, Json(p): Json<Application>) -> Response {
    ecrire(e, move |i| {
        assurer_carte(i)?;
        carte::definir_url_application(i, &p.url)
    })
    .await
}

/// Saisie d'une remarque : création (`id` absent) ou nouvelle version d'une remarque (`id`).
#[derive(Deserialize)]
struct SaisieRemarque {
    id: Option<String>,
    page: String,
    texte: String,
    genre: String,
    gravite: String,
    #[serde(default)]
    images: Vec<String>,
    #[serde(default)]
    garder: Vec<String>,
}

async fn api_remarque(State(e): State<Partage>, Json(p): Json<SaisieRemarque>) -> Response {
    ecrire(e, move |i| {
        assurer_carte(i)?;
        let date = wikik_core::relu::aujourdhui();
        let s = remarques::Saisie {
            page: &p.page,
            texte: &p.texte,
            genre: &p.genre,
            gravite: &p.gravite,
            images: &p.images,
            garder: &p.garder,
            date: &date,
        };
        match &p.id {
            Some(id) => remarques::modifier(i, id, &s),
            None => remarques::creer(i, &s),
        }
    })
    .await
}

#[derive(Deserialize)]
struct Suppression {
    id: String,
}

async fn api_supprimer(State(e): State<Partage>, Json(p): Json<Suppression>) -> Response {
    ecrire(e, move |i| remarques::supprimer(i, &p.id)).await
}

#[derive(Deserialize)]
struct Traitement {
    id: String,
    statut: String,
    #[serde(default)]
    resolution: String,
    commit: Option<String>,
}

async fn api_traiter(State(e): State<Partage>, Json(p): Json<Traitement>) -> Response {
    ecrire(e, move |i| {
        remarques::traiter(i, &p.id, &p.statut, &p.resolution, p.commit.as_deref())
    })
    .await
}

/// Demande au LLM les pages manquantes (hors verrou), puis les ajoute à la carte.
/// @relu 2026-10-19 f5140df8
async fn api_completer(State(e): State<Partage>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Vec<carte::Page>> {
        let (consigne, schema) = {
            let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
            i.mettre_a_jour()?;
            assurer_carte(&i)?;
            carte::demande_completion(&i)
        };
        let reponse = e.fournisseur.proposer(&Demande {
            consigne: &consigne,
            schema: &schema,
            exemple: Some(&Value::Array(vec![])),
        })?;
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        let ajoutees = carte::ajouter_llm(&i, &reponse.elements)?;
        i.mettre_a_jour()?;
        Ok(ajoutees)
    })
    .await;
    resultat(r)
}

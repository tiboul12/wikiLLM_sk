//! `wikik reception` : le cahier de réception (SPEC-22, SPEC-23), servi avec l'API de lecture du wiki et
//! le même front.
//!
//! Les écritures ne touchent que `docs/reception/` ; mêmes gardes que l'atelier (corps JSON, origine
//! identique). Le corps d'une requête peut atteindre 25 Mo : les captures collées voyagent en base64.
//!
//! @req EF-120, EF-121, EF-122, EF-123, EF-124

use crate::atelier::{ecrire, meme_origine, resultat};
use crate::serve::{Etat, Partage, Port, lancer, repondre, routes_wiki, verifier_adresse};
use anyhow::Result;
use axum::Json;
use axum::extract::State;
use axum::response::Response;
use axum::routing::{get, post};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wikik_core::Index;
use wikik_core::atelier::fournisseur::{self, Demande, Fournisseur};
use wikik_core::reception::{carte, remarques};

/// Fournisseur LLM du cahier (complément de la carte).
pub(crate) struct Reception {
    pub(crate) fournisseur: Box<dyn Fournisseur>,
}

/// Démarre le cahier de réception sur `port` (ou le suivant libre) et bloque jusqu'à Ctrl+C. La carte
/// est scannée au premier lancement.
pub fn servir(
    racine: &Path,
    adresse: &str,
    port: Port,
    autoriser_reseau: bool,
    factice: bool,
) -> Result<()> {
    let ip = verifier_adresse(adresse, autoriser_reseau)?;
    let mut index = Index::ouvrir(racine)?;
    index.mettre_a_jour()?;
    if carte::lire(&index).is_empty() {
        let pages = carte::scanner(&index)?;
        index.mettre_a_jour()?;
        eprintln!(
            "wikik reception : carte construite, {} page(s)",
            pages.len()
        );
    }
    let mut config = index.config.atelier.llm.clone();
    if factice {
        config.fournisseur = "factice".into();
    }
    let travail = index
        .racine
        .join(wikik_core::index::DOSSIER)
        .join("reception");
    let fournisseur = fournisseur::depuis_config(&config, &index.racine, &travail)?;
    let racine = index.racine.clone();
    let etat: Partage = Arc::new(Etat {
        index: Mutex::new(index),
        atelier: None,
        reception: Some(Reception { fournisseur }),
    });
    lancer(routes(etat), ip, port, &racine, "reception")
}

/// Routes du wiki et du cahier de réception.
pub(crate) fn routes(etat: Partage) -> axum::Router {
    routes_wiki()
        .route("/api/reception/etat", get(api_etat))
        .route("/api/reception/scanner", post(api_scanner))
        .route("/api/reception/page", post(api_page))
        .route("/api/reception/remarque", post(api_remarque))
        .route("/api/reception/traiter", post(api_traiter))
        .route("/api/reception/completer", post(api_completer))
        .layer(axum::extract::DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(axum::middleware::from_fn(meme_origine))
        .with_state(etat)
}

async fn api_etat(State(e): State<Partage>) -> Response {
    let nom = e
        .reception
        .as_ref()
        .map_or_else(|| "aucun".into(), |r| r.fournisseur.nom());
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
}

async fn api_page(State(e): State<Partage>, Json(p): Json<Correction>) -> Response {
    ecrire(e, move |i| {
        carte::corriger(i, &p.cle, p.nom.as_deref(), p.masque, p.groupe.as_deref())
    })
    .await
}

#[derive(Deserialize)]
struct NouvelleRemarque {
    page: String,
    texte: String,
    genre: String,
    gravite: String,
    #[serde(default)]
    images: Vec<String>,
}

async fn api_remarque(State(e): State<Partage>, Json(p): Json<NouvelleRemarque>) -> Response {
    ecrire(e, move |i| {
        remarques::creer(
            i,
            &p.page,
            &p.texte,
            &p.genre,
            &p.gravite,
            &p.images,
            &wikik_core::relu::aujourdhui(),
        )
    })
    .await
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
async fn api_completer(State(e): State<Partage>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Vec<carte::Page>> {
        let (consigne, schema) = {
            let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
            i.mettre_a_jour()?;
            carte::demande_completion(&i)
        };
        let r = e
            .reception
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("pas de fournisseur"))?;
        let reponse = r.fournisseur.proposer(&Demande {
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

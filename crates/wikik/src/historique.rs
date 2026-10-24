//! Routes de la section Git (EF-166) : l'historique du dépôt pour le graphe des branches, le détail d'un
//! commit et le diff d'un fichier. Lecture seule, comme le reste du wiki.
//!
//! @req EF-166

use crate::serve::{Partage, repondre};
use anyhow::{Context, Result};
use axum::Router;
use axum::extract::{Query, State};
use axum::response::Response;
use axum::routing::get;
use serde::Deserialize;
use wikik_core::Index;
use wikik_core::git::Depot;

/// Routes de lecture de l'historique git.
pub(crate) fn routes() -> Router<Partage> {
    Router::new()
        .route("/api/git/historique", get(api_historique))
        .route("/api/git/commit", get(api_commit))
        .route("/api/git/diff", get(api_diff))
}

/// Dépôt git du projet ; erreur lisible si le projet n'en est pas un.
fn depot(i: &Index) -> Result<Depot> {
    Depot::trouver(&i.racine).context("le projet n'est pas dans un dépôt git")
}

/// Paramètre `?limite=` de l'historique.
#[derive(Deserialize)]
struct ParamHistorique {
    limite: Option<usize>,
}

/// Commits (400 par défaut, 5 000 au plus), références et situation de la branche.
async fn api_historique(State(e): State<Partage>, Query(p): Query<ParamHistorique>) -> Response {
    repondre(e, move |i| {
        Ok(depot(i)?.historique(p.limite.unwrap_or(400).min(5000)))
    })
    .await
}

/// Paramètres d'un commit : hash, et chemin du fichier pour le diff.
#[derive(Deserialize)]
struct ParamCommit {
    hash: String,
    #[serde(default)]
    chemin: String,
}

/// Message, auteur et fichiers changés d'un commit.
async fn api_commit(State(e): State<Partage>, Query(p): Query<ParamCommit>) -> Response {
    repondre(e, move |i| {
        depot(i)?
            .detail(&p.hash)
            .with_context(|| format!("commit inconnu : {}", p.hash))
    })
    .await
}

/// Diff d'un fichier dans un commit.
async fn api_diff(State(e): State<Partage>, Query(p): Query<ParamCommit>) -> Response {
    repondre(e, move |i| {
        depot(i)?
            .diff(&p.hash, &p.chemin)
            .with_context(|| format!("diff introuvable : {} {}", p.hash, p.chemin))
    })
    .await
}

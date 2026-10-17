//! `wikik serve` : sert le wiki en local, en lecture seule, avec le front embarqué (SPEC-09).
//!
//! Chaque appel d'API met l'index à jour (incrémental) puis lit `wikik_core::wiki`. Aucune route
//! n'écrit sur le disque ; une adresse hors boucle locale exige `--autoriser-reseau`.
//!
//! @req EF-040, ENF-004, ENF-005
//! @adr 4

use anyhow::{Result, bail};
use axum::extract::{Query, State};
use axum::http::{HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::{Arc, Mutex};
use wikik_core::{Index, check, wiki};

/// Front compilé (`front/dist`), embarqué dans le binaire en release.
#[derive(rust_embed::Embed)]
#[folder = "../../front/dist"]
struct Front;

struct Etat {
    index: Mutex<Index>,
}

type Partage = Arc<Etat>;

/// Démarre le serveur et bloque jusqu'à Ctrl+C.
pub fn servir(racine: &Path, adresse: &str, port: u16, autoriser_reseau: bool) -> Result<()> {
    let ip: IpAddr = adresse.parse()?;
    if !ip.is_loopback() && !autoriser_reseau {
        bail!(
            "{adresse} n'est pas une adresse locale : ajouter --autoriser-reseau pour exposer le wiki sur le réseau"
        );
    }
    let mut index = Index::ouvrir(racine)?;
    index.mettre_a_jour()?;
    let etat: Partage = Arc::new(Etat {
        index: Mutex::new(index),
    });
    let app = Router::new()
        .route("/api/projet", get(api_projet))
        .route("/api/fichier", get(api_fichier))
        .route("/api/document", get(api_document))
        .route("/api/graphe", get(api_graphe))
        .route("/api/cherche", get(api_cherche))
        .route("/api/check", get(api_check))
        .fallback(statique)
        .layer(axum::middleware::map_response(entetes_securite))
        .with_state(etat);
    let adresse_socket = SocketAddr::new(ip, port);
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let ecoute = tokio::net::TcpListener::bind(adresse_socket).await?;
            eprintln!("wikik serve : http://{adresse_socket} (Ctrl+C pour arrêter)");
            axum::serve(ecoute, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
            Ok(())
        })
}

/// Exécute `f` sur l'index à jour, hors du fil asynchrone, et répond en JSON.
async fn repondre<T, F>(etat: Partage, f: F) -> Response
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce(&Index) -> Result<T> + Send + 'static,
{
    let r = tokio::task::spawn_blocking(move || {
        let mut index = etat.index.lock().unwrap_or_else(|e| e.into_inner());
        index.mettre_a_jour()?;
        f(&index)
    })
    .await;
    match r {
        Ok(Ok(v)) => Json(v).into_response(),
        Ok(Err(e)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "erreur": format!("{e:#}") })),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Deserialize)]
struct ParamChemin {
    chemin: String,
}

#[derive(Deserialize)]
struct ParamId {
    id: String,
}

#[derive(Deserialize)]
struct ParamGraphe {
    centre: Option<String>,
    profondeur: Option<usize>,
}

#[derive(Deserialize)]
struct ParamCherche {
    q: String,
}

async fn api_projet(State(e): State<Partage>) -> Response {
    repondre(e, wiki::projet).await
}

async fn api_fichier(State(e): State<Partage>, Query(p): Query<ParamChemin>) -> Response {
    repondre(e, move |i| wiki::fichier(i, &p.chemin)).await
}

async fn api_document(State(e): State<Partage>, Query(p): Query<ParamId>) -> Response {
    repondre(e, move |i| wiki::document(i, &p.id)).await
}

async fn api_graphe(State(e): State<Partage>, Query(p): Query<ParamGraphe>) -> Response {
    repondre(e, move |i| {
        wiki::graphe(
            i,
            p.centre.as_deref().filter(|c| !c.is_empty()),
            p.profondeur.unwrap_or(1),
        )
    })
    .await
}

async fn api_cherche(State(e): State<Partage>, Query(p): Query<ParamCherche>) -> Response {
    repondre(e, move |i| wiki::cherche(i, &p.q)).await
}

async fn api_check(State(e): State<Partage>) -> Response {
    repondre(e, check::verifier).await
}

/// Fichiers du front ; toute autre route renvoie `index.html` (application monopage).
async fn statique(methode: Method, uri: Uri) -> Response {
    if methode != Method::GET && methode != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let chemin = uri.path().trim_start_matches('/');
    if chemin.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (fichier, contenu) = match Front::get(chemin) {
        Some(c) if !chemin.is_empty() => (chemin, c),
        _ => match Front::get("index.html") {
            Some(c) => ("index.html", c),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };
    let mime = contenu.metadata.mimetype().to_string();
    let cache = if fichier.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, cache.to_string()),
        ],
        contenu.data,
    )
        .into_response()
}

/// Aucune ressource externe, pas de détection de type par le navigateur (SPEC-09 R5).
async fn entetes_securite(mut r: Response) -> Response {
    let h = r.headers_mut();
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
             font-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    r
}

#[cfg(test)]
mod tests {
    #[test]
    fn refuse_le_reseau_sans_option() {
        let d = std::env::temp_dir();
        let e = super::servir(&d, "0.0.0.0", 0, false).unwrap_err();
        assert!(e.to_string().contains("--autoriser-reseau"));
    }
}

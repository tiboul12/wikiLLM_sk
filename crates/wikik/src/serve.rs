//! Le serveur du projet, lancé par `wikik` (ADR-0012) : le wiki (Produit, Projet), l'atelier et le
//! cahier de réception sur un seul port, avec le front embarqué (SPEC-09, SPEC-20, SPEC-23).
//!
//! Chaque appel d'API met l'index à jour (incrémental). Les routes de lecture n'écrivent rien ; celles
//! de l'atelier et de la réception n'écrivent que dans leurs dossiers, et seulement pour une requête
//! JSON de même origine. Lancer le serveur n'écrit rien dans le dépôt. Une adresse hors boucle locale
//! exige `--reseau` (ou `[serveur] reseau = true`).
//!
//! @req EF-040, EF-130, ENF-004, ENF-005
//! @adr 4
//! @adr 12

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
use wikik_core::atelier::fournisseur::{self, Fournisseur};
use wikik_core::{Index, check, wiki};

/// Front compilé (`front/dist`), embarqué dans le binaire en release.
#[derive(rust_embed::Embed)]
#[folder = "../../front/dist"]
pub(crate) struct Front;

pub(crate) struct Etat {
    pub(crate) index: Mutex<Index>,
    /// Fournisseur LLM de l'atelier et du complément de la carte de réception.
    pub(crate) fournisseur: Box<dyn Fournisseur>,
}

pub(crate) type Partage = Arc<Etat>;

/// Port d'écoute : celui demandé (`--port`), ou le port par défaut et, s'il est pris (un autre projet
/// ouvert), les suivants.
#[derive(Clone, Copy)]
pub struct Port {
    pub numero: u16,
    pub automatique: bool,
}

impl Port {
    /// `--port` s'il est donné, sinon `defaut` avec recherche automatique.
    pub fn choisir(demande: Option<u16>, defaut: u16) -> Port {
        Port {
            numero: demande.unwrap_or(defaut),
            automatique: demande.is_none(),
        }
    }
}

/// Démarre le serveur du projet sur `port` (ou le suivant libre) et bloque jusqu'à Ctrl+C ; `factice`
/// remplace le LLM par les exemples des fiches.
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
    let mut config = index.config.atelier.llm.clone();
    if factice {
        config.fournisseur = "factice".into();
    }
    let travail = wikik_core::atelier::parcours::travail(&index.racine);
    let fournisseur = fournisseur::depuis_config(&config, &index.racine, &travail)?;
    eprintln!("wikik : LLM {}", fournisseur.nom());
    let racine = index.racine.clone();
    let etat: Partage = Arc::new(Etat {
        index: Mutex::new(index),
        fournisseur,
    });
    lancer(routes(etat), ip, port, &racine)
}

/// Toutes les routes : lecture du wiki, atelier, réception ; écritures gardées (corps JSON, même origine,
/// 25 Mo au plus pour les captures).
pub(crate) fn routes(etat: Partage) -> Router {
    routes_wiki()
        .merge(crate::atelier::routes())
        .merge(crate::reception::routes())
        .layer(axum::extract::DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(axum::middleware::from_fn(crate::atelier::meme_origine))
        .with_state(etat)
}

/// Refuse une adresse hors boucle locale si le réseau n'est pas autorisé (`--reseau` ou `[serveur] reseau`).
pub(crate) fn verifier_adresse(adresse: &str, autoriser_reseau: bool) -> Result<IpAddr> {
    let ip: IpAddr = adresse.parse()?;
    if !ip.is_loopback() && !autoriser_reseau {
        bail!(
            "{adresse} n'est pas une adresse locale : ajouter --reseau pour exposer le projet sur le réseau"
        );
    }
    Ok(ip)
}

/// Routes de lecture du wiki, et `/api/image` pour les captures des documents.
pub(crate) fn routes_wiki() -> Router<Partage> {
    Router::new()
        .route("/api/projet", get(api_projet))
        .route("/api/fichier", get(api_fichier))
        .route("/api/document", get(api_document))
        .route("/api/graphe", get(api_graphe))
        .route("/api/cherche", get(api_cherche))
        .route("/api/check", get(api_check))
        .route("/api/image", get(api_image))
}

#[derive(Deserialize)]
struct ParamImage {
    chemin: String,
}

/// Image d'un document (capture d'une remarque) : seulement une vraie image PNG, JPEG, GIF ou WebP
/// rangée dans une zone de documents ; jamais un autre fichier du projet (SPEC-23 R3).
async fn api_image(State(e): State<Partage>, Query(p): Query<ParamImage>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Option<(Vec<u8>, &'static str)> {
        let index = e.index.lock().unwrap_or_else(|x| x.into_inner());
        let relatif = p.chemin.trim_start_matches('/');
        if relatif.split('/').any(|c| c == ".." || c.starts_with('.')) {
            return None;
        }
        if !index
            .config
            .zones_documents()
            .iter()
            .any(|z| relatif.starts_with(&format!("{z}/")))
        {
            return None;
        }
        let chemin = index.racine.join(relatif).canonicalize().ok()?;
        if !chemin.starts_with(&index.racine) {
            return None;
        }
        let octets = std::fs::read(&chemin).ok()?;
        let mime = match wikik_core::reception::remarques::type_image(&octets)? {
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "gif" => "image/gif",
            _ => "image/webp",
        };
        Some((octets, mime))
    })
    .await;
    match r {
        Ok(Some((octets, mime))) => ([(header::CONTENT_TYPE, mime)], octets).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Ajoute le front et les en-têtes, puis sert jusqu'à Ctrl+C. Un port automatique déjà pris (autre
/// projet ouvert) fait essayer les 20 suivants ; le port obtenu est noté dans `.wikik/serveur.json`
/// (adresse renvoyée par `atelier_rappel`) et retiré à l'arrêt. En écoute sur toutes les interfaces,
/// affiche aussi l'adresse à ouvrir depuis un autre poste (IP du réseau local).
fn lancer(app: Router, ip: IpAddr, port: Port, racine: &Path) -> Result<()> {
    let app = app
        .fallback(statique)
        .layer(axum::middleware::map_response(entetes_securite));
    let projet = racine
        .file_name()
        .map_or_else(|| "projet".into(), |n| n.to_string_lossy().into_owned());
    let nom = format!("wikik ({projet})");
    let racine = racine.to_path_buf();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let essais = if port.automatique { 20 } else { 1 };
            let mut ecoute = None;
            for p in (port.numero..).take(essais) {
                match tokio::net::TcpListener::bind(SocketAddr::new(ip, p)).await {
                    Ok(e) => {
                        ecoute = Some(e);
                        break;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => continue,
                    Err(e) => return Err(e.into()),
                }
            }
            let Some(ecoute) = ecoute else {
                bail!(
                    "port {} déjà utilisé{} : choisir un autre port avec --port",
                    port.numero,
                    if port.automatique { " (et les 19 suivants)" } else { "" }
                );
            };
            let adresse_socket = ecoute.local_addr()?;
            let port = adresse_socket.port();
            wikik_core::reseau::noter_port(&racine, Some(port));
            if adresse_socket.ip().is_unspecified() {
                // Écoute sur toutes les interfaces : l'adresse utile est celle du réseau local.
                eprintln!("{nom} : http://127.0.0.1:{port} sur cette machine (Ctrl+C pour arrêter)");
                match wikik_core::reseau::ip_locale() {
                    Some(ip) => eprintln!("  → depuis un autre poste : http://{ip}:{port}"),
                    None => eprintln!("  → adresse réseau introuvable : utiliser l'IP de cette machine, port {port}"),
                }
            } else {
                eprintln!("{nom} : http://{adresse_socket} (Ctrl+C pour arrêter)");
            }
            let r = axum::serve(ecoute, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await;
            wikik_core::reseau::noter_port(&racine, None);
            r?;
            Ok(())
        })
}

/// Exécute `f` sur l'index à jour, hors du fil asynchrone, et répond en JSON.
pub(crate) async fn repondre<T, F>(etat: Partage, f: F) -> Response
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
        let e = super::servir(
            &d,
            "0.0.0.0",
            super::Port::choisir(Some(0), 8484),
            false,
            true,
        )
        .unwrap_err();
        assert!(e.to_string().contains("--reseau"));
    }
}

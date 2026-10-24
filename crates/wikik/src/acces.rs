//! Accès du propriétaire (ADR-0013) : ce qui exécute des commandes (terminal, commande du LLM) n'est
//! ouvert qu'à la machine locale, ou au navigateur qui a ouvert l'adresse du lancement avec la clé du
//! serveur. La clé vit dans `.wikik/cle` (non versionné) ; `rm .wikik/cle` puis relancer en tire une autre.
//!
//! @req EF-147, EF-148
//! @adr 13

use crate::serve::Partage;
use anyhow::Result;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::net::SocketAddr;
use std::path::Path;

/// Cookie qui porte la clé du propriétaire dans le navigateur (ADR-0013).
const COOKIE: &str = "wikik_cle";

/// Clé du serveur du projet : celle de `.wikik/cle`, tirée au hasard (128 bits) au premier lancement.
pub(crate) fn cle(racine: &Path) -> Result<String> {
    let chemin = racine.join(wikik_core::index::DOSSIER).join("cle");
    if let Ok(c) = std::fs::read_to_string(&chemin)
        && c.trim().len() >= 32
    {
        return Ok(c.trim().to_string());
    }
    let mut octets = [0u8; 16];
    use std::io::Read;
    std::fs::File::open("/dev/urandom")?.read_exact(&mut octets)?;
    let c: String = octets.iter().map(|o| format!("{o:02x}")).collect();
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(&chemin, &c)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(c)
}

/// Vrai pour la machine locale, ou pour un navigateur qui porte le cookie de la clé.
pub(crate) fn proprietaire(e: &Partage, entetes: &HeaderMap, adresse: SocketAddr) -> bool {
    if adresse.ip().is_loopback() {
        return true;
    }
    entetes
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|c| c.trim().split_once('='))
        .any(|(n, v)| n == COOKIE && egal(v.as_bytes(), e.cle.as_bytes()))
}

/// Comparaison en temps constant : la durée ne dit rien du nombre de caractères justes.
fn egal(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

/// Une connexion WebSocket ouverte par une page d'une autre origine est refusée : un site tiers ne
/// peut pas piloter le terminal (le navigateur envoie toujours `Origin` sur un WebSocket).
pub(crate) fn meme_origine(entetes: &HeaderMap) -> bool {
    let hote = entetes
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    match entetes.get(header::ORIGIN).and_then(|o| o.to_str().ok()) {
        Some(o) => o.split("://").nth(1) == Some(hote),
        None => true,
    }
}

/// `GET …?cle=<clé>` (adresse affichée au lancement) : pose le cookie de la clé et renvoie vers la même
/// page sans la clé dans l'adresse. Une clé fausse est ignorée.
pub(crate) async fn cle_en_cookie(
    State(e): State<Partage>,
    requete: Request,
    suite: Next,
) -> Response {
    let uri = requete.uri().clone();
    let donnee = uri.query().and_then(|q| {
        q.split('&')
            .filter_map(|p| p.split_once('='))
            .find(|(n, _)| *n == "cle")
            .map(|(_, v)| v.to_string())
    });
    match donnee {
        Some(c) if egal(c.as_bytes(), e.cle.as_bytes()) => {
            let reste: Vec<&str> = uri
                .query()
                .unwrap_or_default()
                .split('&')
                .filter(|p| !p.starts_with("cle="))
                .collect();
            let cible = if reste.is_empty() {
                uri.path().to_string()
            } else {
                format!("{}?{}", uri.path(), reste.join("&"))
            };
            let mut r = (StatusCode::SEE_OTHER, [(header::LOCATION, cible)]).into_response();
            if let Ok(v) = HeaderValue::from_str(&format!(
                "{COOKIE}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=31536000",
                e.cle
            )) {
                r.headers_mut().insert(header::SET_COOKIE, v);
            }
            r
        }
        _ => suite.run(requete).await,
    }
}

/// Garde des routes réservées au propriétaire : 403 sinon, avec la marche à suivre.
pub(crate) async fn reserve(
    State(e): State<Partage>,
    ConnectInfo(adresse): ConnectInfo<SocketAddr>,
    requete: Request,
    suite: Next,
) -> Response {
    if proprietaire(&e, requete.headers(), adresse) && meme_origine(requete.headers()) {
        suite.run(requete).await
    } else {
        (
            StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({ "erreur": "réservé au propriétaire du projet : ouvre wikik depuis cette machine, ou avec l'adresse à clé affichée au lancement" })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::egal;

    #[test]
    fn comparaison() {
        assert!(egal(b"abc", b"abc"));
        assert!(!egal(b"abc", b"abd"));
        assert!(!egal(b"abc", b"abcd"));
    }
}

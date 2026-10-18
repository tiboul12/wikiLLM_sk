//! Adresses des serveurs : IP de la machine sur le réseau local (liens ouvrables depuis un autre poste
//! quand les serveurs écoutent sur le réseau, `[serveur] reseau`) et port réellement pris par chaque
//! serveur d'un projet (plusieurs projets ouverts en même temps : le port suivant libre est pris).

use crate::config::Config;
use crate::index::{DOSSIER, Index};
use serde_json::{Value, json};
use std::net::{IpAddr, UdpSocket};
use std::path::Path;

/// Fichier où chaque serveur d'un projet note son port (`{"atelier": 8486, "wiki": 8484}`).
const SERVEURS: &str = "serveurs.json";

/// Note (`Some`) ou retire (`None`) le port d'un serveur du projet (`atelier` ou `wiki`).
pub fn noter_port(racine: &Path, role: &str, port: Option<u16>) {
    let chemin = racine.join(DOSSIER).join(SERVEURS);
    let mut v: Value = std::fs::read_to_string(&chemin)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}));
    let Some(o) = v.as_object_mut() else { return };
    match port {
        Some(p) => o.insert(role.into(), json!(p)),
        None => o.remove(role),
    };
    let _ = std::fs::create_dir_all(chemin.parent().unwrap());
    let _ = std::fs::write(chemin, v.to_string());
}

fn port_note(racine: &Path, role: &str) -> Option<u16> {
    let t = std::fs::read_to_string(racine.join(DOSSIER).join(SERVEURS)).ok()?;
    let v: Value = serde_json::from_str(&t).ok()?;
    v.get(role)?.as_u64().and_then(|p| u16::try_from(p).ok())
}

/// Adresse configurée, avec le port réellement pris si le serveur tourne et que l'adresse est celle de
/// cette machine (boucle locale) ; une adresse explicite (nom, autre IP) reste telle quelle.
fn effective(racine: &Path, url: &str, role: &str) -> String {
    let Some(port) = port_note(racine, role) else {
        return url.to_string();
    };
    for hote in ["127.0.0.1", "localhost"] {
        for schema in ["http://", "https://"] {
            let debut = format!("{schema}{hote}");
            if let Some(reste) = url.strip_prefix(&debut) {
                let chemin = match reste.strip_prefix(':') {
                    Some(r) => r.trim_start_matches(|c: char| c.is_ascii_digit()),
                    None if reste.is_empty() || reste.starts_with('/') => reste,
                    None => continue,
                };
                return format!("{schema}{hote}:{port}{chemin}");
            }
        }
    }
    url.to_string()
}

/// Adresse de l'atelier du projet (`[atelier] url`, port réel s'il tourne).
pub fn adresse_atelier(index: &Index) -> String {
    effective(&index.racine, &index.config.atelier.url, "atelier")
}

/// Adresse du wiki du projet (`[atelier] url_wiki`, port réel si `wikik serve` tourne).
pub fn adresse_wiki(index: &Index) -> String {
    effective(&index.racine, &index.config.atelier.url_wiki, "wiki")
}

/// Adresse IP locale de la machine vers l'extérieur, sans rien envoyer : une socket UDP « connectée »
/// donne l'interface que le système choisirait. `None` hors réseau.
pub fn ip_locale() -> Option<IpAddr> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?;
    let ip = s.local_addr().ok()?.ip();
    (!ip.is_loopback() && !ip.is_unspecified()).then_some(ip)
}

/// Remplace l'hôte de boucle locale d'une URL (`127.0.0.1`, `localhost`) par l'IP du réseau quand les
/// serveurs écoutent sur le réseau ; sinon renvoie l'URL telle quelle. Une adresse configurée
/// explicitement (un nom, une autre IP) n'est jamais touchée.
pub fn pour_le_reseau(url: &str, config: &Config) -> String {
    if !config.serveur.reseau {
        return url.to_string();
    }
    let Some(ip) = ip_locale() else {
        return url.to_string();
    };
    for hote in ["127.0.0.1", "localhost"] {
        for schema in ["http://", "https://"] {
            let debut = format!("{schema}{hote}");
            if let Some(reste) = url.strip_prefix(&debut)
                && (reste.is_empty() || reste.starts_with(':') || reste.starts_with('/'))
            {
                return format!("{schema}{ip}{reste}");
            }
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hote_de_boucle_remplace_seulement_en_reseau() {
        let mut c = Config::default();
        assert_eq!(
            pour_le_reseau("http://127.0.0.1:8485/x", &c),
            "http://127.0.0.1:8485/x"
        );
        c.serveur.reseau = true;
        let r = pour_le_reseau("http://127.0.0.1:8485/x", &c);
        match ip_locale() {
            Some(ip) => assert_eq!(r, format!("http://{ip}:8485/x")),
            None => assert_eq!(r, "http://127.0.0.1:8485/x"),
        }
        assert_eq!(
            pour_le_reseau("http://atelier.maison:8485", &c),
            "http://atelier.maison:8485"
        );
        assert_eq!(
            pour_le_reseau("http://127.0.0.10:1", &c),
            "http://127.0.0.10:1"
        );
    }

    #[test]
    fn port_reellement_pris() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        assert_eq!(
            effective(r, "http://127.0.0.1:8485", "atelier"),
            "http://127.0.0.1:8485"
        );
        noter_port(r, "atelier", Some(8487));
        noter_port(r, "wiki", Some(8484));
        assert_eq!(
            effective(r, "http://127.0.0.1:8485", "atelier"),
            "http://127.0.0.1:8487"
        );
        assert_eq!(
            effective(r, "http://localhost:8485/x", "atelier"),
            "http://localhost:8487/x"
        );
        assert_eq!(
            effective(r, "http://atelier.maison:8485", "atelier"),
            "http://atelier.maison:8485"
        );
        noter_port(r, "atelier", None);
        assert_eq!(
            effective(r, "http://127.0.0.1:8485", "atelier"),
            "http://127.0.0.1:8485"
        );
        assert_eq!(
            effective(r, "http://127.0.0.1:9000", "wiki"),
            "http://127.0.0.1:8484"
        );
    }
}

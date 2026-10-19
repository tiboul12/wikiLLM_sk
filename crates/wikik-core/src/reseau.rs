//! Adresse du serveur d'un projet : IP de la machine sur le réseau local (liens ouvrables depuis un autre
//! poste quand le serveur écoute sur le réseau, `[serveur] reseau`) et port réellement pris (plusieurs
//! projets ouverts en même temps : le port suivant libre est pris).

use crate::config::Config;
use crate::index::{DOSSIER, Index};
use serde_json::{Value, json};
use std::net::{IpAddr, UdpSocket};
use std::path::Path;

/// Fichier où le serveur du projet note son port (`{"port": 8486}`).
const SERVEUR: &str = "serveur.json";

/// Port du serveur quand aucun n'est noté.
pub const PORT_DEFAUT: u16 = 8484;

/// Note (`Some`) ou retire (`None`) le port réellement pris par le serveur du projet.
pub fn noter_port(racine: &Path, port: Option<u16>) {
    let chemin = racine.join(DOSSIER).join(SERVEUR);
    match port {
        Some(p) => {
            let _ = std::fs::create_dir_all(chemin.parent().unwrap());
            let _ = std::fs::write(chemin, json!({ "port": p }).to_string());
        }
        None => {
            let _ = std::fs::remove_file(chemin);
        }
    }
}

fn port_note(racine: &Path) -> Option<u16> {
    let t = std::fs::read_to_string(racine.join(DOSSIER).join(SERVEUR)).ok()?;
    let v: Value = serde_json::from_str(&t).ok()?;
    v.get("port")?.as_u64().and_then(|p| u16::try_from(p).ok())
}

/// Adresse du serveur du projet sur cette machine : le port réellement pris s'il tourne, sinon le port
/// par défaut (liens renvoyés à l'agent, comme celui d'`atelier_rappel`).
pub fn adresse_serveur(index: &Index) -> String {
    format!(
        "http://127.0.0.1:{}",
        port_note(&index.racine).unwrap_or(PORT_DEFAUT)
    )
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
        let mut index = Index::ouvrir(d.path()).unwrap();
        index.mettre_a_jour().unwrap();
        assert_eq!(adresse_serveur(&index), "http://127.0.0.1:8484");
        noter_port(&index.racine, Some(8487));
        assert_eq!(adresse_serveur(&index), "http://127.0.0.1:8487");
        noter_port(&index.racine, None);
        assert_eq!(adresse_serveur(&index), "http://127.0.0.1:8484");
    }
}

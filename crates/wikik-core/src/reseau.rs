//! Adresse de la machine sur le réseau local, pour afficher et renvoyer des liens ouvrables depuis un
//! autre poste quand les serveurs écoutent sur le réseau (`[serveur] reseau`).

use crate::config::Config;
use std::net::{IpAddr, UdpSocket};

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
}

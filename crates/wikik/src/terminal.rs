//! Terminal intégré (EF-148, ADR-0013) : un pseudo-terminal ouvert dans le dossier du projet (le shell
//! de l'utilisateur), relié au navigateur par WebSocket. Un seul terminal par serveur, gardé quand le
//! volet se ferme ; à la reconnexion, la fin de sa sortie est rejouée. « Traiter avec l'agent » y envoie
//! la consigne de traitement : collée dans l'agent s'il tourne, sinon en lançant l'agent avec elle.
//!
//! Protocole : trames binaires = octets tapés (navigateur → serveur) ou sortie du terminal (serveur →
//! navigateur) ; trame texte `{"cols":…,"rows":…}` = nouvelle taille.
//!
//! @req EF-148
//! @adr 13

use crate::acces;
use crate::atelier::resultat;
use crate::serve::Partage;
use anyhow::{Context, Result, bail};
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde::Deserialize;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Sortie gardée pour la rejouer à la reconnexion.
const HISTORIQUE: usize = 256 * 1024;

/// Le terminal du serveur.
pub(crate) struct Terminal {
    ecrivain: Mutex<Box<dyn Write + Send>>,
    maitre: Mutex<Box<dyn MasterPty + Send>>,
    enfant: Mutex<Box<dyn Child + Send + Sync>>,
    sortie: broadcast::Sender<Vec<u8>>,
    historique: Arc<Mutex<Vec<u8>>>,
}

impl Terminal {
    /// Ouvre le shell de l'utilisateur (`$SHELL`, sinon `/bin/bash`) dans `racine`.
    ///
    /// @pitfall La racine de l'index est souvent relative (`.`) : elle est rendue absolue, et `PWD` posé,
    /// sinon un shell lancé d'ailleurs (ou qui relit un `PWD` hérité) s'ouvre hors du projet (REC-066).
    /// @req EF-168
    fn ouvrir(racine: &Path) -> Result<Arc<Terminal>> {
        let racine = racine
            .canonicalize()
            .unwrap_or_else(|_| racine.to_path_buf());
        let paire = native_pty_system()
            .openpty(PtySize {
                rows: 30,
                cols: 100,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("ouverture du pseudo-terminal")?;
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(&racine);
        cmd.env("PWD", &racine);
        cmd.env("TERM", "xterm-256color");
        let enfant = paire
            .slave
            .spawn_command(cmd)
            .context("lancement du shell")?;
        drop(paire.slave);
        let mut lecteur = paire.master.try_clone_reader()?;
        let ecrivain = paire.master.take_writer()?;
        let (sortie, _) = broadcast::channel(256);
        let historique = Arc::new(Mutex::new(Vec::new()));
        let (s, h) = (sortie.clone(), historique.clone());
        std::thread::spawn(move || {
            let mut tampon = [0u8; 8192];
            while let Ok(n) = lecteur.read(&mut tampon) {
                if n == 0 {
                    break;
                }
                let morceau = tampon[..n].to_vec();
                {
                    let mut h = h.lock().unwrap_or_else(|x| x.into_inner());
                    h.extend_from_slice(&morceau);
                    if h.len() > HISTORIQUE {
                        let trop = h.len() - HISTORIQUE;
                        h.drain(..trop);
                    }
                }
                let _ = s.send(morceau);
            }
        });
        Ok(Arc::new(Terminal {
            ecrivain: Mutex::new(ecrivain),
            maitre: Mutex::new(paire.master),
            enfant: Mutex::new(enfant),
            sortie,
            historique,
        }))
    }

    fn vivant(&self) -> bool {
        matches!(
            self.enfant
                .lock()
                .unwrap_or_else(|x| x.into_inner())
                .try_wait(),
            Ok(None)
        )
    }

    fn ecrire(&self, octets: &[u8]) -> Result<()> {
        let mut e = self.ecrivain.lock().unwrap_or_else(|x| x.into_inner());
        e.write_all(octets)?;
        e.flush()?;
        Ok(())
    }

    /// Le shell attend à son invite : aucun programme (l'agent par exemple) n'occupe le premier plan.
    fn a_l_invite(&self) -> bool {
        let shell = self
            .enfant
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .process_id();
        #[cfg(unix)]
        let premier_plan = self
            .maitre
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .process_group_leader()
            .and_then(|p| u32::try_from(p).ok());
        #[cfg(not(unix))]
        let premier_plan: Option<u32> = None;
        shell.is_some() && premier_plan == shell
    }

    fn redimensionner(&self, cols: u16, rows: u16) {
        let _ = self
            .maitre
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .resize(PtySize {
                rows: rows.clamp(5, 500),
                cols: cols.clamp(10, 1000),
                pixel_width: 0,
                pixel_height: 0,
            });
    }
}

/// Routes du terminal ; toutes réservées au propriétaire (machine locale ou clé).
pub(crate) fn routes(etat: Partage) -> Router<Partage> {
    Router::new()
        .route("/api/terminal", get(api_terminal))
        .route("/api/terminal/envoyer", post(api_envoyer))
        .route_layer(axum::middleware::from_fn_with_state(etat, acces::reserve))
}

/// Terminal du serveur, rouvert s'il n'existe pas ou si son shell est sorti.
fn obtenir(e: &Partage) -> Result<Arc<Terminal>> {
    let mut t = e.terminal.lock().unwrap_or_else(|x| x.into_inner());
    if let Some(x) = t.as_ref().filter(|x| x.vivant()) {
        return Ok(x.clone());
    }
    let racine = e
        .index
        .lock()
        .unwrap_or_else(|x| x.into_inner())
        .racine
        .clone();
    let x = Terminal::ouvrir(&racine)?;
    *t = Some(x.clone());
    Ok(x)
}

async fn api_terminal(ws: WebSocketUpgrade, State(e): State<Partage>) -> Response {
    match obtenir(&e) {
        Ok(t) => ws.on_upgrade(move |s| relier(s, t)),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}")).into_response(),
    }
}

/// Relie un navigateur au terminal jusqu'à la fermeture de l'un ou de l'autre.
async fn relier(socket: WebSocket, t: Arc<Terminal>) {
    let (mut vers, mut depuis) = socket.split();
    // S'abonner avant de lire l'historique : rien n'est perdu entre les deux.
    let mut abonne = t.sortie.subscribe();
    let passe = t
        .historique
        .lock()
        .unwrap_or_else(|x| x.into_inner())
        .clone();
    if vers.send(Message::Binary(passe.into())).await.is_err() {
        return;
    }
    let envoi = tokio::spawn(async move {
        loop {
            match abonne.recv().await {
                Ok(o) => {
                    if vers.send(Message::Binary(o.into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });
    while let Some(Ok(m)) = depuis.next().await {
        match m {
            Message::Binary(b) => {
                if t.ecrire(&b).is_err() {
                    break;
                }
            }
            Message::Text(texte) => {
                if let Ok(taille) = serde_json::from_str::<Taille>(&texte) {
                    t.redimensionner(taille.cols, taille.rows);
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    envoi.abort();
}

#[derive(Deserialize)]
struct Taille {
    cols: u16,
    rows: u16,
}

#[derive(Deserialize)]
struct Envoi {
    texte: String,
    /// `commande` : tapé tel quel puis Entrée. `consigne` : pour l'agent ; s'il tourne, collé d'un bloc
    /// (collage « bracketed paste », sans valider ligne par ligne) puis Entrée ; si le shell attend à son
    /// invite, l'agent (`[ia] agent`) est lancé avec la consigne en argument.
    mode: String,
}

/// Argument de shell entre apostrophes.
fn citer(t: &str) -> String {
    format!("'{}'", t.replace('\'', "'\\''"))
}

/// Envoie un texte dans le terminal ouvert (consigne de traitement, commande) ; renvoie `agent` si
/// l'agent a été lancé, `colle` ou `tape` sinon.
async fn api_envoyer(State(e): State<Partage>, Json(p): Json<Envoi>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<&'static str> {
        let agent = e
            .index
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .config
            .ia
            .agent
            .clone();
        let t = e
            .terminal
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .clone()
            .filter(|t| t.vivant());
        let Some(t) = t else {
            bail!("ouvre d'abord le terminal (volet de droite) et lance ton agent");
        };
        let texte = p.texte.trim();
        if texte.is_empty() {
            bail!("rien à envoyer");
        }
        match p.mode.as_str() {
            "commande" => {
                t.ecrire(format!("{texte}\r").as_bytes())?;
                Ok("tape")
            }
            "consigne" if t.a_l_invite() => {
                t.ecrire(format!("{agent} {}\r", citer(texte)).as_bytes())?;
                Ok("agent")
            }
            "consigne" => {
                t.ecrire(format!("\x1b[200~{texte}\x1b[201~").as_bytes())?;
                // Laisser l'agent ranger le collage avant de valider.
                std::thread::sleep(std::time::Duration::from_millis(150));
                t.ecrire(b"\r")?;
                Ok("colle")
            }
            autre => bail!("mode inconnu : {autre} (commande ou consigne)"),
        }
    })
    .await;
    resultat(r)
}

#[cfg(test)]
mod tests {
    #[test]
    fn citation() {
        assert_eq!(super::citer("l'agent"), "'l'\\''agent'");
    }
}

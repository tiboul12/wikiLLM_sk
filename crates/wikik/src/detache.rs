//! Serveur en arrière-plan (ADR-0014) : `wikik` lance le serveur du projet détaché du terminal (nouvelle
//! session, sans terminal de contrôle) ; il survit à la fermeture de la connexion SSH et tourne jusqu'à
//! l'arrêt de la machine ou jusqu'au bouton « Arrêter le serveur » de l'interface. `wikik` rend la main
//! dès que le serveur répond, après avoir affiché ses adresses ; relancé alors qu'il tourne, il les
//! réaffiche sans en démarrer un second.
//!
//! @req EF-130
//! @adr 14

use anyhow::{Context, Result, bail};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use wikik_core::index::DOSSIER;

/// Variable posée sur le serveur lui-même (et par les tests) : il tourne au premier plan.
pub(crate) const PREMIER_PLAN: &str = "WIKIK_PREMIER_PLAN";

/// Journal du serveur détaché : ses adresses, puis ses messages.
fn journal(racine: &Path) -> std::path::PathBuf {
    racine.join(DOSSIER).join("serveur.log")
}

/// Port noté par un serveur du projet qui répond encore.
fn en_marche(racine: &Path) -> Option<u16> {
    let t = std::fs::read_to_string(racine.join(DOSSIER).join("serveur.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&t).ok()?;
    let port = u16::try_from(v.get("port")?.as_u64()?).ok()?;
    TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_millis(300))
        .ok()
        .map(|_| port)
}

/// Les lignes d'adresses écrites par le serveur au démarrage.
fn adresses(racine: &Path) -> String {
    std::fs::read_to_string(journal(racine))
        .unwrap_or_default()
        .lines()
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Démarre le serveur détaché avec les mêmes arguments, attend qu'il réponde et affiche ses adresses.
pub(crate) fn lancer(racine: &Path) -> Result<i32> {
    let racine = racine
        .canonicalize()
        .with_context(|| format!("dossier introuvable : {}", racine.display()))?;
    if en_marche(&racine).is_some() {
        eprintln!("wikik tourne déjà pour ce projet :\n{}", adresses(&racine));
        return Ok(0);
    }
    // Un port noté par un serveur arrêté brutalement ne doit pas faire croire qu'il est prêt.
    let _ = std::fs::remove_file(racine.join(DOSSIER).join("serveur.json"));
    std::fs::create_dir_all(racine.join(DOSSIER))?;
    let sortie = std::fs::File::create(journal(&racine))?;
    let mut cmd = Command::new(std::env::current_exe()?);
    cmd.args(std::env::args_os().skip(1))
        .env(PREMIER_PLAN, "1")
        .stdin(Stdio::null())
        .stdout(sortie.try_clone()?)
        .stderr(sortie);
    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        // Nouvelle session : plus de terminal de contrôle, donc pas de SIGHUP à la fin de la connexion.
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut enfant = cmd.spawn().context("lancement du serveur")?;
    let limite = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(statut) = enfant.try_wait()? {
            let log = std::fs::read_to_string(journal(&racine)).unwrap_or_default();
            bail!(
                "le serveur s'est arrêté au démarrage ({statut}) :\n{}",
                log.trim()
            );
        }
        if en_marche(&racine).is_some() {
            eprintln!("{}", adresses(&racine));
            eprintln!(
                "Le serveur tourne en arrière-plan, même après la fermeture de la connexion SSH.\n\
                 Pour l'arrêter : bouton ⏻ en bas de la barre de gauche (journal : .wikik/serveur.log)."
            );
            return Ok(0);
        }
        if Instant::now() > limite {
            bail!("le serveur ne répond pas après 60 s (journal : .wikik/serveur.log)");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

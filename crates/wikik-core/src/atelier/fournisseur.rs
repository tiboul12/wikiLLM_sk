//! Fournisseur LLM de l'atelier (SPEC-19) : le CLI local (`claude -p`, abonnement) branché sur le serveur
//! MCP de wikik, ou un fournisseur factice pour les tests et la démonstration.
//!
//! @req EF-104, EF-111, ENF-007, ENF-008
//! @adr 11

use super::fiches::valider;
use crate::config::ConfigLlm;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use ts_rs::TS;

/// Utilisation de l'abonnement renvoyée par le CLI (fractions de 0 à 1).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Consommation {
    pub cinq_heures: f64,
    pub sept_jours: f64,
}

/// Une demande de propositions, avec son jeton d'annulation.
pub struct Demande<'a> {
    pub consigne: &'a str,
    /// Schéma JSON de la réponse (objet `{ elements: [...] }`).
    pub schema: &'a Value,
    /// Exemple de la fiche, renvoyé par le fournisseur factice.
    pub exemple: Option<&'a Value>,
    /// Jeton d'annulation pris à l'arrivée de la requête ([`Fournisseur::jeton`]) : une annulation
    /// demandée pendant la préparation (scan, index) arrête quand même l'appel. `None` : celui du début
    /// de l'appel.
    pub jeton: Option<u64>,
}

/// Réponse validée.
#[derive(Debug, Clone)]
pub struct Reponse {
    pub elements: Vec<Value>,
    pub consommation: Option<Consommation>,
    pub duree_ms: u128,
}

/// Fournisseur de propositions, dont les appels en cours peuvent être annulés (SPEC-19 R7).
pub trait Fournisseur: Send + Sync {
    /// Nom affiché (`claude`, `factice`).
    fn nom(&self) -> String;
    /// Demande au LLM ; la réponse est validée contre le schéma.
    fn proposer(&self, demande: &Demande) -> Result<Reponse>;
    /// Arrête les appels en cours (REC-030) : ils échouent avec « appel au LLM annulé ». Sans effet pour
    /// un fournisseur qui répond tout de suite.
    fn annuler(&self) {}
    /// Jeton d'annulation courant : un appel lancé avec ce jeton s'arrête à la prochaine annulation.
    fn jeton(&self) -> u64 {
        0
    }
}

/// Fournisseur factice : renvoie l'exemple de la fiche, sans réseau (SPEC-19 R3).
pub struct Factice;

impl Fournisseur for Factice {
    fn nom(&self) -> String {
        "factice".into()
    }

    fn proposer(&self, d: &Demande) -> Result<Reponse> {
        let elements = d
            .exemple
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(Reponse {
            elements,
            consommation: None,
            duree_ms: 0,
        })
    }
}

/// Fournisseur par ligne de commande (SPEC-19 R2) ; un appel s'arrête au délai ou à l'annulation (R7).
pub struct Cli {
    pub config: ConfigLlm,
    /// Fichier `--mcp-config` écrit par l'atelier (`{mcp}`).
    pub mcp: PathBuf,
    /// Avance à chaque annulation : un appel lancé avant s'arrête.
    annulations: AtomicU64,
}

impl Cli {
    /// Prépare le fournisseur et écrit la configuration MCP qui pointe sur le binaire courant et `racine`.
    /// @relu 2026-10-19 d4900ae3
    pub fn preparer(config: ConfigLlm, racine: &Path, dossier_travail: &Path) -> Result<Cli> {
        std::fs::create_dir_all(dossier_travail)?;
        let exe = std::env::current_exe().context("chemin du binaire wikik")?;
        let mcp = dossier_travail.join("mcp.json");
        let contenu = json!({ "mcpServers": { "wikik": {
            "command": exe.to_string_lossy(),
            "args": ["--racine", racine.to_string_lossy(), "mcp"],
        }}});
        std::fs::write(&mcp, serde_json::to_string_pretty(&contenu)?)?;
        Ok(Cli {
            config,
            mcp,
            annulations: AtomicU64::new(0),
        })
    }

    fn un_appel(&self, d: &Demande, depart: u64) -> Result<(Option<Value>, Option<Consommation>)> {
        let schema = serde_json::to_string(d.schema)?;
        let args: Vec<String> = self
            .config
            .arguments
            .iter()
            .map(|a| {
                a.replace("{consigne}", d.consigne)
                    .replace("{schema}", &schema)
                    .replace("{mcp}", &self.mcp.to_string_lossy())
            })
            .collect();
        let mut commande = Command::new(&self.config.commande);
        // Son propre groupe de processus : l'arrêter arrête aussi ce qu'il a lancé (serveur MCP, shell).
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut commande, 0);
        let mut enfant = commande
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("lancement de `{}`", self.config.commande))?;
        let sortie = enfant.stdout.take().unwrap();
        let lecteur = std::thread::spawn(move || analyser_flux(BufReader::new(sortie)));
        let limite = Instant::now() + Duration::from_secs(self.config.delai.max(1));
        loop {
            if let Some(statut) = enfant.try_wait()? {
                let (donnees, conso, erreur) = lecteur.join().unwrap_or((None, None, None));
                if donnees.is_none() && !statut.success() {
                    let mut msg = String::new();
                    if let Some(mut e) = enfant.stderr.take() {
                        use std::io::Read;
                        let _ = e.read_to_string(&mut msg);
                    }
                    bail!(
                        "le LLM a échoué ({statut}) : {}",
                        erreur.unwrap_or(msg).trim()
                    );
                }
                return Ok((donnees, conso));
            }
            if Instant::now() > limite {
                arreter(&mut enfant);
                bail!("le LLM n'a pas répondu en {} s", self.config.delai);
            }
            if self.annulations.load(Ordering::SeqCst) != depart {
                arreter(&mut enfant);
                bail!("appel au LLM annulé");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

/// Lit le flux `stream-json` : réponse structurée, consommation, message d'erreur éventuel.
pub fn analyser_flux(
    lecteur: impl BufRead,
) -> (Option<Value>, Option<Consommation>, Option<String>) {
    let (mut donnees, mut conso, mut erreur) = (None, None, None);
    for ligne in lecteur.lines().map_while(Result::ok) {
        let Ok(e) = serde_json::from_str::<Value>(&ligne) else {
            continue;
        };
        match e.get("type").and_then(Value::as_str) {
            Some("rate_limit_event") => {
                let f = |k: &str| {
                    e.pointer(&format!("/rate_limit_info/unifiedWindows/{k}/utilization"))
                        .and_then(Value::as_f64)
                };
                if let (Some(a), Some(b)) = (f("five_hour"), f("seven_day")) {
                    conso = Some(Consommation {
                        cinq_heures: a,
                        sept_jours: b,
                    });
                }
            }
            Some("result") => {
                donnees = e.get("structured_output").cloned().filter(|v| !v.is_null());
                if e.get("is_error").and_then(Value::as_bool) == Some(true) {
                    erreur = e.get("result").and_then(Value::as_str).map(str::to_string);
                }
            }
            _ => {}
        }
    }
    (donnees, conso, erreur)
}

impl Fournisseur for Cli {
    fn nom(&self) -> String {
        self.config.commande.clone()
    }

    fn proposer(&self, d: &Demande) -> Result<Reponse> {
        let debut = Instant::now();
        let depart = d
            .jeton
            .unwrap_or_else(|| self.annulations.load(Ordering::SeqCst));
        let mut derniere = String::new();
        // Une relance si la réponse manque ou ne respecte pas le schéma (SPEC-19 R4).
        for _ in 0..2 {
            let (donnees, consommation) = self.un_appel(d, depart)?;
            match donnees {
                Some(v) => match valider(d.schema, &v, "réponse") {
                    Ok(()) => {
                        let elements = v
                            .get("elements")
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default();
                        return Ok(Reponse {
                            elements,
                            consommation,
                            duree_ms: debut.elapsed().as_millis(),
                        });
                    }
                    Err(e) => derniere = e.to_string(),
                },
                None => derniere = "réponse structurée absente".into(),
            }
        }
        bail!("le LLM n'a pas rendu une réponse valide après une relance : {derniere}")
    }

    fn annuler(&self) {
        self.annulations.fetch_add(1, Ordering::SeqCst);
    }

    fn jeton(&self) -> u64 {
        self.annulations.load(Ordering::SeqCst)
    }
}

/// Arrête le CLI et tout son groupe de processus (sous Unix), puis attend sa fin.
fn arreter(enfant: &mut std::process::Child) {
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-TERM", &format!("-{}", enfant.id())])
        .status();
    let _ = enfant.kill();
    let _ = enfant.wait();
}

/// Construit le fournisseur décrit par la configuration.
pub fn depuis_config(
    config: &ConfigLlm,
    racine: &Path,
    dossier_travail: &Path,
) -> Result<Box<dyn Fournisseur>> {
    match config.fournisseur.as_str() {
        "factice" => Ok(Box::new(Factice)),
        "cli" => Ok(Box::new(Cli::preparer(
            config.clone(),
            racine,
            dossier_travail,
        )?)),
        autre => bail!("fournisseur inconnu : {autre} (cli ou factice)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> Value {
        json!({"type":"object","properties":{"elements":{"type":"array","minItems":1,"items":{"type":"object","properties":{"titre":{"type":"string"}},"required":["titre"]}}},"required":["elements"]})
    }

    /// Faux CLI : un script shell qui écrit un flux, pour tester l'analyse et la relance sans LLM.
    fn faux_cli(dossier: &Path, corps: &str) -> ConfigLlm {
        let script = dossier.join("faux.sh");
        std::fs::write(&script, format!("#!/bin/sh\n{corps}\n")).unwrap();
        ConfigLlm {
            fournisseur: "cli".into(),
            commande: "sh".into(),
            arguments: vec![script.to_string_lossy().into(), "{consigne}".into()],
            delai: 5,
        }
    }

    #[test]
    fn flux_et_consommation() {
        let d = tempfile::tempdir().unwrap();
        let config = faux_cli(
            d.path(),
            r#"echo '{"type":"rate_limit_event","rate_limit_info":{"unifiedWindows":{"five_hour":{"utilization":0.46},"seven_day":{"utilization":0.64}}}}'
echo '{"type":"result","is_error":false,"structured_output":{"elements":[{"titre":"Alerte"}]}}'"#,
        );
        let cli = Cli::preparer(config, d.path(), &d.path().join("travail")).unwrap();
        let s = schema();
        let r = cli
            .proposer(&Demande {
                consigne: "x",
                schema: &s,
                exemple: None,
                jeton: None,
            })
            .unwrap();
        assert_eq!(r.elements[0]["titre"], "Alerte");
        assert_eq!(
            r.consommation,
            Some(Consommation {
                cinq_heures: 0.46,
                sept_jours: 0.64
            })
        );
    }

    #[test]
    fn une_seule_relance() {
        let d = tempfile::tempdir().unwrap();
        let compteur = d.path().join("appels");
        let config = faux_cli(
            d.path(),
            &format!(
                r#"echo x >> {}
echo '{{"type":"result","is_error":false}}'"#,
                compteur.display()
            ),
        );
        let cli = Cli::preparer(config, d.path(), &d.path().join("travail")).unwrap();
        let s = schema();
        let e = cli
            .proposer(&Demande {
                consigne: "x",
                schema: &s,
                exemple: None,
                jeton: None,
            })
            .unwrap_err();
        assert!(e.to_string().contains("après une relance"), "{e}");
        assert_eq!(
            std::fs::read_to_string(compteur).unwrap().lines().count(),
            2
        );
    }

    /// Un appel annulé s'arrête tout de suite, sans attendre le délai ni relancer.
    #[test]
    fn annulation() {
        let d = tempfile::tempdir().unwrap();
        let config = faux_cli(d.path(), "sleep 4; echo fini");
        let cli = std::sync::Arc::new(
            Cli::preparer(config, d.path(), &d.path().join("travail")).unwrap(),
        );
        let c2 = cli.clone();
        let t = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            c2.annuler();
        });
        let debut = Instant::now();
        let s = schema();
        let e = cli
            .proposer(&Demande {
                consigne: "x",
                schema: &s,
                exemple: None,
                jeton: None,
            })
            .unwrap_err();
        t.join().unwrap();
        assert!(e.to_string().contains("annulé"), "{e}");
        assert!(debut.elapsed() < Duration::from_secs(2));
    }
}

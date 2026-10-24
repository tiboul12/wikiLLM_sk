//! Connecteurs LLM (ADR-0015, EF-155) : pour chaque fournisseur, la commande à lancer, la façon de lire
//! sa réponse, ses modèles et ses niveaux d'effort. Les CLI d'abonnement (Claude Code, Antigravity,
//! Mistral Vibe) sont lancés tels quels ; les modèles locaux (Ollama, llama.cpp) sont joints en HTTP par
//! `curl`, sans dépendance de plus. Aucune clé d'API n'est gérée ici.
//!
//! @req EF-155
//! @adr 15

use crate::config::ConfigLlm;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::{Value, json};
use std::process::Command;
use ts_rs::TS;

/// Façon de lire la sortie d'un appel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lecture {
    /// Flux `stream-json` de Claude Code : `structured_output` du résultat, consommation de l'abonnement.
    FluxClaude,
    /// Texte libre qui contient un objet JSON (Vibe).
    TexteJson,
    /// Sortie `--output-format json` d'Antigravity : `structured_output`, sinon `response`.
    Agy,
    /// Réponse de `/api/chat` d'Ollama.
    Ollama,
    /// Réponse de `/v1/chat/completions` (llama.cpp, compatible OpenAI).
    OpenAi,
}

/// Un appel prêt à lancer.
#[derive(Debug, Clone)]
pub struct Appel {
    pub programme: String,
    pub arguments: Vec<String>,
    pub environnement: Vec<(String, String)>,
    /// Corps envoyé sur l'entrée standard (requête HTTP de `curl`).
    pub entree: Option<String>,
    pub lecture: Lecture,
}

/// Un connecteur proposé dans les Paramètres.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DescriptionConnecteur {
    pub id: String,
    pub nom: String,
    /// Ce qu'il faut savoir en une phrase (abonnement, MCP, serveur local).
    pub aide: String,
    /// Le LLM lit le projet par les outils MCP de wikik ; sinon la carte du projet lui est donnée.
    pub mcp: bool,
    /// Niveaux d'effort acceptés (vide : pas de réglage).
    pub efforts: Vec<String>,
    /// Le connecteur parle à un serveur local (champ adresse).
    pub serveur: bool,
    pub commande_defaut: String,
    pub url_defaut: String,
}

/// Un modèle proposé par un fournisseur.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Modele {
    pub id: String,
    pub libelle: String,
}

/// Les connecteurs, dans l'ordre de la liste des Paramètres ; les adresses locales par défaut sont celles
/// de Shelley Administration quand sa configuration est trouvée sur la machine.
pub fn connecteurs() -> Vec<DescriptionConnecteur> {
    let (ollama, llama) = adresses_shelley();
    let d = |id: &str,
             nom: &str,
             aide: &str,
             mcp: bool,
             efforts: &[&str],
             serveur: bool,
             commande: &str,
             url: &str| DescriptionConnecteur {
        id: id.into(),
        nom: nom.into(),
        aide: aide.into(),
        mcp,
        efforts: efforts.iter().map(|e| e.to_string()).collect(),
        serveur,
        commande_defaut: commande.into(),
        url_defaut: url.into(),
    };
    vec![
        d(
            "claude",
            "Claude Code",
            "Le CLI `claude` déjà connecté (abonnement) ; il lit le projet par les outils MCP de wikik.",
            true,
            &["low", "medium", "high", "xhigh", "max"],
            false,
            "claude",
            "",
        ),
        d(
            "agy",
            "Antigravity",
            "Le CLI `agy` déjà connecté ; sans MCP : la carte du projet lui est donnée dans la demande.",
            false,
            &["low", "medium", "high"],
            false,
            "agy",
            "",
        ),
        d(
            "vibe",
            "Mistral Vibe",
            "Le CLI `vibe` déjà configuré (`vibe --setup`) ; sans MCP : la carte du projet lui est donnée dans la demande.",
            false,
            &[],
            false,
            "vibe",
            "",
        ),
        d(
            "ollama",
            "Ollama (modèle local)",
            "Un serveur Ollama (celui de Shelley par défaut) ; sans MCP : la carte du projet lui est donnée.",
            false,
            &[],
            true,
            "curl",
            &ollama,
        ),
        d(
            "llama-cpp",
            "llama.cpp (modèle local)",
            "Un serveur llama.cpp (`llama-server`, API compatible OpenAI) ; sans MCP : la carte du projet lui est donnée.",
            false,
            &[],
            true,
            "curl",
            &llama,
        ),
        d(
            "cli",
            "Commande libre",
            "Une commande et ses arguments écrits à la main (jetons {consigne}, {schema}, {mcp}).",
            false,
            &[],
            false,
            "claude",
            "",
        ),
        d(
            "factice",
            "Factice (démonstration)",
            "Renvoie les exemples des fiches, sans LLM : pour essayer l'atelier ou tester.",
            false,
            &[],
            false,
            "",
            "",
        ),
    ]
}

/// Adresses d'Ollama et de llama.cpp : celles de Shelley Administration (`ai-config.json`) si elle est
/// installée sur la machine, sinon les valeurs par défaut des deux serveurs.
fn adresses_shelley() -> (String, String) {
    let mut ollama = "http://localhost:11434".to_string();
    let mut llama = "http://127.0.0.1:8090".to_string();
    let maison = std::env::var("HOME").unwrap_or_default();
    for base in [
        format!("{maison}/Shelley"),
        "/opt/Shelley".into(),
        "/srv/Shelley".into(),
    ] {
        let chemin = format!("{base}/administration/backend/data/ai-config.json");
        let Ok(t) = std::fs::read_to_string(&chemin) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<Value>(&t) else {
            continue;
        };
        if let Some(u) = v.get("ollamaUrl").and_then(Value::as_str) {
            ollama = u.trim_end_matches('/').to_string();
        }
        if let (Some(h), Some(p)) = (
            v.pointer("/llamaCpp/host").and_then(Value::as_str),
            v.pointer("/llamaCpp/port").and_then(Value::as_u64),
        ) {
            llama = format!("http://{h}:{p}");
        }
        break;
    }
    (ollama, llama)
}

/// Description d'un connecteur par identifiant.
fn description(id: &str) -> Option<DescriptionConnecteur> {
    connecteurs().into_iter().find(|c| c.id == id)
}

/// Le connecteur lit-il le projet par MCP ? Sinon, la demande doit porter le contexte.
pub fn avec_mcp(config: &ConfigLlm) -> bool {
    matches!(config.connecteur().as_str(), "claude" | "cli")
}

/// Commande à lancer : celle de la configuration, sauf si elle est vide ou si c'est `claude` (valeur par défaut héritée)
/// pour un autre connecteur.
fn commande(config: &ConfigLlm, defaut: &str) -> String {
    let c = config.commande.trim();
    // `claude` est la commande par défaut de la configuration : elle ne vaut que pour Claude Code.
    if c.is_empty() || (c == "claude" && defaut != "claude") {
        defaut.to_string()
    } else {
        c.to_string()
    }
}

/// Adresse d'Ollama pour les embeddings (ADR-0020) : celle du connecteur s'il est réglé sur Ollama, sinon celle de
/// Shelley ou `localhost:11434`.
pub fn adresse_ollama(config: &ConfigLlm) -> String {
    if config.connecteur() == "ollama" {
        url(config, "ollama")
    } else {
        adresses_shelley().0
    }
}

/// Adresse de l'API : celle de la configuration sans `/` final, sinon celle par défaut du connecteur.
fn url(config: &ConfigLlm, id: &str) -> String {
    let u = config.url.trim().trim_end_matches('/');
    if u.is_empty() {
        description(id).map(|d| d.url_defaut).unwrap_or_default()
    } else {
        u.to_string()
    }
}

/// Consigne complétée pour un connecteur qui ne connaît pas le schéma : il doit répondre en JSON pur.
fn avec_schema(consigne: &str, schema: &str) -> String {
    format!(
        "{consigne}\n\nRéponds uniquement par un objet JSON valide, sans texte autour ni bloc de code, conforme à ce schéma JSON :\n{schema}"
    )
}

/// Prépare l'appel du connecteur de `config` pour `consigne` et `schema` ; `mcp` : fichier
/// `--mcp-config` qui pointe sur le serveur MCP de wikik.
pub fn preparer(config: &ConfigLlm, consigne: &str, schema: &Value, mcp: &str) -> Result<Appel> {
    let texte_schema = serde_json::to_string(schema)?;
    let modele = config.modele.trim();
    let effort = config.effort.trim();
    let mut options = vec![];
    if !modele.is_empty() {
        options.extend(["--model".to_string(), modele.to_string()]);
    }
    if !effort.is_empty() {
        options.extend(["--effort".to_string(), effort.to_string()]);
    }
    let appel = match config.connecteur().as_str() {
        "claude" => {
            let mut a: Vec<String> = ConfigLlm::default()
                .arguments
                .iter()
                .map(|x| {
                    x.replace("{consigne}", consigne)
                        .replace("{schema}", &texte_schema)
                        .replace("{mcp}", mcp)
                })
                .collect();
            a.extend(options);
            Appel {
                programme: commande(config, "claude"),
                arguments: a,
                environnement: vec![],
                entree: None,
                lecture: Lecture::FluxClaude,
            }
        }
        "cli" => Appel {
            programme: commande(config, "claude"),
            arguments: config
                .arguments
                .iter()
                .map(|x| {
                    x.replace("{consigne}", consigne)
                        .replace("{schema}", &texte_schema)
                        .replace("{mcp}", mcp)
                })
                .collect(),
            environnement: vec![],
            entree: None,
            lecture: Lecture::FluxClaude,
        },
        "agy" => {
            let mut a = vec![
                "-p".to_string(),
                avec_schema(consigne, &texte_schema),
                "--output-format".into(),
                "json".into(),
                "--json-schema".into(),
                texte_schema.clone(),
                "--disable-slash-commands".into(),
                "--print-timeout".into(),
                format!("{}s", config.delai.max(1)),
            ];
            a.extend(options);
            Appel {
                programme: commande(config, "agy"),
                arguments: a,
                environnement: vec![],
                entree: None,
                lecture: Lecture::Agy,
            }
        }
        "vibe" => Appel {
            programme: commande(config, "vibe"),
            arguments: vec![
                "-p".into(),
                avec_schema(consigne, &texte_schema),
                "--output".into(),
                "text".into(),
                "--max-turns".into(),
                "3".into(),
                "--trust".into(),
            ],
            // Vibe n'a pas d'option de modèle : son réglage `active_model` se surcharge par l'environnement.
            environnement: if modele.is_empty() {
                vec![]
            } else {
                vec![("VIBE_ACTIVE_MODEL".into(), modele.into())]
            },
            entree: None,
            lecture: Lecture::TexteJson,
        },
        "ollama" => {
            let corps = json!({
                "model": modele,
                "stream": false,
                "format": schema,
                "messages": [{ "role": "user", "content": avec_schema(consigne, &texte_schema) }],
            });
            curl(
                &format!("{}/api/chat", url(config, "ollama")),
                &corps,
                config.delai,
                Lecture::Ollama,
            )
        }
        "llama-cpp" => {
            let mut corps = json!({
                "messages": [{ "role": "user", "content": avec_schema(consigne, &texte_schema) }],
                "response_format": { "type": "json_schema", "json_schema": { "name": "reponse", "schema": schema } },
            });
            if !modele.is_empty() {
                corps["model"] = json!(modele);
            }
            curl(
                &format!("{}/v1/chat/completions", url(config, "llama-cpp")),
                &corps,
                config.delai,
                Lecture::OpenAi,
            )
        }
        autre => bail!("connecteur sans appel : {autre}"),
    };
    Ok(appel)
}

/// Appel HTTP par `curl` : corps JSON envoyé sur l'entrée (jamais en argument, visible dans la liste des processus),
/// délai maximal, et le corps de la réponse gardé même en erreur HTTP pour expliquer l'échec.
fn curl(adresse: &str, corps: &Value, delai: u64, lecture: Lecture) -> Appel {
    Appel {
        programme: "curl".into(),
        arguments: vec![
            "-sS".into(),
            "--fail-with-body".into(),
            "--max-time".into(),
            delai.max(1).to_string(),
            "-H".into(),
            "Content-Type: application/json".into(),
            "--data-binary".into(),
            "@-".into(),
            adresse.into(),
        ],
        environnement: vec![],
        entree: Some(corps.to_string()),
        lecture,
    }
}

/// Premier objet JSON d'un texte (le modèle ajoute parfois une phrase ou un bloc de code).
pub fn extraire_json(texte: &str) -> Option<Value> {
    let debut = texte.find('{')?;
    let fin = texte.rfind('}')?;
    (fin > debut)
        .then(|| serde_json::from_str(&texte[debut..=fin]).ok())
        .flatten()
}

/// Lit la sortie complète d'un appel qui n'est pas un flux Claude : la réponse structurée, ou l'erreur
/// donnée par le serveur.
pub fn lire_sortie(lecture: Lecture, sortie: &str) -> (Option<Value>, Option<String>) {
    match lecture {
        Lecture::FluxClaude => (None, None),
        Lecture::Agy => {
            let Some(v) = extraire_json(sortie) else {
                return (None, Some(format!("réponse illisible : {}", court(sortie))));
            };
            if v.get("status")
                .and_then(Value::as_str)
                .is_some_and(|s| s != "SUCCESS")
            {
                return (
                    None,
                    Some(format!("Antigravity : {}", court(&v.to_string()))),
                );
            }
            match v.get("structured_output").filter(|x| !x.is_null()) {
                Some(x) => (Some(x.clone()), None),
                None => match v
                    .get("response")
                    .and_then(Value::as_str)
                    .and_then(extraire_json)
                {
                    Some(x) => (Some(x), None),
                    None => (
                        None,
                        Some(format!("réponse sans objet JSON : {}", court(sortie))),
                    ),
                },
            }
        }
        Lecture::TexteJson => match extraire_json(sortie) {
            Some(v) => (Some(v), None),
            None => (
                None,
                Some(format!("réponse sans objet JSON : {}", court(sortie))),
            ),
        },
        Lecture::Ollama | Lecture::OpenAi => {
            let Ok(v) = serde_json::from_str::<Value>(sortie) else {
                return (None, Some(format!("réponse illisible : {}", court(sortie))));
            };
            if let Some(e) = v.get("error") {
                let m = e
                    .get("message")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| e.to_string());
                return (None, Some(m));
            }
            let contenu = if lecture == Lecture::Ollama {
                v.pointer("/message/content")
            } else {
                v.pointer("/choices/0/message/content")
            };
            match contenu.and_then(Value::as_str).and_then(extraire_json) {
                Some(x) => (Some(x), None),
                None => (
                    None,
                    Some(format!("réponse sans objet JSON : {}", court(sortie))),
                ),
            }
        }
    }
}

/// Extrait de réponse coupé à 300 caractères, pour un message d'erreur lisible.
fn court(t: &str) -> String {
    let t = t.trim();
    if t.chars().count() > 300 {
        format!("{}…", t.chars().take(300).collect::<String>())
    } else {
        t.to_string()
    }
}

/// Modèles proposés par le connecteur : alias documentés pour Claude Code, `agy models`, `/api/tags`
/// d'Ollama, `/v1/models` de llama.cpp ; aucun pour Vibe et la commande libre (champ libre).
pub fn modeles(config: &ConfigLlm) -> Result<Vec<Modele>> {
    let m = |id: &str, l: &str| Modele {
        id: id.into(),
        libelle: l.into(),
    };
    match config.connecteur().as_str() {
        "claude" => Ok(vec![
            m("opus", "Opus (le plus capable)"),
            m("sonnet", "Sonnet (équilibré)"),
            m("haiku", "Haiku (le plus rapide)"),
            m("fable", "Fable"),
        ]),
        "agy" => {
            let s = Command::new(commande(config, "agy"))
                .arg("models")
                .output()
                .context("lancement de `agy models`")?;
            if !s.status.success() {
                bail!("agy models : {}", String::from_utf8_lossy(&s.stderr).trim());
            }
            Ok(String::from_utf8_lossy(&s.stdout)
                .lines()
                .filter_map(|l| {
                    let (id, libelle) = l.split_once('\t')?;
                    Some(m(id.trim(), libelle.trim()))
                })
                .collect())
        }
        "ollama" | "llama-cpp" => {
            let ollama = config.connecteur() == "ollama";
            let adresse = if ollama {
                format!("{}/api/tags", url(config, "ollama"))
            } else {
                format!("{}/v1/models", url(config, "llama-cpp"))
            };
            let s = Command::new("curl")
                .args(["-sS", "--fail", "--max-time", "5", &adresse])
                .output()
                .context("lancement de curl")?;
            if !s.status.success() {
                bail!(
                    "{adresse} ne répond pas : {}",
                    String::from_utf8_lossy(&s.stderr).trim()
                );
            }
            let v: Value = serde_json::from_slice(&s.stdout).context("réponse illisible")?;
            let liste = if ollama {
                v.get("models")
            } else {
                v.get("data")
            };
            Ok(liste
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|x| {
                            let id = x.get(if ollama { "name" } else { "id" })?.as_str()?;
                            Some(m(id, id))
                        })
                        .collect()
                })
                .unwrap_or_default())
        }
        _ => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(connecteur: &str) -> ConfigLlm {
        ConfigLlm {
            connecteur: Some(connecteur.into()),
            modele: "sonnet".into(),
            effort: "medium".into(),
            ..ConfigLlm::default()
        }
    }

    /// Claude Code garde ses arguments (MCP, schéma) et reçoit le modèle et l'effort choisis.
    #[test]
    fn claude_modele_et_effort() {
        let a = preparer(
            &config("claude"),
            "consigne",
            &json!({"type":"object"}),
            "/m.json",
        )
        .unwrap();
        assert_eq!(a.programme, "claude");
        let t = a.arguments.join(" ");
        assert!(
            t.contains("--mcp-config /m.json")
                && t.contains("--model sonnet")
                && t.contains("--effort medium"),
            "{t}"
        );
        assert_eq!(a.lecture, Lecture::FluxClaude);
    }

    /// Ollama : requête HTTP par curl, schéma dans `format`, adresse par défaut.
    #[test]
    fn ollama_par_curl() {
        let mut c = config("ollama");
        c.url = "http://shelley:11434/".into();
        let a = preparer(&c, "x", &json!({"type":"object"}), "").unwrap();
        assert_eq!(a.programme, "curl");
        assert!(a.arguments.last().unwrap() == "http://shelley:11434/api/chat");
        let corps: Value = serde_json::from_str(a.entree.as_deref().unwrap()).unwrap();
        assert_eq!(corps["model"], "sonnet");
        assert_eq!(corps["format"]["type"], "object");
    }

    /// Une ancienne configuration (`fournisseur`, arguments à la main) garde son comportement.
    #[test]
    fn connecteur_deduit() {
        assert_eq!(ConfigLlm::default().connecteur(), "claude");
        let f = ConfigLlm {
            fournisseur: "factice".into(),
            ..ConfigLlm::default()
        };
        assert_eq!(f.connecteur(), "factice");
        let c = ConfigLlm {
            arguments: vec!["x".into()],
            ..ConfigLlm::default()
        };
        assert_eq!(c.connecteur(), "cli");
    }

    #[test]
    fn lecture_des_reponses() {
        let (v, _) = lire_sortie(
            Lecture::TexteJson,
            "Voici :\n```json\n{\"elements\":[1]}\n```",
        );
        assert_eq!(v.unwrap()["elements"][0], 1);
        let (v, _) = lire_sortie(
            Lecture::Agy,
            r#"{"status":"SUCCESS","response":"x","structured_output":{"elements":[2]}}"#,
        );
        assert_eq!(v.unwrap()["elements"][0], 2);
        let (v, _) = lire_sortie(
            Lecture::Ollama,
            r#"{"message":{"content":"{\"elements\":[]}"}}"#,
        );
        assert!(v.unwrap()["elements"].is_array());
        let (v, e) = lire_sortie(Lecture::OpenAi, r#"{"error":{"message":"modèle absent"}}"#);
        assert!(v.is_none() && e.unwrap().contains("modèle absent"));
    }
}

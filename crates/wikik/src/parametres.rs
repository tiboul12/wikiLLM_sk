//! Routes de la section Paramètres (EF-147, EF-155, EF-156) : lecture libre ; écriture, test de
//! connexion, liste des modèles et skills réservés au propriétaire (ADR-0013), car le serveur exécute la
//! commande du LLM et écrit dans le projet. Le LLM change sans relancer `wikik` (sauf avec `--factice`,
//! qui garde le fournisseur factice).
//!
//! @req EF-147, EF-155, EF-156
//! @adr 13
//! @adr 15
//! @adr 17

use crate::acces;
use crate::atelier::resultat;
use crate::serve::Partage;
use anyhow::Result;
use axum::extract::{ConnectInfo, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use wikik_core::atelier::connecteurs::{self, DescriptionConnecteur};
use wikik_core::atelier::fournisseur::{self, Demande, Fournisseur, Reponse};
use wikik_core::parametres::{self, Parametres};
use wikik_core::skills::{self, Skill};

/// Paramètres et ce que l'écran doit savoir pour les montrer.
/// @relu 2026-10-22 4c7f066a
#[derive(Serialize)]
struct Vue {
    parametres: Parametres,
    /// Le navigateur peut modifier les paramètres et ouvrir le terminal.
    proprietaire: bool,
    /// `wikik --factice` : le LLM réglé ici ne sert qu'au prochain lancement sans `--factice`.
    factice: bool,
    connecteurs: Vec<DescriptionConnecteur>,
    skills: Vec<Skill>,
}

/// Routes des paramètres ; tout sauf la lecture passe par la garde du propriétaire.
pub(crate) fn routes(etat: Partage) -> Router<Partage> {
    Router::new()
        .route("/api/parametres", get(api_lire).post(api_ecrire))
        .route("/api/parametres/modeles", get(api_modeles))
        .route("/api/parametres/tester", post(api_tester))
        .route("/api/skills", post(api_skill))
        .route("/api/skills/retablir", post(api_skill_retablir))
        .route("/api/skills/installer", post(api_skill_installer))
        .route_layer(axum::middleware::from_fn_with_state(
            etat,
            |s, c, r: axum::extract::Request, n: axum::middleware::Next| async move {
                if r.method() == axum::http::Method::GET && r.uri().path() == "/api/parametres" {
                    n.run(r).await
                } else {
                    acces::reserve(s, c, r, n).await
                }
            },
        ))
}

async fn api_lire(
    State(e): State<Partage>,
    ConnectInfo(adresse): ConnectInfo<SocketAddr>,
    entetes: HeaderMap,
) -> Response {
    let proprietaire = acces::proprietaire(&e, &entetes, adresse);
    let r = tokio::task::spawn_blocking(move || -> Result<Vue> {
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        Ok(Vue {
            parametres: Parametres::de(&i.config),
            proprietaire,
            factice: e.factice,
            connecteurs: connecteurs::connecteurs(),
            skills: skills::lister(&i.racine, &i.config),
        })
    })
    .await;
    resultat(r)
}

async fn api_ecrire(State(e): State<Partage>, Json(p): Json<Parametres>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<bool> {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        let config = parametres::ecrire(&i.racine, &p)?;
        if !e.factice {
            let travail = wikik_core::atelier::parcours::travail(&i.racine);
            let f = fournisseur::depuis_config(&config.atelier.llm, &i.racine, &travail)?;
            e.fournisseur.remplacer(Arc::from(f));
        }
        i.config = config;
        i.mettre_a_jour()?;
        Ok(true)
    })
    .await;
    resultat(r)
}

/// Configuration du LLM décrite par des paramètres de l'écran (pas encore enregistrés).
fn config_llm(p: &Parametres) -> wikik_core::config::ConfigLlm {
    wikik_core::config::ConfigLlm {
        connecteur: Some(p.connecteur.clone()),
        commande: p.commande.clone(),
        arguments: p.arguments.clone(),
        modele: p.modele.clone(),
        effort: p.effort.clone(),
        url: p.url.clone(),
        delai: p.delai,
        ..Default::default()
    }
}

#[derive(Deserialize)]
struct ParamModeles {
    connecteur: String,
    #[serde(default)]
    commande: String,
    #[serde(default)]
    url: String,
}

/// Modèles proposés par un connecteur (EF-155) ; une erreur dit pourquoi la liste manque.
async fn api_modeles(Query(q): Query<ParamModeles>) -> Response {
    let r = tokio::task::spawn_blocking(move || {
        connecteurs::modeles(&wikik_core::config::ConfigLlm {
            connecteur: Some(q.connecteur),
            commande: q.commande,
            url: q.url,
            ..Default::default()
        })
    })
    .await;
    resultat(r)
}

/// Test de connexion (EF-155) : un appel minimal avec les paramètres de l'écran, sans les enregistrer ;
/// rend la réponse et sa durée, ou l'erreur du fournisseur.
async fn api_tester(State(e): State<Partage>, Json(p): Json<Parametres>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<serde_json::Value> {
        let racine = e
            .index
            .lock()
            .unwrap_or_else(|x| x.into_inner())
            .racine
            .clone();
        let travail = wikik_core::atelier::parcours::travail(&racine);
        let f = fournisseur::depuis_config(&config_llm(&p), &racine, &travail)?;
        let schema = json!({
            "type": "object", "required": ["elements"],
            "properties": { "elements": { "type": "array", "items": {
                "type": "object", "required": ["reponse"], "properties": { "reponse": { "type": "string" } }
            }}}
        });
        let exemple = json!([{ "reponse": "connexion factice" }]);
        let debut = std::time::Instant::now();
        let reponse = f.proposer(&Demande {
            consigne: "Test de connexion de wikik : réponds par un seul élément dont `reponse` dit bonjour en cinq mots au plus, sans lire le projet.",
            schema: &schema,
            exemple: Some(&exemple),
            jeton: None,
        });
        Ok(match reponse {
            Ok(r) => json!({
                "ok": true,
                "fournisseur": f.nom(),
                "reponse": r.elements.first().and_then(|x| x.get("reponse")).cloned().unwrap_or_default(),
                "duree_ms": debut.elapsed().as_millis() as u64,
            }),
            Err(err) => json!({ "ok": false, "fournisseur": f.nom(), "erreur": format!("{err:#}") }),
        })
    })
    .await;
    resultat(r)
}

#[derive(Deserialize)]
struct CorpsSkill {
    nom: String,
    #[serde(default)]
    texte: String,
}

/// Écrit un skill modifié dans les Paramètres (EF-156).
async fn api_skill(State(e): State<Partage>, Json(c): Json<CorpsSkill>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Vec<Skill>> {
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        skills::ecrire(&i.racine, &c.nom, &c.texte)?;
        Ok(skills::lister(&i.racine, &i.config))
    })
    .await;
    resultat(r)
}

/// Revient au texte d'origine d'un skill (le fichier du projet est supprimé).
async fn api_skill_retablir(State(e): State<Partage>, Json(c): Json<CorpsSkill>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<Vec<Skill>> {
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        skills::retablir(&i.racine, &c.nom)?;
        Ok(skills::lister(&i.racine, &i.config))
    })
    .await;
    resultat(r)
}

/// Installe un skill dans le projet (avant de l'envoyer à l'agent, qui le trouve dans `.claude/skills/`).
async fn api_skill_installer(State(e): State<Partage>, Json(c): Json<CorpsSkill>) -> Response {
    let r = tokio::task::spawn_blocking(move || -> Result<String> {
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        skills::installer(&i.racine, &i.config, &c.nom)?;
        Ok(skills::corps(&i.racine, &i.config, &c.nom))
    })
    .await;
    resultat(r)
}

/// Fournisseur que les paramètres remplacent à chaud ; un appel en cours finit avec l'ancien.
pub(crate) struct Commutable(RwLock<Arc<dyn Fournisseur>>);

impl Commutable {
    pub(crate) fn new(f: Box<dyn Fournisseur>) -> Commutable {
        Commutable(RwLock::new(Arc::from(f)))
    }

    fn courant(&self) -> Arc<dyn Fournisseur> {
        self.0.read().unwrap_or_else(|x| x.into_inner()).clone()
    }

    fn remplacer(&self, f: Arc<dyn Fournisseur>) {
        *self.0.write().unwrap_or_else(|x| x.into_inner()) = f;
    }
}

impl Fournisseur for Commutable {
    fn nom(&self) -> String {
        self.courant().nom()
    }
    fn proposer(&self, d: &Demande) -> Result<Reponse> {
        self.courant().proposer(d)
    }
    fn annuler(&self) {
        self.courant().annuler();
    }
    fn jeton(&self) -> u64 {
        self.courant().jeton()
    }
}

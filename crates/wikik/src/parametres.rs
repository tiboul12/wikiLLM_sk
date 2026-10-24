//! Routes de la section Paramètres (EF-147) : lecture libre ; écriture réservée au propriétaire
//! (ADR-0013), car la commande du LLM est exécutée par le serveur. Le LLM change sans relancer `wikik`
//! (sauf avec `--factice`, qui garde le fournisseur factice).
//!
//! @req EF-147
//! @adr 13

use crate::acces;
use crate::atelier::resultat;
use crate::serve::Partage;
use anyhow::Result;
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use wikik_core::atelier::fournisseur::{self, Demande, Fournisseur, Reponse};
use wikik_core::parametres::{self, Parametres};

/// Paramètres et ce que l'écran doit savoir pour les montrer.
#[derive(Serialize)]
struct Vue {
    parametres: Parametres,
    /// Le navigateur peut modifier les paramètres et ouvrir le terminal.
    proprietaire: bool,
    /// `wikik --factice` : le LLM réglé ici ne sert qu'au prochain lancement sans `--factice`.
    factice: bool,
}

/// Routes des paramètres ; l'écriture passe par la garde du propriétaire.
pub(crate) fn routes(etat: Partage) -> Router<Partage> {
    Router::new().route(
        "/api/parametres",
        get(api_lire)
            .post(api_ecrire)
            .route_layer(axum::middleware::from_fn_with_state(
                etat,
                |s, c, r: axum::extract::Request, n: axum::middleware::Next| async move {
                    if r.method() == axum::http::Method::GET {
                        n.run(r).await
                    } else {
                        acces::reserve(s, c, r, n).await
                    }
                },
            )),
    )
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

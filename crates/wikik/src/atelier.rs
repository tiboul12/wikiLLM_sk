//! Routes de l'atelier de définition du besoin (SPEC-18 à SPEC-20), section du serveur du projet.
//!
//! Les routes d'écriture passent par le moteur de parcours, qui n'écrit que dans `docs/atelier/`,
//! `docs/projet/` et `.wikik/atelier/`. Elles exigent un corps JSON et, si le navigateur l'envoie, une
//! origine identique à l'hôte : une page tierce ne peut pas écrire dans le projet. L'appel au LLM se fait
//! hors du verrou de l'index : le wiki reste lisible pendant qu'il réfléchit.
//!
//! @req EF-100, EF-104, ENF-007, ENF-008
//! @adr 10

use crate::serve::{Partage, repondre};
use anyhow::Result;
use axum::extract::{Query, Request, State};
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use wikik_core::Index;
use wikik_core::atelier::fournisseur::{Demande, Fournisseur};
use wikik_core::atelier::parcours;

/// Routes de l'atelier (état, pilotage, propositions, décisions, saisies, synthèses, rappel, parcours,
/// clarifications, modification d'un atelier : EF-161), la recherche assistée par IA (EF-152) et
/// l'annulation des appels au LLM. Chaque écriture d'un parcours porte `parcours` (`PARCOURS` par défaut, EF-145).
pub(crate) fn routes() -> Router<Partage> {
    Router::new()
        .route("/api/atelier/etat", get(api_etat))
        .route("/api/atelier/parcours", post(api_parcours))
        .route(
            "/api/atelier/parcours/modifier",
            post(api_modifier_parcours),
        )
        .route("/api/atelier/clarification", post(api_clarification))
        .route("/api/llm/annuler", post(api_annuler))
        .route("/api/atelier/piloter", post(api_piloter))
        .route("/api/atelier/selection", post(api_selection))
        .route("/api/atelier/proposer", post(api_proposer))
        .route("/api/atelier/decider", post(api_decider))
        .route("/api/atelier/saisir", post(api_saisir))
        .route("/api/atelier/synthetiser", post(api_synthetiser))
        .route("/api/atelier/rappel", post(api_rappel))
        .route("/api/recherche/ia", post(api_recherche_ia))
}

/// Corps de `/api/recherche/ia` : la question posée en langage naturel.
#[derive(Deserialize)]
struct CorpsRechercheIa {
    question: String,
}

/// Recherche assistée par IA (EF-152) : le LLM configuré lit le projet et rend les éléments qui
/// répondent, chacun avec sa raison ; annulable comme tout appel au LLM.
async fn api_recherche_ia(State(e): State<Partage>, Json(c): Json<CorpsRechercheIa>) -> Response {
    let jeton = e.fournisseur.jeton();
    let r = tokio::task::spawn_blocking(move || -> Result<Vec<Value>> {
        let (consigne, schema) = {
            let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
            i.mettre_a_jour()?;
            wikik_core::wiki::demande_recherche(&i, &c.question)?
        };
        let exemple = Value::Array(vec![]);
        let reponse = e.fournisseur.proposer(&Demande {
            consigne: &consigne,
            schema: &schema,
            exemple: Some(&exemple),
            jeton: Some(jeton),
        })?;
        Ok(reponse.elements)
    })
    .await;
    resultat(r)
}

/// Une écriture venue d'une autre origine (page tierce ouverte dans le navigateur) est refusée.
pub(crate) async fn meme_origine(requete: Request, suite: Next) -> Response {
    if requete.method() != Method::GET && requete.method() != Method::HEAD {
        let hote = requete
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();
        if let Some(origine) = requete.headers().get(header::ORIGIN) {
            let origine = origine.to_str().unwrap_or_default();
            if origine.split("://").nth(1) != Some(hote) {
                return (StatusCode::FORBIDDEN, "origine refusée").into_response();
            }
        }
    }
    suite.run(requete).await
}

/// Parcours visé quand la requête n'en nomme pas : celui d'origine, pour les clients d'avant les parcours multiples.
fn defaut() -> String {
    parcours::DEFAUT.to_string()
}

/// Paramètres de `/api/atelier/etat`.
#[derive(Deserialize)]
struct ParamEtat {
    #[serde(default = "defaut")]
    parcours: String,
}

/// État du parcours demandé (étapes, artefacts, clarifications) et nom du fournisseur LLM, pour l'écran de l'atelier.
async fn api_etat(State(e): State<Partage>, Query(q): Query<ParamEtat>) -> Response {
    let nom = e.fournisseur.nom();
    repondre(e, move |i| parcours::etat(i, &q.parcours, &nom)).await
}

/// Arrête les appels au LLM en cours, de l'atelier comme de la réception (REC-030) : ils répondent
/// « appel au LLM annulé ». Corps JSON exigé comme toute écriture.
async fn api_annuler(State(e): State<Partage>, Json(_): Json<Value>) -> Response {
    e.fournisseur.annuler();
    Json(true).into_response()
}

/// Corps de création d'un parcours : titre obligatoire, version visée facultative.
#[derive(Deserialize)]
struct NouveauParcours {
    titre: String,
    #[serde(default)]
    version: String,
}

/// Crée un atelier et rend son identifiant (REC-039).
async fn api_parcours(State(e): State<Partage>, Json(p): Json<NouveauParcours>) -> Response {
    ecrire(e, move |i| {
        parcours::creer_parcours(i, &p.titre, &p.version)
    })
    .await
}

/// Corps de modification d'un atelier ; un champ vide laisse la valeur en place.
#[derive(Deserialize)]
struct ModificationParcours {
    parcours: String,
    titre: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    statut: String,
}

/// Titre, version et statut de session d'un atelier (EF-161).
async fn api_modifier_parcours(
    State(e): State<Partage>,
    Json(p): Json<ModificationParcours>,
) -> Response {
    ecrire(e, move |i| {
        parcours::modifier_parcours(i, &p.parcours, &p.titre, &p.version, &p.statut)
    })
    .await
}

/// Corps d'une action sur une clarification.
#[derive(Deserialize)]
struct ActionClarification {
    id: String,
    /// `clore`, `rouvrir` ou `supprimer`.
    action: String,
}

/// Clôt, rouvre ou supprime une clarification (EF-161) ; une action inconnue est refusée en 400.
async fn api_clarification(
    State(e): State<Partage>,
    Json(p): Json<ActionClarification>,
) -> Response {
    ecrire(e, move |i| match p.action.as_str() {
        "clore" => wikik_core::atelier::clore(i, &p.id),
        "rouvrir" => wikik_core::atelier::rouvrir(i, &p.id),
        "supprimer" => wikik_core::atelier::supprimer(i, &p.id),
        autre => anyhow::bail!("action inconnue : {autre}"),
    })
    .await
}

/// Corps du pilotage d'une étape : action (`terminer`, `rouvrir`, `sauter`, `mode`) et mode pour l'action `mode`.
#[derive(Deserialize)]
struct Pilotage {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    action: String,
    mode: Option<String>,
}

/// Change l'état ou le mode d'une étape du parcours.
async fn api_piloter(State(e): State<Partage>, Json(p): Json<Pilotage>) -> Response {
    ecrire(e, move |i| {
        parcours::piloter(i, &p.parcours, &p.etape, &p.action, p.mode.as_deref())
    })
    .await
}

/// Corps d'une sélection : les artefacts retenus par une sous-étape de genre `selection`.
#[derive(Deserialize)]
struct Selection {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    ids: Vec<String>,
}

/// Enregistre la sélection d'une sous-étape, qui fixe les cibles des sous-étapes répétées.
async fn api_selection(State(e): State<Partage>, Json(p): Json<Selection>) -> Response {
    ecrire(e, move |i| {
        parcours::selectionner(i, &p.parcours, &p.etape, &p.sous_etape, &p.ids)
    })
    .await
}

/// Corps d'une demande de propositions : la sous-étape, l'artefact visé si elle est répétée, une indication libre de l'utilisateur.
#[derive(Deserialize)]
struct Proposer {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    entree: Option<String>,
}

/// Prépare sous verrou, appelle le LLM sans, range la réponse sous verrou, dans le parcours demandé ; le
/// jeton d'annulation est pris dès l'arrivée de la requête.
/// @relu 2026-10-19 a34aad18
async fn api_proposer(State(e): State<Partage>, Json(p): Json<Proposer>) -> Response {
    let jeton = e.fournisseur.jeton();
    let r = tokio::task::spawn_blocking(move || -> Result<parcours::Lot> {
        let prep = {
            let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
            i.mettre_a_jour()?;
            parcours::preparer(
                &i,
                &p.parcours,
                &p.etape,
                &p.sous_etape,
                p.cible.as_deref(),
                p.entree.as_deref(),
            )?
        };
        let nom = e.fournisseur.nom();
        let reponse = e.fournisseur.proposer(&Demande {
            consigne: &prep.consigne,
            schema: &prep.schema,
            exemple: prep.exemple.as_ref(),
            jeton: Some(jeton),
        });
        let i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        match reponse {
            Ok(r) => parcours::enregistrer(&i, &prep, &r, &nom),
            Err(err) => {
                let _ = parcours::journaliser(&i.racine, &prep, &nom, Err(&format!("{err:#}")));
                Err(err)
            }
        }
    })
    .await;
    resultat(r)
}

/// Corps d'une décision sur une proposition du LLM, désignée par son rang ; `donnees` porte la version corrigée par l'utilisateur.
#[derive(Deserialize)]
struct Decision {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    indice: usize,
    decision: String,
    donnees: Option<Value>,
}

/// Garde, corrige ou écarte une proposition ; une proposition gardée devient un artefact.
async fn api_decider(State(e): State<Partage>, Json(p): Json<Decision>) -> Response {
    ecrire(e, move |i| {
        parcours::decider(
            i,
            &p.parcours,
            &p.etape,
            &p.sous_etape,
            p.cible.as_deref(),
            p.indice,
            &p.decision,
            p.donnees,
        )
    })
    .await
}

/// Corps d'une saisie de l'utilisateur dans une sous-étape (pitch, écrit libre, champ d'un formulaire).
#[derive(Deserialize)]
struct Saisie {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
    cible: Option<String>,
    donnees: Value,
}

/// Enregistre ce que l'utilisateur a écrit lui-même, sans passer par le LLM.
async fn api_saisir(State(e): State<Partage>, Json(p): Json<Saisie>) -> Response {
    ecrire(e, move |i| {
        parcours::saisir(
            i,
            &p.parcours,
            &p.etape,
            &p.sous_etape,
            p.cible.as_deref(),
            &p.donnees,
        )
    })
    .await
}

/// Corps d'une demande de synthèse.
#[derive(Deserialize)]
struct Synthese {
    #[serde(default = "defaut")]
    parcours: String,
    etape: String,
    sous_etape: String,
}

/// Fait rédiger par le LLM le rapport d'une sous-étape de genre `synthese` (bilan, BDR, roadmap).
async fn api_synthetiser(State(e): State<Partage>, Json(p): Json<Synthese>) -> Response {
    ecrire(e, move |i| {
        parcours::synthetiser(i, &p.parcours, &p.etape, &p.sous_etape)
    })
    .await
}

/// Corps d'un rappel : le sujet flou et, si besoin, la question à poser.
#[derive(Deserialize)]
struct Rappel {
    sujet: String,
    #[serde(default)]
    question: String,
}

/// Ouvre une clarification sur un point flou du besoin, comme l'outil MCP `atelier_rappel`.
async fn api_rappel(State(e): State<Partage>, Json(p): Json<Rappel>) -> Response {
    ecrire(e, move |i| {
        wikik_core::atelier::rappel(i, &p.sujet, &p.question)
    })
    .await
}

/// Écriture sous verrou ; l'index est remis à jour après, pour que la lecture suivante voie le résultat.
pub(crate) async fn ecrire<T, F>(e: Partage, f: F) -> Response
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce(&Index) -> Result<T> + Send + 'static,
{
    let r = tokio::task::spawn_blocking(move || {
        let mut i = e.index.lock().unwrap_or_else(|x| x.into_inner());
        i.mettre_a_jour()?;
        let v = f(&i)?;
        i.mettre_a_jour()?;
        Ok(v)
    })
    .await;
    resultat(r)
}

/// Réponse HTTP d'une tâche bloquante : la valeur en JSON, une erreur métier en 400 `{erreur}` avec sa chaîne de causes,
/// une tâche qui a paniqué en 500.
pub(crate) fn resultat<T: serde::Serialize>(
    r: std::result::Result<Result<T>, tokio::task::JoinError>,
) -> Response {
    match r {
        Ok(Ok(v)) => Json(v).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "erreur": format!("{e:#}") })),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

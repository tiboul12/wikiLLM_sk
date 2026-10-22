//! Phase 5 de la v4 : un parcours complet, du pitch à la roadmap, avec le fournisseur factice (SPEC-18, SPEC-19).

use serde_json::{Value, json};
use wikik_core::Index;
use wikik_core::atelier::fournisseur::{Demande, Factice, Fournisseur};
use wikik_core::atelier::parcours as p;

/// Demande, range la réponse factice et garde toutes les propositions du parcours par défaut ; renvoie
/// les artefacts créés.
fn garder_tout(
    i: &mut Index,
    etape: &str,
    sous: &str,
    cible: Option<&str>,
    entree: Option<&str>,
) -> Vec<String> {
    garder_dans(i, p::DEFAUT, etape, sous, cible, entree)
}

/// Comme [`garder_tout`], dans un parcours donné.
/// @relu 2026-10-19 56e78508
fn garder_dans(
    i: &mut Index,
    pid: &str,
    etape: &str,
    sous: &str,
    cible: Option<&str>,
    entree: Option<&str>,
) -> Vec<String> {
    let prep = p::preparer(i, pid, etape, sous, cible, entree).unwrap();
    let r = Factice
        .proposer(&Demande {
            consigne: &prep.consigne,
            schema: &prep.schema,
            exemple: prep.exemple.as_ref(),
            jeton: None,
        })
        .unwrap();
    let lot = p::enregistrer(i, &prep, &r, "factice").unwrap();
    let mut ids = vec![];
    for n in 0..lot.elements.len() {
        if let Some(id) = p::decider(i, pid, etape, sous, cible, n, "garder", None).unwrap() {
            ids.push(id);
        }
    }
    i.mettre_a_jour().unwrap();
    ids
}

/// Contenu d'un fichier produit ; panique avec son chemin s'il manque.
fn lire(d: &std::path::Path, relatif: &str) -> String {
    std::fs::read_to_string(d.join(relatif)).unwrap_or_else(|_| panic!("{relatif} absent"))
}

/// Le parcours Express de MyRandonnée produit les artefacts de l'atelier et la documentation du projet,
/// reliés de l'exigence jusqu'au verbatim.
///
/// @req EF-101, EF-103, EF-105, EF-107
#[test]
fn parcours_complet_factice() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("docs/projet")).unwrap();
    let mut i = Index::ouvrir(d.path()).unwrap();

    assert!(
        p::preparer(&i, p::DEFAUT, "pitch", "pitch", None, None).is_err(),
        "pas de brouillon sans description"
    );
    assert_eq!(
        garder_tout(
            &mut i,
            "pitch",
            "pitch",
            None,
            Some("Une appli pour randonner sans se perdre.")
        ),
        ["PITCH"]
    );
    assert!(
        lire(d.path(), "docs/atelier/pitch.md").contains("Une appli pour randonner"),
        "la description est gardée"
    );

    let axes = garder_tout(&mut i, "free-writing", "axes", None, None);
    assert_eq!(axes.len(), 4);
    let texte = "Bon. j'ai peur de me perdre quand le brouillard tombe. Et aucune appli ne me dit si le sentier est trop dur pour moi ! \
                 Il faudrait une alerte si la météo tourne.";
    let fw = p::saisir(
        &i,
        p::DEFAUT,
        "free-writing",
        "ecriture",
        Some("AXE-001"),
        &json!({ "texte": texte }),
    )
    .unwrap()
    .unwrap();
    i.mettre_a_jour().unwrap();
    let unites = garder_tout(&mut i, "free-writing", "recolte", Some(&fw), None);
    assert_eq!(unites, ["US-001", "US-002", "US-003"]);
    let us = lire(d.path(), "docs/atelier/unites/US-001.md");
    assert!(
        us.contains("axe: \"AXE-001\""),
        "{us}\n{}",
        lire(d.path(), "docs/atelier/ecrits/FW-001.md")
    );

    garder_tout(&mut i, "carte-mentale", "branches", None, None);
    garder_tout(&mut i, "carte-mentale", "liens", None, None);
    p::synthetiser(&i, p::DEFAUT, "carte-mentale", "carte").unwrap();
    i.mettre_a_jour().unwrap();
    assert!(lire(d.path(), "docs/atelier/carte.md").contains("mindmap"));

    let pb = garder_tout(&mut i, "analogies", "problemes", None, None);
    garder_tout(&mut i, "analogies", "analogies", Some(&pb[0]), None);
    garder_tout(&mut i, "analogies", "solutions", Some(&pb[0]), None);

    p::selectionner(&i, p::DEFAUT, "5w1h", "idees", &["IDEE-001".into()]).unwrap();
    assert!(
        p::selectionner(&i, p::DEFAUT, "5w1h", "idees", &["US-001".into()]).is_err(),
        "une unité n'est pas une idée"
    );
    assert!(
        p::preparer(&i, p::DEFAUT, "5w1h", "questions", Some("IDEE-002"), None).is_err(),
        "hors sélection"
    );
    garder_tout(&mut i, "5w1h", "questions", Some("IDEE-001"), None);
    garder_tout(&mut i, "cubing", "faces", Some("IDEE-001"), None);
    p::selectionner(&i, p::DEFAUT, "scamper", "sujets", &["IDEE-001".into()]).unwrap();
    assert_eq!(
        garder_tout(&mut i, "scamper", "leviers", Some("IDEE-001"), None).len(),
        7
    );
    p::synthetiser(&i, p::DEFAUT, "scamper", "rapport").unwrap();

    let sb = garder_tout(&mut i, "storyboard", "scenarios", None, None);
    for sous in ["cadre", "vignettes", "artefacts"] {
        garder_tout(&mut i, "storyboard", sous, Some(&sb[0]), None);
    }
    p::synthetiser(&i, p::DEFAUT, "storyboard", "recit").unwrap();
    let story = lire(d.path(), "docs/atelier/storyboards/SB-001.md");
    assert!(
        story.contains("## Vignettes")
            && story.contains("SB-001-A1")
            && story.contains("statut: \"retenu\""),
        "{story}"
    );

    let besoins = garder_tout(&mut i, "bilan", "besoins", None, None);
    assert_eq!(besoins, ["B-001", "B-002"]);
    p::saisir(
        &i,
        p::DEFAUT,
        "bilan",
        "tri",
        None,
        &json!({ "besoins": [{ "id": "B-002", "priorite": "should" }] }),
    )
    .unwrap();
    p::synthetiser(&i, p::DEFAUT, "bilan", "bdr").unwrap();
    i.mettre_a_jour().unwrap();
    let ef = garder_tout(&mut i, "exigences", "exigences", Some("B-001"), None);
    assert_eq!(ef, ["EF-001"]);
    p::synthetiser(&i, p::DEFAUT, "exigences", "cahier").unwrap();
    garder_tout(&mut i, "roadmap", "horizons", None, None);
    p::synthetiser(&i, p::DEFAUT, "roadmap", "roadmap").unwrap();
    i.mettre_a_jour().unwrap();

    // Sources réelles seulement : l'exemple cite US-003, IDEE-001 et SB-001, qui existent ici.
    let b1 = lire(d.path(), "docs/projet/besoins/B-001.md");
    assert!(b1.contains("statut: \"proposé\""), "{b1}");
    let ex = wikik_core::query::exigence(&i, "EF-001").unwrap().texte;
    assert!(ex.contains("B-001") && ex.contains("200 m"), "{ex}");
    for f in [
        "bdr-1-analyse.md",
        "bdr-2-definition.md",
        "cahier-des-charges.md",
        "roadmap.md",
    ] {
        assert!(
            d.path().join("docs/projet/majeures/v1").join(f).exists(),
            "{f}"
        );
    }
    let bdr2 = lire(d.path(), "docs/projet/majeures/v1/bdr-2-definition.md");
    let table = &bdr2[bdr2.find("## Besoins").unwrap()..];
    assert!(
        table.find("B-001").unwrap() < table.find("B-002").unwrap(),
        "must avant should"
    );
    // Rubriques des gabarits (templates/) : besoin, BDR 1 et 2, cahier des charges.
    assert!(
        b1.contains("**Qui** : Randonneur occasionnel")
            && b1.contains("**Pourquoi** : « j'ai peur"),
        "{b1}"
    );
    let bdr1 = lire(d.path(), "docs/projet/majeures/v1/bdr-1-analyse.md");
    assert!(
        bdr1.contains("(avec preuves)")
            && bdr1.contains(
                "« j'ai peur de me perdre quand le brouillard tombe » ([[US-001]], émotion)"
            ),
        "{bdr1}"
    );
    for rubrique in [
        "## Périmètre (inclus / exclu)",
        "## Cas d'usage prioritaires",
        "[[SB-001]]",
        "## Critères de succès de la version",
        "## Hypothèses et risques",
    ] {
        assert!(bdr2.contains(rubrique), "{rubrique} absent : {bdr2}");
    }
    let cdc = lire(d.path(), "docs/projet/majeures/v1/cahier-des-charges.md");
    assert!(
        cdc.contains("[[EF-001]]") && cdc.contains("## Exigences non fonctionnelles"),
        "{cdc}"
    );
    let r = wikik_core::check::verifier(&i).unwrap();
    let graves: Vec<_> = r
        .diagnostics
        .iter()
        .filter(|x| x.severite == "erreur" || x.code.contains("sans"))
        .map(|x| (&x.code, &x.fichier, &x.message))
        .collect();
    assert!(graves.is_empty(), "{graves:?}");

    // Clarification rappelée depuis le développement.
    let (clar, _) = wikik_core::atelier::rappel(&i, "EF-001", "Pourquoi 200 m ?").unwrap();
    i.mettre_a_jour().unwrap();
    garder_tout(&mut i, "clarification", "reponse", Some(&clar), None);
    wikik_core::atelier::clore(&i, &clar).unwrap();
    let c = lire(d.path(), "docs/atelier/clarifications/CLAR-001.md");
    assert!(
        c.contains("## Pistes") && c.contains("## Conclusion") && c.contains("etat: \"close\""),
        "{c}"
    );

    let e = p::etat(&i, p::DEFAUT, "factice").unwrap();
    assert_eq!(
        e.projet.as_deref(),
        Some(json_titre(&lire(d.path(), "docs/atelier/pitch.md")).as_str())
    );
    assert!(
        e.etapes.iter().all(|x| x.etat == "en_cours"),
        "toutes les étapes ont été ouvertes"
    );
    // Une proposition déjà décidée ne l'est pas deux fois.
    assert!(p::decider(&i, p::DEFAUT, "bilan", "besoins", None, 0, "garder", None).is_err());
}

/// Titre d'un frontmatter écrit en chaîne JSON (`titre: "…"`), décodé.
fn json_titre(t: &str) -> String {
    t.lines()
        .find_map(|l| l.strip_prefix("titre: "))
        .map(|v| {
            serde_json::from_str::<Value>(v)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .unwrap()
}

/// Un verbatim absent du texte libre est écarté (non-altération).
#[test]
fn verbatim_invente_ecarte() {
    let d = tempfile::tempdir().unwrap();
    let mut i = Index::ouvrir(d.path()).unwrap();
    garder_tout(&mut i, "free-writing", "axes", None, None);
    let fw = p::saisir(
        &i,
        p::DEFAUT,
        "free-writing",
        "ecriture",
        Some("AXE-001"),
        &json!({ "texte": "j'ai peur de me perdre quand le brouillard tombe" }),
    )
    .unwrap()
    .unwrap();
    i.mettre_a_jour().unwrap();
    let prep = p::preparer(&i, p::DEFAUT, "free-writing", "recolte", Some(&fw), None).unwrap();
    let r = Factice
        .proposer(&Demande {
            consigne: &prep.consigne,
            schema: &prep.schema,
            exemple: prep.exemple.as_ref(),
            jeton: None,
        })
        .unwrap();
    let lot = p::enregistrer(&i, &prep, &r, "factice").unwrap();
    assert_eq!(lot.elements.len(), 1);
    assert_eq!(lot.ecartees.len(), 2);
}

/// Deux parcours avancent sans se mélanger : chacun son pitch, ses artefacts, ses étapes (EF-145).
///
/// @req EF-145
#[test]
fn deux_parcours_separes() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("docs/projet")).unwrap();
    let mut i = Index::ouvrir(d.path()).unwrap();
    garder_tout(
        &mut i,
        "pitch",
        "pitch",
        None,
        Some("Randonner sans se perdre."),
    );
    assert!(
        p::creer_parcours(&i, " ", "v2").is_err(),
        "un titre est exigé"
    );
    let p2 = p::creer_parcours(&i, "Appli météo", "v2").unwrap();
    assert_eq!(p2, "PARCOURS-2");
    i.mettre_a_jour().unwrap();
    assert_eq!(
        garder_dans(
            &mut i,
            &p2,
            "pitch",
            "pitch",
            None,
            Some("La météo pour investir.")
        ),
        ["PITCH-2"]
    );
    let pitch2 = lire(d.path(), "docs/atelier/parcours-2/pitch.md");
    assert!(pitch2.contains("parcours: \"PARCOURS-2\""), "{pitch2}");
    let axes = garder_dans(&mut i, &p2, "free-writing", "axes", None, None);
    assert!(lire(d.path(), &format!("docs/atelier/axes/{}.md", axes[0])).contains("PARCOURS-2"));
    p::piloter(&i, &p2, "pitch", "terminer", None).unwrap();
    i.mettre_a_jour().unwrap();

    let e1 = p::etat(&i, p::DEFAUT, "factice").unwrap();
    let e2 = p::etat(&i, &p2, "factice").unwrap();
    let fw = |e: &p::EtatAtelier| {
        e.etapes
            .iter()
            .find(|x| x.fiche.id == "free-writing")
            .unwrap()
            .artefacts
            .len()
    };
    assert_eq!((fw(&e1), fw(&e2)), (0, axes.len()));
    let pitch = |e: &p::EtatAtelier| {
        e.etapes
            .iter()
            .find(|x| x.fiche.id == "pitch")
            .unwrap()
            .etat
            .clone()
    };
    assert_eq!(
        (pitch(&e1).as_str(), pitch(&e2).as_str()),
        ("en_cours", "faite")
    );
    let id_pitch = |e: &p::EtatAtelier| {
        e.etapes[0]
            .artefacts
            .iter()
            .map(|a| a.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        (id_pitch(&e1), id_pitch(&e2)),
        (vec!["PITCH".to_string()], vec!["PITCH-2".to_string()])
    );
    assert_eq!(e2.tous_parcours.len(), 2);
    assert_eq!(
        (
            e2.tous_parcours[1].titre.as_str(),
            e2.tous_parcours[1].faites
        ),
        ("Appli météo", 1)
    );
    assert!(p::etat(&i, "PARCOURS-9", "factice").is_err());
    assert!(p::etat(&i, "../x", "factice").is_err());
}

/// Lancer une clarification sans question reprend celle qui est ouverte ; on la clôt par ses pistes
/// gardées, on la supprime (EF-144).
///
/// @req EF-144
#[test]
fn clarification_sans_question() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("docs/projet/besoins")).unwrap();
    std::fs::write(
        d.path().join("docs/projet/besoins/B-001.md"),
        "---\nid: B-001\ntype: besoin\ntitre: Randonner\nstatut: proposé\n---\n\n# B-001\n",
    )
    .unwrap();
    let mut i = Index::ouvrir(d.path()).unwrap();
    i.mettre_a_jour().unwrap();
    let (c1, _) = wikik_core::atelier::rappel(&i, "B-001", "").unwrap();
    i.mettre_a_jour().unwrap();
    let (c2, _) = wikik_core::atelier::rappel(&i, "B-001", "").unwrap();
    assert_eq!(c1, c2, "la clarification ouverte est reprise");
    assert!(lire(d.path(), "docs/atelier/clarifications/CLAR-001.md").contains("reste flou"));
    assert!(
        wikik_core::atelier::clore(&i, &c1).is_err(),
        "rien de gardé"
    );
    garder_tout(&mut i, "clarification", "reponse", Some(&c1), None);
    wikik_core::atelier::clore(&i, &c1).unwrap();
    let c = lire(d.path(), "docs/atelier/clarifications/CLAR-001.md");
    assert!(
        c.contains("etat: \"close\"") && c.contains("## Conclusion"),
        "{c}"
    );
    i.mettre_a_jour().unwrap();
    let (c3, _) = wikik_core::atelier::rappel(&i, "B-001", "").unwrap();
    assert_eq!(c3, "CLAR-002", "une close n'est pas reprise");
    i.mettre_a_jour().unwrap();
    wikik_core::atelier::supprimer(&i, &c3).unwrap();
    assert!(
        !d.path()
            .join("docs/atelier/clarifications/CLAR-002.md")
            .exists()
    );
}

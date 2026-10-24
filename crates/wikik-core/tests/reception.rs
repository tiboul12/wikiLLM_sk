//! v5 : carte des pages construite à partir du code (SPEC-22).

use std::path::Path;
use wikik_core::Index;
use wikik_core::reception::carte;

/// Copie récursive d'un dossier de fixtures dans un dossier temporaire, pour que le test écrive sans toucher au dépôt.
fn copier(de: &Path, vers: &Path) {
    std::fs::create_dir_all(vers).unwrap();
    for e in std::fs::read_dir(de).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copier(&e.path(), &vers.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), vers.join(e.file_name())).unwrap();
        }
    }
}

/// Index d'une copie de la fixture `reception-mini`.
fn fixture() -> (tempfile::TempDir, Index) {
    let d = tempfile::tempdir().unwrap();
    copier(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/reception-mini"),
        d.path(),
    );
    let mut i = Index::ouvrir(d.path()).unwrap();
    i.mettre_a_jour().unwrap();
    (d, i)
}

/// Un cas par détecteur, sans configuration ; chaque page est reliée à son composant.
///
/// @req EF-120
#[test]
fn un_cas_par_detecteur() {
    let (_d, i) = fixture();
    let pages = carte::detecter(&i).unwrap();
    let vue: Vec<(String, String, Option<String>, String)> = pages
        .iter()
        .map(|p| {
            (
                p.cle.clone(),
                p.nom.clone(),
                p.symboles.first().cloned(),
                p.source.clone(),
            )
        })
        .collect();
    let cherche = |cle: &str| {
        vue.iter()
            .find(|v| v.0 == cle)
            .unwrap_or_else(|| panic!("{cle} absente de {vue:#?}"))
    };
    // Angular : composant et chargement différé ; la redirection est ignorée.
    assert_eq!(
        cherche("/").2.as_deref(),
        Some("ng/src/app/pages/accueil/accueil.component.ts::AccueilComponent")
    );
    assert_eq!(cherche("/reglages").3, "route");
    assert!(!vue.iter().any(|v| v.0 == "/ancien"));
    // React Router, Leptos.
    assert_eq!(
        cherche("/panier").2.as_deref(),
        Some("react/src/App.tsx::Panier")
    );
    assert_eq!(
        cherche("/profil").2.as_deref(),
        Some("leptos/src/lib.rs::Profil")
    );
    // SvelteKit : les groupes `(public)` ne font pas partie de la route.
    assert_eq!(cherche("/blog").3, "fichier-route");
    assert_eq!(cherche("/contact").1, "Contact");
    // Énumération : libellé lu à côté de la variante, composant par son nom ou par la condition d'affichage.
    let biblio = cherche("bibliotheque");
    assert_eq!(
        (biblio.1.as_str(), biblio.2.as_deref()),
        ("Mes documents", Some("etat/src/app.rs::BibliothequePage"))
    );
    assert_eq!(
        cherche("edition").2.as_deref(),
        Some("etat/src/app.rs::EditeurPrincipal")
    );
    // HTML : un document complet, pas un fragment.
    assert_eq!(cherche("site/aide.html").1, "Aide en ligne");
    assert!(!vue.iter().any(|v| v.0 == "site/morceau.html"));
    // Composant rangé dans un dossier de pages.
    assert_eq!(
        cherche("tableau").2.as_deref(),
        Some("svelte/src/pages/Tableau.svelte::Tableau")
    );
    // L'exigence citée par le composant suit la page.
    assert_eq!(
        pages.iter().find(|p| p.cle == "/").unwrap().exigences,
        ["EF-001"]
    );
}

/// Les corrections de Tibo survivent à un nouveau scan ; une page qui disparaît du code est gardée.
///
/// @req EF-121
#[test]
fn corrections_gardees() {
    let (d, mut i) = fixture();
    carte::scanner(&i).unwrap();
    carte::corriger(
        &i,
        "/panier",
        Some("Mon panier"),
        Some(true),
        Some("Boutique"),
        None,
    )
    .unwrap();
    std::fs::write(
        d.path().join("react/src/App.tsx"),
        "export function App() { return null; }\n",
    )
    .unwrap();
    i.mettre_a_jour().unwrap();
    let pages = carte::scanner(&i).unwrap();
    let panier = pages.iter().find(|p| p.cle == "/panier").unwrap();
    assert!(
        panier.nom == "Mon panier" && panier.masque && panier.disparue,
        "{panier:?}"
    );
    assert_eq!(pages[0].cle, carte::GENERAL);
    i.mettre_a_jour().unwrap();
    // La carte est un document de l'index (zone réception).
    let n: i64 = i
        .conn
        .query_row(
            "SELECT count(*) FROM documents WHERE id = 'CARTE-RECEPTION'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 1);
}

/// PNG minimal valide (1 × 1), en data URL, pour joindre une capture.
const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

/// Modifier crée une version numérotée liée à la précédente ; supprimer retire toute la chaîne.
///
/// @req EF-127
#[test]
fn versions_et_suppression() {
    use wikik_core::reception::remarques::{self, Saisie};
    let (d, mut i) = fixture();
    carte::scanner(&i).unwrap();
    let saisie =
        |texte: &'static str, images: &'static [String], garder: &'static [String]| Saisie {
            page: "/panier",
            texte,
            genre: "bug",
            gravite: "genant",
            images,
            garder,
            date: "2026-10-18",
        };
    let une: &'static [String] = Box::leak(Box::new([PNG.to_string()]));
    let id = remarques::creer(&i, &saisie("Total faux", une, &[])).unwrap();
    assert_eq!(id, "REC-001");
    let garder: &'static [String] =
        Box::leak(Box::new(
            ["docs/reception/images/REC-001-1.png".to_string()],
        ));
    let v2 =
        remarques::modifier(&i, &id, &saisie("Total faux après retrait", une, garder)).unwrap();
    assert_eq!(v2, "REC-001.2");
    assert!(
        remarques::modifier(&i, &id, &saisie("x", &[], &[])).is_err(),
        "une version remplacée ne se modifie plus"
    );
    remarques::traiter(&i, &v2, "traitee", "Corrigé.", Some("abc")).unwrap();
    let v3 = remarques::modifier(
        &i,
        &v2,
        &saisie("Total faux après retrait d'un article", &[], &[]),
    )
    .unwrap();
    assert_eq!(v3, "REC-001.3");
    i.mettre_a_jour().unwrap();
    let liste = remarques::lister(&i).unwrap();
    let derniere = liste.iter().find(|r| r.id == v3).unwrap();
    assert_eq!(derniere.historique, ["REC-001", "REC-001.2", "REC-001.3"]);
    assert_eq!(
        (derniere.statut.as_str(), derniere.version),
        ("traitee", 3),
        "le traitement suit la nouvelle version"
    );
    assert_eq!(liste.iter().filter(|r| r.statut == "remplacee").count(), 2);
    assert!(remarques::traiter(&i, "REC-001", "rejetee", "x", None).is_err());
    // Le wiki relie les versions (remplace / remplace_par), sans lien cassé.
    let r = wikik_core::check::verifier(&i).unwrap();
    assert!(
        !r.diagnostics.iter().any(|x| x.code == "lien-casse"),
        "{:?}",
        r.diagnostics
    );
    // Suppression : toute la chaîne et ses images.
    assert_eq!(
        remarques::supprimer(&i, &v2).unwrap(),
        ["REC-001", "REC-001.2", "REC-001.3"]
    );
    let reste: Vec<_> = std::fs::read_dir(d.path().join("docs/reception/remarques"))
        .unwrap()
        .collect();
    assert!(reste.is_empty());
    assert_eq!(
        std::fs::read_dir(d.path().join("docs/reception/images"))
            .unwrap()
            .count(),
        0
    );
}

/// L'adresse de l'application et une route ajoutée à la main survivent à un nouveau scan.
///
/// @req EF-126
#[test]
fn adresse_et_route() {
    let (_d, i) = fixture();
    carte::scanner(&i).unwrap();
    assert!(carte::definir_url_application(&i, "ftp://x").is_err());
    carte::definir_url_application(&i, "http://10.0.0.21:4200/").unwrap();
    carte::corriger(&i, "bibliotheque", None, None, None, Some("bibli")).unwrap();
    carte::scanner(&i).unwrap();
    assert_eq!(
        carte::url_application(&i).as_deref(),
        Some("http://10.0.0.21:4200")
    );
    let p = carte::lire(&i)
        .into_iter()
        .find(|p| p.cle == "bibliotheque")
        .unwrap();
    assert_eq!(p.route.as_deref(), Some("/bibli"));
}

/// Graphe global : chaque page de l'application est un nœud de la catégorie `pages`, relié au fichier
/// de son composant (catégorie `code`).
///
/// @req EF-141
#[test]
fn pages_dans_le_graphe_global() {
    let (_d, i) = fixture();
    let g = wikik_core::wiki::graphe(&i, None, 1).unwrap();
    let panier = g.noeuds.iter().find(|n| n.id == "page:/panier").unwrap();
    assert_eq!(
        (panier.categorie.as_str(), panier.genre.as_str()),
        ("pages", "page")
    );
    let a = g
        .aretes
        .iter()
        .find(|a| a.source == "page:/panier" && a.genre == "rend")
        .unwrap();
    let f = g.noeuds.iter().find(|n| n.id == a.cible).unwrap();
    assert_eq!(
        (f.id.as_str(), f.categorie.as_str()),
        ("react/src/App.tsx", "code")
    );
}

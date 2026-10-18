//! v5 : carte des pages construite à partir du code (SPEC-22).

use std::path::Path;
use wikik_core::Index;
use wikik_core::reception::carte;

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

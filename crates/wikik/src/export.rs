//! `wikik export` : écrit le wiki en site statique (front + données JSON), lisible sans serveur wikik.
//!
//! Les pages sont pré-calculées (ADR-0008) ; le front embarqué passe en mode statique (routage par
//! ancre, lecture de `donnees/`). Même commit, mêmes octets (ENF-002).
//!
//! @req EF-080, ENF-002, ENF-004
//! @adr 8

use crate::serve::Front;
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;
use wikik_core::{Index, check, wiki};

/// Nom de fichier sûr pour un identifiant ou un segment de chemin : tout caractère hors
/// `[A-Za-z0-9._-]` devient `~` suivi de ses octets UTF-8 en hexadécimal (même règle dans le front).
pub fn nom_sur(t: &str) -> String {
    let mut s = String::new();
    for c in t.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            s.push(c);
        } else {
            let mut tampon = [0u8; 4];
            for b in c.encode_utf8(&mut tampon).bytes() {
                s.push_str(&format!("~{b:02x}"));
            }
        }
    }
    s
}

fn chemin_sur(chemin: &str) -> String {
    chemin.split('/').map(nom_sur).collect::<Vec<_>>().join("/")
}

fn ecrire<T: Serialize>(racine: &Path, relatif: &str, valeur: &T) -> Result<()> {
    let chemin = racine.join(relatif);
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(&chemin, serde_json::to_vec(valeur)?)
        .with_context(|| format!("écriture de {}", chemin.display()))
}

/// Écrit le site dans `sortie` (créé au besoin, vidé de ses anciens `donnees/` et `assets/`) : accueil,
/// Produit et Projet, en lecture seule (ni atelier ni réception). Renvoie le nombre de fichiers écrits.
pub fn exporter(racine: &Path, sortie: &Path) -> Result<usize> {
    let mut index = Index::ouvrir(racine)?;
    index.mettre_a_jour()?;
    for d in ["donnees", "assets"] {
        let _ = std::fs::remove_dir_all(sortie.join(d));
    }
    std::fs::create_dir_all(sortie)?;
    std::fs::write(
        sortie.join(wikik_core::index::MARQUEUR_EXPORT),
        "Site généré par `wikik export` : ce dossier n'est jamais indexé.\n",
    )?;
    let mut n = 0;

    // Front : index.html en mode statique, assets en chemins relatifs.
    for chemin in Front::iter() {
        let fichier = Front::get(&chemin).unwrap();
        let destination = sortie.join(chemin.as_ref());
        std::fs::create_dir_all(destination.parent().unwrap())?;
        if chemin == "index.html" {
            let html = String::from_utf8_lossy(&fichier.data)
                .replace("\"/assets/", "\"./assets/")
                .replace(
                    "<head>",
                    "<head>\n    <meta name=\"wikik-statique\" content=\"1\" />",
                );
            std::fs::write(&destination, html)?;
        } else {
            std::fs::write(&destination, &fichier.data)?;
        }
        n += 1;
    }

    let projet = wiki::projet(&index)?;
    ecrire(sortie, "donnees/projet.json", &projet)?;
    ecrire(sortie, "donnees/check.json", &check::verifier(&index)?)?;
    ecrire(
        sortie,
        "donnees/graphe.json",
        &wiki::graphe(&index, None, 1)?,
    )?;
    ecrire(
        sortie,
        "donnees/graphe-complet.json",
        &wiki::graphe_complet(&index)?,
    )?;
    ecrire(
        sortie,
        "donnees/recherche.json",
        &wiki::donnees_recherche(&index)?,
    )?;
    n += 5;
    for f in &projet.fichiers {
        ecrire(
            sortie,
            &format!("donnees/fichiers/{}.json", chemin_sur(&f.chemin)),
            &wiki::fichier(&index, &f.chemin)?,
        )?;
        n += 1;
    }
    for d in &projet.documents {
        ecrire(
            sortie,
            &format!("donnees/documents/{}.json", nom_sur(&d.id)),
            &wiki::document(&index, &d.id)?,
        )?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    #[test]
    fn noms_surs() {
        assert_eq!(super::nom_sur("EF-001"), "EF-001");
        assert_eq!(super::nom_sur("a b/é"), "a~20b~2f~c3~a9");
        assert_eq!(super::chemin_sur("src/a b.rs"), "src/a~20b.rs");
    }
}

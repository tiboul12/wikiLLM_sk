//! `wikik init` : prépare un projet (configuration, zone Projet, gabarits) sans rien écraser.
//!
//! Les gabarits sont embarqués dans le binaire et copiés dans `docs/modeles/`, hors de la zone
//! Projet : leurs identifiants factices (`EF-000`) ne doivent pas être indexés.
//!
//! @req EF-033

use anyhow::Result;
use std::path::Path;

const GABARITS: &[(&str, &str)] = &[
    ("adr.md", include_str!("../../../templates/adr.md")),
    (
        "bdr-1-analyse.md",
        include_str!("../../../templates/bdr-1-analyse.md"),
    ),
    (
        "bdr-2-definition.md",
        include_str!("../../../templates/bdr-2-definition.md"),
    ),
    ("besoin.md", include_str!("../../../templates/besoin.md")),
    (
        "cahier-des-charges.md",
        include_str!("../../../templates/cahier-des-charges.md"),
    ),
    (
        "exigence.md",
        include_str!("../../../templates/exigence.md"),
    ),
    ("spec.md", include_str!("../../../templates/spec.md")),
];

const CONFIG: &str = r#"# Configuration wikik (toutes les clés sont optionnelles).
[index]
# Motifs exclus de l'index, en plus du .gitignore.
exclure = []
langages = ["rust"]

[projet]
dossier = "docs/projet"

[check]
# pub | pub-crate | tout : à partir de quelle visibilité un bloc de doc est exigé.
seuil_visibilite = "pub"

[check.severites]
# Exemple : perime-corps = "erreur"   (erreur | avertissement | info | ignore)
"#;

/// Crée ce qui manque et renvoie les chemins créés ou modifiés (vide au second lancement).
pub fn initialiser(racine: &Path) -> Result<Vec<String>> {
    let mut faits = vec![];
    let mut ecrire = |relatif: &str, contenu: &str| -> Result<()> {
        let chemin = racine.join(relatif);
        if !chemin.exists() {
            if let Some(p) = chemin.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::fs::write(&chemin, contenu)?;
            faits.push(relatif.to_string());
        }
        Ok(())
    };
    ecrire("wikik.toml", CONFIG)?;
    for dossier in ["besoins", "exigences", "adr", "specs", "majeures"] {
        ecrire(&format!("docs/projet/{dossier}/.gitkeep"), "")?;
    }
    for (nom, contenu) in GABARITS {
        ecrire(&format!("docs/modeles/{nom}"), contenu)?;
    }
    let gitignore = racine.join(".gitignore");
    let actuel = std::fs::read_to_string(&gitignore).unwrap_or_default();
    if !actuel
        .lines()
        .any(|l| matches!(l.trim(), ".wikik/" | ".wikik" | "/.wikik/" | "/.wikik"))
    {
        let separateur = if actuel.is_empty() || actuel.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        std::fs::write(
            &gitignore,
            format!("{actuel}{separateur}# Index wikik, reconstructible\n.wikik/\n"),
        )?;
        faits.push(".gitignore".into());
    }
    Ok(faits)
}

#[cfg(test)]
mod tests {
    #[test]
    fn idempotent() {
        let d = tempfile::tempdir().unwrap();
        let premier = super::initialiser(d.path()).unwrap();
        assert!(
            premier.contains(&"wikik.toml".to_string())
                && premier.contains(&".gitignore".to_string())
        );
        assert!(super::initialiser(d.path()).unwrap().is_empty());
        crate::config::Config::charger(d.path()).unwrap();
    }
}

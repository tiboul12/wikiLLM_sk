//! `wikik init` : prépare un projet (configuration, zone Projet, gabarits) sans rien écraser.
//!
//! Les gabarits sont embarqués dans le binaire et copiés dans `docs/modeles/`, hors de la zone
//! Projet : leurs identifiants factices (`EF-000`) ne doivent pas être indexés.
//!
//! @req EF-033, EF-082, EF-083, EF-084

use crate::config::Config;
use anyhow::{Result, bail};
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
langages = ["rust", "typescript", "javascript", "svelte"]

[projet]
dossier = "docs/projet"

[check]
# pub | pub-crate | tout : à partir de quelle visibilité un bloc de doc est exigé.
seuil_visibilite = "pub"

[check.severites]
# Exemple : perime-corps = "erreur"   (erreur | avertissement | info | ignore)

[serveur]
# true : `wikik atelier` et `wikik serve` écoutent sur le réseau local (ouvrir l'adresse affichée au
# démarrage depuis un autre poste). L'atelier écrit dans le projet : à réserver à un réseau de confiance.
reseau = false
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

/// Workflow GitHub : `check` annoté sur chaque PR et push ; export publié sur GitHub Pages depuis la
/// branche principale. wikik est compilé depuis son dépôt (front compris, d'où Node).
const WORKFLOW: &str = r#"# Généré par `wikik init --ci` : doc vérifiée sur chaque PR, wiki publié sur GitHub Pages.
name: wikik
on:
  pull_request:
  push:
    branches: [main, master]
permissions:
  contents: read
env:
  WIKIK_DEPOT: https://github.com/tiboul12/wiki_LLM
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # la doc périmée se juge sur l'historique git
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - name: Installer wikik
        run: git clone --depth 1 "$WIKIK_DEPOT" /tmp/wikik && make -C /tmp/wikik install
      - run: wikik check --format github
  publier:
    if: github.event_name == 'push'
    needs: check
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deploiement.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - name: Installer wikik
        run: git clone --depth 1 "$WIKIK_DEPOT" /tmp/wikik && make -C /tmp/wikik install
      - run: wikik export _site
      - uses: actions/upload-pages-artifact@v3
        with:
          path: _site
      - id: deploiement
        uses: actions/deploy-pages@v4
"#;

/// `wikik init --ci` : workflow GitHub et `CODEOWNERS` des validateurs ; ne réécrit rien d'existant.
pub fn ci(racine: &Path, config: &Config) -> Result<Vec<String>> {
    let mut faits = vec![];
    let workflow = racine.join(".github/workflows/wikik.yml");
    if !workflow.exists() {
        std::fs::create_dir_all(workflow.parent().unwrap())?;
        std::fs::write(&workflow, WORKFLOW)?;
        faits.push(".github/workflows/wikik.yml".to_string());
    }
    let validateurs = &config.equipe.validateurs;
    let codeowners = racine.join(".github/CODEOWNERS");
    if !validateurs.is_empty() && !codeowners.exists() {
        let dossier = config.projet.dossier.trim_end_matches('/');
        let qui = validateurs.join(" ");
        std::fs::write(
            &codeowners,
            format!(
                "# Généré par `wikik init --ci` : les documents produit exigent l'approbation des validateurs\n# (wikik.toml, [equipe] validateurs).\n/{dossier}/majeures/ {qui}\n/{dossier}/exigences/ {qui}\n/{dossier}/besoins/ {qui}\n"
            ),
        )?;
        faits.push(".github/CODEOWNERS".to_string());
    }
    Ok(faits)
}

/// `wikik init --hook` : hook `pre-commit` qui lance `wikik check` ; refuse d'écraser un hook existant.
pub fn hook(racine: &Path) -> Result<String> {
    let sortie = std::process::Command::new("git")
        .arg("-C")
        .arg(racine)
        .args(["rev-parse", "--git-path", "hooks"])
        .output()?;
    if !sortie.status.success() {
        bail!("pas de dépôt git dans {}", racine.display());
    }
    let dossier = racine.join(String::from_utf8_lossy(&sortie.stdout).trim());
    let chemin = dossier.join("pre-commit");
    if chemin.exists() {
        bail!(
            "un hook pre-commit existe déjà ({}) : y ajouter la ligne `wikik check || exit 1`",
            chemin.display()
        );
    }
    std::fs::create_dir_all(&dossier)?;
    std::fs::write(
        &chemin,
        "#!/bin/sh\n# Installé par `wikik init --hook` : refuse un commit qui casse la doc (erreurs de `wikik check`).\nexec wikik check\n",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(chemin.display().to_string())
}

/// `wikik nouveau` : crée le fichier du prochain identifiant libre à partir du gabarit (SPEC-16 R6).
pub fn nouveau(racine: &Path, config: &Config, genre: &str, titre: &str) -> Result<String> {
    let (prefixe, largeur, dossier, gabarit, type_doc) = match genre {
        "exigence" => ("EF-", 3, "exigences", "exigence.md", "exigence"),
        "exigence-nf" => ("ENF-", 3, "exigences", "exigence.md", "exigence-nf"),
        "besoin" => ("B-", 3, "besoins", "besoin.md", "besoin"),
        "adr" => ("ADR-", 4, "adr", "adr.md", "adr"),
        "spec" => ("SPEC-", 2, "specs", "spec.md", "spec"),
        autre => bail!("genre inconnu : {autre} (exigence, exigence-nf, besoin, adr, spec)"),
    };
    let base = racine.join(config.projet.dossier.trim_end_matches('/'));
    let mut max = 0u32;
    // Plus grand numéro déjà utilisé, lu dans les noms et les frontmatters (fichiers non encore indexés compris).
    if let Ok(entrees) = std::fs::read_dir(base.join(dossier)) {
        for e in entrees.flatten() {
            let nom = e.file_name().to_string_lossy().to_string();
            let texte = std::fs::read_to_string(e.path()).unwrap_or_default();
            for candidat in std::iter::once(nom.as_str()).chain(
                texte
                    .lines()
                    .filter_map(|l| l.strip_prefix("id:"))
                    .map(str::trim),
            ) {
                let c = candidat.trim_start_matches(prefixe);
                let chiffres: String = c.chars().take_while(|ch| ch.is_ascii_digit()).collect();
                if (candidat.starts_with(prefixe) || genre == "adr") && !chiffres.is_empty() {
                    max = max.max(chiffres.parse().unwrap_or(0));
                }
            }
        }
    }
    let numero = format!("{:0largeur$}", max + 1);
    let id = format!("{prefixe}{numero}");
    let slug = crate::query::ancre(titre);
    let fichier = match genre {
        "adr" => format!("{dossier}/{numero}-{slug}.md"),
        "spec" => format!("{dossier}/{id}-{slug}.md"),
        _ => format!("{dossier}/{id}.md"),
    };
    let modele = GABARITS
        .iter()
        .find(|(n, _)| *n == gabarit)
        .map(|(_, c)| *c)
        .unwrap_or_default();
    let titre_yaml = titre.replace('"', "'");
    let mut contenu = String::new();
    for l in modele.lines() {
        let cle = l.split(':').next().unwrap_or("").trim();
        let ligne = match cle {
            "id" if l.starts_with("id:") => format!("id: {id}"),
            "type" if l.starts_with("type:") => format!("type: {type_doc}"),
            "titre" if l.starts_with("titre:") => format!("titre: \"{titre_yaml}\""),
            "statut" if l.starts_with("statut:") => "statut: proposé".to_string(),
            "date" if l.starts_with("date:") => format!("date: {}", crate::relu::aujourdhui()),
            _ if l.starts_with("# ") => format!("# {id} : {titre}"),
            _ => l.to_string(),
        };
        contenu.push_str(&ligne);
        contenu.push('\n');
    }
    let chemin = base.join(&fichier);
    if chemin.exists() {
        bail!("{} existe déjà", chemin.display());
    }
    std::fs::create_dir_all(chemin.parent().unwrap())?;
    std::fs::write(&chemin, contenu)?;
    Ok(format!(
        "{}/{fichier}",
        config.projet.dossier.trim_end_matches('/')
    ))
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

    #[test]
    fn identifiants_suivants() {
        let d = tempfile::tempdir().unwrap();
        let config = crate::config::Config::default();
        super::initialiser(d.path()).unwrap();
        assert_eq!(
            super::nouveau(d.path(), &config, "exigence", "Premier").unwrap(),
            "docs/projet/exigences/EF-001.md"
        );
        assert_eq!(
            super::nouveau(d.path(), &config, "exigence", "Second").unwrap(),
            "docs/projet/exigences/EF-002.md"
        );
        assert_eq!(
            super::nouveau(d.path(), &config, "adr", "Choix : base").unwrap(),
            "docs/projet/adr/0001-choix-base.md"
        );
        let ef2 =
            std::fs::read_to_string(d.path().join("docs/projet/exigences/EF-002.md")).unwrap();
        assert!(
            ef2.contains("id: EF-002")
                && ef2.contains("titre: \"Second\"")
                && ef2.contains("# EF-002 : Second"),
            "{ef2}"
        );
    }

    #[test]
    fn ci_et_codeowners() {
        let d = tempfile::tempdir().unwrap();
        let mut config = crate::config::Config::default();
        config.equipe.validateurs = vec!["@tibo".into()];
        let faits = super::ci(d.path(), &config).unwrap();
        assert_eq!(faits.len(), 2);
        let co = std::fs::read_to_string(d.path().join(".github/CODEOWNERS")).unwrap();
        assert!(co.contains("/docs/projet/majeures/ @tibo"));
        assert!(
            super::ci(d.path(), &config).unwrap().is_empty(),
            "rien n'est réécrit"
        );
    }
}

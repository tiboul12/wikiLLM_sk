//! Point d'entrée de la CLI `wikik` : analyse les arguments et délègue à `wikik-core`.
//!
//! @adr 5

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use wikik_core::Index;

mod atelier;
mod export;
mod mcp;
mod serve;

/// Doc dans le code, index requêtable par l'agent, wiki local.
#[derive(Parser)]
#[command(name = "wikik", version, about)]
struct Cli {
    /// Racine du projet (par défaut : dossier courant).
    #[arg(long, global = true, default_value = ".")]
    racine: PathBuf,
    #[command(subcommand)]
    commande: Commande,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Texte,
    Json,
    /// Annotations GitHub Actions (`check` seulement).
    Github,
}

#[derive(Subcommand)]
enum Commande {
    /// Crée `wikik.toml` et l'arborescence `docs/projet/` (n'écrase rien).
    Init {
        /// Ajoute le workflow GitHub (check sur PR, wiki publié sur Pages) et `CODEOWNERS`.
        #[arg(long)]
        ci: bool,
        /// Installe un hook git `pre-commit` qui lance `wikik check`.
        #[arg(long)]
        hook: bool,
    },
    /// Crée le fichier du prochain identifiant libre (`exigence`, `exigence-nf`, `besoin`, `adr`, `spec`).
    Nouveau { genre: String, titre: String },
    /// Couverture de la doc : symboles documentés, périmés, exigences couvertes.
    Stats {
        #[arg(long, value_enum, default_value = "texte")]
        format: Format,
    },
    /// Écrit le wiki en site statique dans `dossier` (publiable tel quel, ex. GitHub Pages).
    Export { dossier: std::path::PathBuf },
    /// Construit ou met à jour l'index dans `.wikik/`.
    Index {
        /// Écrit le dump JSON déterministe de l'index (sur la sortie standard si aucun fichier).
        #[arg(long, value_name = "FICHIER", num_args = 0..=1, default_missing_value = "-")]
        dump_json: Option<String>,
    },
    /// Signale la doc périmée, manquante et les références cassées.
    Check {
        #[arg(long, value_enum, default_value = "texte")]
        format: Format,
        /// Traite les avertissements comme des erreurs pour le code de retour.
        #[arg(long)]
        strict: bool,
    },
    /// Interroge l'index (pour l'agent de code).
    Query {
        #[command(subcommand)]
        requete: Requete,
        #[arg(long, value_enum, default_value = "texte", global = true)]
        format: Format,
    },
    /// Serveur MCP sur stdio : les requêtes de l'agent comme outils (Claude Code : `claude mcp add wikik -- wikik mcp`).
    Mcp,
    /// Atelier de définition du besoin (Fast Design Thinking), avec le wiki en lecture.
    Atelier {
        /// Adresse d'écoute (défaut : 127.0.0.1, ou le réseau si `[serveur] reseau = true`).
        #[arg(long)]
        adresse: Option<String>,
        /// Port (défaut : 8485, ou le suivant libre si un autre projet l'occupe).
        #[arg(long)]
        port: Option<u16>,
        /// Écoute sur le réseau local (comme `[serveur] reseau = true`) : l'atelier, qui écrit, devient
        /// visible des autres postes.
        #[arg(long)]
        reseau: bool,
        /// Autorise une `--adresse` hors boucle locale.
        #[arg(long)]
        autoriser_reseau: bool,
        /// Fournisseur factice : renvoie les exemples des fiches, sans LLM.
        #[arg(long)]
        factice: bool,
    },
    /// Marque le bloc d'un symbole comme relu (`@relu AAAA-MM-JJ <jeton du code>`).
    Relu { symbole: String },
    /// Sert le wiki en local, en lecture seule.
    Serve {
        /// Adresse d'écoute (défaut : 127.0.0.1, ou le réseau si `[serveur] reseau = true`).
        #[arg(long)]
        adresse: Option<String>,
        /// Port (défaut : 8484, ou le suivant libre si un autre projet l'occupe).
        #[arg(long)]
        port: Option<u16>,
        /// Écoute sur le réseau local (comme `[serveur] reseau = true`).
        #[arg(long)]
        reseau: bool,
        /// Autorise une `--adresse` hors boucle locale.
        #[arg(long)]
        autoriser_reseau: bool,
    },
}

#[derive(Subcommand)]
enum Requete {
    /// Modules et fichiers avec leur rôle en une ligne.
    Carte {
        /// Détail de chaque fichier même au-delà de 150 fichiers.
        #[arg(long)]
        complet: bool,
    },
    /// En-tête, symboles et imports d'un fichier.
    Fichier { chemin: String },
    /// Doc, signature, appelants, appelés, liens d'un symbole.
    Symbole {
        id: String,
        /// Ajoute le code source du symbole.
        #[arg(long)]
        code: bool,
    },
    /// Appelants transitifs, ADR et exigences concernés.
    Impact {
        id: String,
        #[arg(long, default_value_t = 3)]
        profondeur: usize,
    },
    /// Symboles à la doc périmée ou manquante.
    Perimes,
    /// Recherche plein texte dans les blocs et les documents.
    Cherche { texte: Vec<String> },
    /// Un ADR et les symboles qui le citent.
    Adr { id: String },
    /// Une exigence, ses besoins et les symboles qui la citent.
    Exigence { id: String },
    /// Résout une référence `@wiki:` copiée depuis le wiki.
    Ref { reference: String },
    /// Artefacts de l'atelier par type.
    Atelier,
    /// Un document de l'atelier ou du projet, frontmatter compris.
    Document { id: String },
}

fn main() {
    let cli = Cli::parse();
    let code = match executer(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("wikik : {e:#}");
            2
        }
    };
    std::process::exit(code);
}

/// Exécute la commande et renvoie le code de retour (0 succès, 1 diagnostics bloquants, 2 échec).
/// @relu 2026-10-18 ee72e45d
fn executer(cli: Cli) -> anyhow::Result<i32> {
    match cli.commande {
        Commande::Index { dump_json } => {
            let mut index = Index::ouvrir(&cli.racine)?;
            let r = index.mettre_a_jour()?;
            eprintln!(
                "wikik index : {} fichiers ({} analysés, {} supprimés), {} symboles, {} documents en {} ms",
                r.fichiers, r.analyses, r.supprimes, r.symboles, r.documents, r.duree_ms
            );
            if let Some(dest) = dump_json {
                let texte = serde_json::to_string_pretty(&index.dump_json()?)? + "\n";
                if dest == "-" {
                    print!("{texte}");
                } else {
                    std::fs::write(&dest, texte)?;
                }
            }
            Ok(0)
        }
        Commande::Check { format, strict } => {
            let mut index = Index::ouvrir(&cli.racine)?;
            index.mettre_a_jour()?;
            let rapport = wikik_core::check::verifier(&index)?;
            match format {
                Format::Texte => print!("{}", wikik_core::check::formater_texte(&rapport)),
                Format::Json => println!("{}", serde_json::to_string_pretty(&rapport)?),
                Format::Github => print!("{}", wikik_core::check::formater_github(&rapport)),
            }
            Ok(rapport.code_retour(strict))
        }
        Commande::Init { ci, hook } => {
            let mut faits = wikik_core::init::initialiser(&cli.racine)?;
            let config = wikik_core::config::Config::charger(&cli.racine)?;
            if ci {
                faits.extend(wikik_core::init::ci(&cli.racine, &config)?);
            }
            if hook {
                faits.push(wikik_core::init::hook(&cli.racine)?);
            }
            if faits.is_empty() {
                eprintln!("wikik init : rien à faire, le projet est déjà initialisé");
            }
            for f in faits {
                eprintln!("créé : {f}");
            }
            Ok(0)
        }
        Commande::Nouveau { genre, titre } => {
            let config = wikik_core::config::Config::charger(&cli.racine)?;
            let f = wikik_core::init::nouveau(&cli.racine, &config, &genre, &titre)?;
            println!("{f}");
            Ok(0)
        }
        Commande::Stats { format } => {
            let mut index = Index::ouvrir(&cli.racine)?;
            index.mettre_a_jour()?;
            let s = wikik_core::stats::calculer(&index)?;
            match format {
                Format::Json => println!("{}", serde_json::to_string_pretty(&s)?),
                _ => print!("{}", wikik_core::stats::formater(&s)),
            }
            Ok(0)
        }
        Commande::Export { dossier } => {
            let n = export::exporter(&cli.racine, &dossier)?;
            eprintln!(
                "wikik export : {n} fichiers écrits dans {}",
                dossier.display()
            );
            Ok(0)
        }
        Commande::Query { requete, format } => {
            let mut index = Index::ouvrir(&cli.racine)?;
            index.mettre_a_jour()?;
            use wikik_core::query as q;
            let sortie = match requete {
                Requete::Carte { complet } => q::carte(&index, complet)?,
                Requete::Fichier { chemin } => q::fichier(&index, &chemin)?,
                Requete::Symbole { id, code } => q::symbole(&index, &id, code)?,
                Requete::Impact { id, profondeur } => q::impact(&index, &id, profondeur)?,
                Requete::Perimes => q::perimes(&index)?,
                Requete::Cherche { texte } => q::cherche(&index, &texte.join(" "))?,
                Requete::Adr { id } => q::adr(&index, &id)?,
                Requete::Exigence { id } => q::exigence(&index, &id)?,
                Requete::Ref { reference } => q::reference(&index, &reference)?,
                Requete::Atelier => q::atelier(&index)?,
                Requete::Document { id } => q::document_complet(&index, &id)?,
            };
            match format {
                Format::Json => println!("{}", serde_json::to_string_pretty(&sortie.json)?),
                _ => print!("{}", sortie.texte),
            }
            Ok(i32::from(sortie.ambigu))
        }
        Commande::Mcp => {
            mcp::servir(&cli.racine)?;
            Ok(0)
        }
        Commande::Serve {
            adresse,
            port,
            reseau,
            autoriser_reseau,
        } => {
            let (adresse, autoriser) = ecoute(&cli.racine, adresse, reseau, autoriser_reseau)?;
            serve::servir(
                &cli.racine,
                &adresse,
                serve::Port::choisir(port, 8484),
                autoriser,
            )?;
            Ok(0)
        }
        Commande::Atelier {
            adresse,
            port,
            reseau,
            autoriser_reseau,
            factice,
        } => {
            let (adresse, autoriser) = ecoute(&cli.racine, adresse, reseau, autoriser_reseau)?;
            atelier::servir(
                &cli.racine,
                &adresse,
                serve::Port::choisir(port, 8485),
                autoriser,
                factice,
            )?;
            Ok(0)
        }
        Commande::Relu { symbole } => {
            let mut index = Index::ouvrir(&cli.racine)?;
            index.mettre_a_jour()?;
            let date = wikik_core::relu::aujourdhui();
            let fichier = wikik_core::relu::marquer(&index, &symbole, &date)?;
            index.mettre_a_jour()?;
            eprintln!("wikik relu : {symbole} marqué relu le {date} ({fichier})");
            Ok(0)
        }
    }
}

/// Adresse d'écoute d'un serveur : `--adresse` si donnée ; sinon le réseau local avec `--reseau` ou
/// `[serveur] reseau = true` dans `wikik.toml`, la boucle locale par défaut. Renvoie (adresse, réseau
/// autorisé).
fn ecoute(
    racine: &std::path::Path,
    adresse: Option<String>,
    reseau: bool,
    autoriser: bool,
) -> anyhow::Result<(String, bool)> {
    let config_reseau = wikik_core::config::Config::charger(racine)?.serveur.reseau;
    Ok(match adresse {
        Some(a) => (a, autoriser || reseau),
        None if reseau || config_reseau => ("0.0.0.0".into(), true),
        None => ("127.0.0.1".into(), autoriser),
    })
}

#[cfg(test)]
mod tests {
    use super::ecoute;

    /// `[serveur] reseau = true` suffit pour écouter sur le réseau ; une `--adresse` explicite garde la main.
    #[test]
    fn adresse_d_ecoute() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            ecoute(d.path(), None, false, false).unwrap(),
            ("127.0.0.1".into(), false)
        );
        assert_eq!(
            ecoute(d.path(), None, true, false).unwrap(),
            ("0.0.0.0".into(), true)
        );
        std::fs::write(d.path().join("wikik.toml"), "[serveur]\nreseau = true\n").unwrap();
        assert_eq!(
            ecoute(d.path(), None, false, false).unwrap(),
            ("0.0.0.0".into(), true)
        );
        assert_eq!(
            ecoute(d.path(), Some("127.0.0.1".into()), false, false).unwrap(),
            ("127.0.0.1".into(), false)
        );
    }
}

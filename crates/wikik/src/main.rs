//! Point d'entrée de la CLI `wikik` : analyse les arguments et délègue à `wikik-core`.
//!
//! @adr 5

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use wikik_core::Index;

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
}

#[derive(Subcommand)]
enum Commande {
    /// Crée `wikik.toml` et l'arborescence `docs/projet/` (n'écrase rien).
    Init,
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
    /// Marque le bloc d'un symbole comme relu (`@relu AAAA-MM-JJ`).
    Relu { symbole: String },
    /// Sert le wiki en local, en lecture seule.
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        adresse: String,
        #[arg(long, default_value_t = 8484)]
        port: u16,
    },
}

#[derive(Subcommand)]
enum Requete {
    /// Modules et fichiers avec leur rôle en une ligne.
    Carte,
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
        autre => {
            let nom = match autre {
                Commande::Init => "init",
                Commande::Check { .. } => "check",
                Commande::Query { .. } => "query",
                Commande::Relu { .. } => "relu",
                Commande::Serve { .. } => "serve",
                Commande::Index { .. } => unreachable!(),
            };
            anyhow::bail!("{nom} : pas encore implémenté")
        }
    }
}

//! Point d'entrée de la CLI `wikik` : analyse les arguments et délègue à `wikik-core`.
//!
//! @adr 5

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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
    let _ = (&cli.racine, wikik_core::SCHEMA);
    let nom = match cli.commande {
        Commande::Init => "init",
        Commande::Index { .. } => "index",
        Commande::Check { .. } => "check",
        Commande::Query { .. } => "query",
        Commande::Relu { .. } => "relu",
        Commande::Serve { .. } => "serve",
    };
    eprintln!("wikik {nom} : pas encore implémenté");
    std::process::exit(2);
}

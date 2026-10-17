//! Accès minimal à git en ligne de commande : blame d'un fichier et contenu à un commit.
//!
//! Sert à la détection de doc périmée (ADR-0006). On appelle le binaire `git` plutôt qu'une
//! bibliothèque : zéro dépendance de plus, et le comportement est celui que l'utilisateur connaît.
//!
//! @adr 6

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Commit fictif que `git blame` attribue aux lignes non commitées.
pub const NON_COMMITE: &str = "0000000000000000000000000000000000000000";

/// Dépôt git contenant la racine du projet.
pub struct Depot {
    pub racine_git: PathBuf,
    /// Préfixe de la racine du projet dans le dépôt (`""` si c'est la racine du dépôt), avec `/` final.
    pub prefixe: String,
}

/// Origine d'une ligne du fichier courant.
#[derive(Debug, Clone)]
pub struct LigneBlame {
    pub commit: String,
    pub date: i64,
    /// Chemin du fichier (relatif au dépôt) dans ce commit : suit les renommages.
    pub chemin: String,
}

impl Depot {
    /// `None` si `racine` n'est pas dans un dépôt git ou si git est absent.
    pub fn trouver(racine: &Path) -> Option<Depot> {
        let sortie = Command::new("git")
            .arg("-C")
            .arg(racine)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()?;
        if !sortie.status.success() {
            return None;
        }
        let racine_git = PathBuf::from(String::from_utf8_lossy(&sortie.stdout).trim())
            .canonicalize()
            .ok()?;
        let relatif = racine
            .canonicalize()
            .ok()?
            .strip_prefix(&racine_git)
            .ok()?
            .to_path_buf();
        let mut prefixe = relatif
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if !prefixe.is_empty() {
            prefixe.push('/');
        }
        Some(Depot {
            racine_git,
            prefixe,
        })
    }

    /// Blame de l'arbre de travail : une entrée par ligne (index 0 = ligne 1). `None` si le
    /// fichier n'est pas suivi par git.
    pub fn blame(&self, chemin_projet: &str) -> Option<Vec<LigneBlame>> {
        let chemin = format!("{}{chemin_projet}", self.prefixe);
        let sortie = Command::new("git")
            .arg("-C")
            .arg(&self.racine_git)
            .args(["blame", "--line-porcelain", "--", &chemin])
            .output()
            .ok()?;
        if !sortie.status.success() {
            return None;
        }
        Some(analyser_porcelain(&String::from_utf8_lossy(&sortie.stdout)))
    }

    /// Contenu d'un fichier (chemin relatif au dépôt) à un commit.
    pub fn contenu(&self, commit: &str, chemin_depot: &str) -> Option<Vec<u8>> {
        let sortie = Command::new("git")
            .arg("-C")
            .arg(&self.racine_git)
            .args(["show", &format!("{commit}:{chemin_depot}")])
            .output()
            .ok()?;
        sortie.status.success().then_some(sortie.stdout)
    }
}

fn analyser_porcelain(texte: &str) -> Vec<LigneBlame> {
    let mut lignes = vec![];
    let mut commit = String::new();
    let mut champs: HashMap<&str, &str> = HashMap::new();
    for l in texte.lines() {
        if let Some(contenu) = l.strip_prefix('\t') {
            let _ = contenu;
            lignes.push(LigneBlame {
                commit: commit.clone(),
                date: champs
                    .get("committer-time")
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(0),
                chemin: champs.get("filename").copied().unwrap_or("").to_string(),
            });
            champs.clear();
            continue;
        }
        let (cle, valeur) = l.split_once(' ').unwrap_or((l, ""));
        if cle.len() == 40 && cle.chars().all(|c| c.is_ascii_hexdigit()) {
            commit = cle.to_string();
        } else {
            champs.insert(cle, valeur);
        }
    }
    lignes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn porcelain() {
        let t = "1111111111111111111111111111111111111111 1 1 1\ncommitter-time 10\nfilename a.rs\n\tfn a() {}\n\
                 0000000000000000000000000000000000000000 2 2 1\ncommitter-time 20\nfilename a.rs\n\t// nouveau\n";
        let l = analyser_porcelain(t);
        assert_eq!(l.len(), 2);
        assert_eq!((l[0].date, l[1].commit.as_str()), (10, NON_COMMITE));
    }
}

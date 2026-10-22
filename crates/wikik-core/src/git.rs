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

/// Où en est un commit (EF-165, REC-080, REC-082) : existe-t-il, est-il poussé, dans quelle version.
#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct EtatCommit {
    pub commit: String,
    /// Le commit est dans le dépôt.
    pub existe: bool,
    /// Hash complet, sujet et date (`AAAA-MM-JJ`) du commit.
    pub hash: String,
    pub sujet: String,
    pub date: String,
    /// Branches locales qui le contiennent.
    pub branches: Vec<String>,
    /// Une branche distante le contient : il est poussé.
    pub pousse: bool,
    /// Première version (tag) qui le contient ; `None` : pas encore publié.
    pub version: Option<String>,
}

impl Depot {
    fn git(&self, args: &[&str]) -> Option<String> {
        let s = Command::new("git")
            .arg("-C")
            .arg(&self.racine_git)
            .args(args)
            .output()
            .ok()?;
        s.status
            .success()
            .then(|| String::from_utf8_lossy(&s.stdout).trim().to_string())
    }

    /// État d'un commit cité par une remarque traitée.
    pub fn etat_commit(&self, commit: &str) -> EtatCommit {
        let mut e = EtatCommit {
            commit: commit.to_string(),
            existe: false,
            hash: String::new(),
            sujet: String::new(),
            date: String::new(),
            branches: vec![],
            pousse: false,
            version: None,
        };
        let c = commit.trim();
        if c.is_empty() || !c.chars().all(|x| x.is_ascii_hexdigit()) {
            return e;
        }
        let Some(info) = self.git(&[
            "log",
            "-1",
            "--format=%H%x09%cs%x09%s",
            &format!("{c}^{{commit}}"),
        ]) else {
            return e;
        };
        let mut champs = info.splitn(3, '\t');
        e.existe = true;
        e.hash = champs.next().unwrap_or("").to_string();
        e.date = champs.next().unwrap_or("").to_string();
        e.sujet = champs.next().unwrap_or("").to_string();
        let liste = |args: &[&str]| -> Vec<String> {
            self.git(args)
                .unwrap_or_default()
                .lines()
                .map(|l| l.trim_start_matches(['*', ' ']).trim().to_string())
                .filter(|l| !l.is_empty() && !l.contains("HEAD ->"))
                .collect()
        };
        e.branches = liste(&["branch", "--contains", &e.hash]);
        e.pousse = !liste(&["branch", "-r", "--contains", &e.hash]).is_empty();
        e.version = liste(&["tag", "--contains", &e.hash, "--sort=creatordate"])
            .into_iter()
            .next();
        e
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

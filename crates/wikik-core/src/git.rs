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
    /// Lance git dans le dépôt ; `None` si la commande échoue (git absent, commit inconnu), jamais d'erreur bloquante.
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

/// Un commit de l'historique (EF-166) : de quoi dessiner le graphe des branches.
#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CommitVue {
    pub hash: String,
    pub parents: Vec<String>,
    pub auteur: String,
    /// Date et heure du commit (`AAAA-MM-JJ HH:MM`).
    pub date: String,
    pub sujet: String,
    /// Branches, branches distantes et tags qui pointent dessus ; `HEAD` en tête s'il y est.
    pub refs: Vec<String>,
}

/// Historique du dépôt et sa situation (EF-166).
#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct Historique {
    pub commits: Vec<CommitVue>,
    /// Branche courante (vide : HEAD détaché).
    pub branche: String,
    pub branches: Vec<String>,
    pub distantes: Vec<String>,
    pub tags: Vec<String>,
    /// Fichiers modifiés et non commités.
    pub modifies: u32,
    /// Commits de la branche pas encore poussés, et commits distants pas encore récupérés.
    pub en_avance: u32,
    pub en_retard: u32,
    /// L'historique a été coupé à la limite demandée.
    pub tronque: bool,
}

/// Un fichier touché par un commit.
#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FichierCommit {
    /// `A` ajouté, `M` modifié, `D` supprimé, `R` renommé.
    pub statut: String,
    pub chemin: String,
    pub ajouts: u32,
    pub suppressions: u32,
}

/// Détail d'un commit : message complet et fichiers touchés.
#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DetailCommit {
    pub hash: String,
    pub auteur: String,
    pub date: String,
    pub message: String,
    pub fichiers: Vec<FichierCommit>,
}

/// Hash abrégé ou complet plausible, vérifié avant de le passer à git (une valeur venue de l'écran n'y entre pas telle quelle).
fn hash_valide(h: &str) -> bool {
    (4..=40).contains(&h.len()) && h.chars().all(|c| c.is_ascii_hexdigit())
}

impl Depot {
    /// Historique de toutes les branches, du plus récent au plus ancien (ordre des dates, parents après
    /// leurs enfants), `limite` commits au plus.
    pub fn historique(&self, limite: usize) -> Historique {
        let n = limite.max(1);
        let texte = self
            .git(&[
                "log",
                "--all",
                "--date-order",
                "--date=format:%Y-%m-%d %H:%M",
                "--format=%H%x1f%P%x1f%an%x1f%cd%x1f%D%x1f%s",
                &format!("-n{}", n + 1),
            ])
            .unwrap_or_default();
        let mut commits: Vec<CommitVue> = texte
            .lines()
            .filter_map(|l| {
                let c: Vec<&str> = l.split('\u{1f}').collect();
                if c.len() < 6 {
                    return None;
                }
                let refs = c[4]
                    .split(", ")
                    .filter(|r| !r.is_empty())
                    .map(|r| r.trim_start_matches("tag: ").replace("HEAD -> ", "HEAD → "))
                    .collect();
                Some(CommitVue {
                    hash: c[0].to_string(),
                    parents: c[1].split_whitespace().map(str::to_string).collect(),
                    auteur: c[2].to_string(),
                    date: c[3].to_string(),
                    refs,
                    sujet: c[5..].join("\u{1f}"),
                })
            })
            .collect();
        let tronque = commits.len() > n;
        commits.truncate(n);
        let liste = |args: &[&str]| -> Vec<String> {
            self.git(args)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.ends_with("/HEAD"))
                .map(str::to_string)
                .collect()
        };
        let compte = |args: &[&str]| -> u32 {
            self.git(args)
                .and_then(|t| t.trim().parse().ok())
                .unwrap_or(0)
        };
        let branche = self.git(&["branch", "--show-current"]).unwrap_or_default();
        let amont = self.git(&["rev-parse", "--abbrev-ref", "@{upstream}"]);
        let (en_avance, en_retard) = match &amont {
            Some(a) => (
                compte(&["rev-list", "--count", &format!("{a}..HEAD")]),
                compte(&["rev-list", "--count", &format!("HEAD..{a}")]),
            ),
            None => (0, 0),
        };
        Historique {
            commits,
            branche,
            branches: liste(&["branch", "--format=%(refname:short)"]),
            distantes: liste(&["branch", "-r", "--format=%(refname:short)"]),
            tags: liste(&["tag", "--sort=-creatordate"]),
            modifies: self
                .git(&["status", "--porcelain"])
                .map_or(0, |t| t.lines().count() as u32),
            en_avance,
            en_retard,
            tronque,
        }
    }

    /// Message complet et fichiers touchés d'un commit (par rapport à son premier parent).
    pub fn detail(&self, hash: &str) -> Option<DetailCommit> {
        if !hash_valide(hash) {
            return None;
        }
        let entete = self.git(&[
            "log",
            "-1",
            "--date=format:%Y-%m-%d %H:%M",
            "--format=%H%x1f%an%x1f%cd%x1f%B",
            hash,
        ])?;
        let c: Vec<&str> = entete.splitn(4, '\u{1f}').collect();
        let statuts = self
            .git(&["show", "--format=", "--name-status", "-M", hash])
            .unwrap_or_default();
        let nombres = self
            .git(&["show", "--format=", "--numstat", "-M", hash])
            .unwrap_or_default();
        let compteurs: Vec<(u32, u32)> = nombres
            .lines()
            .map(|l| {
                let mut x = l.split('\t');
                (
                    x.next().and_then(|v| v.parse().ok()).unwrap_or(0),
                    x.next().and_then(|v| v.parse().ok()).unwrap_or(0),
                )
            })
            .collect();
        let fichiers = statuts
            .lines()
            .enumerate()
            .filter_map(|(i, l)| {
                let parts: Vec<&str> = l.split('\t').collect();
                let statut = parts.first()?.chars().next()?.to_string();
                let chemin = parts.last()?.to_string();
                let (ajouts, suppressions) = compteurs.get(i).copied().unwrap_or((0, 0));
                Some(FichierCommit {
                    statut,
                    chemin,
                    ajouts,
                    suppressions,
                })
            })
            .collect();
        Some(DetailCommit {
            hash: c.first()?.to_string(),
            auteur: c.get(1)?.to_string(),
            date: c.get(2)?.to_string(),
            message: c.get(3).unwrap_or(&"").trim().to_string(),
            fichiers,
        })
    }

    /// Diff d'un fichier dans un commit (format unifié), 400 Ko au plus.
    pub fn diff(&self, hash: &str, chemin: &str) -> Option<String> {
        if !hash_valide(hash) || chemin.contains("..") {
            return None;
        }
        let mut d = self.git(&["show", "--format=", "-M", hash, "--", chemin])?;
        if d.len() > 400_000 {
            let mut fin = 400_000;
            while !d.is_char_boundary(fin) {
                fin -= 1;
            }
            d.truncate(fin);
            d.push_str("\n… (diff coupé)");
        }
        Some(d)
    }
}

/// Analyse la sortie `git blame --porcelain` : pour chaque ligne, son commit et la date de ce commit.
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

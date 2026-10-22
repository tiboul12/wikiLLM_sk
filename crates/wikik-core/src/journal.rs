//! Journal d'annulation (ADR-0016, EF-159) : chaque écriture que le serveur fait pour l'interface
//! (documents, atelier, réception, paramètres, skills) garde le contenu d'avant et d'après de chaque
//! fichier touché ; « annuler » restaure l'état d'avant la dernière action, « rétablir » la rejoue. Le
//! journal vit en mémoire (100 actions), une action par requête d'écriture.
//!
//! @req EF-159
//! @adr 16

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Actions gardées au plus.
const TAILLE: usize = 100;

/// Une écriture de fichier : contenus avant et après, de quoi la défaire et la refaire.
#[derive(Debug, Clone)]
struct Entree {
    chemin: PathBuf,
    /// Contenu avant l'écriture (`None` : le fichier n'existait pas).
    avant: Option<Vec<u8>>,
    /// Contenu après (`None` : supprimé).
    apres: Option<Vec<u8>>,
}

/// Une action de l'utilisateur (une requête d'écriture) et toutes les écritures qu'elle a faites, défaites ensemble.
#[derive(Debug, Clone)]
struct Action {
    numero: u64,
    libelle: String,
    entrees: Vec<Entree>,
}

/// Piles des actions faites et défaites, et action en cours d'enregistrement (numéro et libellé).
#[derive(Default)]
struct Journal {
    faites: Vec<Action>,
    defaites: Vec<Action>,
    courante: u64,
    libelle: String,
}

/// Journal du processus : un seul serveur par projet (ADR-0012), donc une seule pile ; perdu à l'arrêt du serveur.
static JOURNAL: Mutex<Journal> = Mutex::new(Journal {
    faites: Vec::new(),
    defaites: Vec::new(),
    courante: 0,
    libelle: String::new(),
});

/// Accès au journal ; un verrou empoisonné par une panique est repris, le journal reste utilisable.
fn journal() -> std::sync::MutexGuard<'static, Journal> {
    JOURNAL.lock().unwrap_or_else(|x| x.into_inner())
}

impl Journal {
    /// Ouvre une nouvelle action : les écritures suivantes s'y rattachent jusqu'à la prochaine ouverture.
    fn ouvrir(&mut self, libelle: &str) {
        self.courante += 1;
        self.libelle = libelle.to_string();
    }

    /// Note une écriture dans l'action courante. Une écriture sans effet est ignorée, toute écriture vide les actions
    /// défaites (on ne rétablit pas par-dessus un nouveau travail), un fichier écrit deux fois garde son premier « avant ».
    /// Les actions les plus anciennes tombent au-delà de `TAILLE`.
    fn noter(&mut self, chemin: &Path, avant: Option<Vec<u8>>, apres: Option<Vec<u8>>) {
        if avant == apres {
            return;
        }
        let numero = self.courante;
        // Une nouvelle écriture rend caduc ce qui avait été annulé.
        self.defaites.clear();
        let entree = Entree {
            chemin: chemin.to_path_buf(),
            avant,
            apres,
        };
        match self.faites.last_mut() {
            Some(a) if a.numero == numero => {
                // Deux écritures du même fichier dans une action : on garde le tout premier « avant ».
                if let Some(e) = a.entrees.iter_mut().find(|e| e.chemin == entree.chemin) {
                    e.apres = entree.apres;
                } else {
                    a.entrees.push(entree);
                }
            }
            _ => {
                self.faites.push(Action {
                    numero,
                    libelle: self.libelle.clone(),
                    entrees: vec![entree],
                });
                if self.faites.len() > TAILLE {
                    self.faites.remove(0);
                }
            }
        }
    }

    /// Écrit un fichier (dossiers créés au besoin) et note l'écriture.
    fn ecrire(&mut self, chemin: &Path, contenu: &[u8]) -> std::io::Result<()> {
        let avant = std::fs::read(chemin).ok();
        if let Some(d) = chemin.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(chemin, contenu)?;
        self.noter(chemin, avant, Some(contenu.to_vec()));
        Ok(())
    }

    /// Supprime un fichier et note la suppression ; un fichier absent n'est pas une erreur.
    fn supprimer(&mut self, chemin: &Path) -> std::io::Result<()> {
        let Ok(avant) = std::fs::read(chemin) else {
            return Ok(());
        };
        std::fs::remove_file(chemin)?;
        self.noter(chemin, Some(avant), None);
        Ok(())
    }

    /// Défait la dernière action ; `None` s'il n'y en a pas.
    fn annuler(&mut self, racine: &Path) -> Result<Option<Bilan>> {
        let Some(a) = self.faites.last().cloned() else {
            return Ok(None);
        };
        appliquer(&a.entrees, true)?;
        self.faites.pop();
        let b = bilan(&a, racine);
        self.defaites.push(a);
        Ok(Some(b))
    }

    /// Refait la dernière action défaite ; `None` s'il n'y en a pas.
    fn retablir(&mut self, racine: &Path) -> Result<Option<Bilan>> {
        let Some(a) = self.defaites.last().cloned() else {
            return Ok(None);
        };
        appliquer(&a.entrees, false)?;
        self.defaites.pop();
        let b = bilan(&a, racine);
        self.faites.push(a);
        Ok(Some(b))
    }
}

/// Ouvre une nouvelle action (une requête d'écriture) : les écritures qui suivent s'y rangent.
pub fn nouvelle_action(libelle: &str) {
    journal().ouvrir(libelle);
}

/// Écrit `contenu` dans `chemin` (dossiers créés au besoin) en notant l'ancien contenu.
pub fn ecrire(chemin: impl AsRef<Path>, contenu: impl AsRef<[u8]>) -> std::io::Result<()> {
    journal().ecrire(chemin.as_ref(), contenu.as_ref())
}

/// Supprime `chemin` s'il existe, en notant son contenu.
pub fn supprimer(chemin: impl AsRef<Path>) -> std::io::Result<()> {
    journal().supprimer(chemin.as_ref())
}

/// Ce qu'une annulation ou un rétablissement a fait : le libellé de l'action et les fichiers touchés.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Bilan {
    pub libelle: String,
    pub fichiers: Vec<String>,
}

/// Remet les fichiers d'une action dans leur état d'avant (ou d'après). Tout est vérifié avant d'écrire :
/// si un seul fichier a changé depuis, rien n'est touché, pour ne pas écraser le travail de l'agent ou d'un éditeur.
fn appliquer(entrees: &[Entree], vers_avant: bool) -> Result<()> {
    // Refus si un fichier a changé depuis (l'agent, l'éditeur) : on n'écrase pas son travail.
    for e in entrees {
        let attendu = if vers_avant { &e.apres } else { &e.avant };
        if std::fs::read(&e.chemin).ok() != *attendu {
            bail!(
                "{} a été modifié depuis : annulation refusée pour ne pas écraser ce travail",
                e.chemin.display()
            );
        }
    }
    let ordre: Box<dyn Iterator<Item = &Entree>> = if vers_avant {
        Box::new(entrees.iter().rev())
    } else {
        Box::new(entrees.iter())
    };
    for e in ordre {
        match if vers_avant { &e.avant } else { &e.apres } {
            Some(c) => {
                if let Some(d) = e.chemin.parent() {
                    std::fs::create_dir_all(d)?;
                }
                std::fs::write(&e.chemin, c)?;
            }
            None => {
                let _ = std::fs::remove_file(&e.chemin);
            }
        }
    }
    Ok(())
}

/// Résumé d'une action défaite ou refaite : libellé et fichiers relatifs à la racine, pour le message à l'écran.
fn bilan(a: &Action, racine: &Path) -> Bilan {
    Bilan {
        libelle: a.libelle.clone(),
        fichiers: a
            .entrees
            .iter()
            .map(|e| {
                e.chemin
                    .strip_prefix(racine)
                    .unwrap_or(&e.chemin)
                    .to_string_lossy()
                    .into_owned()
            })
            .collect(),
    }
}

/// Annule la dernière action ; `None` s'il n'y a rien à annuler.
pub fn annuler(racine: &Path) -> Result<Option<Bilan>> {
    journal().annuler(racine)
}

/// Rejoue la dernière action annulée ; `None` s'il n'y a rien à rétablir.
pub fn retablir(racine: &Path) -> Result<Option<Bilan>> {
    journal().retablir(racine)
}

/// Nombre d'actions annulables et rétablissables.
pub fn etat() -> (usize, usize) {
    let j = journal();
    (j.faites.len(), j.defaites.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Écrire, annuler, rétablir ; une annulation sur un fichier modifié entre-temps est refusée. Le test a
    /// son propre journal : celui du processus est partagé par les autres tests.
    #[test]
    fn annuler_et_retablir() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("a/doc.md");
        let mut j = Journal::default();
        j.ouvrir("créer");
        j.ecrire(&f, b"un").unwrap();
        j.ouvrir("modifier");
        j.ecrire(&f, b"deux").unwrap();
        j.ecrire(&f, b"trois").unwrap();
        let b = j.annuler(d.path()).unwrap().unwrap();
        assert_eq!(
            (b.libelle.as_str(), b.fichiers[0].as_str()),
            ("modifier", "a/doc.md")
        );
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "un");
        j.retablir(d.path()).unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "trois");
        j.annuler(d.path()).unwrap();
        j.annuler(d.path()).unwrap();
        assert!(!f.exists());
        j.retablir(d.path()).unwrap();
        std::fs::write(&f, "écrit par l'agent").unwrap();
        assert!(j.annuler(d.path()).is_err());
        j.ouvrir("supprimer");
        j.supprimer(&f).unwrap();
        j.annuler(d.path()).unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "écrit par l'agent");
    }
}

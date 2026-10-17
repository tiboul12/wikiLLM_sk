//! Types produits par l'extraction et stockés dans l'index (SPEC-02).

use serde::Serialize;

/// Balise d'un bloc de doc (`@adr`, `@req`, `@topic`, `@pitfall`, `@relu`, ou inconnue).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Balise {
    pub nom: String,
    pub valeur: String,
    pub ligne: usize,
}

/// Site d'appel tel qu'il apparaît dans le code, avant résolution globale (SPEC-03).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppelBrut {
    /// `direct`, `chemin`, `methode` ou `rend`.
    pub forme: String,
    pub cible: String,
    /// `self` pour un appel de méthode sur `self`.
    pub qualificatif: Option<String>,
    pub ligne: usize,
}

/// Import `use` aplati : `alias` désigne `chemin` dans le fichier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Import {
    pub alias: String,
    pub chemin: String,
}

/// Symbole extrait d'un fichier de code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Symbole {
    /// `<chemin>::<nom qualifié>[@cfg(..)|#n]`, stable entre machines.
    pub id: String,
    pub nom: String,
    pub nom_qualifie: String,
    /// `fn`, `methode`, `composant`, `struct`, `enum`, `union`, `trait`, `type`, `const`, `static`, `macro`, `mod`.
    pub genre: String,
    /// `pub`, `pub-crate` ou `prive`.
    pub visibilite: String,
    pub ligne_debut: usize,
    pub ligne_fin: usize,
    pub doc_ligne_debut: Option<usize>,
    pub doc_ligne_fin: Option<usize>,
    pub signature: String,
    pub resume: Option<String>,
    pub doc: Option<String>,
    pub hash_corps: String,
    pub hash_signature: String,
    pub hash_doc: Option<String>,
    pub est_test: bool,
    /// Type de l'impl englobant (`Point` pour `Point::norme`), utilisé pour résoudre `self.f()`.
    pub type_parent: Option<String>,
    pub balises: Vec<Balise>,
    /// Prédicat `#[cfg(..)]` compacté, pour désambiguïser deux variantes d'un même item.
    #[serde(skip)]
    pub cfg: Option<String>,
    #[serde(skip)]
    pub appels: Vec<AppelBrut>,
}

/// Tout ce qu'un langage extrait d'un fichier.
/// @relu 2026-10-17 b3b0d155
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtraitFichier {
    pub entete: Option<String>,
    /// Balises du bloc `//!` : elles relient le fichier entier (source d'arête = chemin du fichier).
    pub balises_entete: Vec<Balise>,
    pub symboles: Vec<Symbole>,
    pub imports: Vec<Import>,
    pub lignes: usize,
    pub lignes_macros: usize,
    pub erreurs_parse: usize,
}

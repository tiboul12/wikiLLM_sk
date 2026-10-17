//! Bibliothèque de démonstration : calculs géométriques simples.
//!
//! Sert de fixture aux tests de wikik ; chaque construction couvre un cas de SPEC-02/03.

mod geo;
mod util;

use geo::aire;
use util::double;

/// Point d'entrée : aire doublée d'un carré de côté `cote`.
///
/// @req EF-001
/// @topic calcul
pub fn entree(cote: f64) -> f64 {
    let p = geo::Point::new(cote, cote);
    let n = p.norme();
    double(aire(cote)) + aide(n)
}

/// Ajoute une marge de sécurité de 1 %.
fn aide(x: f64) -> f64 {
    x * 1.01
}

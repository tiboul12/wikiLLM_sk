//! Types et fonctions géométriques.

use std::fmt;

/// Point du plan, coordonnées en mètres.
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Crée un point ; aucune validation (les coordonnées peuvent être négatives).
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    /// Distance à l'origine.
    ///
    /// @adr 1
    /// @pitfall NaN si une coordonnée est NaN.
    pub fn norme(&self) -> f64 {
        self.carre().sqrt()
    }

    fn carre(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
}

impl fmt::Display for Point {
    /// Affiche `(x, y)` avec deux décimales.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

/// Aire d'un carré de côté `cote`.
///
/// @req EF-999
/// @inconnue valeur
pub fn aire(cote: f64) -> f64 {
    cote * cote
}

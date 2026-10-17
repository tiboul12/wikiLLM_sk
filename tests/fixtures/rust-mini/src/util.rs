//! Utilitaires divers.

/// Nom de l'exécutable sous Windows.
#[cfg(windows)]
fn exe() -> &'static str {
    "outil.exe"
}

/// Nom de l'exécutable ailleurs.
#[cfg(not(windows))]
fn exe() -> &'static str {
    "outil"
}

pub fn double(x: f64) -> f64 {
    let _ = exe();
    x * 2.0
}

pub mod interne {
    /// Fonction d'un module en ligne.
    pub fn profond() -> u8 {
        super::double(1.0) as u8
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok() {
        assert_eq!(super::double(2.0), 4.0);
    }
}

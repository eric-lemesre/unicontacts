//! Cœur métier d'unicontacts.
//!
//! Ce crate contiendra le modèle de données, le stockage SQLite,
//! le moteur de synchronisation à trois voies, la file de décisions
//! et la détection de doublons.

/// Version de la spécification implementée.
pub const SPEC_VERSION: &str = "0.1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_version_is_defined() {
        assert_eq!(SPEC_VERSION, "0.1");
    }
}

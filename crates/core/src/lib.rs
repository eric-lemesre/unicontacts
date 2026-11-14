//! Cœur métier d'unicontacts.
//!
//! Modèle de données, lecture et écriture vCard, fusion à trois voies, file
//! de décisions, détection des doublons et stockage. Le crate ne dépend
//! d'aucune bibliothèque Android : il se compile et se teste sur un poste de
//! développement, et la couche Android l'appelle par son interface étrangère
//! (ADR-0003, ADR-0006).
// [impl->req~coeur-independant-android~1]

pub mod champ;
pub mod fiche;
pub mod identifiants;
pub mod temps;

pub use champ::{Champ, Origine, Provenance, Valeur};
pub use fiche::{ErreurFiche, Fiche, Lien};
pub use identifiants::{DecisionId, FicheId, IdentifiantInvalide, SourceId};
pub use temps::Horodatage;

/// Version de la spécification réalisée.
pub const SPEC_VERSION: &str = "0.1";

#[cfg(test)]
mod tests {
    use super::*;

    // Ce test, comme tous ceux du crate, tourne en CI sur un poste Linux
    // sans SDK Android ni émulateur.
    // [utest->req~coeur-independant-android~1]
    #[test]
    fn le_coeur_se_teste_hors_android() {
        assert_eq!(SPEC_VERSION, "0.1");
    }
}

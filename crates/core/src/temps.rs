//! Horodatage des valeurs et des décisions.
//!
//! Le cœur ne lit jamais l'horloge de lui-même : chaque opération datée
//! reçoit son horodatage de l'appelant. Les tests restent ainsi
//! déterministes, et l'heure du téléphone se lit en un seul endroit.

use std::time::{SystemTime, UNIX_EPOCH};

/// Instant UTC, en millisecondes depuis l'époque Unix : la date que porte
/// la provenance d'une valeur.
// [impl->req~provenance-par-champ~1]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Horodatage(i64);

impl Horodatage {
    /// Horodatage à partir d'un nombre de millisecondes depuis l'époque Unix.
    #[must_use]
    pub const fn depuis_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Nombre de millisecondes depuis l'époque Unix.
    #[must_use]
    pub const fn millis(self) -> i64 {
        self.0
    }

    /// Instant présent selon l'horloge du système.
    ///
    /// Une horloge antérieure à 1970 donne l'époque elle-même ; une horloge
    /// au-delà de ce que porte un `i64` en millisecondes donne la valeur
    /// maximale. Ni l'une ni l'autre ne fait échouer l'appelant.
    #[must_use]
    pub fn maintenant() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duree| duree.as_millis())
            .unwrap_or_default();
        Self(i64::try_from(millis).unwrap_or(i64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintenant_est_posterieur_a_l_epoque() {
        assert!(Horodatage::maintenant() > Horodatage::depuis_millis(0));
    }
}

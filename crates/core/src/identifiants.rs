//! Identifiants stables des objets du référentiel.
//!
//! Chaque identifiant est un UUID tiré au hasard à la création de l'objet et
//! qui ne change plus ensuite : il ne dérive d'aucune valeur de l'objet ni
//! d'aucune source, pour survivre à la modification de l'un comme au retrait
//! de l'autre.

use std::fmt;
use std::str::FromStr;

use uuid::Uuid;

/// Erreur de lecture d'un identifiant depuis son écriture textuelle.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("identifiant invalide : « {0} » n'est pas un UUID")]
pub struct IdentifiantInvalide(String);

macro_rules! identifiant {
    ($(#[$doc:meta])* $nom:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $nom(Uuid);

        impl $nom {
            /// Tire un nouvel identifiant, distinct de tous les autres.
            #[must_use]
            pub fn nouveau() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl fmt::Display for $nom {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.hyphenated().fmt(f)
            }
        }

        impl FromStr for $nom {
            type Err = IdentifiantInvalide;

            fn from_str(texte: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(texte)
                    .map(Self)
                    .map_err(|_| IdentifiantInvalide(texte.to_owned()))
            }
        }
    };
}

identifiant!(
    /// Identifiant stable d'une fiche maître, indépendant de toute source.
    // [impl->req~fiche-maitre-identifiant-stable~1]
    FicheId
);

identifiant!(
    /// Identifiant d'une source configurée (carnet CardDAV, compte Google…).
    SourceId
);

identifiant!(
    /// Identifiant d'une décision de la file.
    DecisionId
);

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->req~fiche-maitre-identifiant-stable~1]
    #[test]
    fn deux_identifiants_tires_successivement_different() {
        assert_ne!(FicheId::nouveau(), FicheId::nouveau());
    }

    #[test]
    fn un_identifiant_relu_depuis_son_ecriture_est_le_meme() {
        let id = FicheId::nouveau();

        let relu: Result<FicheId, _> = id.to_string().parse();

        assert_eq!(relu, Ok(id));
    }

    #[test]
    fn un_texte_qui_n_est_pas_un_uuid_est_refuse() {
        let relu: Result<SourceId, _> = "pas-un-uuid".parse();

        assert_eq!(relu, Err(IdentifiantInvalide("pas-un-uuid".to_owned())));
    }
}

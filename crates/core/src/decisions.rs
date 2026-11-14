//! Décisions soumises à l'utilisateur : ce que la synchronisation ne
//! tranche pas seule.

use crate::champ::Champ;
use crate::identifiants::{DecisionId, FicheId, SourceId};
use crate::temps::Horodatage;

/// Ce sur quoi porte une décision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nature {
    /// Un champ a changé différemment dans le référentiel et sur la source.
    Divergence {
        /// Champ concerné.
        champ: Champ,
    },
    /// La fiche a disparu de la source.
    Suppression,
    /// La fiche importée ressemble à une fiche existante, sans certitude.
    DoublonProbable {
        /// Fiche existante à laquelle elle ressemble.
        autre: FicheId,
    },
    /// Une nouvelle photo est proposée pour la fiche (lot 6).
    NouvellePhoto,
}

/// Statut d'une décision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Statut {
    /// À trancher.
    EnAttente,
    /// Remise à plus tard par l'utilisateur.
    Reportee,
    /// Tranchée.
    Resolue,
}

/// Valeurs en présence au moment de la décision, en textes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Versions {
    /// Valeurs du référentiel.
    pub referentiel: Vec<String>,
    /// Valeurs de la source.
    pub source: Vec<String>,
    /// Valeurs du dernier état synchronisé.
    pub base: Vec<String>,
}

/// Issue choisie pour une divergence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Garder la valeur du référentiel.
    GarderReferentiel,
    /// Garder la valeur de la source.
    GarderSource,
    /// Garder les valeurs des deux côtés (champ multivalué seulement).
    GarderLesDeux,
    /// Retenir des valeurs saisies à la main.
    Saisir(Vec<String>),
}

/// Issue choisie pour une suppression détectée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSuppression {
    /// Supprimer la fiche du référentiel et des autres sources.
    Propager,
    /// Réécrire la fiche sur la source.
    Restaurer,
    /// Garder la fiche, sans plus de lien avec cette source.
    Detacher,
}

/// Issue choisie pour un doublon probable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueDoublon {
    /// Réunir les deux fiches en une, la fiche existante subsistant.
    Fusionner,
    /// Garder deux fiches distinctes.
    Distinguer,
}

/// Résolution d'une décision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Résolution d'une divergence.
    Divergence(Action),
    /// Résolution d'une suppression détectée.
    Suppression(IssueSuppression),
    /// Résolution d'un doublon probable.
    Doublon(IssueDoublon),
}

/// Choix offert à l'utilisateur pour une décision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choix {
    /// Garder la valeur du référentiel.
    GarderReferentiel,
    /// Garder la valeur de la source.
    GarderSource,
    /// Garder les deux.
    GarderLesDeux,
    /// Saisir une valeur à la main.
    Saisir,
    /// Propager la suppression partout.
    Propager,
    /// Restaurer la fiche sur la source.
    Restaurer,
    /// Détacher la fiche de la source.
    Detacher,
    /// Réunir les deux fiches.
    Fusionner,
    /// Garder deux fiches distinctes.
    Distinguer,
    /// Remettre à plus tard.
    Reporter,
}

/// Décision de la file.
#[derive(Debug, Clone, PartialEq, Eq)]
// [impl->req~contenu-decision~1]
pub struct Decision {
    /// Identifiant de la décision.
    pub id: DecisionId,
    /// Ce sur quoi elle porte.
    pub nature: Nature,
    /// Fiche concernée.
    pub fiche: FicheId,
    /// Source concernée.
    pub source: SourceId,
    /// Valeurs en présence.
    pub versions: Versions,
    /// Date de création.
    pub creee_le: Horodatage,
    /// Statut.
    pub statut: Statut,
    /// Résolution retenue, une fois résolue.
    pub resolution: Option<Resolution>,
}

impl Decision {
    /// Nouvelle décision en attente.
    #[must_use]
    pub fn new(
        nature: Nature,
        fiche: FicheId,
        source: SourceId,
        versions: Versions,
        creee_le: Horodatage,
    ) -> Self {
        Self {
            id: DecisionId::nouveau(),
            nature,
            fiche,
            source,
            versions,
            creee_le,
            statut: Statut::EnAttente,
            resolution: None,
        }
    }

    /// Vrai si la décision attend encore l'utilisateur : en attente ou
    /// reportée.
    #[must_use]
    pub fn est_ouverte(&self) -> bool {
        self.statut != Statut::Resolue
    }

    /// Choix offerts pour cette décision, report compris.
    // [impl->req~actions-de-resolution~1]
    // [impl->req~issues-suppression-detectee~1]
    #[must_use]
    pub fn choix(&self) -> Vec<Choix> {
        let mut choix = match self.nature {
            Nature::Divergence { champ } => {
                let mut choix = vec![Choix::GarderReferentiel, Choix::GarderSource];
                if champ.est_multivalue() {
                    choix.push(Choix::GarderLesDeux);
                }
                choix.push(Choix::Saisir);
                choix
            }
            Nature::Suppression => vec![Choix::Propager, Choix::Restaurer, Choix::Detacher],
            Nature::DoublonProbable { .. } => vec![Choix::Fusionner, Choix::Distinguer],
            Nature::NouvellePhoto => Vec::new(),
        };
        choix.push(Choix::Reporter);
        choix
    }

    /// Vrai si la résolution convient à la nature de la décision.
    #[must_use]
    pub fn admet(&self, resolution: &Resolution) -> bool {
        match (self.nature, resolution) {
            (Nature::Divergence { champ }, Resolution::Divergence(action)) => {
                *action != Action::GarderLesDeux || champ.est_multivalue()
            }
            (Nature::Suppression, Resolution::Suppression(_))
            | (Nature::DoublonProbable { .. }, Resolution::Doublon(_)) => true,
            _ => false,
        }
    }

    /// Vrai si l'autre décision est un cas similaire : même nature (même
    /// champ pour une divergence), même source.
    #[must_use]
    pub fn est_similaire_a(&self, autre: &Self) -> bool {
        let meme_nature = match (self.nature, autre.nature) {
            (Nature::DoublonProbable { .. }, Nature::DoublonProbable { .. }) => true,
            (a, b) => a == b,
        };
        meme_nature && self.source == autre.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision(nature: Nature) -> Decision {
        Decision::new(
            nature,
            FicheId::nouveau(),
            SourceId::nouveau(),
            Versions {
                referentiel: vec!["Camille Ex.".to_owned()],
                source: vec!["C. Exemple".to_owned()],
                base: vec!["Camille Exemple".to_owned()],
            },
            Horodatage::depuis_millis(42),
        )
    }

    // [utest->req~contenu-decision~1]
    #[test]
    fn une_decision_de_divergence_porte_ses_versions_sa_source_sa_date_et_son_statut() {
        let source = SourceId::nouveau();
        let versions = Versions {
            referentiel: vec!["r".to_owned()],
            source: vec!["s".to_owned()],
            base: vec!["b".to_owned()],
        };
        let date = Horodatage::depuis_millis(7);

        let lue = Decision::new(
            Nature::Divergence {
                champ: Champ::NomAffiche,
            },
            FicheId::nouveau(),
            source,
            versions.clone(),
            date,
        );

        assert_eq!(
            (lue.versions, lue.source, lue.creee_le, lue.statut),
            (versions, source, date, Statut::EnAttente)
        );
    }

    // [utest->req~actions-de-resolution~1]
    #[test]
    fn garder_les_deux_n_est_pas_offert_pour_un_champ_a_valeur_unique() {
        let divergence = decision(Nature::Divergence {
            champ: Champ::NomAffiche,
        });

        assert_eq!(
            divergence.choix(),
            [
                Choix::GarderReferentiel,
                Choix::GarderSource,
                Choix::Saisir,
                Choix::Reporter
            ]
        );
    }

    // [utest->req~actions-de-resolution~1]
    #[test]
    fn garder_les_deux_est_offert_pour_un_champ_multivalue() {
        let divergence = decision(Nature::Divergence {
            champ: Champ::Telephone,
        });

        assert!(divergence.choix().contains(&Choix::GarderLesDeux));
    }

    #[test]
    fn garder_les_deux_n_est_pas_admis_pour_un_champ_a_valeur_unique() {
        let divergence = decision(Nature::Divergence {
            champ: Champ::NomAffiche,
        });

        assert!(!divergence.admet(&Resolution::Divergence(Action::GarderLesDeux)));
    }

    // [utest->req~issues-suppression-detectee~1]
    #[test]
    fn une_suppression_offre_trois_issues_et_le_report() {
        let suppression = decision(Nature::Suppression);

        assert_eq!(
            suppression.choix(),
            [
                Choix::Propager,
                Choix::Restaurer,
                Choix::Detacher,
                Choix::Reporter
            ]
        );
    }

    #[test]
    fn deux_divergences_du_meme_champ_et_de_la_meme_source_sont_similaires() {
        let premiere = decision(Nature::Divergence {
            champ: Champ::Organisation,
        });
        let seconde = Decision {
            id: DecisionId::nouveau(),
            fiche: FicheId::nouveau(),
            ..premiere.clone()
        };

        assert!(premiere.est_similaire_a(&seconde));
    }

    #[test]
    fn deux_divergences_de_champs_differents_ne_sont_pas_similaires() {
        let premiere = decision(Nature::Divergence {
            champ: Champ::Organisation,
        });
        let seconde = Decision {
            nature: Nature::Divergence {
                champ: Champ::Fonction,
            },
            ..premiere.clone()
        };

        assert!(!premiere.est_similaire_a(&seconde));
    }
}

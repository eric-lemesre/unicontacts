//! Sources configurées, leurs réglages et l'autorité qui fait foi par champ.

use std::collections::BTreeMap;

use crate::champ::Champ;
use crate::identifiants::SourceId;
use crate::vcard::Version;

/// Sens dans lequel une source se synchronise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Lue et écrite.
    #[default]
    Bidirectionnelle,
    /// Lue seulement : le cœur n'y écrit jamais.
    LectureSeule,
    /// Écrite seulement : ses modifications sont ignorées et recouvertes.
    EcritureSeule,
}

/// Qui fait foi, de façon générale, pour une source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Autorite {
    /// Le référentiel fait foi.
    #[default]
    Referentiel,
    /// Cette source fait foi : elle est la source de référence générale.
    CetteSource,
}

/// Traitement d'une suppression détectée sur une source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Suppressions {
    /// Créer une décision.
    #[default]
    Demander,
    /// Supprimer la fiche du référentiel et des autres sources.
    Propager,
    /// Détacher la fiche de la source, sans rien supprimer.
    Ignorer,
}

/// Traitement d'un conflit entre une source et le référentiel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Conflits {
    /// Créer une décision.
    #[default]
    Demander,
    /// Retenir la valeur de qui fait foi pour le champ.
    AutoriteGagne,
}

/// Réglages d'une source ; les valeurs par défaut sont celles d'une source
/// nouvellement ajoutée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
// [impl->req~reglages-par-source~1]
pub struct Reglages {
    /// Sens de synchronisation.
    pub direction: Direction,
    /// Autorité générale.
    pub autorite: Autorite,
    /// Traitement des suppressions.
    pub suppressions: Suppressions,
    /// Traitement des conflits.
    pub conflits: Conflits,
}

/// Source configurée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Identifiant de la source.
    pub id: SourceId,
    /// Nom donné par l'utilisateur.
    pub nom: String,
    /// Version des vCard que sert la source.
    pub version: Version,
    /// Réglages de la source.
    pub reglages: Reglages,
}

/// Qui fait foi pour un champ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuiFaitFoi {
    /// Le référentiel.
    Referentiel,
    /// Une source.
    Source(SourceId),
}

/// Qui l'emporte quand un champ diverge entre une source et le référentiel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gagnant {
    /// Personne : la divergence est soumise à décision.
    Personne,
    /// La source.
    Source,
    /// Le référentiel.
    Referentiel,
}

/// Erreur de réglage.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurReglage {
    /// La source n'est pas configurée.
    #[error("source inconnue : {0}")]
    SourceInconnue(SourceId),
    /// Une autre source est déjà la référence générale.
    #[error("la source « {source_en_place} » est déjà la source de référence générale")]
    ReferenceGeneraleDejaDesignee {
        /// Nom de la source déjà désignée.
        source_en_place: String,
    },
    /// Une autre source fait déjà foi pour ce champ.
    #[error("la source « {source_en_place} » fait déjà foi pour le champ {champ:?}")]
    ReferenceDuChampDejaDesignee {
        /// Champ concerné.
        champ: Champ,
        /// Nom de la source déjà désignée.
        source_en_place: String,
    },
}

/// Ensemble des sources configurées et des désignations d'autorité.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sources {
    sources: BTreeMap<SourceId, Source>,
    par_champ: BTreeMap<Champ, SourceId>,
}

impl Sources {
    /// Ajoute une source, avec les réglages par défaut, et rend son
    /// identifiant.
    // [impl->req~reglages-par-source~1]
    pub fn ajouter(&mut self, nom: impl Into<String>, version: Version) -> SourceId {
        let id = SourceId::nouveau();
        self.restaurer(Source {
            id,
            nom: nom.into(),
            version,
            reglages: Reglages::default(),
        });
        id
    }

    /// Replace une source relue depuis le stockage, telle quelle.
    pub fn restaurer(&mut self, source: Source) {
        self.sources.insert(source.id, source);
    }

    /// Replace une désignation d'autorité relue depuis le stockage.
    pub fn restaurer_designation(&mut self, champ: Champ, source: SourceId) {
        self.par_champ.insert(champ, source);
    }

    /// Source configurée, si elle existe.
    #[must_use]
    pub fn source(&self, id: SourceId) -> Option<&Source> {
        self.sources.get(&id)
    }

    /// Toutes les sources configurées.
    pub fn toutes(&self) -> impl Iterator<Item = &Source> {
        self.sources.values()
    }

    /// Désignations d'autorité par champ.
    pub fn designations(&self) -> impl Iterator<Item = (Champ, SourceId)> + '_ {
        self.par_champ.iter().map(|(c, s)| (*c, *s))
    }

    /// Change les réglages d'une source.
    ///
    /// # Errors
    ///
    /// [`ErreurReglage::SourceInconnue`] si la source n'est pas configurée ;
    /// [`ErreurReglage::ReferenceGeneraleDejaDesignee`], qui nomme la source
    /// en place, si une autre source est déjà la référence générale. Les
    /// réglages sont alors inchangés.
    // [impl->req~source-de-reference-unique~2]
    pub fn regler(&mut self, id: SourceId, reglages: Reglages) -> Result<(), ErreurReglage> {
        if reglages.autorite == Autorite::CetteSource
            && let Some(en_place) = self
                .sources
                .values()
                .find(|s| s.id != id && s.reglages.autorite == Autorite::CetteSource)
        {
            return Err(ErreurReglage::ReferenceGeneraleDejaDesignee {
                source_en_place: en_place.nom.clone(),
            });
        }
        let source = self
            .sources
            .get_mut(&id)
            .ok_or(ErreurReglage::SourceInconnue(id))?;
        source.reglages = reglages;
        Ok(())
    }

    /// Désigne la source qui fait foi pour un champ, au-dessus de la
    /// référence générale.
    ///
    /// # Errors
    ///
    /// [`ErreurReglage::SourceInconnue`] si la source n'est pas configurée ;
    /// [`ErreurReglage::ReferenceDuChampDejaDesignee`], qui nomme la source
    /// en place, si une autre source fait déjà foi pour ce champ.
    // [impl->req~autorite-par-champ~1]
    // [impl->req~source-de-reference-unique~2]
    pub fn designer_pour_champ(&mut self, champ: Champ, id: SourceId) -> Result<(), ErreurReglage> {
        if !self.sources.contains_key(&id) {
            return Err(ErreurReglage::SourceInconnue(id));
        }
        match self.par_champ.get(&champ) {
            Some(en_place) if *en_place != id => Err(ErreurReglage::ReferenceDuChampDejaDesignee {
                champ,
                source_en_place: self.nom(*en_place),
            }),
            _ => {
                self.par_champ.insert(champ, id);
                Ok(())
            }
        }
    }

    /// Retire la désignation d'autorité d'un champ : la référence générale
    /// fait de nouveau foi pour lui.
    pub fn retirer_designation(&mut self, champ: Champ) {
        self.par_champ.remove(&champ);
    }

    /// Qui fait foi pour un champ : la source désignée pour lui, à défaut la
    /// source de référence générale, à défaut le référentiel.
    // [impl->req~autorite-par-champ~1]
    #[must_use]
    pub fn qui_fait_foi(&self, champ: Champ) -> QuiFaitFoi {
        self.par_champ
            .get(&champ)
            .copied()
            .or_else(|| {
                self.sources
                    .values()
                    .find(|s| s.reglages.autorite == Autorite::CetteSource)
                    .map(|s| s.id)
            })
            .map_or(QuiFaitFoi::Referentiel, QuiFaitFoi::Source)
    }

    /// Qui l'emporte quand un champ diverge entre une source et le
    /// référentiel.
    ///
    /// Personne si les conflits de la source sont réglés sur « demander ».
    /// Sinon, qui fait foi pour le champ : la source elle-même, ou le
    /// référentiel. Si c'est une autre source, personne : la source
    /// synchronisée ne l'emporte pas, et rien ne dit que le référentiel
    /// porte la valeur de celle qui fait foi.
    // [impl->req~autorite-de-la-reference~2]
    #[must_use]
    pub fn gagnant(&self, source: SourceId, champ: Champ) -> Gagnant {
        let conflits = self.source(source).map(|s| s.reglages.conflits);
        if conflits != Some(Conflits::AutoriteGagne) {
            return Gagnant::Personne;
        }
        match self.qui_fait_foi(champ) {
            QuiFaitFoi::Referentiel => Gagnant::Referentiel,
            QuiFaitFoi::Source(id) if id == source => Gagnant::Source,
            QuiFaitFoi::Source(_) => Gagnant::Personne,
        }
    }

    /// Vrai si le cœur peut écrire vers la source.
    #[must_use]
    pub fn ecrit_vers(&self, source: SourceId) -> bool {
        self.source(source)
            .is_some_and(|s| s.reglages.direction != Direction::LectureSeule)
    }

    /// Vrai si le cœur tient compte de ce que la source contient.
    #[must_use]
    pub fn lit_depuis(&self, source: SourceId) -> bool {
        self.source(source)
            .is_some_and(|s| s.reglages.direction != Direction::EcritureSeule)
    }

    fn nom(&self, id: SourceId) -> String {
        self.source(id)
            .map_or_else(|| id.to_string(), |s| s.nom.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference() -> Reglages {
        Reglages {
            autorite: Autorite::CetteSource,
            ..Reglages::default()
        }
    }

    // [utest->req~reglages-par-source~1]
    #[test]
    fn une_source_nouvellement_ajoutee_a_les_quatre_reglages_par_defaut() {
        let mut sources = Sources::default();

        let id = sources.ajouter("Nextcloud", Version::V3);

        let attendu = Reglages {
            direction: Direction::Bidirectionnelle,
            autorite: Autorite::Referentiel,
            suppressions: Suppressions::Demander,
            conflits: Conflits::Demander,
        };
        assert_eq!(sources.source(id).map(|s| s.reglages), Some(attendu));
    }

    #[test]
    fn une_source_en_lecture_seule_n_est_pas_ecrite() -> Result<(), ErreurReglage> {
        let mut sources = Sources::default();
        let id = sources.ajouter("SIM", Version::V3);
        let reglages = Reglages {
            direction: Direction::LectureSeule,
            ..Reglages::default()
        };

        sources.regler(id, reglages)?;

        assert!(!sources.ecrit_vers(id));
        Ok(())
    }

    mod reference_unique {
        use super::*;

        // [utest->req~source-de-reference-unique~2]
        #[test]
        fn une_seconde_reference_generale_est_refusee_en_nommant_la_premiere()
        -> Result<(), ErreurReglage> {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            let b = sources.ajouter("B", Version::V4);
            sources.regler(a, reference())?;

            let refus = sources.regler(b, reference());

            assert_eq!(
                refus.map_err(|e| e.to_string()),
                Err("la source « A » est déjà la source de référence générale".to_owned())
            );
            Ok(())
        }

        #[test]
        fn la_reference_generale_peut_se_regler_de_nouveau() -> Result<(), ErreurReglage> {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            sources.regler(a, reference())?;

            let reglage = sources.regler(
                a,
                Reglages {
                    conflits: Conflits::AutoriteGagne,
                    ..reference()
                },
            );

            assert_eq!(reglage, Ok(()));
            Ok(())
        }

        // [utest->req~source-de-reference-unique~2]
        #[test]
        fn une_seconde_source_designee_pour_le_meme_champ_est_refusee() -> Result<(), ErreurReglage>
        {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            let b = sources.ajouter("B", Version::V4);
            sources.designer_pour_champ(Champ::Telephone, a)?;

            let refus = sources.designer_pour_champ(Champ::Telephone, b);

            assert_eq!(
                refus,
                Err(ErreurReglage::ReferenceDuChampDejaDesignee {
                    champ: Champ::Telephone,
                    source_en_place: "A".to_owned()
                })
            );
            Ok(())
        }

        // [utest->req~source-de-reference-unique~2]
        #[test]
        fn une_autre_source_peut_etre_designee_pour_un_autre_champ() -> Result<(), ErreurReglage> {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            let b = sources.ajouter("B", Version::V4);
            sources.designer_pour_champ(Champ::Telephone, a)?;

            let designation = sources.designer_pour_champ(Champ::Adresse, b);

            assert_eq!(designation, Ok(()));
            Ok(())
        }
    }

    mod autorite_par_champ {
        use super::*;

        /// Nextcloud référence générale, Google désignée pour les téléphones.
        fn configuration() -> Result<(Sources, SourceId, SourceId), ErreurReglage> {
            let mut sources = Sources::default();
            let nextcloud = sources.ajouter("Nextcloud", Version::V3);
            let google = sources.ajouter("Google", Version::V3);
            sources.regler(nextcloud, reference())?;
            sources.designer_pour_champ(Champ::Telephone, google)?;
            Ok((sources, nextcloud, google))
        }

        // [utest->req~autorite-par-champ~1]
        #[test]
        fn la_source_designee_fait_foi_pour_son_champ() -> Result<(), ErreurReglage> {
            let (sources, _, google) = configuration()?;

            assert_eq!(
                sources.qui_fait_foi(Champ::Telephone),
                QuiFaitFoi::Source(google)
            );
            Ok(())
        }

        // [utest->req~autorite-par-champ~1]
        #[test]
        fn la_reference_generale_fait_foi_pour_les_autres_champs() -> Result<(), ErreurReglage> {
            let (sources, nextcloud, _) = configuration()?;

            assert_eq!(
                sources.qui_fait_foi(Champ::Adresse),
                QuiFaitFoi::Source(nextcloud)
            );
            Ok(())
        }

        #[test]
        fn sans_aucune_designation_le_referentiel_fait_foi() {
            let mut sources = Sources::default();
            sources.ajouter("A", Version::V4);

            assert_eq!(sources.qui_fait_foi(Champ::Email), QuiFaitFoi::Referentiel);
        }
    }

    mod gagnant {
        use super::*;

        fn autorite_gagne() -> Reglages {
            Reglages {
                conflits: Conflits::AutoriteGagne,
                ..reference()
            }
        }

        #[test]
        fn personne_quand_les_conflits_sont_regles_sur_demander() -> Result<(), ErreurReglage> {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            sources.regler(a, reference())?;

            assert_eq!(sources.gagnant(a, Champ::NomAffiche), Gagnant::Personne);
            Ok(())
        }

        // [utest->req~autorite-de-la-reference~2]
        #[test]
        fn la_reference_generale_l_emporte_quand_l_autorite_gagne() -> Result<(), ErreurReglage> {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            sources.regler(a, autorite_gagne())?;

            assert_eq!(sources.gagnant(a, Champ::NomAffiche), Gagnant::Source);
            Ok(())
        }

        // [utest->req~autorite-de-la-reference~2]
        #[test]
        fn la_reference_generale_ne_l_emporte_pas_sur_le_champ_d_une_autre()
        -> Result<(), ErreurReglage> {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            let b = sources.ajouter("B", Version::V4);
            sources.regler(a, autorite_gagne())?;
            sources.designer_pour_champ(Champ::Telephone, b)?;

            assert_ne!(sources.gagnant(a, Champ::Telephone), Gagnant::Source);
            Ok(())
        }

        #[test]
        fn le_referentiel_l_emporte_sans_source_de_reference() -> Result<(), ErreurReglage> {
            let mut sources = Sources::default();
            let a = sources.ajouter("A", Version::V4);
            sources.regler(
                a,
                Reglages {
                    conflits: Conflits::AutoriteGagne,
                    ..Reglages::default()
                },
            )?;

            assert_eq!(sources.gagnant(a, Champ::NomAffiche), Gagnant::Referentiel);
            Ok(())
        }
    }
}

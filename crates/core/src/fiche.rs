//! Fiche maître et ses liens vers les sources.

use std::collections::BTreeMap;

use crate::champ::{Champ, Provenance, Valeur};
use crate::identifiants::{FicheId, SourceId};
use crate::temps::Horodatage;

/// Erreur de modification d'une fiche.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurFiche {
    /// Un champ à valeur unique a reçu plusieurs valeurs.
    #[error("le champ {champ:?} n'admet qu'une valeur, {nombre} reçues")]
    ValeurUniqueAttendue {
        /// Champ concerné.
        champ: Champ,
        /// Nombre de valeurs reçues.
        nombre: usize,
    },
}

/// Correspondance entre une fiche maître et sa copie dans une source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lien {
    /// Source concernée.
    pub source: SourceId,
    /// Identifiant de la copie dans la source (UID de la vCard).
    pub uid: String,
    /// Étiquette de version de la copie, si la source en fournit (ETag).
    pub etag: Option<String>,
    /// Dernier état synchronisé entre le référentiel et la source, champ par
    /// champ : la troisième voie de la fusion.
    pub base: BTreeMap<Champ, Vec<String>>,
    /// Dernière vCard connue de la source, telle quelle : elle porte les
    /// propriétés non interprétées, à renvoyer inchangées à cette source.
    pub carte: Option<String>,
}

impl Lien {
    /// Lien vers une copie que le référentiel ne connaît pas encore.
    #[must_use]
    pub fn new(source: SourceId, uid: impl Into<String>) -> Self {
        Self {
            source,
            uid: uid.into(),
            etag: None,
            base: BTreeMap::new(),
            carte: None,
        }
    }
}

/// Fiche maître : la version de référence d'un contact.
///
/// Son identifiant est tiré à la création et ne change plus, ni quand ses
/// valeurs changent, ni quand une source s'y lie ou s'en détache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fiche {
    id: FicheId,
    valeurs: BTreeMap<Champ, Vec<Valeur>>,
    liens: BTreeMap<SourceId, Lien>,
}

impl Fiche {
    /// Nouvelle fiche vide, d'identifiant neuf.
    // [impl->req~fiche-maitre-identifiant-stable~1]
    #[must_use]
    pub fn nouvelle() -> Self {
        Self::restaurer(FicheId::nouveau(), BTreeMap::new(), Vec::new())
    }

    /// Fiche relue depuis le stockage, avec son identifiant d'origine.
    #[must_use]
    pub fn restaurer(
        id: FicheId,
        valeurs: BTreeMap<Champ, Vec<Valeur>>,
        liens: impl IntoIterator<Item = Lien>,
    ) -> Self {
        Self {
            id,
            valeurs,
            liens: liens.into_iter().map(|l| (l.source, l)).collect(),
        }
    }

    /// Identifiant stable de la fiche.
    #[must_use]
    pub const fn id(&self) -> FicheId {
        self.id
    }

    /// Valeurs d'un champ, dans leur ordre ; vide si le champ n'est pas
    /// renseigné.
    #[must_use]
    pub fn valeurs(&self, champ: Champ) -> &[Valeur] {
        self.valeurs.get(&champ).map_or(&[], Vec::as_slice)
    }

    /// Textes des valeurs d'un champ.
    #[must_use]
    pub fn textes(&self, champ: Champ) -> Vec<String> {
        self.valeurs(champ)
            .iter()
            .map(|v| v.texte.clone())
            .collect()
    }

    /// Champs renseignés et leurs valeurs.
    pub fn champs(&self) -> impl Iterator<Item = (Champ, &[Valeur])> {
        self.valeurs.iter().map(|(c, v)| (*c, v.as_slice()))
    }

    /// Remplace les valeurs d'un champ, chacune avec sa provenance.
    ///
    /// # Errors
    ///
    /// [`ErreurFiche::ValeurUniqueAttendue`] si un champ à valeur unique en
    /// reçoit plusieurs ; la fiche est alors inchangée.
    // [impl->req~provenance-par-champ~1]
    pub fn definir(&mut self, champ: Champ, valeurs: Vec<Valeur>) -> Result<(), ErreurFiche> {
        if !champ.est_multivalue() && valeurs.len() > 1 {
            return Err(ErreurFiche::ValeurUniqueAttendue {
                champ,
                nombre: valeurs.len(),
            });
        }
        if valeurs.is_empty() {
            self.valeurs.remove(&champ);
        } else {
            self.valeurs.insert(champ, valeurs);
        }
        Ok(())
    }

    /// Modifie un champ dans l'application : chaque valeur nouvelle prend
    /// pour provenance le référentiel, à la date donnée ; une valeur déjà
    /// présente à l'identique garde sa provenance et ses types.
    ///
    /// # Errors
    ///
    /// [`ErreurFiche::ValeurUniqueAttendue`], comme [`Fiche::definir`].
    // [impl->req~provenance-par-champ~1]
    pub fn modifier(
        &mut self,
        champ: Champ,
        textes: &[&str],
        date: Horodatage,
    ) -> Result<(), ErreurFiche> {
        let anciennes = self.valeurs(champ);
        let valeurs = textes
            .iter()
            .map(|texte| {
                anciennes
                    .iter()
                    .find(|v| v.texte == *texte)
                    .cloned()
                    .unwrap_or_else(|| Valeur::new(*texte, Provenance::referentiel(date)))
            })
            .collect();
        self.definir(champ, valeurs)
    }

    /// Lien de la fiche vers une source, s'il existe.
    #[must_use]
    pub fn lien(&self, source: SourceId) -> Option<&Lien> {
        self.liens.get(&source)
    }

    /// Lien modifiable de la fiche vers une source, s'il existe.
    pub fn lien_mut(&mut self, source: SourceId) -> Option<&mut Lien> {
        self.liens.get_mut(&source)
    }

    /// Liens de la fiche vers ses sources.
    pub fn liens(&self) -> impl Iterator<Item = &Lien> {
        self.liens.values()
    }

    /// Lie la fiche à une source ; remplace le lien existant vers cette
    /// source.
    pub fn lier(&mut self, lien: Lien) {
        self.liens.insert(lien.source, lien);
    }

    /// Détache la fiche d'une source et rend le lien retiré. La fiche et son
    /// identifiant subsistent.
    // [impl->req~fiche-maitre-identifiant-stable~1]
    pub fn detacher(&mut self, source: SourceId) -> Option<Lien> {
        self.liens.remove(&source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::champ::Origine;

    fn date(millis: i64) -> Horodatage {
        Horodatage::depuis_millis(millis)
    }

    mod identifiant {
        use super::*;

        // [utest->req~fiche-maitre-identifiant-stable~1]
        #[test]
        fn reste_inchange_quand_une_des_deux_sources_est_detachee() {
            let (a, b) = (SourceId::nouveau(), SourceId::nouveau());
            let mut fiche = Fiche::nouvelle();
            fiche.lier(Lien::new(a, "uid-a"));
            fiche.lier(Lien::new(b, "uid-b"));
            let avant = fiche.id();

            fiche.detacher(a);

            assert_eq!(fiche.id(), avant);
        }

        // [utest->req~fiche-maitre-identifiant-stable~1]
        #[test]
        fn differe_entre_deux_fiches_creees_avec_les_memes_valeurs() -> Result<(), ErreurFiche> {
            let mut premiere = Fiche::nouvelle();
            let mut seconde = Fiche::nouvelle();
            premiere.modifier(Champ::NomAffiche, &["Camille Exemple"], date(1))?;
            seconde.modifier(Champ::NomAffiche, &["Camille Exemple"], date(1))?;

            assert_ne!(premiere.id(), seconde.id());
            Ok(())
        }

        // [utest->req~fiche-maitre-identifiant-stable~1]
        #[test]
        fn reste_inchange_quand_la_fiche_est_modifiee() -> Result<(), ErreurFiche> {
            let mut fiche = Fiche::nouvelle();
            let avant = fiche.id();

            fiche.modifier(Champ::Note, &["nouvelle note"], date(1))?;

            assert_eq!(fiche.id(), avant);
            Ok(())
        }
    }

    mod provenance {
        use super::*;

        // [utest->req~provenance-par-champ~1]
        #[test]
        fn une_valeur_recue_d_une_source_nomme_la_source_et_la_date_de_reception()
        -> Result<(), ErreurFiche> {
            let source = SourceId::nouveau();
            let mut fiche = Fiche::nouvelle();

            fiche.definir(
                Champ::Email,
                vec![Valeur::new(
                    "camille@example.org",
                    Provenance::source(source, date(42)),
                )],
            )?;

            let provenances: Vec<_> = fiche
                .valeurs(Champ::Email)
                .iter()
                .map(|v| v.provenance)
                .collect();
            assert_eq!(provenances, [Provenance::source(source, date(42))]);
            Ok(())
        }

        // [utest->req~provenance-par-champ~1]
        #[test]
        fn une_valeur_modifiee_dans_l_application_vient_du_referentiel_a_la_date_de_modification()
        -> Result<(), ErreurFiche> {
            let source = SourceId::nouveau();
            let mut fiche = Fiche::nouvelle();
            fiche.definir(
                Champ::Organisation,
                vec![Valeur::new("Ancienne", Provenance::source(source, date(1)))],
            )?;

            fiche.modifier(Champ::Organisation, &["Nouvelle"], date(7))?;

            let provenances: Vec<_> = fiche
                .valeurs(Champ::Organisation)
                .iter()
                .map(|v| v.provenance)
                .collect();
            assert_eq!(provenances, [Provenance::referentiel(date(7))]);
            Ok(())
        }

        #[test]
        fn une_valeur_inchangee_garde_sa_provenance_a_la_modification_du_champ()
        -> Result<(), ErreurFiche> {
            let source = SourceId::nouveau();
            let mut fiche = Fiche::nouvelle();
            fiche.definir(
                Champ::Telephone,
                vec![Valeur::new(
                    "+33639980000",
                    Provenance::source(source, date(1)),
                )],
            )?;

            fiche.modifier(Champ::Telephone, &["+33639980000", "+33639980001"], date(9))?;

            let origines: Vec<_> = fiche
                .valeurs(Champ::Telephone)
                .iter()
                .map(|v| v.provenance.origine)
                .collect();
            assert_eq!(origines, [Origine::Source(source), Origine::Referentiel]);
            Ok(())
        }
    }

    #[test]
    fn un_champ_a_valeur_unique_refuse_deux_valeurs() {
        let mut fiche = Fiche::nouvelle();

        let resultat = fiche.modifier(Champ::NomAffiche, &["Un", "Deux"], date(1));

        assert_eq!(
            resultat,
            Err(ErreurFiche::ValeurUniqueAttendue {
                champ: Champ::NomAffiche,
                nombre: 2
            })
        );
    }

    #[test]
    fn un_champ_vide_n_est_plus_renseigne() -> Result<(), ErreurFiche> {
        let mut fiche = Fiche::nouvelle();
        fiche.modifier(Champ::Note, &["note"], date(1))?;

        fiche.modifier(Champ::Note, &[], date(2))?;

        assert_eq!(fiche.champs().count(), 0);
        Ok(())
    }
}

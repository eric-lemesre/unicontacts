//! Import de contacts lus hors du cœur, comme le stockage local Android.
//!
//! Chaque contact lu devient la copie vCard d'une source, que le moteur de
//! synchronisation rapproche du référentiel : détection des doublons pour
//! un contact inconnu, fusion pour un contact déjà importé (ADR-0015). Le
//! cœur reste seul à connaître le format vCard (ADR-0004).

use std::collections::BTreeMap;

use crate::champ::{Champ, Provenance, Valeur};
use crate::dates::{Date, DateDeFiche, TypeDate};
use crate::fiche::Fiche;
use crate::identifiants::SourceId;
use crate::referentiel::Referentiel;
use crate::reglages::{Direction, ErreurReglage, Reglages, Suppressions};
use crate::synchronisation::{CopieDistante, ErreurSynchronisation, Rapport};
use crate::temps::Horodatage;
use crate::vcard::{Carte, Version};

/// Valeur d'un champ, telle que la source la porte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValeurLue {
    /// Champ de la valeur ; ni relation, ni groupe, ni date.
    pub champ: Champ,
    /// Texte de la valeur ; les champs structurés (`N`, `ADR`) gardent
    /// leurs composantes séparées par `;`.
    pub texte: String,
    /// Types de la valeur (`home`, `work`, `cell`…).
    pub types: Vec<String>,
}

/// Contact lu dans une source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContactLu {
    /// Identifiant du contact dans sa source, stable d'un import à l'autre.
    pub uid: String,
    /// Valeurs du contact.
    pub valeurs: Vec<ValeurLue>,
    /// Dates du contact : type et texte de la date.
    pub dates: Vec<(TypeDate, String)>,
}

/// Résultat d'un import.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Import {
    /// Rapport du moteur de synchronisation.
    pub rapport: Rapport,
    /// Dates illisibles, laissées de côté : identifiant du contact et texte.
    pub dates_illisibles: Vec<(String, String)>,
}

impl ContactLu {
    /// Copie vCard 4.0 du contact. Un champ à valeur unique qui en reçoit
    /// plusieurs garde la première, sauf la note, dont les textes se
    /// joignent ; une date illisible est rendue à part, sans arrêter le
    /// reste.
    #[must_use]
    pub fn copie(&self) -> (CopieDistante, Vec<String>) {
        let provenance = Provenance::referentiel(Horodatage::depuis_millis(0));
        let mut fiche = Fiche::nouvelle();
        let mut par_champ: BTreeMap<Champ, Vec<Valeur>> = BTreeMap::new();
        for lue in &self.valeurs {
            let texte = lue.texte.trim();
            if texte.is_empty()
                || matches!(lue.champ, Champ::Relation | Champ::Groupes | Champ::Dates)
            {
                continue;
            }
            par_champ
                .entry(lue.champ)
                .or_default()
                .push(Valeur::new(texte, provenance).avec_types(&lue.types));
        }
        let mut illisibles = Vec::new();
        for (type_date, texte) in &self.dates {
            let Some(date) = Date::lire(texte) else {
                illisibles.push(texte.clone());
                continue;
            };
            let (champ, texte) = if *type_date == TypeDate::Naissance {
                (Champ::Anniversaire, date.texte())
            } else {
                let date = DateDeFiche {
                    type_date: type_date.clone(),
                    date,
                };
                (Champ::Dates, date.texte())
            };
            par_champ
                .entry(champ)
                .or_default()
                .push(Valeur::new(texte, provenance));
        }
        for (champ, mut valeurs) in par_champ {
            if champ.est_multivalue() {
                fiche.definir_multivalue(champ, valeurs);
            } else if champ == Champ::Note {
                let jointe = valeurs
                    .iter()
                    .map(|v| v.texte.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n");
                fiche.definir_unique(champ, Valeur::new(jointe, provenance));
            } else {
                valeurs.truncate(1);
                if let Some(premiere) = valeurs.pop() {
                    fiche.definir_unique(champ, premiere);
                }
            }
        }
        let contenu = Carte::reecrire(None, Version::V4, &self.uid, &fiche).ecrire();
        let copie = CopieDistante {
            uid: self.uid.clone(),
            etag: None,
            contenu,
        };
        (copie, illisibles)
    }
}

impl Referentiel {
    /// Ajoute une source d'import : lecture seule, et suppressions ignorées,
    /// de sorte que le moteur n'y écrit jamais et qu'un contact disparu
    /// détache sa fiche sans la toucher (ADR-0015).
    ///
    /// # Errors
    ///
    /// [`ErreurReglage`] si les réglages ne peuvent s'appliquer.
    // [impl->req~import-stockage-local~1]
    // [impl->req~source-sim-lecture-seule~1]
    pub fn ajouter_source_import(&mut self, nom: &str) -> Result<SourceId, ErreurReglage> {
        let source = self.sources_mut().ajouter(nom, Version::V4);
        self.sources_mut().regler(
            source,
            Reglages {
                direction: Direction::LectureSeule,
                suppressions: Suppressions::Ignorer,
                ..Reglages::default()
            },
        )?;
        Ok(source)
    }

    /// Importe les contacts lus d'une source, d'après leur liste complète :
    /// un contact inconnu passe par la détection des doublons, un contact
    /// déjà importé se fusionne avec sa fiche, un contact disparu est traité
    /// selon le réglage des suppressions de la source.
    ///
    /// # Errors
    ///
    /// [`ErreurSynchronisation::SourceInconnue`] si la source n'est pas
    /// configurée.
    // [impl->req~import-stockage-local~1]
    pub fn importer_contacts(
        &mut self,
        source: SourceId,
        contacts: &[ContactLu],
        date: Horodatage,
    ) -> Result<Import, ErreurSynchronisation> {
        let mut dates_illisibles = Vec::new();
        let copies: Vec<CopieDistante> = contacts
            .iter()
            .map(|contact| {
                let (copie, illisibles) = contact.copie();
                dates_illisibles.extend(illisibles.into_iter().map(|t| (contact.uid.clone(), t)));
                copie
            })
            .collect();
        let rapport = self.synchroniser(source, &copies, date)?;
        Ok(Import {
            rapport,
            dates_illisibles,
        })
    }

    /// Identifiants des copies d'une source liées à une fiche : les
    /// contacts déjà versés au référentiel.
    #[must_use]
    pub fn copies_liees(&self, source: SourceId) -> Vec<String> {
        let mut uids: Vec<String> = self
            .fiches()
            .filter_map(|f| f.lien(source).map(|l| l.uid.clone()))
            .collect();
        uids.sort();
        uids
    }
}

#[cfg(test)]
mod tests;

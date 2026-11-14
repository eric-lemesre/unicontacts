//! Lecture et écriture des vCard 3.0 (RFC 2426) et 4.0 (RFC 6350).
//!
//! Une vCard lue garde chacune de ses propriétés telle quelle, lignes
//! repliées comprises. À la réécriture vers la même source, une propriété
//! dont la valeur n'a pas changé est rendue à l'octet près, et une propriété
//! que le cœur n'interprète pas l'est toujours : aucune information de la
//! source ne se perd, et le serveur ne voit changer que ce qui a changé.
// [impl->req~vcard-3-et-4~1]

mod ecriture;
mod lecture;

use std::collections::BTreeMap;

pub use lecture::lire;

use crate::champ::{Champ, Provenance, Valeur};
use crate::identifiants::SourceId;
use crate::temps::Horodatage;

/// Version d'une vCard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
    /// vCard 3.0, RFC 2426.
    V3,
    /// vCard 4.0, RFC 6350.
    V4,
}

impl Version {
    /// Valeur de la propriété `VERSION`.
    #[must_use]
    pub const fn texte(self) -> &'static str {
        match self {
            Self::V3 => "3.0",
            Self::V4 => "4.0",
        }
    }

    /// Version déclarée par une propriété `VERSION`, si elle est prise en
    /// charge.
    #[must_use]
    pub fn depuis_texte(texte: &str) -> Option<Self> {
        match texte.trim() {
            "3.0" => Some(Self::V3),
            "4.0" => Some(Self::V4),
            _ => None,
        }
    }
}

/// Erreur de lecture d'une vCard.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurVcard {
    /// Une propriété apparaît hors d'un bloc `BEGIN:VCARD` … `END:VCARD`.
    #[error("ligne {ligne} : propriété hors d'une vCard (BEGIN:VCARD attendu)")]
    HorsCarte {
        /// Numéro de la ligne, à partir de 1.
        ligne: usize,
    },
    /// Le texte se termine avant le `END:VCARD` d'une carte ouverte.
    #[error("vCard ouverte ligne {ligne} et jamais fermée (END:VCARD attendu)")]
    FinManquante {
        /// Ligne du `BEGIN:VCARD` resté ouvert.
        ligne: usize,
    },
    /// Une carte ne déclare pas sa version.
    #[error("vCard ouverte ligne {ligne} sans propriété VERSION")]
    VersionManquante {
        /// Ligne du `BEGIN:VCARD`.
        ligne: usize,
    },
    /// Une carte déclare une version autre que 3.0 ou 4.0.
    #[error("ligne {ligne} : version « {version} » non prise en charge (3.0 ou 4.0)")]
    VersionNonPriseEnCharge {
        /// Ligne de la propriété `VERSION`.
        ligne: usize,
        /// Version déclarée.
        version: String,
    },
    /// Une ligne ne suit pas la forme `nom;paramètres:valeur`.
    #[error("ligne {ligne} : ligne de contenu invalide (« : » attendu)")]
    LigneInvalide {
        /// Numéro de la ligne, à partir de 1.
        ligne: usize,
    },
}

/// Paramètre d'une propriété (`TYPE=work,voice`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parametre {
    /// Nom du paramètre, en majuscules.
    pub nom: String,
    /// Valeurs du paramètre, guillemets retirés.
    pub valeurs: Vec<String>,
}

/// Propriété d'une vCard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Propriete {
    /// Groupe de la propriété (`item1` dans `item1.TEL`), s'il y en a un.
    pub groupe: Option<String>,
    /// Nom de la propriété, en majuscules.
    pub nom: String,
    /// Paramètres, dans leur ordre.
    pub parametres: Vec<Parametre>,
    /// Valeur, telle qu'écrite dans la vCard (échappements compris).
    pub valeur: String,
    /// Lignes d'origine, repliement compris, séparées par CRLF ; absent pour
    /// une propriété produite par le cœur.
    brut: Option<String>,
}

impl Propriete {
    /// Valeurs du paramètre `TYPE`, en minuscules, triées et sans doublon.
    #[must_use]
    pub fn types(&self) -> Vec<String> {
        let mut types: Vec<String> = self
            .parametres
            .iter()
            .filter(|p| p.nom == "TYPE")
            .flat_map(|p| p.valeurs.iter())
            .flat_map(|v| v.split(','))
            .map(|t| t.trim().to_ascii_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        types.sort();
        types.dedup();
        types
    }
}

/// Une vCard lue ou à écrire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carte {
    /// Version de la carte.
    pub version: Version,
    /// Propriétés, dans leur ordre, hors `BEGIN`, `VERSION` et `END`.
    pub proprietes: Vec<Propriete>,
}

impl Carte {
    /// UID de la carte, s'il est déclaré.
    #[must_use]
    pub fn uid(&self) -> Option<String> {
        self.proprietes
            .iter()
            .find(|p| p.nom == "UID")
            .map(|p| p.valeur.trim().to_owned())
    }

    /// Indices des propriétés interprétées, avec leur champ.
    ///
    /// Un champ à valeur unique n'interprète que sa première occurrence :
    /// les suivantes sont conservées comme les propriétés inconnues.
    fn interpretees(&self) -> BTreeMap<usize, Champ> {
        let mut vus = Vec::new();
        let mut interpretees = BTreeMap::new();
        for (indice, propriete) in self.proprietes.iter().enumerate() {
            let Some(champ) = Champ::depuis_propriete_vcard(&propriete.nom) else {
                continue;
            };
            if champ.est_multivalue() || !vus.contains(&champ) {
                vus.push(champ);
                interpretees.insert(indice, champ);
            }
        }
        interpretees
    }

    /// Valeurs des champs interprétés, avec pour provenance la source et la
    /// date de réception données.
    #[must_use]
    pub fn valeurs(&self, source: SourceId, date: Horodatage) -> BTreeMap<Champ, Vec<Valeur>> {
        let mut valeurs: BTreeMap<Champ, Vec<Valeur>> = BTreeMap::new();
        for (indice, champ) in self.interpretees() {
            let Some(propriete) = self.proprietes.get(indice) else {
                continue;
            };
            let valeur = Valeur::new(
                ecriture::decoder(champ, &propriete.valeur),
                Provenance::source(source, date),
            )
            .avec_types(propriete.types());
            valeurs.entry(champ).or_default().push(valeur);
        }
        valeurs
    }

    /// Propriétés non interprétées : celles que le cœur conserve telles
    /// quelles pour la source.
    pub fn conservees(&self) -> impl Iterator<Item = &Propriete> {
        let interpretees = self.interpretees();
        self.proprietes
            .iter()
            .enumerate()
            .filter(move |(indice, _)| !interpretees.contains_key(indice))
            .map(|(_, propriete)| propriete)
    }
}

#[cfg(test)]
mod tests;

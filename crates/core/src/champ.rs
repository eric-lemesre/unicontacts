//! Champs d'une fiche et provenance de leurs valeurs.

use crate::identifiants::SourceId;
use crate::temps::Horodatage;

/// Champ interprété d'une fiche.
///
/// Toute autre propriété d'une vCard est conservée telle quelle pour la
/// source qui l'a fournie, sans être interprétée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Champ {
    /// Nom affiché (`FN`).
    NomAffiche,
    /// Nom structuré : nom, prénom, autres prénoms, préfixe, suffixe (`N`).
    NomStructure,
    /// Organisation (`ORG`).
    Organisation,
    /// Fonction (`TITLE`).
    Fonction,
    /// Note libre (`NOTE`).
    Note,
    /// Date de naissance (`BDAY`).
    Anniversaire,
    /// Numéros de téléphone (`TEL`).
    Telephone,
    /// Adresses électroniques (`EMAIL`).
    Email,
    /// Adresses postales (`ADR`).
    Adresse,
    /// Adresses web (`URL`).
    Url,
    /// Relations vers d'autres fiches (`RELATED`, ADR-0011).
    Relation,
    /// Groupes, par leur chemin (`CATEGORIES`, ADR-0012).
    Groupes,
    /// Dates autres que la naissance : mariage, fête, dates personnalisées
    /// (`ANNIVERSARY`, `X-ABDATE`, ADR-0011).
    Dates,
}

impl Champ {
    /// Tous les champs interprétés, dans l'ordre d'écriture.
    pub const TOUS: [Self; 13] = [
        Self::NomAffiche,
        Self::NomStructure,
        Self::Organisation,
        Self::Fonction,
        Self::Note,
        Self::Anniversaire,
        Self::Telephone,
        Self::Email,
        Self::Adresse,
        Self::Url,
        Self::Relation,
        Self::Groupes,
        Self::Dates,
    ];

    /// Vrai si le champ peut porter plusieurs valeurs.
    #[must_use]
    pub const fn est_multivalue(self) -> bool {
        matches!(
            self,
            Self::Telephone
                | Self::Email
                | Self::Adresse
                | Self::Url
                | Self::Relation
                | Self::Groupes
                | Self::Dates
        )
    }

    /// Nom de la propriété vCard qui porte ce champ.
    #[must_use]
    pub const fn propriete_vcard(self) -> &'static str {
        match self {
            Self::NomAffiche => "FN",
            Self::NomStructure => "N",
            Self::Organisation => "ORG",
            Self::Fonction => "TITLE",
            Self::Note => "NOTE",
            Self::Anniversaire => "BDAY",
            Self::Telephone => "TEL",
            Self::Email => "EMAIL",
            Self::Adresse => "ADR",
            Self::Url => "URL",
            Self::Relation => "RELATED",
            Self::Groupes => "CATEGORIES",
            Self::Dates => "ANNIVERSARY",
        }
    }

    /// Champ porté par une propriété vCard, s'il est interprété.
    ///
    /// Le nom de propriété se compare sans tenir compte de la casse.
    #[must_use]
    pub fn depuis_propriete_vcard(nom: &str) -> Option<Self> {
        Self::TOUS
            .into_iter()
            .find(|champ| champ.propriete_vcard().eq_ignore_ascii_case(nom))
    }
}

/// D'où vient une valeur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origine {
    /// Saisie ou modifiée dans l'application.
    Referentiel,
    /// Reçue d'une source.
    Source(SourceId),
}

/// Provenance d'une valeur : son origine et la date de sa dernière
/// modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// [impl->req~provenance-par-champ~1]
pub struct Provenance {
    /// Origine de la valeur.
    pub origine: Origine,
    /// Date de la dernière modification de la valeur.
    pub date: Horodatage,
}

impl Provenance {
    /// Provenance d'une valeur modifiée dans l'application.
    #[must_use]
    pub const fn referentiel(date: Horodatage) -> Self {
        Self {
            origine: Origine::Referentiel,
            date,
        }
    }

    /// Provenance d'une valeur reçue d'une source.
    #[must_use]
    pub const fn source(source: SourceId, date: Horodatage) -> Self {
        Self {
            origine: Origine::Source(source),
            date,
        }
    }
}

/// Valeur d'un champ, avec ses types et sa provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Valeur {
    /// Valeur, sous sa forme textuelle décodée. Les champs structurés (`N`,
    /// `ADR`) gardent leurs composantes séparées par `;`, échappées comme en
    /// vCard.
    pub texte: String,
    /// Types de la valeur (`home`, `work`, `cell`…), en minuscules, triés et
    /// sans doublon.
    pub types: Vec<String>,
    /// Provenance de la valeur.
    pub provenance: Provenance,
}

impl Valeur {
    /// Valeur sans type.
    #[must_use]
    pub fn new(texte: impl Into<String>, provenance: Provenance) -> Self {
        Self {
            texte: texte.into(),
            types: Vec::new(),
            provenance,
        }
    }

    /// Même valeur, avec les types donnés, ramenés en minuscules, triés et
    /// dédoublonnés.
    #[must_use]
    pub fn avec_types<I, S>(mut self, types: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut types: Vec<String> = types
            .into_iter()
            .map(|t| t.as_ref().to_ascii_lowercase())
            .collect();
        types.sort();
        types.dedup();
        self.types = types;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_propriete_vcard_se_reconnait_sans_tenir_compte_de_la_casse() {
        assert_eq!(Champ::depuis_propriete_vcard("tel"), Some(Champ::Telephone));
    }

    #[test]
    fn une_propriete_non_interpretee_n_est_pas_un_champ() {
        assert_eq!(Champ::depuis_propriete_vcard("X-PROPRIETAIRE"), None);
    }

    #[test]
    fn chaque_champ_se_retrouve_par_sa_propriete_vcard() {
        let retrouves: Vec<_> = Champ::TOUS
            .into_iter()
            .filter_map(|c| Champ::depuis_propriete_vcard(c.propriete_vcard()))
            .collect();

        assert_eq!(retrouves, Champ::TOUS);
    }

    #[test]
    fn les_types_sont_ramenes_en_minuscules_tries_et_dedoublonnes() {
        let valeur = Valeur::new("x", Provenance::referentiel(Horodatage::depuis_millis(0)))
            .avec_types(["WORK", "cell", "work"]);

        assert_eq!(valeur.types, ["cell", "work"]);
    }
}

//! Dates d'une fiche : naissance, anniversaire de mariage, fête du prénom,
//! dates personnalisées, chacune avec une année facultative ; fête proposée
//! d'après le prénom ; dates à venir.
//!
//! La naissance est le champ [`Champ::Anniversaire`] (`BDAY`) ; les autres
//! dates sont les valeurs du champ [`Champ::Dates`], `type;date`. Une date
//! s'écrit `AAAA-MM-JJ`, ou `--MM-JJ` sans année.
// [impl->req~types-de-dates~1]

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::champ::{Champ, Provenance, Valeur};
use crate::groupes::Filtre;
use crate::identifiants::FicheId;
use crate::normalisation::composantes;
use crate::referentiel::Referentiel;
use crate::temps::Horodatage;

/// Année que la convention Apple écrit pour une date sans année.
pub const ANNEE_OMISE: i32 = 1604;

/// Date de calendrier, année facultative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// Année, si elle est connue.
    pub annee: Option<i32>,
    /// Mois, de 1 à 12.
    pub mois: u8,
    /// Jour, de 1 à 31.
    pub jour: u8,
}

impl Date {
    /// Date lue d'un texte : `1980-05-17`, `19800517`, `--05-17`, `--0517`,
    /// `17/05/1980`, `17/05` ; une heure éventuelle est ignorée, et l'année
    /// 1604 de la convention Apple vaut absence d'année. Aucune pour un
    /// texte qui n'est pas une date valide.
    #[must_use]
    pub fn lire(texte: &str) -> Option<Self> {
        let texte = texte.trim();
        let jour_seul = texte.split('T').next().unwrap_or_default();
        let (annee, mois, jour) = if let Some(reste) = jour_seul.strip_prefix("--") {
            let chiffres: String = reste.chars().filter(char::is_ascii_digit).collect();
            if chiffres.len() != 4 {
                return None;
            }
            (
                None,
                chiffres.get(..2)?.parse().ok()?,
                chiffres.get(2..)?.parse().ok()?,
            )
        } else if jour_seul.contains('/') {
            let parties: Vec<&str> = jour_seul.split('/').map(str::trim).collect();
            match parties.as_slice() {
                [j, m] => (None, m.parse().ok()?, j.parse().ok()?),
                [j, m, a] => (Some(a.parse().ok()?), m.parse().ok()?, j.parse().ok()?),
                _ => return None,
            }
        } else {
            let chiffres: String = jour_seul.chars().filter(char::is_ascii_digit).collect();
            if chiffres.len() != 8 {
                return None;
            }
            (
                Some(chiffres.get(..4)?.parse().ok()?),
                chiffres.get(4..6)?.parse().ok()?,
                chiffres.get(6..)?.parse().ok()?,
            )
        };
        let annee = annee.filter(|a| *a != ANNEE_OMISE);
        let valide = (1..=12).contains(&mois) && jour >= 1 && jour <= jours_du_mois(annee, mois);
        valide.then_some(Self { annee, mois, jour })
    }

    /// Forme enregistrée : `AAAA-MM-JJ`, ou `--MM-JJ` sans année.
    #[must_use]
    pub fn texte(&self) -> String {
        match self.annee {
            Some(annee) => format!("{annee:04}-{:02}-{:02}", self.mois, self.jour),
            None => format!("--{:02}-{:02}", self.mois, self.jour),
        }
    }

    /// Forme vCard 4.0 (RFC 6350, format de base) : `AAAAMMJJ` ou `--MMJJ`.
    #[must_use]
    pub fn texte_vcard4(&self) -> String {
        match self.annee {
            Some(annee) => format!("{annee:04}{:02}{:02}", self.mois, self.jour),
            None => format!("--{:02}{:02}", self.mois, self.jour),
        }
    }

    /// Forme vCard 3.0 : `AAAA-MM-JJ`, l'année 1604 tenant lieu d'année
    /// absente (convention Apple), et vrai dans ce cas.
    #[must_use]
    pub fn texte_vcard3(&self) -> (String, bool) {
        let annee = self.annee.unwrap_or(ANNEE_OMISE);
        (
            format!("{annee:04}-{:02}-{:02}", self.mois, self.jour),
            self.annee.is_none(),
        )
    }
}

/// Vrai pour une année bissextile.
const fn bissextile(annee: i32) -> bool {
    (annee % 4 == 0 && annee % 100 != 0) || annee % 400 == 0
}

/// Nombre de jours d'un mois ; février en a 29 si l'année est inconnue.
fn jours_du_mois(annee: Option<i32>, mois: u8) -> u8 {
    match mois {
        4 | 6 | 9 | 11 => 30,
        2 => {
            if annee.is_none_or(bissextile) {
                29
            } else {
                28
            }
        }
        _ => 31,
    }
}

/// Type d'une date de fiche.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeDate {
    /// Naissance (`BDAY`).
    Naissance,
    /// Anniversaire de mariage.
    Mariage,
    /// Fête du prénom.
    Fete,
    /// Date personnalisée, désignée par son libellé.
    Personnalisee(String),
}

impl TypeDate {
    /// Code enregistré : un mot fixe, ou le libellé d'une date
    /// personnalisée.
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Naissance => "naissance",
            Self::Mariage => "mariage",
            Self::Fete => "fete",
            Self::Personnalisee(libelle) => libelle,
        }
    }

    /// Libellé affiché.
    #[must_use]
    pub fn libelle(&self) -> &str {
        match self {
            Self::Naissance => "Anniversaire",
            Self::Mariage => "Anniversaire de mariage",
            Self::Fete => "Fête du prénom",
            Self::Personnalisee(libelle) => libelle,
        }
    }

    /// Type désigné par un code ou un libellé, sans tenir compte de la
    /// casse ; sinon une date personnalisée.
    #[must_use]
    pub fn depuis_texte(texte: &str) -> Self {
        let texte = texte.trim();
        [Self::Naissance, Self::Mariage, Self::Fete]
            .into_iter()
            .find(|t| {
                t.code().eq_ignore_ascii_case(texte)
                    || t.libelle().to_lowercase() == texte.to_lowercase()
            })
            .unwrap_or_else(|| Self::Personnalisee(texte.to_owned()))
    }
}

/// Date d'une fiche : son type et sa date.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateDeFiche {
    /// Type de la date.
    pub type_date: TypeDate,
    /// Date.
    pub date: Date,
}

impl DateDeFiche {
    /// Texte d'une valeur du champ `Dates` : code du type, `;` échappé, puis
    /// la date.
    #[must_use]
    pub fn texte(&self) -> String {
        let code = self
            .type_date
            .code()
            .replace('\\', "\\\\")
            .replace(';', "\\;");
        format!("{code};{}", self.date.texte())
    }

    /// Date lue d'une valeur du champ `Dates`.
    #[must_use]
    pub fn depuis_texte(texte: &str) -> Option<Self> {
        let (code, date) = texte.rsplit_once(';')?;
        let mut type_texte = String::with_capacity(code.len());
        let mut caracteres = code.chars();
        while let Some(c) = caracteres.next() {
            if c == '\\' {
                type_texte.extend(caracteres.next());
            } else {
                type_texte.push(c);
            }
        }
        let type_date = TypeDate::depuis_texte(&type_texte);
        if type_date == TypeDate::Naissance || type_texte.trim().is_empty() {
            return None;
        }
        Some(Self {
            type_date,
            date: Date::lire(date)?,
        })
    }
}

/// Erreur d'une opération sur les dates.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurDate {
    /// Le texte n'est pas une date valide.
    #[error("date invalide : « {0} » (AAAA-MM-JJ, JJ/MM/AAAA ou JJ/MM)")]
    Invalide(String),
    /// Le libellé d'une date personnalisée est vide.
    #[error("le libellé d'une date ne peut être vide")]
    LibelleVide,
    /// Aucune fiche de cet identifiant.
    #[error("fiche inconnue : {0}")]
    FicheInconnue(FicheId),
}

/// Date à venir d'une fiche.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AVenir {
    /// Fiche concernée.
    pub fiche: FicheId,
    /// Date de la fiche.
    pub date: DateDeFiche,
    /// Nombre de jours d'ici la prochaine occurrence ; 0 : aujourd'hui.
    pub dans_jours: u32,
    /// Âge ou nombre d'années atteint à cette occurrence, si l'année est
    /// connue.
    pub annees: Option<u32>,
}

/// Jour civil (UTC) d'un horodatage : année, mois, jour.
#[must_use]
pub fn jour_civil(horodatage: Horodatage) -> (i32, u8, u8) {
    civil(horodatage.millis().div_euclid(86_400_000))
}

/// Jour civil d'un nombre de jours depuis le 1er janvier 1970
/// (algorithme de H. Hinnant).
fn civil(jours: i64) -> (i32, u8, u8) {
    let z = jours + 719_468;
    let ere = z.div_euclid(146_097);
    let jour_de_l_ere = z.rem_euclid(146_097);
    let annee_de_l_ere = (jour_de_l_ere - jour_de_l_ere / 1460 + jour_de_l_ere / 36_524
        - jour_de_l_ere / 146_096)
        / 365;
    let jour_de_l_annee =
        jour_de_l_ere - (365 * annee_de_l_ere + annee_de_l_ere / 4 - annee_de_l_ere / 100);
    let mp = (5 * jour_de_l_annee + 2) / 153;
    let jour = jour_de_l_annee - (153 * mp + 2) / 5 + 1;
    let mois = if mp < 10 { mp + 3 } else { mp - 9 };
    let annee = annee_de_l_ere + ere * 400 + i64::from(mois <= 2);
    (
        i32::try_from(annee).unwrap_or(i32::MAX),
        u8::try_from(mois).unwrap_or(1),
        u8::try_from(jour).unwrap_or(1),
    )
}

/// Nombre de jours depuis le 1er janvier 1970 d'un jour civil.
fn jours_depuis_epoque(annee: i32, mois: u8, jour: u8) -> i64 {
    let annee = i64::from(annee) - i64::from(mois <= 2);
    let ere = annee.div_euclid(400);
    let annee_de_l_ere = annee.rem_euclid(400);
    let m = i64::from(mois);
    let jour_de_l_annee = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(jour) - 1;
    let jour_de_l_ere =
        annee_de_l_ere * 365 + annee_de_l_ere / 4 - annee_de_l_ere / 100 + jour_de_l_annee;
    ere * 146_097 + jour_de_l_ere - 719_468
}

/// Prochaine occurrence d'une date annuelle à partir d'un jour donné, ce
/// jour compris : son année et le nombre de jours d'ici là. Un 29 février
/// tombe le 28 les années non bissextiles.
fn prochaine(date: Date, aujourdhui: (i32, u8, u8)) -> (i32, u32) {
    let (annee, mois, jour) = aujourdhui;
    let occurrence = |a: i32| {
        let j = if date.mois == 2 && date.jour == 29 && !bissextile(a) {
            28
        } else {
            date.jour
        };
        jours_depuis_epoque(a, date.mois, j)
    };
    let depart = jours_depuis_epoque(annee, mois, jour);
    let (a, jours) = if occurrence(annee) >= depart {
        (annee, occurrence(annee))
    } else {
        (annee + 1, occurrence(annee + 1))
    };
    (a, u32::try_from(jours - depart).unwrap_or(0))
}

/// Calendrier des fêtes : prénom plié (minuscules, sans accents) → jours
/// (mois, jour), dans l'ordre du calendrier (ADR-0013).
fn calendrier() -> &'static BTreeMap<String, Vec<(u8, u8)>> {
    static CALENDRIER: OnceLock<BTreeMap<String, Vec<(u8, u8)>>> = OnceLock::new();
    CALENDRIER.get_or_init(|| {
        let mut calendrier: BTreeMap<String, Vec<(u8, u8)>> = BTreeMap::new();
        for ligne in include_str!("../data/fetes-fr.txt").lines() {
            let Some((jour, prenom)) = ligne.split_once('\t') else {
                continue;
            };
            let Some(date) = Date::lire(&format!("--{jour}")) else {
                continue;
            };
            calendrier
                .entry(plie(prenom))
                .or_default()
                .push((date.mois, date.jour));
        }
        calendrier
    })
}

/// Prénom en minuscules, sans accents.
fn plie(texte: &str) -> String {
    texte
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'ÿ' => 'y',
            autre => autre,
        })
        .collect()
}

/// Jours de fête d'un prénom au calendrier, tel quel ou par son premier
/// élément (« Jean » pour « Jean-Pierre »).
// [impl->req~fete-du-prenom-proposee~1]
#[must_use]
pub fn fetes_du_prenom(prenom: &str) -> Vec<Date> {
    let calendrier = calendrier();
    let entier = plie(prenom);
    let premier = entier
        .split(|c: char| c == '-' || c.is_whitespace())
        .next()
        .unwrap_or_default()
        .to_owned();
    calendrier
        .get(&entier)
        .or_else(|| calendrier.get(&premier))
        .map(|jours| {
            jours
                .iter()
                .map(|(mois, jour)| Date {
                    annee: None,
                    mois: *mois,
                    jour: *jour,
                })
                .collect()
        })
        .unwrap_or_default()
}

impl Referentiel {
    /// Dates d'une fiche : la naissance, puis les autres dans leur ordre.
    #[must_use]
    pub fn dates(&self, fiche: FicheId) -> Vec<DateDeFiche> {
        let Some(f) = self.fiches.get(&fiche) else {
            return Vec::new();
        };
        let naissance = f
            .valeurs(Champ::Anniversaire)
            .iter()
            .filter_map(|v| Date::lire(&v.texte))
            .map(|date| DateDeFiche {
                type_date: TypeDate::Naissance,
                date,
            });
        let autres = f
            .valeurs(Champ::Dates)
            .iter()
            .filter_map(|v| DateDeFiche::depuis_texte(&v.texte));
        naissance.chain(autres).collect()
    }

    /// Ajoute une date à une fiche ; une naissance remplace la précédente.
    ///
    /// # Errors
    ///
    /// [`ErreurDate`] pour une fiche inconnue, une date invalide ou un
    /// libellé vide ; rien n'est alors modifié.
    // [impl->req~types-de-dates~1]
    pub fn ajouter_date(
        &mut self,
        fiche: FicheId,
        type_date: TypeDate,
        texte: &str,
        horodatage: Horodatage,
    ) -> Result<(), ErreurDate> {
        let date = Date::lire(texte).ok_or_else(|| ErreurDate::Invalide(texte.to_owned()))?;
        if type_date.code().trim().is_empty() {
            return Err(ErreurDate::LibelleVide);
        }
        let porteuse = self
            .fiches
            .get_mut(&fiche)
            .ok_or(ErreurDate::FicheInconnue(fiche))?;
        let provenance = Provenance::referentiel(horodatage);
        if type_date == TypeDate::Naissance {
            porteuse.definir_unique(Champ::Anniversaire, Valeur::new(date.texte(), provenance));
            return Ok(());
        }
        let nouvelle = DateDeFiche { type_date, date };
        let mut valeurs = porteuse.valeurs(Champ::Dates).to_vec();
        if !valeurs
            .iter()
            .any(|v| DateDeFiche::depuis_texte(&v.texte).as_ref() == Some(&nouvelle))
        {
            valeurs.push(Valeur::new(nouvelle.texte(), provenance));
            porteuse.definir_multivalue(Champ::Dates, valeurs);
        }
        Ok(())
    }

    /// Retire une date d'une fiche.
    ///
    /// # Errors
    ///
    /// [`ErreurDate::FicheInconnue`].
    pub fn retirer_date(
        &mut self,
        fiche: FicheId,
        retiree: &DateDeFiche,
    ) -> Result<(), ErreurDate> {
        let porteuse = self
            .fiches
            .get_mut(&fiche)
            .ok_or(ErreurDate::FicheInconnue(fiche))?;
        if retiree.type_date == TypeDate::Naissance {
            let portee = porteuse
                .valeurs(Champ::Anniversaire)
                .iter()
                .any(|v| Date::lire(&v.texte) == Some(retiree.date));
            if portee {
                porteuse.vider(Champ::Anniversaire);
            }
            return Ok(());
        }
        let restantes: Vec<Valeur> = porteuse
            .valeurs(Champ::Dates)
            .iter()
            .filter(|v| DateDeFiche::depuis_texte(&v.texte).as_ref() != Some(retiree))
            .cloned()
            .collect();
        porteuse.definir_multivalue(Champ::Dates, restantes);
        Ok(())
    }

    /// Jours de fête proposés pour une fiche qui n'a pas de fête du prénom,
    /// d'après son prénom (nom structuré, sinon premier mot du nom
    /// affiché) ; aucun si elle en a une, ou si le prénom n'est pas au
    /// calendrier.
    // [impl->req~fete-du-prenom-proposee~1]
    #[must_use]
    pub fn fetes_proposees(&self, fiche: FicheId) -> Vec<Date> {
        let Some(f) = self.fiches.get(&fiche) else {
            return Vec::new();
        };
        if self
            .dates(fiche)
            .iter()
            .any(|d| d.type_date == TypeDate::Fete)
        {
            return Vec::new();
        }
        let prenom = f
            .valeurs(Champ::NomStructure)
            .first()
            .and_then(|v| composantes(&v.texte).get(1).cloned())
            .filter(|p| !p.trim().is_empty())
            .or_else(|| {
                f.valeurs(Champ::NomAffiche)
                    .first()
                    .and_then(|v| v.texte.split_whitespace().next().map(str::to_owned))
            })
            .unwrap_or_default();
        fetes_du_prenom(&prenom)
    }

    /// Dates à venir des fiches qui passent le filtre, de la plus proche à
    /// la plus lointaine, à partir du jour de l'horodatage donné.
    // [impl->req~vue-a-venir~1]
    #[must_use]
    pub fn a_venir(&self, maintenant: Horodatage, filtre: &Filtre) -> Vec<AVenir> {
        let aujourdhui = jour_civil(maintenant);
        let mut a_venir: Vec<AVenir> = self
            .filtrer(filtre)
            .into_iter()
            .flat_map(|fiche| {
                self.dates(fiche).into_iter().map(move |date| {
                    let (annee, dans_jours) = prochaine(date.date, aujourdhui);
                    let annees = date
                        .date
                        .annee
                        .and_then(|naissance| u32::try_from(annee - naissance).ok());
                    AVenir {
                        fiche,
                        date,
                        dans_jours,
                        annees,
                    }
                })
            })
            .collect();
        a_venir.sort_by(|a, b| {
            (a.dans_jours, a.date.date.mois, a.date.date.jour, a.fiche).cmp(&(
                b.dans_jours,
                b.date.date.mois,
                b.date.date.jour,
                b.fiche,
            ))
        });
        a_venir
    }
}

#[cfg(test)]
mod tests;

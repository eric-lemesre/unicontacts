//! Dates dans les vCard (ADR-0011) : `BDAY` pour la naissance ;
//! `ANNIVERSARY` (4.0) ou `X-ABDATE` étiqueté `_$!<Anniversary>!$_` (3.0)
//! pour le mariage ; `X-ABDATE` étiqueté par son libellé pour la fête et les
//! dates personnalisées. Sans année : `--MMJJ` en 4.0, `1604-MM-JJ` avec
//! `X-APPLE-OMIT-YEAR=1604` en 3.0.
// [impl->req~synchronisation-dates~1]

use super::{Carte, Parametre, Propriete, Version};
use crate::dates::{ANNEE_OMISE, Date, DateDeFiche, TypeDate};

/// Libellé Apple de l'anniversaire de mariage.
const LIBELLE_MARIAGE: &str = "_$!<Anniversary>!$_";

/// Date lue d'une propriété du champ `Dates`.
pub(super) struct Lue {
    /// Texte de la date, au format du champ `Dates`.
    pub(super) texte: String,
    /// Propriété compagne, qui porte le libellé (convention Apple).
    pub(super) compagnon: Option<usize>,
}

/// Date portée par la propriété d'indice donné, si c'en est une du champ
/// `Dates` que l'on sait lire ; sinon la propriété reste conservée.
pub(super) fn lire(carte: &Carte, indice: usize) -> Option<Lue> {
    let propriete = carte.proprietes.get(indice)?;
    let (type_date, compagnon) = match propriete.nom.as_str() {
        "ANNIVERSARY" | "X-ANNIVERSARY" => (TypeDate::Mariage, None),
        "X-ABDATE" => {
            let groupe = propriete.groupe.as_deref()?;
            let (i, libelle) = carte.proprietes.iter().enumerate().find_map(|(i, p)| {
                (p.nom == "X-ABLABEL"
                    && p.groupe
                        .as_deref()
                        .is_some_and(|g| g.eq_ignore_ascii_case(groupe)))
                .then(|| {
                    (
                        i,
                        super::ecriture::decoder(crate::champ::Champ::Note, &p.valeur),
                    )
                })
            })?;
            let type_date = type_apple(&libelle)?;
            if type_date == TypeDate::Naissance {
                return None;
            }
            (type_date, Some(i))
        }
        _ => return None,
    };
    let date = Date::lire(&propriete.valeur)?;
    Some(Lue {
        texte: DateDeFiche { type_date, date }.texte(),
        compagnon,
    })
}

/// Texte enregistré d'une naissance lue de `BDAY` : la convention Apple de
/// l'année omise devient une date sans année ; un texte qui n'est pas une
/// date reste tel quel.
pub(super) fn naissance_lue(valeur: &str) -> String {
    Date::lire(valeur).map_or_else(|| valeur.to_owned(), |d| d.texte())
}

/// Propriété `BDAY` d'une naissance, dans la forme de la version.
pub(super) fn naissance_produite(texte: &str, version: Version) -> Propriete {
    let Some(date) = Date::lire(texte) else {
        return Propriete::nouvelle("BDAY", Vec::new(), texte.to_owned());
    };
    let (valeur, parametres) = forme(date, version);
    Propriete::nouvelle("BDAY", parametres, valeur)
}

/// Propriétés qui écrivent une date du champ `Dates` dans une carte de
/// cette version. `groupe` est le dernier numéro de groupe `itemN` employé.
pub(super) fn produire(texte: &str, version: Version, groupe: &mut u32) -> Vec<Propriete> {
    let Some(date) = DateDeFiche::depuis_texte(texte) else {
        return Vec::new();
    };
    let (valeur, parametres) = forme(date.date, version);
    if date.type_date == TypeDate::Mariage && version == Version::V4 {
        return vec![Propriete::nouvelle("ANNIVERSARY", parametres, valeur)];
    }
    *groupe += 1;
    let item = format!("item{groupe}");
    let libelle = if date.type_date == TypeDate::Mariage {
        LIBELLE_MARIAGE.to_owned()
    } else {
        super::ecriture::encoder(crate::champ::Champ::Note, date.type_date.libelle())
    };
    let mut valeur_date = Propriete::nouvelle("X-ABDATE", parametres, valeur);
    valeur_date.groupe = Some(item.clone());
    let mut etiquette = Propriete::nouvelle("X-ABLABEL", Vec::new(), libelle);
    etiquette.groupe = Some(item);
    vec![valeur_date, etiquette]
}

/// Valeur et paramètres d'une date selon la version.
fn forme(date: Date, version: Version) -> (String, Vec<Parametre>) {
    match version {
        Version::V4 => (date.texte_vcard4(), Vec::new()),
        Version::V3 => {
            let (valeur, omise) = date.texte_vcard3();
            let parametres = if omise {
                vec![Parametre {
                    nom: "X-APPLE-OMIT-YEAR".to_owned(),
                    valeurs: vec![ANNEE_OMISE.to_string()],
                }]
            } else {
                Vec::new()
            };
            (valeur, parametres)
        }
    }
}

/// Type d'une date d'après son libellé Apple ; aucun pour un libellé vide.
fn type_apple(libelle: &str) -> Option<TypeDate> {
    let libelle = libelle.trim();
    if libelle.is_empty() {
        return None;
    }
    if libelle.eq_ignore_ascii_case(LIBELLE_MARIAGE) {
        return Some(TypeDate::Mariage);
    }
    let interne = libelle
        .strip_prefix("_$!<")
        .and_then(|l| l.strip_suffix(">!$_"))
        .unwrap_or(libelle);
    Some(TypeDate::depuis_texte(interne))
}

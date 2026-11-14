//! Détection des doublons à l'import (D-2026-11-10).
//!
//! Une fiche importée est le doublon d'une fiche existante quand elles
//! partagent un téléphone ou un e-mail (clés normalisées), ou quand leurs
//! noms sont proches : égaux une fois retirés casse, accents, ponctuation et
//! ordre des mots.
//!
//! - **Certain** : un téléphone ou un e-mail commun, et des noms proches ou
//!   l'un des deux absent.
//! - **Probable** : un téléphone ou un e-mail commun avec des noms
//!   différents (une ligne partagée, un e-mail de service), ou des noms
//!   proches sans autre donnée commune (deux homonymes).

use std::collections::BTreeSet;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::champ::Champ;
use crate::fiche::Fiche;
use crate::identifiants::FicheId;
use crate::normalisation::{cle, composantes};

/// Degré de certitude d'un doublon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Certitude {
    /// Les deux fiches désignent sûrement la même personne.
    Certain,
    /// Les deux fiches désignent peut-être la même personne.
    Probable,
}

/// Doublon détecté : la fiche existante et la certitude.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Doublon {
    /// Fiche existante.
    pub fiche: FicheId,
    /// Certitude du rapprochement.
    pub certitude: Certitude,
}

/// Cherche, parmi les fiches existantes, le doublon de la fiche importée :
/// le premier certain, à défaut le premier probable.
// [impl->req~detection-doublons~1]
#[must_use]
pub fn detecter<'a>(
    importee: &Fiche,
    existantes: impl IntoIterator<Item = &'a Fiche>,
) -> Option<Doublon> {
    let mut probable = None;
    for existante in existantes {
        match certitude(importee, existante) {
            Some(Certitude::Certain) => {
                return Some(Doublon {
                    fiche: existante.id(),
                    certitude: Certitude::Certain,
                });
            }
            Some(Certitude::Probable) if probable.is_none() => {
                probable = Some(Doublon {
                    fiche: existante.id(),
                    certitude: Certitude::Probable,
                });
            }
            _ => {}
        }
    }
    probable
}

/// Certitude du rapprochement de deux fiches, s'il y en a un.
#[must_use]
pub fn certitude(a: &Fiche, b: &Fiche) -> Option<Certitude> {
    let coordonnee_commune = [Champ::Telephone, Champ::Email]
        .into_iter()
        .any(|champ| !cles(a, champ).is_disjoint(&cles(b, champ)));
    let noms = (nom_plie(a), nom_plie(b));
    let noms_proches = matches!(&noms, (Some(x), Some(y)) if x == y);
    let un_nom_absent = noms.0.is_none() || noms.1.is_none();
    match (coordonnee_commune, noms_proches || un_nom_absent) {
        (true, true) => Some(Certitude::Certain),
        (true, false) => Some(Certitude::Probable),
        (false, _) if noms_proches => Some(Certitude::Probable),
        (false, _) => None,
    }
}

fn cles(fiche: &Fiche, champ: Champ) -> BTreeSet<String> {
    fiche
        .valeurs(champ)
        .iter()
        .map(|v| cle(champ, &v.texte))
        .collect()
}

/// Nom plié pour comparaison : nom affiché, à défaut prénom et nom ; sans
/// casse, accents ni ponctuation, mots triés.
fn nom_plie(fiche: &Fiche) -> Option<String> {
    let nom = fiche
        .valeurs(Champ::NomAffiche)
        .first()
        .map(|v| v.texte.clone())
        .or_else(|| {
            fiche
                .valeurs(Champ::NomStructure)
                .first()
                .map(|v| composantes(&v.texte).join(" "))
        })?;
    let plie: String = nom
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    let mut mots: Vec<&str> = plie.split_whitespace().collect();
    mots.sort_unstable();
    (!mots.is_empty()).then(|| mots.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fiche::ErreurFiche;
    use crate::temps::Horodatage;

    fn fiche(valeurs: &[(Champ, &str)]) -> Result<Fiche, ErreurFiche> {
        let mut fiche = Fiche::nouvelle();
        for (champ, texte) in valeurs {
            fiche.modifier(*champ, &[texte], Horodatage::depuis_millis(1))?;
        }
        Ok(fiche)
    }

    // [utest->req~detection-doublons~1]
    #[test]
    fn un_contact_importe_de_meme_email_est_detecte() -> Result<(), ErreurFiche> {
        let existante = fiche(&[
            (Champ::NomAffiche, "Camille Exemple"),
            (Champ::Email, "camille@example.org"),
        ])?;
        let importee = fiche(&[(Champ::Email, "Camille@Example.org")])?;

        let doublon = detecter(&importee, [&existante]);

        assert_eq!(doublon.map(|d| d.fiche), Some(existante.id()));
        Ok(())
    }

    // [utest->req~detection-doublons~1]
    #[test]
    fn deux_noms_qui_ne_different_que_d_un_accent_sans_autre_donnee_commune_sont_un_doublon_probable()
    -> Result<(), ErreurFiche> {
        let existante = fiche(&[(Champ::NomAffiche, "Hélène Fictif")])?;
        let importee = fiche(&[(Champ::NomAffiche, "Helene Fictif")])?;

        let doublon = detecter(&importee, [&existante]);

        assert_eq!(doublon.map(|d| d.certitude), Some(Certitude::Probable));
        Ok(())
    }

    #[test]
    fn un_meme_telephone_ecrit_autrement_avec_le_meme_nom_est_certain() -> Result<(), ErreurFiche> {
        let existante = fiche(&[
            (Champ::NomAffiche, "Camille Exemple"),
            (Champ::Telephone, "06 39 98 00 00"),
        ])?;
        let importee = fiche(&[
            (Champ::NomAffiche, "EXEMPLE Camille"),
            (Champ::Telephone, "+33639980000"),
        ])?;

        assert_eq!(certitude(&importee, &existante), Some(Certitude::Certain));
        Ok(())
    }

    #[test]
    fn un_meme_telephone_avec_des_noms_differents_est_probable() -> Result<(), ErreurFiche> {
        let existante = fiche(&[
            (Champ::NomAffiche, "Camille Exemple"),
            (Champ::Telephone, "0199000001"),
        ])?;
        let importee = fiche(&[
            (Champ::NomAffiche, "Accueil Société"),
            (Champ::Telephone, "0199000001"),
        ])?;

        assert_eq!(certitude(&importee, &existante), Some(Certitude::Probable));
        Ok(())
    }

    #[test]
    fn deux_fiches_sans_rien_de_commun_ne_sont_pas_des_doublons() -> Result<(), ErreurFiche> {
        let existante = fiche(&[
            (Champ::NomAffiche, "Camille Exemple"),
            (Champ::Email, "a@example.org"),
        ])?;
        let importee = fiche(&[
            (Champ::NomAffiche, "Dominique Fictif"),
            (Champ::Email, "b@example.org"),
        ])?;

        assert_eq!(certitude(&importee, &existante), None);
        Ok(())
    }

    #[test]
    fn le_nom_structure_sert_quand_le_nom_affiche_manque() -> Result<(), ErreurFiche> {
        let existante = fiche(&[(Champ::NomAffiche, "Camille Exemple")])?;
        let importee = fiche(&[(Champ::NomStructure, "Exemple;Camille;;;")])?;

        assert_eq!(certitude(&importee, &existante), Some(Certitude::Probable));
        Ok(())
    }

    #[test]
    fn un_doublon_certain_l_emporte_sur_un_probable_rencontre_avant() -> Result<(), ErreurFiche> {
        let homonyme = fiche(&[(Champ::NomAffiche, "Camille Exemple")])?;
        let meme_personne = fiche(&[
            (Champ::NomAffiche, "Camille Exemple"),
            (Champ::Email, "c@example.org"),
        ])?;
        let importee = fiche(&[
            (Champ::NomAffiche, "Camille Exemple"),
            (Champ::Email, "c@example.org"),
        ])?;

        let doublon = detecter(&importee, [&homonyme, &meme_personne]);

        assert_eq!(
            doublon,
            Some(Doublon {
                fiche: meme_personne.id(),
                certitude: Certitude::Certain
            })
        );
        Ok(())
    }
}

//! Fusion à trois voies d'un champ (ADR-0005).
//!
//! Chaque champ se compare selon trois valeurs : celle du référentiel, celle
//! de la source et celle du dernier état synchronisé entre eux (la base).
//! Les valeurs se comparent par leur clé normalisée, un champ multivalué
//! comme un ensemble.
//!
//! - Aucun côté n'a changé, ou les deux ont fait le même changement : rien
//!   à faire.
//! - Un seul côté a changé : la modification s'applique à l'autre.
//! - Les deux côtés ont changé différemment : c'est une divergence, sauf
//!   pour un champ multivalué dont les modifications se combinent valeur par
//!   valeur (ajouts et retraits de chaque côté). Elles ne se combinent pas
//!   quand une même valeur de la base a été retirée des deux côtés et que
//!   chacun a ajouté la sienne : c'est une valeur remplacée différemment de
//!   part et d'autre, donc une divergence.
//! - Une divergence se tranche en faveur du gagnant désigné par les
//!   réglages ; sans gagnant, le référentiel garde sa valeur et la base ne
//!   bouge pas, en attendant la décision.

use std::collections::BTreeSet;

use crate::champ::{Champ, Valeur};
use crate::normalisation::cle;
use crate::reglages::Gagnant;

/// Issue de la fusion d'un champ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fusion {
    /// Valeurs du champ dans le référentiel après fusion.
    pub referentiel: Vec<Valeur>,
    /// Vrai si la source doit recevoir les valeurs du référentiel.
    pub ecrire_source: bool,
    /// Nouveau dernier état synchronisé du champ.
    pub base: Vec<String>,
    /// Vrai si le champ diverge sans gagnant : une décision est due.
    pub divergence: bool,
}

/// Fusionne un champ selon ses trois valeurs.
///
/// `source` porte les valeurs reçues de la source, avec leur provenance ;
/// `base` les textes du dernier état synchronisé. Une valeur qui reste dans
/// le référentiel y garde sa provenance.
// [impl->req~fusion-trois-voies~1]
#[must_use]
pub fn fusionner(
    champ: Champ,
    referentiel: &[Valeur],
    source: &[Valeur],
    base: &[String],
    gagnant: Gagnant,
) -> Fusion {
    let cles = |valeurs: &[Valeur]| -> BTreeSet<String> {
        valeurs.iter().map(|v| cle(champ, &v.texte)).collect()
    };
    let (r, s) = (cles(referentiel), cles(source));
    let b: BTreeSet<String> = base.iter().map(|t| cle(champ, t)).collect();

    // [impl->req~modification-unilaterale-appliquee~1]
    if r == s {
        return accord(referentiel);
    }
    if s == b {
        return vers_source(referentiel);
    }
    if r == b {
        return vers_referentiel(champ, referentiel, source);
    }
    if champ.est_multivalue()
        && let Some(valeurs) = combiner(champ, referentiel, source, &r, &s, &b)
    {
        let ecrire_source = cles(&valeurs) != s;
        return Fusion {
            base: textes(&valeurs),
            referentiel: valeurs,
            ecrire_source,
            divergence: false,
        };
    }
    // [impl->req~autorite-de-la-reference~2]
    // [impl->req~divergence-soumise-a-decision~1]
    match gagnant {
        Gagnant::Source => vers_referentiel(champ, referentiel, source),
        Gagnant::Referentiel => vers_source(referentiel),
        Gagnant::Personne => Fusion {
            referentiel: referentiel.to_vec(),
            ecrire_source: false,
            base: base.to_vec(),
            divergence: true,
        },
    }
}

/// Les deux côtés portent les mêmes valeurs.
fn accord(referentiel: &[Valeur]) -> Fusion {
    Fusion {
        referentiel: referentiel.to_vec(),
        ecrire_source: false,
        base: textes(referentiel),
        divergence: false,
    }
}

/// Le référentiel l'emporte : la source reçoit ses valeurs.
fn vers_source(referentiel: &[Valeur]) -> Fusion {
    Fusion {
        referentiel: referentiel.to_vec(),
        ecrire_source: true,
        base: textes(referentiel),
        divergence: false,
    }
}

/// La source l'emporte : le référentiel prend ses valeurs ; celles qu'il
/// avait déjà gardent leur provenance.
fn vers_referentiel(champ: Champ, referentiel: &[Valeur], source: &[Valeur]) -> Fusion {
    let valeurs: Vec<Valeur> = source
        .iter()
        .map(|v| {
            let cle_v = cle(champ, &v.texte);
            referentiel
                .iter()
                .find(|r| cle(champ, &r.texte) == cle_v)
                .unwrap_or(v)
                .clone()
        })
        .collect();
    Fusion {
        base: textes(&valeurs),
        referentiel: valeurs,
        ecrire_source: false,
        divergence: false,
    }
}

/// Combine valeur par valeur les modifications des deux côtés d'un champ
/// multivalué ; rien si une valeur de la base a été remplacée différemment
/// de part et d'autre.
// [impl->req~multivalues-par-valeur-normalisee~1]
fn combiner(
    champ: Champ,
    referentiel: &[Valeur],
    source: &[Valeur],
    r: &BTreeSet<String>,
    s: &BTreeSet<String>,
    b: &BTreeSet<String>,
) -> Option<Vec<Valeur>> {
    let retiree_des_deux_cotes = b.iter().any(|k| !r.contains(k) && !s.contains(k));
    let ajout_propre_au_referentiel = r.iter().any(|k| !b.contains(k) && !s.contains(k));
    let ajout_propre_a_la_source = s.iter().any(|k| !b.contains(k) && !r.contains(k));
    if retiree_des_deux_cotes && ajout_propre_au_referentiel && ajout_propre_a_la_source {
        return None;
    }
    // Une valeur reste si elle est présente d'un côté et n'a pas été retirée
    // de l'autre : ajoutée d'un côté, ou présente des deux.
    let garde = |k: &String| !b.contains(k) || (r.contains(k) && s.contains(k));
    let mut valeurs: Vec<Valeur> = referentiel
        .iter()
        .filter(|v| garde(&cle(champ, &v.texte)))
        .cloned()
        .collect();
    let deja: BTreeSet<String> = valeurs.iter().map(|v| cle(champ, &v.texte)).collect();
    valeurs.extend(
        source
            .iter()
            .filter(|v| {
                let k = cle(champ, &v.texte);
                garde(&k) && !deja.contains(&k)
            })
            .cloned(),
    );
    Some(valeurs)
}

fn textes(valeurs: &[Valeur]) -> Vec<String> {
    valeurs.iter().map(|v| v.texte.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::champ::Provenance;
    use crate::identifiants::SourceId;
    use crate::temps::Horodatage;

    fn du_referentiel(textes: &[&str]) -> Vec<Valeur> {
        let provenance = Provenance::referentiel(Horodatage::depuis_millis(1));
        textes.iter().map(|t| Valeur::new(*t, provenance)).collect()
    }

    fn de_la_source(textes: &[&str]) -> Vec<Valeur> {
        let provenance = Provenance::source(SourceId::nouveau(), Horodatage::depuis_millis(2));
        textes.iter().map(|t| Valeur::new(*t, provenance)).collect()
    }

    fn base(textes: &[&str]) -> Vec<String> {
        textes.iter().map(|t| (*t).to_owned()).collect()
    }

    fn fusion(champ: Champ, r: &[&str], s: &[&str], b: &[&str]) -> Fusion {
        fusionner(
            champ,
            &du_referentiel(r),
            &de_la_source(s),
            &base(b),
            Gagnant::Personne,
        )
    }

    fn textes_du_referentiel(fusion: &Fusion) -> Vec<&str> {
        fusion
            .referentiel
            .iter()
            .map(|v| v.texte.as_str())
            .collect()
    }

    mod trois_voies {
        use super::*;

        // [utest->req~fusion-trois-voies~1]
        #[test]
        fn le_telephone_change_cote_source_est_retenu() {
            let telephone = fusion(
                Champ::Telephone,
                &["0639980000"],
                &["0639980009"],
                &["0639980000"],
            );

            assert_eq!(textes_du_referentiel(&telephone), ["0639980009"]);
        }

        // [utest->req~fusion-trois-voies~1]
        #[test]
        fn l_adresse_changee_cote_referentiel_est_ecrite_vers_la_source() {
            let adresse = fusion(
                Champ::Adresse,
                &[";;2 rue Neuve"],
                &[";;1 rue Vieille"],
                &[";;1 rue Vieille"],
            );

            assert!(adresse.ecrire_source);
        }

        // [utest->req~fusion-trois-voies~1]
        #[test]
        fn deux_champs_changes_chacun_d_un_cote_ne_divergent_pas() {
            let telephone = fusion(
                Champ::Telephone,
                &["0639980000"],
                &["0639980009"],
                &["0639980000"],
            );
            let adresse = fusion(
                Champ::Adresse,
                &[";;2 rue Neuve"],
                &[";;1 rue Vieille"],
                &[";;1 rue Vieille"],
            );

            assert!(!telephone.divergence && !adresse.divergence);
        }

        #[test]
        fn sans_la_base_deux_valeurs_differentes_divergent() {
            let nom = fusion(Champ::NomAffiche, &["Camille"], &["Camille E."], &[]);

            assert!(nom.divergence);
        }
    }

    mod unilaterale {
        use super::*;

        // [utest->req~modification-unilaterale-appliquee~1]
        #[test]
        fn un_email_modifie_sur_la_source_seulement_passe_au_referentiel() {
            let email = fusion(
                Champ::Email,
                &["a@example.org"],
                &["b@example.org"],
                &["a@example.org"],
            );

            assert_eq!(
                (textes_du_referentiel(&email), email.divergence),
                (vec!["b@example.org"], false)
            );
        }

        // [utest->req~modification-unilaterale-appliquee~1]
        #[test]
        fn un_champ_modifie_des_deux_cotes_a_l_identique_n_ecrit_rien() {
            let nom = fusion(
                Champ::NomAffiche,
                &["Camille E."],
                &["Camille  E. "],
                &["Camille"],
            );

            assert_eq!((nom.ecrire_source, nom.divergence), (false, false));
        }

        #[test]
        fn une_valeur_restee_au_referentiel_garde_sa_provenance() {
            let referentiel = du_referentiel(&["a@example.org"]);
            let source = de_la_source(&["A@example.org", "b@example.org"]);

            let email = fusionner(
                Champ::Email,
                &referentiel,
                &source,
                &base(&["a@example.org"]),
                Gagnant::Personne,
            );

            assert_eq!(email.referentiel.first(), referentiel.first());
        }

        #[test]
        fn la_base_suit_la_valeur_retenue() {
            let email = fusion(
                Champ::Email,
                &["a@example.org"],
                &["b@example.org"],
                &["a@example.org"],
            );

            assert_eq!(email.base, ["b@example.org"]);
        }
    }

    mod divergence {
        use super::*;

        // [utest->req~divergence-soumise-a-decision~1]
        #[test]
        fn un_nom_modifie_differemment_des_deux_cotes_diverge_et_le_referentiel_le_garde() {
            let nom = fusion(
                Champ::NomAffiche,
                &["Camille Ex."],
                &["C. Exemple"],
                &["Camille Exemple"],
            );

            assert_eq!(
                (
                    nom.divergence,
                    textes_du_referentiel(&nom),
                    nom.ecrire_source
                ),
                (true, vec!["Camille Ex."], false)
            );
        }

        #[test]
        fn la_base_ne_bouge_pas_tant_que_la_divergence_attend() {
            let nom = fusion(
                Champ::NomAffiche,
                &["Camille Ex."],
                &["C. Exemple"],
                &["Camille Exemple"],
            );

            assert_eq!(nom.base, ["Camille Exemple"]);
        }
    }

    mod multivalues {
        use super::*;

        // [utest->req~multivalues-par-valeur-normalisee~1]
        #[test]
        fn un_telephone_ajoute_sur_la_source_seulement_rejoint_la_fiche_sans_decision() {
            let telephones = fusion(
                Champ::Telephone,
                &["06 39 98 00 00"],
                &["+33639980000", "+33639980001"],
                &["06 39 98 00 00"],
            );

            assert_eq!(
                (textes_du_referentiel(&telephones), telephones.divergence),
                (vec!["06 39 98 00 00", "+33639980001"], false)
            );
        }

        #[test]
        fn des_ajouts_des_deux_cotes_se_combinent() {
            let emails = fusion(
                Champ::Email,
                &["a@example.org", "r@example.org"],
                &["a@example.org", "s@example.org"],
                &["a@example.org"],
            );

            assert_eq!(
                (textes_du_referentiel(&emails), emails.divergence),
                (
                    vec!["a@example.org", "r@example.org", "s@example.org"],
                    false
                )
            );
        }

        #[test]
        fn un_retrait_d_un_cote_et_un_ajout_de_l_autre_se_combinent() {
            let emails = fusion(
                Champ::Email,
                &["b@example.org"],
                &["a@example.org", "b@example.org", "c@example.org"],
                &["a@example.org", "b@example.org"],
            );

            assert_eq!(
                textes_du_referentiel(&emails),
                ["b@example.org", "c@example.org"]
            );
        }

        #[test]
        fn une_valeur_remplacee_differemment_des_deux_cotes_diverge() {
            let telephones = fusion(
                Champ::Telephone,
                &["0639980001"],
                &["0639980002"],
                &["0639980000"],
            );

            assert!(telephones.divergence);
        }

        #[test]
        fn la_combinaison_est_ecrite_vers_la_source_quand_elle_en_differe() {
            let emails = fusion(
                Champ::Email,
                &["a@example.org", "r@example.org"],
                &["a@example.org", "s@example.org"],
                &["a@example.org"],
            );

            assert!(emails.ecrire_source);
        }
    }

    mod autorite {
        use super::*;

        // [utest->req~autorite-de-la-reference~2]
        // [utest->req~autorite-par-champ~1]
        #[test]
        fn la_source_qui_fait_foi_impose_sa_valeur_sans_decision() {
            let nom = fusionner(
                Champ::Telephone,
                &du_referentiel(&["0639980001"]),
                &de_la_source(&["0639980002"]),
                &base(&["0639980000"]),
                Gagnant::Source,
            );

            assert_eq!(
                (textes_du_referentiel(&nom), nom.divergence),
                (vec!["0639980002"], false)
            );
        }

        #[test]
        fn le_referentiel_qui_fait_foi_est_ecrit_vers_la_source() {
            let nom = fusionner(
                Champ::NomAffiche,
                &du_referentiel(&["Camille Ex."]),
                &de_la_source(&["C. Exemple"]),
                &base(&["Camille Exemple"]),
                Gagnant::Referentiel,
            );

            assert_eq!((nom.ecrire_source, nom.divergence), (true, false));
        }
    }
}

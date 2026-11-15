use super::*;
use crate::decisions::{Action, IssueSuppression, Resolution, Statut};
use crate::reglages::{Autorite, Conflits, Direction, Reglages};
use crate::vcard::Version;

type Resultat = Result<(), Box<dyn std::error::Error>>;

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

/// Copie d'une vCard 3.0 portant les lignes données.
fn copie(uid: &str, lignes: &[&str]) -> CopieDistante {
    let mut contenu = format!("BEGIN:VCARD\r\nVERSION:3.0\r\nUID:{uid}\r\n");
    for ligne in lignes {
        contenu.push_str(ligne);
        contenu.push_str("\r\n");
    }
    contenu.push_str("END:VCARD\r\n");
    CopieDistante {
        uid: uid.to_owned(),
        etag: Some(format!("etag-{uid}-{}", lignes.len())),
        contenu,
    }
}

/// Référentiel d'une source, réglée comme demandé, et premier import de
/// ses copies.
fn importe(
    reglages: Reglages,
    copies: &[CopieDistante],
) -> Result<(Referentiel, SourceId), Box<dyn std::error::Error>> {
    let mut referentiel = Referentiel::new();
    let source = referentiel.sources_mut().ajouter("Nextcloud", Version::V3);
    referentiel.sources_mut().regler(source, reglages)?;
    referentiel.synchroniser(source, copies, date(1))?;
    Ok((referentiel, source))
}

fn fiche_de(referentiel: &Referentiel, source: SourceId, uid: &str) -> Result<FicheId, String> {
    referentiel
        .fiches()
        .find(|f| f.lien(source).is_some_and(|l| l.uid == uid))
        .map(Fiche::id)
        .ok_or_else(|| format!("aucune fiche liée à {uid}"))
}

fn textes(referentiel: &Referentiel, fiche: FicheId, champ: Champ) -> Vec<String> {
    referentiel
        .fiche(fiche)
        .map(|f| f.textes(champ))
        .unwrap_or_default()
}

fn ecrit(rapport: &Rapport, uid: &str) -> Option<String> {
    rapport.ecritures.iter().find_map(|e| match e {
        Ecriture::Carte { uid: u, contenu } if u == uid => Some(contenu.clone()),
        _ => None,
    })
}

mod import {
    use super::*;

    #[test]
    fn chaque_copie_inconnue_donne_une_fiche_liee() -> Resultat {
        let (referentiel, source) = importe(
            Reglages::default(),
            &[
                copie("a", &["FN:Camille Exemple"]),
                copie("b", &["FN:Dominique Fictif"]),
            ],
        )?;

        let liees = referentiel
            .fiches()
            .filter(|f| f.lien(source).is_some())
            .count();
        assert_eq!(liees, 2);
        Ok(())
    }

    #[test]
    fn une_copie_illisible_est_signalee_et_ne_vaut_pas_suppression() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille"])])?;
        let illisible = CopieDistante {
            uid: "a".to_owned(),
            etag: None,
            contenu: "pas une vCard".to_owned(),
        };

        let rapport = referentiel.synchroniser(source, &[illisible], date(2))?;

        assert_eq!((rapport.illisibles.len(), rapport.decisions.len()), (1, 0));
        Ok(())
    }
}

mod divergence {
    use super::*;

    // [utest->req~divergence-soumise-a-decision~1]
    #[test]
    fn un_nom_modifie_differemment_cree_une_decision_et_le_referentiel_garde_son_nom() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::NomAffiche, &["Camille Ex."], date(2))?;
        }

        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:C. Exemple"])], date(3))?;

        let nature = rapport
            .decisions
            .first()
            .and_then(|d| referentiel.decision(*d))
            .map(|d| d.nature);
        assert_eq!(
            (nature, textes(&referentiel, fiche, Champ::NomAffiche)),
            (
                Some(Nature::Divergence {
                    champ: Champ::NomAffiche
                }),
                vec!["Camille Ex.".to_owned()]
            )
        );
        Ok(())
    }

    #[test]
    fn une_divergence_qui_persiste_ne_cree_pas_de_seconde_decision() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::NomAffiche, &["Camille Ex."], date(2))?;
        }
        referentiel.synchroniser(source, &[copie("a", &["FN:C. Exemple"])], date(3))?;

        referentiel.synchroniser(source, &[copie("a", &["FN:C. Exemple"])], date(4))?;

        assert_eq!(referentiel.ouvertes().count(), 1);
        Ok(())
    }

    #[test]
    fn une_divergence_devenue_accord_est_close() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::NomAffiche, &["Camille Ex."], date(2))?;
        }
        referentiel.synchroniser(source, &[copie("a", &["FN:C. Exemple"])], date(3))?;

        referentiel.synchroniser(source, &[copie("a", &["FN:Camille Ex."])], date(4))?;

        assert_eq!(referentiel.ouvertes().count(), 0);
        Ok(())
    }
}

mod non_bloquante {
    use super::*;

    // [utest->req~synchronisation-non-bloquante~1]
    #[test]
    fn une_decision_en_attente_n_empeche_ni_le_telephone_de_la_fiche_ni_l_autre_fiche() -> Resultat
    {
        let (mut referentiel, source) = importe(
            Reglages::default(),
            &[
                copie("a", &["FN:Camille Exemple", "TEL:0639980000"]),
                copie("b", &["FN:Dominique Fictif", "TITLE:Ancienne"]),
            ],
        )?;
        let (a, b) = (
            fiche_de(&referentiel, source, "a")?,
            fiche_de(&referentiel, source, "b")?,
        );
        if let Some(f) = referentiel.fiche_mut(a) {
            f.modifier(Champ::NomAffiche, &["Camille Ex."], date(2))?;
        }
        referentiel.synchroniser(
            source,
            &[
                copie("a", &["FN:C. Exemple", "TEL:0639980000"]),
                copie("b", &["FN:Dominique Fictif", "TITLE:Ancienne"]),
            ],
            date(3),
        )?;

        referentiel.synchroniser(
            source,
            &[
                copie("a", &["FN:C. Exemple", "TEL:0639980009"]),
                copie("b", &["FN:Dominique Fictif", "TITLE:Nouvelle"]),
            ],
            date(4),
        )?;

        assert_eq!(
            (
                referentiel.ouvertes().count(),
                textes(&referentiel, a, Champ::Telephone),
                textes(&referentiel, b, Champ::Fonction)
            ),
            (
                1,
                vec!["0639980009".to_owned()],
                vec!["Nouvelle".to_owned()]
            )
        );
        Ok(())
    }
}

mod ecriture {
    use super::*;

    #[test]
    fn une_modification_du_referentiel_est_ecrite_vers_la_source_dans_sa_version() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
        }

        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(3))?;

        let ecrite = ecrit(&rapport, "a").unwrap_or_default();
        assert!(
            ecrite.contains("VERSION:3.0\r\n") && ecrite.contains("TITLE:Responsable\r\n"),
            "{ecrite}"
        );
        Ok(())
    }

    // [utest->req~reglages-par-source~1]
    #[test]
    fn rien_n_est_ecrit_vers_une_source_en_lecture_seule() -> Resultat {
        let lecture_seule = Reglages {
            direction: Direction::LectureSeule,
            ..Reglages::default()
        };
        let (mut referentiel, source) =
            importe(lecture_seule, &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
        }

        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(3))?;

        assert_eq!(rapport.ecritures, []);
        Ok(())
    }

    #[test]
    fn une_source_en_lecture_seule_ne_voit_pas_sa_valeur_ecrasee_au_tour_suivant() -> Resultat {
        let lecture_seule = Reglages {
            direction: Direction::LectureSeule,
            ..Reglages::default()
        };
        let (mut referentiel, source) =
            importe(lecture_seule, &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
        }
        referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(3))?;

        referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(4))?;

        assert_eq!(
            textes(&referentiel, fiche, Champ::Fonction),
            ["Responsable"]
        );
        Ok(())
    }

    #[test]
    fn une_source_en_ecriture_seule_est_recouverte_par_le_referentiel() -> Resultat {
        let ecriture_seule = Reglages {
            direction: Direction::EcritureSeule,
            ..Reglages::default()
        };
        let mut referentiel = Referentiel::new();
        let source = referentiel.sources_mut().ajouter("Miroir", Version::V3);
        referentiel.sources_mut().regler(source, ecriture_seule)?;
        let mut fiche = Fiche::nouvelle();
        fiche.modifier(Champ::NomAffiche, &["Camille Exemple"], date(1))?;
        fiche.lier(Lien::new(source, "a"));
        referentiel.ajouter(fiche);

        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:Autre nom"])], date(2))?;

        let ecrite = ecrit(&rapport, "a").unwrap_or_default();
        assert!(ecrite.contains("FN:Camille Exemple\r\n"), "{ecrite}");
        Ok(())
    }
}

mod autorite {
    use super::*;

    // [utest->req~autorite-de-la-reference~2]
    #[test]
    fn la_reference_generale_impose_sa_valeur_sans_toucher_la_file() -> Resultat {
        let reference = Reglages {
            autorite: Autorite::CetteSource,
            conflits: Conflits::AutoriteGagne,
            ..Reglages::default()
        };
        let (mut referentiel, source) = importe(reference, &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::NomAffiche, &["Camille Ex."], date(2))?;
        }

        referentiel.synchroniser(source, &[copie("a", &["FN:C. Exemple"])], date(3))?;

        assert_eq!(
            (
                textes(&referentiel, fiche, Champ::NomAffiche),
                referentiel.decisions().len()
            ),
            (vec!["C. Exemple".to_owned()], 0)
        );
        Ok(())
    }
}

mod resolution {
    use super::*;

    // [utest->req~actions-de-resolution~1]
    #[test]
    fn une_valeur_saisie_est_ecrite_vers_la_source_a_la_synchronisation_suivante() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::NomAffiche, &["Camille Ex."], date(2))?;
        }
        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:C. Exemple"])], date(3))?;
        let decision = rapport
            .decisions
            .first()
            .copied()
            .ok_or("aucune décision")?;
        let saisie = Action::Saisir(vec!["Camille E.".to_owned()]);
        referentiel.resoudre(decision, Resolution::Divergence(saisie), date(4))?;

        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:C. Exemple"])], date(5))?;

        let ecrite = ecrit(&rapport, "a").unwrap_or_default();
        assert_eq!(
            (
                ecrite.contains("FN:Camille E.\r\n"),
                referentiel.decision(decision).map(|d| d.statut)
            ),
            (true, Some(Statut::Resolue))
        );
        Ok(())
    }
}

mod suppression {
    use super::*;

    // [utest->req~suppression-sans-destruction~1]
    #[test]
    fn une_fiche_disparue_de_la_source_reste_intacte_et_une_decision_est_creee() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        let avant = referentiel.fiche(fiche).cloned();

        let rapport = referentiel.synchroniser(source, &[], date(2))?;

        let nature = rapport
            .decisions
            .first()
            .and_then(|d| referentiel.decision(*d))
            .map(|d| d.nature);
        assert_eq!(
            (referentiel.fiche(fiche).cloned(), nature),
            (avant, Some(Nature::Suppression))
        );
        Ok(())
    }

    // [utest->req~issues-suppression-detectee~1]
    #[test]
    fn une_suppression_resolue_par_restaurer_reecrit_la_fiche_sur_la_source() -> Resultat {
        let (mut referentiel, source) = importe(
            Reglages::default(),
            &[copie("a", &["FN:Camille Exemple", "X-PROPRIETAIRE:garde"])],
        )?;
        let rapport = referentiel.synchroniser(source, &[], date(2))?;
        let decision = rapport
            .decisions
            .first()
            .copied()
            .ok_or("aucune décision")?;
        referentiel.resoudre(
            decision,
            Resolution::Suppression(IssueSuppression::Restaurer),
            date(3),
        )?;

        let rapport = referentiel.synchroniser(source, &[], date(4))?;

        let ecrite = ecrit(&rapport, "a").unwrap_or_default();
        assert!(
            ecrite.contains("FN:Camille Exemple\r\n")
                && ecrite.contains("X-PROPRIETAIRE:garde\r\n"),
            "{ecrite}"
        );
        Ok(())
    }

    #[test]
    fn une_suppression_reglee_sur_ignorer_detache_la_fiche() -> Resultat {
        let ignorer = Reglages {
            suppressions: Suppressions::Ignorer,
            ..Reglages::default()
        };
        let (mut referentiel, source) = importe(ignorer, &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;

        referentiel.synchroniser(source, &[], date(2))?;

        assert_eq!(
            referentiel.fiche(fiche).map(|f| f.lien(source).is_some()),
            Some(false)
        );
        Ok(())
    }

    #[test]
    fn une_suppression_programmee_dans_l_application_est_envoyee_a_la_source() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        referentiel.supprimer_fiche(fiche, &[source])?;

        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(2))?;

        assert_eq!(
            (rapport.ecritures, referentiel.fiches().count()),
            (
                vec![Ecriture::Suppression {
                    uid: "a".to_owned()
                }],
                0
            )
        );
        Ok(())
    }
}

mod doublons {
    use super::*;

    /// Référentiel où une première source a importé Camille.
    fn camille_importee() -> Result<(Referentiel, SourceId), Box<dyn std::error::Error>> {
        importe(
            Reglages::default(),
            &[copie(
                "nc-1",
                &["FN:Camille Exemple", "EMAIL:camille@example.org"],
            )],
        )
    }

    // [utest->req~traitement-des-doublons~1]
    #[test]
    fn un_doublon_certain_laisse_une_seule_fiche_liee_aux_deux_origines() -> Resultat {
        let (mut referentiel, nextcloud) = camille_importee()?;
        let google = referentiel.sources_mut().ajouter("Google", Version::V3);

        referentiel.synchroniser(
            google,
            &[copie(
                "g-1",
                &[
                    "FN:Camille EXEMPLE",
                    "EMAIL:Camille@example.org",
                    "TEL:0639980000",
                ],
            )],
            date(2),
        )?;

        let liens: Vec<_> = referentiel
            .fiches()
            .map(|f| (f.lien(nextcloud).is_some(), f.lien(google).is_some()))
            .collect();
        assert_eq!(liens, [(true, true)]);
        Ok(())
    }

    #[test]
    fn un_doublon_certain_apporte_ses_valeurs_a_la_fiche_existante() -> Resultat {
        let (mut referentiel, nextcloud) = camille_importee()?;
        let google = referentiel.sources_mut().ajouter("Google", Version::V3);
        let fiche = fiche_de(&referentiel, nextcloud, "nc-1")?;

        referentiel.synchroniser(
            google,
            &[copie(
                "g-1",
                &[
                    "FN:Camille Exemple",
                    "EMAIL:camille@example.org",
                    "TEL:0639980000",
                ],
            )],
            date(2),
        )?;

        assert_eq!(
            textes(&referentiel, fiche, Champ::Telephone),
            ["0639980000"]
        );
        Ok(())
    }

    // [utest->req~traitement-des-doublons~1]
    #[test]
    fn un_doublon_probable_laisse_deux_fiches_et_cree_une_decision() -> Resultat {
        let (mut referentiel, _) = camille_importee()?;
        let google = referentiel.sources_mut().ajouter("Google", Version::V3);

        let rapport =
            referentiel.synchroniser(google, &[copie("g-1", &["FN:Camille Exemplé"])], date(2))?;

        let natures: Vec<_> = rapport
            .decisions
            .iter()
            .filter_map(|d| referentiel.decision(*d))
            .map(|d| matches!(d.nature, Nature::DoublonProbable { .. }))
            .collect();
        assert_eq!((referentiel.fiches().count(), natures), (2, vec![true]));
        Ok(())
    }

    // [utest->req~detection-doublons~1]
    #[test]
    fn l_import_detecte_le_doublon_de_meme_email() -> Resultat {
        let (mut referentiel, _) = camille_importee()?;
        let google = referentiel.sources_mut().ajouter("Google", Version::V3);

        let rapport = referentiel.synchroniser(
            google,
            &[copie("g-1", &["EMAIL:camille@example.org"])],
            date(2),
        )?;

        assert_eq!(rapport.decisions.len(), 1);
        Ok(())
    }

    #[test]
    fn une_carte_sans_nom_ne_recoit_rien_de_la_fiche_dont_elle_reprend_l_email() -> Resultat {
        let (mut referentiel, nextcloud) = camille_importee()?;
        let fiche = fiche_de(&referentiel, nextcloud, "nc-1")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::Note, &["note privée"], date(2))?;
        }
        let google = referentiel.sources_mut().ajouter("Google", Version::V3);

        let rapport = referentiel.synchroniser(
            google,
            &[copie("g-1", &["EMAIL:camille@example.org"])],
            date(3),
        )?;

        assert_eq!(rapport.ecritures, []);
        Ok(())
    }
}

mod echec_d_ecriture {
    use super::*;

    #[test]
    fn une_ecriture_annulee_rend_au_lien_son_etat_d_avant() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
        }
        let avant = referentiel
            .fiche(fiche)
            .and_then(|f| f.lien(source))
            .cloned();
        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(3))?;

        referentiel.annuler_ecriture(&rapport, "a");

        assert_eq!(
            referentiel
                .fiche(fiche)
                .and_then(|f| f.lien(source))
                .cloned(),
            avant
        );
        Ok(())
    }

    #[test]
    fn apres_une_ecriture_annulee_la_modification_repart_au_tour_suivant() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
        }
        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(3))?;
        referentiel.annuler_ecriture(&rapport, "a");

        let rapport =
            referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(4))?;

        let ecrite = ecrit(&rapport, "a").unwrap_or_default();
        assert!(ecrite.contains("TITLE:Responsable\r\n"), "{ecrite}");
        Ok(())
    }

    #[test]
    fn sans_annulation_la_modification_serait_perdue() -> Resultat {
        // Témoin de la nécessité de l'annulation : si l'écriture échoue sans
        // être annulée, la base croit la source à jour, et le tour suivant
        // prend l'ancienne valeur de la source pour une modification.
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;
        let fiche = fiche_de(&referentiel, source, "a")?;
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
        }
        referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(3))?;

        referentiel.synchroniser(source, &[copie("a", &["FN:Camille Exemple"])], date(4))?;

        assert_eq!(
            textes(&referentiel, fiche, Champ::Fonction),
            Vec::<String>::new()
        );
        Ok(())
    }

    #[test]
    fn une_suppression_reprogrammee_repart_au_tour_suivant() -> Resultat {
        let (mut referentiel, source) =
            importe(Reglages::default(), &[copie("a", &["FN:Camille Exemple"])])?;

        referentiel.reprogrammer_suppression(source, "a");

        assert_eq!(
            referentiel
                .suppressions_en_attente(source)
                .collect::<Vec<_>>(),
            ["a"]
        );
        Ok(())
    }
}

mod relations {
    use super::*;
    use crate::relations::{Relation, TypeRelation};

    /// Copie d'une vCard de la version donnée portant les lignes données.
    fn copie_en(version: Version, uid: &str, lignes: &[&str]) -> CopieDistante {
        let mut copie = copie(uid, lignes);
        copie.contenu = copie
            .contenu
            .replace("VERSION:3.0", &format!("VERSION:{}", version.texte()));
        copie
    }

    /// Source de la version donnée, et premier import de ses copies.
    fn source_en(
        version: Version,
        copies: &[CopieDistante],
    ) -> Result<(Referentiel, SourceId), Box<dyn std::error::Error>> {
        let mut referentiel = Referentiel::new();
        let source = referentiel.sources_mut().ajouter("Nextcloud", version);
        referentiel.synchroniser(source, copies, date(1))?;
        Ok((referentiel, source))
    }

    fn relations(referentiel: &Referentiel, fiche: FicheId) -> Vec<Relation> {
        referentiel.relations(fiche)
    }

    /// Copie telle que la source la sert après une écriture rendue.
    fn apres_ecriture(rapport: &Rapport, uid: &str) -> Result<CopieDistante, String> {
        let contenu = ecrit(rapport, uid).ok_or_else(|| format!("aucune écriture de {uid}"))?;
        Ok(CopieDistante {
            uid: uid.to_owned(),
            etag: Some(format!("etag-{uid}-ecrit")),
            contenu,
        })
    }

    // [utest->req~synchronisation-relations~1]
    #[test]
    fn en_4_0_une_relation_designe_la_copie_liee_par_son_uid() -> Resultat {
        let copies = [
            copie_en(Version::V4, "a", &["FN:Alix"]),
            copie_en(Version::V4, "b", &["FN:Basile"]),
        ];
        let (mut referentiel, source) = source_en(Version::V4, &copies)?;
        let (a, b) = (
            fiche_de(&referentiel, source, "a")?,
            fiche_de(&referentiel, source, "b")?,
        );
        referentiel.lier(a, TypeRelation::Parent, b, date(2))?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let carte = ecrit(&rapport, "a").ok_or("écriture de a attendue")?;
        assert!(
            carte.contains("RELATED;TYPE=parent:urn:uuid:b\r\n"),
            "{carte}"
        );
        Ok(())
    }

    // [utest->req~synchronisation-relations~1]
    #[test]
    fn en_4_0_une_relation_vers_une_fiche_hors_source_s_ecrit_par_son_nom() -> Resultat {
        let copies = [copie_en(Version::V4, "a", &["FN:Alix"])];
        let (mut referentiel, source) = source_en(Version::V4, &copies)?;
        let a = fiche_de(&referentiel, source, "a")?;
        let mut hors = Fiche::nouvelle();
        hors.modifier(Champ::NomAffiche, &["Docteur Martin"], date(1))?;
        let martin = referentiel.ajouter(hors);
        referentiel.lier(a, TypeRelation::Medecin, martin, date(2))?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let carte = ecrit(&rapport, "a").ok_or("écriture de a attendue")?;
        assert!(
            carte.contains("RELATED;TYPE=contact;X-ABLABEL=Médecin;VALUE=text:Docteur Martin\r\n"),
            "{carte}"
        );
        Ok(())
    }

    // [utest->req~synchronisation-relations~1]
    #[test]
    fn en_4_0_une_relation_recue_se_resout_par_l_uid_de_la_copie() -> Resultat {
        let copies = [
            copie_en(
                Version::V4,
                "a",
                &["FN:Alix", "RELATED;TYPE=sibling:urn:uuid:b"],
            ),
            copie_en(Version::V4, "b", &["FN:Basile"]),
        ];
        let (mut referentiel, source) = source_en(Version::V4, &copies)?;

        // La copie liée n'était pas encore connue au premier passage : la
        // relation se résout au suivant.
        referentiel.synchroniser(source, &copies, date(2))?;

        let (a, b) = (
            fiche_de(&referentiel, source, "a")?,
            fiche_de(&referentiel, source, "b")?,
        );
        assert_eq!(
            relations(&referentiel, a),
            [Relation {
                type_relation: TypeRelation::FrereSoeur,
                fiche: b
            }]
        );
        Ok(())
    }

    // [utest->req~synchronisation-relations~1]
    #[test]
    fn en_3_0_une_relation_suit_la_convention_apple() -> Resultat {
        let copies = [copie("a", &["FN:Alix"]), copie("b", &["FN:Basile"])];
        let (mut referentiel, source) = source_en(Version::V3, &copies)?;
        let (a, b) = (
            fiche_de(&referentiel, source, "a")?,
            fiche_de(&referentiel, source, "b")?,
        );
        referentiel.lier(a, TypeRelation::Ami, b, date(2))?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let carte = ecrit(&rapport, "a").ok_or("écriture de a attendue")?;
        assert!(
            carte.contains("item1.X-ABRELATEDNAMES:Basile\r\nitem1.X-ABLABEL:_$!<Friend>!$_\r\n"),
            "{carte}"
        );
        Ok(())
    }

    // [utest->req~synchronisation-relations~1]
    #[test]
    fn en_3_0_une_relation_apple_recue_se_resout_par_le_nom() -> Resultat {
        let copies = [
            copie(
                "a",
                &[
                    "FN:Alix",
                    "item7.X-ABRELATEDNAMES:Basile",
                    "item7.X-ABLabel:_$!<Mother>!$_",
                ],
            ),
            copie("b", &["FN:Basile"]),
        ];
        let (mut referentiel, source) = source_en(Version::V3, &copies)?;

        referentiel.synchroniser(source, &copies, date(2))?;

        let (a, b) = (
            fiche_de(&referentiel, source, "a")?,
            fiche_de(&referentiel, source, "b")?,
        );
        assert_eq!(
            relations(&referentiel, a),
            [Relation {
                type_relation: TypeRelation::Parent,
                fiche: b
            }]
        );
        Ok(())
    }

    // [utest->req~synchronisation-relations~1]
    // [utest->req~lien-par-identifiant-maitre~1]
    #[test]
    fn en_3_0_renommer_la_fiche_liee_ne_perd_pas_la_relation() -> Resultat {
        let copies = [copie("a", &["FN:Alix"]), copie("b", &["FN:Basile"])];
        let (mut referentiel, source) = source_en(Version::V3, &copies)?;
        let (a, b) = (
            fiche_de(&referentiel, source, "a")?,
            fiche_de(&referentiel, source, "b")?,
        );
        referentiel.lier(a, TypeRelation::Ami, b, date(2))?;
        let rapport = referentiel.synchroniser(source, &copies, date(3))?;
        let ecrites = [
            apres_ecriture(&rapport, "a")?,
            apres_ecriture(&rapport, "b")?,
        ];
        referentiel.fiche_mut(b).ok_or("fiche attendue")?.modifier(
            Champ::NomAffiche,
            &["Basile Renommé"],
            date(4),
        )?;

        referentiel.synchroniser(source, &ecrites, date(5))?;

        assert_eq!(
            relations(&referentiel, a),
            [Relation {
                type_relation: TypeRelation::Ami,
                fiche: b
            }]
        );
        Ok(())
    }

    // [utest->req~synchronisation-relations~1]
    #[test]
    fn une_relation_que_la_source_ne_peut_porter_reste_au_referentiel() -> Resultat {
        let copies = [copie("a", &["FN:Alix"])];
        let (mut referentiel, source) = source_en(Version::V3, &copies)?;
        let a = fiche_de(&referentiel, source, "a")?;
        let mut homonymes = Vec::new();
        for _ in 0..2 {
            let mut fiche = Fiche::nouvelle();
            fiche.modifier(Champ::NomAffiche, &["Basile"], date(1))?;
            homonymes.push(referentiel.ajouter(fiche));
        }
        let b = homonymes.first().copied().ok_or("fiche attendue")?;
        referentiel.lier(a, TypeRelation::depuis_texte("Voisin"), b, date(2))?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;
        referentiel.synchroniser(source, &copies, date(4))?;

        assert!(ecrit(&rapport, "a").is_none());
        assert_eq!(relations(&referentiel, a).len(), 1);
        Ok(())
    }

    // [utest->req~synchronisation-relations~1]
    #[test]
    fn une_relation_qu_on_ne_sait_resoudre_est_conservee_telle_quelle() -> Resultat {
        let inconnue = "RELATED;TYPE=friend:urn:uuid:inconnue";
        let copies = [copie_en(Version::V4, "a", &["FN:Alix", inconnue])];
        let (mut referentiel, source) = source_en(Version::V4, &copies)?;
        let a = fiche_de(&referentiel, source, "a")?;
        referentiel.fiche_mut(a).ok_or("fiche attendue")?.modifier(
            Champ::Note,
            &["note nouvelle"],
            date(2),
        )?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let carte = ecrit(&rapport, "a").ok_or("écriture de a attendue")?;
        assert!(carte.contains(&format!("{inconnue}\r\n")), "{carte}");
        Ok(())
    }
}

mod groupes {
    use super::*;

    // [utest->req~groupes-en-categories~1]
    #[test]
    fn une_fiche_ecrit_ses_groupes_par_leur_chemin_complet() -> Resultat {
        let copies = [copie("a", &["FN:Alix"])];
        let (mut referentiel, source) = importe(Reglages::default(), &copies)?;
        let a = fiche_de(&referentiel, source, "a")?;
        referentiel.ranger(a, &["Travail/Clients/Client X", "Famille, proche"], date(2))?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let carte = ecrit(&rapport, "a").ok_or("écriture de a attendue")?;
        assert!(
            carte.contains("CATEGORIES:Travail/Clients/Client X,Famille\\, proche\r\n"),
            "{carte}"
        );
        Ok(())
    }

    // [utest->req~groupes-en-categories~1]
    #[test]
    fn la_hierarchie_se_reconstruit_a_la_lecture() -> Resultat {
        let (referentiel, source) = importe(
            Reglages::default(),
            &[copie(
                "a",
                &["FN:Alix", "CATEGORIES:Travail/Clients,Famille"],
            )],
        )?;
        let a = fiche_de(&referentiel, source, "a")?;

        assert_eq!(
            (referentiel.groupes_de(a), referentiel.groupes()),
            (
                vec!["Travail/Clients".to_owned(), "Famille".to_owned()],
                vec![
                    "Famille".to_owned(),
                    "Travail".to_owned(),
                    "Travail/Clients".to_owned()
                ]
            )
        );
        Ok(())
    }

    #[test]
    fn des_categories_inchangees_sont_rendues_a_l_octet_pres() -> Resultat {
        let ligne = "CATEGORIES:famille , Voisins";
        let copies = [copie("a", &["FN:Alix", ligne])];
        let (mut referentiel, source) = importe(Reglages::default(), &copies)?;
        let a = fiche_de(&referentiel, source, "a")?;
        referentiel.fiche_mut(a).ok_or("fiche attendue")?.modifier(
            Champ::Note,
            &["note"],
            date(2),
        )?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let carte = ecrit(&rapport, "a").ok_or("écriture de a attendue")?;
        assert!(carte.contains(&format!("{ligne}\r\n")), "{carte}");
        Ok(())
    }

    // [utest->req~renommage-groupe-operation-unique~1]
    #[test]
    fn renommer_un_parent_ne_cree_aucune_decision_et_reecrit_ses_quarante_fiches() -> Resultat {
        let copies: Vec<CopieDistante> = (0..40)
            .map(|i| {
                let groupe = if i % 2 == 0 {
                    "Travail/Clients"
                } else {
                    "Travail/Fournisseurs"
                };
                copie(
                    &format!("f{i}"),
                    &[&format!("FN:Fiche {i}"), &format!("CATEGORIES:{groupe}")],
                )
            })
            .collect();
        let (mut referentiel, source) = importe(Reglages::default(), &copies)?;

        referentiel.renommer_groupe("Travail", "Boulot", date(2))?;
        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let reecrites = rapport
            .ecritures
            .iter()
            .filter(|e| matches!(e, Ecriture::Carte { contenu, .. } if contenu.contains("CATEGORIES:Boulot/")))
            .count();
        assert_eq!((rapport.decisions.len(), reecrites), (0, 40));
        Ok(())
    }
}

mod dates {
    use super::*;
    use crate::dates::{Date, DateDeFiche, TypeDate};

    fn source_en(
        version: Version,
        copies: &[CopieDistante],
    ) -> Result<(Referentiel, SourceId), Box<dyn std::error::Error>> {
        let mut referentiel = Referentiel::new();
        let source = referentiel.sources_mut().ajouter("Nextcloud", version);
        referentiel.synchroniser(source, copies, date(1))?;
        Ok((referentiel, source))
    }

    fn copie_4(uid: &str, lignes: &[&str]) -> CopieDistante {
        let mut copie = copie(uid, lignes);
        copie.contenu = copie.contenu.replace("VERSION:3.0", "VERSION:4.0");
        copie
    }

    /// Carte écrite pour « a » après l'ajout des dates données.
    fn ecrite_avec(
        version: Version,
        dates: &[(TypeDate, &str)],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let copies = [if version == Version::V4 {
            copie_4("a", &["FN:Alix"])
        } else {
            copie("a", &["FN:Alix"])
        }];
        let (mut referentiel, source) = source_en(version, &copies)?;
        let a = fiche_de(&referentiel, source, "a")?;
        for (type_date, texte) in dates {
            referentiel.ajouter_date(a, type_date.clone(), texte, date(2))?;
        }
        let rapport = referentiel.synchroniser(source, &copies, date(3))?;
        Ok(ecrit(&rapport, "a").ok_or("écriture de a attendue")?)
    }

    // [utest->req~synchronisation-dates~1]
    #[test]
    fn en_4_0_naissance_et_mariage_vont_dans_bday_et_anniversary() -> Resultat {
        let carte = ecrite_avec(
            Version::V4,
            &[
                (TypeDate::Naissance, "--05-17"),
                (TypeDate::Mariage, "2005-06-11"),
            ],
        )?;

        assert!(
            carte.contains("BDAY:--0517\r\n") && carte.contains("ANNIVERSARY:20050611\r\n"),
            "{carte}"
        );
        Ok(())
    }

    // [utest->req~synchronisation-dates~1]
    #[test]
    fn en_3_0_mariage_et_annee_omise_suivent_la_convention_apple() -> Resultat {
        let carte = ecrite_avec(
            Version::V3,
            &[
                (TypeDate::Naissance, "--05-17"),
                (TypeDate::Mariage, "2005-06-11"),
            ],
        )?;

        assert!(
            carte.contains("BDAY;X-APPLE-OMIT-YEAR=1604:1604-05-17\r\n")
                && carte.contains(
                    "item1.X-ABDATE:2005-06-11\r\nitem1.X-ABLABEL:_$!<Anniversary>!$_\r\n"
                ),
            "{carte}"
        );
        Ok(())
    }

    // [utest->req~synchronisation-dates~1]
    #[test]
    fn la_fete_et_les_dates_personnalisees_sont_des_x_abdate_etiquetees() -> Resultat {
        let carte = ecrite_avec(
            Version::V4,
            &[
                (TypeDate::Fete, "--05-18"),
                (TypeDate::depuis_texte("Arrivée"), "2010-09-01"),
            ],
        )?;

        assert!(
            carte.contains("item1.X-ABDATE:--0518\r\nitem1.X-ABLABEL:Fête du prénom\r\n")
                && carte.contains("item2.X-ABDATE:20100901\r\nitem2.X-ABLABEL:Arrivée\r\n"),
            "{carte}"
        );
        Ok(())
    }

    // [utest->req~synchronisation-dates~1]
    // [utest->req~types-de-dates~1]
    #[test]
    fn les_dates_recues_se_relisent_selon_leur_type_et_sans_annee_omise() -> Resultat {
        let (referentiel, source) = source_en(
            Version::V3,
            &[copie(
                "a",
                &[
                    "FN:Alix",
                    "BDAY;X-APPLE-OMIT-YEAR=1604:1604-05-17",
                    "item3.X-ABDATE:2005-06-11",
                    "item3.X-ABLabel:_$!<Anniversary>!$_",
                    "item4.X-ABDATE;X-APPLE-OMIT-YEAR=1604:1604-05-18",
                    "item4.X-ABLabel:Fête du prénom",
                ],
            )],
        )?;
        let a = fiche_de(&referentiel, source, "a")?;

        let dates: Vec<String> = referentiel
            .dates(a)
            .iter()
            .map(DateDeFiche::texte)
            .collect();

        assert_eq!(
            dates,
            ["naissance;--05-17", "mariage;2005-06-11", "fete;--05-18"]
        );
        Ok(())
    }

    #[test]
    fn une_x_abdate_sans_libelle_est_conservee_telle_quelle() -> Resultat {
        let ligne = "item9.X-ABDATE:2001-01-01";
        let copies = [copie("a", &["FN:Alix", ligne])];
        let (mut referentiel, source) = source_en(Version::V3, &copies)?;
        let a = fiche_de(&referentiel, source, "a")?;
        referentiel.ajouter_date(a, TypeDate::Mariage, "2005-06-11", date(2))?;

        let rapport = referentiel.synchroniser(source, &copies, date(3))?;

        let carte = ecrit(&rapport, "a").ok_or("écriture de a attendue")?;
        assert!(
            carte.contains(&format!("{ligne}\r\n")) && carte.contains("item10.X-ABDATE:2005-06-11"),
            "{carte}"
        );
        assert_eq!(
            referentiel.dates(a).first().map(|d| d.date),
            Date::lire("2005-06-11")
        );
        Ok(())
    }
}

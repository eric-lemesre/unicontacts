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

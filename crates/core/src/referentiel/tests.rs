use super::*;
use crate::fiche::Lien;
use crate::reglages::{Direction, ErreurReglage, Reglages};
use crate::vcard::Version;

type Resultat = Result<(), Box<dyn std::error::Error>>;

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

fn textes(referentiel: &Referentiel, fiche: FicheId, champ: Champ) -> Vec<String> {
    referentiel
        .fiche(fiche)
        .map(|f| f.textes(champ))
        .unwrap_or_default()
}

/// Référentiel d'une source, avec une fiche liée dont le champ vaut `valeur`.
fn une_fiche_liee(
    champ: Champ,
    valeur: &str,
) -> Result<(Referentiel, SourceId, FicheId), ErreurFiche> {
    let mut referentiel = Referentiel::new();
    let source = referentiel.sources_mut().ajouter("Nextcloud", Version::V3);
    let mut fiche = Fiche::nouvelle();
    fiche.modifier(champ, &[valeur], date(1))?;
    let mut lien = Lien::new(source, "uid-1");
    lien.base.insert(champ, vec![valeur.to_owned()]);
    fiche.lier(lien);
    let id = referentiel.ajouter(fiche);
    Ok((referentiel, source, id))
}

fn divergence(
    referentiel: &mut Referentiel,
    fiche: FicheId,
    source: SourceId,
    champ: Champ,
    [r, s, b]: [&str; 3],
) -> DecisionId {
    let versions = Versions {
        referentiel: vec![r.to_owned()],
        source: vec![s.to_owned()],
        base: vec![b.to_owned()],
    };
    referentiel.soumettre(
        Nature::Divergence { champ },
        fiche,
        source,
        versions,
        date(2),
    )
}

fn statut(referentiel: &Referentiel, id: DecisionId) -> Option<Statut> {
    referentiel.decision(id).map(|d| d.statut)
}

mod soumission {
    use super::*;

    #[test]
    fn une_divergence_deja_ouverte_est_mise_a_jour_et_non_dupliquee() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille")?;
        let champ = Champ::NomAffiche;
        divergence(
            &mut referentiel,
            fiche,
            source,
            champ,
            ["A", "B", "Camille"],
        );

        divergence(
            &mut referentiel,
            fiche,
            source,
            champ,
            ["A", "C", "Camille"],
        );

        assert_eq!(referentiel.ouvertes().count(), 1);
        Ok(())
    }
}

mod report {
    use super::*;

    // [utest->req~report-decision~1]
    #[test]
    fn une_decision_reportee_reste_dans_la_file_avec_le_statut_reportee() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille Ex.")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::NomAffiche,
            ["Camille Ex.", "C. Exemple", "Camille"],
        );

        referentiel.reporter(id)?;

        assert_eq!(statut(&referentiel, id), Some(Statut::Reportee));
        Ok(())
    }

    // [utest->req~report-decision~1]
    #[test]
    fn reporter_ne_modifie_aucune_des_valeurs_en_presence() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille Ex.")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::NomAffiche,
            ["Camille Ex.", "C. Exemple", "Camille"],
        );
        let avant = referentiel.clone();

        referentiel.reporter(id)?;

        assert_eq!(
            (
                &referentiel.fiches,
                referentiel.decision(id).map(|d| d.versions.clone())
            ),
            (
                &avant.fiches,
                avant.decision(id).map(|d| d.versions.clone())
            )
        );
        Ok(())
    }
}

mod actions {
    use super::*;

    // [utest->req~actions-de-resolution~1]
    #[test]
    fn une_valeur_saisie_devient_celle_du_referentiel_et_la_decision_est_resolue() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille Ex.")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::NomAffiche,
            ["Camille Ex.", "C. Exemple", "Camille"],
        );

        referentiel.resoudre(
            id,
            Resolution::Divergence(Action::Saisir(vec!["Camille Exemple".to_owned()])),
            date(3),
        )?;

        assert_eq!(
            (
                textes(&referentiel, fiche, Champ::NomAffiche),
                statut(&referentiel, id)
            ),
            (vec!["Camille Exemple".to_owned()], Some(Statut::Resolue))
        );
        Ok(())
    }

    #[test]
    fn la_base_prend_la_valeur_de_la_source_vue_par_la_decision() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille Ex.")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::NomAffiche,
            ["Camille Ex.", "C. Exemple", "Camille"],
        );

        referentiel.resoudre(
            id,
            Resolution::Divergence(Action::GarderReferentiel),
            date(3),
        )?;

        let base = referentiel
            .fiche(fiche)
            .and_then(|f| f.lien(source))
            .and_then(|l| l.base.get(&Champ::NomAffiche).cloned());
        assert_eq!(base, Some(vec!["C. Exemple".to_owned()]));
        Ok(())
    }

    #[test]
    fn garder_la_source_donne_au_referentiel_la_valeur_de_la_source() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille Ex.")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::NomAffiche,
            ["Camille Ex.", "C. Exemple", "Camille"],
        );

        referentiel.resoudre(id, Resolution::Divergence(Action::GarderSource), date(3))?;

        assert_eq!(
            textes(&referentiel, fiche, Champ::NomAffiche),
            ["C. Exemple"]
        );
        Ok(())
    }

    #[test]
    fn garder_les_deux_reunit_les_valeurs_d_un_champ_multivalue() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::Telephone, "0639980001")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::Telephone,
            ["0639980001", "0639980002", "0639980000"],
        );

        referentiel.resoudre(id, Resolution::Divergence(Action::GarderLesDeux), date(3))?;

        assert_eq!(
            textes(&referentiel, fiche, Champ::Telephone),
            ["0639980001", "0639980002"]
        );
        Ok(())
    }

    #[test]
    fn une_resolution_inadaptee_est_refusee() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::NomAffiche,
            ["A", "B", "Camille"],
        );

        let refus = referentiel.resoudre(
            id,
            Resolution::Suppression(IssueSuppression::Detacher),
            date(3),
        );

        assert_eq!(refus, Err(ErreurDecision::ResolutionInadmise(id)));
        Ok(())
    }

    #[test]
    fn une_decision_resolue_ne_se_resout_pas_deux_fois() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille")?;
        let id = divergence(
            &mut referentiel,
            fiche,
            source,
            Champ::NomAffiche,
            ["A", "B", "Camille"],
        );
        referentiel.resoudre(
            id,
            Resolution::Divergence(Action::GarderReferentiel),
            date(3),
        )?;

        let refus = referentiel.resoudre(id, Resolution::Divergence(Action::GarderSource), date(4));

        assert_eq!(refus, Err(ErreurDecision::DejaResolue(id)));
        Ok(())
    }
}

mod lot {
    use super::*;

    /// Cinq fiches liées à une même source, chacune avec une divergence sur
    /// l'organisation.
    fn cinq_divergences() -> Result<(Referentiel, Vec<(FicheId, DecisionId)>), ErreurFiche> {
        let mut referentiel = Referentiel::new();
        let source = referentiel.sources_mut().ajouter("Nextcloud", Version::V3);
        let mut cas = Vec::new();
        for n in 0..5 {
            let mut fiche = Fiche::nouvelle();
            fiche.modifier(Champ::Organisation, &["Ancienne"], date(1))?;
            fiche.lier(Lien::new(source, format!("uid-{n}")));
            let id = referentiel.ajouter(fiche);
            let decision = divergence(
                &mut referentiel,
                id,
                source,
                Champ::Organisation,
                ["Ancienne", "Nouvelle", "Base"],
            );
            cas.push((id, decision));
        }
        Ok((referentiel, cas))
    }

    // [utest->req~resolution-par-lot~1]
    #[test]
    fn le_nombre_de_cas_similaires_s_annonce_avant_la_resolution() -> Resultat {
        let (referentiel, cas) = cinq_divergences()?;
        let premier = cas.first().map(|(_, d)| *d).ok_or("aucun cas")?;

        assert_eq!(referentiel.cas_similaires(premier)?.len(), 4);
        Ok(())
    }

    // [utest->req~resolution-par-lot~1]
    #[test]
    fn garder_la_source_applique_aux_cas_similaires_resout_les_cinq_de_meme() -> Resultat {
        let (mut referentiel, cas) = cinq_divergences()?;
        let premier = cas.first().map(|(_, d)| *d).ok_or("aucun cas")?;

        referentiel.resoudre_avec_cas_similaires(
            premier,
            &Resolution::Divergence(Action::GarderSource),
            date(3),
        )?;

        let organisations: Vec<_> = cas
            .iter()
            .map(|(f, _)| textes(&referentiel, *f, Champ::Organisation))
            .collect();
        assert_eq!(organisations, vec![vec!["Nouvelle".to_owned()]; 5]);
        Ok(())
    }

    #[test]
    fn une_decision_reportee_n_est_pas_un_cas_similaire_en_attente() -> Resultat {
        let (mut referentiel, cas) = cinq_divergences()?;
        let mut ids = cas.iter().map(|(_, d)| *d);
        let (premier, second) = (
            ids.next().ok_or("aucun cas")?,
            ids.next().ok_or("un seul cas")?,
        );
        referentiel.reporter(second)?;

        assert_eq!(referentiel.cas_similaires(premier)?.len(), 3);
        Ok(())
    }
}

mod suppression_detectee {
    use super::*;

    fn suppression(referentiel: &mut Referentiel, fiche: FicheId, source: SourceId) -> DecisionId {
        referentiel.soumettre(
            Nature::Suppression,
            fiche,
            source,
            Versions::default(),
            date(2),
        )
    }

    // [utest->req~issues-suppression-detectee~1]
    #[test]
    fn detacher_garde_la_fiche_sans_lien_avec_la_source() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille")?;
        let id = suppression(&mut referentiel, fiche, source);

        referentiel.resoudre(
            id,
            Resolution::Suppression(IssueSuppression::Detacher),
            date(3),
        )?;

        assert_eq!(
            referentiel.fiche(fiche).map(|f| f.lien(source).is_some()),
            Some(false)
        );
        Ok(())
    }

    #[test]
    fn restaurer_marque_la_fiche_a_reecrire_sur_la_source() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille")?;
        let id = suppression(&mut referentiel, fiche, source);

        referentiel.resoudre(
            id,
            Resolution::Suppression(IssueSuppression::Restaurer),
            date(3),
        )?;

        let a_restaurer = referentiel
            .fiche(fiche)
            .and_then(|f| f.lien(source))
            .map(|l| l.a_restaurer);
        assert_eq!(a_restaurer, Some(true));
        Ok(())
    }

    #[test]
    fn propager_supprime_la_fiche_et_programme_sa_suppression_des_autres_sources() -> Resultat {
        let (mut referentiel, source, fiche) = une_fiche_liee(Champ::NomAffiche, "Camille")?;
        let autre = referentiel.sources_mut().ajouter("Google", Version::V3);
        if let Some(f) = referentiel.fiche_mut(fiche) {
            f.lier(Lien::new(autre, "uid-google"));
        }
        let id = suppression(&mut referentiel, fiche, source);

        referentiel.resoudre(
            id,
            Resolution::Suppression(IssueSuppression::Propager),
            date(3),
        )?;

        assert_eq!(
            (
                referentiel.fiche(fiche).is_none(),
                referentiel
                    .suppressions_en_attente(autre)
                    .collect::<Vec<_>>()
            ),
            (true, vec!["uid-google"])
        );
        Ok(())
    }
}

mod suppression_locale {
    use super::*;

    /// Fiche liée à une source en lecture seule et à une source
    /// bidirectionnelle.
    fn fiche_a_deux_sources() -> Result<(Referentiel, FicheId, SourceId, SourceId), ErreurReglage> {
        let mut referentiel = Referentiel::new();
        let sim = referentiel.sources_mut().ajouter("SIM", Version::V3);
        let nextcloud = referentiel.sources_mut().ajouter("Nextcloud", Version::V3);
        referentiel.sources_mut().regler(
            sim,
            Reglages {
                direction: Direction::LectureSeule,
                ..Reglages::default()
            },
        )?;
        let mut fiche = Fiche::nouvelle();
        fiche.lier(Lien::new(sim, "sim-1"));
        fiche.lier(Lien::new(nextcloud, "nc-1"));
        let id = referentiel.ajouter(fiche);
        Ok((referentiel, id, sim, nextcloud))
    }

    // [utest->req~portee-suppression-locale~1]
    #[test]
    fn seule_la_source_bidirectionnelle_est_proposee() -> Resultat {
        let (referentiel, fiche, _, nextcloud) = fiche_a_deux_sources()?;

        assert_eq!(referentiel.sources_de_suppression(fiche)?, [nextcloud]);
        Ok(())
    }

    #[test]
    fn choisir_une_source_en_lecture_seule_est_refuse_sans_rien_supprimer() -> Resultat {
        let (mut referentiel, fiche, sim, _) = fiche_a_deux_sources()?;

        let refus = referentiel.supprimer_fiche(fiche, &[sim]);

        assert_eq!(
            (refus, referentiel.fiche(fiche).is_some()),
            (Err(ErreurSuppression::SourceNonAutorisee(sim)), true)
        );
        Ok(())
    }

    #[test]
    fn la_copie_d_une_source_non_choisie_n_est_plus_importee() -> Resultat {
        let (mut referentiel, fiche, sim, nextcloud) = fiche_a_deux_sources()?;

        referentiel.supprimer_fiche(fiche, &[nextcloud])?;

        assert!(referentiel.ignorees.contains(&(sim, "sim-1".to_owned())));
        Ok(())
    }
}

mod doublon {
    use super::*;

    #[test]
    fn fusionner_reunit_les_valeurs_dans_la_fiche_existante() -> Resultat {
        let mut referentiel = Referentiel::new();
        let source = referentiel.sources_mut().ajouter("Google", Version::V3);
        let mut existante = Fiche::nouvelle();
        existante.modifier(Champ::NomAffiche, &["Camille Exemple"], date(1))?;
        existante.modifier(Champ::Email, &["a@example.org"], date(1))?;
        let mut importee = Fiche::nouvelle();
        importee.modifier(Champ::NomAffiche, &["Camille Exemple"], date(1))?;
        importee.modifier(Champ::Email, &["b@example.org"], date(1))?;
        importee.lier(Lien::new(source, "g-1"));
        let (e, i) = (
            referentiel.ajouter(existante),
            referentiel.ajouter(importee),
        );
        let id = referentiel.soumettre(
            Nature::DoublonProbable { autre: e },
            i,
            source,
            Versions::default(),
            date(2),
        );

        referentiel.resoudre(id, Resolution::Doublon(IssueDoublon::Fusionner), date(3))?;

        assert_eq!(
            (
                referentiel.fiche(i).is_none(),
                textes(&referentiel, e, Champ::Email)
            ),
            (
                true,
                vec!["a@example.org".to_owned(), "b@example.org".to_owned()]
            )
        );
        Ok(())
    }
}

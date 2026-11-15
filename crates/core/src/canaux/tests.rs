use super::*;
use crate::fiche::Fiche;
use crate::groupes::Filtre;
use crate::temps::Horodatage;

type Resultat = Result<(), Box<dyn std::error::Error>>;

fn date() -> Horodatage {
    Horodatage::depuis_millis(1)
}

/// Fiche nommée avec les numéros et adresses donnés.
fn fiche(
    referentiel: &mut Referentiel,
    nom: &str,
    numeros: &[&str],
    adresses: &[&str],
) -> Result<FicheId, Box<dyn std::error::Error>> {
    let mut f = Fiche::nouvelle();
    f.modifier(Champ::NomAffiche, &[nom], date())?;
    f.modifier(Champ::Telephone, numeros, date())?;
    f.modifier(Champ::Email, adresses, date())?;
    Ok(referentiel.ajouter(f))
}

fn cles(referentiel: &Referentiel, id: FicheId) -> Vec<String> {
    referentiel
        .canaux(id)
        .iter()
        .map(|d| d.canal.cle())
        .collect()
}

mod identifiants {
    use super::*;

    // [utest->req~canaux-par-identifiant~1]
    #[test]
    fn appel_sms_et_email_decoulent_des_numeros_et_adresses() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(
            &mut referentiel,
            "Camille",
            &["06 39 98 00 00"],
            &["camille@example.org"],
        )?;

        assert_eq!(
            cles(&referentiel, camille),
            [
                "appel:+33639980000",
                "sms:+33639980000",
                "email:camille@example.org"
            ]
        );
        Ok(())
    }

    // [utest->req~canaux-par-identifiant~1]
    #[test]
    fn un_canal_signal_disparait_avec_son_numero() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(
            &mut referentiel,
            "Camille",
            &["+33639980000", "+33639980001"],
            &[],
        )?;
        referentiel.declarer_canal(camille, TypeCanal::Signal, "06 39 98 00 00")?;

        referentiel
            .fiche_mut(camille)
            .ok_or("fiche attendue")?
            .modifier(Champ::Telephone, &["+33639980001"], date())?;

        assert!(!referentiel.dispose_de(camille, &TypeCanal::Signal));
        Ok(())
    }

    // [utest->req~canaux-par-identifiant~1]
    #[test]
    fn une_messagerie_ne_se_rattache_pas_a_un_numero_absent_de_la_fiche() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(&mut referentiel, "Camille", &["+33639980000"], &[])?;

        let refus = referentiel.declarer_canal(camille, TypeCanal::WhatsApp, "+33611111111");

        assert_eq!(
            refus,
            Err(ErreurCanal::IdentifiantAbsent("+33611111111".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn telegram_et_matrix_se_rattachent_a_un_pseudonyme() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(&mut referentiel, "Camille", &[], &[])?;
        referentiel.declarer_canal(camille, TypeCanal::Telegram, "@camille")?;
        referentiel.declarer_canal(camille, TypeCanal::Matrix, "@camille:example.org")?;

        assert_eq!(
            cles(&referentiel, camille),
            ["telegram:@camille", "matrix:@camille:example.org"]
        );
        Ok(())
    }
}

mod classements {
    use super::*;

    // [utest->req~classements-pro-perso~2]
    #[test]
    fn seul_le_classement_perso_renseigne_vaut_pour_le_pro() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(
            &mut referentiel,
            "Camille",
            &["+33639980000"],
            &["camille@example.org"],
        )?;
        referentiel.declarer_canal(camille, TypeCanal::Signal, "+33639980000")?;
        referentiel.classer_canaux(
            camille,
            Contexte::Perso,
            vec![
                "signal:+33639980000".to_owned(),
                "appel:+33639980000".to_owned(),
            ],
        )?;

        let prefere = referentiel
            .canal_prefere(camille, Contexte::Pro)
            .map(|c| c.cle());

        assert_eq!(prefere.as_deref(), Some("signal:+33639980000"));
        Ok(())
    }

    // [utest->req~classements-pro-perso~2]
    #[test]
    fn chaque_contexte_renseigne_garde_son_propre_classement() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(
            &mut referentiel,
            "Camille",
            &["+33639980000"],
            &["camille@example.org"],
        )?;
        referentiel.classer_canaux(
            camille,
            Contexte::Pro,
            vec!["email:camille@example.org".to_owned()],
        )?;
        referentiel.classer_canaux(
            camille,
            Contexte::Perso,
            vec!["sms:+33639980000".to_owned()],
        )?;

        let preferes: Vec<Option<String>> = [Contexte::Pro, Contexte::Perso]
            .into_iter()
            .map(|c| referentiel.canal_prefere(camille, c).map(|c| c.cle()))
            .collect();

        assert_eq!(
            preferes,
            [
                Some("email:camille@example.org".to_owned()),
                Some("sms:+33639980000".to_owned())
            ]
        );
        Ok(())
    }

    #[test]
    fn un_canal_disparu_sort_du_classement() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(
            &mut referentiel,
            "Camille",
            &["+33639980000"],
            &["camille@example.org"],
        )?;
        referentiel.classer_canaux(
            camille,
            Contexte::Perso,
            vec![
                "sms:+33639980000".to_owned(),
                "email:camille@example.org".to_owned(),
            ],
        )?;

        referentiel
            .fiche_mut(camille)
            .ok_or("fiche attendue")?
            .modifier(Champ::Telephone, &[], date())?;

        assert_eq!(
            referentiel.classement(camille, Contexte::Perso),
            ["email:camille@example.org"]
        );
        Ok(())
    }
}

mod notes {
    use super::*;

    // [utest->req~note-par-canal~1]
    #[test]
    fn une_note_s_attache_a_un_canal() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(&mut referentiel, "Camille", &["+33639980000"], &[])?;

        referentiel.annoter_canal(camille, "appel:+33639980000", "après 18 h")?;

        let notes: Vec<Option<String>> = referentiel
            .canaux(camille)
            .into_iter()
            .map(|d| d.note)
            .collect();
        assert_eq!(notes, [Some("après 18 h".to_owned()), None]);
        Ok(())
    }

    // [utest->req~note-par-canal~1]
    #[test]
    fn une_note_vide_retire_la_note() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = fiche(&mut referentiel, "Camille", &["+33639980000"], &[])?;
        referentiel.annoter_canal(camille, "appel:+33639980000", "après 18 h")?;

        referentiel.annoter_canal(camille, "appel:+33639980000", "  ")?;

        assert!(referentiel.canaux_de(camille).is_none());
        Ok(())
    }
}

mod filtre {
    use super::*;

    // [utest->req~filtre-par-canal~1]
    #[test]
    fn le_filtre_par_canal_ne_garde_que_les_fiches_qui_en_disposent() -> Resultat {
        let mut referentiel = Referentiel::new();
        let alix = fiche(&mut referentiel, "Alix", &["+33639980000"], &[])?;
        let _basile = fiche(&mut referentiel, "Basile", &["+33639980001"], &[])?;
        referentiel.declarer_canal(alix, TypeCanal::Signal, "+33639980000")?;
        let filtre = Filtre {
            canal: Some("signal".to_owned()),
            ..Filtre::default()
        };

        assert_eq!(referentiel.filtrer(&filtre), [alix]);
        Ok(())
    }

    // [utest->req~filtre-par-canal~1]
    #[test]
    fn le_filtre_par_e_mail_garde_les_fiches_qui_ont_une_adresse() -> Resultat {
        let mut referentiel = Referentiel::new();
        let _alix = fiche(&mut referentiel, "Alix", &["+33639980000"], &[])?;
        let basile = fiche(&mut referentiel, "Basile", &[], &["basile@example.org"])?;
        let filtre = Filtre {
            canal: Some("email".to_owned()),
            ..Filtre::default()
        };

        assert_eq!(referentiel.filtrer(&filtre), [basile]);
        Ok(())
    }
}

#[test]
fn les_canaux_d_une_fiche_supprimee_disparaissent() -> Resultat {
    let mut referentiel = Referentiel::new();
    let camille = fiche(&mut referentiel, "Camille", &["+33639980000"], &[])?;
    referentiel.annoter_canal(camille, "appel:+33639980000", "note")?;

    referentiel.supprimer_fiche(camille, &[])?;

    assert!(referentiel.canaux_de(camille).is_none());
    Ok(())
}

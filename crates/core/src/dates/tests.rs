use super::*;
use crate::fiche::Fiche;

type Resultat = Result<(), Box<dyn std::error::Error>>;

fn horodatage(annee: i32, mois: u8, jour: u8) -> Horodatage {
    Horodatage::depuis_millis(jours_depuis_epoque(annee, mois, jour) * 86_400_000 + 3_600_000)
}

fn nouvelle_fiche(
    referentiel: &mut Referentiel,
    nom: &str,
) -> Result<FicheId, Box<dyn std::error::Error>> {
    let mut fiche = Fiche::nouvelle();
    fiche.modifier(Champ::NomAffiche, &[nom], Horodatage::depuis_millis(1))?;
    Ok(referentiel.ajouter(fiche))
}

mod lecture {
    use super::*;

    #[test]
    fn les_formes_courantes_se_lisent() {
        let lues: Vec<Option<String>> = [
            "1980-05-17",
            "19800517",
            "--05-17",
            "--0517",
            "17/05/1980",
            "17/05",
            "1604-05-17",
        ]
        .iter()
        .map(|t| Date::lire(t).map(|d| d.texte()))
        .collect();

        assert_eq!(
            lues,
            [
                Some("1980-05-17"),
                Some("1980-05-17"),
                Some("--05-17"),
                Some("--05-17"),
                Some("1980-05-17"),
                Some("--05-17"),
                Some("--05-17")
            ]
            .map(|t| t.map(str::to_owned))
        );
    }

    #[test]
    fn une_date_impossible_est_refusee() {
        assert_eq!(
            [
                Date::lire("31/02/2020"),
                Date::lire("29/02/2023"),
                Date::lire("--1305")
            ],
            [None, None, None]
        );
    }

    #[test]
    fn le_29_fevrier_sans_annee_est_admis() {
        assert!(Date::lire("--02-29").is_some());
    }

    #[test]
    fn le_jour_civil_se_calcule_depuis_l_epoque() {
        assert_eq!(
            [
                jour_civil(Horodatage::depuis_millis(0)),
                jour_civil(horodatage(2026, 11, 15))
            ],
            [(1970, 1, 1), (2026, 11, 15)]
        );
    }
}

mod types {
    use super::*;

    // [utest->req~types-de-dates~1]
    #[test]
    fn un_anniversaire_saisi_sans_annee_se_relit_sans_annee() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = nouvelle_fiche(&mut referentiel, "Camille")?;

        referentiel.ajouter_date(
            camille,
            TypeDate::Naissance,
            "17/05",
            Horodatage::depuis_millis(2),
        )?;

        let dates = referentiel.dates(camille);
        assert_eq!(
            dates,
            [DateDeFiche {
                type_date: TypeDate::Naissance,
                date: Date {
                    annee: None,
                    mois: 5,
                    jour: 17
                }
            }]
        );
        Ok(())
    }

    // [utest->req~types-de-dates~1]
    #[test]
    fn une_fiche_porte_chaque_type_de_date() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = nouvelle_fiche(&mut referentiel, "Camille")?;
        let date = Horodatage::depuis_millis(2);
        referentiel.ajouter_date(camille, TypeDate::Naissance, "1980-05-17", date)?;
        referentiel.ajouter_date(camille, TypeDate::Mariage, "2005-06-11", date)?;
        referentiel.ajouter_date(camille, TypeDate::Fete, "--07-22", date)?;
        referentiel.ajouter_date(
            camille,
            TypeDate::depuis_texte("Arrivée en France"),
            "--09-01",
            date,
        )?;

        let libelles: Vec<String> = referentiel
            .dates(camille)
            .iter()
            .map(|d| format!("{} {}", d.type_date.libelle(), d.date.texte()))
            .collect();

        assert_eq!(
            libelles,
            [
                "Anniversaire 1980-05-17",
                "Anniversaire de mariage 2005-06-11",
                "Fête du prénom --07-22",
                "Arrivée en France --09-01"
            ]
        );
        Ok(())
    }

    // [utest->req~types-de-dates~1]
    #[test]
    fn une_date_invalide_est_refusee_sans_rien_modifier() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = nouvelle_fiche(&mut referentiel, "Camille")?;

        let refus = referentiel.ajouter_date(
            camille,
            TypeDate::Mariage,
            "31/02",
            Horodatage::depuis_millis(2),
        );

        assert_eq!(
            (refus, referentiel.dates(camille).len()),
            (Err(ErreurDate::Invalide("31/02".to_owned())), 0)
        );
        Ok(())
    }

    #[test]
    fn retirer_une_date_la_retire() -> Resultat {
        let mut referentiel = Referentiel::new();
        let camille = nouvelle_fiche(&mut referentiel, "Camille")?;
        referentiel.ajouter_date(
            camille,
            TypeDate::Mariage,
            "2005-06-11",
            Horodatage::depuis_millis(2),
        )?;
        let mariage = referentiel
            .dates(camille)
            .first()
            .cloned()
            .ok_or("date attendue")?;

        referentiel.retirer_date(camille, &mariage)?;

        assert!(referentiel.dates(camille).is_empty());
        Ok(())
    }
}

mod fetes {
    use super::*;

    // [utest->req~fete-du-prenom-proposee~1]
    #[test]
    fn la_fete_d_eric_est_proposee_le_18_mai() -> Resultat {
        let mut referentiel = Referentiel::new();
        let eric = nouvelle_fiche(&mut referentiel, "Éric Lemesre")?;

        let proposees: Vec<String> = referentiel
            .fetes_proposees(eric)
            .iter()
            .map(Date::texte)
            .collect();

        assert_eq!(proposees, ["--05-18"]);
        Ok(())
    }

    // [utest->req~fete-du-prenom-proposee~1]
    #[test]
    fn le_prenom_du_nom_structure_prime_sur_le_nom_affiche() -> Resultat {
        let mut referentiel = Referentiel::new();
        let id = nouvelle_fiche(&mut referentiel, "Docteur Martin")?;
        referentiel
            .fiche_mut(id)
            .ok_or("fiche attendue")?
            .modifier(
                Champ::NomStructure,
                &["Martin;Nicolas;;;"],
                Horodatage::depuis_millis(1),
            )?;

        let proposees: Vec<String> = referentiel
            .fetes_proposees(id)
            .iter()
            .map(Date::texte)
            .collect();

        assert_eq!(proposees, ["--12-06"]);
        Ok(())
    }

    // [utest->req~fete-du-prenom-proposee~1]
    #[test]
    fn une_fiche_qui_a_sa_fete_n_en_recoit_pas_de_proposition() -> Resultat {
        let mut referentiel = Referentiel::new();
        let eric = nouvelle_fiche(&mut referentiel, "Éric")?;
        referentiel.ajouter_date(
            eric,
            TypeDate::Fete,
            "--05-19",
            Horodatage::depuis_millis(2),
        )?;

        assert!(referentiel.fetes_proposees(eric).is_empty());
        Ok(())
    }

    #[test]
    fn un_prenom_compose_se_rabat_sur_son_premier_element() {
        let jours: Vec<String> = fetes_du_prenom("Nicolas-Zyxw")
            .iter()
            .map(Date::texte)
            .collect();

        assert_eq!(jours, ["--12-06"]);
    }

    #[test]
    fn un_prenom_absent_du_calendrier_n_a_pas_de_proposition() {
        assert!(fetes_du_prenom("Zyxwvut").is_empty());
    }
}

mod a_venir {
    use super::*;

    #[test]
    fn les_dates_a_venir_vont_de_la_plus_proche_a_la_plus_lointaine() -> Resultat {
        let mut referentiel = Referentiel::new();
        let (a, b) = (
            nouvelle_fiche(&mut referentiel, "Alix")?,
            nouvelle_fiche(&mut referentiel, "Basile")?,
        );
        let date = Horodatage::depuis_millis(2);
        referentiel.ajouter_date(a, TypeDate::Naissance, "1980-01-10", date)?;
        referentiel.ajouter_date(b, TypeDate::Fete, "--11-20", date)?;
        referentiel.ajouter_date(b, TypeDate::Naissance, "--11-15", date)?;

        let a_venir: Vec<(u32, Option<u32>)> = referentiel
            .a_venir(horodatage(2026, 11, 15), &Filtre::default())
            .iter()
            .map(|d| (d.dans_jours, d.annees))
            .collect();

        assert_eq!(a_venir, [(0, None), (5, None), (56, Some(47))]);
        Ok(())
    }

    #[test]
    fn les_dates_a_venir_se_filtrent_par_groupe() -> Resultat {
        let mut referentiel = Referentiel::new();
        let (a, b) = (
            nouvelle_fiche(&mut referentiel, "Alix")?,
            nouvelle_fiche(&mut referentiel, "Basile")?,
        );
        let date = Horodatage::depuis_millis(2);
        referentiel.ajouter_date(a, TypeDate::Naissance, "--01-10", date)?;
        referentiel.ajouter_date(b, TypeDate::Naissance, "--01-11", date)?;
        referentiel.ranger(b, &["Famille"], date)?;
        let filtre = Filtre {
            groupes: vec!["Famille".to_owned()],
            ..Filtre::default()
        };

        let fiches: Vec<FicheId> = referentiel
            .a_venir(horodatage(2026, 11, 15), &filtre)
            .iter()
            .map(|d| d.fiche)
            .collect();

        assert_eq!(fiches, [b]);
        Ok(())
    }

    #[test]
    fn un_29_fevrier_tombe_le_28_une_annee_non_bissextile() {
        let (annee, jours) = prochaine(
            Date {
                annee: None,
                mois: 2,
                jour: 29,
            },
            (2027, 2, 1),
        );

        assert_eq!((annee, jours), (2027, 27));
    }
}

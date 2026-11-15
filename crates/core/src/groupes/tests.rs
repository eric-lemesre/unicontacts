use super::*;
use crate::fiche::Fiche;

type Resultat = Result<(), Box<dyn std::error::Error>>;

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

/// Référentiel d'une fiche par entrée (nom, groupes), et leurs identifiants.
fn referentiel(
    fiches: &[(&str, &[&str])],
) -> Result<(Referentiel, Vec<FicheId>), Box<dyn std::error::Error>> {
    let mut referentiel = Referentiel::new();
    let mut ids = Vec::new();
    for (nom, groupes) in fiches {
        let mut fiche = Fiche::nouvelle();
        fiche.modifier(Champ::NomAffiche, &[nom], date(1))?;
        let id = referentiel.ajouter(fiche);
        referentiel.ranger(id, groupes, date(1))?;
        ids.push(id);
    }
    Ok((referentiel, ids))
}

/// Noms des fiches retenues par le filtre, triés.
fn retenues(referentiel: &Referentiel, filtre: &Filtre) -> Vec<String> {
    let mut noms: Vec<String> = referentiel
        .filtrer(filtre)
        .into_iter()
        .filter_map(|id| referentiel.fiche(id))
        .flat_map(|f| f.textes(Champ::NomAffiche))
        .collect();
    noms.sort();
    noms
}

fn filtre(groupes: &[&str]) -> Filtre {
    Filtre {
        groupes: groupes.iter().map(|g| (*g).to_owned()).collect(),
        ..Filtre::default()
    }
}

fn voisinage() -> Result<Referentiel, Box<dyn std::error::Error>> {
    Ok(referentiel(&[
        ("Alix", &["Famille"]),
        ("Basile", &["Voisins"]),
        ("Camille", &["Famille", "Voisins"]),
        ("Dominique", &["Famille/Cousins"]),
        ("Eden", &[]),
    ])?
    .0)
}

mod appartenance {
    use super::*;

    // [utest->req~appartenance-multiple~1]
    #[test]
    fn une_fiche_appartient_a_plusieurs_groupes() -> Resultat {
        let (referentiel, fiches) =
            referentiel(&[("Camille", &["Famille", "Voisins", "Travail/Clients"])])?;
        let camille = fiches.first().copied().ok_or("fiche attendue")?;

        assert_eq!(
            referentiel.groupes_de(camille),
            ["Famille", "Voisins", "Travail/Clients"]
        );
        Ok(())
    }

    #[test]
    fn un_chemin_se_normalise_et_ne_se_repete_pas() -> Resultat {
        let (referentiel, fiches) = referentiel(&[(
            "Camille",
            &[" Travail /  Clients ", "Travail/Clients", "//"],
        )])?;
        let camille = fiches.first().copied().ok_or("fiche attendue")?;

        assert_eq!(referentiel.groupes_de(camille), ["Travail/Clients"]);
        Ok(())
    }
}

mod hierarchie {
    use super::*;

    // [utest->req~groupes-hierarchiques~1]
    #[test]
    fn un_groupe_ne_devient_pas_enfant_de_son_propre_enfant() -> Resultat {
        let mut referentiel = Referentiel::new();
        referentiel.creer_groupe("A/B")?;

        let refus = referentiel.deplacer_groupe("A", Some("A/B"), date(2));

        assert_eq!(
            refus,
            Err(ErreurGroupe::Cycle {
                groupe: "A".to_owned(),
                parent: "A/B".to_owned()
            })
        );
        Ok(())
    }

    // [utest->req~groupes-hierarchiques~1]
    #[test]
    fn un_groupe_a_un_parent_facultatif_et_en_change_avec_ses_sous_groupes() -> Resultat {
        let mut referentiel = Referentiel::new();
        referentiel.creer_groupe("Clients/Grands comptes")?;
        referentiel.creer_groupe("Travail")?;

        referentiel.deplacer_groupe("Clients", Some("Travail"), date(2))?;

        assert_eq!(
            referentiel.groupes(),
            [
                "Travail",
                "Travail/Clients",
                "Travail/Clients/Grands comptes"
            ]
        );
        Ok(())
    }

    #[test]
    fn un_groupe_sans_parent_remonte_au_premier_niveau() -> Resultat {
        let mut referentiel = Referentiel::new();
        referentiel.creer_groupe("Travail/Clients")?;

        referentiel.deplacer_groupe("Travail/Clients", None, date(2))?;

        assert!(referentiel.groupes().contains(&"Clients".to_owned()));
        Ok(())
    }

    #[test]
    fn un_groupe_existant_ne_se_cree_pas_deux_fois() -> Resultat {
        let mut referentiel = Referentiel::new();
        referentiel.creer_groupe("Travail/Clients")?;

        assert_eq!(
            referentiel.creer_groupe("Travail"),
            Err(ErreurGroupe::Existant("Travail".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn supprimer_un_groupe_libere_ses_fiches_sans_les_supprimer() -> Resultat {
        let (mut referentiel, fiches) =
            referentiel(&[("Dominique", &["Famille/Cousins", "Voisins"])])?;
        let dominique = fiches.first().copied().ok_or("fiche attendue")?;

        referentiel.supprimer_groupe("Famille");

        assert_eq!(referentiel.groupes_de(dominique), ["Voisins"]);
        Ok(())
    }
}

mod filtre {
    use super::*;

    // [utest->req~filtre-par-groupes~2]
    #[test]
    fn par_defaut_le_filtre_retient_les_fiches_de_l_un_des_groupes() -> Resultat {
        let referentiel = voisinage()?;

        assert_eq!(
            retenues(&referentiel, &filtre(&["Famille", "Voisins"])),
            ["Alix", "Basile", "Camille", "Dominique"]
        );
        Ok(())
    }

    // [utest->req~filtre-par-groupes~2]
    #[test]
    fn en_intersection_seules_les_fiches_des_deux_groupes_sont_retenues() -> Resultat {
        let referentiel = voisinage()?;
        let filtre = Filtre {
            combinaison: Combinaison::Intersection,
            ..filtre(&["Famille", "Voisins"])
        };

        assert_eq!(retenues(&referentiel, &filtre), ["Camille"]);
        Ok(())
    }

    // [utest->req~filtre-par-groupes~2]
    #[test]
    fn sans_les_sous_groupes_un_cousin_n_est_pas_de_la_famille() -> Resultat {
        let referentiel = voisinage()?;
        let filtre = Filtre {
            sous_groupes: false,
            ..filtre(&["Famille"])
        };

        assert_eq!(retenues(&referentiel, &filtre), ["Alix", "Camille"]);
        Ok(())
    }

    // [utest->req~filtre-par-groupes~2]
    #[test]
    fn le_filtre_se_combine_avec_la_recherche_sans_tenir_compte_des_accents() -> Resultat {
        let mut referentiel = voisinage()?;
        let mut fiche = Fiche::nouvelle();
        fiche.modifier(Champ::NomAffiche, &["Céline"], date(1))?;
        let celine = referentiel.ajouter(fiche);
        referentiel.ranger(celine, &["Famille"], date(1))?;
        let filtre = Filtre {
            recherche: "CEL".to_owned(),
            ..filtre(&["Famille"])
        };

        assert_eq!(retenues(&referentiel, &filtre), ["Céline"]);
        Ok(())
    }

    #[test]
    fn un_sous_groupe_n_est_pas_un_groupe_de_meme_prefixe() {
        assert!(!est_dans("Familles", "Famille"));
    }
}

mod renommage {
    use super::*;

    // [utest->req~renommage-groupe-operation-unique~1]
    #[test]
    fn renommer_un_parent_renomme_le_chemin_de_toutes_ses_fiches() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&[
            ("Alix", &["Travail/Clients"]),
            ("Basile", &["Travail", "Famille"]),
        ])?;

        referentiel.renommer_groupe("Travail", "Boulot", date(2))?;

        let groupes: Vec<Vec<String>> = fiches.iter().map(|f| referentiel.groupes_de(*f)).collect();
        assert_eq!(groupes, [vec!["Boulot/Clients"], vec!["Boulot", "Famille"]]);
        Ok(())
    }

    #[test]
    fn renommer_vers_un_groupe_existant_est_refuse() -> Resultat {
        let (mut referentiel, _) = referentiel(&[("Alix", &["Travail", "Famille"])])?;

        let refus = referentiel.renommer_groupe("Travail", "Famille", date(2));

        assert_eq!(refus, Err(ErreurGroupe::Existant("Famille".to_owned())));
        Ok(())
    }
}

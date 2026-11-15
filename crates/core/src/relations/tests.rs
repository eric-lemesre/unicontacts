use super::*;
use crate::fiche::Fiche;

type Resultat = Result<(), Box<dyn std::error::Error>>;

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

/// Référentiel de fiches nommées, et leurs identifiants dans l'ordre.
fn referentiel(noms: &[&str]) -> Result<(Referentiel, Vec<FicheId>), Box<dyn std::error::Error>> {
    let mut referentiel = Referentiel::new();
    let mut ids = Vec::new();
    for nom in noms {
        let mut fiche = Fiche::nouvelle();
        fiche.modifier(Champ::NomAffiche, &[nom], date(1))?;
        ids.push(referentiel.ajouter(fiche));
    }
    Ok((referentiel, ids))
}

fn ids(noms: usize, ids: &[FicheId]) -> Result<Vec<FicheId>, String> {
    (ids.len() == noms)
        .then(|| ids.to_vec())
        .ok_or_else(|| "fiches manquantes".to_owned())
}

mod fiche_moi {
    use super::*;

    // [utest->req~fiche-moi~1]
    #[test]
    fn la_premiere_fiche_moi_perd_cette_qualite_quand_une_autre_est_designee() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Eric", "Camille"])?;
        let [eric, camille] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        referentiel.designer_moi(Some(eric))?;

        referentiel.designer_moi(Some(camille))?;

        assert_eq!(referentiel.moi(), Some(camille));
        Ok(())
    }

    // [utest->req~fiche-moi~1]
    #[test]
    fn une_fiche_inconnue_n_est_pas_designee() -> Resultat {
        let (mut referentiel, _) = referentiel(&["Eric"])?;
        let inconnue = Fiche::nouvelle().id();

        let resultat = referentiel.designer_moi(Some(inconnue));

        assert_eq!(resultat, Err(ErreurRelation::FicheInconnue(inconnue)));
        Ok(())
    }

    #[test]
    fn la_fiche_moi_supprimee_n_est_plus_designee() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Eric"])?;
        referentiel.designer_moi(fiches.first().copied())?;
        let eric = fiches.first().copied().ok_or("fiche attendue")?;

        referentiel.supprimer_fiche(eric, &[])?;

        assert_eq!(referentiel.moi(), None);
        Ok(())
    }
}

mod liens {
    use super::*;

    // [utest->req~liens-types~1]
    #[test]
    fn deux_fiches_quelconques_se_lient_par_un_type_predefini() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Camille", "Dominique"])?;
        let [camille, dominique] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };

        referentiel.lier(camille, TypeRelation::Collegue, dominique, date(2))?;

        assert_eq!(
            referentiel.relations(camille),
            [Relation {
                type_relation: TypeRelation::Collegue,
                fiche: dominique
            }]
        );
        Ok(())
    }

    // [utest->req~liens-types~1]
    #[test]
    fn un_type_personnalise_garde_son_libelle() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Camille", "Dominique"])?;
        let [camille, dominique] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };

        referentiel.lier(
            camille,
            TypeRelation::depuis_texte("Parrain ; marraine"),
            dominique,
            date(2),
        )?;

        let types: Vec<String> = referentiel
            .relations(camille)
            .iter()
            .map(|r| r.type_relation.libelle().to_owned())
            .collect();
        assert_eq!(types, ["Parrain ; marraine"]);
        Ok(())
    }

    #[test]
    fn un_libelle_de_type_predefini_designe_ce_type() {
        assert_eq!(
            TypeRelation::depuis_texte("frère/SŒUR"),
            TypeRelation::FrereSoeur
        );
    }

    #[test]
    fn une_fiche_ne_se_lie_pas_a_elle_meme() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Camille"])?;
        let camille = fiches.first().copied().ok_or("fiche attendue")?;

        let resultat = referentiel.lier(camille, TypeRelation::Ami, camille, date(2));

        assert_eq!(resultat, Err(ErreurRelation::LienSurSoi(camille)));
        Ok(())
    }

    #[test]
    fn un_lien_deja_present_n_est_pas_duplique() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Camille", "Dominique"])?;
        let [camille, dominique] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        referentiel.lier(camille, TypeRelation::Medecin, dominique, date(2))?;

        referentiel.lier(camille, TypeRelation::Medecin, dominique, date(3))?;

        assert_eq!(referentiel.relations(camille).len(), 1);
        Ok(())
    }

    #[test]
    fn le_texte_d_une_relation_se_relit_a_l_identique() {
        let relation = Relation {
            type_relation: TypeRelation::Personnalise(r"a;b\c".to_owned()),
            fiche: Fiche::nouvelle().id(),
        };

        assert_eq!(Relation::depuis_texte(&relation.texte()), Some(relation));
    }

    #[test]
    fn les_liens_vers_une_fiche_supprimee_disparaissent() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Camille", "Dominique"])?;
        let [camille, dominique] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        referentiel.lier(camille, TypeRelation::Ami, dominique, date(2))?;

        referentiel.supprimer_fiche(dominique, &[])?;

        assert!(referentiel.relations(camille).is_empty());
        Ok(())
    }
}

mod reciproques {
    use super::*;

    // [utest->req~reciproques-automatiques~1]
    #[test]
    fn le_parent_d_une_fiche_a_cette_fiche_pour_enfant() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };

        referentiel.lier(a, TypeRelation::Parent, b, date(2))?;

        assert_eq!(
            referentiel.relations(b),
            [Relation {
                type_relation: TypeRelation::Enfant,
                fiche: a
            }]
        );
        Ok(())
    }

    // [utest->req~reciproques-automatiques~1]
    #[test]
    fn un_conjoint_est_conjoint_en_retour() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };

        referentiel.lier(a, TypeRelation::Conjoint, b, date(2))?;

        assert_eq!(
            referentiel.relations(b),
            [Relation {
                type_relation: TypeRelation::Conjoint,
                fiche: a
            }]
        );
        Ok(())
    }

    // [utest->req~reciproques-automatiques~1]
    #[test]
    fn un_type_personnalise_sans_inverse_ne_cree_aucune_reciproque() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };

        referentiel.lier(
            a,
            TypeRelation::depuis_texte("Voisin de palier"),
            b,
            date(2),
        )?;

        assert!(referentiel.relations(b).is_empty());
        Ok(())
    }

    // [utest->req~reciproques-automatiques~1]
    #[test]
    fn un_type_personnalise_prend_l_inverse_declare() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        let parrain = TypeRelation::depuis_texte("Parrain");
        let filleul = TypeRelation::depuis_texte("Filleul");
        referentiel.inverses_mut().declarer(&parrain, &filleul)?;

        referentiel.lier(a, parrain, b, date(2))?;

        assert_eq!(
            referentiel.relations(b),
            [Relation {
                type_relation: filleul,
                fiche: a
            }]
        );
        Ok(())
    }

    #[test]
    fn un_patient_declare_inverse_de_medecin_donne_sa_reciproque() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        let patient = TypeRelation::depuis_texte("Patient");
        referentiel
            .inverses_mut()
            .declarer(&TypeRelation::Medecin, &patient)?;

        referentiel.lier(a, TypeRelation::Medecin, b, date(2))?;

        assert_eq!(
            referentiel.relations(b),
            [Relation {
                type_relation: patient,
                fiche: a
            }]
        );
        Ok(())
    }

    #[test]
    fn un_type_a_inverse_fixe_refuse_un_inverse_declare() {
        let mut inverses = Inverses::default();

        let resultat =
            inverses.declarer(&TypeRelation::Parent, &TypeRelation::depuis_texte("Tuteur"));

        assert_eq!(
            resultat,
            Err(ErreurRelation::InverseFixe("Parent".to_owned()))
        );
    }

    #[test]
    fn retirer_un_lien_retire_sa_reciproque() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        referentiel.lier(a, TypeRelation::Parent, b, date(2))?;

        referentiel.delier(a, &TypeRelation::Parent, b)?;

        assert!(referentiel.relations(b).is_empty());
        Ok(())
    }
}

mod identifiant_maitre {
    use super::*;

    // [utest->req~lien-par-identifiant-maitre~1]
    #[test]
    fn le_lien_designe_toujours_la_fiche_renommee() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        referentiel.lier(a, TypeRelation::Ami, b, date(2))?;

        referentiel.fiche_mut(b).ok_or("fiche attendue")?.modifier(
            Champ::NomAffiche,
            &["Basile Renommé"],
            date(3),
        )?;

        let liees: Vec<FicheId> = referentiel.relations(a).iter().map(|r| r.fiche).collect();
        assert_eq!(liees, [b]);
        Ok(())
    }

    // [utest->req~lien-par-identifiant-maitre~1]
    #[test]
    fn la_valeur_enregistree_porte_l_identifiant_et_non_le_nom() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Alix", "Basile"])?;
        let [a, b] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };

        referentiel.lier(a, TypeRelation::Ami, b, date(2))?;

        let textes = referentiel
            .fiche(a)
            .ok_or("fiche attendue")?
            .textes(Champ::Relation);
        assert_eq!(textes, [format!("ami;{b}")]);
        Ok(())
    }
}

mod proches {
    use super::*;

    #[test]
    fn les_proches_sont_les_fiches_liees_a_la_fiche_moi() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Eric", "Camille", "Dominique"])?;
        let [eric, camille, dominique] = ids(3, &fiches)?[..] else {
            return Err("trois fiches attendues".into());
        };
        referentiel.designer_moi(Some(eric))?;
        referentiel.lier(eric, TypeRelation::Conjoint, camille, date(2))?;
        referentiel.lier(dominique, TypeRelation::Ami, camille, date(2))?;

        assert_eq!(referentiel.proches(), [camille]);
        Ok(())
    }

    #[test]
    fn sans_fiche_moi_il_n_y_a_pas_de_proches() -> Resultat {
        let (mut referentiel, fiches) = referentiel(&["Camille", "Dominique"])?;
        let [camille, dominique] = ids(2, &fiches)?[..] else {
            return Err("deux fiches attendues".into());
        };
        referentiel.lier(camille, TypeRelation::Ami, dominique, date(2))?;

        assert!(referentiel.proches().is_empty());
        Ok(())
    }
}

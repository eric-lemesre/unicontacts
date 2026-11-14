use super::*;
use crate::fiche::{ErreurFiche, Fiche};

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

/// Lit un texte qui ne contient qu'une carte.
fn une_carte(texte: &str) -> Result<Carte, String> {
    let mut cartes = lire(texte).map_err(|e| e.to_string())?;
    match (cartes.pop(), cartes.is_empty()) {
        (Some(carte), true) => Ok(carte),
        _ => Err("une seule carte attendue".to_owned()),
    }
}

/// Fiche reçue d'une source : les valeurs interprétées de la carte.
fn fiche_de(carte: &Carte, source: SourceId) -> Result<Fiche, ErreurFiche> {
    let mut fiche = Fiche::nouvelle();
    for (champ, valeurs) in carte.valeurs(source, date(1)) {
        fiche.definir(champ, valeurs)?;
    }
    Ok(fiche)
}

const INCONNUE: &str =
    "X-PROPRIETAIRE;X-PARAM=1:données propres au serveur\\, à rendre à l'octet p\r\n rès";

fn carte_avec_inconnue() -> String {
    format!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:u1\r\nFN:Sacha Inconnu\r\n{INCONNUE}\r\nNOTE:note initiale\r\nEND:VCARD\r\n"
    )
}

mod lecture {
    use super::*;

    #[test]
    fn une_ligne_repliee_est_depliee() -> Result<(), String> {
        let carte =
            une_carte("BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:début\r\n  et suite\r\nEND:VCARD\r\n")?;

        assert_eq!(
            carte.proprietes.first().map(|p| p.valeur.as_str()),
            Some("début et suite")
        );
        Ok(())
    }

    #[test]
    fn les_fins_de_ligne_lf_sont_admises() -> Result<(), String> {
        let carte = une_carte("BEGIN:VCARD\nVERSION:3.0\nFN:Camille\nEND:VCARD\n")?;

        assert_eq!(carte.version, Version::V3);
        Ok(())
    }

    #[test]
    fn le_groupe_est_separe_du_nom() -> Result<(), String> {
        let carte =
            une_carte("BEGIN:VCARD\r\nVERSION:3.0\r\nitem1.X-ABLabel:Collègue\r\nEND:VCARD\r\n")?;

        let propriete = carte.proprietes.first();
        assert_eq!(
            propriete.map(|p| (p.groupe.as_deref(), p.nom.as_str())),
            Some((Some("item1"), "X-ABLABEL"))
        );
        Ok(())
    }

    #[test]
    fn un_parametre_entre_guillemets_peut_contenir_deux_points() -> Result<(), String> {
        let carte =
            une_carte("BEGIN:VCARD\r\nVERSION:4.0\r\nX-A;X-P=\"a:b;c\":valeur\r\nEND:VCARD\r\n")?;

        let propriete = carte.proprietes.first();
        assert_eq!(
            propriete.map(|p| (p.parametres.clone(), p.valeur.as_str())),
            Some((
                vec![Parametre {
                    nom: "X-P".to_owned(),
                    valeurs: vec!["a:b;c".to_owned()]
                }],
                "valeur"
            ))
        );
        Ok(())
    }

    #[test]
    fn un_parametre_sans_nom_vaut_type() -> Result<(), String> {
        let carte =
            une_carte("BEGIN:VCARD\r\nVERSION:3.0\r\nTEL;CELL;HOME:0639980002\r\nEND:VCARD\r\n")?;

        assert_eq!(
            carte.proprietes.first().map(Propriete::types),
            Some(vec!["cell".to_owned(), "home".to_owned()])
        );
        Ok(())
    }

    #[test]
    fn les_types_se_lisent_en_liste_ou_repetes() -> Result<(), String> {
        let carte = une_carte(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nTEL;TYPE=\"cell,voice\";TYPE=WORK:x\r\nEND:VCARD\r\n",
        )?;

        assert_eq!(
            carte.proprietes.first().map(Propriete::types),
            Some(vec![
                "cell".to_owned(),
                "voice".to_owned(),
                "work".to_owned()
            ])
        );
        Ok(())
    }

    #[test]
    fn plusieurs_cartes_se_lisent_dans_leur_ordre() -> Result<(), ErreurVcard> {
        let cartes = lire(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:a\r\nEND:VCARD\r\nBEGIN:VCARD\r\nVERSION:4.0\r\nUID:b\r\nEND:VCARD\r\n",
        )?;

        let uids: Vec<_> = cartes.iter().filter_map(Carte::uid).collect();
        assert_eq!(uids, ["a", "b"]);
        Ok(())
    }

    #[test]
    fn une_version_2_1_est_refusee() {
        let resultat = lire("BEGIN:VCARD\r\nVERSION:2.1\r\nEND:VCARD\r\n");

        assert_eq!(
            resultat,
            Err(ErreurVcard::VersionNonPriseEnCharge {
                ligne: 2,
                version: "2.1".to_owned()
            })
        );
    }

    #[test]
    fn une_carte_sans_version_est_refusee() {
        let resultat = lire("BEGIN:VCARD\r\nFN:x\r\nEND:VCARD\r\n");

        assert_eq!(resultat, Err(ErreurVcard::VersionManquante { ligne: 1 }));
    }

    #[test]
    fn une_carte_non_fermee_est_refusee() {
        let resultat = lire("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:x\r\n");

        assert_eq!(resultat, Err(ErreurVcard::FinManquante { ligne: 1 }));
    }

    #[test]
    fn une_propriete_hors_carte_est_refusee() {
        let resultat = lire("FN:x\r\n");

        assert_eq!(resultat, Err(ErreurVcard::HorsCarte { ligne: 1 }));
    }

    #[test]
    fn une_ligne_sans_deux_points_est_refusee() {
        let resultat = lire("BEGIN:VCARD\r\nVERSION:4.0\r\nFN\r\nEND:VCARD\r\n");

        assert_eq!(resultat, Err(ErreurVcard::LigneInvalide { ligne: 3 }));
    }
}

mod valeurs {
    use super::*;

    #[test]
    fn un_texte_echappe_est_decode() -> Result<(), String> {
        let carte =
            une_carte("BEGIN:VCARD\r\nVERSION:3.0\r\nNOTE:a\\, b\\; c\\nd\\\\e\r\nEND:VCARD\r\n")?;

        let valeurs = carte.valeurs(SourceId::nouveau(), date(1));
        let notes: Vec<_> = valeurs
            .get(&Champ::Note)
            .into_iter()
            .flatten()
            .map(|v| v.texte.as_str())
            .collect();
        assert_eq!(notes, ["a, b; c\nd\\e"]);
        Ok(())
    }

    #[test]
    fn un_champ_a_valeur_unique_n_interprete_que_sa_premiere_occurrence() -> Result<(), String> {
        let carte =
            une_carte("BEGIN:VCARD\r\nVERSION:3.0\r\nNOTE:une\r\nNOTE:deux\r\nEND:VCARD\r\n")?;

        let conservees: Vec<_> = carte.conservees().map(|p| p.valeur.as_str()).collect();
        assert_eq!(conservees, ["deux"]);
        Ok(())
    }

    #[test]
    fn chaque_valeur_recue_porte_la_source_et_la_date_de_reception() -> Result<(), String> {
        let source = SourceId::nouveau();
        let carte =
            une_carte("BEGIN:VCARD\r\nVERSION:4.0\r\nEMAIL:a@example.org\r\nEND:VCARD\r\n")?;

        let valeurs = carte.valeurs(source, date(5));
        let provenances: Vec<_> = valeurs.values().flatten().map(|v| v.provenance).collect();
        assert_eq!(provenances, [Provenance::source(source, date(5))]);
        Ok(())
    }
}

mod ecriture {
    use super::*;

    #[test]
    fn une_ligne_longue_est_repliee_a_75_octets() -> Result<(), ErreurFiche> {
        let mut fiche = Fiche::nouvelle();
        fiche.modifier(Champ::Note, &["é".repeat(60).as_str()], date(1))?;

        let texte = Carte::reecrire(None, Version::V4, "u", &fiche).ecrire();

        assert!(texte.split("\r\n").all(|l| l.len() <= 75), "{texte}");
        Ok(())
    }

    #[test]
    fn une_ligne_repliee_se_relit_a_l_identique() -> Result<(), String> {
        let mut fiche = Fiche::nouvelle();
        let note = "é, ".repeat(40);
        fiche
            .modifier(Champ::Note, &[note.as_str()], date(1))
            .map_err(|e| e.to_string())?;

        let relue = une_carte(&Carte::reecrire(None, Version::V4, "u", &fiche).ecrire())?;

        let notes: Vec<_> = relue
            .valeurs(SourceId::nouveau(), date(1))
            .remove(&Champ::Note)
            .into_iter()
            .flatten()
            .map(|v| v.texte)
            .collect();
        assert_eq!(notes, [note]);
        Ok(())
    }

    // [utest->req~vcard-3-et-4~1]
    #[test]
    fn une_carte_ecrite_vers_une_source_3_0_declare_la_version_3_0() -> Result<(), ErreurFiche> {
        let mut fiche = Fiche::nouvelle();
        fiche.modifier(Champ::NomAffiche, &["Camille Exemple"], date(1))?;

        let texte = Carte::reecrire(None, Version::V3, "u", &fiche).ecrire();

        assert!(
            texte.starts_with("BEGIN:VCARD\r\nVERSION:3.0\r\n"),
            "{texte}"
        );
        Ok(())
    }

    #[test]
    fn une_carte_3_0_porte_toujours_n() -> Result<(), String> {
        let fiche = Fiche::nouvelle();

        let carte = une_carte(&Carte::reecrire(None, Version::V3, "u", &fiche).ecrire())?;

        assert!(carte.proprietes.iter().any(|p| p.nom == "N"));
        Ok(())
    }

    #[test]
    fn une_carte_sans_nom_affiche_le_compose_du_prenom_et_du_nom() -> Result<(), String> {
        let mut fiche = Fiche::nouvelle();
        fiche
            .modifier(Champ::NomStructure, &["Exemple;Camille;;;"], date(1))
            .map_err(|e| e.to_string())?;

        let carte = une_carte(&Carte::reecrire(None, Version::V4, "u", &fiche).ecrire())?;

        let fn_ = carte
            .proprietes
            .iter()
            .find(|p| p.nom == "FN")
            .map(|p| p.valeur.as_str());
        assert_eq!(fn_, Some("Camille Exemple"));
        Ok(())
    }

    #[test]
    fn une_carte_neuve_porte_l_uid_du_lien() -> Result<(), String> {
        let carte =
            une_carte(&Carte::reecrire(None, Version::V4, "uid-42", &Fiche::nouvelle()).ecrire())?;

        assert_eq!(carte.uid().as_deref(), Some("uid-42"));
        Ok(())
    }

    #[test]
    fn une_valeur_inchangee_garde_sa_forme_d_origine() -> Result<(), String> {
        let texte =
            "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:x\r\nTEL;TYPE=CELL:06 39 98 00 01\r\nEND:VCARD\r\n";
        let precedente = une_carte(texte)?;
        let mut fiche = fiche_de(&precedente, SourceId::nouveau()).map_err(|e| e.to_string())?;
        // Même numéro, autre écriture : la source garde la sienne.
        let tel =
            Valeur::new("+33639980001", Provenance::referentiel(date(2))).avec_types(["cell"]);
        fiche
            .definir(Champ::Telephone, vec![tel])
            .map_err(|e| e.to_string())?;

        let reecrite = Carte::reecrire(Some(&precedente), Version::V3, "u", &fiche).ecrire();

        assert_eq!(reecrite, texte);
        Ok(())
    }

    #[test]
    fn une_valeur_modifiee_remplace_l_ancienne() -> Result<(), String> {
        let precedente =
            une_carte("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:x\r\nTITLE:ancienne\r\nEND:VCARD\r\n")?;
        let mut fiche = fiche_de(&precedente, SourceId::nouveau()).map_err(|e| e.to_string())?;
        fiche
            .modifier(Champ::Fonction, &["nouvelle"], date(2))
            .map_err(|e| e.to_string())?;

        let reecrite =
            une_carte(&Carte::reecrire(Some(&precedente), Version::V4, "u", &fiche).ecrire())?;

        let titres: Vec<_> = reecrite
            .proprietes
            .iter()
            .filter(|p| p.nom == "TITLE")
            .map(|p| p.valeur.as_str())
            .collect();
        assert_eq!(titres, ["nouvelle"]);
        Ok(())
    }

    #[test]
    fn une_valeur_retiree_disparait_de_la_carte() -> Result<(), String> {
        let precedente = une_carte(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:x\r\nEMAIL:a@example.org\r\nEND:VCARD\r\n",
        )?;
        let mut fiche = fiche_de(&precedente, SourceId::nouveau()).map_err(|e| e.to_string())?;
        fiche
            .modifier(Champ::Email, &[], date(2))
            .map_err(|e| e.to_string())?;

        let reecrite =
            une_carte(&Carte::reecrire(Some(&precedente), Version::V4, "u", &fiche).ecrire())?;

        assert!(!reecrite.proprietes.iter().any(|p| p.nom == "EMAIL"));
        Ok(())
    }
}

mod conservation {
    use super::*;

    // [utest->req~conservation-proprietes-inconnues~1]
    #[test]
    fn une_propriete_inconnue_est_rendue_a_l_octet_pres_apres_modification() -> Result<(), String> {
        let precedente = une_carte(&carte_avec_inconnue())?;
        let mut fiche = fiche_de(&precedente, SourceId::nouveau()).map_err(|e| e.to_string())?;
        fiche
            .modifier(Champ::Note, &["note modifiée"], date(2))
            .map_err(|e| e.to_string())?;

        let reecrite = Carte::reecrire(Some(&precedente), Version::V4, "u1", &fiche).ecrire();

        assert!(
            reecrite.contains(&format!("\r\n{INCONNUE}\r\n")),
            "{reecrite}"
        );
        Ok(())
    }

    // [utest->req~conservation-proprietes-inconnues~1]
    #[test]
    fn une_propriete_inconnue_n_est_pas_recopiee_vers_une_autre_source() -> Result<(), String> {
        let precedente = une_carte(&carte_avec_inconnue())?;
        let fiche = fiche_de(&precedente, SourceId::nouveau()).map_err(|e| e.to_string())?;

        // Seconde source : aucune carte connue d'elle pour cette fiche.
        let autre = Carte::reecrire(None, Version::V4, "uid-autre", &fiche).ecrire();

        assert!(!autre.contains("X-PROPRIETAIRE"), "{autre}");
        Ok(())
    }
}

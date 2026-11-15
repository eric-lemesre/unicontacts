// [utest->req~import-stockage-local~1]
// [utest->req~source-sim-lecture-seule~1]
use super::*;
use crate::decisions::Nature;
use crate::fiche::Fiche;

type Resultat = Result<(), Box<dyn std::error::Error>>;

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

fn valeur(champ: Champ, texte: &str, types: &[&str]) -> ValeurLue {
    ValeurLue {
        champ,
        texte: texte.to_owned(),
        types: types.iter().map(|t| (*t).to_owned()).collect(),
    }
}

/// Contact local au nom, au téléphone et à l'e-mail donnés.
fn contact(uid: &str, nom: &str, telephone: &str, email: &str) -> ContactLu {
    let mut valeurs = vec![valeur(Champ::NomAffiche, nom, &[])];
    if !telephone.is_empty() {
        valeurs.push(valeur(Champ::Telephone, telephone, &["cell"]));
    }
    if !email.is_empty() {
        valeurs.push(valeur(Champ::Email, email, &["home"]));
    }
    ContactLu {
        uid: uid.to_owned(),
        valeurs,
        dates: Vec::new(),
    }
}

fn local(referentiel: &mut Referentiel) -> Result<SourceId, ErreurReglage> {
    referentiel.ajouter_source_import("Stockage local")
}

fn fiche_liee<'a>(referentiel: &'a Referentiel, source: SourceId, uid: &str) -> Option<&'a Fiche> {
    referentiel
        .fiches()
        .find(|f| f.lien(source).is_some_and(|l| l.uid == uid))
}

#[test]
fn un_contact_local_inconnu_donne_une_fiche_liee_a_sa_copie() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    let contacts = [contact("local-1", "Alice Martin", "06 12 34 56 78", "")];
    referentiel.importer_contacts(source, &contacts, date(1))?;
    let fiche = fiche_liee(&referentiel, source, "local-1").ok_or("aucune fiche liée")?;
    assert_eq!(fiche.textes(Champ::Telephone), ["06 12 34 56 78"]);
    Ok(())
}

#[test]
fn un_contact_local_au_meme_email_et_au_meme_nom_rejoint_la_fiche_existante() -> Resultat {
    let mut referentiel = Referentiel::new();
    let carnet = referentiel.ajouter_source_import("Carnet")?;
    referentiel.importer_contacts(
        carnet,
        &[contact("c-1", "Alice Martin", "", "alice@exemple.fr")],
        date(1),
    )?;
    let source = local(&mut referentiel)?;
    let contacts = [contact(
        "local-1",
        "alice martin",
        "0612345678",
        "alice@exemple.fr",
    )];
    referentiel.importer_contacts(source, &contacts, date(2))?;
    assert_eq!(referentiel.fiches().count(), 1);
    Ok(())
}

#[test]
fn un_contact_local_au_seul_email_commun_est_soumis_en_decision() -> Resultat {
    let mut referentiel = Referentiel::new();
    let carnet = referentiel.ajouter_source_import("Carnet")?;
    referentiel.importer_contacts(
        carnet,
        &[contact("c-1", "Alice Martin", "", "alice@exemple.fr")],
        date(1),
    )?;
    let source = local(&mut referentiel)?;
    let contacts = [contact("local-1", "Accueil", "", "alice@exemple.fr")];
    let import = referentiel.importer_contacts(source, &contacts, date(2))?;
    let natures: Vec<bool> = import
        .rapport
        .decisions
        .iter()
        .filter_map(|id| referentiel.decision(*id))
        .map(|d| matches!(d.nature, Nature::DoublonProbable { .. }))
        .collect();
    assert_eq!(natures, [true]);
    Ok(())
}

#[test]
fn un_import_rejoue_ne_cree_aucune_fiche_en_double() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    let contacts = [
        contact("local-1", "Alice Martin", "0612345678", ""),
        contact("local-2", "Bruno Petit", "", "bruno@exemple.fr"),
    ];
    referentiel.importer_contacts(source, &contacts, date(1))?;
    referentiel.importer_contacts(source, &contacts, date(2))?;
    assert_eq!(referentiel.fiches().count(), 2);
    Ok(())
}

#[test]
fn un_import_n_ecrit_jamais_vers_le_stockage_local() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    let contacts = [contact("local-1", "Alice Martin", "0612345678", "")];
    referentiel.importer_contacts(source, &contacts, date(1))?;
    let id = fiche_liee(&referentiel, source, "local-1")
        .ok_or("aucune fiche")?
        .id();
    referentiel.fiche_mut(id).ok_or("fiche perdue")?.modifier(
        Champ::Note,
        &["vue au salon"],
        date(2),
    )?;
    let import = referentiel.importer_contacts(source, &contacts, date(3))?;
    assert!(import.rapport.ecritures.is_empty());
    Ok(())
}

#[test]
fn une_modification_locale_apres_import_rejoint_la_fiche() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    referentiel.importer_contacts(
        source,
        &[contact("local-1", "Alice Martin", "0612345678", "")],
        date(1),
    )?;
    let modifie = contact("local-1", "Alice Martin", "0612345678", "alice@exemple.fr");
    referentiel.importer_contacts(source, &[modifie], date(2))?;
    let fiche = fiche_liee(&referentiel, source, "local-1").ok_or("aucune fiche")?;
    assert_eq!(fiche.textes(Champ::Email), ["alice@exemple.fr"]);
    Ok(())
}

#[test]
fn les_originaux_retires_detachent_leurs_fiches_sans_les_toucher() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    referentiel.importer_contacts(
        source,
        &[contact("local-1", "Alice Martin", "0612345678", "")],
        date(1),
    )?;
    referentiel.importer_contacts(source, &[], date(2))?;
    let fiches: Vec<(Vec<String>, bool)> = referentiel
        .fiches()
        .map(|f| (f.textes(Champ::Telephone), f.lien(source).is_some()))
        .collect();
    assert_eq!(fiches, [(vec!["0612345678".to_owned()], false)]);
    Ok(())
}

#[test]
fn les_copies_liees_sont_les_contacts_deja_importes() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    let contacts = [
        contact("local-2", "Bruno Petit", "", "bruno@exemple.fr"),
        contact("local-1", "Alice Martin", "0612345678", ""),
    ];
    referentiel.importer_contacts(source, &contacts, date(1))?;
    assert_eq!(referentiel.copies_liees(source), ["local-1", "local-2"]);
    Ok(())
}

#[test]
fn les_dates_d_un_contact_local_rejoignent_la_fiche() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    let mut lu = contact("local-1", "Alice Martin", "0612345678", "");
    lu.dates = vec![
        (TypeDate::Naissance, "1980-05-17".to_owned()),
        (TypeDate::Mariage, "--06-21".to_owned()),
    ];
    referentiel.importer_contacts(source, &[lu], date(1))?;
    let fiche = fiche_liee(&referentiel, source, "local-1").ok_or("aucune fiche")?;
    let dates = (
        fiche.textes(Champ::Anniversaire),
        fiche.textes(Champ::Dates),
    );
    assert_eq!(
        dates,
        (
            vec!["1980-05-17".to_owned()],
            vec!["mariage;--06-21".to_owned()]
        )
    );
    Ok(())
}

#[test]
fn une_date_illisible_est_signalee_sans_arreter_l_import() -> Resultat {
    let mut referentiel = Referentiel::new();
    let source = local(&mut referentiel)?;
    let mut lu = contact("local-1", "Alice Martin", "0612345678", "");
    lu.dates = vec![(TypeDate::Naissance, "bientôt".to_owned())];
    let import = referentiel.importer_contacts(source, &[lu], date(1))?;
    assert_eq!(
        (import.dates_illisibles, referentiel.fiches().count()),
        (vec![("local-1".to_owned(), "bientôt".to_owned())], 1)
    );
    Ok(())
}

#[test]
fn deux_notes_d_un_contact_local_se_joignent() {
    let mut lu = contact("local-1", "Alice Martin", "", "");
    lu.valeurs.push(valeur(Champ::Note, "voisine", &[]));
    lu.valeurs
        .push(valeur(Champ::Note, "allergique aux chats", &[]));
    let (copie, _) = lu.copie();
    assert!(
        copie
            .contenu
            .contains("NOTE:voisine\\n\\nallergique aux chats")
    );
}

/// Entrée de SIM : nom et numéro seulement (ADR-0016).
fn entree_sim(uid: &str, nom: &str, numero: &str) -> ContactLu {
    ContactLu {
        uid: uid.to_owned(),
        valeurs: vec![
            valeur(Champ::NomAffiche, nom, &[]),
            valeur(Champ::Telephone, numero, &[]),
        ],
        dates: Vec::new(),
    }
}

#[test]
fn une_entree_sim_donne_une_fiche_portant_son_nom_et_son_numero() -> Resultat {
    let mut referentiel = Referentiel::new();
    let sim = referentiel.ajouter_source_import("Carte SIM")?;
    referentiel.importer_contacts(
        sim,
        &[entree_sim("sim-1-3", "Garage", "+33 4 78 00 00 00")],
        date(1),
    )?;
    let fiche = fiche_liee(&referentiel, sim, "sim-1-3").ok_or("aucune fiche liée")?;
    assert_eq!(
        (
            fiche.textes(Champ::NomAffiche),
            fiche.textes(Champ::Telephone)
        ),
        (
            vec!["Garage".to_owned()],
            vec!["+33 4 78 00 00 00".to_owned()]
        )
    );
    Ok(())
}

#[test]
fn une_fiche_liee_a_la_sim_modifiee_ne_donne_aucune_ecriture_vers_la_sim() -> Resultat {
    let mut referentiel = Referentiel::new();
    let sim = referentiel.ajouter_source_import("Carte SIM")?;
    let entrees = [entree_sim("sim-1-3", "Garage", "+33 4 78 00 00 00")];
    referentiel.importer_contacts(sim, &entrees, date(1))?;
    let id = fiche_liee(&referentiel, sim, "sim-1-3")
        .ok_or("aucune fiche")?
        .id();
    referentiel.fiche_mut(id).ok_or("fiche perdue")?.modifier(
        Champ::Telephone,
        &["+33 4 78 11 11 11"],
        date(2),
    )?;
    let import = referentiel.importer_contacts(sim, &entrees, date(3))?;
    assert!(import.rapport.ecritures.is_empty());
    Ok(())
}

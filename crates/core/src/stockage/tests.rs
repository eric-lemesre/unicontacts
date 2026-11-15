use super::*;
use crate::synchronisation::CopieDistante;

type Essai = Result<(), Box<dyn std::error::Error>>;

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

fn copie(uid: &str, lignes: &[&str]) -> CopieDistante {
    let mut contenu = format!("BEGIN:VCARD\r\nVERSION:4.0\r\nUID:{uid}\r\n");
    for ligne in lignes {
        contenu.push_str(ligne);
        contenu.push_str("\r\n");
    }
    contenu.push_str("END:VCARD\r\n");
    CopieDistante {
        uid: uid.to_owned(),
        etag: Some(format!("\"{uid}\"")),
        contenu,
    }
}

/// Référentiel qui exerce chaque table : sources réglées et désignées,
/// fiches aux valeurs de provenances diverses, liens, bases, propriétés
/// inconnues, décisions de chaque statut, copies à supprimer et ignorées,
/// relations et leur mémoire, fiche « moi », inverses déclarés.
fn referentiel_complet() -> Result<Referentiel, Box<dyn std::error::Error>> {
    let mut referentiel = Referentiel::new();
    let nextcloud = referentiel.sources_mut().ajouter("Nextcloud", Version::V4);
    let google = referentiel.sources_mut().ajouter("Google", Version::V3);
    referentiel.sources_mut().regler(
        nextcloud,
        Reglages {
            autorite: Autorite::CetteSource,
            suppressions: Suppressions::Demander,
            ..Reglages::default()
        },
    )?;
    referentiel.sources_mut().regler(
        google,
        Reglages {
            direction: Direction::LectureSeule,
            conflits: Conflits::AutoriteGagne,
            ..Reglages::default()
        },
    )?;
    referentiel
        .sources_mut()
        .designer_pour_champ(Champ::Telephone, google)?;

    referentiel.synchroniser(
        nextcloud,
        &[
            copie(
                "a",
                &[
                    "FN:Camille Exemple",
                    "TEL;TYPE=cell,voice:+33639980000",
                    "EMAIL:camille@example.org",
                    "X-PROPRIETAIRE:garde",
                ],
            ),
            copie("b", &["FN:Dominique Fictif", "ORG:Société Fictive"]),
            copie("c", &["FN:Alix Démo", "NOTE:à supprimer"]),
            copie("d", &["FN:Sacha Inconnu"]),
        ],
        date(1),
    )?;
    let fiche = |uid: &str| {
        referentiel
            .fiches()
            .find(|f| f.lien(nextcloud).is_some_and(|l| l.uid == uid))
            .map(Fiche::id)
            .ok_or("fiche absente")
    };
    let (a, b, c) = (fiche("a")?, fiche("b")?, fiche("c")?);
    if let Some(f) = referentiel.fiche_mut(a) {
        f.modifier(Champ::NomAffiche, &["Camille Ex."], date(2))?;
    }
    if let Some(f) = referentiel.fiche_mut(b) {
        f.modifier(Champ::Organisation, &["Autre"], date(2))?;
    }
    referentiel.synchroniser(
        nextcloud,
        &[
            copie(
                "a",
                &[
                    "FN:C. Exemple",
                    "TEL;TYPE=cell,voice:+33639980000",
                    "X-PROPRIETAIRE:garde",
                ],
            ),
            copie(
                "b",
                &[
                    "FN:Dominique Fictif",
                    "ORG:Société Nouvelle",
                    "RELATED;TYPE=friend:urn:uuid:a",
                ],
            ),
            copie("c", &["FN:Alix Démo", "NOTE:à supprimer"]),
        ],
        date(3),
    )?;
    referentiel.designer_moi(Some(a))?;
    let patient = TypeRelation::depuis_texte("Patient");
    referentiel
        .inverses_mut()
        .declarer(&TypeRelation::Medecin, &patient)?;
    referentiel.lier(a, TypeRelation::Medecin, b, date(3))?;
    let ouvertes: Vec<_> = referentiel.ouvertes().map(|d| (d.id, d.nature)).collect();
    for (id, nature) in ouvertes {
        match nature {
            Nature::Divergence {
                champ: Champ::NomAffiche,
            } => referentiel.resoudre(
                id,
                Resolution::Divergence(Action::Saisir(vec!["Camille E.".to_owned()])),
                date(4),
            )?,
            Nature::Suppression => referentiel.resoudre(
                id,
                Resolution::Suppression(IssueSuppression::Restaurer),
                date(4),
            )?,
            _ => referentiel.reporter(id)?,
        }
    }
    referentiel.supprimer_fiche(c, &[nextcloud])?;
    referentiel.synchroniser(
        google,
        // Homonyme de la fiche « d » de Nextcloud, sans autre donnée
        // commune : doublon probable, décision laissée en attente.
        &[copie("g-1", &["FN:Sacha Inconnú"])],
        date(5),
    )?;
    Ok(referentiel)
}

mod aller_retour {
    use super::*;

    // [utest->req~fiche-maitre-identifiant-stable~1]
    // [utest->req~provenance-par-champ~1]
    // [utest->req~conservation-proprietes-inconnues~1]
    #[test]
    fn le_referentiel_relu_est_identique_a_celui_enregistre() -> Essai {
        let referentiel = referentiel_complet()?;
        let mut stockage = Stockage::en_memoire()?;

        stockage.enregistrer(&referentiel)?;

        assert_eq!(stockage.charger()?, referentiel);
        Ok(())
    }

    #[test]
    fn le_jeu_d_essai_exerce_chaque_statut_de_decision() -> Essai {
        let referentiel = referentiel_complet()?;

        let mut statuts: Vec<_> = referentiel
            .decisions()
            .iter()
            .map(|d| code_statut(d.statut))
            .collect();
        statuts.sort_unstable();
        statuts.dedup();
        assert_eq!(statuts, ["en_attente", "reportee", "resolue"]);
        Ok(())
    }

    #[test]
    fn un_second_enregistrement_remplace_le_premier() -> Essai {
        let mut referentiel = referentiel_complet()?;
        let mut stockage = Stockage::en_memoire()?;
        stockage.enregistrer(&referentiel)?;
        let premiere = referentiel
            .fiches()
            .next()
            .map(Fiche::id)
            .ok_or("aucune fiche")?;
        referentiel.supprimer_fiche(premiere, &[])?;

        stockage.enregistrer(&referentiel)?;

        assert_eq!(stockage.charger()?, referentiel);
        Ok(())
    }

    #[test]
    fn une_base_neuve_donne_un_referentiel_vide() -> Essai {
        let stockage = Stockage::en_memoire()?;

        assert_eq!(stockage.charger()?, Referentiel::new());
        Ok(())
    }
}

mod fichier {
    use super::*;

    /// Chemin de base unique, retiré à la fin de l'essai.
    struct BaseTemporaire(std::path::PathBuf);

    impl BaseTemporaire {
        fn new() -> Self {
            Self(
                std::env::temp_dir()
                    .join(format!("unicontacts-essai-{}.sqlite", FicheId::nouveau())),
            )
        }
    }

    impl Drop for BaseTemporaire {
        fn drop(&mut self) {
            for suffixe in ["", "-wal", "-shm"] {
                let mut chemin = self.0.clone().into_os_string();
                chemin.push(suffixe);
                // Nettoyage d'essai : un fichier déjà absent n'est pas une erreur.
                let _ = std::fs::remove_file(chemin);
            }
        }
    }

    #[test]
    fn le_referentiel_survit_a_la_reouverture_de_la_base() -> Essai {
        let base = BaseTemporaire::new();
        let referentiel = referentiel_complet()?;
        Stockage::ouvrir(&base.0)?.enregistrer(&referentiel)?;

        let relu = Stockage::ouvrir(&base.0)?.charger()?;

        assert_eq!(relu, referentiel);
        Ok(())
    }

    #[test]
    fn une_base_d_un_schema_plus_recent_est_refusee() -> Essai {
        let base = BaseTemporaire::new();
        Connection::open(&base.0)?.pragma_update(None, "user_version", 99)?;

        let refus = Stockage::ouvrir(&base.0);

        assert!(
            matches!(refus, Err(ErreurStockage::SchemaInconnu(99))),
            "{refus:?}"
        );
        Ok(())
    }
}

#[test]
fn le_jeu_d_essai_exerce_relations_memoire_moi_et_inverses() -> Essai {
    let referentiel = referentiel_complet()?;

    let memoires = referentiel
        .fiches()
        .flat_map(Fiche::liens)
        .filter(|l| !l.memoire.is_empty())
        .count();
    let relations: usize = referentiel
        .fiches()
        .map(|f| referentiel.relations(f.id()).len())
        .sum();

    assert_eq!(
        (
            memoires,
            relations,
            referentiel.moi().is_some(),
            referentiel.inverses().couples().count()
        ),
        (1, 3, true, 1)
    );
    Ok(())
}

#[test]
fn une_base_du_schema_1_passe_au_schema_courant_sans_perte() -> Essai {
    let connexion = Connection::open_in_memory()?;
    connexion.execute_batch(SCHEMA_1)?;
    connexion.pragma_update(None, "user_version", 1)?;
    connexion.execute(
        "INSERT INTO fiches (id) VALUES (?1)",
        params![Fiche::nouvelle().id().to_string()],
    )?;

    let stockage = Stockage::preparer(connexion)?;

    let version: i64 = stockage
        .connexion
        .pragma_query_value(None, "user_version", |l| l.get(0))?;
    assert_eq!(
        (version, stockage.charger()?.fiches().count()),
        (VERSION_SCHEMA, 1)
    );
    Ok(())
}

#[test]
fn une_base_neuve_porte_la_version_du_schema() -> Essai {
    let stockage = Stockage::en_memoire()?;

    let version: i64 = stockage
        .connexion
        .pragma_query_value(None, "user_version", |l| l.get(0))?;

    assert_eq!(version, VERSION_SCHEMA);
    Ok(())
}

#[test]
fn une_valeur_alteree_est_signalee_et_non_devinee() -> Essai {
    let mut stockage = Stockage::en_memoire()?;
    stockage.enregistrer(&referentiel_complet()?)?;
    stockage
        .connexion
        .execute("UPDATE decisions SET statut = 'inconnu' WHERE rang = 0", [])?;

    let refus = stockage.charger();

    assert!(
        matches!(
            refus,
            Err(ErreurStockage::ValeurIllisible {
                colonne: "decisions.statut",
                ..
            })
        ),
        "{refus:?}"
    );
    Ok(())
}

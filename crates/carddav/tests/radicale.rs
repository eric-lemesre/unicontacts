//! Tests d'intégration du connecteur contre un vrai serveur CardDAV,
//! Radicale (ADR-0008). Lancés par `sh tools/radicale.sh test`, qui démarre
//! le serveur et donne son URL dans `UNICONTACTS_RADICALE` ; ignorés par un
//! simple `cargo test`.

use std::time::Duration;

use unicontacts_carddav::{
    Carnet, ClientCardDav, Compte, ErreurCardDav, MotDePasse, Precondition, Ressource, synchroniser,
};
use unicontacts_core::decisions::Nature;
use unicontacts_core::referentiel::Referentiel;
use unicontacts_core::vcard::Version;
use unicontacts_core::{Champ, Fiche, FicheId, Horodatage, SourceId};
use ureq::Agent;
use ureq::http::Request;

type Essai = Result<(), Box<dyn std::error::Error>>;

const UTILISATEUR: &str = "essai";

/// Carnet neuf sur le serveur de test, et accès HTTP direct pour le
/// préparer et l'inspecter comme le ferait un autre client.
struct Serveur {
    agent: Agent,
    carnet: String,
}

impl Serveur {
    fn nouveau() -> Result<Self, Box<dyn std::error::Error>> {
        let base = std::env::var("UNICONTACTS_RADICALE")
            .map_err(|_| "UNICONTACTS_RADICALE absent : lancer « sh tools/radicale.sh test »")?;
        let agent: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .allow_non_standard_methods(true)
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        let serveur = Self {
            agent,
            carnet: format!("{base}/{UTILISATEUR}/{}/", FicheId::nouveau()),
        };
        let corps = r#"<?xml version="1.0"?><mkcol xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav"><set><prop><resourcetype><collection/><C:addressbook/></resourcetype></prop></set></mkcol>"#;
        let statut = serveur
            .envoyer(
                "MKCOL",
                &serveur.carnet,
                &[("Content-Type", "application/xml")],
                corps,
            )?
            .0;
        if statut != 201 {
            return Err(format!("MKCOL : statut {statut}").into());
        }
        Ok(serveur)
    }

    fn envoyer(
        &self,
        methode: &str,
        url: &str,
        entetes: &[(&str, &str)],
        corps: &str,
    ) -> Result<(u16, String), Box<dyn std::error::Error>> {
        let mut requete = Request::builder()
            .method(methode)
            .uri(url)
            .header("Authorization", "Basic ZXNzYWk6eA==");
        for (nom, valeur) in entetes {
            requete = requete.header(*nom, *valeur);
        }
        let mut reponse = self.agent.run(requete.body(corps.to_owned())?)?;
        let statut = reponse.status().as_u16();
        Ok((statut, reponse.body_mut().read_to_string()?))
    }

    /// Dépose une carte comme le ferait un autre client du carnet.
    fn deposer(&self, nom: &str, vcard: &str) -> Essai {
        let url = format!("{}{nom}.vcf", self.carnet);
        let statut = self
            .envoyer("PUT", &url, &[("Content-Type", "text/vcard")], vcard)?
            .0;
        if !matches!(statut, 201 | 204) {
            return Err(format!("PUT {nom} : statut {statut}").into());
        }
        Ok(())
    }

    fn lire(&self, nom: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self
            .envoyer("GET", &format!("{}{nom}.vcf", self.carnet), &[], "")?
            .1)
    }

    fn retirer(&self, nom: &str) -> Essai {
        self.envoyer("DELETE", &format!("{}{nom}.vcf", self.carnet), &[], "")?;
        Ok(())
    }

    fn client(&self) -> Result<ClientCardDav, ErreurCardDav> {
        ClientCardDav::new(&Compte {
            carnet: self.carnet.clone(),
            utilisateur: UTILISATEUR.to_owned(),
            mot_de_passe: MotDePasse::new("x"),
        })
    }
}

fn vcard(uid: &str, lignes: &[&str]) -> String {
    let mut texte = format!("BEGIN:VCARD\r\nVERSION:3.0\r\nUID:{uid}\r\n");
    for ligne in lignes {
        texte.push_str(ligne);
        texte.push_str("\r\n");
    }
    texte.push_str("END:VCARD\r\n");
    texte
}

fn date(millis: i64) -> Horodatage {
    Horodatage::depuis_millis(millis)
}

fn referentiel() -> (Referentiel, SourceId) {
    let mut referentiel = Referentiel::new();
    let source = referentiel.sources_mut().ajouter("Radicale", Version::V3);
    (referentiel, source)
}

fn fiche_de(referentiel: &Referentiel, source: SourceId, uid: &str) -> Result<FicheId, String> {
    referentiel
        .fiches()
        .find(|f| f.lien(source).is_some_and(|l| l.uid == uid))
        .map(Fiche::id)
        .ok_or_else(|| format!("aucune fiche liée à {uid}"))
}

fn etag_lie(referentiel: &Referentiel, source: SourceId, uid: &str) -> Option<String> {
    referentiel
        .fiches()
        .find_map(|f| f.lien(source).filter(|l| l.uid == uid))
        .and_then(|l| l.etag.clone())
}

fn etag_serveur(client: &ClientCardDav, href_fin: &str) -> Result<Option<String>, ErreurCardDav> {
    Ok(client
        .lister()?
        .into_iter()
        .find(|r| r.href.ends_with(href_fin))
        .and_then(|r| r.etag))
}

/// Carnet d'une carte importée, puis fiche modifiée dans le référentiel.
fn fiche_modifiee(
    serveur: &Serveur,
) -> Result<(Referentiel, SourceId, FicheId), Box<dyn std::error::Error>> {
    serveur.deposer(
        "a",
        &vcard(
            "a",
            &[
                "FN:Camille Exemple",
                "N:Exemple;Camille;;;",
                "X-PROPRIETAIRE:garde",
            ],
        ),
    )?;
    let (mut referentiel, source) = referentiel();
    synchroniser(&mut referentiel, source, &serveur.client()?, date(1))?;
    let fiche = fiche_de(&referentiel, source, "a")?;
    if let Some(f) = referentiel.fiche_mut(fiche) {
        f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
    }
    Ok((referentiel, source, fiche))
}

// [itest->req~source-carddav~1]
#[test]
#[ignore = "requiert Radicale : sh tools/radicale.sh test"]
fn chaque_vcard_du_serveur_donne_une_fiche_liee_a_son_uid_et_a_son_etag() -> Essai {
    let serveur = Serveur::nouveau()?;
    serveur.deposer(
        "un",
        &vcard("uid-1", &["FN:Camille Exemple", "N:Exemple;Camille;;;"]),
    )?;
    serveur.deposer(
        "deux",
        &vcard("uid-2", &["FN:Dominique Fictif", "N:Fictif;Dominique;;;"]),
    )?;
    let (mut referentiel, source) = referentiel();
    let client = serveur.client()?;

    synchroniser(&mut referentiel, source, &client, date(1))?;

    let attendus = (
        etag_serveur(&client, "/un.vcf")?,
        etag_serveur(&client, "/deux.vcf")?,
    );
    assert_eq!(
        (
            etag_lie(&referentiel, source, "uid-1"),
            etag_lie(&referentiel, source, "uid-2")
        ),
        attendus
    );
    assert!(
        attendus.0.is_some() && attendus.1.is_some(),
        "ETag absents : {attendus:?}"
    );
    Ok(())
}

// [itest->req~source-carddav~1]
#[test]
#[ignore = "requiert Radicale : sh tools/radicale.sh test"]
fn une_fiche_modifiee_dans_le_referentiel_est_ecrite_sur_le_serveur_et_son_etag_mis_a_jour() -> Essai
{
    let serveur = Serveur::nouveau()?;
    let (mut referentiel, source, _) = fiche_modifiee(&serveur)?;
    let client = serveur.client()?;
    let avant = etag_lie(&referentiel, source, "a");

    let bilan = synchroniser(&mut referentiel, source, &client, date(3))?;

    let apres = etag_lie(&referentiel, source, "a");
    assert_eq!(
        (
            bilan.refus.len(),
            serveur.lire("a")?.contains("TITLE:Responsable"),
            apres.clone(),
            apres != avant
        ),
        (0, true, etag_serveur(&client, "/a.vcf")?, true)
    );
    Ok(())
}

// [itest->req~source-carddav~1]
#[test]
#[ignore = "requiert Radicale : sh tools/radicale.sh test"]
fn une_propriete_inconnue_reste_sur_le_serveur_apres_reecriture() -> Essai {
    let serveur = Serveur::nouveau()?;
    let (mut referentiel, source, _) = fiche_modifiee(&serveur)?;

    synchroniser(&mut referentiel, source, &serveur.client()?, date(3))?;

    let carte = serveur.lire("a")?;
    assert!(carte.contains("X-PROPRIETAIRE:garde"), "{carte}");
    Ok(())
}

/// Carnet qui laisse un autre client modifier la carte entre la lecture et
/// l'écriture du connecteur.
struct CarnetConcurrent<'a> {
    client: ClientCardDav,
    serveur: &'a Serveur,
}

impl Carnet for CarnetConcurrent<'_> {
    fn lister(&self) -> Result<Vec<Ressource>, ErreurCardDav> {
        let ressources = self.client.lister()?;
        self.serveur
            .deposer(
                "a",
                &vcard(
                    "a",
                    &[
                        "FN:Camille Exemple",
                        "N:Exemple;Camille;;;",
                        "X-PROPRIETAIRE:garde",
                        "NOTE:ajoutée ailleurs",
                    ],
                ),
            )
            .map_err(|e| ErreurCardDav::Transport(e.to_string()))?;
        Ok(ressources)
    }

    fn ecrire(
        &self,
        href: &str,
        contenu: &str,
        precondition: &Precondition,
    ) -> Result<Option<String>, ErreurCardDav> {
        self.client.ecrire(href, contenu, precondition)
    }

    fn supprimer(&self, href: &str, etag: Option<&str>) -> Result<(), ErreurCardDav> {
        self.client.supprimer(href, etag)
    }

    fn href_nouveau(&self, uid: &str) -> String {
        self.client.href_nouveau(uid)
    }
}

// [itest->req~source-carddav~1]
#[test]
#[ignore = "requiert Radicale : sh tools/radicale.sh test"]
fn une_carte_modifiee_entre_lecture_et_ecriture_n_est_pas_ecrasee_et_les_deux_changements_se_rejoignent()
-> Essai {
    let serveur = Serveur::nouveau()?;
    let (mut referentiel, source, _) = fiche_modifiee(&serveur)?;
    let concurrent = CarnetConcurrent {
        client: serveur.client()?,
        serveur: &serveur,
    };
    let bilan = synchroniser(&mut referentiel, source, &concurrent, date(3))?;

    synchroniser(&mut referentiel, source, &serveur.client()?, date(4))?;

    let carte = serveur.lire("a")?;
    assert_eq!(
        (
            bilan.refus.first().map(|r| r.erreur.clone()),
            carte.contains("TITLE:Responsable"),
            carte.contains("NOTE:ajoutée ailleurs")
        ),
        (Some(ErreurCardDav::PreconditionEchouee), true, true),
        "{carte}"
    );
    Ok(())
}

// [itest->req~source-carddav~1]
#[test]
#[ignore = "requiert Radicale : sh tools/radicale.sh test"]
fn une_carte_retiree_du_serveur_devient_une_decision_sans_rien_detruire() -> Essai {
    let serveur = Serveur::nouveau()?;
    let (mut referentiel, source, fiche) = fiche_modifiee(&serveur)?;
    serveur.retirer("a")?;

    let bilan = synchroniser(&mut referentiel, source, &serveur.client()?, date(3))?;

    let natures: Vec<_> = bilan
        .rapport
        .decisions
        .iter()
        .filter_map(|d| referentiel.decision(*d))
        .map(|d| d.nature)
        .collect();
    assert_eq!(
        (natures, referentiel.fiche(fiche).is_some()),
        (vec![Nature::Suppression], true)
    );
    Ok(())
}

// [itest->req~source-carddav~1]
#[test]
#[ignore = "requiert Radicale : sh tools/radicale.sh test"]
fn une_fiche_supprimee_dans_l_application_est_supprimee_du_serveur() -> Essai {
    let serveur = Serveur::nouveau()?;
    let (mut referentiel, source, fiche) = fiche_modifiee(&serveur)?;
    referentiel.supprimer_fiche(fiche, &[source])?;

    let bilan = synchroniser(&mut referentiel, source, &serveur.client()?, date(3))?;

    assert_eq!(
        (bilan.supprimees, serveur.client()?.lister()?.len()),
        (1, 0)
    );
    Ok(())
}

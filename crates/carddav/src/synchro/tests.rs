// Le connecteur et le moteur du cœur éprouvés ensemble, contre un carnet
// simulé : pannes, préconditions et coupures à la demande, que le serveur
// réel (tests/radicale.rs) ne sait pas produire.
// [itest->req~source-carddav~1]

use std::cell::RefCell;
use std::collections::BTreeMap;

use unicontacts_core::vcard::Version;
use unicontacts_core::{Champ, Fiche};

use super::*;
use crate::Ressource;

type Essai = Result<(), Box<dyn std::error::Error>>;

/// Carnet simulé : des cartes par chemin, chacune avec son ETag, et des
/// pannes à la demande.
#[derive(Default)]
struct CarnetSimule {
    cartes: RefCell<BTreeMap<String, (String, String)>>,
    version: RefCell<u32>,
    /// Chemins dont la prochaine écriture échoue par précondition.
    conflits: RefCell<Vec<String>>,
    /// Vrai si toute écriture échoue faute de liaison.
    coupe: RefCell<bool>,
    /// Nombre d'écritures tentées.
    tentatives: RefCell<usize>,
}

impl CarnetSimule {
    fn avec(cartes: &[(&str, &str)]) -> Self {
        let carnet = Self::default();
        for (href, contenu) in cartes {
            carnet.deposer(href, contenu);
        }
        carnet
    }

    fn deposer(&self, href: &str, contenu: &str) -> String {
        let mut version = self.version.borrow_mut();
        *version += 1;
        let etag = format!("\"v{version}\"");
        self.cartes
            .borrow_mut()
            .insert(href.to_owned(), (etag.clone(), contenu.to_owned()));
        etag
    }

    fn contenu(&self, href: &str) -> Option<String> {
        self.cartes.borrow().get(href).map(|(_, c)| c.clone())
    }
}

impl Carnet for CarnetSimule {
    fn lister(&self) -> Result<Vec<Ressource>, ErreurCardDav> {
        Ok(self
            .cartes
            .borrow()
            .iter()
            .map(|(href, (etag, contenu))| Ressource {
                href: href.clone(),
                etag: Some(etag.clone()),
                contenu: contenu.clone(),
            })
            .collect())
    }

    fn ecrire(
        &self,
        href: &str,
        contenu: &str,
        precondition: &Precondition,
    ) -> Result<Option<String>, ErreurCardDav> {
        *self.tentatives.borrow_mut() += 1;
        if *self.coupe.borrow() {
            return Err(ErreurCardDav::Transport("réseau coupé".to_owned()));
        }
        let mut conflits = self.conflits.borrow_mut();
        if let Some(position) = conflits.iter().position(|h| h == href) {
            conflits.remove(position);
            return Err(ErreurCardDav::PreconditionEchouee);
        }
        let actuel = self.cartes.borrow().get(href).map(|(e, _)| e.clone());
        let admis = match (precondition, &actuel) {
            (Precondition::Creation, None) | (Precondition::Aucune, _) => true,
            (Precondition::Remplacement(attendu), Some(etag)) => attendu == etag,
            _ => false,
        };
        if !admis {
            return Err(ErreurCardDav::PreconditionEchouee);
        }
        Ok(Some(self.deposer(href, contenu)))
    }

    fn supprimer(&self, href: &str, _etag: Option<&str>) -> Result<(), ErreurCardDav> {
        if *self.coupe.borrow() {
            return Err(ErreurCardDav::Transport("réseau coupé".to_owned()));
        }
        self.cartes.borrow_mut().remove(href);
        Ok(())
    }

    fn href_nouveau(&self, uid: &str) -> String {
        format!("/carnet/{uid}.vcf")
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
    let source = referentiel.sources_mut().ajouter("Nextcloud", Version::V3);
    (referentiel, source)
}

fn fiche_de(
    referentiel: &Referentiel,
    source: SourceId,
    uid: &str,
) -> Result<unicontacts_core::FicheId, String> {
    referentiel
        .fiches()
        .find(|f| f.lien(source).is_some_and(|l| l.uid == uid))
        .map(Fiche::id)
        .ok_or_else(|| format!("aucune fiche liée à {uid}"))
}

/// Carnet d'une carte, importée, puis fiche modifiée dans le référentiel.
fn fiche_modifiee() -> Result<(Referentiel, SourceId, CarnetSimule), Box<dyn std::error::Error>> {
    let carnet = CarnetSimule::avec(&[("/carnet/a.vcf", &vcard("a", &["FN:Camille Exemple"]))]);
    let (mut referentiel, source) = referentiel();
    synchroniser(&mut referentiel, source, &carnet, date(1))?;
    let fiche = fiche_de(&referentiel, source, "a")?;
    if let Some(f) = referentiel.fiche_mut(fiche) {
        f.modifier(Champ::Fonction, &["Responsable"], date(2))?;
    }
    Ok((referentiel, source, carnet))
}

#[test]
fn chaque_carte_du_carnet_donne_une_fiche_liee_a_son_uid_et_a_son_etag() -> Essai {
    let carnet = CarnetSimule::avec(&[
        ("/carnet/1.vcf", &vcard("a", &["FN:Camille Exemple"])),
        ("/carnet/2.vcf", &vcard("b", &["FN:Dominique Fictif"])),
    ]);
    let (mut referentiel, source) = referentiel();

    synchroniser(&mut referentiel, source, &carnet, date(1))?;

    let mut liens: Vec<_> = referentiel
        .fiches()
        .filter_map(|f| f.lien(source))
        .map(|l| (l.uid.clone(), l.etag.clone()))
        .collect();
    liens.sort();
    assert_eq!(
        liens,
        [
            ("a".to_owned(), Some("\"v1\"".to_owned())),
            ("b".to_owned(), Some("\"v2\"".to_owned()))
        ]
    );
    Ok(())
}

#[test]
fn une_modification_du_referentiel_est_ecrite_au_meme_chemin_et_l_etag_suit() -> Essai {
    let (mut referentiel, source, carnet) = fiche_modifiee()?;

    let bilan = synchroniser(&mut referentiel, source, &carnet, date(3))?;

    let etag = referentiel
        .fiches()
        .find_map(|f| f.lien(source))
        .and_then(|l| l.etag.clone());
    assert_eq!(
        (
            bilan.ecrites,
            carnet
                .contenu("/carnet/a.vcf")
                .is_some_and(|c| c.contains("TITLE:Responsable")),
            etag
        ),
        (1, true, Some("\"v2\"".to_owned()))
    );
    Ok(())
}

#[test]
fn une_carte_modifiee_entre_lecture_et_ecriture_n_est_pas_ecrasee() -> Essai {
    let (mut referentiel, source, carnet) = fiche_modifiee()?;
    carnet
        .conflits
        .borrow_mut()
        .push("/carnet/a.vcf".to_owned());

    let bilan = synchroniser(&mut referentiel, source, &carnet, date(3))?;

    assert_eq!(
        (
            bilan
                .refus
                .iter()
                .map(|r| r.erreur.clone())
                .collect::<Vec<_>>(),
            carnet
                .contenu("/carnet/a.vcf")
                .is_some_and(|c| c.contains("TITLE"))
        ),
        (vec![ErreurCardDav::PreconditionEchouee], false)
    );
    Ok(())
}

#[test]
fn une_ecriture_refusee_repart_au_tour_suivant() -> Essai {
    let (mut referentiel, source, carnet) = fiche_modifiee()?;
    carnet
        .conflits
        .borrow_mut()
        .push("/carnet/a.vcf".to_owned());
    synchroniser(&mut referentiel, source, &carnet, date(3))?;

    let bilan = synchroniser(&mut referentiel, source, &carnet, date(4))?;

    assert_eq!(
        (
            bilan.ecrites,
            carnet
                .contenu("/carnet/a.vcf")
                .is_some_and(|c| c.contains("TITLE:Responsable"))
        ),
        (1, true)
    );
    Ok(())
}

#[test]
fn une_liaison_rompue_arrete_les_tentatives_suivantes() -> Essai {
    let carnet = CarnetSimule::avec(&[
        ("/carnet/1.vcf", &vcard("a", &["FN:Camille Exemple"])),
        ("/carnet/2.vcf", &vcard("b", &["FN:Dominique Fictif"])),
    ]);
    let (mut referentiel, source) = referentiel();
    synchroniser(&mut referentiel, source, &carnet, date(1))?;
    let ids: Vec<_> = referentiel.fiches().map(Fiche::id).collect();
    for id in ids {
        if let Some(f) = referentiel.fiche_mut(id) {
            f.modifier(Champ::Note, &["note"], date(2))?;
        }
    }
    *carnet.coupe.borrow_mut() = true;

    let bilan = synchroniser(&mut referentiel, source, &carnet, date(3))?;

    assert_eq!((bilan.refus.len(), *carnet.tentatives.borrow()), (2, 1));
    Ok(())
}

#[test]
fn une_suppression_programmee_supprime_la_carte_du_carnet() -> Essai {
    let carnet = CarnetSimule::avec(&[("/carnet/a.vcf", &vcard("a", &["FN:Camille Exemple"]))]);
    let (mut referentiel, source) = referentiel();
    synchroniser(&mut referentiel, source, &carnet, date(1))?;
    let fiche = fiche_de(&referentiel, source, "a")?;
    referentiel.supprimer_fiche(fiche, &[source])?;

    let bilan = synchroniser(&mut referentiel, source, &carnet, date(2))?;

    assert_eq!(
        (bilan.supprimees, carnet.contenu("/carnet/a.vcf")),
        (1, None)
    );
    Ok(())
}

#[test]
fn une_suppression_qui_n_a_pu_partir_est_reprogrammee() -> Essai {
    let carnet = CarnetSimule::avec(&[("/carnet/a.vcf", &vcard("a", &["FN:Camille Exemple"]))]);
    let (mut referentiel, source) = referentiel();
    synchroniser(&mut referentiel, source, &carnet, date(1))?;
    let fiche = fiche_de(&referentiel, source, "a")?;
    referentiel.supprimer_fiche(fiche, &[source])?;
    *carnet.coupe.borrow_mut() = true;

    synchroniser(&mut referentiel, source, &carnet, date(2))?;

    assert_eq!(
        referentiel
            .suppressions_en_attente(source)
            .collect::<Vec<_>>(),
        ["a"]
    );
    Ok(())
}

#[test]
fn une_carte_sans_uid_se_designe_par_son_chemin() -> Essai {
    let sans_uid = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Sacha Inconnu\r\nEND:VCARD\r\n";
    let carnet = CarnetSimule::avec(&[("/carnet/x.vcf", sans_uid)]);
    let (mut referentiel, source) = referentiel();

    synchroniser(&mut referentiel, source, &carnet, date(1))?;

    assert!(fiche_de(&referentiel, source, "/carnet/x.vcf").is_ok());
    Ok(())
}

#[test]
fn un_carnet_illisible_laisse_le_referentiel_inchange() -> Essai {
    struct CarnetEnPanne;
    impl Carnet for CarnetEnPanne {
        fn lister(&self) -> Result<Vec<Ressource>, ErreurCardDav> {
            Err(ErreurCardDav::Transport("hors ligne".to_owned()))
        }
        fn ecrire(
            &self,
            _: &str,
            _: &str,
            _: &Precondition,
        ) -> Result<Option<String>, ErreurCardDav> {
            Ok(None)
        }
        fn supprimer(&self, _: &str, _: Option<&str>) -> Result<(), ErreurCardDav> {
            Ok(())
        }
        fn href_nouveau(&self, uid: &str) -> String {
            uid.to_owned()
        }
    }
    let (mut referentiel, source) = referentiel();
    let avant = referentiel.clone();

    let resultat = synchroniser(&mut referentiel, source, &CarnetEnPanne, date(1));

    assert_eq!((resultat.is_err(), referentiel), (true, avant));
    Ok(())
}

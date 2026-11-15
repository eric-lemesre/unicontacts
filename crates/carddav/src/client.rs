//! Client HTTP d'un carnet CardDAV (ADR-0007).

use std::fmt;
use std::time::Duration;

use ureq::Agent;
use ureq::http::{Request, Response, StatusCode};

use crate::multistatus::{self, Ressource};
use crate::oauth::Jeton;
use crate::{Carnet, ErreurCardDav, Precondition};

/// Durée maximale d'une requête, transfert compris.
const DELAI: Duration = Duration::from_secs(120);

/// Taille maximale d'une réponse lue : un carnet de quelques milliers de
/// fiches avec photos.
const TAILLE_MAX: u64 = 256 * 1024 * 1024;

const REQUETE_LISTE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<C:addressbook-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
  <D:prop><D:getetag/><C:address-data/></D:prop>
</C:addressbook-query>"#;

const REQUETE_ETAGS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:"><D:prop><D:getetag/></D:prop></D:propfind>"#;

/// Cartes demandées par `addressbook-multiget` à la fois.
const LOT_MULTIGET: usize = 100;

/// Racine des carnets CardDAV de Google (ADR-0017).
const CARNETS_GOOGLE: &str = "https://www.googleapis.com/carddav/v1/principals/";

/// Manière de lister les cartes d'un carnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Liste {
    /// `REPORT addressbook-query` : cartes et ETags en une requête.
    Requete,
    /// `PROPFIND` des ETags, puis `REPORT addressbook-multiget` par lots :
    /// ce que sert Google (ADR-0017).
    Multiget,
}

/// Mot de passe d'un compte : son `Debug` ne l'affiche jamais.
#[derive(Clone, PartialEq, Eq)]
pub struct MotDePasse(String);

impl MotDePasse {
    /// Mot de passe donné par l'utilisateur (de préférence un mot de passe
    /// d'application).
    #[must_use]
    pub fn new(valeur: impl Into<String>) -> Self {
        Self(valeur.into())
    }
}

impl fmt::Debug for MotDePasse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MotDePasse(***)")
    }
}

/// Compte d'accès à un carnet CardDAV.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compte {
    /// URL de la collection du carnet (`https://…/addressbooks/…/contacts/`).
    pub carnet: String,
    /// Identifiant de connexion.
    pub utilisateur: String,
    /// Mot de passe.
    pub mot_de_passe: MotDePasse,
}

/// Client d'un carnet CardDAV.
pub struct ClientCardDav {
    agent: Agent,
    /// URL de la collection, terminée par `/`.
    carnet: String,
    /// `schéma://hôte[:port]` du carnet, pour les chemins absolus.
    origine: String,
    /// En-tête `Authorization` : il porte le mot de passe ou le jeton.
    autorisation: String,
    /// Manière de lister les cartes.
    liste: Liste,
}

impl fmt::Debug for ClientCardDav {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientCardDav")
            .field("carnet", &self.carnet)
            .finish_non_exhaustive()
    }
}

impl ClientCardDav {
    /// Client du carnet d'un compte.
    ///
    /// # Errors
    ///
    /// [`ErreurCardDav::Url`] si l'URL n'est pas en `http` ou `https`, ou si
    /// elle porte des identifiants : ceux-ci passent par le compte, jamais
    /// par l'URL, qui peut finir dans un message d'erreur.
    // [impl->req~source-carddav~1]
    pub fn new(compte: &Compte) -> Result<Self, ErreurCardDav> {
        let identifiants = format!("{}:{}", compte.utilisateur, compte.mot_de_passe.0);
        Self::construire(
            &compte.carnet,
            format!("Basic {}", base64(identifiants.as_bytes())),
            Liste::Requete,
        )
    }

    /// Client d'un carnet sous jeton d'accès OAuth (`Bearer`), qui liste
    /// ses cartes par `PROPFIND` puis `addressbook-multiget`, comme le sert
    /// Google (ADR-0017).
    ///
    /// # Errors
    ///
    /// [`ErreurCardDav::Url`] comme pour [`ClientCardDav::new`].
    // [impl->req~source-google~1]
    pub fn avec_jeton(carnet: &str, acces: &Jeton) -> Result<Self, ErreurCardDav> {
        Self::construire(
            carnet,
            format!("Bearer {}", acces.valeur()),
            Liste::Multiget,
        )
    }

    /// Le même client, qui liste ses cartes par `PROPFIND` puis
    /// `addressbook-multiget`.
    #[must_use]
    pub fn par_multiget(mut self) -> Self {
        self.liste = Liste::Multiget;
        self
    }

    /// Carnet par défaut du compte Google d'une adresse.
    ///
    /// # Errors
    ///
    /// [`ErreurCardDav::Url`] si l'adresse n'a pas la forme d'une adresse
    /// électronique, ou porte un caractère qui sortirait du chemin.
    // [impl->req~source-google~1]
    pub fn carnet_google(adresse: &str) -> Result<String, ErreurCardDav> {
        let adresse = adresse.trim();
        let valide = adresse.split_once('@').is_some_and(|(local, domaine)| {
            !local.is_empty() && domaine.contains('.') && !domaine.contains('@')
        }) && adresse
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '-' | '_' | '+'));
        if valide {
            Ok(format!("{CARNETS_GOOGLE}{adresse}/lists/default/"))
        } else {
            Err(ErreurCardDav::Url(format!(
                "adresse Google invalide : {adresse}"
            )))
        }
    }

    fn construire(carnet: &str, autorisation: String, liste: Liste) -> Result<Self, ErreurCardDav> {
        let invalide = || ErreurCardDav::Url(carnet.to_owned());
        let (schema, reste) = carnet.split_once("://").ok_or_else(invalide)?;
        if !matches!(schema, "http" | "https") {
            return Err(invalide());
        }
        let autorite = reste.split('/').next().unwrap_or_default();
        if autorite.is_empty() || autorite.contains('@') {
            return Err(invalide());
        }
        let mut carnet = carnet.to_owned();
        if !carnet.ends_with('/') {
            carnet.push('/');
        }
        let agent: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .allow_non_standard_methods(true)
            .timeout_global(Some(DELAI))
            .build()
            .into();
        Ok(Self {
            agent,
            origine: format!("{schema}://{autorite}"),
            carnet,
            autorisation,
            liste,
        })
    }

    /// Corps XML d'une réponse multistatus attendue.
    fn multistatus(
        &self,
        methode: &'static str,
        entetes: &[(&str, &str)],
        corps: &str,
    ) -> Result<String, ErreurCardDav> {
        let mut reponse = self.envoyer(methode, &self.carnet, entetes, corps)?;
        verifier(reponse.status(), methode, &[StatusCode::MULTI_STATUS])?;
        reponse
            .body_mut()
            .with_config()
            .limit(TAILLE_MAX)
            .read_to_string()
            .map_err(|e| ErreurCardDav::Transport(e.to_string()))
    }

    /// Cartes du carnet par `PROPFIND` puis `addressbook-multiget` par lots.
    // [impl->req~source-google~1]
    fn lister_par_multiget(&self) -> Result<Vec<Ressource>, ErreurCardDav> {
        let xml_type = ("Content-Type", "application/xml; charset=utf-8");
        let etags = multistatus::lire_etags(&self.multistatus(
            "PROPFIND",
            &[("Depth", "1"), xml_type],
            REQUETE_ETAGS,
        )?)?;
        let mut ressources = Vec::with_capacity(etags.len());
        for lot in etags.chunks(LOT_MULTIGET) {
            let mut corps = String::from(
                r#"<?xml version="1.0" encoding="utf-8"?>
<C:addressbook-multiget xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
  <D:prop><D:getetag/><C:address-data/></D:prop>"#,
            );
            for (href, _) in lot {
                corps.push_str("\n  <D:href>");
                corps.push_str(&echapper_xml(href));
                corps.push_str("</D:href>");
            }
            corps.push_str("\n</C:addressbook-multiget>");
            ressources.extend(multistatus::lire(&self.multistatus(
                "REPORT",
                &[("Depth", "0"), xml_type],
                &corps,
            )?)?);
        }
        Ok(ressources)
    }

    /// URL complète d'un chemin rendu par le serveur.
    fn url(&self, href: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            href.to_owned()
        } else if href.starts_with('/') {
            format!("{}{href}", self.origine)
        } else {
            format!("{}{href}", self.carnet)
        }
    }

    fn envoyer(
        &self,
        methode: &str,
        url: &str,
        entetes: &[(&str, &str)],
        corps: &str,
    ) -> Result<Response<ureq::Body>, ErreurCardDav> {
        let mut requete = Request::builder()
            .method(methode)
            .uri(url)
            .header("Authorization", &self.autorisation);
        for (nom, valeur) in entetes {
            requete = requete.header(*nom, *valeur);
        }
        let requete = requete
            .body(corps.to_owned())
            .map_err(|e| ErreurCardDav::Url(format!("{url} : {e}")))?;
        self.agent
            .run(requete)
            .map_err(|e| ErreurCardDav::Transport(e.to_string()))
    }
}

impl Carnet for ClientCardDav {
    // [impl->req~source-carddav~1]
    fn lister(&self) -> Result<Vec<Ressource>, ErreurCardDav> {
        match self.liste {
            Liste::Requete => {
                let entetes = [
                    ("Depth", "1"),
                    ("Content-Type", "application/xml; charset=utf-8"),
                ];
                multistatus::lire(&self.multistatus("REPORT", &entetes, REQUETE_LISTE)?)
            }
            Liste::Multiget => self.lister_par_multiget(),
        }
    }

    // [impl->req~source-carddav~1]
    fn ecrire(
        &self,
        href: &str,
        contenu: &str,
        precondition: &Precondition,
    ) -> Result<Option<String>, ErreurCardDav> {
        let mut entetes = vec![("Content-Type", "text/vcard; charset=utf-8")];
        match precondition {
            Precondition::Creation => entetes.push(("If-None-Match", "*")),
            Precondition::Remplacement(etag) => entetes.push(("If-Match", etag.as_str())),
            Precondition::Aucune => {}
        }
        let reponse = self.envoyer("PUT", &self.url(href), &entetes, contenu)?;
        verifier(
            reponse.status(),
            "PUT",
            &[StatusCode::OK, StatusCode::CREATED, StatusCode::NO_CONTENT],
        )?;
        Ok(reponse
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned))
    }

    // [impl->req~source-carddav~1]
    fn supprimer(&self, href: &str, etag: Option<&str>) -> Result<(), ErreurCardDav> {
        let entetes: Vec<(&str, &str)> = etag.map(|e| ("If-Match", e)).into_iter().collect();
        let reponse = self.envoyer("DELETE", &self.url(href), &entetes, "")?;
        // Une ressource déjà absente est l'effet voulu.
        verifier(
            reponse.status(),
            "DELETE",
            &[
                StatusCode::OK,
                StatusCode::NO_CONTENT,
                StatusCode::NOT_FOUND,
            ],
        )
    }

    fn href_nouveau(&self, uid: &str) -> String {
        let nom: String = uid
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        format!("{}{nom}.vcf", self.carnet)
    }
}

/// Contrôle le statut d'une réponse.
fn verifier(
    statut: StatusCode,
    methode: &'static str,
    attendus: &[StatusCode],
) -> Result<(), ErreurCardDav> {
    if attendus.contains(&statut) {
        Ok(())
    } else if statut == StatusCode::PRECONDITION_FAILED {
        Err(ErreurCardDav::PreconditionEchouee)
    } else {
        Err(ErreurCardDav::Statut {
            methode,
            code: statut.as_u16(),
        })
    }
}

/// Texte échappé pour un nœud XML.
fn echapper_xml(texte: &str) -> String {
    texte
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Encodage base64 (RFC 4648, § 4), pour l'en-tête `Authorization`.
fn base64(octets: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let symbole = |indice: u32| {
        ALPHABET
            .get((indice & 0x3f) as usize)
            .map_or('=', |&c| char::from(c))
    };
    let mut texte = String::with_capacity(octets.len().div_ceil(3) * 4);
    for bloc in octets.chunks(3) {
        let mut valeur = 0u32;
        for (i, octet) in bloc.iter().enumerate() {
            valeur |= u32::from(*octet) << (16 - 8 * i);
        }
        for i in 0..4 {
            if i <= bloc.len() {
                texte.push(symbole(valeur >> (18 - 6 * i)));
            } else {
                texte.push('=');
            }
        }
    }
    texte
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compte(carnet: &str) -> Compte {
        Compte {
            carnet: carnet.to_owned(),
            utilisateur: "camille".to_owned(),
            mot_de_passe: MotDePasse::new("s3cret"),
        }
    }

    #[test]
    fn base64_suit_les_vecteurs_de_la_rfc_4648() {
        let encodes: Vec<String> = ["", "f", "fo", "foo", "foob", "fooba", "foobar"]
            .into_iter()
            .map(|t| base64(t.as_bytes()))
            .collect();

        assert_eq!(
            encodes,
            [
                "", "Zg==", "Zm8=", "Zm9v", "Zm9vYg==", "Zm9vYmE=", "Zm9vYmFy"
            ]
        );
    }

    #[test]
    fn le_debug_d_un_compte_ne_montre_pas_le_mot_de_passe() {
        let texte = format!("{:?}", compte("https://exemple.org/carnet/"));

        assert!(!texte.contains("s3cret"), "{texte}");
    }

    #[test]
    fn le_debug_d_un_client_ne_montre_pas_l_autorisation() -> Result<(), ErreurCardDav> {
        let client = ClientCardDav::new(&compte("https://exemple.org/carnet/"))?;

        let texte = format!("{client:?}");

        assert!(!texte.contains("Basic"), "{texte}");
        Ok(())
    }

    // [utest->req~source-google~1]
    #[test]
    fn le_debug_d_un_client_sous_jeton_ne_montre_pas_le_jeton() -> Result<(), ErreurCardDav> {
        let client =
            ClientCardDav::avec_jeton("https://exemple.org/carnet/", &Jeton::new("ya29.secret"))?;

        let texte = format!("{client:?}");

        assert!(!texte.contains("secret"), "{texte}");
        Ok(())
    }

    // [utest->req~source-google~1]
    #[test]
    fn le_carnet_google_d_une_adresse_est_sa_liste_par_defaut() -> Result<(), ErreurCardDav> {
        assert_eq!(
            ClientCardDav::carnet_google(" alice.martin@gmail.com ")?,
            "https://www.googleapis.com/carddav/v1/principals/alice.martin@gmail.com/lists/default/"
        );
        Ok(())
    }

    // [utest->req~source-google~1]
    #[test]
    fn une_adresse_google_qui_sortirait_du_chemin_est_refusee() {
        let refusees: Vec<bool> = ["alice", "a/../b@gmail.com", "a@gmail.com?x", "@gmail.com"]
            .into_iter()
            .map(|a| matches!(ClientCardDav::carnet_google(a), Err(ErreurCardDav::Url(_))))
            .collect();

        assert_eq!(refusees, [true; 4]);
    }

    #[test]
    fn une_url_qui_porte_des_identifiants_est_refusee() {
        let resultat = ClientCardDav::new(&compte("https://camille:s3cret@exemple.org/carnet/"));

        assert!(matches!(resultat, Err(ErreurCardDav::Url(_))));
    }

    #[test]
    fn une_url_hors_http_est_refusee() {
        let resultat = ClientCardDav::new(&compte("ftp://exemple.org/carnet/"));

        assert!(matches!(resultat, Err(ErreurCardDav::Url(_))));
    }

    #[test]
    fn un_chemin_absolu_se_complete_par_l_origine_du_carnet() -> Result<(), ErreurCardDav> {
        let client = ClientCardDav::new(&compte("https://exemple.org:8443/dav/carnet"))?;

        assert_eq!(
            client.url("/dav/carnet/a.vcf"),
            "https://exemple.org:8443/dav/carnet/a.vcf"
        );
        Ok(())
    }

    #[test]
    fn un_nouveau_chemin_ne_garde_que_des_caracteres_surs() -> Result<(), ErreurCardDav> {
        let client = ClientCardDav::new(&compte("https://exemple.org/carnet/"))?;

        assert_eq!(
            client.href_nouveau("urn:uuid:a/b c"),
            "https://exemple.org/carnet/urn_uuid_a_b_c.vcf"
        );
        Ok(())
    }
}

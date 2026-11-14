//! Client HTTP d'un carnet CardDAV (ADR-0007).

use std::fmt;
use std::time::Duration;

use ureq::Agent;
use ureq::http::{Request, Response, StatusCode};

use crate::multistatus::{self, Ressource};
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
    /// En-tête `Authorization` : il porte le mot de passe.
    autorisation: String,
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
        let invalide = || ErreurCardDav::Url(compte.carnet.clone());
        let (schema, reste) = compte.carnet.split_once("://").ok_or_else(invalide)?;
        if !matches!(schema, "http" | "https") {
            return Err(invalide());
        }
        let autorite = reste.split('/').next().unwrap_or_default();
        if autorite.is_empty() || autorite.contains('@') {
            return Err(invalide());
        }
        let mut carnet = compte.carnet.clone();
        if !carnet.ends_with('/') {
            carnet.push('/');
        }
        let agent: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .allow_non_standard_methods(true)
            .timeout_global(Some(DELAI))
            .build()
            .into();
        let identifiants = format!("{}:{}", compte.utilisateur, compte.mot_de_passe.0);
        Ok(Self {
            agent,
            origine: format!("{schema}://{autorite}"),
            carnet,
            autorisation: format!("Basic {}", base64(identifiants.as_bytes())),
        })
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
        let entetes = [
            ("Depth", "1"),
            ("Content-Type", "application/xml; charset=utf-8"),
        ];
        let mut reponse = self.envoyer("REPORT", &self.carnet, &entetes, REQUETE_LISTE)?;
        verifier(reponse.status(), "REPORT", &[StatusCode::MULTI_STATUS])?;
        let xml = reponse
            .body_mut()
            .with_config()
            .limit(TAILLE_MAX)
            .read_to_string()
            .map_err(|e| ErreurCardDav::Transport(e.to_string()))?;
        multistatus::lire(&xml)
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

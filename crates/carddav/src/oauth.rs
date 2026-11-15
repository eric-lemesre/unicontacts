//! Autorisation OAuth 2.0 d'un compte Google pour son interface CardDAV
//! (ADR-0017) : client natif public, flux « code d'autorisation » avec
//! PKCE `S256` (RFC 7636), sans secret client ni services Play.

use std::fmt;
use std::time::Duration;

use ring::digest::{SHA256, digest};
use ring::rand::{SecureRandom, SystemRandom};
use ureq::Agent;
use ureq::http::Request;

/// Durée maximale d'un échange avec le serveur d'autorisation.
const DELAI: Duration = Duration::from_secs(60);

/// Taille maximale d'une réponse du serveur d'autorisation.
const TAILLE_MAX: u64 = 64 * 1024;

/// Seule portée demandée : les contacts, par CardDAV.
pub const PORTEE_CARDDAV: &str = "https://www.googleapis.com/auth/carddav";

/// Adresses du serveur d'autorisation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Points {
    /// Page d'autorisation, ouverte dans le navigateur.
    pub autorisation: String,
    /// Point d'échange des jetons.
    pub echange: String,
}

impl Points {
    /// Adresses de Google.
    #[must_use]
    pub fn google() -> Self {
        Self {
            autorisation: "https://accounts.google.com/o/oauth2/v2/auth".to_owned(),
            echange: "https://oauth2.googleapis.com/token".to_owned(),
        }
    }
}

/// Jeton OAuth (d'accès, de renouvellement) ou vérificateur PKCE : son
/// `Debug` ne l'affiche jamais.
#[derive(Clone, PartialEq, Eq)]
pub struct Jeton(String);

impl Jeton {
    /// Jeton reçu ou relu du coffre.
    #[must_use]
    pub fn new(valeur: impl Into<String>) -> Self {
        Self(valeur.into())
    }

    /// Valeur du jeton, pour l'en-tête qui le porte ou le coffre qui le garde.
    #[must_use]
    pub fn valeur(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Jeton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Jeton(***)")
    }
}

/// Client OAuth : identifiant client et adresse de retour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientOAuth {
    /// Identifiant du client, créé par l'utilisateur dans la console Google.
    pub identifiant: String,
    /// Adresse de retour, captée par l'application.
    pub retour: String,
    /// Adresses du serveur d'autorisation.
    pub points: Points,
}

/// Demande d'autorisation : la page à ouvrir, et ce qu'il faut garder pour
/// recevoir son retour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Demande {
    /// Page d'autorisation à ouvrir dans le navigateur.
    pub url: String,
    /// Vérificateur PKCE, à présenter à l'échange du code.
    pub verificateur: Jeton,
    /// Valeur de `state`, que le retour doit rendre à l'identique.
    pub etat: String,
}

/// Jetons rendus par un échange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jetons {
    /// Jeton d'accès, valable le temps indiqué.
    pub acces: Jeton,
    /// Jeton de renouvellement, s'il en est rendu un.
    pub renouvellement: Option<Jeton>,
    /// Durée de validité du jeton d'accès, en secondes.
    pub expire_dans: u64,
}

/// Erreur d'autorisation. Aucun message ne porte de jeton.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurOAuth {
    /// L'aléa du système est indisponible.
    #[error("aléa indisponible")]
    Alea,
    /// L'échange n'a pu aboutir (réseau, TLS, délai).
    #[error("échange impossible avec le serveur d'autorisation : {0}")]
    Transport(String),
    /// Le serveur refuse : `invalid_grant` pour un jeton révoqué ou expiré,
    /// qui demande une nouvelle autorisation.
    #[error("autorisation refusée par le serveur : {code}")]
    Refus {
        /// Code d'erreur OAuth (`invalid_grant`, `invalid_client`…).
        code: String,
    },
    /// La réponse du serveur ne se lit pas.
    #[error("réponse d'autorisation illisible : {0}")]
    ReponseIllisible(String),
}

impl ErreurOAuth {
    /// Vrai si l'utilisateur doit autoriser à nouveau l'application.
    #[must_use]
    pub fn autorisation_a_renouveler(&self) -> bool {
        matches!(self, Self::Refus { code } if code == "invalid_grant")
    }
}

impl ClientOAuth {
    /// Demande d'autorisation, avec un vérificateur PKCE et un `state`
    /// neufs ; `indice` préremplit l'adresse du compte.
    ///
    /// # Errors
    ///
    /// [`ErreurOAuth::Alea`] si l'aléa du système fait défaut.
    // [impl->req~source-google~1]
    pub fn demander(&self, indice: Option<&str>) -> Result<Demande, ErreurOAuth> {
        let verificateur = aleatoire(32)?;
        let etat = aleatoire(16)?;
        let mut parametres = vec![
            ("client_id", self.identifiant.as_str()),
            ("redirect_uri", self.retour.as_str()),
            ("response_type", "code"),
            ("scope", PORTEE_CARDDAV),
            ("access_type", "offline"),
            ("prompt", "consent"),
            ("code_challenge_method", "S256"),
            ("state", etat.as_str()),
        ];
        let defi = defi(&verificateur);
        parametres.push(("code_challenge", defi.as_str()));
        if let Some(adresse) = indice.filter(|a| !a.trim().is_empty()) {
            parametres.push(("login_hint", adresse));
        }
        Ok(Demande {
            url: format!("{}?{}", self.points.autorisation, formulaire(&parametres)),
            verificateur: Jeton(verificateur),
            etat,
        })
    }

    /// Échange le code reçu au retour contre les jetons.
    ///
    /// # Errors
    ///
    /// [`ErreurOAuth`] si l'échange échoue ou que le serveur refuse.
    // [impl->req~source-google~1]
    pub fn echanger(&self, code: &str, verificateur: &Jeton) -> Result<Jetons, ErreurOAuth> {
        self.poster(&[
            ("client_id", self.identifiant.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("code_verifier", verificateur.valeur()),
            ("redirect_uri", self.retour.as_str()),
        ])
    }

    /// Jeton d'accès neuf, d'après le jeton de renouvellement.
    ///
    /// # Errors
    ///
    /// [`ErreurOAuth::Refus`] `invalid_grant` si le jeton est révoqué ou
    /// expiré ; [`ErreurOAuth`] pour tout autre échec.
    // [impl->req~source-google~1]
    pub fn renouveler(&self, renouvellement: &Jeton) -> Result<Jeton, ErreurOAuth> {
        Ok(self
            .poster(&[
                ("client_id", self.identifiant.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", renouvellement.valeur()),
            ])?
            .acces)
    }

    fn poster(&self, parametres: &[(&str, &str)]) -> Result<Jetons, ErreurOAuth> {
        let agent: Agent = Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(DELAI))
            .build()
            .into();
        let requete = Request::builder()
            .method("POST")
            .uri(&self.points.echange)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body(formulaire(parametres))
            .map_err(|e| ErreurOAuth::Transport(e.to_string()))?;
        let mut reponse = agent
            .run(requete)
            .map_err(|e| ErreurOAuth::Transport(e.to_string()))?;
        let texte = reponse
            .body_mut()
            .with_config()
            .limit(TAILLE_MAX)
            .read_to_string()
            .map_err(|e| ErreurOAuth::Transport(e.to_string()))?;
        lire_reponse(&texte)
    }
}

/// Jetons d'une réponse du point d'échange, ou le refus qu'elle porte.
///
/// # Errors
///
/// [`ErreurOAuth::Refus`] pour une réponse d'erreur ;
/// [`ErreurOAuth::ReponseIllisible`] pour une réponse qui n'est pas du
/// JSON attendu.
pub fn lire_reponse(texte: &str) -> Result<Jetons, ErreurOAuth> {
    let json: serde_json::Value =
        serde_json::from_str(texte).map_err(|e| ErreurOAuth::ReponseIllisible(e.to_string()))?;
    if let Some(code) = json.get("error").and_then(serde_json::Value::as_str) {
        return Err(ErreurOAuth::Refus {
            code: code.to_owned(),
        });
    }
    let acces = json
        .get("access_token")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ErreurOAuth::ReponseIllisible("access_token absent".to_owned()))?;
    Ok(Jetons {
        acces: Jeton::new(acces),
        renouvellement: json
            .get("refresh_token")
            .and_then(serde_json::Value::as_str)
            .map(Jeton::new),
        expire_dans: json
            .get("expires_in")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
    })
}

/// Défi PKCE `S256` d'un vérificateur : `BASE64URL(SHA256(vérificateur))`.
#[must_use]
pub fn defi(verificateur: &str) -> String {
    base64url(digest(&SHA256, verificateur.as_bytes()).as_ref())
}

/// Texte aléatoire de `octets` octets, en base64url sans remplissage.
fn aleatoire(octets: usize) -> Result<String, ErreurOAuth> {
    let mut tampon = vec![0_u8; octets];
    SystemRandom::new()
        .fill(&mut tampon)
        .map_err(|_| ErreurOAuth::Alea)?;
    Ok(base64url(&tampon))
}

/// Encodage base64url sans remplissage (RFC 4648, § 5).
fn base64url(octets: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut texte = String::with_capacity(octets.len().div_ceil(3) * 4);
    for bloc in octets.chunks(3) {
        let n = bloc
            .iter()
            .enumerate()
            .fold(0_u32, |n, (i, o)| n | (u32::from(*o) << (16 - 8 * i)));
        for i in 0..=bloc.len() {
            let indice = (n >> (18 - 6 * i)) & 0x3f;
            texte.extend(
                usize::try_from(indice)
                    .ok()
                    .and_then(|i| ALPHABET.get(i))
                    .map(|o| char::from(*o)),
            );
        }
    }
    texte
}

/// Corps `application/x-www-form-urlencoded` (ou requête d'URL).
fn formulaire(parametres: &[(&str, &str)]) -> String {
    parametres
        .iter()
        .map(|(nom, valeur)| format!("{}={}", encoder(nom), encoder(valeur)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Encodage pourcent : seuls les caractères non réservés restent en clair.
fn encoder(texte: &str) -> String {
    let mut encode = String::with_capacity(texte.len());
    for octet in texte.bytes() {
        if octet.is_ascii_alphanumeric() || matches!(octet, b'-' | b'.' | b'_' | b'~') {
            encode.push(char::from(octet));
        } else {
            encode.push_str(&format!("%{octet:02X}"));
        }
    }
    encode
}

#[cfg(test)]
mod tests;

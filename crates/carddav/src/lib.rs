//! Connecteur CardDAV d'unicontacts : relie le référentiel à un carnet
//! d'adresses CardDAV (Murena, Nextcloud), dans les deux sens.
//!
//! Le connecteur fait le travail réseau autour du moteur du cœur : il liste
//! les cartes du carnet, confie la fusion au moteur, puis écrit vers le
//! carnet ce que le moteur demande, sous précondition `If-Match` pour ne
//! jamais écraser une carte modifiée entre-temps. Une écriture refusée est
//! annulée dans le référentiel et repart au tour suivant. Aucune transaction
//! de stockage n'est ouverte pendant ce travail : l'appelant charge le
//! référentiel avant et l'enregistre après (ADR-0007).
// [impl->req~source-carddav~1]

mod client;
mod multistatus;
pub mod oauth;
mod synchro;

pub use client::{ClientCardDav, Compte, MotDePasse};
pub use multistatus::Ressource;
pub use synchro::{
    Bilan, ErreurConnecteur, Issue, Lecture, Refus, envoyer, lire_carnet, reporter, synchroniser,
};

/// Erreur d'échange avec un carnet CardDAV.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurCardDav {
    /// L'URL du carnet est invalide ou porte des identifiants.
    #[error("URL de carnet invalide : {0}")]
    Url(String),
    /// La requête n'a pu aboutir (réseau, TLS, délai).
    #[error("échange impossible avec le carnet : {0}")]
    Transport(String),
    /// La carte a changé sur le serveur depuis sa lecture (412).
    #[error("la carte a changé sur le serveur depuis sa lecture")]
    PreconditionEchouee,
    /// Le serveur a répondu par un statut inattendu.
    #[error("{methode} refusé par le serveur (statut {code})")]
    Statut {
        /// Méthode HTTP.
        methode: &'static str,
        /// Statut reçu.
        code: u16,
    },
    /// La réponse du serveur ne se lit pas.
    #[error("réponse illisible : {0}")]
    ReponseIllisible(String),
}

impl ErreurCardDav {
    /// Vrai si l'erreur tient à la liaison elle-même : les requêtes
    /// suivantes échoueraient de même.
    #[must_use]
    pub const fn liaison_rompue(&self) -> bool {
        matches!(self, Self::Transport(_) | Self::Url(_))
    }
}

/// Condition posée à une écriture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Precondition {
    /// La carte ne doit pas exister (`If-None-Match: *`).
    Creation,
    /// La carte doit être encore dans la version lue (`If-Match`).
    Remplacement(String),
    /// Aucune condition : carte existante dont le serveur n'a pas donné
    /// d'ETag, qu'aucune précondition ne peut protéger.
    Aucune,
}

/// Ce qu'un carnet sait faire : le connecteur n'en demande pas plus, ce qui
/// permet de l'éprouver contre un carnet simulé comme contre un serveur.
pub trait Carnet {
    /// Toutes les cartes du carnet.
    ///
    /// # Errors
    ///
    /// [`ErreurCardDav`] si le carnet ne répond pas comme attendu.
    fn lister(&self) -> Result<Vec<Ressource>, ErreurCardDav>;

    /// Écrit une carte et rend son nouvel ETag, si le serveur le donne.
    ///
    /// # Errors
    ///
    /// [`ErreurCardDav::PreconditionEchouee`] si la carte a changé depuis
    /// sa lecture ; autre [`ErreurCardDav`] sinon.
    fn ecrire(
        &self,
        href: &str,
        contenu: &str,
        precondition: &Precondition,
    ) -> Result<Option<String>, ErreurCardDav>;

    /// Supprime une carte ; une carte déjà absente n'est pas une erreur.
    ///
    /// # Errors
    ///
    /// [`ErreurCardDav::PreconditionEchouee`] si la carte a changé depuis
    /// sa lecture ; autre [`ErreurCardDav`] sinon.
    fn supprimer(&self, href: &str, etag: Option<&str>) -> Result<(), ErreurCardDav>;

    /// Chemin d'une carte à créer, d'après son UID.
    fn href_nouveau(&self, uid: &str) -> String;
}

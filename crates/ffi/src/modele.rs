//! Types échangés avec Kotlin, et leur traduction depuis le cœur.
// [impl->req~vue-comparee~1]

use unicontacts_carddav::Bilan;
use unicontacts_core::decisions::{
    Action, Choix, Decision, IssueDoublon, IssueSuppression, Nature, Resolution, Statut,
};
use unicontacts_core::referentiel::Referentiel;
use unicontacts_core::reglages::{Source, Sources};
use unicontacts_core::vcard::Version;
use unicontacts_core::{Champ, Fiche, Origine, Valeur};

/// Nom de l'origine « référentiel », tel que l'affiche l'application.
const REFERENTIEL: &str = "unicontacts";

/// Champ d'une fiche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ChampFfi {
    /// Nom affiché.
    NomAffiche,
    /// Nom structuré : nom, prénom, autres prénoms, préfixe, suffixe.
    NomStructure,
    /// Organisation.
    Organisation,
    /// Fonction.
    Fonction,
    /// Note.
    Note,
    /// Date de naissance.
    Anniversaire,
    /// Téléphones.
    Telephone,
    /// E-mails.
    Email,
    /// Adresses postales.
    Adresse,
    /// Adresses web.
    Url,
}

impl From<ChampFfi> for Champ {
    fn from(champ: ChampFfi) -> Self {
        match champ {
            ChampFfi::NomAffiche => Self::NomAffiche,
            ChampFfi::NomStructure => Self::NomStructure,
            ChampFfi::Organisation => Self::Organisation,
            ChampFfi::Fonction => Self::Fonction,
            ChampFfi::Note => Self::Note,
            ChampFfi::Anniversaire => Self::Anniversaire,
            ChampFfi::Telephone => Self::Telephone,
            ChampFfi::Email => Self::Email,
            ChampFfi::Adresse => Self::Adresse,
            ChampFfi::Url => Self::Url,
        }
    }
}

impl From<Champ> for ChampFfi {
    fn from(champ: Champ) -> Self {
        match champ {
            Champ::NomAffiche => Self::NomAffiche,
            Champ::NomStructure => Self::NomStructure,
            Champ::Organisation => Self::Organisation,
            Champ::Fonction => Self::Fonction,
            Champ::Note => Self::Note,
            Champ::Anniversaire => Self::Anniversaire,
            Champ::Telephone => Self::Telephone,
            Champ::Email => Self::Email,
            Champ::Adresse => Self::Adresse,
            Champ::Url => Self::Url,
        }
    }
}

/// Valeur d'un champ, avec sa provenance.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ValeurFfi {
    /// Valeur.
    pub texte: String,
    /// Types (`home`, `work`, `cell`…).
    pub types: Vec<String>,
    /// Nom de la source d'où vient la valeur, ou « unicontacts ».
    pub origine: String,
    /// Date de sa dernière modification, en millisecondes depuis 1970.
    pub date_millis: i64,
}

/// Valeurs d'un champ.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ChampValeursFfi {
    /// Champ.
    pub champ: ChampFfi,
    /// Valeurs, dans leur ordre.
    pub valeurs: Vec<ValeurFfi>,
}

/// Fiche en entier.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FicheFfi {
    /// Identifiant stable.
    pub id: String,
    /// Nom à afficher.
    pub nom: String,
    /// Champs renseignés.
    pub champs: Vec<ChampValeursFfi>,
    /// Noms des sources auxquelles la fiche est liée.
    pub sources: Vec<String>,
}

/// Résumé d'une fiche, pour une liste.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ResumeFicheFfi {
    /// Identifiant stable.
    pub id: String,
    /// Nom à afficher.
    pub nom: String,
    /// Premier téléphone.
    pub telephone: Option<String>,
    /// Premier e-mail.
    pub email: Option<String>,
}

/// Source configurée.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SourceFfi {
    /// Identifiant.
    pub id: String,
    /// Nom donné par l'utilisateur.
    pub nom: String,
    /// Vrai si la source sert du vCard 4.0.
    pub vcard4: bool,
}

/// Ce sur quoi porte une décision.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum NatureFfi {
    /// Un champ a changé différemment des deux côtés.
    Divergence {
        /// Champ concerné.
        champ: ChampFfi,
    },
    /// La fiche a disparu de la source.
    Suppression,
    /// La fiche ressemble à une autre, sans certitude.
    DoublonProbable {
        /// Identifiant de l'autre fiche.
        autre_fiche: String,
        /// Nom de l'autre fiche.
        autre_nom: String,
    },
    /// Nouvelle photo proposée.
    NouvellePhoto,
}

/// Statut d'une décision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum StatutFfi {
    /// À trancher.
    EnAttente,
    /// Remise à plus tard.
    Reportee,
    /// Tranchée.
    Resolue,
}

/// Choix offert pour une décision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ChoixFfi {
    /// Garder la valeur du référentiel.
    GarderReferentiel,
    /// Garder la valeur de la source.
    GarderSource,
    /// Garder les deux.
    GarderLesDeux,
    /// Saisir une valeur.
    Saisir,
    /// Propager la suppression.
    Propager,
    /// Restaurer la fiche sur la source.
    Restaurer,
    /// Détacher la fiche de la source.
    Detacher,
    /// Réunir les deux fiches.
    Fusionner,
    /// Garder deux fiches.
    Distinguer,
    /// Remettre à plus tard.
    Reporter,
}

/// Décision, avec tout ce que la vue comparée affiche.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DecisionFfi {
    /// Identifiant.
    pub id: String,
    /// Nature.
    pub nature: NatureFfi,
    /// Fiche concernée.
    pub fiche_id: String,
    /// Nom de la fiche.
    pub fiche_nom: String,
    /// Nom de la source concernée.
    pub source_nom: String,
    /// Date de création, en millisecondes depuis 1970.
    pub creee_le_millis: i64,
    /// Statut.
    pub statut: StatutFfi,
    /// Choix offerts.
    pub choix: Vec<ChoixFfi>,
    /// Valeurs du référentiel, avec leur origine et leur date.
    pub referentiel: Vec<ValeurFfi>,
    /// Valeurs de la source, avec la source et la date de réception.
    pub source: Vec<ValeurFfi>,
    /// Valeurs du dernier état synchronisé.
    pub base: Vec<String>,
}

/// Résolution choisie.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum ResolutionFfi {
    /// Garder la valeur du référentiel.
    GarderReferentiel,
    /// Garder la valeur de la source.
    GarderSource,
    /// Garder les deux.
    GarderLesDeux,
    /// Retenir des valeurs saisies.
    Saisir {
        /// Valeurs saisies.
        valeurs: Vec<String>,
    },
    /// Propager la suppression partout.
    Propager,
    /// Restaurer la fiche sur la source.
    Restaurer,
    /// Détacher la fiche de la source.
    Detacher,
    /// Réunir les deux fiches.
    Fusionner,
    /// Garder deux fiches.
    Distinguer,
}

impl From<ResolutionFfi> for Resolution {
    fn from(resolution: ResolutionFfi) -> Self {
        match resolution {
            ResolutionFfi::GarderReferentiel => Self::Divergence(Action::GarderReferentiel),
            ResolutionFfi::GarderSource => Self::Divergence(Action::GarderSource),
            ResolutionFfi::GarderLesDeux => Self::Divergence(Action::GarderLesDeux),
            ResolutionFfi::Saisir { valeurs } => Self::Divergence(Action::Saisir(valeurs)),
            ResolutionFfi::Propager => Self::Suppression(IssueSuppression::Propager),
            ResolutionFfi::Restaurer => Self::Suppression(IssueSuppression::Restaurer),
            ResolutionFfi::Detacher => Self::Suppression(IssueSuppression::Detacher),
            ResolutionFfi::Fusionner => Self::Doublon(IssueDoublon::Fusionner),
            ResolutionFfi::Distinguer => Self::Doublon(IssueDoublon::Distinguer),
        }
    }
}

/// Bilan d'une synchronisation.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct BilanFfi {
    /// Cartes écrites sur le serveur.
    pub ecrites: u32,
    /// Cartes supprimées du serveur.
    pub supprimees: u32,
    /// Écritures refusées, qui repartiront au tour suivant.
    pub refusees: u32,
    /// Décisions créées ou mises à jour.
    pub decisions: u32,
    /// Cartes illisibles laissées en l'état.
    pub illisibles: u32,
}

fn compte(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

pub(crate) fn bilan(bilan: &Bilan) -> BilanFfi {
    BilanFfi {
        ecrites: compte(bilan.ecrites),
        supprimees: compte(bilan.supprimees),
        refusees: compte(bilan.refus.len()),
        decisions: compte(bilan.rapport.decisions.len()),
        illisibles: compte(bilan.rapport.illisibles.len()),
    }
}

/// Nom à afficher : nom affiché, sinon prénom et nom, sinon organisation,
/// sinon premier e-mail ou téléphone.
pub(crate) fn nom(fiche: &Fiche) -> String {
    let premier = |champ: Champ| fiche.valeurs(champ).first().map(|v| v.texte.clone());
    premier(Champ::NomAffiche)
        .or_else(|| {
            premier(Champ::NomStructure).map(|n| {
                let parties: Vec<&str> = n.split(';').collect();
                [parties.get(1), parties.first()]
                    .into_iter()
                    .flatten()
                    .filter(|p| !p.trim().is_empty())
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
        })
        .filter(|n| !n.trim().is_empty())
        .or_else(|| {
            premier(Champ::Organisation).map(|o| o.split(';').next().unwrap_or_default().to_owned())
        })
        .or_else(|| premier(Champ::Email))
        .or_else(|| premier(Champ::Telephone))
        .unwrap_or_else(|| "(sans nom)".to_owned())
}

fn valeur(valeur: &Valeur, sources: &Sources) -> ValeurFfi {
    let origine = match valeur.provenance.origine {
        Origine::Referentiel => REFERENTIEL.to_owned(),
        Origine::Source(id) => sources
            .source(id)
            .map_or_else(|| id.to_string(), |s| s.nom.clone()),
    };
    ValeurFfi {
        texte: valeur.texte.clone(),
        types: valeur.types.clone(),
        origine,
        date_millis: valeur.provenance.date.millis(),
    }
}

pub(crate) fn resume(fiche: &Fiche) -> ResumeFicheFfi {
    let premier = |champ: Champ| fiche.valeurs(champ).first().map(|v| v.texte.clone());
    ResumeFicheFfi {
        id: fiche.id().to_string(),
        nom: nom(fiche),
        telephone: premier(Champ::Telephone),
        email: premier(Champ::Email),
    }
}

pub(crate) fn fiche(fiche: &Fiche, sources: &Sources) -> FicheFfi {
    FicheFfi {
        id: fiche.id().to_string(),
        nom: nom(fiche),
        champs: fiche
            .champs()
            .map(|(champ, valeurs)| ChampValeursFfi {
                champ: champ.into(),
                valeurs: valeurs.iter().map(|v| valeur(v, sources)).collect(),
            })
            .collect(),
        sources: fiche
            .liens()
            .filter_map(|l| sources.source(l.source))
            .map(|s| s.nom.clone())
            .collect(),
    }
}

pub(crate) fn source(source: &Source) -> SourceFfi {
    SourceFfi {
        id: source.id.to_string(),
        nom: source.nom.clone(),
        vcard4: source.version == Version::V4,
    }
}

fn choix(choix: Choix) -> ChoixFfi {
    match choix {
        Choix::GarderReferentiel => ChoixFfi::GarderReferentiel,
        Choix::GarderSource => ChoixFfi::GarderSource,
        Choix::GarderLesDeux => ChoixFfi::GarderLesDeux,
        Choix::Saisir => ChoixFfi::Saisir,
        Choix::Propager => ChoixFfi::Propager,
        Choix::Restaurer => ChoixFfi::Restaurer,
        Choix::Detacher => ChoixFfi::Detacher,
        Choix::Fusionner => ChoixFfi::Fusionner,
        Choix::Distinguer => ChoixFfi::Distinguer,
        Choix::Reporter => ChoixFfi::Reporter,
    }
}

/// Décision traduite : pour une divergence, les valeurs du référentiel
/// viennent de la fiche, avec leur provenance ; celles de la source portent
/// le nom de la source et la date de la décision, qui est celle de leur
/// réception.
pub(crate) fn decision(decision: &Decision, referentiel: &Referentiel) -> DecisionFfi {
    let sources = referentiel.sources();
    let source_nom = sources
        .source(decision.source)
        .map_or_else(|| decision.source.to_string(), |s| s.nom.clone());
    let fiche = referentiel.fiche(decision.fiche);
    let recu = |textes: &[String], origine: &str| -> Vec<ValeurFfi> {
        textes
            .iter()
            .map(|t| ValeurFfi {
                texte: t.clone(),
                types: Vec::new(),
                origine: origine.to_owned(),
                date_millis: decision.creee_le.millis(),
            })
            .collect()
    };
    let nature = match decision.nature {
        Nature::Divergence { champ } => NatureFfi::Divergence {
            champ: champ.into(),
        },
        Nature::Suppression => NatureFfi::Suppression,
        Nature::DoublonProbable { autre } => NatureFfi::DoublonProbable {
            autre_fiche: autre.to_string(),
            autre_nom: referentiel.fiche(autre).map(nom).unwrap_or_default(),
        },
        Nature::NouvellePhoto => NatureFfi::NouvellePhoto,
    };
    let referentiel_valeurs = match (decision.nature, fiche) {
        (Nature::Divergence { champ }, Some(fiche)) => fiche
            .valeurs(champ)
            .iter()
            .map(|v| valeur(v, sources))
            .collect(),
        _ => recu(&decision.versions.referentiel, REFERENTIEL),
    };
    DecisionFfi {
        id: decision.id.to_string(),
        nature,
        fiche_id: decision.fiche.to_string(),
        fiche_nom: fiche.map(nom).unwrap_or_default(),
        source_nom: source_nom.clone(),
        creee_le_millis: decision.creee_le.millis(),
        statut: match decision.statut {
            Statut::EnAttente => StatutFfi::EnAttente,
            Statut::Reportee => StatutFfi::Reportee,
            Statut::Resolue => StatutFfi::Resolue,
        },
        choix: decision.choix().into_iter().map(choix).collect(),
        referentiel: referentiel_valeurs,
        source: recu(&decision.versions.source, &source_nom),
        base: decision.versions.base.clone(),
    }
}

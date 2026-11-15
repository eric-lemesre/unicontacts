//! Types échangés avec Kotlin, et leur traduction depuis le cœur.
// [impl->req~vue-comparee~1]

use std::collections::BTreeSet;
use unicontacts_carddav::Bilan;
use unicontacts_core::decisions::{
    Action, Choix, Decision, IssueDoublon, IssueSuppression, Nature, Resolution, Statut,
};
use unicontacts_core::referentiel::Referentiel;
use unicontacts_core::reglages::{Source, Sources};
use unicontacts_core::vcard::Version;

use unicontacts_core::canaux::Contexte;
use unicontacts_core::dates::DateDeFiche;
use unicontacts_core::groupes::{Combinaison, Filtre};
use unicontacts_core::relations::{Relation, TypeRelation};
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
    /// Relations : présentées à part ([`FicheFfi::relations`]), jamais
    /// éditées comme un champ.
    Relation,
    /// Groupes, par leur chemin : présentés à part ([`FicheFfi::groupes`]).
    Groupes,
    /// Dates autres que la naissance : présentées à part
    /// ([`FicheFfi::dates`]).
    Dates,
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
            ChampFfi::Relation => Self::Relation,
            ChampFfi::Groupes => Self::Groupes,
            ChampFfi::Dates => Self::Dates,
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
            Champ::Relation => Self::Relation,
            Champ::Groupes => Self::Groupes,
            Champ::Dates => Self::Dates,
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
    /// Relations vers d'autres fiches, dans leur ordre.
    pub relations: Vec<RelationFfi>,
    /// Vrai pour la fiche du propriétaire.
    pub est_moi: bool,
    /// Groupes de la fiche, par leur chemin.
    pub groupes: Vec<String>,
    /// Dates de la fiche, naissance comprise.
    pub dates: Vec<DateFfi>,
    /// Canaux disponibles, dans leur ordre.
    pub canaux: Vec<CanalFfi>,
    /// Classement pro effectif, par clés de canal.
    pub classement_pro: Vec<String>,
    /// Classement perso effectif, par clés de canal.
    pub classement_perso: Vec<String>,
}

/// Canal disponible d'une fiche.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct CanalFfi {
    /// Clé du canal : type et identifiant normalisé.
    pub cle: String,
    /// Code du type (`appel`, `sms`, `email`, `signal`…).
    pub code: String,
    /// Libellé du type.
    pub libelle: String,
    /// Numéro, adresse ou pseudonyme.
    pub identifiant: String,
    /// Note libre.
    pub note: Option<String>,
    /// Vrai pour une messagerie déclarée.
    pub declare: bool,
    /// Vrai si c'est le canal préféré pour une communication pro.
    pub prefere_pro: bool,
    /// Vrai si c'est le canal préféré pour une communication perso.
    pub prefere_perso: bool,
}

/// Date d'une fiche.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DateFfi {
    /// Code du type : `naissance`, `mariage`, `fete`, ou le libellé d'une
    /// date personnalisée.
    pub code: String,
    /// Libellé du type.
    pub libelle: String,
    /// Date : `AAAA-MM-JJ`, ou `--MM-JJ` sans année.
    pub texte: String,
    /// Année, si elle est connue.
    pub annee: Option<i32>,
    /// Mois, de 1 à 12.
    pub mois: u8,
    /// Jour, de 1 à 31.
    pub jour: u8,
}

/// Date à venir d'une fiche.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DateAVenirFfi {
    /// Fiche concernée.
    pub fiche_id: String,
    /// Nom de la fiche.
    pub nom: String,
    /// Date.
    pub date: DateFfi,
    /// Jours d'ici la prochaine occurrence ; 0 : aujourd'hui.
    pub dans_jours: u32,
    /// Âge ou années atteintes à cette occurrence, si l'année est connue.
    pub annees: Option<u32>,
}

/// Date traduite.
pub(crate) fn date(date: &DateDeFiche) -> DateFfi {
    DateFfi {
        code: date.type_date.code().to_owned(),
        libelle: date.type_date.libelle().to_owned(),
        texte: date.date.texte(),
        annee: date.date.annee,
        mois: date.date.mois,
        jour: date.date.jour,
    }
}

/// Texte lisible d'une valeur du champ `Dates` : « Fête du prénom : --05-18 ».
fn date_lisible(texte: &str) -> String {
    DateDeFiche::depuis_texte(texte).map_or_else(
        || texte.to_owned(),
        |d| format!("{} : {}", d.type_date.libelle(), d.date.texte()),
    )
}

/// Filtre de la liste des fiches.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FiltreFfi {
    /// Groupes retenus, par leur chemin ; aucun : pas de filtre de groupe.
    pub groupes: Vec<String>,
    /// Vrai pour les fiches de tous les groupes à la fois ; sinon de l'un
    /// d'eux.
    pub intersection: bool,
    /// Vrai si un sous-groupe compte pour son groupe.
    pub sous_groupes: bool,
    /// Texte cherché dans le nom.
    pub recherche: String,
    /// Code du canal dont les fiches doivent disposer, s'il y en a un.
    pub canal: Option<String>,
}

impl From<FiltreFfi> for Filtre {
    fn from(filtre: FiltreFfi) -> Self {
        Self {
            groupes: filtre.groupes,
            combinaison: if filtre.intersection {
                Combinaison::Intersection
            } else {
                Combinaison::Union
            },
            sous_groupes: filtre.sous_groupes,
            recherche: filtre.recherche,
            canal: filtre.canal,
        }
    }
}

/// Relation d'une fiche vers une autre.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RelationFfi {
    /// Code du type : mot fixe d'un type prédéfini, ou libellé.
    pub code: String,
    /// Libellé du type.
    pub libelle: String,
    /// Identifiant de la fiche liée.
    pub fiche_id: String,
    /// Nom de la fiche liée.
    pub nom: String,
}

/// Type de relation proposé à l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct TypeRelationFfi {
    /// Code du type : mot fixe d'un type prédéfini, ou libellé.
    pub code: String,
    /// Libellé du type.
    pub libelle: String,
    /// Libellé de son inverse, s'il en a un.
    pub inverse: Option<String>,
    /// Vrai pour un type prédéfini.
    pub predefini: bool,
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

pub(crate) fn fiche(fiche: &Fiche, referentiel: &Referentiel) -> FicheFfi {
    let sources = referentiel.sources();
    FicheFfi {
        id: fiche.id().to_string(),
        nom: nom(fiche),
        champs: fiche
            .champs()
            .filter(|(champ, _)| !matches!(champ, Champ::Relation | Champ::Groupes | Champ::Dates))
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
        relations: referentiel
            .relations(fiche.id())
            .iter()
            .map(|r| relation(r, referentiel))
            .collect(),
        est_moi: referentiel.moi() == Some(fiche.id()),
        groupes: fiche.textes(Champ::Groupes),
        dates: referentiel.dates(fiche.id()).iter().map(date).collect(),
        canaux: canaux(fiche, referentiel),
        classement_pro: referentiel.classement(fiche.id(), Contexte::Pro),
        classement_perso: referentiel.classement(fiche.id(), Contexte::Perso),
    }
}

/// Canaux disponibles d'une fiche, préférés pro et perso marqués.
// [impl->req~actions-triees-par-preference~2]
fn canaux(fiche: &Fiche, referentiel: &Referentiel) -> Vec<CanalFfi> {
    let prefere = |contexte| {
        referentiel
            .canal_prefere(fiche.id(), contexte)
            .map(|c| c.cle())
    };
    let (pro, perso) = (prefere(Contexte::Pro), prefere(Contexte::Perso));
    referentiel
        .canaux(fiche.id())
        .into_iter()
        .map(|d| {
            let cle = d.canal.cle();
            CanalFfi {
                prefere_pro: pro.as_ref() == Some(&cle),
                prefere_perso: perso.as_ref() == Some(&cle),
                cle,
                code: d.canal.type_canal.code().to_owned(),
                libelle: d.canal.type_canal.libelle().to_owned(),
                identifiant: d.canal.identifiant,
                note: d.note,
                declare: d.declare,
            }
        })
        .collect()
}

/// Relation traduite, avec le nom de la fiche liée.
// [impl->req~synchronisation-relations~1]
pub(crate) fn relation(relation: &Relation, referentiel: &Referentiel) -> RelationFfi {
    RelationFfi {
        code: relation.type_relation.jeton().to_owned(),
        libelle: relation.type_relation.libelle().to_owned(),
        fiche_id: relation.fiche.to_string(),
        nom: referentiel
            .fiche(relation.fiche)
            .map_or_else(|| "(fiche disparue)".to_owned(), nom),
    }
}

/// Texte lisible d'une valeur de relation : « Parent : Basile ».
fn relation_lisible(texte: &str, referentiel: &Referentiel) -> String {
    Relation::depuis_texte(texte).map_or_else(
        || texte.to_owned(),
        |r| {
            let lue = relation(&r, referentiel);
            format!("{} : {}", lue.libelle, lue.nom)
        },
    )
}

/// Types de relation à proposer : les prédéfinis, puis les personnalisés
/// déjà employés ou dotés d'un inverse, par ordre alphabétique.
pub(crate) fn types_relation(referentiel: &Referentiel) -> Vec<TypeRelationFfi> {
    let inverses = referentiel.inverses();
    let mut personnalises: BTreeSet<String> = inverses
        .couples()
        .flat_map(|(a, b)| [a.to_owned(), b.to_owned()])
        .collect();
    for fiche in referentiel.fiches() {
        for r in referentiel.relations(fiche.id()) {
            if let TypeRelation::Personnalise(libelle) = r.type_relation {
                personnalises.insert(libelle);
            }
        }
    }
    let predefinis = TypeRelation::PREDEFINIS.into_iter();
    let autres = personnalises
        .iter()
        .map(|l| TypeRelation::depuis_texte(l))
        .filter(|t| matches!(t, TypeRelation::Personnalise(_)));
    predefinis
        .chain(autres)
        .map(|t| TypeRelationFfi {
            code: t.jeton().to_owned(),
            libelle: t.libelle().to_owned(),
            inverse: inverses.inverse(&t).map(|i| i.libelle().to_owned()),
            predefini: !matches!(t, TypeRelation::Personnalise(_)),
        })
        .collect()
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
    let champ_divergent = match decision.nature {
        Nature::Divergence { champ } => Some(champ),
        _ => None,
    };
    let relations = champ_divergent == Some(Champ::Relation);
    let lisible = |texte: &str| match champ_divergent {
        Some(Champ::Relation) => relation_lisible(texte, referentiel),
        Some(Champ::Dates) => date_lisible(texte),
        _ => texte.to_owned(),
    };
    let recu = |textes: &[String], origine: &str| -> Vec<ValeurFfi> {
        textes
            .iter()
            .map(|t| ValeurFfi {
                texte: lisible(t),
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
            .map(|v| ValeurFfi {
                texte: lisible(&v.texte),
                ..valeur(v, sources)
            })
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
        // Une relation se choisit, elle ne se saisit pas : sa valeur désigne
        // une fiche par son identifiant.
        choix: decision
            .choix()
            .into_iter()
            .filter(|c| !(relations && *c == Choix::Saisir))
            .map(choix)
            .collect(),
        referentiel: referentiel_valeurs,
        source: recu(&decision.versions.source, &source_nom),
        base: decision.versions.base.iter().map(|t| lisible(t)).collect(),
    }
}

//! Synchronisation d'une source CardDAV : liste, fusion, écritures.

use std::collections::BTreeMap;

use unicontacts_core::referentiel::Referentiel;
use unicontacts_core::synchronisation::{CopieDistante, Ecriture, ErreurSynchronisation, Rapport};
use unicontacts_core::vcard::lire;
use unicontacts_core::{Horodatage, SourceId};

use crate::{Carnet, ErreurCardDav, Precondition};

/// Écriture que le carnet n'a pas reçue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refus {
    /// Identifiant de la copie (UID).
    pub uid: String,
    /// Motif du refus.
    pub erreur: ErreurCardDav,
}

/// Bilan d'une synchronisation CardDAV.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bilan {
    /// Compte rendu du moteur : écritures prévues, décisions, copies
    /// illisibles.
    pub rapport: Rapport,
    /// Nombre de cartes écrites.
    pub ecrites: usize,
    /// Nombre de cartes supprimées.
    pub supprimees: usize,
    /// Écritures et suppressions refusées : annulées dans le référentiel,
    /// elles repartiront au tour suivant.
    pub refus: Vec<Refus>,
}

/// Erreur d'une synchronisation CardDAV, qui l'a empêchée de commencer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurConnecteur {
    /// Le carnet n'a pu être lu : le référentiel est inchangé.
    #[error(transparent)]
    Carnet(#[from] ErreurCardDav),
    /// Le moteur a refusé la synchronisation.
    #[error(transparent)]
    Moteur(#[from] ErreurSynchronisation),
}

/// Cartes lues d'un carnet : les copies pour le moteur, et l'emplacement
/// (chemin, ETag) de chacune pour les écritures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lecture {
    /// Copies à confier au moteur.
    pub copies: Vec<CopieDistante>,
    emplacements: BTreeMap<String, (String, Option<String>)>,
}

/// Issue d'une écriture vers le carnet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Issue {
    /// Carte écrite, avec le nouvel ETag si le serveur l'a donné.
    Ecrite {
        /// Identifiant de la copie.
        uid: String,
        /// Nouvel ETag.
        etag: Option<String>,
    },
    /// Carte supprimée, ou déjà absente.
    Supprimee {
        /// Identifiant de la copie.
        uid: String,
    },
    /// Écriture ou suppression refusée.
    Refusee {
        /// Identifiant de la copie.
        uid: String,
        /// Vrai pour une suppression.
        suppression: bool,
        /// Motif.
        erreur: ErreurCardDav,
    },
}

/// Phase 1, réseau seul : lit toutes les cartes du carnet. Une carte se
/// désigne par son UID (RFC 6352, § 5.1) ; à défaut, ou si l'UID est en
/// double dans le carnet, par son chemin.
///
/// # Errors
///
/// [`ErreurCardDav`] si le carnet ne peut être lu.
// [impl->req~source-carddav~1]
pub fn lire_carnet(carnet: &impl Carnet) -> Result<Lecture, ErreurCardDav> {
    let ressources = carnet.lister()?;
    let mut emplacements: BTreeMap<String, (String, Option<String>)> = BTreeMap::new();
    let mut copies = Vec::with_capacity(ressources.len());
    for ressource in ressources {
        let uid = lire(&ressource.contenu)
            .ok()
            .and_then(|cartes| cartes.into_iter().next())
            .and_then(|carte| carte.uid())
            .filter(|uid| !uid.is_empty() && !emplacements.contains_key(uid))
            .unwrap_or_else(|| ressource.href.clone());
        emplacements.insert(uid.clone(), (ressource.href, ressource.etag.clone()));
        copies.push(CopieDistante {
            uid,
            etag: ressource.etag,
            contenu: ressource.contenu,
        });
    }
    Ok(Lecture {
        copies,
        emplacements,
    })
}

/// Phase 3, réseau seul : envoie au carnet les écritures du rapport — une
/// carte connue sous précondition `If-Match` (son ETag lu), une carte
/// nouvelle sous `If-None-Match: *`. Quand la liaison est rompue, les
/// écritures restantes ne sont pas tentées et sont rendues refusées.
// [impl->req~source-carddav~1]
pub fn envoyer(carnet: &impl Carnet, lecture: &Lecture, rapport: &Rapport) -> Vec<Issue> {
    let mut issues = Vec::with_capacity(rapport.ecritures.len());
    let mut rompue: Option<ErreurCardDav> = None;
    for ecriture in &rapport.ecritures {
        let (uid, suppression, resultat) = match ecriture {
            Ecriture::Carte { uid, contenu } => {
                let resultat = match &rompue {
                    Some(erreur) => Err(erreur.clone()),
                    None => {
                        let (href, precondition) = match lecture.emplacements.get(uid) {
                            Some((href, Some(etag))) => {
                                (href.clone(), Precondition::Remplacement(etag.clone()))
                            }
                            Some((href, None)) => (href.clone(), Precondition::Aucune),
                            None => (carnet.href_nouveau(uid), Precondition::Creation),
                        };
                        carnet.ecrire(&href, contenu, &precondition)
                    }
                };
                (uid, false, resultat)
            }
            Ecriture::Suppression { uid } => {
                // Déjà absente du carnet : la suppression est acquise.
                let resultat = match (lecture.emplacements.get(uid), &rompue) {
                    (None, _) => Ok(None),
                    (Some(_), Some(erreur)) => Err(erreur.clone()),
                    (Some((href, etag)), None) => {
                        carnet.supprimer(href, etag.as_deref()).map(|()| None)
                    }
                };
                (uid, true, resultat)
            }
        };
        issues.push(match resultat {
            Ok(_) if suppression => Issue::Supprimee { uid: uid.clone() },
            Ok(etag) => Issue::Ecrite {
                uid: uid.clone(),
                etag,
            },
            Err(erreur) => {
                if erreur.liaison_rompue() {
                    rompue = Some(erreur.clone());
                }
                Issue::Refusee {
                    uid: uid.clone(),
                    suppression,
                    erreur,
                }
            }
        });
    }
    issues
}

/// Phase 4, locale : reporte au référentiel l'issue des écritures — ETag
/// confirmés, écritures refusées annulées, suppressions refusées
/// reprogrammées.
// [impl->req~source-carddav~1]
pub fn reporter(
    referentiel: &mut Referentiel,
    source: SourceId,
    rapport: Rapport,
    issues: Vec<Issue>,
) -> Bilan {
    let mut bilan = Bilan {
        rapport,
        ecrites: 0,
        supprimees: 0,
        refus: Vec::new(),
    };
    for issue in issues {
        match issue {
            Issue::Ecrite { uid, etag } => {
                referentiel.confirmer_ecriture(source, &uid, etag);
                bilan.ecrites += 1;
            }
            Issue::Supprimee { .. } => bilan.supprimees += 1,
            Issue::Refusee {
                uid,
                suppression,
                erreur,
            } => {
                if suppression {
                    referentiel.reprogrammer_suppression(source, &uid);
                } else {
                    referentiel.annuler_ecriture(&bilan.rapport, &uid);
                }
                bilan.refus.push(Refus { uid, erreur });
            }
        }
    }
    bilan
}

/// Synchronise une source CardDAV avec le référentiel, en enchaînant les
/// quatre phases : lecture du carnet, fusion par le moteur, écritures,
/// report. Une application qui partage le référentiel entre plusieurs fils
/// enchaîne elle-même les phases, pour ne le verrouiller qu'aux phases 2
/// et 4, jamais pendant le réseau.
///
/// # Errors
///
/// [`ErreurConnecteur`] si le carnet ne peut être lu ou si la source n'est
/// pas configurée ; le référentiel est alors inchangé.
// [impl->req~source-carddav~1]
pub fn synchroniser(
    referentiel: &mut Referentiel,
    source: SourceId,
    carnet: &impl Carnet,
    date: Horodatage,
) -> Result<Bilan, ErreurConnecteur> {
    let lecture = lire_carnet(carnet)?;
    let rapport = referentiel.synchroniser(source, &lecture.copies, date)?;
    let issues = envoyer(carnet, &lecture, &rapport);
    Ok(reporter(referentiel, source, rapport, issues))
}

#[cfg(test)]
mod tests;

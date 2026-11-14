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

/// Synchronise une source CardDAV avec le référentiel.
///
/// Lit toutes les cartes du carnet, confie la fusion au moteur, puis écrit
/// vers le carnet : une carte connue sous précondition `If-Match` (son ETag
/// lu), une carte nouvelle sous `If-None-Match: *`. Une écriture refusée est
/// annulée dans le référentiel ; une suppression refusée est reprogrammée.
/// Quand la liaison est rompue, les écritures restantes ne sont pas tentées
/// et sont annulées de même.
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
    let ressources = carnet.lister()?;

    // Une carte se désigne par son UID (RFC 6352, § 5.1) ; à défaut, ou si
    // l'UID est en double dans le carnet, par son chemin.
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

    let rapport = referentiel.synchroniser(source, &copies, date)?;

    let mut bilan = Bilan {
        rapport: rapport.clone(),
        ecrites: 0,
        supprimees: 0,
        refus: Vec::new(),
    };
    let mut rompue: Option<ErreurCardDav> = None;
    for ecriture in &rapport.ecritures {
        match ecriture {
            Ecriture::Carte { uid, contenu } => {
                let resultat = match &rompue {
                    Some(erreur) => Err(erreur.clone()),
                    None => {
                        let (href, precondition) = match emplacements.get(uid) {
                            Some((href, Some(etag))) => {
                                (href.clone(), Precondition::Remplacement(etag.clone()))
                            }
                            Some((href, None)) => (href.clone(), Precondition::Aucune),
                            None => (carnet.href_nouveau(uid), Precondition::Creation),
                        };
                        carnet.ecrire(&href, contenu, &precondition)
                    }
                };
                match resultat {
                    Ok(etag) => {
                        referentiel.confirmer_ecriture(source, uid, etag);
                        bilan.ecrites += 1;
                    }
                    Err(erreur) => {
                        referentiel.annuler_ecriture(&rapport, uid);
                        if erreur.liaison_rompue() {
                            rompue = Some(erreur.clone());
                        }
                        bilan.refus.push(Refus {
                            uid: uid.clone(),
                            erreur,
                        });
                    }
                }
            }
            Ecriture::Suppression { uid } => {
                let Some((href, etag)) = emplacements.get(uid) else {
                    // Déjà absente du carnet : la suppression est acquise.
                    bilan.supprimees += 1;
                    continue;
                };
                let resultat = match &rompue {
                    Some(erreur) => Err(erreur.clone()),
                    None => carnet.supprimer(href, etag.as_deref()),
                };
                match resultat {
                    Ok(()) => bilan.supprimees += 1,
                    Err(erreur) => {
                        referentiel.reprogrammer_suppression(source, uid);
                        if erreur.liaison_rompue() {
                            rompue = Some(erreur.clone());
                        }
                        bilan.refus.push(Refus {
                            uid: uid.clone(),
                            erreur,
                        });
                    }
                }
            }
        }
    }
    Ok(bilan)
}

#[cfg(test)]
mod tests;

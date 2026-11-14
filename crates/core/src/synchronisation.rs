//! Synchronisation du référentiel avec une source.
//!
//! Le moteur reçoit la liste complète des copies que porte la source, la
//! confronte au référentiel et rend les écritures à faire vers la source.
//! Il ne fait aucun appel réseau : le connecteur de la source lit les
//! copies avant, et applique les écritures après (lot 2). Tout ce qui ne se
//! tranche pas seul devient une décision, et le reste se poursuit : une
//! décision en attente ne suspend ni les autres champs, ni les autres
//! fiches.

use std::collections::{BTreeMap, BTreeSet};

use crate::champ::{Champ, Valeur};
use crate::decisions::{Nature, Versions};
use crate::doublons::{self, Certitude};
use crate::fiche::{ErreurFiche, Fiche, Lien};
use crate::fusion::{Fusion, fusionner};
use crate::identifiants::{DecisionId, FicheId, SourceId};
use crate::normalisation::cle;
use crate::referentiel::Referentiel;
use crate::reglages::{Gagnant, Source, Suppressions};
use crate::temps::Horodatage;
use crate::vcard::{Carte, lire};

/// Copie d'une fiche telle que la source la sert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopieDistante {
    /// Identifiant de la copie dans la source (UID).
    pub uid: String,
    /// Étiquette de version de la copie, si la source en fournit (ETag).
    pub etag: Option<String>,
    /// Texte de la vCard.
    pub contenu: String,
}

/// Écriture à faire vers la source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ecriture {
    /// Créer ou remplacer la copie.
    Carte {
        /// Identifiant de la copie.
        uid: String,
        /// Texte de la vCard à écrire.
        contenu: String,
    },
    /// Supprimer la copie.
    Suppression {
        /// Identifiant de la copie.
        uid: String,
    },
}

/// Compte rendu d'une synchronisation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rapport {
    /// Écritures à faire vers la source, dans l'ordre.
    pub ecritures: Vec<Ecriture>,
    /// Décisions créées ou mises à jour.
    pub decisions: Vec<DecisionId>,
    /// Copies illisibles, laissées en l'état : identifiant et motif. Elles
    /// ne valent pas suppression.
    pub illisibles: Vec<(String, String)>,
}

/// Erreur de synchronisation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurSynchronisation {
    /// La source n'est pas configurée.
    #[error("source inconnue : {0}")]
    SourceInconnue(SourceId),
    /// Une fiche n'a pu recevoir les valeurs fusionnées.
    #[error(transparent)]
    Fiche(#[from] ErreurFiche),
}

/// Contexte d'une synchronisation : la source et ce que ses réglages
/// permettent.
struct Contexte {
    source: Source,
    lit: bool,
    ecrit: bool,
    gagnants: BTreeMap<Champ, Gagnant>,
    date: Horodatage,
}

impl Referentiel {
    /// Synchronise le référentiel avec une source, d'après la liste complète
    /// des copies qu'elle porte.
    ///
    /// Une copie liée à une fiche se fusionne champ par champ ; une copie
    /// inconnue s'importe, en passant par la détection des doublons ; une
    /// fiche dont la copie a disparu est traitée selon le réglage des
    /// suppressions de la source. Une copie illisible est laissée en l'état
    /// et signalée.
    ///
    /// # Errors
    ///
    /// [`ErreurSynchronisation::SourceInconnue`] si la source n'est pas
    /// configurée.
    // [impl->req~synchronisation-non-bloquante~1]
    pub fn synchroniser(
        &mut self,
        source: SourceId,
        copies: &[CopieDistante],
        date: Horodatage,
    ) -> Result<Rapport, ErreurSynchronisation> {
        let contexte = self.contexte(source, date)?;
        let mut rapport = Rapport::default();

        let programmees: Vec<String> = self
            .suppressions_en_attente(source)
            .map(str::to_owned)
            .collect();
        if contexte.ecrit {
            for uid in &programmees {
                self.a_supprimer.remove(&(source, uid.clone()));
                rapport
                    .ecritures
                    .push(Ecriture::Suppression { uid: uid.clone() });
            }
        }

        let liees: BTreeMap<String, FicheId> = self
            .fiches
            .values()
            .filter_map(|f| f.lien(source).map(|l| (l.uid.clone(), f.id())))
            .collect();
        let mut vues = BTreeSet::new();
        for copie in copies {
            vues.insert(copie.uid.clone());
            if programmees.contains(&copie.uid)
                || self.ignorees.contains(&(source, copie.uid.clone()))
            {
                continue;
            }
            let carte = match lire(&copie.contenu)
                .map(Vec::into_iter)
                .map(|mut c| c.next())
            {
                Ok(Some(carte)) => carte,
                Ok(None) => {
                    rapport
                        .illisibles
                        .push((copie.uid.clone(), "aucune vCard".to_owned()));
                    continue;
                }
                Err(erreur) => {
                    rapport
                        .illisibles
                        .push((copie.uid.clone(), erreur.to_string()));
                    continue;
                }
            };
            match liees.get(&copie.uid) {
                Some(fiche) => {
                    self.fusionner_copie(*fiche, copie, &carte, &contexte, &mut rapport)?
                }
                None if contexte.lit => self.importer(copie, &carte, &contexte, &mut rapport)?,
                None => {}
            }
        }

        for (uid, fiche) in liees {
            if !vues.contains(&uid) {
                self.traiter_disparition(fiche, &contexte, &mut rapport);
            }
        }
        Ok(rapport)
    }

    /// Enregistre l'étiquette de version qu'a rendue la source après une
    /// écriture.
    pub fn confirmer_ecriture(&mut self, source: SourceId, uid: &str, etag: Option<String>) {
        if let Some(lien) = self
            .fiches
            .values_mut()
            .filter_map(|f| f.lien_mut(source))
            .find(|l| l.uid == uid)
        {
            lien.etag = etag;
        }
    }

    fn contexte(
        &self,
        source: SourceId,
        date: Horodatage,
    ) -> Result<Contexte, ErreurSynchronisation> {
        let reglee = self
            .sources
            .source(source)
            .cloned()
            .ok_or(ErreurSynchronisation::SourceInconnue(source))?;
        Ok(Contexte {
            source: reglee,
            lit: self.sources.lit_depuis(source),
            ecrit: self.sources.ecrit_vers(source),
            gagnants: Champ::TOUS
                .into_iter()
                .map(|champ| (champ, self.sources.gagnant(source, champ)))
                .collect(),
            date,
        })
    }

    /// Fusionne la copie d'une fiche liée, champ par champ.
    // [impl->req~divergence-soumise-a-decision~1]
    // [impl->req~reglages-par-source~1]
    fn fusionner_copie(
        &mut self,
        id: FicheId,
        copie: &CopieDistante,
        carte: &Carte,
        contexte: &Contexte,
        rapport: &mut Rapport,
    ) -> Result<(), ErreurSynchronisation> {
        let source = contexte.source.id;
        let recues = carte.valeurs(source, contexte.date);
        let Some(fiche) = self.fiches.get_mut(&id) else {
            return Ok(());
        };
        let Some(lien) = fiche.lien(source).cloned() else {
            return Ok(());
        };
        let mut a_ecrire = lien.a_restaurer;
        let mut base = BTreeMap::new();
        let mut divergences = Vec::new();
        let mut accords = Vec::new();
        for champ in Champ::TOUS {
            let referentiel = fiche.valeurs(champ).to_vec();
            let recu = recues.get(&champ).map_or(&[][..], Vec::as_slice);
            let precedente = lien.base.get(&champ).map_or(&[][..], Vec::as_slice);
            let gagnant = contexte
                .gagnants
                .get(&champ)
                .copied()
                .unwrap_or(Gagnant::Personne);
            let fusion = if contexte.lit {
                fusionner(champ, &referentiel, recu, precedente, gagnant)
            } else {
                recouvrir(champ, &referentiel, recu)
            };
            let mut nouvelle_base = fusion.base;
            if fusion.ecrire_source {
                if contexte.ecrit {
                    a_ecrire = true;
                } else {
                    // Rien n'est écrit vers la source : la base retient ce
                    // qu'elle porte, pour ne pas confondre la valeur du
                    // référentiel avec une modification de la source.
                    nouvelle_base = textes(recu);
                }
            }
            if fusion.divergence {
                divergences.push((
                    champ,
                    Versions {
                        referentiel: textes(&referentiel),
                        source: textes(recu),
                        base: precedente.to_vec(),
                    },
                ));
            } else {
                accords.push(champ);
            }
            fiche.definir(champ, fusion.referentiel)?;
            if !nouvelle_base.is_empty() {
                base.insert(champ, nouvelle_base);
            }
        }
        let contenu = if a_ecrire && contexte.ecrit {
            let contenu =
                Carte::reecrire(Some(carte), contexte.source.version, &copie.uid, fiche).ecrire();
            rapport.ecritures.push(Ecriture::Carte {
                uid: copie.uid.clone(),
                contenu: contenu.clone(),
            });
            contenu
        } else {
            copie.contenu.clone()
        };
        if let Some(lien) = fiche.lien_mut(source) {
            lien.base = base;
            lien.carte = Some(contenu);
            lien.etag.clone_from(&copie.etag);
            lien.a_restaurer = false;
        }
        for (champ, versions) in divergences {
            let decision = self.soumettre(
                Nature::Divergence { champ },
                id,
                source,
                versions,
                contexte.date,
            );
            rapport.decisions.push(decision);
        }
        for champ in accords {
            self.clore_sans_objet(Nature::Divergence { champ }, id, source);
        }
        Ok(())
    }

    /// Importe une copie inconnue : nouvelle fiche, ou rattachement à son
    /// doublon certain ; un doublon probable donne une nouvelle fiche et une
    /// décision.
    // [impl->req~traitement-des-doublons~1]
    fn importer(
        &mut self,
        copie: &CopieDistante,
        carte: &Carte,
        contexte: &Contexte,
        rapport: &mut Rapport,
    ) -> Result<(), ErreurSynchronisation> {
        let source = contexte.source.id;
        let mut importee = Fiche::nouvelle();
        for (champ, valeurs) in carte.valeurs(source, contexte.date) {
            importee.definir(champ, valeurs)?;
        }
        let mut lien = Lien::new(source, copie.uid.clone());
        lien.etag.clone_from(&copie.etag);
        lien.carte = Some(copie.contenu.clone());
        let doublon = doublons::detecter(
            &importee,
            self.fiches.values().filter(|f| f.lien(source).is_none()),
        );
        match doublon {
            Some(doublon) if doublon.certitude == Certitude::Certain => {
                // Rattachée sans base : la fusion réunit ce que chacune porte,
                // et soumet ce qui diverge.
                if let Some(existante) = self.fiches.get_mut(&doublon.fiche) {
                    existante.lier(lien);
                }
                self.fusionner_copie(doublon.fiche, copie, carte, contexte, rapport)?;
            }
            Some(doublon) => {
                lien.base = valeurs_en_textes(&importee);
                importee.lier(lien);
                let versions = Versions {
                    referentiel: self
                        .fiche(doublon.fiche)
                        .map(|f| f.textes(Champ::NomAffiche))
                        .unwrap_or_default(),
                    source: importee.textes(Champ::NomAffiche),
                    base: Vec::new(),
                };
                let id = self.ajouter(importee);
                let decision = self.soumettre(
                    Nature::DoublonProbable {
                        autre: doublon.fiche,
                    },
                    id,
                    source,
                    versions,
                    contexte.date,
                );
                rapport.decisions.push(decision);
            }
            None => {
                lien.base = valeurs_en_textes(&importee);
                importee.lier(lien);
                self.ajouter(importee);
            }
        }
        Ok(())
    }

    /// Traite une fiche liée dont la copie a disparu de la source.
    // [impl->req~suppression-sans-destruction~1]
    // [impl->req~issues-suppression-detectee~1]
    fn traiter_disparition(&mut self, id: FicheId, contexte: &Contexte, rapport: &mut Rapport) {
        let source = contexte.source.id;
        let Some(fiche) = self.fiches.get_mut(&id) else {
            return;
        };
        let Some(lien) = fiche.lien(source).cloned() else {
            return;
        };
        // Restauration demandée, ou source en écriture seule : la copie est
        // réécrite, à partir de la dernière connue pour garder ses
        // propriétés non interprétées.
        if lien.a_restaurer || !contexte.lit {
            if contexte.ecrit {
                let precedente = lien
                    .carte
                    .as_deref()
                    .and_then(|texte| lire(texte).ok())
                    .and_then(|cartes| cartes.into_iter().next());
                let contenu = Carte::reecrire(
                    precedente.as_ref(),
                    contexte.source.version,
                    &lien.uid,
                    fiche,
                )
                .ecrire();
                rapport.ecritures.push(Ecriture::Carte {
                    uid: lien.uid.clone(),
                    contenu: contenu.clone(),
                });
                let base = valeurs_en_textes(fiche);
                if let Some(lien) = fiche.lien_mut(source) {
                    lien.carte = Some(contenu);
                    lien.base = base;
                    lien.etag = None;
                    lien.a_restaurer = false;
                }
            }
            return;
        }
        match contexte.source.reglages.suppressions {
            Suppressions::Demander => {
                let versions = Versions {
                    referentiel: fiche.textes(Champ::NomAffiche),
                    source: Vec::new(),
                    base: lien
                        .base
                        .get(&Champ::NomAffiche)
                        .cloned()
                        .unwrap_or_default(),
                };
                let decision =
                    self.soumettre(Nature::Suppression, id, source, versions, contexte.date);
                rapport.decisions.push(decision);
            }
            Suppressions::Propager => self.propager_suppression(id, source),
            Suppressions::Ignorer => {
                fiche.detacher(source);
            }
        }
    }
}

/// Source en écriture seule : le référentiel recouvre ce qu'elle porte.
fn recouvrir(champ: Champ, referentiel: &[Valeur], source: &[Valeur]) -> Fusion {
    let cles = |valeurs: &[Valeur]| -> BTreeSet<String> {
        valeurs.iter().map(|v| cle(champ, &v.texte)).collect()
    };
    Fusion {
        referentiel: referentiel.to_vec(),
        ecrire_source: cles(referentiel) != cles(source),
        base: textes(referentiel),
        divergence: false,
    }
}

fn textes(valeurs: &[Valeur]) -> Vec<String> {
    valeurs.iter().map(|v| v.texte.clone()).collect()
}

/// Textes de tous les champs renseignés d'une fiche : la base d'une copie
/// qui porte exactement ces valeurs.
fn valeurs_en_textes(fiche: &Fiche) -> BTreeMap<Champ, Vec<String>> {
    fiche
        .champs()
        .map(|(champ, valeurs)| (champ, textes(valeurs)))
        .collect()
}

#[cfg(test)]
mod tests;

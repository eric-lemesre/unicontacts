//! Le référentiel : fiches maîtres, sources, file de décisions.
//!
//! C'est l'agrégat que la synchronisation modifie et que le stockage
//! enregistre. Toute opération y est synchrone et locale : aucune n'attend
//! le réseau.

use std::collections::{BTreeMap, BTreeSet};

use crate::champ::{Champ, Provenance, Valeur};
use crate::decisions::{
    Action, Decision, IssueDoublon, IssueSuppression, Nature, Resolution, Statut, Versions,
};
use crate::fiche::{ErreurFiche, Fiche};
use crate::identifiants::{DecisionId, FicheId, SourceId};
use crate::normalisation::cle;
use crate::reglages::Sources;
use crate::temps::Horodatage;

/// Erreur de traitement d'une décision.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurDecision {
    /// Aucune décision de cet identifiant.
    #[error("décision inconnue : {0}")]
    Inconnue(DecisionId),
    /// La décision est déjà résolue.
    #[error("décision déjà résolue : {0}")]
    DejaResolue(DecisionId),
    /// La résolution ne convient pas à la nature de la décision.
    #[error("résolution inadaptée à la décision {0}")]
    ResolutionInadmise(DecisionId),
    /// La fiche concernée n'existe plus.
    #[error("fiche disparue : {0}")]
    FicheDisparue(FicheId),
    /// La résolution n'a pu s'appliquer à la fiche.
    #[error(transparent)]
    Fiche(#[from] ErreurFiche),
}

/// Erreur de suppression d'une fiche dans l'application.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurSuppression {
    /// Aucune fiche de cet identifiant.
    #[error("fiche inconnue : {0}")]
    FicheInconnue(FicheId),
    /// La source n'est pas liée à la fiche, ou ses réglages n'y autorisent
    /// pas l'écriture.
    #[error("la suppression ne peut s'appliquer à la source {0}")]
    SourceNonAutorisee(SourceId),
}

/// Le référentiel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Referentiel {
    pub(crate) fiches: BTreeMap<FicheId, Fiche>,
    pub(crate) sources: Sources,
    pub(crate) decisions: Vec<Decision>,
    /// Copies à supprimer des sources à leur prochaine synchronisation.
    pub(crate) a_supprimer: BTreeSet<(SourceId, String)>,
    /// Copies que la synchronisation ne doit plus importer : leur fiche a
    /// été supprimée du référentiel sans l'être de leur source.
    pub(crate) ignorees: BTreeSet<(SourceId, String)>,
}

impl Referentiel {
    /// Référentiel vide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sources configurées.
    #[must_use]
    pub const fn sources(&self) -> &Sources {
        &self.sources
    }

    /// Sources configurées, modifiables.
    pub const fn sources_mut(&mut self) -> &mut Sources {
        &mut self.sources
    }

    /// Fiche maître, si elle existe.
    #[must_use]
    pub fn fiche(&self, id: FicheId) -> Option<&Fiche> {
        self.fiches.get(&id)
    }

    /// Fiche maître modifiable, si elle existe.
    pub fn fiche_mut(&mut self, id: FicheId) -> Option<&mut Fiche> {
        self.fiches.get_mut(&id)
    }

    /// Toutes les fiches maîtres.
    pub fn fiches(&self) -> impl Iterator<Item = &Fiche> {
        self.fiches.values()
    }

    /// Ajoute une fiche et rend son identifiant.
    pub fn ajouter(&mut self, fiche: Fiche) -> FicheId {
        let id = fiche.id();
        self.fiches.insert(id, fiche);
        id
    }

    /// Toutes les décisions, dans leur ordre de création.
    #[must_use]
    pub fn decisions(&self) -> &[Decision] {
        &self.decisions
    }

    /// Décision, si elle existe.
    #[must_use]
    pub fn decision(&self, id: DecisionId) -> Option<&Decision> {
        self.decisions.iter().find(|d| d.id == id)
    }

    /// Décisions qui attendent l'utilisateur : en attente ou reportées.
    pub fn ouvertes(&self) -> impl Iterator<Item = &Decision> {
        self.decisions.iter().filter(|d| d.est_ouverte())
    }

    /// Copies à supprimer d'une source à sa prochaine synchronisation.
    pub fn suppressions_en_attente(&self, source: SourceId) -> impl Iterator<Item = &str> {
        self.a_supprimer
            .iter()
            .filter(move |(s, _)| *s == source)
            .map(|(_, uid)| uid.as_str())
    }

    /// Soumet une décision : met à jour les versions de la décision ouverte
    /// de même nature sur la même fiche et la même source, ou en crée une.
    pub(crate) fn soumettre(
        &mut self,
        nature: Nature,
        fiche: FicheId,
        source: SourceId,
        versions: Versions,
        date: Horodatage,
    ) -> DecisionId {
        if let Some(ouverte) = self.decisions.iter_mut().find(|d| {
            d.est_ouverte() && d.nature == nature && d.fiche == fiche && d.source == source
        }) {
            ouverte.versions = versions;
            return ouverte.id;
        }
        let decision = Decision::new(nature, fiche, source, versions, date);
        let id = decision.id;
        self.decisions.push(decision);
        id
    }

    /// Clôt, sans résolution, la décision ouverte de cette nature sur la
    /// fiche et la source : elle est devenue sans objet.
    pub(crate) fn clore_sans_objet(&mut self, nature: Nature, fiche: FicheId, source: SourceId) {
        for decision in self.decisions.iter_mut().filter(|d| {
            d.est_ouverte() && d.nature == nature && d.fiche == fiche && d.source == source
        }) {
            decision.statut = Statut::Resolue;
        }
    }

    /// Clôt toutes les décisions ouvertes d'une fiche qui disparaît.
    fn clore_decisions_de(&mut self, fiche: FicheId) {
        for decision in self
            .decisions
            .iter_mut()
            .filter(|d| d.est_ouverte() && d.fiche == fiche)
        {
            decision.statut = Statut::Resolue;
        }
    }

    fn decision_ouverte(&self, id: DecisionId) -> Result<&Decision, ErreurDecision> {
        let decision = self.decision(id).ok_or(ErreurDecision::Inconnue(id))?;
        if decision.est_ouverte() {
            Ok(decision)
        } else {
            Err(ErreurDecision::DejaResolue(id))
        }
    }

    /// Reporte une décision : elle reste dans la file, reportée, et rien
    /// des valeurs en présence ne change.
    ///
    /// # Errors
    ///
    /// [`ErreurDecision::Inconnue`] ou [`ErreurDecision::DejaResolue`].
    // [impl->req~report-decision~1]
    pub fn reporter(&mut self, id: DecisionId) -> Result<(), ErreurDecision> {
        self.decision_ouverte(id)?;
        if let Some(decision) = self.decisions.iter_mut().find(|d| d.id == id) {
            decision.statut = Statut::Reportee;
        }
        Ok(())
    }

    /// Décisions en attente similaires à celle-ci (même nature, même champ,
    /// même source), elle exclue : leur nombre s'annonce avant de les
    /// résoudre en lot.
    ///
    /// # Errors
    ///
    /// [`ErreurDecision::Inconnue`] ou [`ErreurDecision::DejaResolue`].
    // [impl->req~resolution-par-lot~1]
    pub fn cas_similaires(&self, id: DecisionId) -> Result<Vec<DecisionId>, ErreurDecision> {
        let modele = self.decision_ouverte(id)?;
        Ok(self
            .decisions
            .iter()
            .filter(|d| d.id != id && d.statut == Statut::EnAttente && modele.est_similaire_a(d))
            .map(|d| d.id)
            .collect())
    }

    /// Résout une décision et applique la résolution.
    ///
    /// # Errors
    ///
    /// [`ErreurDecision`] si la décision est inconnue, déjà résolue, si la
    /// résolution ne lui convient pas, ou si sa fiche a disparu. Rien n'est
    /// alors modifié.
    pub fn resoudre(
        &mut self,
        id: DecisionId,
        resolution: Resolution,
        date: Horodatage,
    ) -> Result<(), ErreurDecision> {
        let decision = self.decision_ouverte(id)?.clone();
        if !decision.admet(&resolution) {
            return Err(ErreurDecision::ResolutionInadmise(id));
        }
        match &resolution {
            Resolution::Divergence(action) => self.appliquer_divergence(&decision, action, date)?,
            Resolution::Suppression(issue) => self.appliquer_suppression(&decision, *issue)?,
            Resolution::Doublon(issue) => self.appliquer_doublon(&decision, *issue)?,
        }
        if let Some(resolue) = self.decisions.iter_mut().find(|d| d.id == id) {
            resolue.statut = Statut::Resolue;
            resolue.resolution = Some(resolution);
        }
        Ok(())
    }

    /// Résout une décision et, de la même façon, tous ses cas similaires en
    /// attente ; rend le nombre de décisions résolues.
    ///
    /// # Errors
    ///
    /// [`ErreurDecision`] à la première décision qui ne peut se résoudre ;
    /// celles déjà résolues le restent.
    // [impl->req~resolution-par-lot~1]
    pub fn resoudre_avec_cas_similaires(
        &mut self,
        id: DecisionId,
        resolution: &Resolution,
        date: Horodatage,
    ) -> Result<usize, ErreurDecision> {
        let similaires = self.cas_similaires(id)?;
        self.resoudre(id, resolution.clone(), date)?;
        for autre in &similaires {
            self.resoudre(*autre, resolution.clone(), date)?;
        }
        Ok(similaires.len() + 1)
    }

    /// Applique l'action choisie pour une divergence. La base du lien prend
    /// la valeur de la source telle que la décision l'a vue : la prochaine
    /// synchronisation y écrira la valeur retenue si elle en diffère.
    // [impl->req~actions-de-resolution~1]
    fn appliquer_divergence(
        &mut self,
        decision: &Decision,
        action: &Action,
        date: Horodatage,
    ) -> Result<(), ErreurDecision> {
        let Nature::Divergence { champ } = decision.nature else {
            return Err(ErreurDecision::ResolutionInadmise(decision.id));
        };
        let fiche = self
            .fiches
            .get_mut(&decision.fiche)
            .ok_or(ErreurDecision::FicheDisparue(decision.fiche))?;
        let de_la_source = |textes: &[String]| -> Vec<Valeur> {
            textes
                .iter()
                .map(|t| Valeur::new(t.clone(), Provenance::source(decision.source, date)))
                .collect()
        };
        match action {
            Action::GarderReferentiel => {}
            Action::GarderSource => {
                fiche.definir(champ, de_la_source(&decision.versions.source))?
            }
            Action::GarderLesDeux => {
                let mut valeurs = fiche.valeurs(champ).to_vec();
                for valeur in de_la_source(&decision.versions.source) {
                    let k = cle(champ, &valeur.texte);
                    if !valeurs.iter().any(|v| cle(champ, &v.texte) == k) {
                        valeurs.push(valeur);
                    }
                }
                fiche.definir(champ, valeurs)?;
            }
            Action::Saisir(textes) => {
                let textes: Vec<&str> = textes.iter().map(String::as_str).collect();
                fiche.modifier(champ, &textes, date)?;
            }
        }
        if let Some(lien) = fiche.lien_mut(decision.source) {
            lien.base.insert(champ, decision.versions.source.clone());
        }
        Ok(())
    }

    /// Applique l'issue choisie pour une suppression détectée.
    // [impl->req~issues-suppression-detectee~1]
    fn appliquer_suppression(
        &mut self,
        decision: &Decision,
        issue: IssueSuppression,
    ) -> Result<(), ErreurDecision> {
        let fiche = self
            .fiches
            .get_mut(&decision.fiche)
            .ok_or(ErreurDecision::FicheDisparue(decision.fiche))?;
        match issue {
            IssueSuppression::Detacher => {
                fiche.detacher(decision.source);
            }
            IssueSuppression::Restaurer => {
                if let Some(lien) = fiche.lien_mut(decision.source) {
                    lien.a_restaurer = true;
                }
            }
            IssueSuppression::Propager => {
                self.propager_suppression(decision.fiche, decision.source)
            }
        }
        Ok(())
    }

    /// Supprime une fiche disparue d'une source : du référentiel, et des
    /// autres sources où elle est liée et que leurs réglages autorisent en
    /// écriture, à leur prochaine synchronisation.
    pub(crate) fn propager_suppression(&mut self, id: FicheId, depuis: SourceId) {
        let Some(fiche) = self.fiches.remove(&id) else {
            return;
        };
        for lien in fiche.liens() {
            if lien.source != depuis && self.sources.ecrit_vers(lien.source) {
                self.a_supprimer.insert((lien.source, lien.uid.clone()));
            }
        }
        self.clore_decisions_de(id);
    }

    /// Applique l'issue choisie pour un doublon probable. Pour une fusion,
    /// la fiche existante reçoit les valeurs qui lui manquent et les liens
    /// de la fiche importée, qui disparaît.
    fn appliquer_doublon(
        &mut self,
        decision: &Decision,
        issue: IssueDoublon,
    ) -> Result<(), ErreurDecision> {
        let Nature::DoublonProbable { autre } = decision.nature else {
            return Err(ErreurDecision::ResolutionInadmise(decision.id));
        };
        if issue == IssueDoublon::Distinguer {
            return Ok(());
        }
        let importee = self
            .fiches
            .remove(&decision.fiche)
            .ok_or(ErreurDecision::FicheDisparue(decision.fiche))?;
        let Some(existante) = self.fiches.get_mut(&autre) else {
            self.fiches.insert(importee.id(), importee);
            return Err(ErreurDecision::FicheDisparue(autre));
        };
        reunir(existante, &importee)?;
        for lien in importee.liens() {
            if existante.lien(lien.source).is_some() {
                // La source a déjà sa copie de la fiche existante : l'autre
                // copie n'est plus importée, sans être supprimée.
                self.ignorees.insert((lien.source, lien.uid.clone()));
            } else {
                existante.lier(lien.clone());
            }
        }
        for decision in &mut self.decisions {
            if decision.fiche == importee.id() {
                decision.fiche = autre;
            }
        }
        Ok(())
    }

    /// Sources où une suppression demandée dans l'application peut
    /// s'appliquer : celles liées à la fiche que leurs réglages autorisent
    /// en écriture.
    ///
    /// # Errors
    ///
    /// [`ErreurSuppression::FicheInconnue`].
    // [impl->req~portee-suppression-locale~1]
    pub fn sources_de_suppression(
        &self,
        fiche: FicheId,
    ) -> Result<Vec<SourceId>, ErreurSuppression> {
        let fiche = self
            .fiches
            .get(&fiche)
            .ok_or(ErreurSuppression::FicheInconnue(fiche))?;
        Ok(fiche
            .liens()
            .map(|l| l.source)
            .filter(|s| self.sources.ecrit_vers(*s))
            .collect())
    }

    /// Supprime une fiche dans l'application, et de chacune des sources
    /// choisies à leur prochaine synchronisation. Les copies des sources non
    /// choisies y restent, sans plus être importées.
    ///
    /// # Errors
    ///
    /// [`ErreurSuppression`] si la fiche est inconnue ou si une source
    /// choisie n'est pas parmi [`Referentiel::sources_de_suppression`] ;
    /// rien n'est alors supprimé.
    // [impl->req~portee-suppression-locale~1]
    pub fn supprimer_fiche(
        &mut self,
        id: FicheId,
        choisies: &[SourceId],
    ) -> Result<(), ErreurSuppression> {
        let possibles = self.sources_de_suppression(id)?;
        if let Some(refusee) = choisies.iter().find(|s| !possibles.contains(s)) {
            return Err(ErreurSuppression::SourceNonAutorisee(*refusee));
        }
        let Some(fiche) = self.fiches.remove(&id) else {
            return Err(ErreurSuppression::FicheInconnue(id));
        };
        for lien in fiche.liens() {
            let copie = (lien.source, lien.uid.clone());
            if choisies.contains(&lien.source) {
                self.a_supprimer.insert(copie);
            } else {
                self.ignorees.insert(copie);
            }
        }
        self.clore_decisions_de(id);
        Ok(())
    }
}

/// Ajoute à la fiche existante les valeurs de l'importée qui lui manquent :
/// les valeurs nouvelles d'un champ multivalué, et un champ à valeur unique
/// qu'elle n'a pas.
fn reunir(existante: &mut Fiche, importee: &Fiche) -> Result<(), ErreurFiche> {
    for champ in Champ::TOUS {
        let mut valeurs = existante.valeurs(champ).to_vec();
        for valeur in importee.valeurs(champ) {
            let k = cle(champ, &valeur.texte);
            let present = valeurs.iter().any(|v| cle(champ, &v.texte) == k);
            if !present && (champ.est_multivalue() || valeurs.is_empty()) {
                valeurs.push(valeur.clone());
            }
        }
        existante.definir(champ, valeurs)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;

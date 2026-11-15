//! Groupes de fiches (ADR-0012).
//!
//! Un groupe est désigné par son chemin complet, ses noms successifs du
//! plus général au plus précis séparés par `/` (`Travail/Clients/Client X`).
//! Une fiche appartient à ses groupes par les valeurs de son champ
//! [`Champ::Groupes`], une par groupe : c'est ce que porte `CATEGORIES` dans
//! une vCard. Le parent d'un groupe est le chemin sans son dernier nom : la
//! hiérarchie ne peut avoir de cycle.
// [impl->req~appartenance-multiple~1]
// [impl->req~groupes-hierarchiques~1]

use std::collections::BTreeSet;

use crate::champ::{Champ, Provenance, Valeur};
use crate::identifiants::FicheId;
use crate::referentiel::Referentiel;
use crate::temps::Horodatage;

/// Séparateur des noms d'un chemin de groupe.
pub const SEPARATEUR: char = '/';

/// Erreur d'une opération sur les groupes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurGroupe {
    /// Le chemin ne nomme aucun groupe.
    #[error("chemin de groupe vide")]
    CheminVide,
    /// Aucun groupe de ce chemin.
    #[error("groupe inconnu : {0}")]
    Inconnu(String),
    /// Un groupe de ce chemin existe déjà.
    #[error("le groupe {0} existe déjà")]
    Existant(String),
    /// Le groupe deviendrait son propre ancêtre.
    #[error("le groupe {groupe} ne peut devenir enfant de {parent}, qui en descend")]
    Cycle {
        /// Groupe déplacé.
        groupe: String,
        /// Parent demandé.
        parent: String,
    },
    /// Aucune fiche de cet identifiant.
    #[error("fiche inconnue : {0}")]
    FicheInconnue(FicheId),
}

/// Chemin normalisé : chaque nom sans espaces de bord ni espaces répétées,
/// noms vides retirés. Vide si le texte ne nomme aucun groupe.
#[must_use]
pub fn chemin(texte: &str) -> String {
    texte
        .split(SEPARATEUR)
        .map(|nom| nom.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|nom| !nom.is_empty())
        .collect::<Vec<_>>()
        .join(&SEPARATEUR.to_string())
}

/// Chemin du parent d'un groupe ; aucun pour un groupe de premier niveau.
#[must_use]
pub fn parent(chemin: &str) -> Option<&str> {
    chemin.rsplit_once(SEPARATEUR).map(|(parent, _)| parent)
}

/// Dernier nom d'un chemin : le nom propre du groupe.
#[must_use]
pub fn nom(chemin: &str) -> &str {
    chemin
        .rsplit_once(SEPARATEUR)
        .map_or(chemin, |(_, nom)| nom)
}

/// Vrai si `chemin` est `groupe` ou l'un de ses sous-groupes.
#[must_use]
pub fn est_dans(chemin: &str, groupe: &str) -> bool {
    chemin == groupe
        || chemin
            .strip_prefix(groupe)
            .is_some_and(|reste| reste.starts_with(SEPARATEUR))
}

/// Chemins d'un groupe et de tous ses ancêtres.
fn avec_ancetres(chemin: &str) -> impl Iterator<Item = String> + '_ {
    chemin
        .match_indices(SEPARATEUR)
        .map(|(i, _)| chemin[..i].to_owned())
        .chain(std::iter::once(chemin.to_owned()))
}

/// Combinaison de plusieurs groupes dans un filtre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Combinaison {
    /// Fiches de l'un des groupes : par défaut.
    #[default]
    Union,
    /// Fiches de tous les groupes à la fois.
    Intersection,
}

/// Filtre de la liste des fiches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filtre {
    /// Groupes retenus, par leur chemin ; aucun : pas de filtre de groupe.
    pub groupes: Vec<String>,
    /// Union ou intersection des groupes.
    pub combinaison: Combinaison,
    /// Vrai si un sous-groupe compte pour son groupe : par défaut.
    pub sous_groupes: bool,
    /// Texte cherché dans le nom affiché ; vide : pas de recherche.
    pub recherche: String,
}

impl Default for Filtre {
    fn default() -> Self {
        Self {
            groupes: Vec::new(),
            combinaison: Combinaison::Union,
            sous_groupes: true,
            recherche: String::new(),
        }
    }
}

impl Filtre {
    /// Vrai si la fiche passe le filtre.
    // [impl->req~filtre-par-groupes~2]
    #[must_use]
    pub fn retient(&self, groupes_de_la_fiche: &[String], nom_affiche: &str) -> bool {
        let dans = |groupe: &String| {
            groupes_de_la_fiche.iter().any(|g| {
                if self.sous_groupes {
                    est_dans(g, groupe)
                } else {
                    g == groupe
                }
            })
        };
        let groupes = self.groupes.is_empty()
            || match self.combinaison {
                Combinaison::Union => self.groupes.iter().any(dans),
                Combinaison::Intersection => self.groupes.iter().all(dans),
            };
        let recherche = self.recherche.trim();
        groupes && (recherche.is_empty() || plie(nom_affiche).contains(&plie(recherche)))
    }
}

/// Texte en minuscules, sans accents : pour une recherche tolérante.
fn plie(texte: &str) -> String {
    texte
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' | 'ã' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' | 'ì' => 'i',
            'ô' | 'ö' | 'ó' | 'ò' | 'õ' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            'ÿ' => 'y',
            autre => autre,
        })
        .collect()
}

impl Referentiel {
    /// Tous les groupes, par ordre de chemin : ceux créés dans
    /// l'application, ceux des fiches, et leurs ancêtres.
    #[must_use]
    pub fn groupes(&self) -> Vec<String> {
        let mut groupes: BTreeSet<String> = BTreeSet::new();
        let des_fiches = self
            .fiches
            .values()
            .flat_map(|f| f.valeurs(Champ::Groupes))
            .map(|v| v.texte.as_str());
        for chemin in self.groupes.iter().map(String::as_str).chain(des_fiches) {
            groupes.extend(avec_ancetres(chemin));
        }
        groupes.into_iter().collect()
    }

    /// Crée un groupe vide, sous un parent éventuel (créé avec lui).
    ///
    /// # Errors
    ///
    /// [`ErreurGroupe::CheminVide`] ; [`ErreurGroupe::Existant`].
    // [impl->req~groupes-hierarchiques~1]
    pub fn creer_groupe(&mut self, texte: &str) -> Result<String, ErreurGroupe> {
        let chemin = chemin(texte);
        if chemin.is_empty() {
            return Err(ErreurGroupe::CheminVide);
        }
        if self.groupes().contains(&chemin) {
            return Err(ErreurGroupe::Existant(chemin));
        }
        self.groupes.insert(chemin.clone());
        Ok(chemin)
    }

    /// Groupes d'une fiche, par leur chemin.
    #[must_use]
    pub fn groupes_de(&self, fiche: FicheId) -> Vec<String> {
        self.fiches
            .get(&fiche)
            .map(|f| f.textes(Champ::Groupes))
            .unwrap_or_default()
    }

    /// Range une fiche dans les groupes donnés, à la place des précédents ;
    /// un groupe déjà porté garde sa provenance.
    ///
    /// # Errors
    ///
    /// [`ErreurGroupe::FicheInconnue`] ; la fiche est alors inchangée.
    // [impl->req~appartenance-multiple~1]
    pub fn ranger(
        &mut self,
        fiche: FicheId,
        groupes: &[&str],
        date: Horodatage,
    ) -> Result<(), ErreurGroupe> {
        let porteuse = self
            .fiches
            .get_mut(&fiche)
            .ok_or(ErreurGroupe::FicheInconnue(fiche))?;
        let mut chemins: Vec<String> = Vec::new();
        for texte in groupes {
            let c = chemin(texte);
            if !c.is_empty() && !chemins.contains(&c) {
                chemins.push(c);
            }
        }
        let anciennes = porteuse.valeurs(Champ::Groupes).to_vec();
        let valeurs = chemins
            .iter()
            .map(|c| {
                anciennes
                    .iter()
                    .find(|v| v.texte == *c)
                    .cloned()
                    .unwrap_or_else(|| Valeur::new(c.clone(), Provenance::referentiel(date)))
            })
            .collect();
        porteuse.definir_multivalue(Champ::Groupes, valeurs);
        Ok(())
    }

    /// Renomme ou déplace un groupe, sous-groupes compris, en une seule
    /// opération : le registre et toutes les fiches concernées passent au
    /// nouveau chemin, sans décision ; la synchronisation suivante l'écrit
    /// vers les sources comme une modification de l'application.
    ///
    /// # Errors
    ///
    /// [`ErreurGroupe::CheminVide`], [`ErreurGroupe::Inconnu`],
    /// [`ErreurGroupe::Existant`], ou [`ErreurGroupe::Cycle`] si le nouveau
    /// chemin descend de l'ancien ; rien n'est alors modifié.
    // [impl->req~renommage-groupe-operation-unique~1]
    // [impl->req~groupes-hierarchiques~1]
    pub fn renommer_groupe(
        &mut self,
        ancien: &str,
        nouveau: &str,
        date: Horodatage,
    ) -> Result<String, ErreurGroupe> {
        let (ancien, nouveau) = (chemin(ancien), chemin(nouveau));
        if ancien.is_empty() || nouveau.is_empty() {
            return Err(ErreurGroupe::CheminVide);
        }
        let existants = self.groupes();
        if !existants.contains(&ancien) {
            return Err(ErreurGroupe::Inconnu(ancien));
        }
        if nouveau == ancien {
            return Ok(nouveau);
        }
        if est_dans(&nouveau, &ancien) {
            return Err(ErreurGroupe::Cycle {
                groupe: ancien,
                parent: parent(&nouveau).unwrap_or_default().to_owned(),
            });
        }
        if existants.contains(&nouveau) {
            return Err(ErreurGroupe::Existant(nouveau));
        }
        let renomme = |c: &str| -> Option<String> {
            est_dans(c, &ancien).then(|| format!("{nouveau}{}", &c[ancien.len()..]))
        };
        self.groupes = std::mem::take(&mut self.groupes)
            .into_iter()
            .map(|c| renomme(&c).unwrap_or(c))
            .collect();
        for fiche in self.fiches.values_mut() {
            let valeurs = fiche.valeurs(Champ::Groupes);
            if !valeurs.iter().any(|v| renomme(&v.texte).is_some()) {
                continue;
            }
            let valeurs = valeurs
                .iter()
                .map(|v| match renomme(&v.texte) {
                    Some(texte) => Valeur::new(texte, Provenance::referentiel(date)),
                    None => v.clone(),
                })
                .collect();
            fiche.definir_multivalue(Champ::Groupes, valeurs);
        }
        Ok(nouveau)
    }

    /// Donne à un groupe un autre parent, ou aucun : le groupe garde son
    /// nom et emmène ses sous-groupes.
    ///
    /// # Errors
    ///
    /// Comme [`Referentiel::renommer_groupe`] ; [`ErreurGroupe::Cycle`] si le
    /// parent demandé est le groupe ou l'un de ses descendants.
    // [impl->req~groupes-hierarchiques~1]
    pub fn deplacer_groupe(
        &mut self,
        groupe: &str,
        parent: Option<&str>,
        date: Horodatage,
    ) -> Result<String, ErreurGroupe> {
        let groupe = chemin(groupe);
        let nouveau = match parent.map(chemin).filter(|p| !p.is_empty()) {
            Some(parent) => {
                if est_dans(&parent, &groupe) {
                    return Err(ErreurGroupe::Cycle { groupe, parent });
                }
                format!("{parent}{SEPARATEUR}{}", nom(&groupe))
            }
            None => nom(&groupe).to_owned(),
        };
        self.renommer_groupe(&groupe, &nouveau, date)
    }

    /// Supprime un groupe et ses sous-groupes : du registre, et des fiches
    /// qui y étaient rangées. Les fiches demeurent.
    pub fn supprimer_groupe(&mut self, groupe: &str) {
        let groupe = chemin(groupe);
        self.groupes.retain(|c| !est_dans(c, &groupe));
        for fiche in self.fiches.values_mut() {
            let valeurs: Vec<Valeur> = fiche
                .valeurs(Champ::Groupes)
                .iter()
                .filter(|v| !est_dans(&v.texte, &groupe))
                .cloned()
                .collect();
            fiche.definir_multivalue(Champ::Groupes, valeurs);
        }
    }

    /// Fiches qui passent le filtre.
    // [impl->req~filtre-par-groupes~2]
    #[must_use]
    pub fn filtrer(&self, filtre: &Filtre) -> Vec<FicheId> {
        self.fiches
            .values()
            .filter(|f| {
                // Nom affiché, puis composantes du nom et organisation : une
                // fiche sans nom affiché se trouve encore.
                let noms: Vec<String> =
                    [Champ::NomAffiche, Champ::NomStructure, Champ::Organisation]
                        .into_iter()
                        .flat_map(|c| f.textes(c))
                        .map(|t| t.replace(';', " "))
                        .collect();
                filtre.retient(&f.textes(Champ::Groupes), &noms.join(" "))
            })
            .map(crate::fiche::Fiche::id)
            .collect()
    }
}

#[cfg(test)]
mod tests;

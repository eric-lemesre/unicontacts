//! Relations entre fiches : types, réciproques, fiche « moi », et leur
//! portage dans les vCard (ADR-0011).
//!
//! Une relation est une valeur du champ [`Champ::Relation`] de la fiche qui
//! la porte : son type et l'identifiant maître de la fiche liée, jamais son
//! nom. Le nom n'apparaît qu'à l'écriture vers une source qui ne sait
//! désigner la fiche liée autrement.
// [impl->req~liens-types~1]
// [impl->req~lien-par-identifiant-maitre~1]

use std::collections::BTreeMap;

use crate::champ::{Champ, Provenance, Valeur};
use crate::identifiants::FicheId;
use crate::referentiel::Referentiel;
use crate::temps::Horodatage;

/// Type d'une relation : l'un des types prédéfinis, ou un libellé choisi par
/// l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeRelation {
    /// Conjoint.
    Conjoint,
    /// Enfant.
    Enfant,
    /// Parent.
    Parent,
    /// Frère ou sœur.
    FrereSoeur,
    /// Ami.
    Ami,
    /// Collègue.
    Collegue,
    /// Médecin.
    Medecin,
    /// Type personnalisé, désigné par son libellé.
    Personnalise(String),
}

impl TypeRelation {
    /// Types prédéfinis, dans l'ordre de présentation.
    pub const PREDEFINIS: [Self; 7] = [
        Self::Conjoint,
        Self::Enfant,
        Self::Parent,
        Self::FrereSoeur,
        Self::Ami,
        Self::Collegue,
        Self::Medecin,
    ];

    /// Jeton enregistré du type : un mot fixe pour un type prédéfini, le
    /// libellé pour un type personnalisé.
    #[must_use]
    pub fn jeton(&self) -> &str {
        match self {
            Self::Conjoint => "conjoint",
            Self::Enfant => "enfant",
            Self::Parent => "parent",
            Self::FrereSoeur => "frere-soeur",
            Self::Ami => "ami",
            Self::Collegue => "collegue",
            Self::Medecin => "medecin",
            Self::Personnalise(libelle) => libelle,
        }
    }

    /// Libellé du type, tel que l'affiche l'application.
    #[must_use]
    pub fn libelle(&self) -> &str {
        match self {
            Self::Conjoint => "Conjoint",
            Self::Enfant => "Enfant",
            Self::Parent => "Parent",
            Self::FrereSoeur => "Frère/sœur",
            Self::Ami => "Ami",
            Self::Collegue => "Collègue",
            Self::Medecin => "Médecin",
            Self::Personnalise(libelle) => libelle,
        }
    }

    /// Type désigné par un jeton ou un libellé : un type prédéfini si le
    /// texte est son jeton ou son libellé, sans tenir compte de la casse ;
    /// sinon un type personnalisé, espaces de bord retirées.
    #[must_use]
    pub fn depuis_texte(texte: &str) -> Self {
        let texte = texte.trim();
        Self::PREDEFINIS
            .into_iter()
            .find(|t| {
                t.jeton().eq_ignore_ascii_case(texte)
                    || t.libelle().to_lowercase() == texte.to_lowercase()
            })
            .unwrap_or_else(|| Self::Personnalise(texte.to_owned()))
    }

    /// Inverse fixé d'un type prédéfini ; aucun pour médecin et les types
    /// personnalisés, dont l'inverse se déclare.
    // [impl->req~reciproques-automatiques~1]
    #[must_use]
    pub const fn inverse_predefini(&self) -> Option<Self> {
        match self {
            Self::Conjoint => Some(Self::Conjoint),
            Self::Enfant => Some(Self::Parent),
            Self::Parent => Some(Self::Enfant),
            Self::FrereSoeur => Some(Self::FrereSoeur),
            Self::Ami => Some(Self::Ami),
            Self::Collegue => Some(Self::Collegue),
            Self::Medecin | Self::Personnalise(_) => None,
        }
    }
}

/// Relation portée par une fiche : son type et la fiche liée.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Relation {
    /// Type de la relation.
    pub type_relation: TypeRelation,
    /// Identifiant maître de la fiche liée.
    pub fiche: FicheId,
}

impl Relation {
    /// Texte de la valeur du champ `Relation` : le jeton du type, `;`
    /// échappé, puis l'identifiant de la fiche liée.
    #[must_use]
    pub fn texte(&self) -> String {
        let jeton = self
            .type_relation
            .jeton()
            .replace('\\', "\\\\")
            .replace(';', "\\;");
        format!("{jeton};{}", self.fiche)
    }

    /// Relation lue d'une valeur du champ `Relation`, si elle est bien
    /// formée.
    #[must_use]
    pub fn depuis_texte(texte: &str) -> Option<Self> {
        let (jeton, fiche) = texte.rsplit_once(';')?;
        let fiche = fiche.trim().parse().ok()?;
        let mut type_texte = String::with_capacity(jeton.len());
        let mut caracteres = jeton.chars();
        while let Some(c) = caracteres.next() {
            if c == '\\' {
                type_texte.extend(caracteres.next());
            } else {
                type_texte.push(c);
            }
        }
        if type_texte.trim().is_empty() {
            return None;
        }
        Some(Self {
            type_relation: TypeRelation::depuis_texte(&type_texte),
            fiche,
        })
    }
}

/// Erreur d'une opération sur les relations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurRelation {
    /// Aucune fiche de cet identifiant.
    #[error("fiche inconnue : {0}")]
    FicheInconnue(FicheId),
    /// Une fiche ne se lie pas à elle-même.
    #[error("une fiche ne se lie pas à elle-même : {0}")]
    LienSurSoi(FicheId),
    /// Le libellé d'un type personnalisé est vide.
    #[error("le libellé d'un type de relation ne peut être vide")]
    LibelleVide,
    /// Ce type a déjà un inverse fixé.
    #[error("le type « {0} » a déjà un inverse fixé")]
    InverseFixe(String),
}

/// Inverses déclarés par l'utilisateur, valables dans les deux sens, entre
/// types sans inverse fixé (médecin et types personnalisés), par jeton.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Inverses(BTreeMap<String, String>);

impl Inverses {
    /// Couples déclarés, chacun une seule fois.
    pub fn couples(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .filter(|(a, b)| a <= b)
            .map(|(a, b)| (a.as_str(), b.as_str()))
    }

    /// Inverse d'un type : fixé pour un type prédéfini qui en a un, sinon
    /// celui que l'utilisateur a déclaré.
    // [impl->req~reciproques-automatiques~1]
    #[must_use]
    pub fn inverse(&self, type_relation: &TypeRelation) -> Option<TypeRelation> {
        type_relation.inverse_predefini().or_else(|| {
            self.0
                .get(type_relation.jeton())
                .map(|inverse| TypeRelation::depuis_texte(inverse))
        })
    }

    /// Déclare deux types inverses l'un de l'autre ; un type peut être son
    /// propre inverse. Remplace les déclarations précédentes de l'un et de
    /// l'autre.
    ///
    /// # Errors
    ///
    /// [`ErreurRelation::InverseFixe`] si l'un des types a un inverse fixé ;
    /// [`ErreurRelation::LibelleVide`] pour un libellé vide.
    pub fn declarer(&mut self, a: &TypeRelation, b: &TypeRelation) -> Result<(), ErreurRelation> {
        for t in [a, b] {
            if t.jeton().trim().is_empty() {
                return Err(ErreurRelation::LibelleVide);
            }
            if t.inverse_predefini().is_some() {
                return Err(ErreurRelation::InverseFixe(t.libelle().to_owned()));
            }
        }
        for t in [a, b] {
            if let Some(ancien) = self.0.remove(t.jeton()) {
                self.0.remove(&ancien);
            }
        }
        self.0.insert(a.jeton().to_owned(), b.jeton().to_owned());
        self.0.insert(b.jeton().to_owned(), a.jeton().to_owned());
        Ok(())
    }
}

impl Referentiel {
    /// Fiche du propriétaire, si une fiche est désignée « moi ».
    #[must_use]
    pub const fn moi(&self) -> Option<FicheId> {
        self.moi
    }

    /// Désigne la fiche du propriétaire ; la précédente perd cette qualité.
    /// `None` retire la désignation.
    ///
    /// # Errors
    ///
    /// [`ErreurRelation::FicheInconnue`] ; la désignation est alors
    /// inchangée.
    // [impl->req~fiche-moi~1]
    pub fn designer_moi(&mut self, fiche: Option<FicheId>) -> Result<(), ErreurRelation> {
        if let Some(id) = fiche
            && !self.fiches.contains_key(&id)
        {
            return Err(ErreurRelation::FicheInconnue(id));
        }
        self.moi = fiche;
        Ok(())
    }

    /// Inverses déclarés des types de relation.
    #[must_use]
    pub const fn inverses(&self) -> &Inverses {
        &self.inverses
    }

    /// Inverses déclarés, modifiables.
    pub const fn inverses_mut(&mut self) -> &mut Inverses {
        &mut self.inverses
    }

    /// Relations portées par une fiche, dans leur ordre ; vide pour une
    /// fiche inconnue.
    #[must_use]
    pub fn relations(&self, fiche: FicheId) -> Vec<Relation> {
        self.fiches.get(&fiche).map_or_else(Vec::new, |f| {
            f.valeurs(Champ::Relation)
                .iter()
                .filter_map(|v| Relation::depuis_texte(&v.texte))
                .collect()
        })
    }

    /// Fiches liées à la fiche « moi », dans l'ordre de ses relations, sans
    /// doublon : la section « Proches ».
    // [impl->req~section-proches~1]
    #[must_use]
    pub fn proches(&self) -> Vec<FicheId> {
        let Some(moi) = self.moi else {
            return Vec::new();
        };
        let mut proches = Vec::new();
        for relation in self.relations(moi) {
            if self.fiches.contains_key(&relation.fiche) && !proches.contains(&relation.fiche) {
                proches.push(relation.fiche);
            }
        }
        proches
    }

    /// Lie une fiche à une autre, et crée la réciproque si le type a un
    /// inverse. Un lien déjà présent n'est pas dupliqué.
    ///
    /// # Errors
    ///
    /// [`ErreurRelation`] si une fiche est inconnue, ou pour un lien d'une
    /// fiche vers elle-même ; rien n'est alors modifié.
    // [impl->req~liens-types~1]
    // [impl->req~reciproques-automatiques~1]
    pub fn lier(
        &mut self,
        fiche: FicheId,
        type_relation: TypeRelation,
        liee: FicheId,
        date: Horodatage,
    ) -> Result<(), ErreurRelation> {
        self.verifier_lien(fiche, &type_relation, liee)?;
        let inverse = self.inverses.inverse(&type_relation);
        self.ajouter_relation(fiche, type_relation, liee, date);
        if let Some(inverse) = inverse {
            self.ajouter_relation(liee, inverse, fiche, date);
        }
        Ok(())
    }

    /// Retire un lien, et sa réciproque si elle existe.
    ///
    /// # Errors
    ///
    /// [`ErreurRelation::FicheInconnue`] si une fiche est inconnue.
    // [impl->req~reciproques-automatiques~1]
    pub fn delier(
        &mut self,
        fiche: FicheId,
        type_relation: &TypeRelation,
        liee: FicheId,
    ) -> Result<(), ErreurRelation> {
        for id in [fiche, liee] {
            if !self.fiches.contains_key(&id) {
                return Err(ErreurRelation::FicheInconnue(id));
            }
        }
        let inverse = self.inverses.inverse(type_relation);
        self.retirer_relations(fiche, |r| {
            r.type_relation == *type_relation && r.fiche == liee
        });
        if let Some(inverse) = inverse {
            self.retirer_relations(liee, |r| r.type_relation == inverse && r.fiche == fiche);
        }
        Ok(())
    }

    fn verifier_lien(
        &self,
        fiche: FicheId,
        type_relation: &TypeRelation,
        liee: FicheId,
    ) -> Result<(), ErreurRelation> {
        for id in [fiche, liee] {
            if !self.fiches.contains_key(&id) {
                return Err(ErreurRelation::FicheInconnue(id));
            }
        }
        if fiche == liee {
            return Err(ErreurRelation::LienSurSoi(fiche));
        }
        if type_relation.jeton().trim().is_empty() {
            return Err(ErreurRelation::LibelleVide);
        }
        Ok(())
    }

    fn ajouter_relation(
        &mut self,
        fiche: FicheId,
        type_relation: TypeRelation,
        liee: FicheId,
        date: Horodatage,
    ) {
        let Some(porteuse) = self.fiches.get_mut(&fiche) else {
            return;
        };
        let relation = Relation {
            type_relation,
            fiche: liee,
        };
        let mut valeurs = porteuse.valeurs(Champ::Relation).to_vec();
        let deja = valeurs
            .iter()
            .any(|v| Relation::depuis_texte(&v.texte).as_ref() == Some(&relation));
        if !deja {
            valeurs.push(Valeur::new(relation.texte(), Provenance::referentiel(date)));
            porteuse.definir_multivalue(Champ::Relation, valeurs);
        }
    }

    fn retirer_relations(&mut self, fiche: FicheId, retirer: impl Fn(&Relation) -> bool) {
        let Some(porteuse) = self.fiches.get_mut(&fiche) else {
            return;
        };
        let valeurs: Vec<Valeur> = porteuse
            .valeurs(Champ::Relation)
            .iter()
            .filter(|v| !Relation::depuis_texte(&v.texte).is_some_and(|r| retirer(&r)))
            .cloned()
            .collect();
        porteuse.definir_multivalue(Champ::Relation, valeurs);
    }

    /// Retire toute trace d'une fiche disparue : les liens qui la désignent
    /// et sa qualité de fiche « moi ».
    pub(crate) fn oublier_fiche(&mut self, disparue: FicheId) {
        if self.moi == Some(disparue) {
            self.moi = None;
        }
        let porteuses: Vec<FicheId> = self.fiches.keys().copied().collect();
        for fiche in porteuses {
            self.retirer_relations(fiche, |r| r.fiche == disparue);
        }
    }

    /// Reporte sur une fiche les liens et la qualité « moi » d'une autre,
    /// réunie avec elle.
    pub(crate) fn rediriger_fiche(&mut self, absorbee: FicheId, reunie: FicheId) {
        if self.moi == Some(absorbee) {
            self.moi = Some(reunie);
        }
        for porteuse in self.fiches.values_mut() {
            let id = porteuse.id();
            let mut vues = Vec::new();
            let valeurs: Vec<Valeur> = porteuse
                .valeurs(Champ::Relation)
                .iter()
                .filter_map(|v| {
                    let Some(mut relation) = Relation::depuis_texte(&v.texte) else {
                        return Some(v.clone());
                    };
                    if relation.fiche == absorbee {
                        relation.fiche = reunie;
                    }
                    if relation.fiche == id || vues.contains(&relation) {
                        return None;
                    }
                    vues.push(relation.clone());
                    let mut valeur = v.clone();
                    valeur.texte = relation.texte();
                    Some(valeur)
                })
                .collect();
            porteuse.definir_multivalue(Champ::Relation, valeurs);
        }
    }
}

#[cfg(test)]
mod tests;

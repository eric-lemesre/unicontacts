//! Relations dans les vCard : `RELATED` en 4.0, convention Apple
//! (`X-ABRELATEDNAMES` et `X-ABLabel` d'un même groupe) en 3.0 (ADR-0011).
// [impl->req~synchronisation-relations~1]

use std::collections::BTreeMap;

use super::{Carte, Parametre, Propriete, Version};
use crate::identifiants::FicheId;
use crate::relations::{Relation, TypeRelation};

/// Mémoire des relations d'une copie : chaque écriture de relation déjà lue
/// ou écrite dans la source, et le texte de la relation qu'elle désigne.
pub type Memoire = BTreeMap<String, String>;

/// Ce qu'il faut savoir des fiches pour désigner une fiche liée dans une
/// source : l'UID de sa copie dans cette source, et son nom affiché.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Annuaire {
    par_uid: BTreeMap<String, FicheId>,
    uids: BTreeMap<FicheId, String>,
    par_nom: BTreeMap<String, Vec<FicheId>>,
    noms: BTreeMap<FicheId, String>,
}

impl Annuaire {
    /// Annuaire vide : aucune fiche liée ne s'y désigne.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute une fiche, avec son nom affiché et l'UID de sa copie dans la
    /// source, s'ils existent.
    pub fn ajouter(&mut self, fiche: FicheId, nom: Option<&str>, uid: Option<&str>) {
        if let Some(nom) = nom.map(str::trim).filter(|n| !n.is_empty()) {
            self.par_nom.entry(nom.to_owned()).or_default().push(fiche);
            self.noms.insert(fiche, nom.to_owned());
        }
        if let Some(uid) = uid.map(str::trim).filter(|u| !u.is_empty()) {
            self.par_uid.insert(uid.to_owned(), fiche);
            self.uids.insert(fiche, uid.to_owned());
        }
    }

    fn fiche_de_uid(&self, uid: &str) -> Option<FicheId> {
        self.par_uid.get(uid.trim()).copied()
    }

    /// Fiche d'un nom affiché, s'il n'est porté que par elle.
    fn fiche_de_nom(&self, nom: &str) -> Option<FicheId> {
        match self.par_nom.get(nom.trim()).map(Vec::as_slice) {
            Some([fiche]) => Some(*fiche),
            _ => None,
        }
    }

    /// Nom affiché d'une fiche, s'il la désigne seule.
    fn nom_unique(&self, fiche: FicheId) -> Option<&str> {
        self.noms
            .get(&fiche)
            .filter(|nom| self.fiche_de_nom(nom) == Some(fiche))
            .map(String::as_str)
    }

    /// Vrai si la relation peut s'écrire dans une carte de cette version :
    /// la fiche liée y a une copie (4.0), ou un nom qui la désigne seule.
    #[must_use]
    pub fn peut_porter(&self, texte: &str, version: Version) -> bool {
        Relation::depuis_texte(texte).is_some_and(|r| {
            (version == Version::V4 && self.uids.contains_key(&r.fiche))
                || self.nom_unique(r.fiche).is_some()
        })
    }
}

/// Relation lue d'une propriété.
pub(super) struct Lue {
    /// Texte de la relation, au format du champ `Relation`.
    pub(super) texte: String,
    /// Propriété compagne, qui porte le libellé (convention Apple).
    pub(super) compagnon: Option<usize>,
    /// Écriture de la relation, clé de la mémoire.
    pub(super) rendu: String,
}

/// Relation portée par la propriété d'indice donné, si c'en est une que
/// l'on sait résoudre ; sinon la propriété reste conservée telle quelle.
pub(super) fn lire(
    carte: &Carte,
    indice: usize,
    annuaire: &Annuaire,
    memoire: &Memoire,
) -> Option<Lue> {
    let propriete = carte.proprietes.get(indice)?;
    let (type_relation, valeur, compagnon, rendu) = if propriete.nom == "RELATED" {
        let types = propriete.types();
        let libelle = parametre(propriete, "X-ABLABEL");
        let type_relation = match &libelle {
            Some(libelle) => TypeRelation::depuis_texte(libelle),
            None => types.iter().find_map(|t| type_rfc(t))?,
        };
        let texte = parametre(propriete, "VALUE").is_some_and(|v| v.eq_ignore_ascii_case("text"));
        let valeur = if texte {
            decoder(&propriete.valeur)
        } else {
            propriete.valeur.trim().to_owned()
        };
        let rendu = format!(
            "RELATED|{}|{}|{valeur}",
            types.join(","),
            libelle.unwrap_or_default()
        );
        (type_relation, valeur, None, rendu)
    } else if propriete.nom == "X-ABRELATEDNAMES" {
        let groupe = propriete.groupe.as_deref()?;
        let (indice_libelle, libelle) =
            carte.proprietes.iter().enumerate().find_map(|(i, p)| {
                (p.nom == "X-ABLABEL"
                    && p.groupe
                        .as_deref()
                        .is_some_and(|g| g.eq_ignore_ascii_case(groupe)))
                .then(|| (i, decoder(&p.valeur)))
            })?;
        let valeur = decoder(&propriete.valeur);
        let rendu = format!("X-ABRELATEDNAMES|{libelle}|{valeur}");
        (type_apple(&libelle)?, valeur, Some(indice_libelle), rendu)
    } else {
        return None;
    };
    if let Some(texte) = memoire.get(&rendu) {
        return Some(Lue {
            texte: texte.clone(),
            compagnon,
            rendu,
        });
    }
    let fiche = if propriete.nom == "X-ABRELATEDNAMES"
        || parametre(propriete, "VALUE").is_some_and(|v| v.eq_ignore_ascii_case("text"))
    {
        annuaire.fiche_de_nom(&valeur)?
    } else {
        let uid = valeur
            .get(..9)
            .filter(|debut| debut.eq_ignore_ascii_case("urn:uuid:"))
            .and_then(|_| valeur.get(9..))
            .unwrap_or(&valeur);
        annuaire
            .fiche_de_uid(uid)
            .or_else(|| annuaire.fiche_de_uid(&valeur))?
    };
    Some(Lue {
        texte: Relation {
            type_relation,
            fiche,
        }
        .texte(),
        compagnon,
        rendu,
    })
}

/// Propriétés qui écrivent une relation dans une carte de cette version ;
/// aucune si la fiche liée ne peut s'y désigner. `groupe` est le dernier
/// numéro de groupe `itemN` employé dans la carte.
pub(super) fn produire(
    texte: &str,
    version: Version,
    annuaire: &Annuaire,
    groupe: &mut u32,
) -> Vec<Propriete> {
    let Some(relation) = Relation::depuis_texte(texte) else {
        return Vec::new();
    };
    match version {
        Version::V4 => {
            let mut parametres = vec![Parametre {
                nom: "TYPE".to_owned(),
                valeurs: vec![
                    rfc_de(&relation.type_relation)
                        .unwrap_or("contact")
                        .to_owned(),
                ],
            }];
            if rfc_de(&relation.type_relation).is_none() {
                parametres.push(Parametre {
                    nom: "X-ABLABEL".to_owned(),
                    valeurs: vec![relation.type_relation.libelle().to_owned()],
                });
            }
            let valeur = if let Some(uid) = annuaire.uids.get(&relation.fiche) {
                if uid.contains(':') {
                    uid.clone()
                } else {
                    format!("urn:uuid:{uid}")
                }
            } else if let Some(nom) = annuaire.nom_unique(relation.fiche) {
                parametres.push(Parametre {
                    nom: "VALUE".to_owned(),
                    valeurs: vec!["text".to_owned()],
                });
                encoder(nom)
            } else {
                return Vec::new();
            };
            vec![Propriete::nouvelle("RELATED", parametres, valeur)]
        }
        Version::V3 => {
            let Some(nom) = annuaire.nom_unique(relation.fiche) else {
                return Vec::new();
            };
            *groupe += 1;
            let item = format!("item{groupe}");
            let mut nom_lie = Propriete::nouvelle("X-ABRELATEDNAMES", Vec::new(), encoder(nom));
            nom_lie.groupe = Some(item.clone());
            let mut libelle = Propriete::nouvelle(
                "X-ABLABEL",
                Vec::new(),
                encoder(&libelle_apple(&relation.type_relation)),
            );
            libelle.groupe = Some(item);
            vec![nom_lie, libelle]
        }
    }
}

/// Plus grand numéro `N` des groupes `itemN` des propriétés.
pub(super) fn dernier_groupe<'a>(proprietes: impl Iterator<Item = &'a Propriete>) -> u32 {
    proprietes
        .filter_map(|p| p.groupe.as_deref())
        .filter_map(|g| {
            g.get(..4)
                .filter(|debut| debut.eq_ignore_ascii_case("item"))
                .and_then(|_| g.get(4..))
        })
        .filter_map(|n| n.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
}

fn parametre(propriete: &Propriete, nom: &str) -> Option<String> {
    propriete
        .parametres
        .iter()
        .find(|p| p.nom.eq_ignore_ascii_case(nom))
        .and_then(|p| p.valeurs.first())
        .cloned()
}

/// Type d'une valeur de `TYPE` du RFC 6350, s'il a un équivalent.
fn type_rfc(jeton: &str) -> Option<TypeRelation> {
    match jeton {
        "spouse" => Some(TypeRelation::Conjoint),
        "child" => Some(TypeRelation::Enfant),
        "parent" => Some(TypeRelation::Parent),
        "sibling" => Some(TypeRelation::FrereSoeur),
        "friend" => Some(TypeRelation::Ami),
        "colleague" | "co-worker" => Some(TypeRelation::Collegue),
        _ => None,
    }
}

/// Valeur de `TYPE` du RFC 6350 d'un type, s'il en a une.
const fn rfc_de(type_relation: &TypeRelation) -> Option<&'static str> {
    match type_relation {
        TypeRelation::Conjoint => Some("spouse"),
        TypeRelation::Enfant => Some("child"),
        TypeRelation::Parent => Some("parent"),
        TypeRelation::FrereSoeur => Some("sibling"),
        TypeRelation::Ami => Some("friend"),
        TypeRelation::Collegue => Some("colleague"),
        TypeRelation::Medecin | TypeRelation::Personnalise(_) => None,
    }
}

/// Type d'un libellé Apple : `_$!<Spouse>!$_` pour un libellé prédéfini,
/// sinon le libellé en clair. Aucun pour un libellé vide.
fn type_apple(libelle: &str) -> Option<TypeRelation> {
    let interne = libelle
        .trim()
        .strip_prefix("_$!<")
        .and_then(|l| l.strip_suffix(">!$_"));
    let Some(interne) = interne else {
        return (!libelle.trim().is_empty()).then(|| TypeRelation::depuis_texte(libelle));
    };
    Some(match interne.to_ascii_lowercase().as_str() {
        "spouse" | "partner" | "husband" | "wife" => TypeRelation::Conjoint,
        "child" | "son" | "daughter" => TypeRelation::Enfant,
        "parent" | "father" | "mother" => TypeRelation::Parent,
        "sibling" | "brother" | "sister" => TypeRelation::FrereSoeur,
        "friend" => TypeRelation::Ami,
        _ => TypeRelation::depuis_texte(interne),
    })
}

/// Libellé Apple d'un type.
fn libelle_apple(type_relation: &TypeRelation) -> String {
    match type_relation {
        TypeRelation::Conjoint => "_$!<Spouse>!$_".to_owned(),
        TypeRelation::Enfant => "_$!<Child>!$_".to_owned(),
        TypeRelation::Parent => "_$!<Parent>!$_".to_owned(),
        TypeRelation::Ami => "_$!<Friend>!$_".to_owned(),
        autre => autre.libelle().to_owned(),
    }
}

fn decoder(valeur: &str) -> String {
    super::ecriture::decoder(crate::champ::Champ::Note, valeur)
}

fn encoder(texte: &str) -> String {
    super::ecriture::encoder(crate::champ::Champ::Note, texte)
}

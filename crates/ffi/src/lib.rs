//! Interface étrangère d'unicontacts (ADR-0006, ADR-0009) : le cœur et le
//! connecteur CardDAV vus de Kotlin, par UniFFI.
//!
//! Un seul objet, [`Coeur`], tient le référentiel en mémoire et sa base
//! SQLite. Chaque opération qui modifie le référentiel l'enregistre aussitôt,
//! en une transaction courte. La synchronisation d'une source ne verrouille
//! le référentiel qu'aux phases locales (fusion, report), jamais pendant le
//! réseau (ADR-0007). Aucune logique métier ne vit côté Kotlin : ce crate
//! traduit, il ne décide rien.
// [impl->req~synchronisation-directe~1]
// [impl->req~ecriture-sous-compte-unique~1]

uniffi::setup_scaffolding!();

mod modele;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use unicontacts_carddav::{ClientCardDav, Compte, MotDePasse, envoyer, lire_carnet, reporter};
use unicontacts_core::referentiel::Referentiel;
use unicontacts_core::relations::TypeRelation;
use unicontacts_core::stockage::Stockage;
use unicontacts_core::vcard::Version;
use unicontacts_core::{DecisionId, Fiche, Horodatage, SourceId};

pub use modele::{
    BilanFfi, ChampFfi, ChoixFfi, DecisionFfi, FicheFfi, FiltreFfi, NatureFfi, RelationFfi,
    ResolutionFfi, ResumeFicheFfi, SourceFfi, StatutFfi, TypeRelationFfi, ValeurFfi,
};

/// Erreur rendue à Kotlin.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum ErreurFfi {
    /// La base ne s'ouvre pas, ou ne s'enregistre pas.
    #[error("stockage : {0}")]
    Stockage(String),
    /// Objet introuvable (fiche, source, décision).
    #[error("introuvable : {0}")]
    Introuvable(String),
    /// Demande refusée par le cœur.
    #[error("refusé : {0}")]
    Refuse(String),
    /// Le carnet CardDAV n'a pu être lu.
    #[error("carnet : {0}")]
    Carnet(String),
    /// État interne inutilisable : un fil a échoué en tenant le verrou.
    #[error("état interne inutilisable")]
    Interne,
}

impl ErreurFfi {
    fn refuse(erreur: impl std::fmt::Display) -> Self {
        Self::Refuse(erreur.to_string())
    }
}

/// Référentiel ouvert et sa base.
struct Etat {
    referentiel: Referentiel,
    stockage: Stockage,
}

impl Etat {
    fn enregistrer(&mut self) -> Result<(), ErreurFfi> {
        self.stockage
            .enregistrer(&self.referentiel)
            .map_err(|e| ErreurFfi::Stockage(e.to_string()))
    }
}

/// Le cœur d'unicontacts, ouvert sur sa base.
#[derive(uniffi::Object)]
pub struct Coeur {
    etat: Mutex<Etat>,
}

fn identifiant<T: FromStr>(texte: &str, quoi: &str) -> Result<T, ErreurFfi> {
    texte
        .parse()
        .map_err(|_| ErreurFfi::Introuvable(format!("{quoi} {texte}")))
}

#[uniffi::export]
impl Coeur {
    /// Ouvre la base du référentiel, la crée au besoin.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Stockage`] si la base ne s'ouvre pas.
    #[uniffi::constructor]
    pub fn ouvrir(chemin: String) -> Result<Arc<Self>, ErreurFfi> {
        let stockage = Stockage::ouvrir(&PathBuf::from(chemin))
            .map_err(|e| ErreurFfi::Stockage(e.to_string()))?;
        let referentiel = stockage
            .charger()
            .map_err(|e| ErreurFfi::Stockage(e.to_string()))?;
        Ok(Arc::new(Self {
            etat: Mutex::new(Etat {
                referentiel,
                stockage,
            }),
        }))
    }

    /// Résumé de toutes les fiches, triées par nom.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    pub fn fiches(&self) -> Result<Vec<ResumeFicheFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        let mut fiches: Vec<ResumeFicheFfi> =
            etat.referentiel.fiches().map(modele::resume).collect();
        fiches.sort_by_key(|f| f.nom.to_lowercase());
        Ok(fiches)
    }

    /// Toutes les fiches en entier : ce qu'Android écrit dans ses contacts,
    /// une fois chacune, sous le seul compte de l'application.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    // [impl->req~ecriture-sous-compte-unique~1]
    pub fn fiches_completes(&self) -> Result<Vec<FicheFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        Ok(etat
            .referentiel
            .fiches()
            .map(|f| modele::fiche(f, &etat.referentiel))
            .collect())
    }

    /// Une fiche en entier, valeurs et provenance comprises.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Introuvable`] si la fiche n'existe pas.
    pub fn fiche(&self, id: String) -> Result<FicheFfi, ErreurFfi> {
        let etat = self.verrou()?;
        let fiche = etat
            .referentiel
            .fiche(identifiant(&id, "fiche")?)
            .ok_or_else(|| ErreurFfi::Introuvable(format!("fiche {id}")))?;
        Ok(modele::fiche(fiche, &etat.referentiel))
    }

    /// Crée une fiche vide et rend son identifiant.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Stockage`] si l'enregistrement échoue.
    pub fn creer_fiche(&self) -> Result<String, ErreurFfi> {
        let mut etat = self.verrou()?;
        let id = etat.referentiel.ajouter(Fiche::nouvelle());
        etat.enregistrer()?;
        Ok(id.to_string())
    }

    /// Modifie un champ d'une fiche dans l'application.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Introuvable`] si la fiche n'existe pas ;
    /// [`ErreurFfi::Refuse`] si un champ à valeur unique reçoit plusieurs
    /// valeurs.
    pub fn modifier_fiche(
        &self,
        id: String,
        champ: ChampFfi,
        valeurs: Vec<String>,
    ) -> Result<(), ErreurFfi> {
        if matches!(champ, ChampFfi::Relation | ChampFfi::Groupes) {
            return Err(ErreurFfi::Refuse(
                "relations et groupes ont leurs propres opérations".to_owned(),
            ));
        }
        let mut etat = self.verrou()?;
        let fiche = etat
            .referentiel
            .fiche_mut(identifiant(&id, "fiche")?)
            .ok_or_else(|| ErreurFfi::Introuvable(format!("fiche {id}")))?;
        let textes: Vec<&str> = valeurs
            .iter()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .collect();
        fiche
            .modifier(champ.into(), &textes, Horodatage::maintenant())
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Lie une fiche à une autre par un type de relation (code ou libellé),
    /// et crée la réciproque si le type a un inverse.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Introuvable`] pour un identifiant mal formé ;
    /// [`ErreurFfi::Refuse`] si une fiche est inconnue, ou pour un lien
    /// d'une fiche vers elle-même.
    // [impl->req~liens-types~1]
    // [impl->req~reciproques-automatiques~1]
    pub fn lier(&self, id: String, type_relation: String, liee: String) -> Result<(), ErreurFfi> {
        let (fiche, liee) = (identifiant(&id, "fiche")?, identifiant(&liee, "fiche")?);
        let mut etat = self.verrou()?;
        etat.referentiel
            .lier(
                fiche,
                TypeRelation::depuis_texte(&type_relation),
                liee,
                Horodatage::maintenant(),
            )
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Retire un lien, et sa réciproque.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Introuvable`] pour un identifiant mal formé ;
    /// [`ErreurFfi::Refuse`] si une fiche est inconnue.
    pub fn delier(&self, id: String, type_relation: String, liee: String) -> Result<(), ErreurFfi> {
        let (fiche, liee) = (identifiant(&id, "fiche")?, identifiant(&liee, "fiche")?);
        let mut etat = self.verrou()?;
        etat.referentiel
            .delier(fiche, &TypeRelation::depuis_texte(&type_relation), liee)
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Désigne la fiche du propriétaire, ou retire la désignation.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] si la fiche est inconnue.
    // [impl->req~fiche-moi~1]
    pub fn designer_moi(&self, id: Option<String>) -> Result<(), ErreurFfi> {
        let fiche = id.map(|id| identifiant(&id, "fiche")).transpose()?;
        let mut etat = self.verrou()?;
        etat.referentiel
            .designer_moi(fiche)
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Fiches liées à la fiche « moi » : la section « Proches ».
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    // [impl->req~section-proches~1]
    pub fn proches(&self) -> Result<Vec<ResumeFicheFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        Ok(etat
            .referentiel
            .proches()
            .into_iter()
            .filter_map(|id| etat.referentiel.fiche(id))
            .map(modele::resume)
            .collect())
    }

    /// Types de relation à proposer.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    pub fn types_relation(&self) -> Result<Vec<TypeRelationFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        Ok(modele::types_relation(&etat.referentiel))
    }

    /// Déclare deux types de relation inverses l'un de l'autre.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] si l'un a un inverse fixé, ou un libellé vide.
    // [impl->req~reciproques-automatiques~1]
    pub fn declarer_inverse(
        &self,
        type_relation: String,
        inverse: String,
    ) -> Result<(), ErreurFfi> {
        let mut etat = self.verrou()?;
        etat.referentiel
            .inverses_mut()
            .declarer(
                &TypeRelation::depuis_texte(&type_relation),
                &TypeRelation::depuis_texte(&inverse),
            )
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Tous les groupes, par ordre de chemin.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    pub fn groupes(&self) -> Result<Vec<String>, ErreurFfi> {
        Ok(self.verrou()?.referentiel.groupes())
    }

    /// Crée un groupe vide, par son chemin, et rend le chemin normalisé.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] pour un chemin vide ou déjà pris.
    // [impl->req~groupes-hierarchiques~1]
    pub fn creer_groupe(&self, chemin: String) -> Result<String, ErreurFfi> {
        let mut etat = self.verrou()?;
        let chemin = etat
            .referentiel
            .creer_groupe(&chemin)
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()?;
        Ok(chemin)
    }

    /// Range une fiche dans les groupes donnés, à la place des précédents.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Introuvable`] pour un identifiant mal formé ;
    /// [`ErreurFfi::Refuse`] si la fiche est inconnue.
    // [impl->req~appartenance-multiple~1]
    pub fn ranger(&self, id: String, groupes: Vec<String>) -> Result<(), ErreurFfi> {
        let fiche = identifiant(&id, "fiche")?;
        let mut etat = self.verrou()?;
        let groupes: Vec<&str> = groupes.iter().map(String::as_str).collect();
        etat.referentiel
            .ranger(fiche, &groupes, Horodatage::maintenant())
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Renomme un groupe, ou le déplace par un nouveau chemin, sous-groupes
    /// et fiches compris, en une seule opération.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] : groupe inconnu, chemin pris ou vide, cycle.
    // [impl->req~renommage-groupe-operation-unique~1]
    pub fn renommer_groupe(&self, ancien: String, nouveau: String) -> Result<String, ErreurFfi> {
        let mut etat = self.verrou()?;
        let chemin = etat
            .referentiel
            .renommer_groupe(&ancien, &nouveau, Horodatage::maintenant())
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()?;
        Ok(chemin)
    }

    /// Donne à un groupe un autre parent, ou aucun.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] : groupe inconnu, chemin pris, cycle.
    // [impl->req~groupes-hierarchiques~1]
    pub fn deplacer_groupe(
        &self,
        groupe: String,
        parent: Option<String>,
    ) -> Result<String, ErreurFfi> {
        let mut etat = self.verrou()?;
        let chemin = etat
            .referentiel
            .deplacer_groupe(&groupe, parent.as_deref(), Horodatage::maintenant())
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()?;
        Ok(chemin)
    }

    /// Supprime un groupe et ses sous-groupes ; les fiches demeurent.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Stockage`] si l'enregistrement échoue.
    pub fn supprimer_groupe(&self, groupe: String) -> Result<(), ErreurFfi> {
        let mut etat = self.verrou()?;
        etat.referentiel.supprimer_groupe(&groupe);
        etat.enregistrer()
    }

    /// Fiches qui passent le filtre, triées par nom.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    // [impl->req~filtre-par-groupes~2]
    pub fn fiches_filtrees(&self, filtre: FiltreFfi) -> Result<Vec<ResumeFicheFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        let mut fiches: Vec<ResumeFicheFfi> = etat
            .referentiel
            .filtrer(&filtre.into())
            .into_iter()
            .filter_map(|id| etat.referentiel.fiche(id))
            .map(modele::resume)
            .collect();
        fiches.sort_by_key(|f| f.nom.to_lowercase());
        Ok(fiches)
    }

    /// Sources où la suppression d'une fiche peut s'appliquer.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Introuvable`] si la fiche n'existe pas.
    pub fn sources_de_suppression(&self, id: String) -> Result<Vec<SourceFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        let ids = etat
            .referentiel
            .sources_de_suppression(identifiant(&id, "fiche")?)
            .map_err(|e| ErreurFfi::Introuvable(e.to_string()))?;
        let sources = etat.referentiel.sources();
        Ok(ids
            .into_iter()
            .filter_map(|s| sources.source(s))
            .map(modele::source)
            .collect())
    }

    /// Supprime une fiche, et des sources choisies à leur prochaine
    /// synchronisation.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] si une source choisie n'est pas proposée.
    pub fn supprimer_fiche(&self, id: String, sources: Vec<String>) -> Result<(), ErreurFfi> {
        let mut etat = self.verrou()?;
        let choisies = sources
            .iter()
            .map(|s| identifiant::<SourceId>(s, "source"))
            .collect::<Result<Vec<_>, _>>()?;
        etat.referentiel
            .supprimer_fiche(identifiant(&id, "fiche")?, &choisies)
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Sources configurées.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    pub fn sources(&self) -> Result<Vec<SourceFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        Ok(etat
            .referentiel
            .sources()
            .toutes()
            .map(modele::source)
            .collect())
    }

    /// Ajoute une source, aux réglages par défaut, et rend son identifiant.
    /// Les paramètres de connexion (URL, identifiant, mot de passe) restent
    /// côté Android : le cœur ne les garde pas.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Stockage`] si l'enregistrement échoue.
    pub fn ajouter_source(&self, nom: String, vcard4: bool) -> Result<String, ErreurFfi> {
        let mut etat = self.verrou()?;
        let version = if vcard4 { Version::V4 } else { Version::V3 };
        let id = etat.referentiel.sources_mut().ajouter(nom, version);
        etat.enregistrer()?;
        Ok(id.to_string())
    }

    /// Nombre de décisions en attente ou reportées.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    // [impl->req~boite-de-decisions~1]
    pub fn nombre_decisions(&self) -> Result<u32, ErreurFfi> {
        let etat = self.verrou()?;
        Ok(u32::try_from(etat.referentiel.ouvertes().count()).unwrap_or(u32::MAX))
    }

    /// Décisions en attente ou reportées, les plus anciennes d'abord, avec
    /// pour chaque champ les valeurs en présence, leur source et leur date.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Interne`] si l'état est inutilisable.
    // [impl->req~boite-de-decisions~1]
    // [impl->req~vue-comparee~1]
    pub fn decisions(&self) -> Result<Vec<DecisionFfi>, ErreurFfi> {
        let etat = self.verrou()?;
        Ok(etat
            .referentiel
            .ouvertes()
            .map(|d| modele::decision(d, &etat.referentiel))
            .collect())
    }

    /// Reporte une décision.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] si la décision est inconnue ou résolue.
    pub fn reporter(&self, id: String) -> Result<(), ErreurFfi> {
        let mut etat = self.verrou()?;
        etat.referentiel
            .reporter(identifiant::<DecisionId>(&id, "décision")?)
            .map_err(ErreurFfi::refuse)?;
        etat.enregistrer()
    }

    /// Nombre de cas similaires en attente, à annoncer avant une résolution
    /// en lot.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] si la décision est inconnue ou résolue.
    pub fn cas_similaires(&self, id: String) -> Result<u32, ErreurFfi> {
        let etat = self.verrou()?;
        let cas = etat
            .referentiel
            .cas_similaires(identifiant(&id, "décision")?)
            .map_err(ErreurFfi::refuse)?;
        Ok(u32::try_from(cas.len()).unwrap_or(u32::MAX))
    }

    /// Résout une décision, et ses cas similaires si demandé ; rend le
    /// nombre de décisions résolues.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Refuse`] si la résolution ne convient pas.
    pub fn resoudre(
        &self,
        id: String,
        resolution: ResolutionFfi,
        cas_similaires: bool,
    ) -> Result<u32, ErreurFfi> {
        let mut etat = self.verrou()?;
        let id: DecisionId = identifiant(&id, "décision")?;
        let resolution = resolution.into();
        let date = Horodatage::maintenant();
        let resolues = if cas_similaires {
            etat.referentiel
                .resoudre_avec_cas_similaires(id, &resolution, date)
                .map_err(ErreurFfi::refuse)?
        } else {
            etat.referentiel
                .resoudre(id, resolution, date)
                .map_err(ErreurFfi::refuse)?;
            1
        };
        etat.enregistrer()?;
        Ok(u32::try_from(resolues).unwrap_or(u32::MAX))
    }

    /// Synchronise une source CardDAV, directement avec son serveur.
    ///
    /// Le référentiel n'est verrouillé qu'à la fusion et au report ; la
    /// lecture du carnet et les écritures se font hors verrou, et la base
    /// s'enregistre à la fin, en une transaction.
    ///
    /// # Errors
    ///
    /// [`ErreurFfi::Carnet`] si le carnet ne peut être lu ;
    /// [`ErreurFfi::Introuvable`] si la source n'est pas configurée.
    // [impl->req~synchronisation-directe~1]
    pub fn synchroniser_carddav(
        &self,
        source: String,
        carnet: String,
        utilisateur: String,
        mot_de_passe: String,
    ) -> Result<BilanFfi, ErreurFfi> {
        let source: SourceId = identifiant(&source, "source")?;
        let client = ClientCardDav::new(&Compte {
            carnet,
            utilisateur,
            mot_de_passe: MotDePasse::new(mot_de_passe),
        })
        .map_err(|e| ErreurFfi::Carnet(e.to_string()))?;

        let lecture = lire_carnet(&client).map_err(|e| ErreurFfi::Carnet(e.to_string()))?;
        let rapport = {
            let mut etat = self.verrou()?;
            etat.referentiel
                .synchroniser(source, &lecture.copies, Horodatage::maintenant())
                .map_err(|e| ErreurFfi::Introuvable(e.to_string()))?
        };
        let issues = envoyer(&client, &lecture, &rapport);
        let mut etat = self.verrou()?;
        let bilan = reporter(&mut etat.referentiel, source, rapport, issues);
        etat.enregistrer()?;
        Ok(modele::bilan(&bilan))
    }
}

impl Coeur {
    fn verrou(&self) -> Result<MutexGuard<'_, Etat>, ErreurFfi> {
        self.etat.lock().map_err(|_| ErreurFfi::Interne)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Essai = Result<(), Box<dyn std::error::Error>>;

    /// Base jetable, retirée à la fin de l'essai.
    struct Base(PathBuf);

    impl Base {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!(
                "unicontacts-ffi-{}.sqlite",
                unicontacts_core::FicheId::nouveau()
            )))
        }

        fn chemin(&self) -> String {
            self.0.to_string_lossy().into_owned()
        }
    }

    impl Drop for Base {
        fn drop(&mut self) {
            for suffixe in ["", "-wal", "-shm"] {
                let mut chemin = self.0.clone().into_os_string();
                chemin.push(suffixe);
                // Nettoyage d'essai : un fichier déjà absent n'est pas une erreur.
                let _ = std::fs::remove_file(chemin);
            }
        }
    }

    /// Fiche nommée, créée dans le cœur.
    fn fiche_nommee(coeur: &Coeur, nom: &str) -> Result<String, ErreurFfi> {
        let id = coeur.creer_fiche()?;
        coeur.modifier_fiche(id.clone(), ChampFfi::NomAffiche, vec![nom.to_owned()])?;
        Ok(id)
    }

    #[test]
    fn une_relation_se_presente_par_le_nom_de_la_fiche_liee_et_survit_a_la_reouverture() -> Essai {
        let base = Base::new();
        let (eric, camille) = {
            let coeur = Coeur::ouvrir(base.chemin())?;
            let (eric, camille) = (
                fiche_nommee(&coeur, "Eric")?,
                fiche_nommee(&coeur, "Camille")?,
            );
            coeur.designer_moi(Some(eric.clone()))?;
            coeur.lier(eric.clone(), "Conjoint".to_owned(), camille.clone())?;
            (eric, camille)
        };

        let coeur = Coeur::ouvrir(base.chemin())?;

        let fiche = coeur.fiche(eric)?;
        let relations: Vec<(String, String)> = fiche
            .relations
            .into_iter()
            .map(|r| (r.code, r.nom))
            .collect();
        let proches: Vec<String> = coeur.proches()?.into_iter().map(|p| p.id).collect();
        assert_eq!(
            (relations, fiche.est_moi, proches),
            (
                vec![("conjoint".to_owned(), "Camille".to_owned())],
                true,
                vec![camille]
            )
        );
        Ok(())
    }

    #[test]
    fn le_filtre_retient_les_fiches_rangees_dans_un_sous_groupe() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;
        let (alix, basile) = (
            fiche_nommee(&coeur, "Alix")?,
            fiche_nommee(&coeur, "Basile")?,
        );
        coeur.ranger(alix.clone(), vec!["Famille/Cousins".to_owned()])?;
        coeur.ranger(basile, vec!["Voisins".to_owned()])?;

        let retenues: Vec<String> = coeur
            .fiches_filtrees(FiltreFfi {
                groupes: vec!["Famille".to_owned()],
                intersection: false,
                sous_groupes: true,
                recherche: String::new(),
            })?
            .into_iter()
            .map(|f| f.id)
            .collect();

        assert_eq!(retenues, [alix]);
        Ok(())
    }

    #[test]
    fn une_relation_ne_se_saisit_pas_comme_un_champ() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;
        let id = coeur.creer_fiche()?;

        let refus = coeur.modifier_fiche(id, ChampFfi::Relation, vec!["ami;x".to_owned()]);

        assert!(matches!(refus, Err(ErreurFfi::Refuse(_))), "{refus:?}");
        Ok(())
    }

    #[test]
    fn les_types_proposes_comptent_un_personnalise_avec_son_inverse() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;
        coeur.declarer_inverse("Parrain".to_owned(), "Filleul".to_owned())?;

        let parrain = coeur
            .types_relation()?
            .into_iter()
            .find(|t| t.libelle == "Parrain")
            .ok_or("type Parrain attendu")?;

        assert_eq!(
            (parrain.inverse.as_deref(), parrain.predefini),
            (Some("Filleul"), false)
        );
        Ok(())
    }

    #[test]
    fn une_fiche_modifiee_se_retrouve_a_la_reouverture() -> Essai {
        let base = Base::new();
        let id = {
            let coeur = Coeur::ouvrir(base.chemin())?;
            let id = coeur.creer_fiche()?;
            coeur.modifier_fiche(
                id.clone(),
                ChampFfi::NomAffiche,
                vec!["Camille Exemple".to_owned()],
            )?;
            id
        };

        let relue = Coeur::ouvrir(base.chemin())?.fiche(id)?;

        assert_eq!(relue.nom, "Camille Exemple");
        Ok(())
    }

    #[test]
    fn une_valeur_saisie_vient_de_unicontacts() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;
        let id = coeur.creer_fiche()?;
        coeur.modifier_fiche(
            id.clone(),
            ChampFfi::Email,
            vec!["a@example.org".to_owned()],
        )?;

        let origines: Vec<String> = coeur
            .fiche(id)?
            .champs
            .into_iter()
            .flat_map(|c| c.valeurs)
            .map(|v| v.origine)
            .collect();

        assert_eq!(origines, ["unicontacts"]);
        Ok(())
    }

    #[test]
    fn deux_valeurs_pour_un_champ_unique_sont_refusees() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;
        let id = coeur.creer_fiche()?;

        let refus = coeur.modifier_fiche(
            id,
            ChampFfi::NomAffiche,
            vec!["Un".to_owned(), "Deux".to_owned()],
        );

        assert!(matches!(refus, Err(ErreurFfi::Refuse(_))), "{refus:?}");
        Ok(())
    }

    #[test]
    fn une_fiche_inconnue_est_introuvable() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;

        let refus = coeur.fiche("pas-un-identifiant".to_owned());

        assert!(matches!(refus, Err(ErreurFfi::Introuvable(_))), "{refus:?}");
        Ok(())
    }

    #[test]
    fn une_source_ajoutee_est_listee() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;

        let id = coeur.ajouter_source("Murena".to_owned(), false)?;

        assert_eq!(
            coeur.sources()?,
            [SourceFfi {
                id,
                nom: "Murena".to_owned(),
                vcard4: false
            }]
        );
        Ok(())
    }

    #[test]
    fn un_carnet_injoignable_laisse_le_referentiel_intact() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;
        let source = coeur.ajouter_source("Murena".to_owned(), false)?;

        let refus = coeur.synchroniser_carddav(
            source,
            "http://127.0.0.1:9/carnet/".to_owned(),
            "u".to_owned(),
            "p".to_owned(),
        );

        assert_eq!(
            (
                matches!(refus, Err(ErreurFfi::Carnet(_))),
                coeur.fiches()?.len()
            ),
            (true, 0)
        );
        Ok(())
    }

    #[test]
    fn un_referentiel_neuf_n_a_aucune_decision() -> Essai {
        let base = Base::new();
        let coeur = Coeur::ouvrir(base.chemin())?;

        assert_eq!(coeur.nombre_decisions()?, 0);
        Ok(())
    }
}

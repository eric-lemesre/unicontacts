//! Stockage du référentiel dans une base SQLite locale (ADR-0003).
//!
//! Le référentiel se charge en entier et s'enregistre en entier, dans une
//! seule transaction d'écriture : un carnet d'adresses tient en mémoire, et
//! la base reste à chaque instant l'image d'un état cohérent.
//!
//! Discipline de transaction : l'écriture prend son verrou d'emblée
//! (`BEGIN IMMEDIATE`, sans escalade d'un verrou de lecture) ; elle ne
//! contient que des ordres SQL préparés, jamais d'attente extérieure (réseau,
//! analyse de vCard) ; elle touche les tables toujours dans le même ordre —
//! enfants avant parents pour vider, parents avant enfants pour remplir.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::champ::{Champ, Origine, Provenance, Valeur};
use crate::decisions::{
    Action, Decision, IssueDoublon, IssueSuppression, Nature, Resolution, Statut, Versions,
};
use crate::fiche::{Fiche, Lien};
use crate::identifiants::{DecisionId, FicheId, SourceId};
use crate::referentiel::Referentiel;
use crate::reglages::{Autorite, Conflits, Direction, Reglages, Source, Suppressions};
use crate::relations::TypeRelation;
use crate::temps::Horodatage;
use crate::vcard::Version;

/// Version du schéma que ce code écrit et sait lire.
const VERSION_SCHEMA: i64 = 3;

/// Migrations du schéma : chacune mène de la version précédente à la sienne.
const MIGRATIONS: [(i64, &str); 3] = [(1, SCHEMA_1), (2, SCHEMA_2), (3, SCHEMA_3)];

/// Schéma d'origine.
const SCHEMA_1: &str = "
CREATE TABLE sources (
    id           TEXT PRIMARY KEY,
    nom          TEXT NOT NULL,
    version      TEXT NOT NULL CHECK (version IN ('3.0', '4.0')),
    direction    TEXT NOT NULL,
    autorite     TEXT NOT NULL,
    suppressions TEXT NOT NULL,
    conflits     TEXT NOT NULL
) STRICT;
CREATE TABLE autorite_par_champ (
    champ     TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES sources (id) ON DELETE CASCADE
) STRICT;
CREATE TABLE fiches (
    id TEXT PRIMARY KEY
) STRICT;
CREATE TABLE valeurs (
    fiche_id TEXT    NOT NULL REFERENCES fiches (id) ON DELETE CASCADE,
    champ    TEXT    NOT NULL,
    rang     INTEGER NOT NULL,
    texte    TEXT    NOT NULL,
    types    TEXT    NOT NULL,
    origine  TEXT,
    date     INTEGER NOT NULL,
    PRIMARY KEY (fiche_id, champ, rang)
) STRICT;
CREATE TABLE liens (
    fiche_id    TEXT    NOT NULL REFERENCES fiches (id) ON DELETE CASCADE,
    source_id   TEXT    NOT NULL REFERENCES sources (id),
    uid         TEXT    NOT NULL,
    etag        TEXT,
    carte       TEXT,
    a_restaurer INTEGER NOT NULL CHECK (a_restaurer IN (0, 1)),
    PRIMARY KEY (fiche_id, source_id),
    UNIQUE (source_id, uid)
) STRICT;
CREATE TABLE bases (
    fiche_id  TEXT    NOT NULL,
    source_id TEXT    NOT NULL,
    champ     TEXT    NOT NULL,
    rang      INTEGER NOT NULL,
    texte     TEXT    NOT NULL,
    PRIMARY KEY (fiche_id, source_id, champ, rang),
    FOREIGN KEY (fiche_id, source_id) REFERENCES liens (fiche_id, source_id) ON DELETE CASCADE
) STRICT;
CREATE TABLE decisions (
    id          TEXT    PRIMARY KEY,
    rang        INTEGER NOT NULL UNIQUE,
    nature      TEXT    NOT NULL,
    champ       TEXT,
    autre_fiche TEXT,
    fiche_id    TEXT    NOT NULL,
    source_id   TEXT    NOT NULL,
    creee_le    INTEGER NOT NULL,
    statut      TEXT    NOT NULL,
    resolution  TEXT
) STRICT;
CREATE TABLE decision_textes (
    decision_id TEXT    NOT NULL REFERENCES decisions (id) ON DELETE CASCADE,
    role        TEXT    NOT NULL CHECK (role IN ('referentiel', 'source', 'base', 'saisie')),
    rang        INTEGER NOT NULL,
    texte       TEXT    NOT NULL,
    PRIMARY KEY (decision_id, role, rang)
) STRICT;
CREATE TABLE a_supprimer (
    source_id TEXT NOT NULL,
    uid       TEXT NOT NULL,
    PRIMARY KEY (source_id, uid)
) STRICT;
CREATE TABLE ignorees (
    source_id TEXT NOT NULL,
    uid       TEXT NOT NULL,
    PRIMARY KEY (source_id, uid)
) STRICT;
";

/// Version 2 : relations (lot 4) — mémoire des relations par lien, fiche
/// « moi », inverses déclarés des types de relation.
const SCHEMA_2: &str = "
CREATE TABLE memoires (
    fiche_id  TEXT NOT NULL,
    source_id TEXT NOT NULL,
    rendu     TEXT NOT NULL,
    texte     TEXT NOT NULL,
    PRIMARY KEY (fiche_id, source_id, rendu),
    FOREIGN KEY (fiche_id, source_id) REFERENCES liens (fiche_id, source_id) ON DELETE CASCADE
) STRICT;
CREATE TABLE moi (
    seule    INTEGER PRIMARY KEY CHECK (seule = 1),
    fiche_id TEXT    NOT NULL REFERENCES fiches (id) ON DELETE CASCADE
) STRICT;
CREATE TABLE inverses (
    type    TEXT PRIMARY KEY,
    inverse TEXT NOT NULL
) STRICT;
";

/// Version 3 : groupes créés dans l'application, par leur chemin (lot 4).
const SCHEMA_3: &str = "
CREATE TABLE groupes (
    chemin TEXT PRIMARY KEY
) STRICT;
";

/// Erreur de stockage.
#[derive(Debug, thiserror::Error)]
pub enum ErreurStockage {
    /// Erreur rendue par SQLite.
    #[error("base SQLite : {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// La base a été écrite par une version plus récente du schéma.
    #[error("schéma de version {0} inconnu de cette version (au plus {VERSION_SCHEMA})")]
    SchemaInconnu(i64),
    /// Une valeur relue ne se décode pas : base altérée.
    #[error("valeur illisible dans la colonne {colonne} : « {valeur} »")]
    ValeurIllisible {
        /// Table et colonne.
        colonne: &'static str,
        /// Valeur relue.
        valeur: String,
    },
}

type Resultat<T> = Result<T, ErreurStockage>;

/// Base SQLite du référentiel.
#[derive(Debug)]
pub struct Stockage {
    connexion: Connection,
}

impl Stockage {
    /// Ouvre la base, la crée au besoin et met son schéma à jour.
    ///
    /// # Errors
    ///
    /// [`ErreurStockage`] si la base ne s'ouvre pas ou porte un schéma plus
    /// récent que ce code.
    pub fn ouvrir(chemin: &Path) -> Resultat<Self> {
        Self::preparer(Connection::open(chemin)?)
    }

    /// Base en mémoire, pour les essais.
    ///
    /// # Errors
    ///
    /// [`ErreurStockage::Sqlite`] si SQLite ne l'ouvre pas.
    pub fn en_memoire() -> Resultat<Self> {
        Self::preparer(Connection::open_in_memory()?)
    }

    fn preparer(mut connexion: Connection) -> Resultat<Self> {
        connexion.pragma_update(None, "foreign_keys", true)?;
        connexion.pragma_update(None, "journal_mode", "WAL")?;
        connexion.busy_timeout(std::time::Duration::from_secs(5))?;
        let version: i64 = connexion.pragma_query_value(None, "user_version", |l| l.get(0))?;
        if version > VERSION_SCHEMA {
            return Err(ErreurStockage::SchemaInconnu(version));
        }
        if version < VERSION_SCHEMA {
            // Toutes les migrations dues en une transaction : la base passe
            // à la version courante, ou reste où elle était.
            let transaction =
                connexion.transaction_with_behavior(TransactionBehavior::Immediate)?;
            for (cible, schema) in MIGRATIONS {
                if cible > version {
                    transaction.execute_batch(schema)?;
                }
            }
            transaction.pragma_update(None, "user_version", VERSION_SCHEMA)?;
            transaction.commit()?;
        }
        Ok(Self { connexion })
    }

    /// Enregistre le référentiel entier, à la place du précédent, en une
    /// seule transaction.
    ///
    /// # Errors
    ///
    /// [`ErreurStockage::Sqlite`] ; la base garde alors l'état précédent.
    // [impl->req~fiche-maitre-identifiant-stable~1]
    // [impl->req~provenance-par-champ~1]
    // [impl->req~conservation-proprietes-inconnues~1]
    pub fn enregistrer(&mut self, referentiel: &Referentiel) -> Resultat<()> {
        let transaction = self
            .connexion
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        vider(&transaction)?;
        ecrire_sources(&transaction, referentiel)?;
        ecrire_fiches(&transaction, referentiel)?;
        ecrire_decisions(&transaction, referentiel)?;
        ecrire_copies(&transaction, referentiel)?;
        transaction.commit()?;
        Ok(())
    }

    /// Charge le référentiel enregistré ; un référentiel vide si rien ne
    /// l'a encore été.
    ///
    /// # Errors
    ///
    /// [`ErreurStockage`] si la lecture échoue ou si une valeur ne se
    /// décode pas.
    pub fn charger(&self) -> Resultat<Referentiel> {
        let mut referentiel = Referentiel::new();
        lire_sources(&self.connexion, &mut referentiel)?;
        lire_fiches(&self.connexion, &mut referentiel)?;
        lire_decisions(&self.connexion, &mut referentiel)?;
        lire_copies(&self.connexion, &mut referentiel)?;
        Ok(referentiel)
    }
}

// ---------------------------------------------------------------- écriture

fn vider(transaction: &Transaction<'_>) -> Resultat<()> {
    transaction.execute_batch(
        "DELETE FROM groupes;
         DELETE FROM inverses;
         DELETE FROM moi;
         DELETE FROM memoires;
         DELETE FROM ignorees;
         DELETE FROM a_supprimer;
         DELETE FROM decision_textes;
         DELETE FROM decisions;
         DELETE FROM bases;
         DELETE FROM liens;
         DELETE FROM valeurs;
         DELETE FROM fiches;
         DELETE FROM autorite_par_champ;
         DELETE FROM sources;",
    )?;
    Ok(())
}

fn ecrire_sources(transaction: &Transaction<'_>, referentiel: &Referentiel) -> Resultat<()> {
    let mut source = transaction.prepare_cached(
        "INSERT INTO sources (id, nom, version, direction, autorite, suppressions, conflits)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for s in referentiel.sources().toutes() {
        let r = s.reglages;
        source.execute(params![
            s.id.to_string(),
            s.nom,
            s.version.texte(),
            code_direction(r.direction),
            code_autorite(r.autorite),
            code_suppressions(r.suppressions),
            code_conflits(r.conflits),
        ])?;
    }
    let mut designation = transaction
        .prepare_cached("INSERT INTO autorite_par_champ (champ, source_id) VALUES (?1, ?2)")?;
    for (champ, id) in referentiel.sources().designations() {
        designation.execute(params![champ.propriete_vcard(), id.to_string()])?;
    }
    Ok(())
}

fn ecrire_fiches(transaction: &Transaction<'_>, referentiel: &Referentiel) -> Resultat<()> {
    let mut fiche = transaction.prepare_cached("INSERT INTO fiches (id) VALUES (?1)")?;
    let mut valeur = transaction.prepare_cached(
        "INSERT INTO valeurs (fiche_id, champ, rang, texte, types, origine, date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    let mut lien = transaction.prepare_cached(
        "INSERT INTO liens (fiche_id, source_id, uid, etag, carte, a_restaurer)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    let mut base = transaction.prepare_cached(
        "INSERT INTO bases (fiche_id, source_id, champ, rang, texte) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut memoire = transaction.prepare_cached(
        "INSERT INTO memoires (fiche_id, source_id, rendu, texte) VALUES (?1, ?2, ?3, ?4)",
    )?;
    for f in referentiel.fiches() {
        let id = f.id().to_string();
        fiche.execute(params![id])?;
        for (champ, valeurs) in f.champs() {
            for (rang, v) in valeurs.iter().enumerate() {
                let origine = match v.provenance.origine {
                    Origine::Referentiel => None,
                    Origine::Source(source) => Some(source.to_string()),
                };
                valeur.execute(params![
                    id,
                    champ.propriete_vcard(),
                    rang_sql(rang),
                    v.texte,
                    v.types.join(","),
                    origine,
                    v.provenance.date.millis(),
                ])?;
            }
        }
        for l in f.liens() {
            let source = l.source.to_string();
            lien.execute(params![id, source, l.uid, l.etag, l.carte, l.a_restaurer])?;
            for (champ, textes) in &l.base {
                for (rang, texte) in textes.iter().enumerate() {
                    base.execute(params![
                        id,
                        source,
                        champ.propriete_vcard(),
                        rang_sql(rang),
                        texte
                    ])?;
                }
            }
            for (rendu, texte) in &l.memoire {
                memoire.execute(params![id, source, rendu, texte])?;
            }
        }
    }
    if let Some(moi) = referentiel.moi() {
        transaction.execute(
            "INSERT INTO moi (seule, fiche_id) VALUES (1, ?1)",
            params![moi.to_string()],
        )?;
    }
    let mut inverse =
        transaction.prepare_cached("INSERT INTO inverses (type, inverse) VALUES (?1, ?2)")?;
    for (a, b) in referentiel.inverses().couples() {
        inverse.execute(params![a, b])?;
    }
    let mut groupe = transaction.prepare_cached("INSERT INTO groupes (chemin) VALUES (?1)")?;
    for chemin in &referentiel.groupes {
        groupe.execute(params![chemin])?;
    }
    Ok(())
}

fn ecrire_decisions(transaction: &Transaction<'_>, referentiel: &Referentiel) -> Resultat<()> {
    let mut decision = transaction.prepare_cached(
        "INSERT INTO decisions
            (id, rang, nature, champ, autre_fiche, fiche_id, source_id, creee_le, statut, resolution)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?;
    let mut texte = transaction.prepare_cached(
        "INSERT INTO decision_textes (decision_id, role, rang, texte) VALUES (?1, ?2, ?3, ?4)",
    )?;
    for (rang, d) in referentiel.decisions().iter().enumerate() {
        let id = d.id.to_string();
        let (nature, champ, autre) = match d.nature {
            Nature::Divergence { champ } => ("divergence", Some(champ.propriete_vcard()), None),
            Nature::Suppression => ("suppression", None, None),
            Nature::DoublonProbable { autre } => ("doublon", None, Some(autre.to_string())),
            Nature::NouvellePhoto => ("photo", None, None),
        };
        decision.execute(params![
            id,
            rang_sql(rang),
            nature,
            champ,
            autre,
            d.fiche.to_string(),
            d.source.to_string(),
            d.creee_le.millis(),
            code_statut(d.statut),
            d.resolution.as_ref().map(code_resolution),
        ])?;
        let saisie = match &d.resolution {
            Some(Resolution::Divergence(Action::Saisir(textes))) => textes.as_slice(),
            _ => &[],
        };
        let roles = [
            ("referentiel", d.versions.referentiel.as_slice()),
            ("source", d.versions.source.as_slice()),
            ("base", d.versions.base.as_slice()),
            ("saisie", saisie),
        ];
        for (role, textes) in roles {
            for (rang, t) in textes.iter().enumerate() {
                texte.execute(params![id, role, rang_sql(rang), t])?;
            }
        }
    }
    Ok(())
}

fn ecrire_copies(transaction: &Transaction<'_>, referentiel: &Referentiel) -> Resultat<()> {
    let mut a_supprimer =
        transaction.prepare_cached("INSERT INTO a_supprimer (source_id, uid) VALUES (?1, ?2)")?;
    for (source, uid) in &referentiel.a_supprimer {
        a_supprimer.execute(params![source.to_string(), uid])?;
    }
    let mut ignoree =
        transaction.prepare_cached("INSERT INTO ignorees (source_id, uid) VALUES (?1, ?2)")?;
    for (source, uid) in &referentiel.ignorees {
        ignoree.execute(params![source.to_string(), uid])?;
    }
    Ok(())
}

// ---------------------------------------------------------------- lecture

fn lire_sources(connexion: &Connection, referentiel: &mut Referentiel) -> Resultat<()> {
    let mut requete = connexion.prepare(
        "SELECT id, nom, version, direction, autorite, suppressions, conflits FROM sources",
    )?;
    let lignes = requete.query_map([], |l| {
        Ok((
            l.get::<_, String>(0)?,
            l.get::<_, String>(1)?,
            l.get::<_, String>(2)?,
            l.get::<_, String>(3)?,
            l.get::<_, String>(4)?,
            l.get::<_, String>(5)?,
            l.get::<_, String>(6)?,
        ))
    })?;
    for ligne in lignes {
        let (id, nom, version, direction, autorite, suppressions, conflits) = ligne?;
        let source = Source {
            id: identifiant(&id, "sources.id")?,
            nom,
            version: Version::depuis_texte(&version)
                .ok_or_else(|| illisible("sources.version", &version))?,
            reglages: Reglages {
                direction: decoder(&direction, "sources.direction", direction_de)?,
                autorite: decoder(&autorite, "sources.autorite", autorite_de)?,
                suppressions: decoder(&suppressions, "sources.suppressions", suppressions_de)?,
                conflits: decoder(&conflits, "sources.conflits", conflits_de)?,
            },
        };
        referentiel.sources.restaurer(source);
    }
    let mut requete = connexion.prepare("SELECT champ, source_id FROM autorite_par_champ")?;
    let lignes = requete.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?)))?;
    for ligne in lignes {
        let (champ, source) = ligne?;
        referentiel.sources.restaurer_designation(
            champ_de(&champ, "autorite_par_champ.champ")?,
            identifiant(&source, "autorite_par_champ.source_id")?,
        );
    }
    Ok(())
}

fn lire_fiches(connexion: &Connection, referentiel: &mut Referentiel) -> Resultat<()> {
    let mut valeurs: BTreeMap<FicheId, BTreeMap<Champ, Vec<Valeur>>> = BTreeMap::new();
    let mut requete = connexion.prepare("SELECT id FROM fiches")?;
    for id in requete.query_map([], |l| l.get::<_, String>(0))? {
        valeurs.insert(identifiant(&id?, "fiches.id")?, BTreeMap::new());
    }

    let mut requete = connexion.prepare(
        "SELECT fiche_id, champ, texte, types, origine, date FROM valeurs ORDER BY fiche_id, champ, rang",
    )?;
    let lignes = requete.query_map([], |l| {
        Ok((
            l.get::<_, String>(0)?,
            l.get::<_, String>(1)?,
            l.get::<_, String>(2)?,
            l.get::<_, String>(3)?,
            l.get::<_, Option<String>>(4)?,
            l.get::<_, i64>(5)?,
        ))
    })?;
    for ligne in lignes {
        let (fiche, champ, texte, types, origine, date) = ligne?;
        let origine = match origine {
            None => Origine::Referentiel,
            Some(source) => Origine::Source(identifiant(&source, "valeurs.origine")?),
        };
        let provenance = Provenance {
            origine,
            date: Horodatage::depuis_millis(date),
        };
        let valeur =
            Valeur::new(texte, provenance).avec_types(types.split(',').filter(|t| !t.is_empty()));
        valeurs
            .entry(identifiant(&fiche, "valeurs.fiche_id")?)
            .or_default()
            .entry(champ_de(&champ, "valeurs.champ")?)
            .or_default()
            .push(valeur);
    }

    let mut bases: BTreeMap<(FicheId, SourceId), BTreeMap<Champ, Vec<String>>> = BTreeMap::new();
    let mut requete = connexion.prepare(
        "SELECT fiche_id, source_id, champ, texte FROM bases ORDER BY fiche_id, source_id, champ, rang",
    )?;
    let lignes = requete.query_map([], |l| {
        Ok((
            l.get::<_, String>(0)?,
            l.get::<_, String>(1)?,
            l.get::<_, String>(2)?,
            l.get::<_, String>(3)?,
        ))
    })?;
    for ligne in lignes {
        let (fiche, source, champ, texte) = ligne?;
        bases
            .entry((
                identifiant(&fiche, "bases.fiche_id")?,
                identifiant(&source, "bases.source_id")?,
            ))
            .or_default()
            .entry(champ_de(&champ, "bases.champ")?)
            .or_default()
            .push(texte);
    }

    let mut memoires: BTreeMap<(FicheId, SourceId), BTreeMap<String, String>> = BTreeMap::new();
    let mut requete =
        connexion.prepare("SELECT fiche_id, source_id, rendu, texte FROM memoires")?;
    let lignes = requete.query_map([], |l| {
        Ok((
            l.get::<_, String>(0)?,
            l.get::<_, String>(1)?,
            l.get::<_, String>(2)?,
            l.get::<_, String>(3)?,
        ))
    })?;
    for ligne in lignes {
        let (fiche, source, rendu, texte) = ligne?;
        memoires
            .entry((
                identifiant(&fiche, "memoires.fiche_id")?,
                identifiant(&source, "memoires.source_id")?,
            ))
            .or_default()
            .insert(rendu, texte);
    }

    let mut liens: BTreeMap<FicheId, Vec<Lien>> = BTreeMap::new();
    let mut requete = connexion
        .prepare("SELECT fiche_id, source_id, uid, etag, carte, a_restaurer FROM liens")?;
    let lignes = requete.query_map([], |l| {
        Ok((
            l.get::<_, String>(0)?,
            l.get::<_, String>(1)?,
            l.get::<_, String>(2)?,
            l.get::<_, Option<String>>(3)?,
            l.get::<_, Option<String>>(4)?,
            l.get::<_, bool>(5)?,
        ))
    })?;
    for ligne in lignes {
        let (fiche, source, uid, etag, carte, a_restaurer) = ligne?;
        let (fiche, source) = (
            identifiant(&fiche, "liens.fiche_id")?,
            identifiant(&source, "liens.source_id")?,
        );
        let mut lien = Lien::new(source, uid);
        lien.etag = etag;
        lien.carte = carte;
        lien.a_restaurer = a_restaurer;
        lien.base = bases.remove(&(fiche, source)).unwrap_or_default();
        lien.memoire = memoires.remove(&(fiche, source)).unwrap_or_default();
        liens.entry(fiche).or_default().push(lien);
    }

    for (id, champs) in valeurs {
        let fiche = Fiche::restaurer(id, champs, liens.remove(&id).unwrap_or_default());
        referentiel.ajouter(fiche);
    }

    let moi: Option<String> = connexion
        .query_row("SELECT fiche_id FROM moi", [], |l| l.get(0))
        .optional()?;
    if let Some(moi) = moi {
        let moi = identifiant(&moi, "moi.fiche_id")?;
        referentiel
            .designer_moi(Some(moi))
            .map_err(|_| illisible("moi.fiche_id", &moi.to_string()))?;
    }
    let mut requete = connexion.prepare("SELECT type, inverse FROM inverses")?;
    let lignes = requete.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?)))?;
    for ligne in lignes {
        let (a, b) = ligne?;
        referentiel
            .inverses_mut()
            .declarer(
                &TypeRelation::depuis_texte(&a),
                &TypeRelation::depuis_texte(&b),
            )
            .map_err(|_| illisible("inverses.type", &a))?;
    }
    let mut requete = connexion.prepare("SELECT chemin FROM groupes")?;
    for chemin in requete.query_map([], |l| l.get::<_, String>(0))? {
        referentiel.groupes.insert(chemin?);
    }
    Ok(())
}

fn lire_decisions(connexion: &Connection, referentiel: &mut Referentiel) -> Resultat<()> {
    let mut textes: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    let mut requete = connexion.prepare(
        "SELECT decision_id, role, texte FROM decision_textes ORDER BY decision_id, role, rang",
    )?;
    let lignes = requete.query_map([], |l| {
        Ok((
            l.get::<_, String>(0)?,
            l.get::<_, String>(1)?,
            l.get::<_, String>(2)?,
        ))
    })?;
    for ligne in lignes {
        let (id, role, texte) = ligne?;
        textes.entry((id, role)).or_default().push(texte);
    }
    let mut prendre = |id: &str, role: &str| {
        textes
            .remove(&(id.to_owned(), role.to_owned()))
            .unwrap_or_default()
    };

    let mut requete = connexion.prepare(
        "SELECT id, nature, champ, autre_fiche, fiche_id, source_id, creee_le, statut, resolution
         FROM decisions ORDER BY rang",
    )?;
    let lignes = requete.query_map([], |l| {
        Ok((
            l.get::<_, String>(0)?,
            l.get::<_, String>(1)?,
            l.get::<_, Option<String>>(2)?,
            l.get::<_, Option<String>>(3)?,
            l.get::<_, String>(4)?,
            l.get::<_, String>(5)?,
            l.get::<_, i64>(6)?,
            l.get::<_, String>(7)?,
            l.get::<_, Option<String>>(8)?,
        ))
    })?;
    for ligne in lignes {
        let (id, nature, champ, autre, fiche, source, creee_le, statut, resolution) = ligne?;
        let nature = match (nature.as_str(), champ, autre) {
            ("divergence", Some(champ), _) => Nature::Divergence {
                champ: champ_de(&champ, "decisions.champ")?,
            },
            ("suppression", _, _) => Nature::Suppression,
            ("doublon", _, Some(autre)) => Nature::DoublonProbable {
                autre: identifiant(&autre, "decisions.autre_fiche")?,
            },
            ("photo", _, _) => Nature::NouvellePhoto,
            _ => return Err(illisible("decisions.nature", &nature)),
        };
        let resolution = resolution
            .map(|code| resolution_de(&code, prendre(&id, "saisie")))
            .transpose()?;
        referentiel.decisions.push(Decision {
            id: identifiant::<DecisionId>(&id, "decisions.id")?,
            nature,
            fiche: identifiant(&fiche, "decisions.fiche_id")?,
            source: identifiant(&source, "decisions.source_id")?,
            versions: Versions {
                referentiel: prendre(&id, "referentiel"),
                source: prendre(&id, "source"),
                base: prendre(&id, "base"),
            },
            creee_le: Horodatage::depuis_millis(creee_le),
            statut: decoder(&statut, "decisions.statut", statut_de)?,
            resolution,
        });
    }
    Ok(())
}

fn lire_copies(connexion: &Connection, referentiel: &mut Referentiel) -> Resultat<()> {
    for (table, ensemble) in [
        ("a_supprimer", &mut referentiel.a_supprimer),
        ("ignorees", &mut referentiel.ignorees),
    ] {
        let mut requete = connexion.prepare(&format!("SELECT source_id, uid FROM {table}"))?;
        let lignes =
            requete.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?)))?;
        for ligne in lignes {
            let (source, uid) = ligne?;
            ensemble.insert((identifiant(&source, "copies.source_id")?, uid));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- codes

/// Rang d'une valeur dans sa liste, en entier SQLite.
fn rang_sql(rang: usize) -> i64 {
    i64::try_from(rang).unwrap_or(i64::MAX)
}

fn illisible(colonne: &'static str, valeur: &str) -> ErreurStockage {
    ErreurStockage::ValeurIllisible {
        colonne,
        valeur: valeur.to_owned(),
    }
}

fn identifiant<T: std::str::FromStr>(texte: &str, colonne: &'static str) -> Resultat<T> {
    texte.parse().map_err(|_| illisible(colonne, texte))
}

fn decoder<T>(texte: &str, colonne: &'static str, lire: fn(&str) -> Option<T>) -> Resultat<T> {
    lire(texte).ok_or_else(|| illisible(colonne, texte))
}

fn champ_de(texte: &str, colonne: &'static str) -> Resultat<Champ> {
    Champ::depuis_propriete_vcard(texte).ok_or_else(|| illisible(colonne, texte))
}

const fn code_direction(d: Direction) -> &'static str {
    match d {
        Direction::Bidirectionnelle => "bidirectionnelle",
        Direction::LectureSeule => "lecture_seule",
        Direction::EcritureSeule => "ecriture_seule",
    }
}

fn direction_de(t: &str) -> Option<Direction> {
    [
        Direction::Bidirectionnelle,
        Direction::LectureSeule,
        Direction::EcritureSeule,
    ]
    .into_iter()
    .find(|d| code_direction(*d) == t)
}

const fn code_autorite(a: Autorite) -> &'static str {
    match a {
        Autorite::Referentiel => "referentiel",
        Autorite::CetteSource => "cette_source",
    }
}

fn autorite_de(t: &str) -> Option<Autorite> {
    [Autorite::Referentiel, Autorite::CetteSource]
        .into_iter()
        .find(|a| code_autorite(*a) == t)
}

const fn code_suppressions(s: Suppressions) -> &'static str {
    match s {
        Suppressions::Demander => "demander",
        Suppressions::Propager => "propager",
        Suppressions::Ignorer => "ignorer",
    }
}

fn suppressions_de(t: &str) -> Option<Suppressions> {
    [
        Suppressions::Demander,
        Suppressions::Propager,
        Suppressions::Ignorer,
    ]
    .into_iter()
    .find(|s| code_suppressions(*s) == t)
}

const fn code_conflits(c: Conflits) -> &'static str {
    match c {
        Conflits::Demander => "demander",
        Conflits::AutoriteGagne => "autorite_gagne",
    }
}

fn conflits_de(t: &str) -> Option<Conflits> {
    [Conflits::Demander, Conflits::AutoriteGagne]
        .into_iter()
        .find(|c| code_conflits(*c) == t)
}

const fn code_statut(s: Statut) -> &'static str {
    match s {
        Statut::EnAttente => "en_attente",
        Statut::Reportee => "reportee",
        Statut::Resolue => "resolue",
    }
}

fn statut_de(t: &str) -> Option<Statut> {
    [Statut::EnAttente, Statut::Reportee, Statut::Resolue]
        .into_iter()
        .find(|s| code_statut(*s) == t)
}

fn code_resolution(r: &Resolution) -> &'static str {
    match r {
        Resolution::Divergence(Action::GarderReferentiel) => "garder_referentiel",
        Resolution::Divergence(Action::GarderSource) => "garder_source",
        Resolution::Divergence(Action::GarderLesDeux) => "garder_les_deux",
        Resolution::Divergence(Action::Saisir(_)) => "saisir",
        Resolution::Suppression(IssueSuppression::Propager) => "propager",
        Resolution::Suppression(IssueSuppression::Restaurer) => "restaurer",
        Resolution::Suppression(IssueSuppression::Detacher) => "detacher",
        Resolution::Doublon(IssueDoublon::Fusionner) => "fusionner",
        Resolution::Doublon(IssueDoublon::Distinguer) => "distinguer",
    }
}

fn resolution_de(code: &str, saisie: Vec<String>) -> Resultat<Resolution> {
    Ok(match code {
        "garder_referentiel" => Resolution::Divergence(Action::GarderReferentiel),
        "garder_source" => Resolution::Divergence(Action::GarderSource),
        "garder_les_deux" => Resolution::Divergence(Action::GarderLesDeux),
        "saisir" => Resolution::Divergence(Action::Saisir(saisie)),
        "propager" => Resolution::Suppression(IssueSuppression::Propager),
        "restaurer" => Resolution::Suppression(IssueSuppression::Restaurer),
        "detacher" => Resolution::Suppression(IssueSuppression::Detacher),
        "fusionner" => Resolution::Doublon(IssueDoublon::Fusionner),
        "distinguer" => Resolution::Doublon(IssueDoublon::Distinguer),
        _ => return Err(illisible("decisions.resolution", code)),
    })
}

#[cfg(test)]
mod tests;

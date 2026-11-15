# Journal des changements

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) —
versionnement : [SemVer](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté

- Spécification v0.1 découpée dans la chaîne tracée : 9 besoins, 9 domaines
  d'exigences en proposition (59 exigences proposées), 7 questions ouvertes,
  ADR-0003 à ADR-0005 (architecture, proposés). Le document d'origine est
  conservé dans `spec/01-besoins/sources/`.
- Arbitrage du 2026-11-14 (D-2026-11-07 à D-2026-11-09) : OQ-001 à OQ-006
  tranchées (OQ-007, le nom, reste ouverte) ; cinq exigences révisées et deux
  ajoutées (autorité par champ, dates reportées dans l'agenda) ; ADR-0006
  (Kotlin / Compose et UniFFI) ajouté. Approbation lot par lot, selon la
  méthode de greffier : lot 1 ouvert, domaines referentiel, synchronisation
  et file-decisions ACTIVE, 23 exigences approuvées, ADR-0003 et ADR-0005
  acceptés ; le reste demeure proposé jusqu'à son lot.
- ADR-0002 : just comme façade ; `AGENTS.md` et `CONTRIBUTING.md` citent ses
  recettes.
- Lot 1, cœur Rust (`crates/core`, ToDo-0002) : fiche maître à identifiant
  stable et provenance par valeur ; lecture et écriture vCard 3.0 et 4.0
  sans perte des propriétés non interprétées ; fusion à trois voies champ
  par champ, autorité réglée par source et par champ ; file de décisions
  (divergences, suppressions, doublons probables) ; détection des doublons à
  l'import (D-2026-11-10) ; stockage SQLite. Les 23 exigences du lot sont
  réalisées, testées et jugées conformes.
- Lot 2, connecteur CardDAV (`crates/carddav`, ToDo-0003, ADR-0007) :
  synchronisation bidirectionnelle avec un carnet Murena ou Nextcloud,
  écritures sous précondition `If-Match`, écriture refusée annulée et
  reprise au tour suivant ; tests d'intégration contre Radicale 3.9.1
  épinglé (`sh tools/radicale.sh test`, ADR-0008).
- Lot 3, application Android (`android/`, `crates/ffi`, ToDo-0004,
  ADR-0004, ADR-0009, ADR-0010) : compte système et adaptateur de
  synchronisation, un contact par fiche sous le seul compte d'unicontacts,
  écrans des fiches, des sources et des décisions avec vue comparée ;
  mots de passe chiffrés par le Keystore. Vérification sur l'appareil à
  exécuter.
- Lot 4, relations (ToDo-0005, D-2026-11-13, ADR-0011) : fiche « moi »,
  liens typés entre fiches (prédéfinis ou personnalisés) avec réciproques,
  section « Proches » en tête de liste ; relations portées par `RELATED` en
  vCard 4.0, par la convention Apple en 3.0, et par le champ « Relation »
  d'Android. Schéma SQLite en version 2, migré à l'ouverture.
- Lot 4, groupes (ToDo-0005, ADR-0012) : une fiche dans plusieurs groupes
  hiérarchiques, désignés par leur chemin ; filtre de la liste par groupes
  (union ou intersection, sous-groupes compris) combiné à une recherche ;
  groupes portés par `CATEGORIES` ; renommage ou déplacement d'un groupe en
  une seule opération, sans décision. Écran de gestion des groupes. Schéma
  SQLite en version 3.
- Lot 4, dates (ToDo-0005, ADR-0011, ADR-0013) : naissance, anniversaire de
  mariage, fête du prénom et dates personnalisées, année facultative ; fête
  proposée d'après le calendrier des fêtes de Wikidata (CC0, vérifié par
  empreinte, `sh tools/fetes.sh`) ; vue « À venir » filtrable par groupe ;
  dates portées par `BDAY`, `ANNIVERSARY` et `X-ABDATE`, et par le champ
  « Événement » d'Android ; report en événements annuels dans un agenda du
  téléphone choisi, qui émet les rappels.

## [0.1.0] - 2026-11-14

### Ajouté

- Instanciation de unicontacts depuis le gabarit alm-forge.

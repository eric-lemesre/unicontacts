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

## [0.1.0] - 2026-11-14

### Ajouté

- Instanciation de unicontacts depuis le gabarit alm-forge.

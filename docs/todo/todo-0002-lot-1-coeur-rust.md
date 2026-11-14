# ToDo-0002 — Lot 1 : cœur Rust

**Statut : 🚧 en cours**

Modèle, stockage SQLite, vCard 3.0 et 4.0, fusion à trois voies et file de
décisions, testés sur des jeux de fichiers vCard (spécification v0.1, § 13).
C'est la partie la plus délicate du produit, et la seule testable entièrement
hors du téléphone. Lot ouvert le 2026-11-14 : ses 23 exigences sont
approuvées (D-2026-11-09).

## Reste à faire

- Constituer les jeux de vCard de référence (3.0, 4.0, propriétés inconnues).
- Réaliser et tester les exigences approuvées des domaines référentiel,
  synchronisation (hors sources) et file de décisions (hors écrans).
- Porter le verdict d'audit de chacune à la livraison.
- Retirer `crates/core/src/lib.rs` de `tools/rattachement-herites.txt` dès
  qu'il porte la trace d'une exigence.

## Renvois

- Exigences (approuvées, D-2026-11-09) :
  - `req~fiche-maitre-identifiant-stable~1`
  - `req~provenance-par-champ~1`
  - `req~conservation-proprietes-inconnues~1`
  - `req~vcard-3-et-4~1`
  - `req~coeur-independant-android~1`
  - `req~reglages-par-source~1`
  - `req~autorite-par-champ~1`
  - `req~source-de-reference-unique~2`
  - `req~autorite-de-la-reference~2`
  - `req~fusion-trois-voies~1`
  - `req~modification-unilaterale-appliquee~1`
  - `req~divergence-soumise-a-decision~1`
  - `req~multivalues-par-valeur-normalisee~1`
  - `req~synchronisation-non-bloquante~1`
  - `req~suppression-sans-destruction~1`
  - `req~detection-doublons~1`
  - `req~traitement-des-doublons~1`
  - `req~contenu-decision~1`
  - `req~actions-de-resolution~1`
  - `req~report-decision~1`
  - `req~resolution-par-lot~1`
  - `req~issues-suppression-detectee~1`
  - `req~portee-suppression-locale~1`
- Décisions : ADR-0003, ADR-0005

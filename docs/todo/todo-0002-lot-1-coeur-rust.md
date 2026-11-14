# ToDo-0002 — Lot 1 : cœur Rust

**Statut : ⏳ à venir**

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

- Exigences : approuvées des domaines `referentiel`, `synchronisation`,
  `file-decisions`
- Décisions : ADR-0003, ADR-0005

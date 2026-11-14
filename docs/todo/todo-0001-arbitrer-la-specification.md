# ToDo-0001 — Arbitrer la spécification v0.1

**Statut : 🚧 en cours**

La spécification v0.1 est découpée en besoins, exigences, ADR et questions
ouvertes, mais rien n'est encore opposable : les neuf domaines sont en
proposition. Tant qu'un domaine n'est pas actif, aucune de ses exigences ne
peut être réalisée (la porte refuse un tag de code vers une exigence non
approuvée).

## Reste à faire

- Trancher OQ-001 à OQ-007 (`spec/open-questions.md`) ; OQ-001 conditionne
  la couche Android.
- Relire les 59 exigences proposées, puis acter au registre le passage en
  ACTIVE de chaque domaine retenu et approuver ses exigences — en commençant
  par référentiel, synchronisation et file de décisions (lot 1).
- Accepter ou rejeter ADR-0003, ADR-0004 et ADR-0005.
- Retirer `crates/core/src/lib.rs` de `tools/rattachement-herites.txt` dès
  qu'il porte la trace d'une exigence approuvée.

## Renvois

- Exigences : `spec/02-exigences/` (tous domaines)
- Décisions : ADR-0003, ADR-0004, ADR-0005
- Questions : OQ-001 à OQ-007

# ToDo-0003 — Lot 2 : connecteur CardDAV

**Statut : 🚧 en cours**

Premier connecteur de source, vers Murena et Nextcloud (spécification v0.1,
§ 5.1 et 13). Il éprouve la fusion à trois voies contre un vrai serveur.

## Reste à faire

- Lot ouvert le 2026-11-14 (D-2026-11-11) : `req~source-carddav~1`
  approuvée, ADR-0007 et ADR-0008 acceptés.
- Brancher le connecteur sur le moteur : il fournit les copies
  (`CopieDistante`), applique les écritures du `Rapport`, puis rend les
  nouveaux ETag (`Referentiel::confirmer_ecriture`).
- Concevoir la synchronisation incrémentale (sync-token) : le moteur attend
  aujourd'hui la liste complète des copies d'une source.
- Câbler Radicale (`tools/radicale.sh`, module de bootstrap, CI) pour les
  tests d'intégration (ADR-0008).

## Renvois

- Exigences : `req~source-carddav~1` (approuvée, D-2026-11-11)
- Décisions : ADR-0007, ADR-0008 ; ADR-0004 (proposé, lot 3)

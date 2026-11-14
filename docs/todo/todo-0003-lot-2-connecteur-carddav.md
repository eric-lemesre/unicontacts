# ToDo-0003 — Lot 2 : connecteur CardDAV

**Statut : ✅ soldé**

Premier connecteur de source, vers Murena et Nextcloud (spécification v0.1,
§ 5.1 et 13). Il éprouve la fusion à trois voies contre un vrai serveur.

## Reste à faire

- Rien : lot livré le 2026-11-14. Le crate `crates/carddav` réalise
  `req~source-carddav~1`, éprouvée contre Radicale (`sh tools/radicale.sh
  test`) et jugée conforme à l'audit.
- Limites assumées, à reprendre si l'usage l'exige :
  - chaque synchronisation relit le carnet entier ; la synchronisation
    incrémentale (sync-token, RFC 6578) n'est pas faite ;
  - l'URL du carnet se donne telle quelle, sans découverte
    (`.well-known/carddav`) ;
  - la vérification des certificats par le magasin du système Android
    revient au lot 3 (ADR-0007).

## Renvois

- Exigences : `req~source-carddav~1` (approuvée, D-2026-11-11)
- Décisions : ADR-0007, ADR-0008 ; ADR-0004 (proposé, lot 3)

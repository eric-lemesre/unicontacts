# ToDo-0003 — Lot 2 : connecteur CardDAV

**Statut : ⏳ à venir**

Premier connecteur de source, vers Murena et Nextcloud (spécification v0.1,
§ 5.1 et 13). Il éprouve la fusion à trois voies contre un vrai serveur.

## Reste à faire

- Tout : le cœur du lot 1 est livré (ToDo-0002).
- Brancher le connecteur sur le moteur : il fournit les copies
  (`CopieDistante`), applique les écritures du `Rapport`, puis rend les
  nouveaux ETag (`Referentiel::confirmer_ecriture`).
- Concevoir la synchronisation incrémentale (sync-token) : le moteur attend
  aujourd'hui la liste complète des copies d'une source.
- À l'ouverture : approuver les exigences du lot par une décision au
  registre (domaine passé ACTIVE s'il ne l'est pas), cadrer et accepter ses
  ADR ; reporter « à élever » ou mécaniser ses vérifications manuelles.
- Prévoir un serveur CardDAV de test pour les tests d'intégration.

## Renvois

- Exigences : `req~source-carddav~1` (proposée)
- Décisions : ADR-0004 (proposé)

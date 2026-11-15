# ToDo-0006 — Lot 5 : Google, SIM, import local

**Statut : 🚧 en cours**

Les autres sources, une fois CardDAV éprouvé (spécification v0.1, § 5.1 et
13).

## Reste à faire

- Lot ouvert le 2026-11-15 (D-2026-11-15) : trois pull requests, une par
  source, Google en dernier ; chaque exigence approuvée dans la sienne.
- Stockage local : réalisé — import à la demande comme une source en
  lecture seule, puis retrait des originaux après confirmation (ADR-0015) ;
  reste à vérifier sur le Fairphone.
- SIM : réalisée — source d'import en lecture seule, relue à chaque synchronisation
  par `SimPhonebookContract`, jamais écrite (ADR-0016) ; reste à vérifier
  sur le Fairphone.
- Google : réalisé — CardDAV de Google sous OAuth 2.0 (PKCE, sans services
  Play), jeton de renouvellement chiffré, lecture par `PROPFIND` puis
  `addressbook-multiget` (ADR-0017). Reste à l'utilisateur de créer son
  client OAuth dans la console Google Cloud (procédure dans
  `docs/verification-manuelle/android.md`), puis à vérifier sur le Fairphone
  avec un vrai compte.

## Vérifications manuelles

- `req~import-stockage-local~1` : tests du cœur et JVM, complétés par une
  procédure sur l'appareil (`docs/verification-manuelle/android.md`) pour
  la lecture et le retrait dans le vrai stockage local.
- `req~source-sim-lecture-seule~1` : tests du cœur et JVM, complétés par une
  procédure sur l'appareil pour la lecture de la vraie SIM.
- `req~source-google~1` : tests unitaires (OAuth contre un serveur local,
  vecteur de la RFC 7636), test d'intégration contre Radicale (lecture par
  multiget), procédure sur l'appareil avec un vrai compte Google.

## Renvois

- Exigences approuvées (D-2026-11-15) : `req~import-stockage-local~1`,
  `req~source-sim-lecture-seule~1`, `req~source-google~1`
- Décisions : ADR-0015, ADR-0016, ADR-0017

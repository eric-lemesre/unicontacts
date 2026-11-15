# ToDo-0005 — Lot 4 : relations, groupes, dates, canaux

**Statut : 🚧 en cours**

Les fonctions qui font d'un carnet synchronisé un carnet utile
(spécification v0.1, § 7, 8, 10, 11 et 13).

## Reste à faire

- Lot ouvert le 2026-11-15 (D-2026-11-13) : exigences approuvées par
  l'utilisateur, livrées en quatre pull requests, une par domaine ; ADR-0011
  accepté.
- Relations : réalisées — fiche « moi », liens typés et réciproques,
  section « Proches », portage vCard et Android (ADR-0011), schéma SQLite 2 ;
  la section « Proches » reste à vérifier sur le Fairphone.
- Groupes : réalisés — appartenance multiple, hiérarchie, filtre et
  recherche, `CATEGORIES`, renommage en une opération (ADR-0012), schéma
  SQLite 3.
- Dates : à ouvrir dans leur pull request ; calendrier des fêtes tiré de
  Wikidata (propriété P1750, France, CC0), export versionné avec sa requête
  et son empreinte.
- Canaux : à ouvrir dans leur pull request.

## Vérifications manuelles à élever

- `req~section-proches~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)

## Renvois

- Exigences approuvées (D-2026-11-13) : domaines `relations`, `groupes`
- Exigences à approuver dans leur pull request : domaines `dates`, `canaux`
- Questions : OQ-003 à OQ-006 (tranchées)
- Décisions : ADR-0011, ADR-0012

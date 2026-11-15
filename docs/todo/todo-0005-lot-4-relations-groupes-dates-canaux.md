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
- Dates : réalisées — types et année facultative, fête proposée d'après le
  calendrier de Wikidata (ADR-0013), vue « À venir », portage vCard et
  Android (ADR-0011), report dans un agenda du téléphone ; vue et agenda
  restent à vérifier sur le Fairphone.
- Canaux : réalisés — canaux rattachés aux identifiants de la fiche,
  notes, classements pro et perso, boutons d'action, détection assistée des
  messageries, filtre par canal (ADR-0014), schéma SQLite 4 ; détection et
  boutons restent à vérifier sur le Fairphone.
- Reste du lot : exécuter sur le Fairphone les procédures de
  `docs/verification-manuelle/android.md` (section « Proches », « À venir »,
  agenda, messageries, boutons d'action), puis solder la fiche.

## Vérifications manuelles à élever

- `req~section-proches~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~vue-a-venir~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~dates-dans-l-agenda~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~detection-assistee-canaux~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~actions-triees-par-preference~2` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)

## Renvois

- Exigences approuvées (D-2026-11-13) : domaines `relations`, `groupes`,
  `dates`, `canaux`
- Questions : OQ-003 à OQ-006 (tranchées)
- Décisions : ADR-0011, ADR-0012, ADR-0013, ADR-0014

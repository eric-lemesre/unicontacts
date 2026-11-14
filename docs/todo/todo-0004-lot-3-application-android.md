# ToDo-0004 — Lot 3 : application Android

**Statut : 🚧 en cours**

Compte système, écriture dans `ContactsContract`, écrans de liste, d'édition
et de résolution, en Kotlin / Compose relié au cœur par UniFFI (spécification
v0.1, § 3.2 et 13 ; ADR-0006). C'est ce lot qui rend le référentiel visible
au reste du téléphone.

## Reste à faire

- Lot ouvert le 2026-11-14 (D-2026-11-12) : cinq exigences approuvées,
  ADR-0004, ADR-0009 et ADR-0010 acceptés.
- Crate `ffi` (UniFFI) et projet Gradle ; porte « android » (construction,
  lint, tests JVM) en voie complète et en CI.
- Compte système et adaptateur de synchronisation ; écriture des fiches dans
  `ContactsContract`, un seul contact brut par fiche, en lecture seule.
- Écrans : liste des fiches et compteur de décisions, fiche et édition,
  sources (ajout CardDAV, synchronisation), décisions et vue comparée.
- Identifiants des sources chiffrés par le Keystore (ADR-0010).
- Exécuter une première fois les procédures de
  `docs/verification-manuelle/android.md` sur le Fairphone.

## Vérifications manuelles à élever

- `req~compte-systeme-dedie~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~ecriture-sous-compte-unique~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~synchronisation-directe~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~boite-de-decisions~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)
- `req~vue-comparee~1` (à élever : vérification sur l'appareil faute de tests instrumentés en CI, ADR-0009)

## Renvois

- Exigences (approuvées, D-2026-11-12) : `req~compte-systeme-dedie~1`,
  `req~ecriture-sous-compte-unique~1`, `req~synchronisation-directe~1`,
  `req~boite-de-decisions~1`, `req~vue-comparee~1`
- Décisions : ADR-0004, ADR-0006, ADR-0009, ADR-0010

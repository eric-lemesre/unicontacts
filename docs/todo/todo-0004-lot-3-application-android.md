# ToDo-0004 — Lot 3 : application Android

**Statut : 🚧 en cours**

Compte système, écriture dans `ContactsContract`, écrans de liste, d'édition
et de résolution, en Kotlin / Compose relié au cœur par UniFFI (spécification
v0.1, § 3.2 et 13 ; ADR-0006). C'est ce lot qui rend le référentiel visible
au reste du téléphone.

## Reste à faire

- Exécuter une première fois les procédures de
  `docs/verification-manuelle/android.md` sur le Fairphone, consigner le
  journal des exécutions et porter les verdicts d'audit.
- Fait le 2026-11-14 : crate `ffi` (UniFFI) ; projet Gradle et porte
  « android » (lint aux avertissements bloquants, tests JVM, APK) en voie
  complète et en CI ; compte système et adaptateur de synchronisation ;
  écriture des fiches dans les contacts Android ; écrans (liste et
  compteur, fiche, sources, décisions et vue comparée) ; mots de passe au
  Keystore (ADR-0010).
- Limites assumées : l'application n'a pas encore tourné sur un appareil
  (l'émulateur ne démarre pas dans l'environnement de développement de
  l'agent) ; réglages des sources (direction, autorité, suppressions,
  conflits) non exposés dans les écrans ; icône et nom provisoires
  (OQ-007).

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

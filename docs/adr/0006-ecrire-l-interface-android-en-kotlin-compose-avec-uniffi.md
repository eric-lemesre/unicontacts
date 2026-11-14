# ADR-0006 — Écrire l'interface Android en Kotlin / Compose, reliée au cœur par UniFFI

**Statut** : Accepté
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-07 au registre
**Portée** : durable

## Contexte

Le cœur métier est en Rust, indépendant d'Android (ADR-0003). Le compte
système, le `SyncAdapter`, l'écriture dans `ContactsContract` (ADR-0004) et
les écrans vivent côté Android. Il fallait choisir le langage de cette couche
et le pont vers le cœur (OQ-001).

## Décision

La couche Android est écrite en **Kotlin**, avec **Jetpack Compose** pour les
écrans. Elle appelle le cœur par des liaisons générées par **UniFFI**, depuis
un crate `ffi` du workspace qui expose l'interface publique du cœur ; aucune
logique métier n'est écrite côté Kotlin.

## Alternatives écartées

### Tauri 2

Rust de bout en bout et interface web, mais l'intégration au compte système
et au `SyncAdapter` exige de toute façon des greffons Kotlin : le produit
repose précisément sur cette intégration, que Tauri ne dispense pas d'écrire.

## Conséquences

- Ce que cela apporte : accès natif à `AccountAuthenticator`, `SyncAdapter`,
  `ContactsContract` et `CalendarContract` ; interface conforme aux usages
  Android.
- Ce que cela coûte : deux langages et deux chaînes de construction (Cargo et
  Gradle) ; une frontière UniFFI à tenir stable.
- Ce que cela ferme : une interface partagée avec un client de bureau.

## Vérification

- Revue : le code Kotlin ne contient pas de logique de fusion, de décision ni
  de traitement vCard ; il appelle le crate `ffi`.
- Les tests du cœur restent exécutables sans Android
  (`req~coeur-independant-android~1`).
- Portes Gradle (construction, lint, tests instrumentés) : **non câblées** —
  aucun code Kotlin à ce jour ; à câbler avec le premier module Android
  (lot 3, ToDo-0004), ADR d'outillage à l'appui (R7).

## Références

- OQ-001 (`spec/open-questions.md`), ADR-0003, ADR-0004.
- <https://mozilla.github.io/uniffi-rs/>

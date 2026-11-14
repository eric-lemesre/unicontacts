# ADR-0009 — Construire l'application Android avec Gradle, cargo-ndk et UniFFI

**Statut** : Accepté
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-12 au registre
**Portée** : durable

## Contexte

Le lot 3 donne au référentiel son application Android, en Kotlin / Compose
reliée au cœur par UniFFI (ADR-0006). Il faut une chaîne de construction
reproductible, en local comme en CI, et dire ce que la CI en vérifie. La
cible première est un Fairphone 6 sous /e/OS (Android 15).

## Décision

- **Projet Gradle** dans `android/`, module `app`, wrapper Gradle 9.9.1
  épinglé par l'empreinte SHA-256 de sa distribution ; Android Gradle
  Plugin 9.5.1 (Kotlin intégré), Kotlin 2.4.21 et son greffon Compose,
  Compose BOM 2026.09.00. Les versions vivent dans
  `android/gradle/libs.versions.toml`.
- **SDK** : minSdk 33 (Android 13), targetSdk et compileSdk 37 (exigé par
  les bibliothèques Compose actuelles) ; NDK installé dans le SDK.
- **Pont Rust** : crate `crates/ffi` (UniFFI 0.32.2, macros procédurales),
  compilé par cargo-ndk 4.1.2 pour arm64-v8a (appareil) et x86_64
  (émulateur), en release ; liaisons Kotlin générées par le
  `uniffi-bindgen` du workspace, de même version que la bibliothèque. JNA
  5.19.1 côté Kotlin.
- **Logique dans `tools/`** : `sh tools/android.sh natif` produit
  bibliothèque et liaisons dans `android/app/build/` (jamais versionnées,
  R8) ; Gradle l'appelle avant chaque construction. `sh tools/android.sh
  verifier` enchaîne lint (avertissements bloquants), tests JVM et APK de
  débogage : c'est la porte « android » de la voie complète et de la CI.
- **Code unsafe** : le crate `ffi` est le seul à relâcher `unsafe_code` de
  `forbid` à `deny`, parce que le code généré par UniFFI en contient et le
  déclare. Le code écrit à la main n'en contient aucun.
- **Vérification des exigences Android** : manuelle sur l'appareil (items
  `mtest`, `docs/verification-manuelle/android.md`), faute de tests
  instrumentés en CI ; chacune est reportée « à élever » dans ToDo-0004.

Nouvelles dépendances d'outillage : SDK et NDK Android, cibles Rust
`aarch64-linux-android` et `x86_64-linux-android`, cargo-ndk (module de
bootstrap `tools/bootstrap.d/70-android.sh`).

## Alternatives écartées

### Tests instrumentés sur émulateur en CI

Plus probants pour le compte et `ContactsContract`, mais des jobs longs et
instables ; écartés au lot 3 par décision du porteur, à reprendre pour
élever les vérifications manuelles.

### Liaisons Kotlin versionnées

Pas de génération à la construction, mais un artefact généré dans le dépôt,
qu'il faudrait tenir à jour à chaque changement de l'interface (R8).

## Conséquences

- Ce que cela apporte : une construction reproductible depuis `git clone` ;
  la même bibliothèque Rust, testée sur Linux, embarquée dans l'APK.
- Ce que cela coûte : un SDK et un NDK Android sur le poste et en CI ; une
  construction Android de quelques minutes.
- Ce que cela ferme : rien d'irréversible.

## Vérification

- `sh tools/android.sh verifier` est câblé dans la voie complète de
  `tools/ci.sh` et dans la CI GitHub (R7) ; lint en mode
  `warningsAsErrors`.
- `gradle-wrapper.properties` porte `distributionSha256Sum` : le wrapper
  refuse une distribution dont l'empreinte diffère.
- Revue : `grep -rn unsafe crates/ffi/src` ne trouve rien ; aucun autre
  manifeste ne relâche `unsafe_code`.
- Les exigences Android restent couvertes par leurs procédures manuelles
  tant qu'elles ne sont pas élevées (porte de classification).

## Références

- ADR-0003, ADR-0004, ADR-0006, ADR-0007 ; ToDo-0004.
- <https://mozilla.github.io/uniffi-rs/>, <https://github.com/bbqsrc/cargo-ndk>

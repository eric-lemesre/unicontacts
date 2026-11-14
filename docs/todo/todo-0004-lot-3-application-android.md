# ToDo-0004 — Lot 3 : application Android

**Statut : ⏳ à venir**

Compte système, écriture dans `ContactsContract`, écrans de liste, d'édition
et de résolution, en Kotlin / Compose relié au cœur par UniFFI (spécification
v0.1, § 3.2 et 13 ; ADR-0006). C'est ce lot qui rend le référentiel visible
au reste du téléphone.

## Reste à faire

- Tout : attend les lots 1 et 2.
- À l'ouverture : approuver les exigences du lot par une décision au
  registre (domaine passé ACTIVE s'il ne l'est pas), cadrer et accepter ses
  ADR ; reporter « à élever » ou mécaniser ses vérifications manuelles.
- Créer le crate `ffi` (UniFFI) et le projet Gradle ; câbler leurs portes
  (construction, lint, tests instrumentés) — ADR d'outillage à l'appui.

## Renvois

- Exigences : domaine `android`, écrans du domaine `file-decisions`
  (proposées)
- Décisions : ADR-0004 (proposé), ADR-0006

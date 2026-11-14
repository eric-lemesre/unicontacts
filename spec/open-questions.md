# Questions ouvertes

Registre des blocages en attente d'arbitrage humain. Protocole :
docs/methode/06-questions-ouvertes.md — modèle : templates/fr/open-question.md.

Les questions OQ-001 à OQ-007 reprennent les points ouverts de la
[spécification v0.1](01-besoins/sources/2026-11-14-specification-v0.1.md)
(§ 12). Quand elle donnait une hypothèse par défaut, celle-ci est la
recommandation ; sinon, la recommandation est celle de la rédaction du
2026-11-14 et le dit.

---

## OQ-001 — Technologie de l'interface Android

- **Bloque** : `req~compte-systeme-dedie~1`, `req~ecriture-sous-compte-unique~1`
  et toutes les exigences d'écran ; le crate `ffi` à venir.
- **Contexte** : le cœur est en Rust ; l'interface, le compte système et le
  `SyncAdapter` vivent côté Android. Le choix fixe le langage de l'interface
  et le pont vers le cœur.
- **Options** :
  - A) Kotlin / Jetpack Compose, relié au cœur par UniFFI : accès natif à
    `AccountAuthenticator`, `SyncAdapter` et `ContactsContract` ; deux
    langages à tenir.
  - B) Tauri 2 : interface web et Rust de bout en bout ; l'intégration au
    compte système et au `SyncAdapter` passe par des greffons Android à
    écrire en Kotlin de toute façon.
- **Recommandation** : A (hypothèse par défaut de la spécification) — le
  cœur du produit est l'intégration système d'Android, que B ne dispense pas
  d'écrire en Kotlin.
- **Décision** : A — Kotlin / Jetpack Compose relié au cœur par UniFFI (Eric Lemesre, 2026-11-14, D-2026-11-07) ; acté par ADR-0006.
- **Statut** : resolved

## OQ-002 — Autorité réglée par source ou par champ

- **Bloque** : `req~reglages-par-source~1`, `req~autorite-de-la-reference~1`.
- **Contexte** : une source de référence l'emporte dans les conflits. Faut-il
  pouvoir dire « Google fait foi pour les téléphones, Nextcloud pour les
  adresses » ?
- **Options** :
  - A) Par source seulement, le modèle restant extensible à un réglage par
    champ plus tard.
  - B) Par source et par champ dès la première version : plus souple, plus
    de réglages à expliquer et à tester.
- **Recommandation** : A (hypothèse par défaut de la spécification).
- **Décision** : B — l'autorité se règle par source et par champ dès la première version (Eric Lemesre, 2026-11-14, D-2026-11-08) : `req~autorite-par-champ~1`, `req~source-de-reference-unique~2`, `req~autorite-de-la-reference~2`.
- **Statut** : resolved

## OQ-003 — Opérateur du filtre multi-groupes

- **Bloque** : `req~filtre-par-groupes~1`.
- **Contexte** : filtrer par « Famille » et « Voisins » peut vouloir dire les
  fiches des deux groupes à la fois, ou de l'un ou l'autre.
- **Options** :
  - A) « ou » (union) seulement.
  - B) « et » (intersection) seulement.
  - C) Au choix de l'utilisateur, « ou » par défaut.
- **Recommandation** : C, de la rédaction (la spécification ne tranchait
  pas) — l'union est la lecture la plus courante d'une sélection multiple,
  l'intersection reste accessible.
- **Décision** : C — au choix de l'utilisateur, union par défaut (Eric Lemesre, 2026-11-14, D-2026-11-08) : `req~filtre-par-groupes~2`.
- **Statut** : resolved

## OQ-004 — Rappels des dates : par l'application ou par l'agenda

- **Bloque** : aucune exigence rédigée ; une exigence de rappel suivra la
  décision.
- **Contexte** : la vue « À venir » existe dans tous les cas. Reste à savoir
  qui prévient l'utilisateur.
- **Options** :
  - A) Notifications émises par l'application.
  - B) Alimentation d'un agenda du téléphone (événements récurrents), qui
    gère lui-même les rappels.
  - C) Les deux, au choix.
- **Recommandation** : B, de la rédaction (la spécification ne tranchait
  pas) — l'agenda sait déjà rappeler et se synchronise ; à confirmer selon
  l'agenda utilisé sous /e/OS.
- **Décision** : B — l'application alimente un agenda du téléphone, qui émet les rappels (Eric Lemesre, 2026-11-14, D-2026-11-08) : `req~dates-dans-l-agenda~1`.
- **Statut** : resolved

## OQ-005 — Détermination du contexte pro / perso

- **Bloque** : `req~classements-pro-perso~1`, `req~actions-triees-par-preference~1`.
- **Contexte** : chaque fiche a un classement de canaux pro et un perso ; il
  faut savoir lequel appliquer à un instant donné.
- **Options** :
  - A) Bascule manuelle, globale à l'application.
  - B) Automatique selon l'horaire ou le profil de travail Android.
  - C) Manuelle d'abord, automatique en option plus tard.
- **Recommandation** : A (hypothèse par défaut de la spécification),
  extensible vers C.
- **Décision** : Aucune des options : il n'y a pas de contexte pro / perso global. « Ce n'est pas général, mais un contact peut avoir une préférence de canal pour les communications pro et une pour le perso » (Eric Lemesre, 2026-11-14, D-2026-11-08) : `req~classements-pro-perso~2`, `req~actions-triees-par-preference~2`.
- **Statut** : resolved

## OQ-006 — Groupes hiérarchiques

- **Bloque** : `req~groupes-hierarchiques~1`, `req~groupes-en-categories~1`,
  `req~renommage-groupe-operation-unique~1`.
- **Contexte** : `CATEGORIES` est à plat ; la hiérarchie s'y encode dans le
  libellé (`Travail/Clients/Client X`), que les autres clients afficheront
  tel quel.
- **Options** :
  - A) Groupes hiérarchiques, chemin encodé dans le libellé.
  - B) Groupes à plat.
- **Recommandation** : A (orientation de la spécification, à confirmer).
- **Décision** : A — groupes hiérarchiques, chemin encodé dans `CATEGORIES` (Eric Lemesre, 2026-11-14, D-2026-11-08).
- **Statut** : resolved

## OQ-007 — Nom définitif du projet

- **Bloque** : le nom du compte système, de l'application publiée et du
  dépôt ; aucune exigence.
- **Contexte** : `unicontacts` est un nom de travail.
- **Options** :
  - A) Garder `unicontacts`.
  - B) Choisir un autre nom avant la première publication.
- **Recommandation** : trancher avant toute publication (F-Droid, dépôt
  public) ; renommer ensuite coûte un changement d'identifiant d'application.
- **Décision** : —
- **Statut** : open

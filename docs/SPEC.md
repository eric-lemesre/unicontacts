# Unicontacts — Spécification fonctionnelle et technique (v0.1)

Date : 2026-11-14
Statut : brouillon de conception. Les hypothèses par défaut et les points ouverts sont signalés explicitement.
Nom de travail : `unicontacts` (provisoire).

## 1. Objectif

Android gère les contacts **par source** (compte Google, CardDAV, SIM, stockage local…) et les agrège à l'affichage. Une modification ne touche qu'une source, et les fusions automatiques sont fragiles.

Unicontacts inverse cette logique :

- un **référentiel maître unique** contient les contacts, gérés comme un seul bloc ;
- les sources ne sont que des **cibles de synchronisation** ;
- toute divergence non triviale est soumise à l'utilisateur, qui tranche en connaissance de cause, éventuellement plus tard.

Cible : Android (`aarch64-linux-android`), en premier lieu un Fairphone 6 sous /e/OS.

## 2. Principes directeurs

1. **La synchronisation ne bloque jamais.** Ce qui est trivial est fusionné automatiquement ; le reste part dans une file de décisions.
2. **Rien n'est détruit sans décision.** Une suppression détectée sur une source ne supprime rien d'office.
3. **Traçabilité par champ.** Chaque valeur garde sa provenance (source, date).
4. **Rien ne sort du téléphone sans action explicite** pour les services tiers (photos de profil).
5. **Aucune perte d'information** : les propriétés vCard inconnues sont conservées telles quelles et renvoyées à la source.
6. **Cœur métier indépendant d'Android**, testable sur poste de développement.

## 3. Architecture

### 3.1 Cœur Rust

- Modèle de données et stockage SQLite.
- Moteur de synchronisation : fusion à trois voies (référentiel, source, dernier état synchronisé), champ par champ.
- File de décisions.
- Détection des doublons.
- Connecteurs de sources et fournisseurs de photos (derrière des traits).
- Parsing et génération vCard 3 et 4.

### 3.2 Couche Android

Hypothèse par défaut : **Kotlin / Jetpack Compose**, reliée au cœur par **UniFFI**. Alternative non retenue à ce stade : Tauri 2 (point ouvert n° 1).

- Compte système dédié (`AccountAuthenticator`) et `SyncAdapter`.
- Le référentiel est écrit dans `ContactsContract` **sous le seul compte de l'application** : le téléphone, les SMS et les messageries ne voient qu'un bloc.
- UI propre d'édition, de résolution et de configuration.

### 3.3 Contrainte de déploiement

Le moteur synchronise **directement** avec les serveurs. Les comptes correspondants doivent être retirés de DAVx5 ou de la synchronisation /e/OS, sinon les contacts des sources réapparaissent en parallèle dans Android.

## 4. Modèle de données

| Entité | Rôle |
|---|---|
| `contact` | Fiche maître (identifiant stable, drapeau « moi » pour le propriétaire) |
| `field` | Valeur typée d'une fiche (nom, téléphone, e-mail, adresse, note…), avec provenance et date |
| `source` | Source configurée (type, identifiants, paramètres de synchro) |
| `source_link` | Correspondance fiche maître ↔ identifiant dans une source (UID vCard, ETag) |
| `sync_base` | Dernier état synchronisé par fiche et par source (base de la fusion à trois voies) |
| `decision` | Élément de la file de décisions |
| `relation` | Lien typé entre deux fiches |
| `group` / `group_member` | Groupes hiérarchiques et appartenance |
| `event_date` | Dates associées à une fiche |
| `channel` / `channel_pref` | Canaux disponibles et classements pro / perso |
| `photo_candidate` | Photos candidates avec provenance ; une seule retenue |
| `social_handle` | Identifiants sociaux d'une fiche |
| `raw_property` | Propriétés vCard non interprétées, conservées par source |

## 5. Sources et synchronisation

### 5.1 Sources visées

Toutes, par ordre de mise en œuvre :

1. **CardDAV** (Murena, Nextcloud) — connecteur principal.
2. **Google** — via son interface CardDAV de préférence (évite la dépendance aux services Play) ; OAuth requis. À vérifier à l'implémentation.
3. **Stockage local Android** — import puis migration vers le référentiel.
4. **SIM** — nom et numéro seulement ; lecture seule ou import ponctuel par défaut.

Les messageries (WhatsApp, Signal…) ne sont pas des sources : leurs entrées se rattachent par numéro.

### 5.2 Configuration par source

| Paramètre | Valeurs | Défaut |
|---|---|---|
| Direction | bidirectionnelle, lecture seule, écriture seule | bidirectionnelle |
| Autorité | référentiel, ou cette source comme référence | référentiel |
| Suppressions | demander, propager, ignorer | demander |
| Conflits | demander, l'autorité gagne | demander |

- Une seule source peut être désignée comme référence ; ses conflits se résolvent alors en sa faveur sans passer par la file.
- Hypothèse par défaut : autorité réglée **par source**. Le modèle doit permettre un réglage par champ ultérieur (point ouvert n° 2).

### 5.3 Fusion

- Comparaison à trois voies, champ par champ.
- Modification d'un seul côté : appliquée automatiquement.
- Modification des deux côtés avec la même valeur : rien à faire.
- Modification divergente : décision créée ; le référentiel garde sa valeur en attendant.
- Champs multivalués (téléphones, e-mails) : comparaison par valeur normalisée ; ajout d'un côté = fusion automatique.

### 5.4 Import initial et doublons

- Détection par numéro normalisé, e-mail, nom proche.
- Doublons certains fusionnés ; doublons probables soumis en décision, avec traitement par lot.

## 6. File de décisions

- Types : divergence de champ, suppression, doublon probable, nouvelle photo.
- Contenu : les trois versions (référentiel, source, dernier état synchronisé), la source, la date, le statut (en attente, reportée, résolue).
- Suppression détectée sur une source, trois issues : propager partout, restaurer sur la source, détacher la fiche de cette source.
- Suppression demandée dans l'application : choix explicite de la portée, selon la configuration des sources.

### UI de résolution

- Boîte de réception avec compteur.
- Vue comparée champ par champ, avec source et date de chaque valeur.
- Actions : garder A, garder B, garder les deux (multivalués), éditer à la main, reporter.
- « Appliquer aux cas similaires » pour le traitement en lot.

## 7. Relations

- Fiche « moi » pour le propriétaire.
- Liens typés entre **toutes** les fiches : conjoint, enfant, parent, frère/sœur, ami, collègue, médecin, types personnalisés.
- Réciproques créées automatiquement (parent ↔ enfant, conjoint ↔ conjoint) ; inverse définissable pour les types personnalisés.
- Le lien pointe vers l'identifiant maître, pas vers un nom.
- Section « Proches » en tête de liste : vue des relations de la fiche « moi » (favoris resserrés).
- Synchronisation : `RELATED` en vCard 4, extensions en vCard 3, champ « Relation » côté Android ; sinon conservé dans le référentiel seul.

## 8. Groupes

- Une fiche peut appartenir à plusieurs groupes.
- Groupes **hiérarchiques** (parent optionnel) — orientation retenue, à confirmer.
- Filtrage par un ou plusieurs groupes, sous-groupes inclus par défaut, combinable avec la recherche.
- Synchronisation : `CATEGORIES` (à plat), chemin encodé dans le libellé (`Travail/Clients/Client X`).
- Renommer ou déplacer un groupe parent est traité comme **une seule opération**, pas comme une série de divergences.
- Point ouvert n° 3 : filtre multi-groupes en « et », en « ou », ou au choix.

## 9. Photos

### 9.1 Fournisseurs

Distincts des sources de synchro, en lecture seule, désactivés par défaut, activables un par un.

- Accessibles a priori : Gravatar / Libravatar (e-mail haché), Mastodon / Fediverse, Bluesky, GitHub, GitLab (pseudo), Telegram (via le compte de l'utilisateur), Matrix.
- Fermés : Facebook, Instagram, LinkedIn, X, WhatsApp, Signal — repli manuel (partage de l'image vers l'application, ou galerie).
- Les conditions d'accès de chaque API sont à vérifier à l'implémentation.

### 9.2 Récupération

- **Toujours à la demande**, jamais en arrière-plan.
- Trois périmètres : une fiche, un groupe, tous les contacts.
- Choix des fournisseurs à chaque lancement.
- Sur un lot : tâche de fond avec progression, puis écran de revue (validation fiche par fiche ou en lot).

### 9.3 Modèle et UI

- Plusieurs photos candidates par fiche, avec provenance et date ; une seule retenue, poussée vers les sources après redimensionnement.
- UI : galerie des candidates, prise de photo, choix dans la galerie, recadrage.

## 10. Dates

- Types : anniversaire, fête du prénom, anniversaire de mariage, dates personnalisées avec libellé.
- Année facultative.
- Fête du prénom proposée d'après le prénom, corrigeable.
- Vue « À venir », filtrable par groupe.
- Synchronisation : `BDAY` et `ANNIVERSARY` en vCard ; champ « Événement » côté Android ; le reste en propriétés étendues ou dans le référentiel.
- Point ouvert n° 4 : rappels émis par l'application, ou alimentation de l'agenda. Le modèle n'en dépend pas.

## 11. Canaux de communication

- Canaux disponibles par fiche : appel, SMS, e-mail, WhatsApp, Signal, Telegram, Matrix…, chacun rattaché à un identifiant de la fiche.
- **Deux classements par fiche : pro et perso.** Si un seul est renseigné, il vaut pour les deux.
- Note libre possible par canal.
- Détection assistée : les messageries installées déclarent leurs contacts dans Android ; l'application suggère, l'utilisateur confirme.
- Sur la fiche : boutons d'action triés selon la préférence, ouvrant l'application concernée.
- Filtrage des contacts par canal disponible.
- Point ouvert n° 5 : détermination du contexte pro / perso (hypothèse : bascule manuelle).

## 12. Points ouverts

| N° | Sujet | Hypothèse par défaut |
|---|---|---|
| 1 | UI : Kotlin/Compose + UniFFI, ou Tauri 2 | Kotlin/Compose + UniFFI |
| 2 | Autorité par source ou par champ | Par source, extensible |
| 3 | Filtre multi-groupes : et / ou / au choix | Non tranché |
| 4 | Rappels de dates : application ou agenda | Non tranché |
| 5 | Contexte pro / perso | Bascule manuelle |
| 6 | Groupes hiérarchiques | Oui, à confirmer |
| 7 | Nom définitif du projet | `unicontacts` provisoire |

## 13. Découpage proposé

1. **Cœur Rust** : modèle, SQLite, vCard, fusion à trois voies, file de décisions — testé sur des jeux de fichiers vCard.
2. **Connecteur CardDAV.**
3. **Application Android** : compte système, écriture dans `ContactsContract`, UI de liste, d'édition et de résolution.
4. **Relations, groupes, dates, canaux** dans l'UI.
5. **Google, SIM, import local.**
6. **Fournisseurs de photos.**

## 14. Structure initiale du dépôt

```
unicontacts/
├── Cargo.toml            # workspace
├── crates/
│   └── core/             # modèle, stockage, fusion, décisions
├── docs/
│   └── SPEC.md           # ce document
├── README.md
└── .gitignore
```

Les crates `vcard`, `carddav`, `ffi` (UniFFI) et le projet Android seront ajoutés au fil des étapes.

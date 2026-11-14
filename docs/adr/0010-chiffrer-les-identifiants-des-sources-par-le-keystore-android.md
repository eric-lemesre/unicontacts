# ADR-0010 — Chiffrer les identifiants des sources par le Keystore Android

**Statut** : Accepté
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-12 au registre
**Portée** : durable

## Contexte

Chaque source CardDAV se connecte avec un identifiant et un mot de passe
(de préférence un mot de passe d'application). Le téléphone doit les garder
pour synchroniser sans les redemander, sans jamais les écrire en clair, ni
dans la base SQLite du cœur, ni dans un journal.

## Décision

Les paramètres de connexion d'une source (URL du carnet, identifiant, mot de
passe) sont gardés **côté Android**, jamais dans le cœur Rust : le cœur ne
connaît que l'identifiant et les réglages de la source, et reçoit le mot de
passe à l'appel de la synchronisation, le temps de la requête.

Le mot de passe est **chiffré en AES-GCM** par une clé du **Keystore
Android**, propre à l'application, non exportable ; seul le texte chiffré et
son vecteur d'initialisation sont écrits dans les préférences privées de
l'application. L'URL et l'identifiant y sont écrits en clair.

## Alternatives écartées

### AccountManager

Simple, et déjà présent pour le compte système, mais il garde le mot de
passe pour le compte de l'application sans le chiffrer par une clé qui lui
soit propre.

### Base du cœur

Un seul lieu de stockage, mais le mot de passe y serait en clair, et la
base se sauvegarde et se copie plus facilement qu'une clé du Keystore.

## Conséquences

- Ce que cela apporte : un mot de passe illisible hors de l'application, même
  sur une copie des fichiers ; aucun secret dans la base du cœur.
- Ce que cela coûte : une réinstallation, ou la perte de la clé du Keystore,
  oblige à ressaisir les mots de passe.
- Ce que cela ferme : la sauvegarde des mots de passe par la sauvegarde
  Android (`allowBackup` désactivé).

## Vérification

- Revue : le schéma SQLite du cœur (`crates/core/src/stockage.rs`) n'a
  aucune colonne de mot de passe ; `synchroniser_carddav` reçoit le mot de
  passe en paramètre et ne le garde pas.
- Revue : le code Kotlin n'écrit le mot de passe qu'après chiffrement par la
  clé du Keystore, et ne le journalise pas.
- Le `Debug` du mot de passe côté Rust ne l'affiche pas (test du crate
  `carddav`).

## Références

- ADR-0006, ADR-0009 ; ToDo-0004.
- <https://developer.android.com/privacy-and-security/keystore>

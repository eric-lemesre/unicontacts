# ADR-0008 — Éprouver le connecteur CardDAV contre Radicale

**Statut** : Accepté
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-11 au registre
**Portée** : durable

## Contexte

L'exigence `req~source-carddav~1` déclare des tests d'intégration
(`itest`) : le connecteur doit être éprouvé contre un vrai serveur CardDAV,
en local comme en CI, sans compte Murena ni Nextcloud. Un serveur simulé ne
prouverait que notre propre lecture du protocole.

## Décision

Les tests d'intégration CardDAV tournent contre **Radicale**, serveur
CardDAV libre, lancé pour l'occasion sur `127.0.0.1`, avec un stockage
jetable.

- Version **épinglée** (Radicale 3.9.1) avec ses dépendances, chacune par
  son empreinte SHA-256 (`tools/radicale-requirements.txt`, installé par
  `pip --require-hashes`), dans un environnement Python propre au dépôt
  (`.cache/radicale-3.9.1/`) : rien n'est installé hors du dépôt.
- `sh tools/radicale.sh test` installe au besoin, démarre le serveur, lance
  les tests marqués « requiert Radicale » et arrête le serveur ; la CI le
  joue dans sa voie complète.
- Les tests d'intégration sont marqués `#[ignore]` avec leur raison : un
  simple `cargo test` les montre ignorés, jamais réussis.

Nouvelle dépendance d'outillage : **Python 3.9 ou plus, avec son module
`venv`** (module de bootstrap `tools/bootstrap.d/60-python.sh`).

## Alternatives écartées

### Serveur simulé en Rust

Autonome et rapide, mais il ne prouve que la conformité à notre propre
lecture du protocole : précisément ce qu'un test d'intégration doit mettre
à l'épreuve.

### Conteneur Nextcloud

Le serveur le plus proche de l'usage réel, mais lourd (base, PHP), lent à
démarrer, et dépendant d'un moteur de conteneurs sur le poste.

## Conséquences

- Ce que cela apporte : préconditions `If-Match`, ETag, `REPORT` et
  réponses réelles d'un serveur ; tests reproductibles hors ligne une fois
  l'environnement installé.
- Ce que cela coûte : Python sur le poste et en CI ; un premier lancement
  qui télécharge environ 1 Mo depuis PyPI.
- Ce que cela ferme : rien ; un autre serveur pourra s'ajouter derrière le
  même script.

## Vérification

- `sh tools/radicale.sh test` est câblé dans la voie complète de
  `tools/ci.sh` et dans la CI GitHub (R7).
- R3 : à la mise en place, un test d'intégration rendu faux doit faire
  échouer `sh tools/radicale.sh test`, et le script échoue si Radicale ne
  démarre pas, plutôt que de passer sans rien tester.
- `pip --require-hashes` refuse tout paquet dont l'empreinte diffère du
  fichier épinglé.

## Références

- `req~source-carddav~1` ; ADR-0007 ; ToDo-0003.
- <https://radicale.org/>

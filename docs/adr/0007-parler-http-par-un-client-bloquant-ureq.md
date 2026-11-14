# ADR-0007 — Parler HTTP par un client bloquant, ureq

**Statut** : Accepté
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-11 au registre
**Portée** : durable

## Contexte

Le lot 2 relie le référentiel à un carnet CardDAV (Murena, Nextcloud) : il
faut un client HTTP dans le workspace Rust. Ce code tournera sur Android,
appelé depuis Kotlin par UniFFI (ADR-0006), dans une tâche de fond. Une
synchronisation enchaîne quelques requêtes par carnet ; elle n'a pas besoin
de milliers de requêtes concurrentes.

## Décision

Les connecteurs HTTP du workspace utilisent **ureq** (version 3), client
**synchrone**, avec TLS par rustls. Ils exposent des fonctions bloquantes ;
la couche Android les appelle depuis un fil de fond (WorkManager), jamais
depuis le fil de l'interface. Aucun runtime asynchrone (tokio) n'entre dans
le workspace pour ce besoin.

Le réseau reste hors du cœur : le connecteur vit dans son propre crate
(`crates/carddav`), derrière un trait, et n'appelle le cœur qu'avant et après
le travail réseau. Aucune transaction de stockage n'est ouverte pendant un
appel réseau.

## Alternatives écartées

### reqwest et tokio

Client asynchrone répandu, mais il embarque le runtime tokio et imposerait
des fonctions asynchrones à travers UniFFI. Le gain — la concurrence de
nombreuses requêtes — ne sert pas une synchronisation de carnet.

### Client CardDAV tout fait

Les crates CardDAV existantes reposent sur hyper et tokio, ou couvrent mal
les préconditions d'écriture (`If-Match`) dont la fusion a besoin. Le
protocole utile (REPORT, PUT, DELETE) est court.

## Conséquences

- Ce que cela apporte : peu de dépendances, aucune frontière asynchrone à
  tenir dans UniFFI ; un code séquentiel, simple à tester.
- Ce que cela coûte : chaque synchronisation occupe un fil le temps de ses
  requêtes ; les carnets se synchronisent l'un après l'autre.
- Ce que cela ferme : rien d'irréversible ; un connecteur pourrait passer à
  l'asynchrone derrière le même trait.
- À reprendre au lot 3 : la vérification des certificats par le magasin du
  système Android (au lieu des racines embarquées).

## Vérification

- `crates/carddav/Cargo.toml` déclare `ureq` et aucune dépendance `tokio`
  ni `reqwest` : revue de chaque PR qui touche un manifeste.
- `crates/core` ne dépend d'aucun client HTTP
  (`req~coeur-independant-android~1` : ses tests tournent sans réseau).

## Références

- ADR-0003, ADR-0006 ; ToDo-0003.
- <https://docs.rs/ureq/3>

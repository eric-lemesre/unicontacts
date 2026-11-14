# ADR-0004 — Écrire le référentiel sous le seul compte de l'application, synchroniser en direct

**Statut** : Proposé
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur) ; à accepter à l'arbitrage du
domaine intégration Android
**Portée** : durable

## Contexte

Android agrège les contacts de chaque compte (Google, DAVx5, SIM, local) et
les fusionne à l'affichage, de façon fragile. Pour que le téléphone, les SMS
et les messageries ne voient qu'un bloc, il faut qu'un seul compte porte les
contacts. Spécification v0.1, § 3.2 et 3.3.

## Décision

L'application déclare son propre compte système (`AccountAuthenticator`) et
un `SyncAdapter`, et écrit le référentiel dans `ContactsContract` sous ce
seul compte. Elle synchronise chaque source directement avec son serveur ;
les comptes de ces sources sont retirés de DAVx5 et de la synchronisation
/e/OS, consigne de déploiement documentée pour l'utilisateur.

## Alternatives écartées

### Écrire dans les comptes existants des sources

Chaque source garde son compte et l'application corrige leurs contenus :
c'est le modèle actuel d'Android, avec ses doublons et ses agrégations
fragiles, que le projet veut quitter.

### S'appuyer sur DAVx5 pour le transport CardDAV

Moins de code réseau, mais DAVx5 écrit sous ses propres comptes : les
contacts réapparaissent en parallèle du bloc.

## Conséquences

- Ce que cela apporte : un seul bloc vu par tout le téléphone ; maîtrise
  complète de la synchronisation et de ses conflits.
- Ce que cela coûte : écrire et tenir les connecteurs (CardDAV, Google) ;
  une étape de migration pour l'utilisateur (retirer ses comptes DAVx5 /
  /e/OS).
- Ce que cela ferme : la cohabitation avec un autre synchroniseur des mêmes
  carnets.

## Vérification

- `req~ecriture-sous-compte-unique~1` et `req~synchronisation-directe~1`,
  vérifiées par procédure manuelle sur l'appareil cible (`mtest`).

## Références

- Spécification v0.1, § 3.2 et 3.3 (`spec/01-besoins/sources/`).
- `req~compte-systeme-dedie~1`, `req~ecriture-sous-compte-unique~1`,
  `req~synchronisation-directe~1`, OQ-001.

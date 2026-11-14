# ADR-0005 — Fusionner à trois voies, champ par champ, et différer les divergences

**Statut** : Proposé
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur) ; à accepter à l'arbitrage des
domaines synchronisation et file de décisions
**Portée** : durable

## Contexte

Une synchronisation à deux voies (référentiel contre source) ne sait pas quel
côté a changé : toute différence devient un conflit, ou l'un des côtés écrase
l'autre. La spécification v0.1 (§ 2 et 5.3) veut au contraire une
synchronisation qui ne bloque jamais et qui ne détruit rien sans décision.

## Décision

Le moteur conserve, par fiche et par source, le dernier état synchronisé
(`sync_base`) et compare chaque champ selon trois valeurs : référentiel,
source, dernier état synchronisé. Un changement d'un seul côté s'applique ;
un même changement des deux côtés ne fait rien ; un changement divergent
crée une décision et le référentiel garde sa valeur en attendant. Les champs
multivalués se comparent par valeur normalisée. Une suppression détectée
devient une décision (selon le réglage de la source), jamais une
destruction. La synchronisation poursuit le reste sans attendre.

## Alternatives écartées

### Deux voies, « le plus récent gagne »

Simple, mais repose sur des horloges de serveurs et perd silencieusement une
des deux modifications.

### Fusion par fiche entière

Moins d'état à garder, mais une modification du téléphone d'un côté et de
l'adresse de l'autre deviendrait un conflit.

## Conséquences

- Ce que cela apporte : des conflits rares et réels ; aucune perte
  silencieuse ; une synchronisation jamais suspendue.
- Ce que cela coûte : stocker un état de base par fiche et par source ;
  normaliser les valeurs multivaluées.
- Ce que cela ferme : rien.

## Vérification

- Les exigences du domaine synchronisation (`req~fusion-trois-voies~1`,
  `req~modification-unilaterale-appliquee~1`,
  `req~divergence-soumise-a-decision~1`,
  `req~multivalues-par-valeur-normalisee~1`,
  `req~synchronisation-non-bloquante~1`,
  `req~suppression-sans-destruction~1`) sont couvertes par des tests
  unitaires du cœur sur des jeux de vCard.

## Références

- Spécification v0.1, § 2, 5.3 et 6 (`spec/01-besoins/sources/`).

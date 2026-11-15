# ADR-0013 — Tirer le calendrier des fêtes de Wikidata

**Statut** : Accepté
**Date** : 2026-11-15
**Décideur** : Eric Lemesre — source choisie le 2026-11-15 (« jeu de données ouvert »), D-2026-11-13 au registre
**Portée** : durable

## Contexte

L'exigence `req~fete-du-prenom-proposee~1` demande un calendrier prénom →
jour de fête.
L'utilisateur a choisi un jeu de données ouvert plutôt qu'une table compilée
de mémoire. Le dépôt est sous GPL-3.0 : la licence du jeu doit permettre de
le redistribuer.

## Décision

Le calendrier vient de **Wikidata** : propriété P1750 (« fête du prénom »)
qualifiée par P17 = France (Q142), le calendrier traditionnel français,
sous licence **CC0**. Il est versionné dans `crates/core/data/fetes-fr.txt`
(une ligne `MM-JJ<tab>prénom`), embarqué dans le cœur, avec son empreinte
SHA-256 à côté.

Le fichier ne s'édite jamais à la main (R8) : `sh tools/fetes.sh regenerer`
interroge Wikidata et le réécrit (réseau requis, hors CI) ; on relit le diff
et on committe fichier et empreinte. `sh tools/fetes.sh verifier`, en voie
documentaire, refuse un calendrier dont l'empreinte ne correspond pas.

Un prénom peut avoir plusieurs jours de fête (Jean : 8 mars, 23 octobre,
27 décembre) : tous sont proposés, l'utilisateur choisit. Un prénom composé
absent du calendrier se rabat sur son premier élément.

Le fichier est un `.txt` et non un `.csv` : la porte « données » refuse les
exports tabulaires parce qu'ils trahissent d'ordinaire des données réelles
d'utilisateurs ; un calendrier public, sous CC0, n'en est pas un, et sa
provenance, sa régénération et son empreinte sont tracées ici.

## Alternatives écartées

### Table compilée de mémoire

Rapide, mais sans source vérifiable : une date fausse ne se corrigerait
qu'au hasard d'un signalement.

### Jeu « Les saints et fêtes du calendrier » de data.gouv.fr

Licence Ouverte, mais le fichier est faux : mai et juin y sont inversés
(Éric au 18 juin), anomalie signalée et non corrigée depuis 2024.

### Fichier `fr.json` du dépôt daniel-bocan/namedays

Plus de prénoms (550), sous MIT, mais de provenance mêlée (en partie
reprise de Wikipédia, sous CC BY-SA) et créé deux semaines plus tôt.

## Conséquences

- Ce que cela apporte : une source vérifiable et corrigeable en amont, une
  licence sans condition, un calendrier régénérable à l'identique.
- Ce que cela coûte : environ 360 prénoms, sans variantes orthographiques ;
  une régénération manuelle quand on veut profiter des corrections.
- Ce que cela ferme : les calendriers d'autres pays, jusqu'à un réglage
  qui les demanderait.

## Vérification

- `sh tools/fetes.sh verifier` en CI (voie documentaire) : éprouvé sur une
  retouche à la main, refusée (R3).
- Tests du cœur : Éric proposé au 18 mai, Nicolas au 6 décembre, aucune
  proposition pour une fiche qui a sa fête.

## Références

- `req~fete-du-prenom-proposee~1` ; ToDo-0005.
- <https://www.wikidata.org/wiki/Property:P1750>

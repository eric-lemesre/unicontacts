# Exigences — domaine groupes

**Statut : ACTIVE** — domaine acté le 2026-11-15 (D-2026-11-13) au lot 4,
rédigé depuis la spécification v0.1 (§ 8). Hiérarchie des groupes : OQ-006 ;
opérateur du filtre : OQ-003 (tranchées le 2026-11-14). Désignation des
groupes par leur chemin : ADR-0012.

## Appartenance à plusieurs groupes

`req~appartenance-multiple~1`
**Statut : approuvée**

Le système DOIT permettre qu'une fiche appartienne à plusieurs groupes.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

## Groupes hiérarchiques

`req~groupes-hierarchiques~1`
**Statut : approuvée**

Le système DOIT permettre de donner à un groupe un groupe parent facultatif,
sans cycle.

Rationale:
Orientation de la spécification v0.1, confirmée par OQ-006 (2026-11-14).

Critères d'acceptation :

- Étant donné B enfant de A, quand A est désigné enfant de B, alors le réglage
  est refusé.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

## Filtrage par groupes

`req~filtre-par-groupes~2`
**Statut : approuvée**

Quand l'utilisateur filtre la liste par un ou plusieurs groupes, le système
DOIT n'afficher que les fiches de l'un de ces groupes (union, par défaut) ou
de tous ces groupes (intersection), au choix de l'utilisateur, sous-groupes
compris par défaut, et permettre de combiner ce filtre avec la recherche.

Rationale:
Révision 2 (2026-11-14) : l'opérateur est tranché par OQ-003.

Critères d'acceptation :

- Étant donné les groupes « Famille » et « Voisins » sélectionnés sans autre
  réglage, quand le filtre s'applique, alors les fiches de l'un ou l'autre
  groupe sont affichées.
- Étant donné la même sélection en intersection, quand le filtre s'applique,
  alors seules les fiches des deux groupes à la fois sont affichées.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

## Groupes synchronisés en catégories

`req~groupes-en-categories~1`
**Statut : approuvée**

Quand une fiche est écrite vers une source vCard, le système DOIT porter ses
groupes dans `CATEGORIES`, chaque groupe désigné par son chemin complet
(`Travail/Clients/Client X`), et reconstruire la hiérarchie à la lecture.

Critères d'acceptation :

- Étant donné une fiche du groupe « Client X » enfant de « Clients » enfant de
  « Travail », quand elle est écrite, alors `CATEGORIES` contient
  `Travail/Clients/Client X`.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

## Renommage ou déplacement en une seule opération

`req~renommage-groupe-operation-unique~1`
**Statut : approuvée**

Quand un groupe parent est renommé ou déplacé, le système DOIT propager le
changement aux fiches concernées comme une seule opération, sans créer de
décision de divergence par fiche.

Critères d'acceptation :

- Étant donné 40 fiches dans des sous-groupes de « Travail », quand
  « Travail » est renommé, alors aucune décision n'est créée et les 40 fiches
  portent le nouveau chemin à la synchronisation suivante.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

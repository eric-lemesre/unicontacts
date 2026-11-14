# Exigences — domaine groupes

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 8) ; passe ACTIVE quand une décision du registre l'acte. La hiérarchie des
groupes (OQ-006) et l'opérateur du filtre (OQ-003) restent à trancher.

## Appartenance à plusieurs groupes

`req~appartenance-multiple~1`
Status: proposed

**Statut : proposée**

Le système DOIT permettre qu'une fiche appartienne à plusieurs groupes.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

## Groupes hiérarchiques

`req~groupes-hierarchiques~1`
Status: proposed

**Statut : proposée**

Le système DOIT permettre de donner à un groupe un groupe parent facultatif,
sans cycle.

Rationale:
Orientation de la spécification v0.1, à confirmer : OQ-006.

Critères d'acceptation :

- Étant donné B enfant de A, quand A est désigné enfant de B, alors le réglage
  est refusé.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

## Filtrage par groupes

`req~filtre-par-groupes~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur filtre la liste par un ou plusieurs groupes, le système
DOIT n'afficher que les fiches de ces groupes, sous-groupes compris par
défaut, et permettre de combiner ce filtre avec la recherche.

Rationale:
L'opérateur entre plusieurs groupes (« et », « ou », au choix) est l'objet
d'OQ-003.

Covers:

- feat~organisation-en-groupes~1

Needs: impl, utest

## Groupes synchronisés en catégories

`req~groupes-en-categories~1`
Status: proposed

**Statut : proposée**

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
Status: proposed

**Statut : proposée**

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

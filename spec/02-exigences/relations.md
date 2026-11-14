# Exigences — domaine relations

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 7) ; passe ACTIVE à l'ouverture de son lot, par une décision du registre.

## Fiche « moi »

`req~fiche-moi~1`
Status: proposed

**Statut : proposée**

Le système DOIT permettre de désigner une fiche comme celle du propriétaire
(« moi »), et une seule à la fois.

Critères d'acceptation :

- Étant donné une fiche « moi », quand une autre fiche est désignée « moi »,
  alors la première perd cette qualité.

Covers:

- feat~relations-entre-contacts~1

Needs: impl, utest

## Liens typés entre fiches

`req~liens-types~1`
Status: proposed

**Statut : proposée**

Le système DOIT permettre de lier deux fiches quelconques par un type de
relation, prédéfini (conjoint, enfant, parent, frère/sœur, ami, collègue,
médecin) ou personnalisé par l'utilisateur.

Covers:

- feat~relations-entre-contacts~1

Needs: impl, utest

## Réciproques automatiques

`req~reciproques-automatiques~1`
Status: proposed

**Statut : proposée**

Quand un lien typé est créé entre deux fiches, le système DOIT créer le lien
réciproque (parent ↔ enfant, conjoint ↔ conjoint), en utilisant pour un type
personnalisé l'inverse défini par l'utilisateur s'il existe.

Critères d'acceptation :

- Étant donné A lié à B comme « parent », quand le lien est créé, alors B est
  lié à A comme « enfant ».
- Étant donné un type personnalisé sans inverse défini, quand le lien est
  créé, alors aucun lien réciproque n'est créé.

Covers:

- feat~relations-entre-contacts~1

Needs: impl, utest

## Lien par identifiant maître

`req~lien-par-identifiant-maitre~1`
Status: proposed

**Statut : proposée**

Le système DOIT enregistrer un lien entre fiches par l'identifiant maître de
la fiche liée, jamais par son nom.

Critères d'acceptation :

- Étant donné A lié à B, quand B est renommée, alors le lien de A désigne
  toujours B.

Covers:

- feat~relations-entre-contacts~1

Needs: impl, utest

## Section « Proches »

`req~section-proches~1`
Status: proposed

**Statut : proposée**

Là où une fiche « moi » a des relations, le système DOIT afficher en tête de
la liste des contacts une section « Proches » formée des fiches liées à la
fiche « moi ».

Covers:

- feat~relations-entre-contacts~1

Needs: impl, mtest

## Synchronisation des relations

`req~synchronisation-relations~1`
Status: proposed

**Statut : proposée**

Quand une fiche est écrite vers une source ou vers Android, le système DOIT y
porter ses relations sous la forme que la cible accepte — `RELATED` en vCard
4.0, propriété étendue en vCard 3.0, champ « Relation » d'Android — et les
conserver dans le seul référentiel pour une cible qui n'en accepte aucune.

Covers:

- feat~relations-entre-contacts~1

Needs: impl, utest

# Exigences — domaine dates

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 10) ; passe ACTIVE à l'ouverture de son lot, par une décision du registre. Les rappels sont
confiés à l'agenda du téléphone (OQ-004, tranchée le 2026-11-14).

## Types de dates, année facultative

`req~types-de-dates~1`
Status: proposed

**Statut : proposée**

Le système DOIT permettre d'associer à une fiche des dates de type
anniversaire, fête du prénom, anniversaire de mariage ou personnalisé avec
libellé, chacune avec une année facultative.

Critères d'acceptation :

- Étant donné un anniversaire saisi sans année, quand il est enregistré puis
  relu, alors il porte jour et mois, et aucune année.

Covers:

- feat~dates-a-retenir~1

Needs: impl, utest

## Fête du prénom proposée

`req~fete-du-prenom-proposee~1`
Status: proposed

**Statut : proposée**

Quand une fiche a un prénom figurant au calendrier des fêtes et pas de fête
du prénom, le système DOIT proposer la date correspondante, que l'utilisateur
accepte ou corrige.

Covers:

- feat~dates-a-retenir~1

Needs: impl, utest

## Vue « À venir »

`req~vue-a-venir~1`
Status: proposed

**Statut : proposée**

Le système DOIT présenter les dates à venir dans l'ordre chronologique,
filtrables par groupe.

Covers:

- feat~dates-a-retenir~1

Needs: impl, mtest

## Synchronisation des dates

`req~synchronisation-dates~1`
Status: proposed

**Statut : proposée**

Quand une fiche est écrite vers une source ou vers Android, le système DOIT
porter l'anniversaire dans `BDAY`, l'anniversaire de mariage dans
`ANNIVERSARY` et les autres dates en propriétés étendues côté vCard, et les
dates dans le champ « Événement » côté Android ; une date qu'une cible
n'accepte pas reste dans le référentiel.

Covers:

- feat~dates-a-retenir~1

Needs: impl, utest

## Dates reportées dans l'agenda

`req~dates-dans-l-agenda~1`
Status: proposed

**Statut : proposée**

Quand une date d'une fiche est créée, modifiée ou supprimée, le système DOIT
reporter le changement sur un événement annuel récurrent d'un agenda du
téléphone choisi par l'utilisateur, et laisser à cet agenda l'émission des
rappels.

Rationale:
OQ-004, tranchée le 2026-11-14 : l'agenda sait déjà rappeler et se
synchronise ; l'application n'émet pas ses propres notifications de date.

Critères d'acceptation :

- Étant donné un agenda choisi, quand un anniversaire est ajouté à une fiche,
  alors un événement annuel récurrent apparaît dans cet agenda.
- Étant donné cet événement, quand l'anniversaire est supprimé de la fiche,
  alors l'événement disparaît de l'agenda.

Covers:

- feat~dates-a-retenir~1

Needs: impl, mtest

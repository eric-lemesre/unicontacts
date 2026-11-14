# Exigences — domaine file de décisions

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 6) ; passe ACTIVE quand une décision du registre l'acte.

## Contenu d'une décision

`req~contenu-decision~1`
Status: proposed

**Statut : proposée**

Le système DOIT présenter, pour chaque décision, son type (divergence de
champ, suppression, doublon probable, nouvelle photo), les versions en
présence (référentiel, source, dernier état synchronisé), la source
concernée, sa date de création et son statut (en attente, reportée,
résolue).

Critères d'acceptation :

- Étant donné une décision de divergence, quand elle est lue, alors ses trois
  versions, sa source, sa date et son statut sont présents.

Covers:

- feat~arbitrage-differe~1

Needs: impl, utest

## Boîte de réception avec compteur

`req~boite-de-decisions~1`
Status: proposed

**Statut : proposée**

Tant qu'au moins une décision est en attente ou reportée, le système DOIT en
afficher le nombre et donner accès à la liste des décisions.

Critères d'acceptation :

- Étant donné trois décisions en attente et une reportée, quand l'écran
  principal s'affiche, alors le compteur indique 4 et ouvre la liste.

Covers:

- feat~arbitrage-differe~1

Needs: impl, mtest

## Vue comparée champ par champ

`req~vue-comparee~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur ouvre une décision, le système DOIT afficher côte à côte,
champ par champ, les valeurs en présence avec la source et la date de chacune.

Covers:

- feat~arbitrage-differe~1

Needs: impl, mtest

## Actions de résolution

`req~actions-de-resolution~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur résout une décision de divergence, le système DOIT lui
offrir : garder la valeur du référentiel, garder celle de la source, garder
les deux (champs multivalués seulement), saisir une valeur à la main, ou
reporter.

Critères d'acceptation :

- Étant donné une divergence sur un champ à valeur unique, quand les actions
  sont proposées, alors « garder les deux » n'en fait pas partie.
- Étant donné une divergence résolue par une valeur saisie, quand la source
  est synchronisée, alors cette valeur est écrite vers la source et la
  décision est résolue.

Covers:

- feat~arbitrage-differe~1

Needs: impl, utest

## Report d'une décision

`req~report-decision~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur reporte une décision, le système DOIT la conserver avec
le statut « reportée », sans rien modifier des valeurs en présence.

Covers:

- feat~arbitrage-differe~1

Needs: impl, utest

## Résolution en lot des cas similaires

`req~resolution-par-lot~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur résout une décision en demandant de l'appliquer aux cas
similaires, le système DOIT appliquer la même action à toutes les décisions
en attente de même type, même source et même champ, après en avoir annoncé
le nombre.

Rationale:
« Appliquer aux cas similaires » (§ 6) ; la définition de « similaire »
retenue ici est à confirmer à l'arbitrage du domaine.

Critères d'acceptation :

- Étant donné cinq divergences sur le champ « organisation » d'une même
  source, quand l'une est résolue en « garder la source » appliqué aux cas
  similaires, alors les cinq sont résolues de même.

Covers:

- feat~arbitrage-differe~1

Needs: impl, utest

## Issues d'une suppression détectée

`req~issues-suppression-detectee~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur résout une décision de suppression détectée sur une
source, le système DOIT lui offrir trois issues et appliquer celle choisie :
propager la suppression partout, restaurer la fiche sur la source, ou
détacher la fiche de cette source.

Critères d'acceptation :

- Étant donné une suppression résolue par « détacher », quand la résolution
  est appliquée, alors la fiche maître subsiste sans correspondance avec
  cette source.
- Étant donné une suppression résolue par « restaurer », quand la source est
  synchronisée, alors la fiche y est réécrite.

Covers:

- feat~arbitrage-differe~1

Needs: impl, utest

## Portée d'une suppression demandée dans l'application

`req~portee-suppression-locale~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur supprime une fiche dans l'application, le système DOIT
lui faire choisir explicitement les sources où la suppression s'applique,
parmi celles que leurs réglages autorisent en écriture.

Critères d'acceptation :

- Étant donné une fiche liée à une source en lecture seule et à une source
  bidirectionnelle, quand sa suppression est demandée, alors seule la source
  bidirectionnelle est proposée.

Covers:

- feat~arbitrage-differe~1

Needs: impl, utest

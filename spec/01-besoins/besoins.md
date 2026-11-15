# Besoins — unicontacts

Porteur : Eric Lemesre, qui arbitre les questions ouvertes
(`spec/open-questions.md`).
Source : [spécification v0.1 du 2026-11-14](sources/2026-11-14-specification-v0.1.md),
découpée ici en besoins (items feat) et dans `spec/02-exigences/` en
exigences, approuvées lot par lot à l'ouverture de chaque lot (lot 1 :
D-2026-11-09). Un besoin reste en brouillon tant qu'aucune de ses exigences
n'est approuvée.

## Un carnet unique, géré comme un seul bloc

`feat~referentiel-unique~1`

En tant que propriétaire d'un téléphone Android, je veux que mes contacts
forment un seul référentiel maître, quelle que soit leur source (compte
Google, CardDAV, SIM, stockage local), et que les sources ne soient que des
cibles de synchronisation, afin qu'une modification faite une fois vaille
partout.

Rationale:
Android gère les contacts par source et ne les agrège qu'à l'affichage : une
modification ne touche qu'une source, et les fusions automatiques sont
fragiles. Inverser la logique (spécification v0.1, § 1 et 2) supprime les
fiches en double et les modifications perdues. Aucune information ne doit se
perdre en route, y compris ce que l'application ne comprend pas.

Needs: req

## Une synchronisation qui ne bloque jamais

`feat~synchronisation-sans-blocage~1`

Je veux synchroniser le référentiel avec toutes mes sources, dans l'ordre
CardDAV, Google, stockage local, SIM, en réglant chacune (direction,
autorité, suppressions, conflits), sans que la synchronisation s'arrête pour
attendre mon avis.

Rationale:
Ce qui est trivial doit être fusionné sans moi ; ce qui ne l'est pas doit
attendre ma décision sans rien détruire ni bloquer le reste (spécification
v0.1, § 2 et 5).

Needs: req

## Trancher les divergences en connaissance de cause, éventuellement plus tard

`feat~arbitrage-differe~1`

Je veux voir dans une file chaque divergence non triviale (valeurs
différentes, suppression, doublon probable, nouvelle photo), avec les
versions en présence, leur source et leur date, et la trancher tout de
suite, plus tard, ou en lot avec les cas semblables.

Rationale:
Rien ne doit être détruit sans décision (spécification v0.1, § 2 et 6) ;
une décision éclairée exige de voir ce qui s'oppose et d'où cela vient.

Needs: req

## Le téléphone ne voit qu'un carnet

`feat~integration-android~1`

Je veux que le téléphone, les SMS et les messageries voient mes contacts
comme un seul bloc, sans doublons venus des sources.

Rationale:
Le référentiel ne sert que s'il remplace, pour le reste du téléphone,
l'agrégat des sources (spécification v0.1, § 3.2 et 3.3). Cible : Fairphone 6
sous /e/OS.

Needs: req

## Relier les personnes entre elles

`feat~relations-entre-contacts~1`

Je veux relier mes contacts entre eux et à moi-même (conjoint, enfant,
parent, collègue, médecin, types personnalisés), et retrouver mes proches en
tête de liste.

Rationale:
Les relations donnent du sens au carnet et servent de favoris naturels
(spécification v0.1, § 7).

Needs: req

## Ranger les contacts en groupes

`feat~organisation-en-groupes~1`

Je veux ranger un contact dans plusieurs groupes, éventuellement imbriqués,
et filtrer la liste par groupes.

Rationale:
Les groupes structurent un carnet de plusieurs centaines de fiches
(spécification v0.1, § 8) ; ils sont hiérarchiques (OQ-006).

Needs: req

## Des photos choisies, jamais récupérées à mon insu

`feat~photos-maitrisees~1`
Status: draft

Je veux compléter les photos de mes contacts depuis des services en ligne que
j'active un par un, seulement quand je le demande, et choisir moi-même la
photo retenue.

Rationale:
Rien ne sort du téléphone sans action explicite (spécification v0.1, § 2
et 9) : interroger un service tiers révèle l'identité de mes contacts.

Needs: req

## Ne pas oublier les dates qui comptent

`feat~dates-a-retenir~1`

Je veux noter les anniversaires, fêtes, anniversaires de mariage et dates
personnalisées de mes contacts, et voir celles qui approchent.

Rationale:
Spécification v0.1, § 10 ; les rappels sont confiés à l'agenda
(OQ-004).

Needs: req

## Joindre chacun par le bon canal

`feat~canaux-preferes~1`
Status: draft

Je veux savoir, pour chaque contact, par quels canaux le joindre (appel, SMS,
e-mail, messageries) et lequel il préfère pour une communication pro et pour
une communication perso.

Rationale:
Spécification v0.1, § 11 ; la préférence est propre à chaque contact, sans
contexte global (OQ-005).

Needs: req

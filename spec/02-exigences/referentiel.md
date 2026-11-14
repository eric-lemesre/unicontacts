# Exigences — domaine référentiel

**Statut : ACTIVE** — domaine acté le 2026-11-14 (D-2026-11-09) à l'ouverture
du lot 1, rédigé depuis la spécification v0.1 (§ 2 à 4). Seules les exigences
du lot 1 sont approuvées ; les autres le seront à l'ouverture de leur lot.

## Identifiant stable de la fiche maître

`req~fiche-maitre-identifiant-stable~1`

**Statut : approuvée**

Le système DOIT attribuer à chaque fiche maître un identifiant stable,
indépendant de toute source, qui ne change ni à la modification de la fiche
ni à l'ajout ou au retrait d'une source.

Rationale:
Relations, groupes et correspondances avec les sources pointent vers la fiche
maître ; un identifiant emprunté à une source disparaîtrait avec elle.

Critères d'acceptation :

- Étant donné une fiche liée à deux sources, quand l'une des sources est
  détachée, alors l'identifiant de la fiche est inchangé.
- Étant donné deux fiches créées avec les mêmes valeurs, quand elles sont
  enregistrées, alors leurs identifiants diffèrent.

Covers:

- feat~referentiel-unique~1

Needs: impl, utest

## Provenance de chaque valeur

`req~provenance-par-champ~1`

**Statut : approuvée**

Le système DOIT conserver, pour chaque valeur d'un champ de fiche, la source
dont elle provient et la date de sa dernière modification.

Rationale:
Principe « traçabilité par champ » (§ 2.3) : la file de décisions affiche la
source et la date de chaque valeur, et la fusion en a besoin.

Critères d'acceptation :

- Étant donné une valeur reçue d'une source CardDAV, quand elle est
  enregistrée, alors sa provenance nomme cette source et porte la date de
  réception.
- Étant donné une valeur modifiée dans l'application, quand elle est
  enregistrée, alors sa provenance devient le référentiel, à la date de la
  modification.

Covers:

- feat~referentiel-unique~1

Needs: impl, utest

## Conservation des propriétés vCard non interprétées

`req~conservation-proprietes-inconnues~1`

**Statut : approuvée**

Quand une vCard reçue d'une source contient une propriété que le système
n'interprète pas, le système DOIT la conserver telle quelle pour cette source
et la renvoyer inchangée à cette même source lors des écritures suivantes.

Rationale:
Principe « aucune perte d'information » (§ 2.5) : une propriété ignorée puis
omise à la réécriture serait effacée sur le serveur.

Critères d'acceptation :

- Étant donné une vCard portant une propriété `X-PROPRIETAIRE`, quand la
  fiche est modifiée puis réécrite vers sa source, alors la vCard écrite porte
  la même propriété, à l'octet près.
- Étant donné la même fiche liée à une seconde source, quand elle est écrite
  vers cette seconde source, alors la propriété n'y est pas recopiée.

Covers:

- feat~referentiel-unique~1

Needs: impl, utest

## Lecture et écriture vCard 3.0 et 4.0

`req~vcard-3-et-4~1`

**Statut : approuvée**

Le système DOIT lire et écrire les vCard des versions 3.0 et 4.0, et écrire
vers chaque source dans la version qu'elle utilise.

Critères d'acceptation :

- Étant donné un jeu de vCard 3.0 et 4.0 de référence, quand chacune est lue
  puis réécrite sans modification, alors le résultat est équivalent à
  l'original, propriété par propriété.
- Étant donné une source qui sert du vCard 3.0, quand une fiche y est écrite,
  alors la vCard produite déclare `VERSION:3.0`.

Covers:

- feat~referentiel-unique~1

Needs: impl, utest

## Cœur métier indépendant d'Android

`req~coeur-independant-android~1`

**Statut : approuvée**

Le cœur métier (modèle, stockage, fusion, décisions, vCard) DOIT se compiler
et passer ses tests sur un poste de développement Linux, sans SDK Android ni
émulateur.

Rationale:
Principe 6 (§ 2) : la logique la plus délicate — la fusion — doit pouvoir
être éprouvée vite et souvent, hors du téléphone.

Critères d'acceptation :

- Étant donné le dépôt sur un poste Linux sans SDK Android, quand la CI
  lance les tests du cœur, alors ils compilent et s'exécutent.

Covers:

- feat~referentiel-unique~1

Needs: impl, utest

# ADR-0011 — Porter relations et dates dans les vCard : RELATED en 4.0, convention Apple en 3.0

**Statut** : Accepté
**Date** : 2026-11-15
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-13 au registre
**Portée** : durable

## Contexte

Le lot 4 synchronise des relations entre fiches et des dates autres que la
naissance (`req~synchronisation-relations~1`, `req~synchronisation-dates~1`).
La vCard 4.0 a `RELATED` et `ANNIVERSARY` ; la vCard 3.0, que servent encore
bien des carnets, n'a ni l'une ni l'autre. Il faut aussi désigner la fiche
liée : le référentiel la tient par son identifiant maître
(`req~lien-par-identifiant-maitre~1`), qu'aucune autre application ne
connaît.

## Décision

**Dans le référentiel**, une relation est une valeur du champ `Relation` :
son type et l'identifiant maître de la fiche liée (`parent;<identifiant>`).
Types prédéfinis : conjoint, enfant, parent, frère/sœur, ami, collègue,
médecin. Réciproques : parent ↔ enfant, conjoint, frère/sœur, ami et
collègue envers eux-mêmes ; médecin n'en a pas, sauf si l'utilisateur
définit un type personnalisé dont il est l'inverse (« patient »). L'inverse
d'un type personnalisé se déclare à sa création et vaut dans les deux sens.

La réciproque se crée quand l'utilisateur crée un lien dans l'application,
et se retire quand il le retire. Un lien reçu d'une source n'en crée pas : le
référentiel ne modifie pas d'office une autre fiche sur la foi d'une source.

**En vCard 4.0**, une relation s'écrit `RELATED` :

- `TYPE` du RFC 6350 pour les types qui en ont un (`spouse`, `child`,
  `parent`, `sibling`, `friend`, `colleague`) ; `TYPE=contact` et le
  paramètre `X-ABLABEL` portant le libellé pour médecin et les types
  personnalisés ;
- valeur `urn:uuid:<UID>` quand la fiche liée a une copie dans la même
  source, sinon `VALUE=text` et le nom affiché de la fiche liée.

**En vCard 3.0**, la convention d'Apple, comprise par iOS, macOS, Nextcloud
et DAVx5 : deux propriétés d'un même groupe, `itemN.X-ABRELATEDNAMES` (nom
affiché de la fiche liée) et `itemN.X-ABLabel` (`_$!<Spouse>!$_`,
`_$!<Child>!$_`, `_$!<Parent>!$_`, `_$!<Friend>!$_`, sinon le libellé en
clair). Ces propriétés sont aussi lues dans une carte 4.0.

**Désignation de la fiche liée à la lecture** : par l'UID de sa copie dans la
source ; à défaut, par un nom affiché égal porté par une seule fiche. Chaque
écriture ou lecture résolue est mémorisée par lien vers la source : une
écriture déjà vue désigne toujours la même fiche, même renommée depuis. Une
propriété qu'on ne sait pas résoudre, ou sans libellé, est conservée telle
quelle pour la source (`req~conservation-proprietes-inconnues~1`).

**Relation qu'une source ne peut porter** (fiche liée sans copie dans une
source 3.0, ou de nom absent ou partagé) : elle reste dans le seul
référentiel, et la source n'est pas réputée l'avoir retirée.

**Dates** : `BDAY` pour la naissance ; l'anniversaire de mariage en
`ANNIVERSARY` (4.0) ou `itemN.X-ABDATE` étiqueté `_$!<Anniversary>!$_`
(3.0) ; fête du prénom et dates personnalisées en `itemN.X-ABDATE` étiqueté
par leur libellé, dans les deux versions. Date sans année : `--MMJJ` en 4.0,
`1604-MM-JJ` avec `X-APPLE-OMIT-YEAR=1604` en 3.0, lues l'une et l'autre.

**Android** : une relation s'écrit dans le champ « Relation » (`data1` le nom
de la fiche liée ; `data2` 14 conjoint, 3 enfant, 9 parent, 6 ami ; sinon
0 et le libellé en `data3`).

## Alternatives écartées

### Propriétés propres `X-UNICONTACTS-*`

Elles porteraient l'identifiant maître exact, mais aucune autre application
ne les comprendrait : le carnet perdrait ses relations hors d'unicontacts.

### Désigner toujours par le nom

Simple, mais fragile : deux homonymes ou un renommage rompent le lien. L'UID
en 4.0, et la mémoire des écritures, l'évitent là où c'est possible.

## Conséquences

- Ce que cela apporte : des relations et des dates lisibles par les
  carnets et téléphones courants, en 3.0 comme en 4.0.
- Ce que cela coûte : une table de mémoire par lien ; un nom de fiche liée
  qui peut rester ancien dans la source après un renommage, tant que la
  relation n'y est pas réécrite.
- Ce que cela ferme : la désignation exacte d'une fiche liée hors de la
  même source en 3.0 ; elle passe par le nom.

## Vérification

- Tests du cœur : écriture et relecture d'une relation en 4.0 (UID, nom) et
  en 3.0 (convention Apple) ; renommage de la fiche liée sans perte ;
  propriété non résolue conservée ; relation impossible à porter gardée au
  référentiel sans être réputée retirée.
- Tests JVM des lignes Android d'une relation.
- Revue : toute propriété de relation ou de date nouvelle passe par les
  correspondances de cet ADR.

## Références

- `req~liens-types~1`, `req~reciproques-automatiques~1`,
  `req~lien-par-identifiant-maitre~1`, `req~synchronisation-relations~1`,
  `req~synchronisation-dates~1` ; ToDo-0005.
- RFC 6350, § 6.6.6 (`RELATED`) et § 6.2.6 (`ANNIVERSARY`).

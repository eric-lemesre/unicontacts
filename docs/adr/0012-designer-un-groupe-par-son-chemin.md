# ADR-0012 — Désigner un groupe par son chemin

**Statut** : Accepté
**Date** : 2026-11-15
**Décideur** : Eric Lemesre — conception déléguée (« cycle complet » du lot 4, 2026-11-15), D-2026-11-13 au registre
**Portée** : durable

## Contexte

Les groupes sont hiérarchiques (OQ-006), se synchronisent dans `CATEGORIES`
sous leur chemin complet (`req~groupes-en-categories~1`), et le renommage
ou le déplacement d'un parent doit passer comme une seule opération, sans
décision de divergence par fiche (`req~renommage-groupe-operation-unique~1`).
`CATEGORIES` est à plat : une liste de libellés, sans identifiant ni
parent.

## Décision

Un groupe est **désigné par son chemin** : ses noms du plus général au plus
précis, séparés par `/` (`Travail/Clients/Client X`), chaque nom sans
espaces de bord ni espaces répétées. Le parent d'un groupe est son chemin
sans le dernier nom ; la hiérarchie se déduit des chemins et ne peut avoir
de cycle : un groupe ne peut être déplacé sous lui-même ou sous l'un de ses
descendants.

Une fiche appartient à ses groupes par les valeurs de son champ `Groupes`,
une par chemin ; la vCard les porte dans `CATEGORIES`, lu et écrit tel quel,
virgules échappées. Le référentiel tient en outre le registre des groupes
créés dans l'application, pour garder un groupe encore vide ; la liste des
groupes est ce registre, les chemins des fiches et leurs ancêtres.

Renommer ou déplacer un groupe réécrit, en une opération, son chemin et
ceux de ses sous-groupes, dans le registre et dans toutes les fiches. Pour
la fusion à trois voies, c'est une modification de l'application sur un
champ que la source n'a pas touché : elle s'écrit vers la source au tour
suivant, sans décision.

## Alternatives écartées

### Identifiant de groupe et parent en référence

Plus classique, mais il faudrait traduire chaque `CATEGORIES` lu en
identifiants (et créer des groupes pendant la lecture), et faire porter la
base de fusion par des identifiants que la source ignore ; le renommage
n'y gagnerait rien, puisque la source ne voit que des chemins.

## Conséquences

- Ce que cela apporte : une lecture et une écriture de `CATEGORIES` sans
  traduction ; une hiérarchie sans cycle par construction ; un renommage qui
  passe par le chemin ordinaire de la fusion.
- Ce que cela coûte : un nom de groupe ne peut contenir `/` ; renommer un
  groupe réécrit toutes ses fiches dans les sources.
- Ce que cela ferme : deux groupes de même chemin distincts.

## Vérification

- Tests du cœur : hiérarchie et refus du cycle, filtre (union,
  intersection, sous-groupes, recherche), `CATEGORIES` écrit et relu,
  renommage de « Travail » sur 40 fiches sans aucune décision.
- Revue : aucune opération sur les groupes n'écrit ailleurs que dans le
  registre et le champ `Groupes` des fiches.

## Références

- `req~appartenance-multiple~1`, `req~groupes-hierarchiques~1`,
  `req~filtre-par-groupes~2`, `req~groupes-en-categories~1`,
  `req~renommage-groupe-operation-unique~1` ; OQ-003, OQ-006 ; ToDo-0005.
- RFC 6350, § 6.7.1 (`CATEGORIES`).

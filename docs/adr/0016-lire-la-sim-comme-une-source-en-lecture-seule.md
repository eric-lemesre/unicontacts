# ADR-0016 — Lire la SIM par SimPhonebookContract, comme une source en lecture seule

**Statut** : Accepté
**Date** : 2026-11-15
**Décideur** : Eric Lemesre — lot 5 ouvert par D-2026-11-15, conception déléguée
**Portée** : phase (condition de réouverture : besoin d'écrire sur la SIM)

## Contexte

L'exigence `req~source-sim-lecture-seule~1` veut une carte SIM configurée
comme source, lue nom et numéro, sans jamais y écrire sauf réglage explicite.
Le cœur sait déjà rapprocher les contacts lus d'une source d'import en
lecture seule (ADR-0015). Android expose depuis la version 12 l'annuaire des
SIM par une interface publique, `SimPhonebookContract`, sous la seule
permission de lecture des contacts.

## Décision

La SIM est une **source d'import** du cœur, nommée « Carte SIM » : lecture
seule, suppressions ignorées. L'utilisateur l'ajoute depuis l'écran Sources ;
elle est ensuite **relue à chaque synchronisation** par l'adaptateur, avec
les sources CardDAV.

**Lecture** : les fichiers élémentaires `ADN` de `SimPhonebookContract`
donnent les abonnements présents ; chaque enregistrement en donne le nom et
le numéro, désignés par `sim-<abonnement>-<numéro d'enregistrement>`. Une
entrée devient une fiche par la détection des doublons, ou rejoint une fiche
existante.

**Absence de SIM** : si aucun annuaire `ADN` n'est lisible (SIM retirée,
verrouillée), la synchronisation de la source est sautée ; une lecture vide
ne vaut pas disparition de toutes les entrées.

**Écriture** : aucune. Le réglage en écriture n'est pas offert, et aucun
connecteur n'écrit sur la SIM : lever cette limite demandera un ADR.

## Alternatives écartées

### `content://icc/adn`

Fournisseur historique, hors de l'interface publique et sans garantie de
stabilité ; `SimPhonebookContract` le remplace.

### Import ponctuel de la SIM, comme le stockage local

L'exigence parle d'une source configurée : une entrée ajoutée sur la SIM
depuis un autre téléphone doit rejoindre le référentiel sans geste de plus.

## Conséquences

- Ce que cela apporte : la SIM suit le référentiel sans risque de
  troncature ; doublons et fusion sont ceux de toute source.
- Ce que cela coûte : un changement de SIM change l'abonnement, et ses
  entrées sont rapprochées à nouveau par la détection des doublons.
- Ce que cela ferme : l'écriture sur la SIM, jusqu'à un nouvel ADR.

## Vérification

- Tests du cœur : entrée SIM importée en fiche portant nom et numéro ;
  fiche liée modifiée sans aucune écriture vers la SIM.
- Tests JVM : traduction d'un enregistrement SIM en contact à importer.
- Procédure manuelle sur l'appareil : SIM ajoutée, synchronisée, puis
  fiche modifiée sans effet sur la SIM (`docs/verification-manuelle/android.md`).

## Références

- `req~source-sim-lecture-seule~1` ; ADR-0015 ; D-2026-11-15 ; ToDo-0006.

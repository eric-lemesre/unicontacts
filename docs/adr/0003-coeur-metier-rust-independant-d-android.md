# ADR-0003 — Écrire le cœur métier en Rust, indépendant d'Android, sur SQLite

**Statut** : Proposé
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur) ; à accepter à l'arbitrage des
domaines référentiel et synchronisation
**Portée** : durable

## Contexte

La valeur d'unicontacts est dans la fusion : comparaison à trois voies champ
par champ, file de décisions, détection des doublons, vCard 3.0 et 4.0 sans
perte. C'est la partie la plus délicate et la plus testable. La
spécification v0.1 (§ 2, principe 6, et § 3.1) la place dans un cœur Rust
testable sur un poste de développement, la couche Android n'étant qu'un
client de ce cœur.

## Décision

Le cœur métier — modèle de données, stockage, fusion, file de décisions,
doublons, vCard, connecteurs de sources et fournisseurs de photos derrière
des traits — est écrit en Rust, dans le workspace Cargo (`crates/`), sans
aucune dépendance à Android. Le stockage du référentiel est une base SQLite
locale au téléphone. La couche Android y accède par une interface étrangère
(crate `ffi` à venir, technologie selon OQ-001).

## Alternatives écartées

### Tout en Kotlin

Une seule pile côté Android, mais la logique de fusion ne se testerait
qu'avec l'outillage Android, plus lent, et resterait liée à la plateforme.

### Stockage dans les seuls contacts Android (`ContactsContract`)

Aucune base à tenir, mais ni provenance par champ, ni dernier état
synchronisé, ni propriétés inconnues par source : les principes 3 et 5 de la
spécification deviennent impossibles.

## Conséquences

- Ce que cela apporte : le cœur se teste en CI sur Linux, sans émulateur ;
  un cœur réutilisable hors Android.
- Ce que cela coûte : une frontière FFI à concevoir et à tenir ; deux
  langages dans le dépôt.
- Ce que cela ferme : rien d'irréversible tant qu'aucune version n'est
  publiée.

## Vérification

- `req~coeur-independant-android~1` : les tests du cœur tournent dans
  `tools/ci.sh` (étapes « rust — ») sur un runner Linux sans SDK Android.
- Revue : aucune dépendance `android`/`jni` dans `crates/core`.

## Références

- Spécification v0.1, § 2 et 3.1 (`spec/01-besoins/sources/`).
- `req~coeur-independant-android~1`, OQ-001.

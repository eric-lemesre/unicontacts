# ToDo-0002 — Lot 1 : cœur Rust

**Statut : ✅ soldé**

Modèle, stockage SQLite, vCard 3.0 et 4.0, fusion à trois voies et file de
décisions, testés sur des jeux de fichiers vCard (spécification v0.1, § 13).
C'est la partie la plus délicate du produit, et la seule testable entièrement
hors du téléphone. Lot ouvert le 2026-11-14 : ses 23 exigences sont
approuvées (D-2026-11-09).

## Reste à faire

- Rien : lot livré le 2026-11-14. Le crate `crates/core` réalise et teste
  les 23 exigences, toutes jugées conformes à l'audit ; jeux de vCard de
  référence dans `crates/core/tests/vcard/` ; critères des doublons actés
  (D-2026-11-10).
- Limites assumées, reportées à leur lot :
  - le moteur attend la liste complète des copies d'une source : la
    synchronisation incrémentale relève du connecteur (lot 2, ToDo-0003) ;
  - les numéros nationaux se rapportent à l'indicatif +33, faute de réglage
    du pays de l'utilisateur ;
  - le stockage réécrit le référentiel entier à chaque enregistrement, en
    une transaction : à revoir si la taille des carnets l'exige.

## Renvois

- Exigences (approuvées, D-2026-11-09) :
  - `req~fiche-maitre-identifiant-stable~1`
  - `req~provenance-par-champ~1`
  - `req~conservation-proprietes-inconnues~1`
  - `req~vcard-3-et-4~1`
  - `req~coeur-independant-android~1`
  - `req~reglages-par-source~1`
  - `req~autorite-par-champ~1`
  - `req~source-de-reference-unique~2`
  - `req~autorite-de-la-reference~2`
  - `req~fusion-trois-voies~1`
  - `req~modification-unilaterale-appliquee~1`
  - `req~divergence-soumise-a-decision~1`
  - `req~multivalues-par-valeur-normalisee~1`
  - `req~synchronisation-non-bloquante~1`
  - `req~suppression-sans-destruction~1`
  - `req~detection-doublons~1`
  - `req~traitement-des-doublons~1`
  - `req~contenu-decision~1`
  - `req~actions-de-resolution~1`
  - `req~report-decision~1`
  - `req~resolution-par-lot~1`
  - `req~issues-suppression-detectee~1`
  - `req~portee-suppression-locale~1`
- Décisions : ADR-0003, ADR-0005

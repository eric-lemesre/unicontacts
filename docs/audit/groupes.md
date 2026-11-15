# Audit — groupes

<!-- Miroir de spec/02-exigences/groupes.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette groupes -->

Miroir de [`spec/02-exigences/groupes.md`](../../spec/02-exigences/groupes.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Exigences du lot 4 (D-2026-11-13), jugées sur le code de `crates/core`,
`crates/ffi` et `android/`.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~appartenance-multiple~1` | conforme | champ `Groupes` multivalué, une valeur par chemin (`Referentiel::ranger`, `crates/core/src/groupes.rs`), normalisé et sans doublon ; tests `groupes::tests::appartenance` |
| `req~groupes-hierarchiques~1` | conforme | parent déduit du chemin (ADR-0012), `deplacer_groupe` refuse un parent descendant du groupe ; création, déplacement, suppression ; tests `groupes::tests::hierarchie` |
| `req~filtre-par-groupes~2` | conforme | union par défaut ou intersection, sous-groupes compris par défaut, combiné à la recherche sans casse ni accents (`Filtre`, `Referentiel::filtrer`, `Coeur::fiches_filtrees`, liste Android) ; tests `groupes::tests::filtre` |
| `req~groupes-en-categories~1` | conforme | `CATEGORIES` écrit avec le chemin complet, virgules échappées, relu et hiérarchie reconstruite par les ancêtres des chemins ; catégories inchangées rendues à l'octet près (`vcard/ecriture.rs`) ; tests `synchronisation::tests::groupes` |
| `req~renommage-groupe-operation-unique~1` | conforme | `renommer_groupe` réécrit registre et fiches en une opération ; la fusion y voit une modification de l'application : 40 fiches réécrites, aucune décision (test `renommer_un_parent_ne_cree_aucune_decision_et_reecrit_ses_quarante_fiches`) |

## Écarts ouverts

- Aucun.

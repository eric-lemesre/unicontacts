# Audit — file-decisions

<!-- Miroir de spec/02-exigences/file-decisions.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette file-decisions -->

Miroir de [`spec/02-exigences/file-decisions.md`](../../spec/02-exigences/file-decisions.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Exigences du lot 1 (D-2026-11-09), jugées sur le code et les tests du crate
`crates/core` (ToDo-0002). Les exigences encore proposées n'ont pas de
ligne : elles ne se jugent pas avant leur lot.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~contenu-decision~1` | conforme | `crates/core/src/decisions.rs` : `Decision` porte nature (divergence de champ, suppression, doublon probable, nouvelle photo), versions référentiel/source/base, source, date de création et statut ; test du critère |
| `req~actions-de-resolution~1` | conforme | `Decision::choix` n'offre « garder les deux » que pour un champ multivalué, `admet` le refuse sinon ; `Referentiel::resoudre` applique l'action et aligne la base sur la source vue ; tests des deux critères, la valeur saisie étant écrite à la synchronisation suivante |
| `req~report-decision~1` | conforme | `Referentiel::reporter` passe la décision « reportée » sans rien modifier ; tests (statut, valeurs et versions inchangées) |
| `req~resolution-par-lot~1` | conforme | `cas_similaires` annonce les décisions en attente de même nature, champ et source (D-2026-11-10), `resoudre_avec_cas_similaires` leur applique la même résolution ; test du critère (cinq divergences sur l'organisation) |
| `req~issues-suppression-detectee~1` | conforme | `Decision::choix` offre propager, restaurer, détacher ; `Referentiel::resoudre` les applique (restaurer réécrit la fiche à la synchronisation suivante, propriétés inconnues comprises) ; tests des deux critères |
| `req~portee-suppression-locale~1` | conforme | `Referentiel::sources_de_suppression` ne propose que les sources liées autorisées en écriture, `supprimer_fiche` refuse toute autre ; les copies non choisies ne sont plus importées ; test du critère |

## Écarts ouverts

- Aucun.

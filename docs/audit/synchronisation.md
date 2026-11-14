# Audit — synchronisation

<!-- Miroir de spec/02-exigences/synchronisation.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette synchronisation -->

Miroir de [`spec/02-exigences/synchronisation.md`](../../spec/02-exigences/synchronisation.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Lot 1 ouvert le 2026-11-14 (D-2026-11-09) : chaque exigence approuvée est
un écart tant que son lot n'a rien livré. Les exigences encore proposées
n'ont pas de ligne : elles ne se jugent pas avant leur lot.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~reglages-par-source~1` | écart | à réaliser au lot 1 (ToDo-0002) : aucun code ni test livré |
| `req~autorite-par-champ~1` | conforme | `Sources::qui_fait_foi` : la source désignée pour le champ, à défaut la référence générale ; `fusion.rs` : la source gagnante impose sa valeur ; tests des deux critères (Google pour les téléphones, Nextcloud pour les adresses) |
| `req~source-de-reference-unique~2` | conforme | `crates/core/src/reglages.rs` : `Sources::regler` et `designer_pour_champ` refusent une seconde référence au même niveau, l'erreur nomme la source en place ; tests des deux critères |
| `req~autorite-de-la-reference~2` | conforme | `Sources::gagnant` : la source qui fait foi pour le champ l'emporte si ses conflits sont réglés sur « l'autorité gagne », jamais la référence générale sur le champ d'une autre ; `fusionner` retient alors sa valeur sans divergence ; tests des deux critères |
| `req~fusion-trois-voies~1` | conforme | `crates/core/src/fusion.rs` : `fusionner` compare référentiel, source et base par clés normalisées, champ par champ ; test du critère (téléphone côté source, adresse côté référentiel, sans divergence) |
| `req~modification-unilaterale-appliquee~1` | conforme | `fusionner` : un seul côté changé s'applique à l'autre, un même changement des deux côtés n'écrit rien ; tests des deux critères |
| `req~divergence-soumise-a-decision~1` | écart | à réaliser au lot 1 (ToDo-0002) : aucun code ni test livré |
| `req~multivalues-par-valeur-normalisee~1` | conforme | `normalisation.rs` (clés) et `fusion.rs` (combinaison valeur par valeur ; divergence seulement pour une valeur remplacée différemment des deux côtés) ; tests des deux critères |
| `req~synchronisation-non-bloquante~1` | écart | à réaliser au lot 1 (ToDo-0002) : aucun code ni test livré |
| `req~suppression-sans-destruction~1` | écart | à réaliser au lot 1 (ToDo-0002) : aucun code ni test livré |
| `req~detection-doublons~1` | écart | à réaliser au lot 1 (ToDo-0002) : aucun code ni test livré |
| `req~traitement-des-doublons~1` | écart | à réaliser au lot 1 (ToDo-0002) : aucun code ni test livré |

## Écarts ouverts

- Exigences du lot 1 : à réaliser (ToDo-0002).

# Audit — referentiel

<!-- Miroir de spec/02-exigences/referentiel.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette referentiel -->

Miroir de [`spec/02-exigences/referentiel.md`](../../spec/02-exigences/referentiel.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Lot 1 ouvert le 2026-11-14 (D-2026-11-09) : chaque exigence approuvée est
un écart tant que son lot n'a rien livré. Les exigences encore proposées
n'ont pas de ligne : elles ne se jugent pas avant leur lot.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~fiche-maitre-identifiant-stable~1` | conforme | `crates/core/src/identifiants.rs` : UUID v4 tiré à la création, sans lien avec les valeurs ni les sources ; `fiche.rs` : `detacher` ne touche pas l'identifiant ; tests des deux critères (détachement, mêmes valeurs) et de la modification |
| `req~provenance-par-champ~1` | conforme | `crates/core/src/champ.rs` : chaque `Valeur` porte sa `Provenance` (origine, date) ; `Fiche::modifier` date au référentiel les seules valeurs nouvelles ; tests des deux critères |
| `req~conservation-proprietes-inconnues~1` | conforme | `Carte::reecrire` reprend à leur place, lignes repliées comprises, les propriétés non interprétées de la dernière carte connue de la source (`Lien::carte`), et seulement d'elle ; tests des deux critères (rendue à l'octet près après modification, absente vers une autre source) |
| `req~vcard-3-et-4~1` | conforme | `crates/core/src/vcard/` : lecture (dépliage, groupes, paramètres entre guillemets ou sans nom) et écriture (repliement à 75 octets) des versions 3.0 et 4.0 ; `Carte::reecrire` écrit dans la version de la source ; `tests/vcard_reference.rs` relit et réécrit les jeux 3.0 et 4.0 de `tests/vcard/`, équivalents propriété par propriété et identiques à l'octet près |
| `req~coeur-independant-android~1` | conforme | `crates/core` ne dépend que de `thiserror` et `uuid` ; ses tests tournent en CI sur Linux (`sh tools/rust.sh test`), sans SDK Android |

## Écarts ouverts

- Exigences du lot 1 : à réaliser (ToDo-0002).

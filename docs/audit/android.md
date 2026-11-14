# Audit — android

<!-- Miroir de spec/02-exigences/android.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette android -->

Miroir de [`spec/02-exigences/android.md`](../../spec/02-exigences/android.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Exigences du lot 3 (D-2026-11-12), jugées sur le code de `crates/ffi` et
`android/` et sur l'exécution des procédures de
`docs/verification-manuelle/android.md`.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~compte-systeme-dedie~1` | écart | à réaliser au lot 3 (ToDo-0004) : aucun code ni test livré |
| `req~ecriture-sous-compte-unique~1` | écart | à réaliser au lot 3 (ToDo-0004) : aucun code ni test livré |
| `req~synchronisation-directe~1` | écart | à réaliser au lot 3 (ToDo-0004) : aucun code ni test livré |

## Écarts ouverts

- Exigences du lot 3 : à réaliser (ToDo-0004).

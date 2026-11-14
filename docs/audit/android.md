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
| `req~compte-systeme-dedie~1` | écart | réalisée, vérification sur l'appareil à exécuter : `compte/` (authentificateur, compte créé au lancement, synchronisable et périodique) ; `mtest~compte-unicontacts-visible~1` |
| `req~ecriture-sous-compte-unique~1` | écart | réalisée, vérification sur l'appareil à exécuter : `synchro/EcrivainContacts.kt` : un contact brut par fiche (`SOURCE_ID` = identifiant de la fiche), sous le seul compte, en lecture seule ; traduction testée sur la JVM (`LignesContactsTest`) ; `mtest~un-seul-bloc-dans-contacts~1` |
| `req~synchronisation-directe~1` | écart | réalisée, vérification sur l'appareil à exécuter : `synchro/AdaptateurSynchro.kt` appelle `synchroniser_carddav` du crate `ffi` pour chaque source (connecteur éprouvé contre Radicale) ; mots de passe au Keystore (ADR-0010) ; `mtest~synchronisation-directe-sur-l-appareil~1` |

## Écarts ouverts

- Exigences du lot 3 : réalisées, à vérifier sur le Fairphone (`docs/verification-manuelle/android.md`, ToDo-0004).

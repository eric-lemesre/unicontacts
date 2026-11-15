# Audit — canaux

<!-- Miroir de spec/02-exigences/canaux.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette canaux -->

Miroir de [`spec/02-exigences/canaux.md`](../../spec/02-exigences/canaux.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Exigences du lot 4 (D-2026-11-13), jugées sur le code de `crates/core`,
`crates/ffi` et `android/`, et sur l'exécution des procédures de
`docs/verification-manuelle/android.md`.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~canaux-par-identifiant~1` | conforme | appel et SMS par numéro, e-mail par adresse ; messageries rattachées à un numéro, une adresse ou un pseudonyme, refusées sur un numéro absent, disparues avec lui (`crates/core/src/canaux.rs`, ADR-0014) ; tests `canaux::tests::identifiants` |
| `req~classements-pro-perso~2` | conforme | classements pro et perso par clé de canal, limités aux canaux disponibles, un seul renseigné valant pour les deux (`Referentiel::classement`, `canal_prefere`) ; tests `canaux::tests::classements`, test du pont |
| `req~note-par-canal~1` | conforme | note libre par clé de canal, retirée si vide, présentée sous le bouton du canal ; tests `canaux::tests::notes` |
| `req~detection-assistee-canaux~1` | écart | réalisée, vérification sur l'appareil à exécuter : `LecteurMessageries` lit les contacts que WhatsApp, Signal et Telegram déclarent, `DetectionCanaux` les rapproche des numéros (tests JVM `CanauxTest`), suggestion ajoutée seulement sur « Ajouter » ; `mtest~detection-messageries-sur-l-appareil~1` |
| `req~actions-triees-par-preference~2` | écart | réalisée, vérification sur l'appareil à exécuter : un bouton par canal, préférés pro et perso marqués, adresse d'action par canal (`ActionsCanaux`, tests JVM `CanauxTest`), sans mode global ; `mtest~actions-canaux-sur-l-appareil~1` |
| `req~filtre-par-canal~1` | conforme | `Filtre::canal`, appliqué par `Referentiel::filtrer` sur les canaux disponibles, choix du canal dans le filtre de la liste ; tests `canaux::tests::filtre` |

## Écarts ouverts

- `req~detection-assistee-canaux~1`, `req~actions-triees-par-preference~2` :
  réalisées, à vérifier sur le Fairphone (`docs/verification-manuelle/android.md`,
  ToDo-0005).

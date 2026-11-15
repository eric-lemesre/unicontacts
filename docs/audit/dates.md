# Audit — dates

<!-- Miroir de spec/02-exigences/dates.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette dates -->

Miroir de [`spec/02-exigences/dates.md`](../../spec/02-exigences/dates.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Exigences du lot 4 (D-2026-11-13), jugées sur le code de `crates/core`,
`crates/ffi` et `android/`, et sur l'exécution des procédures de
`docs/verification-manuelle/android.md`.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~types-de-dates~1` | conforme | naissance (`BDAY`), mariage, fête et dates personnalisées (champ `Dates`), année facultative, saisie validée (`crates/core/src/dates.rs`) ; un anniversaire sans année se relit sans année ; tests `dates::tests::types` |
| `req~fete-du-prenom-proposee~1` | conforme | jours de fête proposés d'après le prénom (nom structuré, sinon nom affiché) quand la fiche n'en a pas, acceptés sur la fiche ; calendrier Wikidata vérifié par empreinte (ADR-0013) ; tests `dates::tests::fetes` |
| `req~vue-a-venir~1` | écart | réalisée, vérification sur l'appareil à exécuter : `Referentiel::a_venir` (ordre, délai, années, filtre de groupes ; tests `dates::tests::a_venir`), écran « À venir » ; `mtest~a-venir-sur-l-appareil~1` |
| `req~synchronisation-dates~1` | conforme | `BDAY`, `ANNIVERSARY` en 4.0, `X-ABDATE` étiquetées en 3.0 et pour fête et dates personnalisées, année omise selon la version (`vcard/dates.rs`, ADR-0011) ; champ « Événement » d'Android (`LignesContacts.evenement`) ; tests `synchronisation::tests::dates`, `LignesContactsTest` |
| `req~dates-dans-l-agenda~1` | écart | réalisée, vérification sur l'appareil à exécuter : `agenda/AgendaDates.kt` (événements annuels récurrents dans l'agenda choisi, créés et retirés à chaque synchronisation), plan éprouvé sur la JVM (`EvenementsAgendaTest`) ; `mtest~dates-dans-l-agenda-sur-l-appareil~1` |

## Écarts ouverts

- `req~vue-a-venir~1`, `req~dates-dans-l-agenda~1` : réalisées, à vérifier
  sur le Fairphone (`docs/verification-manuelle/android.md`, ToDo-0005).

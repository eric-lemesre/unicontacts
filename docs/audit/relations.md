# Audit — relations

<!-- Miroir de spec/02-exigences/relations.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette relations -->

Miroir de [`spec/02-exigences/relations.md`](../../spec/02-exigences/relations.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Exigences du lot 4 (D-2026-11-13), jugées sur le code de `crates/core`,
`crates/ffi` et `android/`.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~fiche-moi~1` | conforme | une seule fiche « moi » (`Referentiel::designer_moi`, `crates/core/src/relations.rs`), oubliée quand elle est supprimée, enregistrée (table `moi`) ; tests `relations::tests::fiche_moi` |
| `req~liens-types~1` | conforme | types prédéfinis et personnalisés (`TypeRelation`), lien entre deux fiches quelconques, sans doublon ni lien sur soi ; liens vers une fiche supprimée retirés ; tests `relations::tests::liens` |
| `req~reciproques-automatiques~1` | conforme | inverses fixés (parent ↔ enfant, conjoint…) et déclarés pour les types sans inverse fixé (`Inverses`), réciproque créée et retirée avec le lien ; aucune pour un type sans inverse ; tests `relations::tests::reciproques` |
| `req~lien-par-identifiant-maitre~1` | conforme | valeur `type;identifiant` du champ `Relation`, jamais le nom ; lien intact au renommage, dans le référentiel comme à travers une source 3.0 (mémoire des écritures, ADR-0011) ; tests `relations::tests::identifiant_maitre`, `synchronisation::tests::relations` |
| `req~section-proches~1` | écart | réalisée, vérification sur l'appareil à exécuter : `Referentiel::proches`, `Coeur::proches`, section en tête de `EcranListe` ; `mtest~section-proches-sur-l-appareil~1` |
| `req~synchronisation-relations~1` | conforme | `RELATED` en 4.0 (UID de la copie, sinon nom), convention Apple en 3.0, relation impossible à porter gardée au référentiel sans être réputée retirée, propriété non résolue conservée (`vcard/relations.rs`, `synchronisation.rs`, ADR-0011) ; champ « Relation » d'Android (`LignesContacts.relation`) ; tests `synchronisation::tests::relations`, `LignesContactsTest` |

## Écarts ouverts

- `req~section-proches~1` : réalisée, à vérifier sur le Fairphone
  (`docs/verification-manuelle/android.md`, ToDo-0005).

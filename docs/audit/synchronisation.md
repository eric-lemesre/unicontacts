# Audit — synchronisation

<!-- Miroir de spec/02-exigences/synchronisation.md. Les verdicts sont écrits À LA MAIN :
     c'est le jugement que la porte ne sait pas rendre. Le tableau de bord
     docs/audit/README.md, lui, est généré — ne pas l'éditer.
     Squelette des lignes manquantes :
       sh tools/audit.sh --squelette synchronisation -->

Miroir de [`spec/02-exigences/synchronisation.md`](../../spec/02-exigences/synchronisation.md).
Verdicts admis : `conforme` · `écart` · `non examiné`.

Exigences des lots 1 (D-2026-11-09), 2 (D-2026-11-11) et 5 (D-2026-11-15), jugées sur le code
et les tests des crates `crates/core` (ToDo-0002) et `crates/carddav`
(ToDo-0003). Les exigences encore proposées n'ont pas de ligne : elles ne se
jugent pas avant leur lot.

| Exigence | Verdict | Preuve / écart |
| --- | --- | --- |
| `req~source-carddav~1` | conforme | `crates/carddav` : REPORT addressbook-query, PUT sous `If-Match` (ou `If-None-Match: *`), DELETE ; carte désignée par son UID ; écriture refusée annulée dans le référentiel (`annuler_ecriture`), suppression reprogrammée ; mot de passe masqué au `Debug`, URL à identifiants refusée ; `tests/radicale.rs` (`sh tools/radicale.sh test`, Radicale 3.9.1 épinglé, ADR-0008) éprouve les deux critères, la conservation des propriétés inconnues, la modification concurrente et les suppressions |
| `req~reglages-par-source~1` | conforme | `reglages.rs` : `Reglages::default()` porte les quatre valeurs par défaut, `Sources::ajouter` les applique ; `synchronisation.rs` n'écrit vers une source que si sa direction le permet (lecture seule : base alignée sur la source, écriture seule : recouverte) ; tests des deux critères |
| `req~autorite-par-champ~1` | conforme | `Sources::qui_fait_foi` : la source désignée pour le champ, à défaut la référence générale ; `fusion.rs` : la source gagnante impose sa valeur ; tests des deux critères (Google pour les téléphones, Nextcloud pour les adresses) |
| `req~source-de-reference-unique~2` | conforme | `crates/core/src/reglages.rs` : `Sources::regler` et `designer_pour_champ` refusent une seconde référence au même niveau, l'erreur nomme la source en place ; tests des deux critères |
| `req~autorite-de-la-reference~2` | conforme | `Sources::gagnant` : la source qui fait foi pour le champ l'emporte si ses conflits sont réglés sur « l'autorité gagne », jamais la référence générale sur le champ d'une autre ; `fusionner` retient alors sa valeur sans divergence ; tests des deux critères |
| `req~fusion-trois-voies~1` | conforme | `crates/core/src/fusion.rs` : `fusionner` compare référentiel, source et base par clés normalisées, champ par champ ; test du critère (téléphone côté source, adresse côté référentiel, sans divergence) |
| `req~modification-unilaterale-appliquee~1` | conforme | `fusionner` : un seul côté changé s'applique à l'autre, un même changement des deux côtés n'écrit rien ; tests des deux critères |
| `req~divergence-soumise-a-decision~1` | conforme | `fusionner` signale la divergence sans gagnant et garde la valeur du référentiel ; `Referentiel::synchroniser` soumet une décision de divergence (mise à jour, jamais dupliquée ; close quand l'accord revient) ; test du critère au niveau du moteur |
| `req~multivalues-par-valeur-normalisee~1` | conforme | `normalisation.rs` (clés) et `fusion.rs` (combinaison valeur par valeur ; divergence seulement pour une valeur remplacée différemment des deux côtés) ; tests des deux critères |
| `req~synchronisation-non-bloquante~1` | conforme | `Referentiel::synchroniser` fusionne chaque champ et chaque fiche indépendamment des décisions ouvertes ; test du critère (nom en attente, téléphone de la même fiche et autre fiche appliqués) |
| `req~suppression-sans-destruction~1` | conforme | `traiter_disparition` : sur « demander », la fiche et son lien restent intacts et une décision de suppression est soumise ; une copie illisible ne vaut pas suppression ; test du critère |
| `req~detection-doublons~1` | conforme | `crates/core/src/doublons.rs` : téléphone et e-mail par clés normalisées, nom plié (casse, accents, ponctuation, ordre des mots) ; certain ou probable selon D-2026-11-10 ; tests des deux critères, dont l'import de même e-mail |
| `req~traitement-des-doublons~1` | conforme | `Referentiel::synchroniser` (import) : un doublon certain se rattache à la fiche existante et s'y fusionne, un probable donne une seconde fiche et une décision ; tests des deux critères |
| `req~import-stockage-local~1` | écart | réalisée : `crates/core/src/import.rs` (source d'import en lecture seule, suppressions ignorées ; copie vCard 4.0 rapprochée par `synchroniser`), pont `importer` et `copies_liees`, `android/…/stockage/` et `SectionStockageLocal.kt` ; tests du cœur des trois critères (doublons, import rejoué, originaux retirés : fiches intactes et détachées), test du pont après réouverture, tests JVM de la traduction ; lecture et retrait dans le vrai stockage local à vérifier sur le Fairphone (`mtest~import-stockage-local-sur-l-appareil~1`) |
| `req~source-sim-lecture-seule~1` | écart | réalisée : source d'import en lecture seule (`ajouter_source_import`), `android/…/stockage/CarteSim.kt` (`SimPhonebookContract`, ADN de chaque abonnement, source sautée sans annuaire lisible), relecture par `AdaptateurSynchro`, aucun code d'écriture ; tests du cœur des deux critères (entrée en fiche nom et numéro, fiche modifiée sans écriture), tests JVM de la traduction ; lecture de la vraie SIM à vérifier sur le Fairphone (`mtest~sim-en-lecture-seule-sur-l-appareil~1`) |

## Écarts ouverts

- `req~import-stockage-local~1` : procédure sur l'appareil à exécuter (ToDo-0006).
- `req~source-sim-lecture-seule~1` : procédure sur l'appareil à exécuter (ToDo-0006).

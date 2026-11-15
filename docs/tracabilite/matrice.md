# Matrice de traçabilité

<!-- GÉNÉRÉ par tools/matrice.sh — ne pas éditer à la main (R8). -->

Vue de synthèse, informative : elle n'ajoute aucun contrôle à `task trace` (OpenFastTrace) ni à `task ci` (`tools/classification.sh`) et ne peut jamais faire échouer une porte. Section 1 : chaque exigence des spécifications, avec ses couvertures RÉELLEMENT trouvées par balayage du dépôt (pas seulement les `Needs:` déclarés). Section 2 : son inverse, les fichiers d'outillage sans aucune trace de traçabilité — une liste de revue, pas un verdict.

## 1. Exigences → couvertures

### spec/02-exigences/android.md

#### `req~compte-systeme-dedie~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles (8) :
  - `impl` — android/app/build.gradle.kts:2
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/UnicontactsApp.kt:3
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/compte/Authentificateur.kt:4
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/compte/Compte.kt:3
  - `impl` — android/build.gradle.kts:2
  - `impl` — android/settings.gradle.kts:2
  - `impl` — tools/android.sh:3
  - `impl` — tools/bootstrap.d/70-android.sh:3

#### `req~ecriture-sous-compte-unique~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles (5) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/synchro/EcrivainContacts.kt:5
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/synchro/LignesContacts.kt:4
  - `impl` — android/app/src/test/java/fr/lemesre/unicontacts/synchro/LignesContactsTest.kt:2
  - `impl` — crates/ffi/src/lib.rs:11
  - `impl` — crates/ffi/src/lib.rs:127

#### `req~synchronisation-directe~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles (8) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/sources/Coffre.kt:3
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/sources/ConfigurationsSources.kt:4
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/synchro/AdaptateurSynchro.kt:4
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranSources.kt:3
  - `impl` — android/app/src/test/java/fr/lemesre/unicontacts/sources/ConfigurationsSourcesTest.kt:2
  - `impl` — crates/ffi/src/bin/uniffi-bindgen.rs:4
  - `impl` — crates/ffi/src/lib.rs:10
  - `impl` — crates/ffi/src/lib.rs:572

### spec/02-exigences/canaux.md

#### `req~canaux-par-identifiant~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~classements-pro-perso~2`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~note-par-canal~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~detection-assistee-canaux~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~actions-triees-par-preference~2`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~filtre-par-canal~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

### spec/02-exigences/dates.md

#### `req~types-de-dates~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~fete-du-prenom-proposee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~vue-a-venir~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~synchronisation-dates~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~dates-dans-l-agenda~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

### spec/02-exigences/file-decisions.md

#### `req~contenu-decision~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (2) :
  - `impl` — crates/core/src/decisions.rs:120
  - `utest` — crates/core/src/decisions.rs:234

#### `req~boite-de-decisions~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles (5) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/ActivitePrincipale.kt:3
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcransDecisions.kt:3
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcransFiches.kt:4
  - `impl` — crates/ffi/src/lib.rs:481
  - `impl` — crates/ffi/src/lib.rs:493

#### `req~vue-comparee~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles (3) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcransDecisions.kt:4
  - `impl` — crates/ffi/src/lib.rs:494
  - `impl` — crates/ffi/src/modele.rs:2

#### `req~actions-de-resolution~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (6) :
  - `impl` — crates/core/src/decisions.rs:170
  - `utest` — crates/core/src/decisions.rs:261
  - `utest` — crates/core/src/decisions.rs:279
  - `impl` — crates/core/src/referentiel.rs:276
  - `utest` — crates/core/src/referentiel/tests.rs:141
  - `utest` — crates/core/src/synchronisation/tests.rs:344

#### `req~report-decision~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (3) :
  - `impl` — crates/core/src/referentiel.rs:195
  - `utest` — crates/core/src/referentiel/tests.rs:91
  - `utest` — crates/core/src/referentiel/tests.rs:109

#### `req~resolution-par-lot~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/referentiel.rs:211
  - `impl` — crates/core/src/referentiel.rs:258
  - `utest` — crates/core/src/referentiel/tests.rs:304
  - `utest` — crates/core/src/referentiel/tests.rs:314

#### `req~issues-suppression-detectee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (6) :
  - `impl` — crates/core/src/decisions.rs:171
  - `utest` — crates/core/src/decisions.rs:298
  - `impl` — crates/core/src/referentiel.rs:327
  - `utest` — crates/core/src/referentiel/tests.rs:362
  - `impl` — crates/core/src/synchronisation.rs:428
  - `utest` — crates/core/src/synchronisation/tests.rs:403

#### `req~portee-suppression-locale~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (3) :
  - `impl` — crates/core/src/referentiel.rs:417
  - `impl` — crates/core/src/referentiel.rs:442
  - `utest` — crates/core/src/referentiel/tests.rs:451

### spec/02-exigences/groupes.md

#### `req~appartenance-multiple~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranGroupes.kt:4
  - `impl` — crates/core/src/groupes.rs:9
  - `impl` — crates/core/src/groupes.rs:216
  - `utest` — crates/core/src/groupes/tests.rs:59
  - `impl` — crates/ffi/src/lib.rs:331

#### `req~groupes-hierarchiques~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (9) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranGroupes.kt:5
  - `impl` — crates/core/src/groupes.rs:10
  - `impl` — crates/core/src/groupes.rs:188
  - `impl` — crates/core/src/groupes.rs:260
  - `impl` — crates/core/src/groupes.rs:318
  - `utest` — crates/core/src/groupes/tests.rs:89
  - `utest` — crates/core/src/groupes/tests.rs:107
  - `impl` — crates/ffi/src/lib.rs:314
  - `impl` — crates/ffi/src/lib.rs:364

#### `req~filtre-par-groupes~2`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (9) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranGroupes.kt:6
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcransFiches.kt:6
  - `impl` — crates/core/src/groupes.rs:126
  - `impl` — crates/core/src/groupes.rs:355
  - `utest` — crates/core/src/groupes/tests.rs:166
  - `utest` — crates/core/src/groupes/tests.rs:178
  - `utest` — crates/core/src/groupes/tests.rs:191
  - `utest` — crates/core/src/groupes/tests.rs:204
  - `impl` — crates/ffi/src/lib.rs:395

#### `req~groupes-en-categories~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `utest` — crates/core/src/synchronisation/tests.rs:933
  - `utest` — crates/core/src/synchronisation/tests.rs:951
  - `impl` — crates/core/src/vcard/ecriture.rs:196
  - `impl` — crates/core/src/vcard/ecriture.rs:311
  - `impl` — crates/core/src/vcard/ecriture.rs:339

#### `req~renommage-groupe-operation-unique~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranGroupes.kt:7
  - `impl` — crates/core/src/groupes.rs:259
  - `utest` — crates/core/src/groupes/tests.rs:230
  - `utest` — crates/core/src/synchronisation/tests.rs:996
  - `impl` — crates/ffi/src/lib.rs:348

### spec/02-exigences/photos.md

#### `req~fournisseurs-desactives-par-defaut~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~recuperation-a-la-demande~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~lancement-recuperation~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~revue-des-photos-recuperees~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~photos-candidates~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~photo-poussee-redimensionnee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~photo-manuelle~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

### spec/02-exigences/referentiel.md

#### `req~fiche-maitre-identifiant-stable~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (9) :
  - `impl` — crates/core/src/fiche.rs:75
  - `impl` — crates/core/src/fiche.rs:206
  - `utest` — crates/core/src/fiche.rs:224
  - `utest` — crates/core/src/fiche.rs:238
  - `utest` — crates/core/src/fiche.rs:250
  - `impl` — crates/core/src/identifiants.rs:52
  - `utest` — crates/core/src/identifiants.rs:70
  - `impl` — crates/core/src/stockage.rs:219
  - `utest` — crates/core/src/stockage/tests.rs:147

#### `req~provenance-par-champ~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (8) :
  - `impl` — crates/core/src/champ.rs:111
  - `impl` — crates/core/src/fiche.rs:128
  - `impl` — crates/core/src/fiche.rs:161
  - `utest` — crates/core/src/fiche.rs:266
  - `utest` — crates/core/src/fiche.rs:290
  - `impl` — crates/core/src/stockage.rs:220
  - `utest` — crates/core/src/stockage/tests.rs:148
  - `impl` — crates/core/src/temps.rs:11

#### `req~conservation-proprietes-inconnues~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (6) :
  - `impl` — crates/core/src/stockage.rs:221
  - `utest` — crates/core/src/stockage/tests.rs:149
  - `impl` — crates/core/src/vcard/ecriture.rs:175
  - `impl` — crates/core/src/vcard/ecriture.rs:193
  - `utest` — crates/core/src/vcard/tests.rs:363
  - `utest` — crates/core/src/vcard/tests.rs:381

#### `req~vcard-3-et-4~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (8) :
  - `impl` — crates/core/src/vcard/ecriture.rs:150
  - `impl` — crates/core/src/vcard/ecriture.rs:176
  - `impl` — crates/core/src/vcard/ecriture.rs:194
  - `impl` — crates/core/src/vcard/lecture.rs:25
  - `impl` — crates/core/src/vcard/mod.rs:8
  - `utest` — crates/core/src/vcard/tests.rs:249
  - `utest` — crates/core/tests/vcard_reference.rs:35
  - `utest` — crates/core/tests/vcard_reference.rs:55

#### `req~coeur-independant-android~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (2) :
  - `impl` — crates/core/src/lib.rs:8
  - `utest` — crates/core/src/lib.rs:40

### spec/02-exigences/relations.md

#### `req~fiche-moi~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranRelations.kt:4
  - `impl` — crates/core/src/relations.rs:247
  - `utest` — crates/core/src/relations/tests.rs:31
  - `utest` — crates/core/src/relations/tests.rs:46
  - `impl` — crates/ffi/src/lib.rs:241

#### `req~liens-types~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (6) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranRelations.kt:5
  - `impl` — crates/core/src/relations.rs:8
  - `impl` — crates/core/src/relations.rs:305
  - `utest` — crates/core/src/relations/tests.rs:74
  - `utest` — crates/core/src/relations/tests.rs:94
  - `impl` — crates/ffi/src/lib.rs:205

#### `req~reciproques-automatiques~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (11) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcranRelations.kt:6
  - `impl` — crates/core/src/relations.rs:100
  - `impl` — crates/core/src/relations.rs:195
  - `impl` — crates/core/src/relations.rs:306
  - `impl` — crates/core/src/relations.rs:328
  - `utest` — crates/core/src/relations/tests.rs:179
  - `utest` — crates/core/src/relations/tests.rs:199
  - `utest` — crates/core/src/relations/tests.rs:219
  - `utest` — crates/core/src/relations/tests.rs:238
  - `impl` — crates/ffi/src/lib.rs:206
  - `impl` — crates/ffi/src/lib.rs:283

#### `req~lien-par-identifiant-maitre~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/relations.rs:9
  - `utest` — crates/core/src/relations/tests.rs:315
  - `utest` — crates/core/src/relations/tests.rs:335
  - `utest` — crates/core/src/synchronisation/tests.rs:853

#### `req~section-proches~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles (3) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/ui/EcransFiches.kt:5
  - `impl` — crates/core/src/relations.rs:283
  - `impl` — crates/ffi/src/lib.rs:256

#### `req~synchronisation-relations~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (14) :
  - `impl` — android/app/src/main/java/fr/lemesre/unicontacts/synchro/LignesContacts.kt:5
  - `utest` — android/app/src/test/java/fr/lemesre/unicontacts/synchro/LignesContactsTest.kt:3
  - `impl` — crates/core/src/synchronisation.rs:516
  - `utest` — crates/core/src/synchronisation/tests.rs:723
  - `utest` — crates/core/src/synchronisation/tests.rs:747
  - `utest` — crates/core/src/synchronisation/tests.rs:768
  - `utest` — crates/core/src/synchronisation/tests.rs:799
  - `utest` — crates/core/src/synchronisation/tests.rs:820
  - `utest` — crates/core/src/synchronisation/tests.rs:852
  - `utest` — crates/core/src/synchronisation/tests.rs:886
  - `utest` — crates/core/src/synchronisation/tests.rs:909
  - `impl` — crates/core/src/vcard/ecriture.rs:195
  - `impl` — crates/core/src/vcard/relations.rs:3
  - `impl` — crates/ffi/src/modele.rs:443

### spec/02-exigences/synchronisation.md

#### `req~source-carddav~1`

- Niveau normatif : DOIT
- Needs: impl, itest
- Couvertures réelles (19) :
  - `impl` — crates/carddav/src/client.rs:81
  - `impl` — crates/carddav/src/client.rs:146
  - `impl` — crates/carddav/src/client.rs:163
  - `impl` — crates/carddav/src/client.rs:189
  - `impl` — crates/carddav/src/lib.rs:11
  - `impl` — crates/carddav/src/multistatus.rs:53
  - `impl` — crates/carddav/src/synchro.rs:89
  - `impl` — crates/carddav/src/synchro.rs:118
  - `impl` — crates/carddav/src/synchro.rs:176
  - `impl` — crates/carddav/src/synchro.rs:223
  - `itest` — crates/carddav/src/synchro/tests.rs:4
  - `itest` — crates/carddav/tests/radicale.rs:176
  - `itest` — crates/carddav/tests/radicale.rs:212
  - `itest` — crates/carddav/tests/radicale.rs:237
  - `itest` — crates/carddav/tests/radicale.rs:296
  - `itest` — crates/carddav/tests/radicale.rs:324
  - `itest` — crates/carddav/tests/radicale.rs:348
  - `itest` — tools/bootstrap.d/60-python.sh:3
  - `itest` — tools/radicale.sh:3

#### `req~source-google~1`

- Niveau normatif : DOIT
- Needs: impl, itest
- Couvertures réelles : — À COUVRIR

#### `req~import-stockage-local~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~source-sim-lecture-seule~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~reglages-par-source~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `impl` — crates/core/src/reglages.rs:56
  - `impl` — crates/core/src/reglages.rs:133
  - `utest` — crates/core/src/reglages.rs:301
  - `impl` — crates/core/src/synchronisation.rs:249
  - `utest` — crates/core/src/synchronisation/tests.rs:244

#### `req~autorite-par-champ~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `utest` — crates/core/src/fusion.rs:453
  - `impl` — crates/core/src/reglages.rs:207
  - `impl` — crates/core/src/reglages.rs:233
  - `utest` — crates/core/src/reglages.rs:420
  - `utest` — crates/core/src/reglages.rs:432

#### `req~source-de-reference-unique~2`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `impl` — crates/core/src/reglages.rs:179
  - `impl` — crates/core/src/reglages.rs:208
  - `utest` — crates/core/src/reglages.rs:335
  - `utest` — crates/core/src/reglages.rs:371
  - `utest` — crates/core/src/reglages.rs:392

#### `req~autorite-de-la-reference~2`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (6) :
  - `impl` — crates/core/src/fusion.rs:81
  - `utest` — crates/core/src/fusion.rs:452
  - `impl` — crates/core/src/reglages.rs:256
  - `utest` — crates/core/src/reglages.rs:473
  - `utest` — crates/core/src/reglages.rs:484
  - `utest` — crates/core/src/synchronisation/tests.rs:314

#### `req~fusion-trois-voies~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/fusion.rs:45
  - `utest` — crates/core/src/fusion.rs:222
  - `utest` — crates/core/src/fusion.rs:235
  - `utest` — crates/core/src/fusion.rs:248

#### `req~modification-unilaterale-appliquee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (3) :
  - `impl` — crates/core/src/fusion.rs:60
  - `utest` — crates/core/src/fusion.rs:278
  - `utest` — crates/core/src/fusion.rs:294

#### `req~divergence-soumise-a-decision~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/fusion.rs:82
  - `utest` — crates/core/src/fusion.rs:339
  - `impl` — crates/core/src/synchronisation.rs:248
  - `utest` — crates/core/src/synchronisation/tests.rs:103

#### `req~multivalues-par-valeur-normalisee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/fusion.rs:140
  - `utest` — crates/core/src/fusion.rs:375
  - `impl` — crates/core/src/normalisation.rs:15
  - `utest` — crates/core/src/normalisation.rs:122

#### `req~synchronisation-non-bloquante~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (2) :
  - `impl` — crates/core/src/synchronisation.rs:105
  - `utest` — crates/core/src/synchronisation/tests.rs:169

#### `req~suppression-sans-destruction~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (2) :
  - `impl` — crates/core/src/synchronisation.rs:427
  - `utest` — crates/core/src/synchronisation/tests.rs:381

#### `req~detection-doublons~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/doublons.rs:50
  - `utest` — crates/core/src/doublons.rs:137
  - `utest` — crates/core/src/doublons.rs:152
  - `utest` — crates/core/src/synchronisation/tests.rs:560

#### `req~traitement-des-doublons~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (3) :
  - `impl` — crates/core/src/synchronisation.rs:362
  - `utest` — crates/core/src/synchronisation/tests.rs:488
  - `utest` — crates/core/src/synchronisation/tests.rs:541

### Bilan (couvertures réelles)

40/61 exigence(s) testable(s) couverte(s) par au moins une trace réelle (65.6 %) — 21 à couvrir.

## 2. L'inverse — outillage sans exigence

Fichiers d'outillage (`tools/`, hooks git, gabarits de forge) qui ne portent AUCUN tag de traçabilité (`->req~…`). **Avertissement honnête** : l'absence de tag ne prouve PAS l'absence d'exigence — un fichier peut très bien servir une exigence couverte ailleurs. Cette liste est une liste DE REVUE, pas un verdict. `examples/` et `templates/` en sont exclus (spécification propre, ou identifiants d'exemple seulement — même exclusion qu'OpenFastTrace, `tools/oft.sh`).

### `.githooks` (2)

- `.githooks/commit-msg`
- `.githooks/pre-commit`

### `.github` (2)

- `.github/PULL_REQUEST_TEMPLATE.md`
- `.github/dependabot.yml`

### `.github/ISSUE_TEMPLATE` (4)

- `.github/ISSUE_TEMPLATE/amelioration.yml`
- `.github/ISSUE_TEMPLATE/anomalie.yml`
- `.github/ISSUE_TEMPLATE/config.yml`
- `.github/ISSUE_TEMPLATE/question-ouverte.yml`

### `.github/workflows` (2)

- `.github/workflows/ci.yml`
- `.github/workflows/security.yml`

### `tools` (52)

- `tools/adr-new.sh`
- `tools/audit.sh`
- `tools/automatismes-test.sh`
- `tools/badges.sh`
- `tools/bootstrap-test.sh`
- `tools/bootstrap.sh`
- `tools/bump.sh`
- `tools/checks-test.sh`
- `tools/checks.sh`
- `tools/ci-test.sh`
- `tools/ci.sh`
- `tools/classification.sh`
- `tools/clean-test.sh`
- `tools/clean.sh`
- `tools/derive-test.sh`
- `tools/derive.sh`
- `tools/donnees-test.sh`
- `tools/donnees.sh`
- `tools/hygiene-compilation-test.sh`
- `tools/hygiene-compilation.sh`
- `tools/install-git.ps1`
- `tools/install-git.sh`
- `tools/issues.sh`
- `tools/journalisation-test.sh`
- `tools/journalisation.sh`
- `tools/journaux-revisions-editoriales.txt`
- `tools/journaux.sh`
- `tools/lexique.sh`
- `tools/liens-test.sh`
- `tools/liens.sh`
- `tools/locale-test.sh`
- `tools/oft-doublons-test.sh`
- `tools/oft-test.sh`
- `tools/oft.sh`
- `tools/paralleles-test.sh`
- `tools/paralleles.sh`
- `tools/perimetre-test.sh`
- `tools/perimetre.sh`
- `tools/radicale-requirements.txt`
- `tools/rattachement-herites.txt`
- `tools/rattachement-test.sh`
- `tools/releves.sh`
- `tools/revues.sh`
- `tools/rust-kit-test.sh`
- `tools/rust.sh`
- `tools/secrets-test.sh`
- `tools/secrets.sh`
- `tools/setup-test.sh`
- `tools/setup.sh`
- `tools/socle-test.sh`
- `tools/statuts-test.sh`
- `tools/todo.sh`

### `tools/bootstrap.d` (3)

- `tools/bootstrap.d/10-curl.sh`
- `tools/bootstrap.d/20-java.sh`
- `tools/bootstrap.d/50-rust.sh`

### `tools/hooks` (2)

- `tools/hooks/commit-msg.sh`
- `tools/hooks/pre-commit.sh`

**Total : 67/76 fichier(s) d'outillage sans trace de traçabilité.**


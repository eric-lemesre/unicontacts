# Matrice de traçabilité

<!-- GÉNÉRÉ par tools/matrice.sh — ne pas éditer à la main (R8). -->

Vue de synthèse, informative : elle n'ajoute aucun contrôle à `task trace` (OpenFastTrace) ni à `task ci` (`tools/classification.sh`) et ne peut jamais faire échouer une porte. Section 1 : chaque exigence des spécifications, avec ses couvertures RÉELLEMENT trouvées par balayage du dépôt (pas seulement les `Needs:` déclarés). Section 2 : son inverse, les fichiers d'outillage sans aucune trace de traçabilité — une liste de revue, pas un verdict.

## 1. Exigences → couvertures

### spec/02-exigences/android.md

#### `req~compte-systeme-dedie~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~ecriture-sous-compte-unique~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~synchronisation-directe~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

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
- Couvertures réelles : — À COUVRIR

#### `req~vue-comparee~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~actions-de-resolution~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (6) :
  - `impl` — crates/core/src/decisions.rs:170
  - `utest` — crates/core/src/decisions.rs:261
  - `utest` — crates/core/src/decisions.rs:279
  - `impl` — crates/core/src/referentiel.rs:269
  - `utest` — crates/core/src/referentiel/tests.rs:141
  - `utest` — crates/core/src/synchronisation/tests.rs:344

#### `req~report-decision~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (3) :
  - `impl` — crates/core/src/referentiel.rs:188
  - `utest` — crates/core/src/referentiel/tests.rs:91
  - `utest` — crates/core/src/referentiel/tests.rs:109

#### `req~resolution-par-lot~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/referentiel.rs:204
  - `impl` — crates/core/src/referentiel.rs:251
  - `utest` — crates/core/src/referentiel/tests.rs:304
  - `utest` — crates/core/src/referentiel/tests.rs:314

#### `req~issues-suppression-detectee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (6) :
  - `impl` — crates/core/src/decisions.rs:171
  - `utest` — crates/core/src/decisions.rs:298
  - `impl` — crates/core/src/referentiel.rs:320
  - `utest` — crates/core/src/referentiel/tests.rs:362
  - `impl` — crates/core/src/synchronisation.rs:370
  - `utest` — crates/core/src/synchronisation/tests.rs:403

#### `req~portee-suppression-locale~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (3) :
  - `impl` — crates/core/src/referentiel.rs:408
  - `impl` — crates/core/src/referentiel.rs:433
  - `utest` — crates/core/src/referentiel/tests.rs:451

### spec/02-exigences/groupes.md

#### `req~appartenance-multiple~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~groupes-hierarchiques~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~filtre-par-groupes~2`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~groupes-en-categories~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~renommage-groupe-operation-unique~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

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
  - `impl` — crates/core/src/fiche.rs:71
  - `impl` — crates/core/src/fiche.rs:192
  - `utest` — crates/core/src/fiche.rs:210
  - `utest` — crates/core/src/fiche.rs:224
  - `utest` — crates/core/src/fiche.rs:236
  - `impl` — crates/core/src/identifiants.rs:52
  - `utest` — crates/core/src/identifiants.rs:70
  - `impl` — crates/core/src/stockage.rs:178
  - `utest` — crates/core/src/stockage/tests.rs:131

#### `req~provenance-par-champ~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (8) :
  - `impl` — crates/core/src/champ.rs:98
  - `impl` — crates/core/src/fiche.rs:124
  - `impl` — crates/core/src/fiche.rs:147
  - `utest` — crates/core/src/fiche.rs:252
  - `utest` — crates/core/src/fiche.rs:276
  - `impl` — crates/core/src/stockage.rs:179
  - `utest` — crates/core/src/stockage/tests.rs:132
  - `impl` — crates/core/src/temps.rs:11

#### `req~conservation-proprietes-inconnues~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (5) :
  - `impl` — crates/core/src/stockage.rs:180
  - `utest` — crates/core/src/stockage/tests.rs:133
  - `impl` — crates/core/src/vcard/ecriture.rs:170
  - `utest` — crates/core/src/vcard/tests.rs:363
  - `utest` — crates/core/src/vcard/tests.rs:381

#### `req~vcard-3-et-4~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (7) :
  - `impl` — crates/core/src/vcard/ecriture.rs:145
  - `impl` — crates/core/src/vcard/ecriture.rs:171
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
  - `utest` — crates/core/src/lib.rs:38

### spec/02-exigences/relations.md

#### `req~fiche-moi~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~liens-types~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~reciproques-automatiques~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~lien-par-identifiant-maitre~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~section-proches~1`

- Niveau normatif : DOIT
- Needs: impl, mtest
- Couvertures réelles : — À COUVRIR

#### `req~synchronisation-relations~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

### spec/02-exigences/synchronisation.md

#### `req~source-carddav~1`

- Niveau normatif : DOIT
- Needs: impl, itest
- Couvertures réelles : — À COUVRIR

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
  - `impl` — crates/core/src/synchronisation.rs:208
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
  - `impl` — crates/core/src/synchronisation.rs:207
  - `utest` — crates/core/src/synchronisation/tests.rs:103

#### `req~multivalues-par-valeur-normalisee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (4) :
  - `impl` — crates/core/src/fusion.rs:140
  - `utest` — crates/core/src/fusion.rs:375
  - `impl` — crates/core/src/normalisation.rs:15
  - `utest` — crates/core/src/normalisation.rs:120

#### `req~synchronisation-non-bloquante~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (2) :
  - `impl` — crates/core/src/synchronisation.rs:100
  - `utest` — crates/core/src/synchronisation/tests.rs:169

#### `req~suppression-sans-destruction~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles (2) :
  - `impl` — crates/core/src/synchronisation.rs:369
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
  - `impl` — crates/core/src/synchronisation.rs:307
  - `utest` — crates/core/src/synchronisation/tests.rs:488
  - `utest` — crates/core/src/synchronisation/tests.rs:541

### Bilan (couvertures réelles)

23/61 exigence(s) testable(s) couverte(s) par au moins une trace réelle (37.7 %) — 38 à couvrir.

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

### `tools` (51)

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

**Total : 66/71 fichier(s) d'outillage sans trace de traçabilité.**


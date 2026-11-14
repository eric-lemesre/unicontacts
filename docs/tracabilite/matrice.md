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
- Couvertures réelles : — À COUVRIR

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
- Couvertures réelles : — À COUVRIR

#### `req~report-decision~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~resolution-par-lot~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~issues-suppression-detectee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~portee-suppression-locale~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

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
- Couvertures réelles : — À COUVRIR

#### `req~provenance-par-champ~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~conservation-proprietes-inconnues~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~vcard-3-et-4~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~coeur-independant-android~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

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
- Couvertures réelles : — À COUVRIR

#### `req~autorite-par-champ~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~source-de-reference-unique~2`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~autorite-de-la-reference~2`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~fusion-trois-voies~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~modification-unilaterale-appliquee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~divergence-soumise-a-decision~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~multivalues-par-valeur-normalisee~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~synchronisation-non-bloquante~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~suppression-sans-destruction~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~detection-doublons~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

#### `req~traitement-des-doublons~1`

- Niveau normatif : DOIT
- Needs: impl, utest
- Couvertures réelles : — À COUVRIR

### Bilan (couvertures réelles)

0/61 exigence(s) testable(s) couverte(s) par au moins une trace réelle (0.0 %) — 61 à couvrir.

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
- `tools/gouvernance-test.sh`
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

**Total : 67/71 fichier(s) d'outillage sans trace de traçabilité.**


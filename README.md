# Unicontacts

Application Android de gestion unifiée des contacts, avec un cœur métier écrit
en Rust.

[English summary](README.en.md)

## Principe

Un référentiel maître unique contient les contacts, gérés comme un seul bloc.
Les sources (CardDAV, Google, SIM, stockage local…) ne sont que des cibles de
synchronisation. Toute divergence non triviale est soumise à l'utilisateur.

## Spécification

- [docs/SPEC.md](docs/SPEC.md) — spécification fonctionnelle et technique
  v0.1 (2026-11-14), en cours de découpage dans la chaîne tracée ci-dessous.
- `spec/01-besoins/` (besoins), `spec/02-exigences/` (exigences EARS),
  `docs/adr/` (décisions), `spec/open-questions.md` (points à arbitrer).

## Méthode

Projet instancié depuis le gabarit
[alm-forge](https://github.com/eric-lemesre/alm-forge) : exigences tracées du
besoin aux tests par OpenFastTrace, décisions en ADR, portes vérifiées en
local et en CI.

- Méthode : `docs/methode/` — contrat de travail : `AGENTS.md`.
- Façade : [just](https://just.systems/) (`just` liste les recettes) ; chaque
  recette délègue à `tools/*.sh`, appelable directement.
- Accueil après clone : `just setup` (dépendances, hooks, état des portes).
- Avant de proposer : `just ci` (ou `sh tools/ci.sh`).

## État

**Conception** — aucun code métier pour l'instant : squelette du workspace
Cargo, spécification et outillage de la méthode.

## Structure

| Chemin | Contenu |
| --- | --- |
| `crates/core/` | `unicontacts-core` : modèle, stockage, fusion, décisions |
| `spec/` | Besoins, exigences, questions ouvertes |
| `docs/` | Spécification v0.1, méthode, ADR, audit, revues |
| `tools/` | Portes qualité (scripts POSIX) — point d'entrée `ci.sh` |
| `rust-toolchain.toml` | Chaîne Rust épinglée (version exacte) |

## Licence

[GPL-3.0-or-later](LICENSE). Porteur : Eric Lemesre.

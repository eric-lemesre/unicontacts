# Unicontacts

Application Android de gestion unifiée des contacts, avec un cœur métier écrit
en Rust.

[English summary](README.en.md)

## Principe

Un référentiel maître unique contient les contacts, gérés comme un seul bloc.
Les sources (CardDAV, Google, SIM, stockage local…) ne sont que des cibles de
synchronisation. Toute divergence non triviale est soumise à l'utilisateur.

## Spécification

- [Spécification v0.1](spec/01-besoins/sources/2026-11-14-specification-v0.1.md)
  (2026-11-14) — document source, conservé tel quel.
- Découpée en [besoins](spec/01-besoins/besoins.md), en
  [exigences EARS](spec/02-exigences/README.md) par domaine (approuvées lot
  par lot, à l'ouverture de chaque lot), en [décisions](docs/adr/README.md) et en
  [questions ouvertes](spec/open-questions.md).

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

**Spécification arbitrée, réalisation à venir** — aucun code métier pour
l'instant ; prochain chantier : le lot 1, cœur Rust
([feuille de route](docs/todo/README.md)).

## Structure

| Chemin | Contenu |
| --- | --- |
| `crates/core/` | `unicontacts-core` : modèle, stockage, fusion, décisions |
| `spec/` | Besoins, exigences, questions ouvertes |
| `docs/` | Méthode, ADR, registre des décisions, audit, revues |
| `tools/` | Portes qualité (scripts POSIX) — point d'entrée `ci.sh` |
| `rust-toolchain.toml` | Chaîne Rust épinglée (version exacte) |

## Licence

[GPL-3.0-or-later](LICENSE). Porteur : Eric Lemesre.

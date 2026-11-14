# Registre des décisions

Registre chronologique des décisions durables actées hors pull request.
Identifiant : D-AAAA-MM-NN ; chaque entrée renvoie vers l'ADR ou le relevé qui
la développe (modèle : templates/fr/decision-record.md, relevés archivés ici).

| ID | Date | Décision | Développée dans |
| --- | --- | --- | --- |
| D-2026-11-01 | 2026-11-14 | Cycle de vie des issues : circuit réduit — flux de consultation [Spec]/[Orga] remplacés par questions ouvertes et ADR | ADR-0013 du gabarit alm-forge |
| D-2026-11-02 | 2026-11-14 | Mention de l'IA dans les commits : interdite — aucun trailer d'agent ni de co-auteur généré, refusés par le hook commit-msg | ADR-0011 du gabarit alm-forge |
| D-2026-11-03 | 2026-11-14 | Langue du dépôt : fr-en — documentation normative en français, modèles bilingues, résumé anglais (README.en.md) | ADR-0009 du gabarit alm-forge |
| D-2026-11-04 | 2026-11-14 | Façade d'orchestration : just (justfile de templates/orchestrators/), Taskfile.yml retiré | ADR-0007 du gabarit alm-forge |
| D-2026-11-05 | 2026-11-14 | Forge : GitHub seule — façade CI GitHub conservée, .gitlab-ci.yml et .gitlab/ retirés | ADR-0010 du gabarit alm-forge |
| D-2026-11-06 | 2026-11-14 | Licence : GPL-3.0-or-later (texte GPL-3 dans LICENSE, identifiant SPDX dans Cargo.toml) | ADR-0001 |
| D-2026-11-07 | 2026-11-14 | Interface Android en Kotlin / Jetpack Compose, reliée au cœur Rust par UniFFI (OQ-001) | ADR-0006 |
| D-2026-11-08 | 2026-11-14 | Questions OQ-002 à OQ-006 tranchées : autorité par source et par champ ; filtre multi-groupes au choix, union par défaut ; rappels de dates confiés à l'agenda ; pas de contexte pro / perso global, préférence pro et perso par contact ; groupes hiérarchiques. OQ-007 (nom) reste ouverte | [questions ouvertes](../../spec/open-questions.md) |
| D-2026-11-09 | 2026-11-14 | Ouverture du lot 1 (cœur Rust), selon la méthode de greffier — on n'approuve que ce qu'on réalise : les domaines referentiel, synchronisation et file-decisions passent ACTIVE et leurs 23 exigences du lot 1 sont approuvées, à réaliser ; les autres exigences et les six autres domaines restent proposés jusqu'à l'ouverture de leur lot ; ADR-0003 et ADR-0005 acceptés, ADR-0004 reste proposé | ADR-0003 |


# Registre des décisions d'architecture (ADR)


Une décision d'architecture est un choix justifié qui répond à une exigence
fonctionnelle ou non fonctionnelle de portée structurante. Chaque décision est
consignée dans un ADR : un fichier, une décision, sa motivation, ses
alternatives écartées et son moyen de vérification.

## Quand écrire un ADR

- Le choix engage la structure, la sécurité, les données, l'outillage ou la
  méthode, et coûterait cher à défaire.
- Le choix tranche entre plusieurs options défendables : les alternatives
  écartées doivent survivre à la discussion.
- Le choix interprète un besoin ambigu : l'ADR matérialise le jugement rendu et
  renvoie à la question ouverte qui l'a soulevé.

Ce qui n'est **pas** un ADR : le contenu d'une spécification (il vit dans
`spec/`), la coordination d'un chantier (elle vit dans les issues), le
compte-rendu d'une séance (il vit dans `docs/decisions/`).

## Conventions

- Fichier `NNNN-titre-en-kebab-case.md`, numérotation continue, jamais
  réutilisée : un ADR rejeté consomme son numéro.
- Titre : la décision formulée à l'infinitif.
- Modèle : [`templates/fr/adr.md`](../../templates/fr/adr.md) (dérivé de MADR
  4.0, enrichi de la section Vérification).
- **Un ADR accepté ne se modifie pas.** On écrit un nouvel ADR qui le
  remplace ; l'ancien passe au statut « Remplacé par ADR-XXXX ».
- La section **Vérification** est obligatoire pour tout ADR accepté : une
  décision qu'aucun mécanisme ne protège dérivera. Ce point est contrôlé par
  les portes qualité (`task check`).
- Quand la décision répond à une exigence tracée, l'ADR porte un item de
  conception OpenFastTrace (type dsn) qui couvre l'exigence : la décision entre
  ainsi dans la matrice de traçabilité.

## Cycle de vie

```text
Proposé ──▶ Accepté ──▶ Déprécié
   │                       ▲
   └──▶ Rejeté             └── Remplacé par ADR-XXXX
```

## Niveaux d'enjeu et cérémonie

| Niveau | Nature | Circuit |
| --- | --- | --- |
| S — structurant | irréversible, sécurité, réglementaire, données | séance + relevé de décision, puis ADR |
| F — fonctionnel | arbitrage de conception courant | consultation asynchrone, décision du responsable de domaine |
| É — éditorial | reformulation, outillage mineur | simple pull request |

## Index

| ADR | Titre | Statut |
| --- | --- | --- |
| [ADR-0001](0001-publier-sous-gpl-3-0-ou-ulterieure.md) | Publier unicontacts sous GPL-3.0 ou ultérieure | Accepté |
| [ADR-0002](0002-adopter-just-comme-facade-d-orchestration.md) | Adopter just comme façade d'orchestration | Accepté |
| [ADR-0003](0003-coeur-metier-rust-independant-d-android.md) | Écrire le cœur métier en Rust, indépendant d'Android, sur SQLite | Accepté |
| [ADR-0004](0004-ecrire-le-referentiel-sous-le-seul-compte-de-l-application.md) | Écrire le référentiel sous le seul compte de l'application, synchroniser en direct | Proposé |
| [ADR-0005](0005-fusionner-a-trois-voies-champ-par-champ.md) | Fusionner à trois voies, champ par champ, et différer les divergences | Accepté |
| [ADR-0006](0006-ecrire-l-interface-android-en-kotlin-compose-avec-uniffi.md) | Écrire l'interface Android en Kotlin / Compose, reliée au cœur par UniFFI | Accepté |

Les décisions fondatrices du gabarit alm-forge restent consultables dans son
dépôt d'origine.

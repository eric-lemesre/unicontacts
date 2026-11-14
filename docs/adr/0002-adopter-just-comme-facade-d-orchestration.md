# ADR-0002 — Adopter just comme façade d'orchestration

**Statut** : Accepté
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-04 au registre
**Portée** : durable

## Contexte

Le gabarit alm-forge laisse le choix de la façade d'orchestration au projet
instancié (ADR-0007 du gabarit) : Task par défaut, just en variante, ou aucune.
La logique vit dans `tools/*.sh` dans tous les cas. Le contrat de dépôt
(`AGENTS.md`) et le guide de contribution, hérités du gabarit, citaient les
commandes Task ; ils doivent citer la façade réellement en place.

## Décision

La façade d'orchestration d'unicontacts est **just** : `justfile` à la racine,
copié de `templates/orchestrators/justfile`, en pure délégation à
`tools/*.sh`. `Taskfile.yml` est retiré. `AGENTS.md` et `CONTRIBUTING.md`
citent les recettes just ; les équivalents directs `sh tools/…` restent
valables sans aucune façade.

## Alternatives écartées

### Task (défaut du gabarit)

Fonctionne de même, mais ajoute un fichier YAML et un exécutable de plus sur
le poste ; just est déjà la façade de greffier, autre projet Rust du porteur,
ce qui garde un seul jeu de réflexes d'un dépôt à l'autre.

### Aucune façade

Les scripts s'appellent directement, mais la liste des recettes (`just`) sert
d'orientation à un arrivant ; la perdre n'apporte rien.

## Conséquences

- Ce que cela apporte : une seule façade, cohérente avec greffier ; des
  recettes sans le séparateur `--` de Task (`just adr-new titre`).
- Ce que cela coûte : just est un prérequis optionnel du poste (les scripts
  restent appelables sans lui).
- Ce que cela ferme : rien ; changer de façade reste un nouvel ADR.

## Vérification

- `tools/checks.sh` refuse une façade qui cite un script absent ou un contrat
  qui renvoie vers une commande inexistante (« aucune référence pendante
  depuis les façades ni le contrat »).
- `tools/hygiene-compilation.sh` examine le `justfile` comme toute façade qui
  lancerait des tests Cargo.

## Références

- ADR-0007 du gabarit alm-forge (orchestrateur en façade interchangeable).
- Registre des décisions : D-2026-11-04.

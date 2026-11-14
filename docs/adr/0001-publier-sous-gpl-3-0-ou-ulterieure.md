# ADR-0001 — Publier unicontacts sous GPL-3.0 ou ultérieure

**Statut** : Accepté
**Date** : 2026-11-14
**Décideur** : Eric Lemesre — seul (porteur), D-2026-11-06 au registre
**Portée** : durable

## Contexte

Le squelette initial déclarait MIT dans le manifeste de la crate, sans fichier
de licence ; l'instanciation du gabarit alm-forge réinitialise `LICENSE`. Le
choix devait donc être posé explicitement avant toute publication.

Unicontacts est une application Android visant d'abord un Fairphone 6 sous
/e/OS, écosystème d'applications libres (F-Droid notamment) où le copyleft est
l'usage courant. L'application manipule le carnet d'adresses complet de son
utilisateur : pouvoir auditer ce que fait toute version distribuée, y compris
un dérivé, a de la valeur pour ses utilisateurs. Le choix de licence est
irréversible pour ce qui est déjà publié.

## Décision

Le code d'unicontacts est publié sous **GPL-3.0-or-later**. Le texte de la
GPL version 3 est le fichier `LICENSE` ; l'identifiant SPDX
`GPL-3.0-or-later` est déclaré une fois, dans `[workspace.package]` du
`Cargo.toml` racine, et hérité par chaque crate.

## Alternatives écartées

### MIT ou Apache-2.0 (permissives)

Réutilisation maximale, y compris dans un produit fermé qui reprendrait le
moteur de synchronisation sans rien redonner ni permettre l'audit de ce qu'il
fait des contacts. Apache-2.0 apporte une clause de brevets explicite, que la
GPL v3 couvre aussi.

### EUPL-1.2

Copyleft européen compatible GPL, retenu pour greffier. Pour une application
Android distribuée par les canaux du logiciel libre mobile, la GPL est la
licence attendue et la plus directement lisible par les contributeurs.

## Conséquences

- Ce que cela apporte : tout dérivé distribué reste libre et auditable ;
  cohérence avec l'écosystème F-Droid et /e/OS.
- Ce que cela coûte : chaque dépendance doit être compatible GPL-3.0
  (bibliothèques Rust et Android) ; à vérifier à chaque ajout.
- Ce que cela ferme : le retour à une licence permissive ou fermée pour le
  code déjà publié.

## Vérification

- `LICENSE` contient le texte de la GPL version 3 (copie de
  `/usr/share/common-licenses/GPL-3`, non modifiée).
- La porte des badges (`tools/badges.sh`) lit l'identifiant SPDX de
  `Cargo.toml` et affiche « GPL-3.0-or-later » ; la porte anti-dérive refuse
  un badge qui le contredirait.
- Contrôle de compatibilité des licences des dépendances : **non câblé** —
  aucune dépendance à ce jour ; à câbler (par exemple `cargo deny check
  licenses`) avec la première dépendance ajoutée, ADR d'outillage à l'appui
  (R7).

## Références

- <https://www.gnu.org/licenses/gpl-3.0.html>
- Registre des décisions : D-2026-11-06.

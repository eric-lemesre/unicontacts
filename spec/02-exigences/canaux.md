# Exigences — domaine canaux de communication

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 11) ; passe ACTIVE quand une décision du registre l'acte. La
détermination du contexte pro / perso (OQ-005) reste à trancher.

## Canaux rattachés à un identifiant de la fiche

`req~canaux-par-identifiant~1`
Status: proposed

**Statut : proposée**

Le système DOIT enregistrer chaque canal disponible d'une fiche (appel, SMS,
e-mail, WhatsApp, Signal, Telegram, Matrix…) rattaché à un identifiant de
cette fiche (numéro, adresse, pseudonyme).

Critères d'acceptation :

- Étant donné un canal Signal rattaché à un numéro, quand ce numéro est retiré
  de la fiche, alors le canal n'est plus disponible.

Covers:

- feat~canaux-preferes~1

Needs: impl, utest

## Classements pro et perso

`req~classements-pro-perso~1`
Status: proposed

**Statut : proposée**

Le système DOIT tenir pour chaque fiche deux classements des canaux par
préférence, l'un pro et l'un perso ; si un seul est renseigné, il DOIT valoir
pour les deux contextes.

Critères d'acceptation :

- Étant donné une fiche dont seul le classement perso est renseigné, quand le
  contexte est pro, alors le classement perso s'applique.

Covers:

- feat~canaux-preferes~1

Needs: impl, utest

## Note libre par canal

`req~note-par-canal~1`
Status: proposed

**Statut : proposée**

Le système DOIT permettre d'attacher une note libre à chaque canal d'une
fiche.

Covers:

- feat~canaux-preferes~1

Needs: impl, utest

## Détection assistée des messageries

`req~detection-assistee-canaux~1`
Status: proposed

**Statut : proposée**

Quand une messagerie installée a déclaré dans Android un contact
correspondant à un identifiant d'une fiche, le système DOIT suggérer le canal
correspondant, et NE DOIT l'ajouter qu'après confirmation de l'utilisateur.

Covers:

- feat~canaux-preferes~1

Needs: impl, mtest

## Actions triées selon la préférence

`req~actions-triees-par-preference~1`
Status: proposed

**Statut : proposée**

Quand une fiche est affichée, le système DOIT présenter un bouton d'action
par canal disponible, dans l'ordre du classement du contexte courant, chaque
bouton ouvrant l'application concernée.

Covers:

- feat~canaux-preferes~1

Needs: impl, mtest

## Filtrage par canal

`req~filtre-par-canal~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur filtre la liste par un canal, le système DOIT n'afficher
que les fiches qui disposent de ce canal.

Covers:

- feat~canaux-preferes~1

Needs: impl, utest

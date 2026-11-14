# Exigences — domaine canaux de communication

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 11) ; passe ACTIVE à l'ouverture de son lot, par une décision du registre. Pas de
contexte pro / perso global : chaque fiche porte une préférence pro et une
préférence perso (OQ-005, tranchée le 2026-11-14).

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

## Préférences pro et perso par fiche

`req~classements-pro-perso~2`
Status: proposed

**Statut : proposée**

Le système DOIT tenir pour chaque fiche deux classements de ses canaux par
préférence, l'un pour les communications pro, l'autre pour les
communications perso ; si un seul est renseigné, il DOIT valoir pour les
deux.

Rationale:
Révision 2 (2026-11-14) : OQ-005 écarte tout contexte pro / perso global ;
la préférence est propre à chaque contact.

Critères d'acceptation :

- Étant donné une fiche dont seul le classement perso est renseigné, quand
  son canal préféré pour une communication pro est demandé, alors c'est le
  premier canal du classement perso.

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

## Actions selon les préférences pro et perso

`req~actions-triees-par-preference~2`
Status: proposed

**Statut : proposée**

Quand une fiche est affichée, le système DOIT présenter un bouton d'action
par canal disponible, en distinguant le canal préféré pour une communication
pro et celui pour une communication perso, chaque bouton ouvrant
l'application concernée, sans dépendre d'aucun mode pro / perso global.

Rationale:
Révision 2 (2026-11-14) : OQ-005 — c'est l'utilisateur qui sait, à chaque
communication, si elle est pro ou perso.

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

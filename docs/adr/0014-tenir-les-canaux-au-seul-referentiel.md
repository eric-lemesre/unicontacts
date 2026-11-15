# ADR-0014 — Tenir les canaux au seul référentiel, rattachés aux identifiants de la fiche

**Statut** : Accepté
**Date** : 2026-11-15
**Décideur** : Eric Lemesre — conception déléguée (« cycle complet » du lot 4, 2026-11-15), D-2026-11-13 au registre
**Portée** : phase (condition de réouverture : besoin de partager canaux ou préférences avec un autre carnet)

## Contexte

Le domaine canaux veut, pour chaque fiche, des canaux rattachés à un
identifiant (numéro, adresse, pseudonyme), une note par canal, deux
classements pro et perso, des boutons d'action et un filtre par canal
(OQ-005 : pas de contexte pro / perso global). Aucune exigence ne demande
de les synchroniser vers les sources.

## Décision

**Canaux qui découlent de la fiche** : appel et SMS pour chaque numéro,
e-mail pour chaque adresse ; ils ne se déclarent pas.

**Messageries** (WhatsApp, Signal, Telegram, Matrix, ou une autre nommée)
: déclarées par l'utilisateur, ou suggérées d'après les contacts que les
messageries installées déclarent dans Android puis confirmées. Elles se
rattachent à un numéro de la fiche (WhatsApp, Signal), à un numéro ou un
pseudonyme `@…` (Telegram), ou à un pseudonyme (Matrix, autres). Rattachée
à un numéro ou une adresse, une messagerie n'est disponible que tant que la
fiche le porte.

Un canal se désigne par sa **clé** : son type et son identifiant normalisé
(`signal:+33639980000`). Notes et classements se rattachent aux clés ; un
classement ne retient que les canaux disponibles, et un classement vide
laisse l'autre valoir pour les deux contextes.

Messageries déclarées, notes et classements restent **au seul référentiel**
(tables `canaux`, `notes_canaux`, `classements`) : aucune vCard ne les
porte.

## Alternatives écartées

### `IMPP` et propriétés étendues dans les vCard

`IMPP` porterait les messageries, mais pas les notes ni les classements, et
les carnets courants l'interprètent mal ; on ajouterait aux cartes des
données que personne d'autre ne lirait.

### Canaux libres, sans rattachement

Plus simple, mais un numéro retiré laisserait des canaux morts, contraire à
l'exigence `req~canaux-par-identifiant~1`.

## Conséquences

- Ce que cela apporte : des canaux toujours cohérents avec les numéros et
  adresses de la fiche ; des préférences propres à chaque contact.
- Ce que cela coûte : messageries, notes et préférences ne se retrouvent
  pas sur un autre appareil ni dans un autre carnet.
- Ce que cela ferme : rien de durable ; un portage en `IMPP` pourra venir.

## Vérification

- Tests du cœur : canal Signal disparu avec son numéro, identifiant absent
  refusé, classement perso valant pour le pro, notes, filtre par canal ;
  aller-retour SQLite.
- Tests JVM : adresses d'action de chaque canal, rapprochement des
  messageries déclarées dans Android.

## Références

- `req~canaux-par-identifiant~1`, `req~classements-pro-perso~2`,
  `req~note-par-canal~1`, `req~detection-assistee-canaux~1`,
  `req~actions-triees-par-preference~2`, `req~filtre-par-canal~1` ; OQ-005 ;
  ToDo-0005.

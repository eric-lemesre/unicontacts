# Exigences — domaine photos

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 2 et 9) ; passe ACTIVE quand une décision du registre l'acte. Les
conditions d'accès de chaque fournisseur sont à vérifier à l'implémentation.

## Fournisseurs désactivés par défaut

`req~fournisseurs-desactives-par-defaut~1`
Status: proposed

**Statut : proposée**

Le système DOIT tenir chaque fournisseur de photos (Gravatar/Libravatar,
Mastodon, Bluesky, GitHub, GitLab, Telegram, Matrix…) désactivé tant que
l'utilisateur ne l'a pas activé, un par un.

Covers:

- feat~photos-maitrisees~1

Needs: impl, utest

## Aucune requête à l'insu de l'utilisateur

`req~recuperation-a-la-demande~1`
Status: proposed

**Statut : proposée**

Si aucune récupération de photos n'a été lancée par l'utilisateur, alors le
système NE DOIT émettre aucune requête vers un fournisseur de photos.

Rationale:
Principe 4 (§ 2) : interroger un service tiers lui révèle l'identité des
contacts (e-mail haché, pseudonyme).

Critères d'acceptation :

- Étant donné des fournisseurs activés, quand une synchronisation complète
  des sources s'exécute, alors aucune requête n'est émise vers eux.

Covers:

- feat~photos-maitrisees~1

Needs: impl, utest

## Périmètre et fournisseurs choisis à chaque lancement

`req~lancement-recuperation~1`
Status: proposed

**Statut : proposée**

Quand l'utilisateur lance une récupération de photos, le système DOIT lui
faire choisir le périmètre (une fiche, un groupe, tous les contacts) et les
fournisseurs interrogés, parmi ceux activés.

Covers:

- feat~photos-maitrisees~1

Needs: impl, mtest

## Récupération par lot en tâche de fond, puis revue

`req~revue-des-photos-recuperees~1`
Status: proposed

**Statut : proposée**

Quand une récupération porte sur plusieurs fiches, le système DOIT l'exécuter
en tâche de fond avec sa progression, puis présenter les photos trouvées en
revue, à valider fiche par fiche ou en lot.

Covers:

- feat~photos-maitrisees~1

Needs: impl, mtest

## Photos candidates, une seule retenue

`req~photos-candidates~1`
Status: proposed

**Statut : proposée**

Le système DOIT conserver pour une fiche plusieurs photos candidates, chacune
avec sa provenance et sa date, dont une seule est retenue.

Critères d'acceptation :

- Étant donné deux candidates, quand la seconde est retenue, alors la
  première reste candidate et n'est plus retenue.

Covers:

- feat~photos-maitrisees~1

Needs: impl, utest

## Photo retenue poussée redimensionnée

`req~photo-poussee-redimensionnee~1`
Status: proposed

**Statut : proposée**

Quand une photo est retenue, le système DOIT l'écrire vers les sources en
écriture et vers Android, redimensionnée à la taille que chaque cible
accepte.

Covers:

- feat~photos-maitrisees~1

Needs: impl, utest

## Photo ajoutée à la main

`req~photo-manuelle~1`
Status: proposed

**Statut : proposée**

Le système DOIT permettre d'ajouter une photo candidate par prise de vue, par
choix dans la galerie ou par partage d'une image vers l'application, et de la
recadrer.

Rationale:
Seul recours pour les services fermés (Facebook, Instagram, LinkedIn, X,
WhatsApp, Signal) (§ 9.1).

Covers:

- feat~photos-maitrisees~1

Needs: impl, mtest

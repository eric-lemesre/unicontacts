# Exigences — domaine intégration Android

**Statut : PROPOSITION** — rédigé le 2026-11-14 depuis la spécification v0.1
(§ 3.2 et 3.3) ; passe ACTIVE à l'ouverture de son lot, par une décision du registre.

## Compte système dédié

`req~compte-systeme-dedie~1`
Status: proposed

**Statut : proposée**

Le système DOIT déclarer auprès d'Android un compte qui lui est propre et
synchroniser le référentiel avec Android sous ce compte.

Critères d'acceptation :

- Étant donné l'application installée et configurée, quand l'utilisateur
  consulte les comptes du téléphone, alors le compte d'unicontacts y figure.

Covers:

- feat~integration-android~1

Needs: impl, mtest

## Un seul bloc dans les contacts du téléphone

`req~ecriture-sous-compte-unique~1`
Status: proposed

**Statut : proposée**

Le système DOIT écrire chaque fiche maître une seule fois dans les contacts
Android (`ContactsContract`), sous le seul compte de l'application, quel que
soit le nombre de sources auxquelles elle est liée.

Rationale:
Le téléphone, les SMS et les messageries ne doivent voir qu'un bloc (§ 3.2).

Critères d'acceptation :

- Étant donné une fiche liée à trois sources, quand Android est synchronisé,
  alors l'application Contacts du téléphone n'en montre qu'une, rattachée au
  compte d'unicontacts.

Covers:

- feat~integration-android~1

Needs: impl, mtest

## Synchronisation directe avec les serveurs

`req~synchronisation-directe~1`
Status: proposed

**Statut : proposée**

Le système DOIT synchroniser chaque source directement avec son serveur, sans
passer par un autre synchroniseur du téléphone (DAVx5, synchronisation
/e/OS).

Rationale:
§ 3.3 : un compte resté synchronisé par DAVx5 ou /e/OS fait réapparaître les
contacts des sources en parallèle du bloc ; le retirer est une consigne de
déploiement, à documenter pour l'utilisateur.

Critères d'acceptation :

- Étant donné un téléphone sans DAVx5 ni compte /e/OS synchronisant les
  contacts, quand une source CardDAV est synchronisée, alors ses contacts
  apparaissent sous le seul compte d'unicontacts.

Covers:

- feat~integration-android~1

Needs: impl, mtest

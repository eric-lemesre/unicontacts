# Exigences — domaine synchronisation

**Statut : ACTIVE** — domaine acté le 2026-11-14 (D-2026-11-09) à l'ouverture
du lot 1, rédigé depuis la spécification v0.1 (§ 2 et 5). Seules les exigences
du lot 1 sont approuvées ; les autres le seront à l'ouverture de leur lot.

## Source CardDAV

`req~source-carddav~1`

**Statut : approuvée**

Le système DOIT synchroniser le référentiel avec un carnet d'adresses CardDAV
(Murena, Nextcloud), dans les deux sens par défaut.

Rationale:
Connecteur principal, premier mis en œuvre (§ 5.1).

Critères d'acceptation :

- Étant donné un serveur CardDAV de test et un référentiel vide, quand la
  source est synchronisée, alors chaque vCard du serveur donne une fiche
  maître liée à son UID et à son ETag.
- Étant donné une fiche modifiée dans le référentiel, quand la source est
  synchronisée, alors la vCard du serveur porte la modification et son ETag
  est mis à jour.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, itest

## Source Google

`req~source-google~1`
Status: proposed

**Statut : proposée**

Le système DOIT synchroniser le référentiel avec les contacts d'un compte
Google, de préférence par son interface CardDAV et une autorisation OAuth,
sans dépendre des services Google Play.

Rationale:
§ 5.1 ; la faisabilité de l'interface CardDAV de Google est à vérifier à
l'implémentation. /e/OS ne fournit pas les services Play.

Critères d'acceptation :

- Étant donné un compte Google de test autorisé par OAuth, quand la source
  est synchronisée sur un appareil sans services Google Play, alors ses
  contacts donnent des fiches maîtres liées.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, itest

## Import du stockage local Android

`req~import-stockage-local~1`

**Statut : approuvée**

Quand l'utilisateur le demande, le système DOIT importer dans le référentiel
les contacts du stockage local Android, en passant par la détection des
doublons, sans modifier ni retirer les originaux ; il DOIT ensuite permettre
de retirer du stockage local les originaux importés, par une action distincte
confirmée explicitement.

Rationale:
Le stockage local est une source de migration vers le référentiel (§ 5.1).
La migration va au bout sans jamais être implicite : l'import ne détruit
rien, le retrait est un geste à part (D-2026-11-15, ADR-0015).

Critères d'acceptation :

- Étant donné des contacts dans le stockage local, quand l'import est lancé,
  alors chacun donne une fiche maître ou rejoint une fiche existante selon la
  détection des doublons, et le stockage local reste inchangé.
- Étant donné un import déjà fait, quand il est relancé, alors aucune fiche
  n'est créée en double.
- Étant donné des originaux importés, quand l'utilisateur confirme leur
  retrait, alors ils disparaissent du stockage local et les fiches du
  référentiel restent intactes ; sans confirmation, rien n'est retiré.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest, mtest

## Source SIM en lecture seule par défaut

`req~source-sim-lecture-seule~1`
Status: proposed

**Statut : proposée**

Là où une carte SIM est configurée comme source, le système DOIT en lire le
nom et le numéro de chaque entrée et NE DOIT rien y écrire, sauf si
l'utilisateur règle explicitement cette source en écriture.

Rationale:
Une SIM ne porte que nom et numéro (§ 5.1) ; y écrire tronquerait les fiches.

Critères d'acceptation :

- Étant donné une SIM réglée par défaut, quand une fiche liée est modifiée
  dans le référentiel, alors aucune écriture n'est faite sur la SIM.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, mtest

## Réglages par source

`req~reglages-par-source~1`

**Statut : approuvée**

Le système DOIT permettre de régler, pour chaque source, sa direction
(bidirectionnelle, lecture seule, écriture seule), son autorité (référentiel
ou cette source), le traitement des suppressions (demander, propager,
ignorer) et celui des conflits (demander, l'autorité gagne), avec pour
valeurs par défaut : bidirectionnelle, référentiel, demander, demander.

Rationale:
§ 5.2. L'autorité se règle aussi champ par champ : OQ-002 et
l'exigence `req~autorite-par-champ~1`.

Critères d'acceptation :

- Étant donné une source nouvellement ajoutée, quand ses réglages sont lus,
  alors ils valent les quatre valeurs par défaut.
- Étant donné une source réglée en lecture seule, quand une fiche liée est
  modifiée dans le référentiel, alors rien n'est écrit vers cette source.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Autorité réglée par champ

`req~autorite-par-champ~1`

**Statut : approuvée**

Le système DOIT permettre de désigner, pour un champ donné (par exemple les
téléphones ou les adresses), la source qui fait foi pour ce champ, et cette
désignation DOIT l'emporter, pour ce champ, sur la source de référence
générale.

Rationale:
OQ-002, tranchée le 2026-11-14 : « Google fait foi pour les téléphones,
Nextcloud pour les adresses » doit pouvoir se dire.

Critères d'acceptation :

- Étant donné Nextcloud source de référence et Google désignée pour les
  téléphones, quand un téléphone diverge entre Google et le référentiel,
  alors c'est Google qui fait foi pour ce téléphone.
- Étant donné la même configuration, quand une adresse diverge, alors c'est
  Nextcloud qui fait foi.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Une seule source de référence à chaque niveau

`req~source-de-reference-unique~2`

**Statut : approuvée**

Si l'utilisateur désigne une source comme référence, générale ou pour un
champ, alors qu'une autre l'est déjà au même niveau, alors le système DOIT
refuser le réglage en nommant la source qui l'est déjà.

Rationale:
Révision 2 (2026-11-14) : l'autorité se règle aussi par champ (OQ-002).

Critères d'acceptation :

- Étant donné une source A de référence générale, quand la source B est
  désignée comme référence générale, alors le réglage est refusé et le
  message nomme A.
- Étant donné A désignée pour les téléphones, quand B est désignée pour les
  téléphones, alors le réglage est refusé ; désigner B pour les adresses est
  accepté.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Conflits résolus en faveur de la source qui fait foi

`req~autorite-de-la-reference~2`

**Statut : approuvée**

Là où une source fait foi pour un champ — désignée pour ce champ, ou à
défaut source de référence générale — et que ses conflits sont réglés sur
« l'autorité gagne », quand ce champ diverge entre elle et le référentiel, le
système DOIT retenir la valeur de cette source sans créer de décision.

Rationale:
Révision 2 (2026-11-14) : l'autorité se règle aussi par champ (OQ-002).

Critères d'acceptation :

- Étant donné une source de référence générale dont un champ et celui du
  référentiel ont changé avec des valeurs différentes, quand elle est
  synchronisée, alors le référentiel prend la valeur de la source et la file
  de décisions est inchangée.
- Étant donné une autre source désignée pour ce champ, quand la source de
  référence générale diverge sur ce champ, alors elle ne l'emporte pas.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Comparaison à trois voies, champ par champ

`req~fusion-trois-voies~1`

**Statut : approuvée**

Quand une source est synchronisée, le système DOIT comparer chaque champ
selon trois valeurs : celle du référentiel, celle de la source et celle du
dernier état synchronisé entre eux.

Rationale:
Sans le dernier état synchronisé, impossible de savoir quel côté a changé
(§ 5.3) ; la comparaison champ par champ évite de traiter une fiche entière
comme un conflit.

Critères d'acceptation :

- Étant donné une fiche dont seul le téléphone a changé côté source et seule
  l'adresse côté référentiel, quand la source est synchronisée, alors les
  deux modifications sont retenues sans décision.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Modification d'un seul côté appliquée

`req~modification-unilaterale-appliquee~1`

**Statut : approuvée**

Quand un champ n'a changé que d'un côté depuis le dernier état synchronisé,
le système DOIT appliquer la modification à l'autre côté sans créer de
décision.

Critères d'acceptation :

- Étant donné un e-mail modifié sur la source seulement, quand elle est
  synchronisée, alors le référentiel prend la nouvelle valeur et aucune
  décision n'est créée.
- Étant donné un champ modifié des deux côtés avec la même valeur, quand la
  source est synchronisée, alors rien n'est écrit et aucune décision n'est
  créée.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Divergence soumise à décision

`req~divergence-soumise-a-decision~1`

**Statut : approuvée**

Si un champ a changé des deux côtés avec des valeurs différentes, et que les
réglages de la source ne désignent pas d'autorité qui l'emporte, alors le
système DOIT créer une décision et garder la valeur du référentiel tant
qu'elle n'est pas résolue.

Critères d'acceptation :

- Étant donné un nom modifié différemment dans le référentiel et sur la
  source, quand elle est synchronisée, alors une décision de divergence est
  créée et le référentiel garde son nom.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Champs multivalués comparés par valeur normalisée

`req~multivalues-par-valeur-normalisee~1`

**Statut : approuvée**

Quand un champ multivalué (téléphones, e-mails) est comparé, le système DOIT
comparer ses valeurs sous forme normalisée et fusionner sans décision une
valeur ajoutée d'un seul côté.

Critères d'acceptation :

- Étant donné `06 14 00 00 00` dans le référentiel et `+33614000000` sur la
  source, quand ils sont comparés, alors ils sont reconnus égaux.
- Étant donné un téléphone ajouté sur la source seulement, quand elle est
  synchronisée, alors il rejoint la fiche sans décision.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Synchronisation jamais suspendue par une décision

`req~synchronisation-non-bloquante~1`

**Statut : approuvée**

Tant qu'une décision est en attente, le système DOIT poursuivre la
synchronisation des autres champs et des autres fiches.

Rationale:
Principe 1 (§ 2) : la synchronisation ne bloque jamais.

Critères d'acceptation :

- Étant donné une décision en attente sur le nom d'une fiche, quand la source
  est synchronisée avec un téléphone modifié sur cette même fiche et une
  autre fiche modifiée, alors les deux modifications sont appliquées.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Aucune suppression sans décision

`req~suppression-sans-destruction~1`

**Statut : approuvée**

Si une suppression est détectée sur une source dont les suppressions sont
réglées sur « demander », alors le système DOIT créer une décision de
suppression et NE DOIT supprimer aucune fiche ni aucune donnée.

Rationale:
Principe 2 (§ 2) : rien n'est détruit sans décision.

Critères d'acceptation :

- Étant donné une fiche disparue de la source, quand celle-ci est
  synchronisée, alors la fiche maître et sa correspondance avec la source
  sont intactes et une décision de suppression est créée.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Détection des doublons à l'import

`req~detection-doublons~1`

**Statut : approuvée**

Quand des contacts sont importés d'une source, le système DOIT détecter les
doublons avec les fiches existantes par numéro de téléphone normalisé, par
e-mail et par nom proche, et classer chaque doublon comme certain ou
probable.

Critères d'acceptation :

- Étant donné une fiche existante et un contact importé de même e-mail,
  quand l'import s'exécute, alors le doublon est détecté.
- Étant donné deux noms qui ne diffèrent que d'un accent, sans autre donnée
  commune, quand l'import s'exécute, alors le doublon est classé probable.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

## Doublons certains fusionnés, probables soumis

`req~traitement-des-doublons~1`

**Statut : approuvée**

Quand un doublon est détecté, le système DOIT fusionner sans décision un
doublon certain et créer une décision pour un doublon probable.

Rationale:
§ 5.4 ; les critères de certitude sont à fixer avec la détection.

Critères d'acceptation :

- Étant donné un doublon certain, quand l'import s'exécute, alors une seule
  fiche maître subsiste, liée aux deux origines.
- Étant donné un doublon probable, quand l'import s'exécute, alors les deux
  fiches subsistent et une décision de doublon est créée.

Covers:

- feat~synchronisation-sans-blocage~1

Needs: impl, utest

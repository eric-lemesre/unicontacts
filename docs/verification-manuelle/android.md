# Vérifications manuelles — application Android

Procédures de vérification **manuelle assumée** (items `mtest`, méthode
[ch. 5](../methode/05-tests-et-verification.md)) des exigences qui ne se
constatent que sur un téléphone : compte système, contacts d'Android,
écrans. Elles ne sont pas mécanisées faute de tests instrumentés en CI
(ADR-0009) ; chacune est reportée « à élever » dans la fiche de son lot
(ToDo-0004, ToDo-0005).

**Appareil de référence** : Fairphone 6 sous /e/OS. **Cadence** : à chaque
version publiée, et à chaque PR qui touche `android/` ou `crates/ffi/`.

**Préconditions communes** : APK de débogage installé
(`sh tools/android.sh verifier`, puis
`adb install -r android/app/build/outputs/apk/debug/app-debug.apk`) ;
permission d'accès aux contacts accordée ; un carnet CardDAV de test
(Murena ou Nextcloud) d'au moins trois contacts, dont un présent aussi dans
un second carnet.

## Le compte d'unicontacts figure parmi les comptes du téléphone

`mtest~compte-unicontacts-visible~1`

**Geste** — ouvrir l'application une première fois, puis Paramètres →
Comptes (ou Mots de passe et comptes).

**Critère** — un compte « unicontacts » y figure, avec la synchronisation
des contacts proposée ; le supprimer puis rouvrir l'application le recrée.

Covers:
- req~compte-systeme-dedie~1

## Une fiche liée à plusieurs sources n'apparaît qu'une fois

`mtest~un-seul-bloc-dans-contacts~1`

**Geste** — configurer deux sources CardDAV qui partagent un contact (même
e-mail, même nom), synchroniser, puis ouvrir l'application Contacts du
téléphone et chercher ce contact.

**Critère** — un seul contact, rattaché au compte d'unicontacts, porte les
valeurs réunies des deux sources ; il n'est pas modifiable dans l'application
Contacts (lecture seule pour le reste du téléphone).

Covers:
- req~ecriture-sous-compte-unique~1

## Une source se synchronise directement, sans autre synchroniseur

`mtest~synchronisation-directe-sur-l-appareil~1`

**Geste** — sur un téléphone sans DAVx5 ni compte /e/OS synchronisant les
contacts, ajouter une source CardDAV dans l'application (URL du carnet,
identifiant, mot de passe d'application), lancer « Synchroniser », puis
ouvrir l'application Contacts.

**Critère** — les contacts du carnet apparaissent sous le seul compte
d'unicontacts ; une fiche modifiée dans l'application est modifiée sur le
serveur (vérifié depuis l'interface web du carnet) au tour suivant.

Covers:
- req~synchronisation-directe~1

## Le compteur de décisions ouvre leur liste

`mtest~compteur-de-decisions~1`

**Geste** — provoquer des divergences : modifier le nom d'un contact dans
l'application et, sur le serveur, le modifier autrement ; synchroniser ;
reporter une des décisions créées.

**Critère** — l'écran principal affiche le nombre de décisions en attente et
reportées ; le toucher ouvre leur liste ; une fois toutes résolues, le
compteur disparaît.

Covers:
- req~boite-de-decisions~1

## Une décision s'affiche champ par champ, valeurs côte à côte

`mtest~vue-comparee-sur-l-appareil~1`

**Geste** — ouvrir une décision de divergence depuis la liste.

**Critère** — l'écran montre côte à côte la valeur d'unicontacts et celle de
la source, chacune avec son origine et sa date, ainsi que le dernier état
synchronisé ; les choix proposés sont ceux de la décision (« garder les
deux » seulement pour un champ multivalué).

Covers:
- req~vue-comparee~1

## La section « Proches » rassemble les fiches liées à la fiche « moi »

`mtest~section-proches-sur-l-appareil~1`

**Geste** — ouvrir sa propre fiche, cocher « C'est moi » ; y ajouter deux
relations (un conjoint, un ami) vers deux autres fiches ; revenir à la
liste ; puis décocher « C'est moi ».

**Critère** — la liste s'ouvre sur une section « Proches » qui contient les
deux fiches liées, au-dessus de tous les contacts ; la fiche liée comme
conjoint porte en retour la relation « Conjoint » vers sa propre fiche ;
sans fiche « moi », la section disparaît.

Covers:
- req~section-proches~1

## Les dates à venir se lisent dans l'ordre, filtrables par groupe

`mtest~a-venir-sur-l-appareil~1`

**Geste** — donner à trois fiches des dates proches (une naissance sans
année, un anniversaire de mariage avec année, une fête), ranger l'une d'elles
dans un groupe, ouvrir « À venir », puis filtrer sur ce groupe.

**Critère** — les dates s'affichent de la plus proche à la plus lointaine,
avec le délai (« aujourd'hui », « dans 5 jours ») et l'âge quand l'année est
connue ; le filtre ne garde que la fiche du groupe.

Covers:
- req~vue-a-venir~1

## Les dates sont reportées dans l'agenda choisi

`mtest~dates-dans-l-agenda-sur-l-appareil~1`

**Geste** — dans « Sources », accorder l'accès à l'agenda et en choisir un ;
ajouter un anniversaire à une fiche ; synchroniser ; ouvrir l'agenda du
téléphone ; puis retirer cet anniversaire et synchroniser de nouveau.

**Critère** — un événement annuel récurrent « Anniversaire de … » apparaît
le jour dit, chaque année, et l'agenda en émet le rappel selon ses propres
réglages ; une fois la date retirée, l'événement disparaît.

Covers:
- req~dates-dans-l-agenda~1

## Journal des exécutions

| Date | Exécutant | Version | Verdict |
| --- | --- | --- | --- |

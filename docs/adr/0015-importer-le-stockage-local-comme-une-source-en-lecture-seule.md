# ADR-0015 — Importer le stockage local comme une source en lecture seule

**Statut** : Accepté
**Date** : 2026-11-15
**Décideur** : Eric Lemesre — choix de l'import non destructif suivi d'un retrait confirmé (D-2026-11-15), conception déléguée à l'ouverture du lot 5
**Portée** : phase (condition de réouverture : besoin de synchroniser en continu le stockage local, ou d'y écrire)

## Contexte

L'exigence `req~import-stockage-local~1` demande d'importer à la demande les contacts
du stockage local Android en passant par la détection des doublons, sans
toucher aux originaux, puis de permettre leur retrait par une action
distincte et confirmée. Le moteur du cœur sait déjà rapprocher les copies
d'une source du référentiel : import par la détection des doublons, fusion
à trois voies des copies déjà liées, traitement des copies disparues selon
le réglage des suppressions.

## Décision

Le stockage local devient une **source du cœur**, nommée « Stockage local »,
créée au premier import avec les réglages **lecture seule** et suppressions
**ignorées** : le moteur n'y écrit jamais, et une copie disparue détache sa
fiche sans la toucher.

**Lecture, côté Android** : sont lus les contacts bruts sans compte (ou d'un
compte local connu du système), non supprimés. Leurs lignes de données se
traduisent en valeurs de champs (nom, téléphones, e-mails, adresses,
organisation, fonction, note, sites, dates), réciproque de la traduction qui
publie les fiches (`LignesContacts`). Chaque contact brut est une copie
désignée par `local-<identifiant du contact brut>`.

**Rapprochement, côté cœur** : le cœur bâtit la vCard 4.0 de chaque copie
depuis ces valeurs et la confie à `synchroniser`. Un contact inconnu passe
par la détection des doublons ; un contact déjà importé se fusionne avec sa
fiche ; relancer l'import ne crée donc aucun doublon.

**Retrait des originaux** : l'application relance d'abord l'import, pour que
la dernière modification locale soit prise, puis annonce le nombre de
contacts concernés et ne retire qu'après confirmation explicite. Seuls
partent les contacts bruts liés à une fiche ; un dernier import détache les
liens devenus sans objet.

La source n'a pas de configuration de connexion : l'adaptateur de
synchronisation l'ignore, l'import reste un geste de l'utilisateur.

## Alternatives écartées

### Import sans source ni lien

Plus simple, mais un second import dupliquerait les fiches, et le retrait ne
saurait plus quels contacts ont été versés au référentiel.

### Écrire les vCard côté Kotlin

Doublerait la connaissance du format, que seul le cœur porte (ADR-0004).

### Retirer les originaux dès l'import

Écarté par l'utilisateur (D-2026-11-15) : destructif sans étape de contrôle.

## Conséquences

- Ce que cela apporte : doublons, fusion et provenance de l'import sont ceux
  de toute source ; l'import est rejouable sans risque.
- Ce que cela coûte : une source « Stockage local » figure parmi les
  sources ; les comptes locaux propres à un constructeur doivent être
  reconnus un par un.
- Ce que cela ferme : rien de durable ; une synchronisation continue
  demanderait seulement de lever la lecture seule.

## Vérification

- Tests du cœur : import de contacts locaux en fiches, doublon certain
  rattaché, import rejoué sans doublon, aucune écriture vers la source,
  liens détachés une fois les originaux retirés, fiches intactes.
- Tests JVM : traduction des lignes de données en valeurs de champs.
- Procédure manuelle sur l'appareil : import, puis retrait confirmé
  (`docs/verification-manuelle/android.md`).

## Références

- `req~import-stockage-local~1` ; D-2026-11-15 ; ADR-0004 ; ToDo-0006.

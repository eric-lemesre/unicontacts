# ADR-0017 — Synchroniser Google par CardDAV sous OAuth, client natif à PKCE

**Statut** : Accepté
**Date** : 2026-11-15
**Décideur** : Eric Lemesre — voie OAuth propre retenue (D-2026-11-15), conception déléguée
**Portée** : phase (condition de réouverture : Google ferme son CardDAV, ou retire le schéma d'URI personnalisé aux clients Android)

## Contexte

L'exigence `req~source-google~1` veut les contacts d'un compte Google
synchronisés sans services Play. Depuis 2023, Google n'ouvre son CardDAV
qu'à OAuth 2.0. Un client OAuth de type Android y accède sans services Play
par le navigateur et un schéma d'URI personnalisé, à activer dans les
réglages avancés du client — c'est ainsi que procède DAVx⁵. L'utilisateur a
retenu cette voie avec un identifiant client à lui (D-2026-11-15). Le
CardDAV de Google sert du vCard 3.0, et n'implémente pas tous les rapports
de la norme ; `addressbook-multiget` y est documenté.

## Décision

**Autorisation** : flux « code d'autorisation » d'un client natif public,
avec PKCE `S256` (RFC 7636) et un paramètre `state` vérifié au retour ;
aucun secret client. La seule portée demandée est
`https://www.googleapis.com/auth/carddav`, avec `access_type=offline` pour
recevoir un jeton de renouvellement. Le navigateur s'ouvre sur la page de
Google ; le retour arrive par `fr.lemesre.unicontacts:/oauth2redirect`,
que capte une activité de l'application.

**Identifiant client** : saisi par l'utilisateur dans l'application, gardé
dans ses préférences ; il n'est pas versionné dans le dépôt.

**Jetons** : le jeton de renouvellement est chiffré par le Keystore comme un
mot de passe de source (ADR-0010) ; le jeton d'accès, valable une heure,
n'existe qu'en mémoire, le temps d'une synchronisation. Ni l'un ni l'autre
ne s'affiche au `Debug` ni n'entre dans un message d'erreur. Un refus
`invalid_grant` (jeton révoqué ou expiré) arrête la source avec une demande
de nouvelle autorisation, sans toucher au référentiel.

**Carnet** : `https://www.googleapis.com/carddav/v1/principals/<adresse>/lists/default/`,
en vCard 3.0, sous `Authorization: Bearer`. Le connecteur y liste les
cartes par `PROPFIND` (ETags) puis `addressbook-multiget` par lots, au lieu
de `addressbook-query` ; écriture et suppression restent celles du
connecteur CardDAV, sous `If-Match`.

**Code** : le module `oauth` du connecteur CardDAV construit la demande,
échange le code et renouvelle le jeton, par ureq (ADR-0007). Il emploie
directement `ring` (aléa et SHA-256) et `serde_json`, déjà présents dans
l'arbre des dépendances (par rustls et UniFFI) : aucune bibliothèque neuve
n'entre dans l'application.

## Alternatives écartées

### Compte Google du gestionnaire de comptes de /e/OS

Écarté par l'utilisateur (D-2026-11-15) : dépendance à un composant tiers
dont les envois vers Google sont signalés défaillants.

### Import d'un export `.vcf`

Écarté par l'utilisateur : ce n'est pas une synchronisation.

### Bibliothèque AppAuth

Standard du flux natif, mais peu maintenue et seconde pile HTTP ; le flux
tient en quelques fonctions éprouvées par les vecteurs de la RFC 7636.

## Conséquences

- Ce que cela apporte : Google rejoint le connecteur CardDAV existant ;
  seule l'authentification et la manière de lister changent.
- Ce que cela coûte : l'utilisateur crée et tient son client OAuth (projet
  Google Cloud, client Android, schéma d'URI personnalisé activé, écran de
  consentement publié en production) ; Google affiche « application non
  vérifiée » à la connexion. En mode test, les jetons expirent en sept jours.
- Ce que cela ferme : rien de durable ; une vérification par Google de
  l'application ne changerait que l'identifiant client.

## Vérification

- Tests unitaires : vecteur de l'annexe B de la RFC 7636 ; demande
  d'autorisation complète ; échange et renouvellement contre un serveur
  local ; refus `invalid_grant` ; `Debug` des jetons masqué.
- Test d'intégration : lecture par `PROPFIND` puis `addressbook-multiget`
  contre Radicale (ADR-0008).
- Procédure manuelle sur l'appareil, avec un vrai compte Google
  (`docs/verification-manuelle/android.md`).

## Références

- `req~source-google~1` ; D-2026-11-15 ; ADR-0007, ADR-0008, ADR-0010 ;
  ToDo-0006 ; RFC 6749, RFC 7636, RFC 8252, RFC 6352.

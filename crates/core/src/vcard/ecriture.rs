//! Écriture des vCard : réécriture fidèle vers une source.

use std::collections::{BTreeMap, BTreeSet};

use super::{Annuaire, Carte, Interpretee, Memoire, Parametre, Propriete, Version, relations};
use crate::champ::{Champ, Valeur};
use crate::fiche::Fiche;
use crate::normalisation::{cle, composantes};

/// Longueur maximale d'une ligne physique, en octets, fin de ligne exclue
/// (RFC 6350, § 3.2).
const LONGUEUR_MAX: usize = 75;

/// Vrai si la valeur du champ est du texte à échapper ; les champs
/// structurés gardent leurs échappements, et une URL n'en porte pas.
const fn est_texte(champ: Champ) -> bool {
    !matches!(
        champ,
        Champ::NomStructure | Champ::Adresse | Champ::Organisation | Champ::Url | Champ::Relation
    )
}

/// Valeur décodée d'une propriété interprétée.
pub(super) fn decoder(champ: Champ, valeur: &str) -> String {
    if !est_texte(champ) {
        return valeur.to_owned();
    }
    let mut decodee = String::with_capacity(valeur.len());
    let mut caracteres = valeur.chars();
    while let Some(c) = caracteres.next() {
        if c != '\\' {
            decodee.push(c);
            continue;
        }
        match caracteres.next() {
            Some('n' | 'N') => decodee.push('\n'),
            Some(autre) => decodee.push(autre),
            None => decodee.push('\\'),
        }
    }
    decodee
}

/// Valeur encodée pour une propriété.
pub(super) fn encoder(champ: Champ, texte: &str) -> String {
    if !est_texte(champ) {
        return texte.to_owned();
    }
    let mut encodee = String::with_capacity(texte.len());
    for c in texte.chars() {
        match c {
            '\\' => encodee.push_str("\\\\"),
            ',' => encodee.push_str("\\,"),
            ';' => encodee.push_str("\\;"),
            '\n' => encodee.push_str("\\n"),
            '\r' => {}
            autre => encodee.push(autre),
        }
    }
    encodee
}

impl Propriete {
    /// Propriété produite par le cœur pour une valeur de champ.
    fn produite(champ: Champ, valeur: &Valeur) -> Self {
        let parametres = if valeur.types.is_empty() {
            Vec::new()
        } else {
            vec![Parametre {
                nom: "TYPE".to_owned(),
                valeurs: valeur.types.clone(),
            }]
        };
        Self::nouvelle(
            champ.propriete_vcard(),
            parametres,
            encoder(champ, &valeur.texte),
        )
    }

    /// Propriété produite par le cœur.
    pub(super) fn nouvelle(nom: &str, parametres: Vec<Parametre>, valeur: String) -> Self {
        Self {
            groupe: None,
            nom: nom.to_owned(),
            parametres,
            valeur,
            brut: None,
        }
    }

    /// Lignes physiques de la propriété, séparées par CRLF : celles
    /// d'origine si elle a été lue, sinon sa ligne logique repliée.
    fn lignes(&self) -> String {
        if let Some(brut) = &self.brut {
            return brut.clone();
        }
        let mut ligne = String::new();
        if let Some(groupe) = &self.groupe {
            ligne.push_str(groupe);
            ligne.push('.');
        }
        ligne.push_str(&self.nom);
        for parametre in &self.parametres {
            ligne.push(';');
            ligne.push_str(&parametre.nom);
            ligne.push('=');
            let valeurs: Vec<String> = parametre
                .valeurs
                .iter()
                .map(|v| {
                    if v.contains([':', ';', ',']) {
                        format!("\"{v}\"")
                    } else {
                        v.clone()
                    }
                })
                .collect();
            ligne.push_str(&valeurs.join(","));
        }
        ligne.push(':');
        ligne.push_str(&self.valeur);
        replier(&ligne)
    }
}

/// Replie une ligne logique en lignes d'au plus 75 octets, sans couper un
/// caractère : chaque suite commence par une espace.
fn replier(ligne: &str) -> String {
    let mut resultat = String::with_capacity(ligne.len() + ligne.len() / LONGUEUR_MAX * 3);
    let mut longueur = 0;
    for c in ligne.chars() {
        if longueur + c.len_utf8() > LONGUEUR_MAX {
            resultat.push_str("\r\n ");
            longueur = 1;
        }
        resultat.push(c);
        longueur += c.len_utf8();
    }
    resultat
}

impl Carte {
    /// Texte de la carte, lignes terminées par CRLF.
    // [impl->req~vcard-3-et-4~1]
    #[must_use]
    pub fn ecrire(&self) -> String {
        let mut texte = format!("BEGIN:VCARD\r\nVERSION:{}\r\n", self.version.texte());
        for propriete in &self.proprietes {
            texte.push_str(&propriete.lignes());
            texte.push_str("\r\n");
        }
        texte.push_str("END:VCARD\r\n");
        texte
    }

    /// Carte à écrire vers une source pour une fiche.
    ///
    /// `precedente` est la dernière carte connue de cette source pour la
    /// fiche, s'il y en a une. Ses propriétés non interprétées sont reprises
    /// telles quelles, à leur place ; ses propriétés interprétées dont la
    /// valeur (normalisée) et les types n'ont pas changé aussi. Les autres
    /// disparaissent, et les valeurs nouvelles de la fiche s'ajoutent à la
    /// fin. Une carte neuve reçoit l'UID donné.
    ///
    /// La carte est écrite dans la `version` de la source. Elle porte `FN`,
    /// et `N` en 3.0, que ces versions exigent — sauf si la carte précédente
    /// s'en passait : la source l'acceptait ainsi, et le cœur ne lui ajoute
    /// rien qu'elle n'ait demandé.
    // [impl->req~conservation-proprietes-inconnues~1]
    // [impl->req~vcard-3-et-4~1]
    #[must_use]
    pub fn reecrire(precedente: Option<&Self>, version: Version, uid: &str, fiche: &Fiche) -> Self {
        Self::reecrire_avec(
            precedente,
            version,
            uid,
            fiche,
            &Annuaire::new(),
            &Memoire::new(),
        )
    }

    /// Comme [`Carte::reecrire`], les relations de la fiche écrites selon
    /// l'annuaire de la source et la mémoire de la copie (ADR-0011). Une
    /// relation dont la fiche liée ne se désigne pas dans la source n'est
    /// pas écrite.
    // [impl->req~conservation-proprietes-inconnues~1]
    // [impl->req~vcard-3-et-4~1]
    // [impl->req~synchronisation-relations~1]
    #[must_use]
    pub fn reecrire_avec(
        precedente: Option<&Self>,
        version: Version,
        uid: &str,
        fiche: &Fiche,
        annuaire: &Annuaire,
        memoire: &Memoire,
    ) -> Self {
        let mut restantes: BTreeMap<Champ, Vec<&Valeur>> = fiche
            .champs()
            .map(|(champ, valeurs)| (champ, valeurs.iter().collect()))
            .collect();
        let mut proprietes = Vec::new();
        match precedente {
            Some(precedente) => {
                let interpretees = precedente.interpreter(annuaire, memoire);
                let mut gardees = BTreeSet::new();
                for (indice, interpretee) in &interpretees {
                    if retirer_identique(&mut restantes, interpretee) {
                        gardees.insert(*indice);
                        gardees.extend(interpretee.compagnon);
                    }
                }
                let compagnons: BTreeSet<usize> =
                    interpretees.values().filter_map(|i| i.compagnon).collect();
                for (indice, propriete) in precedente.proprietes.iter().enumerate() {
                    let interpretee =
                        interpretees.contains_key(&indice) || compagnons.contains(&indice);
                    if !interpretee || gardees.contains(&indice) {
                        proprietes.push(propriete.clone());
                    }
                }
            }
            None => proprietes.push(Self::uid_produit(uid)),
        }
        let mut groupe = relations::dernier_groupe(proprietes.iter());
        for (champ, valeurs) in restantes {
            for valeur in valeurs {
                if champ == Champ::Relation {
                    proprietes.extend(relations::produire(
                        &valeur.texte,
                        version,
                        annuaire,
                        &mut groupe,
                    ));
                } else {
                    proprietes.push(Propriete::produite(champ, valeur));
                }
            }
        }
        for nom in ["FN", "N"] {
            let exigee = (nom == "FN" || version == Version::V3)
                && precedente.is_none_or(|p| p.proprietes.iter().any(|q| q.nom == nom))
                && !proprietes.iter().any(|q| q.nom == nom);
            if exigee {
                let valeur = if nom == "FN" {
                    nom_de_repli(fiche)
                } else {
                    ";;;;".to_owned()
                };
                proprietes.push(Propriete::nouvelle(nom, Vec::new(), valeur));
            }
        }
        Self {
            version,
            proprietes,
        }
    }

    fn uid_produit(uid: &str) -> Propriete {
        Propriete::nouvelle("UID", Vec::new(), uid.to_owned())
    }
}

/// Retire des valeurs restantes celle qu'écrit déjà la propriété
/// interprétée, et dit si elle y était.
fn retirer_identique(
    restantes: &mut BTreeMap<Champ, Vec<&Valeur>>,
    interpretee: &Interpretee,
) -> bool {
    let champ = interpretee.champ;
    let Some(valeurs) = restantes.get_mut(&champ) else {
        return false;
    };
    let cle_ecrite = cle(champ, &interpretee.texte);
    let Some(position) = valeurs
        .iter()
        .position(|v| cle(champ, &v.texte) == cle_ecrite && v.types == interpretee.types)
    else {
        return false;
    };
    valeurs.remove(position);
    true
}

/// Nom affiché de repli d'une fiche qui n'en a pas : prénom et nom, sinon
/// organisation, sinon premier e-mail, sinon vide.
fn nom_de_repli(fiche: &Fiche) -> String {
    let nom = fiche.valeurs(Champ::NomStructure).first().map(|v| {
        let parties = composantes(&v.texte);
        let morceaux: Vec<String> = [parties.get(1), parties.first()]
            .into_iter()
            .flatten()
            .map(|p| decoder(Champ::NomAffiche, p).trim().to_owned())
            .filter(|p| !p.is_empty())
            .collect();
        morceaux.join(" ")
    });
    let repli = nom
        .filter(|n| !n.is_empty())
        .or_else(|| {
            fiche
                .valeurs(Champ::Organisation)
                .first()
                .and_then(|v| composantes(&v.texte).into_iter().next())
                .map(|o| decoder(Champ::NomAffiche, &o))
        })
        .or_else(|| fiche.valeurs(Champ::Email).first().map(|v| v.texte.clone()))
        .unwrap_or_default();
    encoder(Champ::NomAffiche, &repli)
}

//! Lecture des vCard : dépliage des lignes, découpage des propriétés.

use super::{Carte, ErreurVcard, Parametre, Propriete, Version};

/// Ligne logique : lignes physiques dépliées, avec leur forme d'origine.
struct LigneLogique {
    /// Numéro de la première ligne physique, à partir de 1.
    numero: usize,
    /// Contenu déplié.
    contenu: String,
    /// Lignes physiques d'origine, séparées par CRLF.
    brut: String,
}

/// Lit toutes les vCard d'un texte, dans leur ordre.
///
/// Les fins de ligne CRLF et LF sont admises ; une ligne qui commence par
/// une espace ou une tabulation continue la précédente (RFC 6350, § 3.2).
///
/// # Errors
///
/// [`ErreurVcard`] à la première ligne qui ne peut se lire : propriété hors
/// d'une carte, carte non fermée, version absente ou autre que 3.0 et 4.0,
/// ligne sans `:`.
// [impl->req~vcard-3-et-4~1]
pub fn lire(texte: &str) -> Result<Vec<Carte>, ErreurVcard> {
    let mut cartes = Vec::new();
    // Carte ouverte : ligne du BEGIN, version lue, propriétés lues.
    let mut ouverte: Option<(usize, Option<Version>, Vec<Propriete>)> = None;
    for ligne in deplier(texte) {
        let propriete = decouper(&ligne)?;
        let Some((debut, version, proprietes)) = ouverte.as_mut() else {
            if propriete.nom == "BEGIN" && propriete.valeur.trim().eq_ignore_ascii_case("VCARD") {
                ouverte = Some((ligne.numero, None, Vec::new()));
                continue;
            }
            return Err(ErreurVcard::HorsCarte {
                ligne: ligne.numero,
            });
        };
        match propriete.nom.as_str() {
            "END" if propriete.valeur.trim().eq_ignore_ascii_case("VCARD") => {
                let version = version.ok_or(ErreurVcard::VersionManquante { ligne: *debut })?;
                cartes.push(Carte {
                    version,
                    proprietes: std::mem::take(proprietes),
                });
                ouverte = None;
            }
            "VERSION" => {
                *version = Some(Version::depuis_texte(&propriete.valeur).ok_or_else(|| {
                    ErreurVcard::VersionNonPriseEnCharge {
                        ligne: ligne.numero,
                        version: propriete.valeur.trim().to_owned(),
                    }
                })?);
            }
            _ => proprietes.push(propriete),
        }
    }
    match ouverte {
        Some((debut, _, _)) => Err(ErreurVcard::FinManquante { ligne: debut }),
        None => Ok(cartes),
    }
}

/// Déplie les lignes physiques en lignes logiques ; les lignes vides sont
/// ignorées.
fn deplier(texte: &str) -> Vec<LigneLogique> {
    let mut lignes: Vec<LigneLogique> = Vec::new();
    for (indice, physique) in texte.split('\n').enumerate() {
        let physique = physique.strip_suffix('\r').unwrap_or(physique);
        let suite = physique
            .strip_prefix(' ')
            .or_else(|| physique.strip_prefix('\t'));
        match (suite, lignes.last_mut()) {
            (Some(suite), Some(courante)) => {
                courante.contenu.push_str(suite);
                courante.brut.push_str("\r\n");
                courante.brut.push_str(physique);
            }
            _ if physique.trim().is_empty() => {}
            _ => lignes.push(LigneLogique {
                numero: indice + 1,
                contenu: physique.to_owned(),
                brut: physique.to_owned(),
            }),
        }
    }
    lignes
}

/// Découpe une ligne logique en propriété : `[groupe.]nom *(;paramètre):valeur`.
fn decouper(ligne: &LigneLogique) -> Result<Propriete, ErreurVcard> {
    let invalide = || ErreurVcard::LigneInvalide {
        ligne: ligne.numero,
    };
    let contenu = ligne.contenu.as_str();
    let fin_nom = contenu.find([';', ':']).ok_or_else(invalide)?;
    let (nom_complet, mut reste) = contenu.split_at(fin_nom);
    let (groupe, nom) = match nom_complet.split_once('.') {
        Some((groupe, nom)) => (Some(groupe.to_owned()), nom),
        None => (None, nom_complet),
    };
    if nom.is_empty() {
        return Err(invalide());
    }
    let mut parametres = Vec::new();
    while let Some(suite) = reste.strip_prefix(';') {
        let (parametre, apres) = lire_parametre(suite).ok_or_else(invalide)?;
        parametres.push(parametre);
        reste = apres;
    }
    let valeur = reste.strip_prefix(':').ok_or_else(invalide)?;
    Ok(Propriete {
        groupe,
        nom: nom.to_ascii_uppercase(),
        parametres,
        valeur: valeur.to_owned(),
        brut: Some(ligne.brut.clone()),
    })
}

/// Lit un paramètre en tête de `texte` et rend le reste, qui commence par
/// `;` ou `:`. Un paramètre sans nom (`TEL;CELL:…`, admis par l'usage en
/// 3.0) vaut `TYPE`.
fn lire_parametre(texte: &str) -> Option<(Parametre, &str)> {
    let fin_nom = texte.find(['=', ';', ':'])?;
    let (nom, reste) = texte.split_at(fin_nom);
    let Some(mut reste) = reste.strip_prefix('=') else {
        let parametre = Parametre {
            nom: "TYPE".to_owned(),
            valeurs: vec![nom.to_owned()],
        };
        return Some((parametre, reste));
    };
    let mut valeurs = Vec::new();
    loop {
        let (valeur, apres) = lire_valeur_de_parametre(reste)?;
        valeurs.push(valeur);
        match apres.strip_prefix(',') {
            Some(suite) => reste = suite,
            None => {
                reste = apres;
                break;
            }
        }
    }
    let parametre = Parametre {
        nom: nom.to_ascii_uppercase(),
        valeurs,
    };
    Some((parametre, reste))
}

/// Lit une valeur de paramètre, entre guillemets ou non.
fn lire_valeur_de_parametre(texte: &str) -> Option<(String, &str)> {
    if let Some(entre) = texte.strip_prefix('"') {
        let fin = entre.find('"')?;
        let (valeur, apres) = entre.split_at(fin);
        Some((valeur.to_owned(), apres.get(1..)?))
    } else {
        let fin = texte.find([',', ';', ':'])?;
        let (valeur, apres) = texte.split_at(fin);
        Some((valeur.to_owned(), apres))
    }
}

//! Forme normalisée des valeurs, pour les comparer.
//!
//! Deux valeurs d'un même champ sont égales quand leurs clés normalisées le
//! sont : `06 14 00 00 00` et `+33614000000` désignent le même numéro. La
//! clé ne sert qu'à comparer ; la valeur affichée et écrite reste celle
//! qu'a saisie l'utilisateur ou que porte la source.

use crate::champ::Champ;

/// Indicatif téléphonique appliqué aux numéros nationaux (`0…`), faute de
/// réglage du pays de l'utilisateur.
pub const INDICATIF_PAR_DEFAUT: &str = "33";

/// Clé normalisée d'une valeur, selon son champ.
// [impl->req~multivalues-par-valeur-normalisee~1]
#[must_use]
pub fn cle(champ: Champ, texte: &str) -> String {
    match champ {
        Champ::Telephone => telephone(texte),
        Champ::Email => email(texte),
        Champ::Anniversaire => date(texte),
        Champ::NomStructure | Champ::Adresse | Champ::Organisation => structure(texte),
        Champ::Note => texte.trim().replace("\r\n", "\n"),
        Champ::NomAffiche | Champ::Fonction | Champ::Url => espaces(texte),
        Champ::Relation => texte.trim().to_owned(),
        Champ::Groupes => crate::groupes::chemin(texte),
    }
}

/// Numéro au format international : chiffres seuls, précédés de `+`.
fn telephone(texte: &str) -> String {
    let sans_schema = texte.trim();
    let sans_schema = sans_schema
        .strip_prefix("tel:")
        .or_else(|| sans_schema.strip_prefix("TEL:"))
        .unwrap_or(sans_schema);
    // Une URI tel: peut porter des paramètres (`;ext=…`) : ils ne font pas
    // partie du numéro.
    let numero = sans_schema.split(';').next().unwrap_or_default();
    let international = numero.trim_start().starts_with('+');
    let chiffres: String = numero.chars().filter(char::is_ascii_digit).collect();
    if international {
        format!("+{chiffres}")
    } else if let Some(reste) = chiffres.strip_prefix("00") {
        format!("+{reste}")
    } else if let Some(reste) = chiffres.strip_prefix('0') {
        format!("+{INDICATIF_PAR_DEFAUT}{reste}")
    } else {
        chiffres
    }
}

fn email(texte: &str) -> String {
    let texte = texte.trim();
    let sans_schema = texte
        .get(..7)
        .filter(|debut| debut.eq_ignore_ascii_case("mailto:"))
        .and_then(|_| texte.get(7..))
        .unwrap_or(texte);
    sans_schema.trim().to_lowercase()
}

/// Date sans séparateurs ni heure : `1980-05-17` et `19800517` se
/// rejoignent, comme `--05-17` et `--0517` (date sans année).
fn date(texte: &str) -> String {
    let jour = texte.trim().split('T').next().unwrap_or_default();
    let chiffres: String = jour.chars().filter(char::is_ascii_digit).collect();
    if jour.starts_with("--") {
        format!("--{chiffres}")
    } else {
        chiffres
    }
}

/// Composantes séparées par `;` (non échappé), chacune aux espaces
/// normalisés ; les composantes vides de fin ne comptent pas.
fn structure(texte: &str) -> String {
    let mut composantes: Vec<String> = composantes(texte).iter().map(|c| espaces(c)).collect();
    while composantes.last().is_some_and(String::is_empty) {
        composantes.pop();
    }
    composantes.join(";")
}

/// Découpe une valeur structurée sur ses `;` non échappés.
pub(crate) fn composantes(texte: &str) -> Vec<String> {
    let mut resultat = Vec::new();
    let mut courante = String::new();
    let mut echappe = false;
    for c in texte.chars() {
        if echappe {
            courante.push('\\');
            courante.push(c);
            echappe = false;
        } else if c == '\\' {
            echappe = true;
        } else if c == ';' {
            resultat.push(std::mem::take(&mut courante));
        } else {
            courante.push(c);
        }
    }
    if echappe {
        courante.push('\\');
    }
    resultat.push(courante);
    resultat
}

/// Espaces de début et de fin retirés, suites d'espaces réduites à une.
fn espaces(texte: &str) -> String {
    texte.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod telephone {
        use super::*;

        // [utest->req~multivalues-par-valeur-normalisee~1]
        #[test]
        fn un_numero_national_et_sa_forme_internationale_sont_egaux() {
            assert_eq!(
                cle(Champ::Telephone, "06 14 00 00 00"),
                cle(Champ::Telephone, "+33614000000")
            );
        }

        #[test]
        fn le_prefixe_00_vaut_le_signe_plus() {
            assert_eq!(cle(Champ::Telephone, "0033 6 39 98 00 00"), "+33639980000");
        }

        #[test]
        fn une_uri_tel_perd_son_schema_et_ses_parametres() {
            assert_eq!(
                cle(Champ::Telephone, "tel:+33-6-39-98-00-00;ext=12"),
                "+33639980000"
            );
        }

        #[test]
        fn points_tirets_et_parentheses_sont_ignores() {
            assert_eq!(cle(Champ::Telephone, "(06).39-98.00.00"), "+33639980000");
        }
    }

    #[test]
    fn un_email_se_compare_sans_casse_ni_schema() {
        assert_eq!(
            cle(Champ::Email, " mailto:Camille@Example.ORG "),
            cle(Champ::Email, "camille@example.org")
        );
    }

    #[test]
    fn une_date_de_vcard_3_et_sa_forme_vcard_4_sont_egales() {
        assert_eq!(
            cle(Champ::Anniversaire, "1980-05-17"),
            cle(Champ::Anniversaire, "19800517")
        );
    }

    #[test]
    fn une_date_sans_annee_garde_sa_marque() {
        assert_eq!(cle(Champ::Anniversaire, "--05-17"), "--0517");
    }

    #[test]
    fn un_nom_structure_ignore_ses_composantes_vides_de_fin() {
        assert_eq!(
            cle(Champ::NomStructure, "Exemple; Camille ;;;"),
            cle(Champ::NomStructure, "Exemple;Camille")
        );
    }

    #[test]
    fn un_point_virgule_echappe_ne_separe_pas_les_composantes() {
        assert_eq!(composantes(r"A\;B;C"), [r"A\;B", "C"]);
    }

    #[test]
    fn un_nom_affiche_ignore_les_espaces_superflus() {
        assert_eq!(
            cle(Champ::NomAffiche, "  Camille   Exemple "),
            "Camille Exemple"
        );
    }
}

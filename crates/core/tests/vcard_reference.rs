//! Jeux de vCard de référence : chaque carte lue puis réécrite sans
//! modification vers sa source est équivalente à l'originale, propriété par
//! propriété.

use unicontacts_core::vcard::{Carte, Propriete, lire};
use unicontacts_core::{Fiche, Horodatage, SourceId};

const JEUX: [(&str, &str); 3] = [
    ("reference-3.0.vcf", include_str!("vcard/reference-3.0.vcf")),
    ("reference-4.0.vcf", include_str!("vcard/reference-4.0.vcf")),
    ("inconnues.vcf", include_str!("vcard/inconnues.vcf")),
];

/// Ce que compare l'équivalence : groupe, nom, paramètres et valeur.
fn forme(propriete: &Propriete) -> String {
    format!(
        "{:?}|{}|{:?}|{}",
        propriete.groupe, propriete.nom, propriete.parametres, propriete.valeur
    )
}

/// Carte réécrite vers sa source à partir des seules valeurs qu'elle porte.
fn reecrite(carte: &Carte) -> Result<Carte, String> {
    let source = SourceId::nouveau();
    let mut fiche = Fiche::nouvelle();
    for (champ, valeurs) in carte.valeurs(source, Horodatage::depuis_millis(1)) {
        fiche.definir(champ, valeurs).map_err(|e| e.to_string())?;
    }
    let uid = carte.uid().unwrap_or_default();
    let texte = Carte::reecrire(Some(carte), carte.version, &uid, &fiche).ecrire();
    let mut relues = lire(&texte).map_err(|e| e.to_string())?;
    relues.pop().ok_or_else(|| "aucune carte relue".to_owned())
}

// [utest->req~vcard-3-et-4~1]
#[test]
fn chaque_carte_de_reference_reecrite_sans_modification_est_equivalente() -> Result<(), String> {
    for (nom, texte) in JEUX {
        for carte in lire(texte).map_err(|e| format!("{nom} : {e}"))? {
            let relue = reecrite(&carte)?;

            let attendu: Vec<_> = carte.proprietes.iter().map(forme).collect();
            let obtenu: Vec<_> = relue.proprietes.iter().map(forme).collect();
            assert_eq!(
                (relue.version, obtenu),
                (carte.version, attendu),
                "{nom}, carte {:?}",
                carte.uid()
            );
        }
    }
    Ok(())
}

// [utest->req~vcard-3-et-4~1]
#[test]
fn chaque_jeu_de_reference_reecrit_sans_modification_est_identique_a_l_octet_pres()
-> Result<(), String> {
    for (nom, texte) in JEUX {
        let mut reecrit = String::new();
        for carte in lire(texte).map_err(|e| format!("{nom} : {e}"))? {
            reecrit.push_str(&reecrite(&carte)?.ecrire());
        }

        assert_eq!(reecrit, texte, "{nom}");
    }
    Ok(())
}

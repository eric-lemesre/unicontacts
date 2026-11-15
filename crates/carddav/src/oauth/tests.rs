// [utest->req~source-google~1]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

use super::*;

type Essai = Result<(), Box<dyn std::error::Error>>;

fn client(echange: &str) -> ClientOAuth {
    ClientOAuth {
        identifiant: "123-abc.apps.googleusercontent.com".to_owned(),
        retour: "fr.lemesre.unicontacts:/oauth2redirect".to_owned(),
        points: Points {
            autorisation: "https://accounts.example/auth".to_owned(),
            echange: echange.to_owned(),
        },
    }
}

/// Serveur d'autorisation local, qui répond une fois `statut` et `corps`,
/// et rend la requête reçue.
fn serveur(
    statut: u16,
    corps: &'static str,
) -> Result<(String, mpsc::Receiver<String>), std::io::Error> {
    let ecoute = TcpListener::bind("127.0.0.1:0")?;
    let adresse = format!("http://{}/token", ecoute.local_addr()?);
    let (envoi, reception) = mpsc::channel();
    thread::spawn(move || {
        let Ok((mut flux, _)) = ecoute.accept() else {
            return;
        };
        let mut lu = Vec::new();
        let mut tampon = [0_u8; 4096];
        loop {
            let n = flux.read(&mut tampon).unwrap_or(0);
            lu.extend(tampon.iter().take(n));
            let texte = String::from_utf8_lossy(&lu).into_owned();
            let complet = texte
                .split_once("\r\n\r\n")
                .is_some_and(|(entetes, reste)| {
                    let longueur = entetes
                        .to_ascii_lowercase()
                        .lines()
                        .find_map(|l| {
                            l.strip_prefix("content-length:")
                                .map(str::trim)
                                .map(str::to_owned)
                        })
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(0);
                    reste.len() >= longueur
                });
            if n == 0 || complet {
                let _ = envoi.send(texte);
                break;
            }
        }
        let _ = write!(
            flux,
            "HTTP/1.1 {statut} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corps}",
            corps.len()
        );
    });
    Ok((adresse, reception))
}

#[test]
fn le_defi_suit_le_vecteur_de_l_annexe_b_de_la_rfc_7636() {
    assert_eq!(
        defi("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[test]
fn la_demande_porte_le_client_le_retour_la_portee_le_defi_et_l_etat() -> Essai {
    let demande = client("http://localhost/token").demander(Some("alice@gmail.com"))?;
    let attendu = format!(
        "https://accounts.example/auth?client_id=123-abc.apps.googleusercontent.com\
         &redirect_uri=fr.lemesre.unicontacts%3A%2Foauth2redirect&response_type=code\
         &scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcarddav&access_type=offline\
         &prompt=consent&code_challenge_method=S256&state={}&code_challenge={}\
         &login_hint=alice%40gmail.com",
        demande.etat,
        defi(demande.verificateur.valeur())
    );
    assert_eq!(demande.url, attendu);
    Ok(())
}

#[test]
fn deux_demandes_n_ont_ni_le_meme_verificateur_ni_le_meme_etat() -> Essai {
    let client = client("http://localhost/token");
    let (a, b) = (client.demander(None)?, client.demander(None)?);
    assert!(a.verificateur != b.verificateur && a.etat != b.etat);
    Ok(())
}

#[test]
fn le_verificateur_a_la_longueur_que_demande_la_rfc() -> Essai {
    let longueur = client("http://localhost/token")
        .demander(None)?
        .verificateur
        .valeur()
        .len();
    assert!((43..=128).contains(&longueur), "longueur {longueur}");
    Ok(())
}

#[test]
fn l_echange_du_code_rend_les_jetons_et_presente_le_verificateur() -> Essai {
    let (adresse, requete) = serveur(
        200,
        r#"{"access_token":"ya29.acces","expires_in":3599,"refresh_token":"1//renouv","token_type":"Bearer"}"#,
    )?;
    let jetons = client(&adresse).echanger("4/code", &Jeton::new("verif"))?;
    let recue = requete.recv()?;
    assert_eq!(
        (
            jetons,
            recue.contains("grant_type=authorization_code&code=4%2Fcode&code_verifier=verif")
        ),
        (
            Jetons {
                acces: Jeton::new("ya29.acces"),
                renouvellement: Some(Jeton::new("1//renouv")),
                expire_dans: 3599
            },
            true
        )
    );
    Ok(())
}

#[test]
fn le_renouvellement_rend_un_jeton_d_acces_neuf() -> Essai {
    let (adresse, requete) = serveur(200, r#"{"access_token":"ya29.neuf","expires_in":3599}"#)?;
    let acces = client(&adresse).renouveler(&Jeton::new("1//renouv"))?;
    let recue = requete.recv()?;
    assert_eq!(
        (
            acces,
            recue.contains("grant_type=refresh_token&refresh_token=1%2F%2Frenouv")
        ),
        (Jeton::new("ya29.neuf"), true)
    );
    Ok(())
}

#[test]
fn un_jeton_revoque_demande_une_nouvelle_autorisation() -> Essai {
    let (adresse, _requete) = serveur(
        400,
        r#"{"error":"invalid_grant","error_description":"Token has been expired or revoked."}"#,
    )?;
    let erreur = client(&adresse)
        .renouveler(&Jeton::new("1//revoque"))
        .err()
        .ok_or("le renouvellement aurait dû échouer")?;
    assert!(erreur.autorisation_a_renouveler(), "{erreur:?}");
    Ok(())
}

#[test]
fn un_serveur_injoignable_ne_demande_pas_de_nouvelle_autorisation() -> Essai {
    let erreur = client("http://127.0.0.1:9/token")
        .renouveler(&Jeton::new("1//renouv"))
        .err()
        .ok_or("le renouvellement aurait dû échouer")?;
    assert!(!erreur.autorisation_a_renouveler(), "{erreur:?}");
    Ok(())
}

#[test]
fn une_reponse_sans_jeton_d_acces_est_illisible() {
    assert!(matches!(
        lire_reponse(r#"{"token_type":"Bearer"}"#),
        Err(ErreurOAuth::ReponseIllisible(_))
    ));
}

#[test]
fn le_debug_des_jetons_ne_montre_aucun_jeton() {
    let jetons = Jetons {
        acces: Jeton::new("ya29.secret"),
        renouvellement: Some(Jeton::new("1//secret")),
        expire_dans: 1,
    };
    assert!(!format!("{jetons:?}").contains("secret"));
}

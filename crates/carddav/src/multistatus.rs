//! Lecture des réponses `207 Multi-Status` (RFC 4918, § 13) d'un
//! `REPORT addressbook-query` (RFC 6352, § 8.6).

use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::{Namespace, ResolveResult};

use crate::ErreurCardDav;

const DAV: &str = "DAV:";
const CARDDAV: &str = "urn:ietf:params:xml:ns:carddav";

/// Ressource décrite par une réponse : son chemin, son ETag et sa vCard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ressource {
    /// Chemin (`href`) de la ressource, tel que le serveur l'écrit.
    pub href: String,
    /// ETag de la ressource, guillemets compris.
    pub etag: Option<String>,
    /// Texte de la vCard.
    pub contenu: String,
}

/// Élément dont on recueille le texte.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Champ {
    Href,
    Etag,
    Donnees,
    Statut,
}

/// Réponse en cours de lecture.
#[derive(Default)]
struct Reponse {
    href: String,
    etag: Option<String>,
    donnees: Option<String>,
    /// Vrai si le propstat lu porte un statut 200.
    propstat_ok: bool,
    /// Valeurs du propstat en cours, retenues seulement s'il est en 200.
    etag_lu: Option<String>,
    donnees_lues: Option<String>,
}

/// Ressources d'une réponse multistatus : celles qui portent une vCard dans
/// un propstat de statut 200. La collection elle-même et les ressources
/// sans données sont ignorées.
///
/// # Errors
///
/// [`ErreurCardDav::ReponseIllisible`] si le XML est mal formé.
// [impl->req~source-carddav~1]
pub fn lire(xml: &str) -> Result<Vec<Ressource>, ErreurCardDav> {
    Ok(reponses(xml)?
        .into_iter()
        .filter_map(|(href, etag, donnees)| {
            donnees.map(|contenu| Ressource {
                href,
                etag,
                contenu,
            })
        })
        .collect())
}

/// Chemins et ETags d'une réponse multistatus à un `PROPFIND` : les
/// ressources qui portent un ETag, hors collections (chemin terminé par
/// `/`).
///
/// # Errors
///
/// [`ErreurCardDav::ReponseIllisible`] si le XML est mal formé.
// [impl->req~source-google~1]
pub fn lire_etags(xml: &str) -> Result<Vec<(String, String)>, ErreurCardDav> {
    Ok(reponses(xml)?
        .into_iter()
        .filter(|(href, _, _)| !href.ends_with('/'))
        .filter_map(|(href, etag, _)| etag.map(|e| (href, e)))
        .collect())
}

/// Réponse lue : chemin, ETag et données retenus d'un propstat en 200.
type Lue = (String, Option<String>, Option<String>);

/// Chaque réponse d'un multistatus.
fn reponses(xml: &str) -> Result<Vec<Lue>, ErreurCardDav> {
    let mut lecteur = NsReader::from_str(xml);
    let mut ressources = Vec::new();
    let mut reponse: Option<Reponse> = None;
    let mut champ: Option<Champ> = None;
    let mut texte = String::new();
    loop {
        let (espace, evenement) = lecteur
            .read_resolved_event()
            .map_err(|e| ErreurCardDav::ReponseIllisible(e.to_string()))?;
        match evenement {
            Event::Start(debut) => {
                let nom = debut.local_name();
                match (espace_de(&espace), nom.as_ref()) {
                    (Some(DAV), "response") => reponse = Some(Reponse::default()),
                    (Some(DAV), "propstat") => {
                        if let Some(r) = reponse.as_mut() {
                            r.propstat_ok = false;
                            r.etag_lu = None;
                            r.donnees_lues = None;
                        }
                    }
                    (Some(DAV), "href") => champ = Some(Champ::Href),
                    (Some(DAV), "getetag") => champ = Some(Champ::Etag),
                    (Some(DAV), "status") => champ = Some(Champ::Statut),
                    (Some(CARDDAV), "address-data") => champ = Some(Champ::Donnees),
                    _ => {}
                }
                texte.clear();
            }
            Event::Text(t) => texte.push_str(&t.xml10_content()),
            Event::CData(c) => texte.push_str(&c.xml10_content()),
            Event::GeneralRef(r) => texte.push_str(&reference(&r)?),
            Event::End(fin) => {
                let nom = fin.local_name();
                if let (Some(c), Some(r)) = (champ.take(), reponse.as_mut()) {
                    let valeur = std::mem::take(&mut texte);
                    match c {
                        Champ::Href => r.href = valeur.trim().to_owned(),
                        Champ::Etag => r.etag_lu = Some(valeur.trim().to_owned()),
                        Champ::Donnees => r.donnees_lues = Some(valeur),
                        Champ::Statut => r.propstat_ok = statut_ok(&valeur),
                    }
                }
                match nom.as_ref() {
                    "propstat" => {
                        if let Some(r) = reponse.as_mut().filter(|r| r.propstat_ok) {
                            r.etag = r.etag_lu.take().or(r.etag.take());
                            r.donnees = r.donnees_lues.take().or(r.donnees.take());
                        }
                    }
                    "response" => {
                        if let Some(r) = reponse.take() {
                            ressources.push((r.href, r.etag, r.donnees));
                        }
                    }
                    _ => {}
                }
            }
            Event::Empty(_) => champ = None,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(ressources)
}

fn espace_de<'a>(resolu: &'a ResolveResult<'_>) -> Option<&'a str> {
    match resolu {
        ResolveResult::Bound(Namespace(espace)) => Some(espace),
        _ => None,
    }
}

/// Vrai pour une ligne de statut `HTTP/1.1 200 OK`.
fn statut_ok(ligne: &str) -> bool {
    ligne.split_whitespace().nth(1) == Some("200")
}

/// Texte d'une référence d'entité ou de caractère.
fn reference(r: &quick_xml::events::BytesRef<'_>) -> Result<String, ErreurCardDav> {
    if r.is_char_ref() {
        let caractere = r
            .resolve_char_ref()
            .map_err(|e| ErreurCardDav::ReponseIllisible(e.to_string()))?;
        return Ok(caractere.map(String::from).unwrap_or_default());
    }
    let nom = r.xml10_content();
    let texte = match nom.as_ref() {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        autre => {
            return Err(ErreurCardDav::ReponseIllisible(format!(
                "entité inconnue : {autre}"
            )));
        }
    };
    Ok(texte.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Réponse de Radicale 3.9 à un REPORT addressbook-query.
    const RADICALE: &str = r#"<?xml version='1.0' encoding='utf-8'?>
<multistatus xmlns="DAV:" xmlns:CR="urn:ietf:params:xml:ns:carddav"><response><href>/essai/carnet/u1.vcf</href><propstat><prop><getetag>"d5388"</getetag><CR:address-data>BEGIN:VCARD
VERSION:3.0
UID:u1
FN:Camille &amp; Dominique
END:VCARD
</CR:address-data></prop><status>HTTP/1.1 200 OK</status></propstat></response></multistatus>"#;

    /// Forme à préfixes et CDATA, comme en servent d'autres serveurs, avec
    /// la collection elle-même et une propriété introuvable.
    const PREFIXES: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:response><d:href>/carnet/</d:href>
    <d:propstat><d:prop><d:getetag/></d:prop><d:status>HTTP/1.1 404 Not Found</d:status></d:propstat>
  </d:response>
  <d:response><d:href>/carnet/a.vcf</d:href>
    <d:propstat><d:prop><d:getetag>"e1"</d:getetag><card:address-data><![CDATA[BEGIN:VCARD
VERSION:4.0
UID:a
FN:Alix <Démo>
END:VCARD
]]></card:address-data></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat>
    <d:propstat><d:prop><d:displayname/></d:prop><d:status>HTTP/1.1 404 Not Found</d:status></d:propstat>
  </d:response>
</d:multistatus>"#;

    /// Réponse à un PROPFIND des ETags : la collection, deux cartes, dont
    /// une sans ETag.
    const PROPFIND: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:">
  <d:response><d:href>/carnet/</d:href><d:propstat><d:prop><d:getetag>"c0"</d:getetag></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>
  <d:response><d:href>/carnet/a.vcf</d:href><d:propstat><d:prop><d:getetag>"e1"</d:getetag></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>
  <d:response><d:href>/carnet/b.vcf</d:href><d:propstat><d:prop><d:getetag/></d:prop><d:status>HTTP/1.1 404 Not Found</d:status></d:propstat></d:response>
</d:multistatus>"#;

    // [utest->req~source-google~1]
    #[test]
    fn un_propfind_donne_les_etags_des_cartes_hors_collection() -> Result<(), ErreurCardDav> {
        assert_eq!(
            lire_etags(PROPFIND)?,
            [("/carnet/a.vcf".to_owned(), "\"e1\"".to_owned())]
        );
        Ok(())
    }

    #[test]
    fn la_reponse_de_radicale_donne_href_etag_et_vcard() -> Result<(), ErreurCardDav> {
        let ressources = lire(RADICALE)?;

        assert_eq!(
            ressources,
            [Ressource {
                href: "/essai/carnet/u1.vcf".to_owned(),
                etag: Some("\"d5388\"".to_owned()),
                contenu: "BEGIN:VCARD\nVERSION:3.0\nUID:u1\nFN:Camille & Dominique\nEND:VCARD\n"
                    .to_owned(),
            }]
        );
        Ok(())
    }

    #[test]
    fn la_collection_et_les_proprietes_introuvables_sont_ignorees() -> Result<(), ErreurCardDav> {
        let hrefs: Vec<_> = lire(PREFIXES)?.into_iter().map(|r| r.href).collect();

        assert_eq!(hrefs, ["/carnet/a.vcf"]);
        Ok(())
    }

    #[test]
    fn une_vcard_en_cdata_est_lue_telle_quelle() -> Result<(), ErreurCardDav> {
        let contenus: Vec<_> = lire(PREFIXES)?.into_iter().map(|r| r.contenu).collect();

        assert_eq!(
            contenus,
            ["BEGIN:VCARD\nVERSION:4.0\nUID:a\nFN:Alix <Démo>\nEND:VCARD\n"]
        );
        Ok(())
    }

    #[test]
    fn un_xml_mal_forme_est_refuse() {
        let resultat = lire("<multistatus xmlns=\"DAV:\"><response></multistatus>");

        assert!(
            matches!(resultat, Err(ErreurCardDav::ReponseIllisible(_))),
            "{resultat:?}"
        );
    }
}

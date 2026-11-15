//! Canaux de communication d'une fiche (ADR-0014).
//!
//! Appel et SMS sont offerts par chaque numéro de la fiche, l'e-mail par
//! chaque adresse : ces canaux découlent de la fiche. Les messageries
//! (WhatsApp, Signal, Telegram, Matrix…) se déclarent, rattachées à un
//! numéro ou une adresse de la fiche, ou à un pseudonyme ; rattachées à un
//! numéro, elles disparaissent avec lui. Chaque fiche tient deux classements
//! de ses canaux, pro et perso, et une note par canal. Ces données restent
//! au référentiel : aucune source ne les porte.
// [impl->req~canaux-par-identifiant~1]

use std::collections::BTreeMap;

use crate::champ::Champ;
use crate::identifiants::FicheId;
use crate::normalisation::cle;
use crate::referentiel::Referentiel;

/// Type d'un canal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeCanal {
    /// Appel téléphonique.
    Appel,
    /// SMS.
    Sms,
    /// E-mail.
    Email,
    /// WhatsApp.
    WhatsApp,
    /// Signal.
    Signal,
    /// Telegram.
    Telegram,
    /// Matrix.
    Matrix,
    /// Autre messagerie, désignée par son nom.
    Autre(String),
}

/// Ce qui désigne le correspondant sur un canal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Identifiant {
    /// Un numéro de téléphone de la fiche.
    Numero,
    /// Une adresse électronique de la fiche.
    Adresse,
    /// Un numéro de la fiche, ou un pseudonyme.
    NumeroOuPseudonyme,
    /// Un pseudonyme, propre au canal.
    Pseudonyme,
}

impl TypeCanal {
    /// Messageries proposées, dans l'ordre de présentation.
    pub const MESSAGERIES: [Self; 4] = [Self::WhatsApp, Self::Signal, Self::Telegram, Self::Matrix];

    /// Code enregistré : un mot fixe, ou le nom d'une autre messagerie.
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Appel => "appel",
            Self::Sms => "sms",
            Self::Email => "email",
            Self::WhatsApp => "whatsapp",
            Self::Signal => "signal",
            Self::Telegram => "telegram",
            Self::Matrix => "matrix",
            Self::Autre(nom) => nom,
        }
    }

    /// Libellé affiché.
    #[must_use]
    pub fn libelle(&self) -> &str {
        match self {
            Self::Appel => "Appel",
            Self::Sms => "SMS",
            Self::Email => "E-mail",
            Self::WhatsApp => "WhatsApp",
            Self::Signal => "Signal",
            Self::Telegram => "Telegram",
            Self::Matrix => "Matrix",
            Self::Autre(nom) => nom,
        }
    }

    /// Type désigné par un code ou un libellé, sans tenir compte de la
    /// casse ; sinon une autre messagerie.
    #[must_use]
    pub fn depuis_texte(texte: &str) -> Self {
        let texte = texte.trim();
        [Self::Appel, Self::Sms, Self::Email]
            .into_iter()
            .chain(Self::MESSAGERIES)
            .find(|t| {
                t.code().eq_ignore_ascii_case(texte) || t.libelle().eq_ignore_ascii_case(texte)
            })
            .unwrap_or_else(|| Self::Autre(texte.to_owned()))
    }

    /// Ce qui désigne le correspondant sur ce canal.
    #[must_use]
    pub const fn identifiant(&self) -> Identifiant {
        match self {
            Self::Appel | Self::Sms | Self::WhatsApp | Self::Signal => Identifiant::Numero,
            Self::Email => Identifiant::Adresse,
            Self::Telegram => Identifiant::NumeroOuPseudonyme,
            Self::Matrix | Self::Autre(_) => Identifiant::Pseudonyme,
        }
    }
}

/// Canal d'une fiche : son type et l'identifiant qui y désigne le
/// correspondant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Canal {
    /// Type du canal.
    pub type_canal: TypeCanal,
    /// Numéro, adresse ou pseudonyme, tel qu'enregistré.
    pub identifiant: String,
}

impl Canal {
    /// Clé du canal : type et identifiant normalisé ; un même numéro écrit
    /// autrement donne la même clé.
    #[must_use]
    pub fn cle(&self) -> String {
        let identifiant = match self.type_canal.identifiant() {
            Identifiant::Numero => cle(Champ::Telephone, &self.identifiant),
            Identifiant::Adresse => cle(Champ::Email, &self.identifiant),
            Identifiant::NumeroOuPseudonyme if !self.identifiant.trim().starts_with('@') => {
                cle(Champ::Telephone, &self.identifiant)
            }
            Identifiant::NumeroOuPseudonyme | Identifiant::Pseudonyme => {
                self.identifiant.trim().to_owned()
            }
        };
        format!("{}:{identifiant}", self.type_canal.code().to_lowercase())
    }
}

/// Contexte d'une communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Contexte {
    /// Communication professionnelle.
    Pro,
    /// Communication personnelle.
    Perso,
}

/// Canaux déclarés, notes et classements d'une fiche.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CanauxDeFiche {
    /// Messageries déclarées, dans leur ordre.
    pub declares: Vec<Canal>,
    /// Note libre par canal, par clé de canal.
    pub notes: BTreeMap<String, String>,
    /// Classement pro, par clés de canal, du préféré au moins préféré.
    pub pro: Vec<String>,
    /// Classement perso, de même.
    pub perso: Vec<String>,
}

impl CanauxDeFiche {
    /// Vrai si rien n'est déclaré, noté ni classé.
    #[must_use]
    pub fn est_vide(&self) -> bool {
        self.declares.is_empty()
            && self.notes.is_empty()
            && self.pro.is_empty()
            && self.perso.is_empty()
    }
}

/// Canal disponible d'une fiche, avec sa note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disponible {
    /// Canal.
    pub canal: Canal,
    /// Note libre, s'il y en a une.
    pub note: Option<String>,
    /// Vrai pour une messagerie déclarée ; faux pour un canal qui découle
    /// de la fiche (appel, SMS, e-mail).
    pub declare: bool,
}

/// Erreur d'une opération sur les canaux.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurCanal {
    /// Aucune fiche de cet identifiant.
    #[error("fiche inconnue : {0}")]
    FicheInconnue(FicheId),
    /// L'identifiant n'est pas un numéro ou une adresse de la fiche.
    #[error("« {0} » n'est pas un numéro ou une adresse de la fiche")]
    IdentifiantAbsent(String),
    /// Identifiant ou nom de messagerie vide.
    #[error("identifiant ou nom de messagerie vide")]
    Vide,
}

impl Referentiel {
    /// Canaux disponibles d'une fiche : appel et SMS par numéro, e-mail par
    /// adresse, puis les messageries déclarées dont l'identifiant tient
    /// toujours ; chacun avec sa note.
    // [impl->req~canaux-par-identifiant~1]
    // [impl->req~note-par-canal~1]
    #[must_use]
    pub fn canaux(&self, fiche: FicheId) -> Vec<Disponible> {
        let Some(f) = self.fiches.get(&fiche) else {
            return Vec::new();
        };
        let propres = self.canaux.get(&fiche);
        let numeros = f.textes(Champ::Telephone);
        let adresses = f.textes(Champ::Email);
        let mut canaux: Vec<(Canal, bool)> = Vec::new();
        for numero in &numeros {
            for type_canal in [TypeCanal::Appel, TypeCanal::Sms] {
                canaux.push((
                    Canal {
                        type_canal,
                        identifiant: numero.clone(),
                    },
                    false,
                ));
            }
        }
        for adresse in &adresses {
            canaux.push((
                Canal {
                    type_canal: TypeCanal::Email,
                    identifiant: adresse.clone(),
                },
                false,
            ));
        }
        let tient = |canal: &Canal| {
            let present = |champ: Champ, valeurs: &[String]| {
                let k = cle(champ, &canal.identifiant);
                valeurs.iter().any(|v| cle(champ, v) == k)
            };
            match canal.type_canal.identifiant() {
                Identifiant::Numero => present(Champ::Telephone, &numeros),
                Identifiant::Adresse => present(Champ::Email, &adresses),
                Identifiant::NumeroOuPseudonyme => {
                    canal.identifiant.trim().starts_with('@') || present(Champ::Telephone, &numeros)
                }
                Identifiant::Pseudonyme => true,
            }
        };
        if let Some(propres) = propres {
            canaux.extend(
                propres
                    .declares
                    .iter()
                    .filter(|c| tient(c))
                    .map(|c| (c.clone(), true)),
            );
        }
        let mut vues = Vec::new();
        canaux
            .into_iter()
            .filter(|(canal, _)| {
                let cle = canal.cle();
                let nouvelle = !vues.contains(&cle);
                vues.push(cle);
                nouvelle
            })
            .map(|(canal, declare)| Disponible {
                note: propres.and_then(|p| p.notes.get(&canal.cle()).cloned()),
                canal,
                declare,
            })
            .collect()
    }

    /// Déclare une messagerie pour une fiche, rattachée à un de ses
    /// numéros, une de ses adresses, ou un pseudonyme selon le canal.
    ///
    /// # Errors
    ///
    /// [`ErreurCanal`] : fiche inconnue, identifiant vide, ou numéro ou
    /// adresse que la fiche ne porte pas ; rien n'est alors modifié.
    // [impl->req~canaux-par-identifiant~1]
    pub fn declarer_canal(
        &mut self,
        fiche: FicheId,
        type_canal: TypeCanal,
        identifiant: &str,
    ) -> Result<(), ErreurCanal> {
        let f = self
            .fiches
            .get(&fiche)
            .ok_or(ErreurCanal::FicheInconnue(fiche))?;
        let identifiant = identifiant.trim();
        if identifiant.is_empty() || type_canal.code().trim().is_empty() {
            return Err(ErreurCanal::Vide);
        }
        let porte = |champ: Champ| {
            let k = cle(champ, identifiant);
            f.textes(champ).iter().any(|v| cle(champ, v) == k)
        };
        let admis = match type_canal.identifiant() {
            Identifiant::Numero => porte(Champ::Telephone),
            Identifiant::Adresse => porte(Champ::Email),
            Identifiant::NumeroOuPseudonyme => {
                identifiant.starts_with('@') || porte(Champ::Telephone)
            }
            Identifiant::Pseudonyme => true,
        };
        if !admis {
            return Err(ErreurCanal::IdentifiantAbsent(identifiant.to_owned()));
        }
        let canal = Canal {
            type_canal,
            identifiant: identifiant.to_owned(),
        };
        let propres = self.canaux.entry(fiche).or_default();
        if !propres.declares.iter().any(|c| c.cle() == canal.cle()) {
            propres.declares.push(canal);
        }
        Ok(())
    }

    /// Retire une messagerie déclarée, par sa clé, avec sa note et ses
    /// places dans les classements.
    pub fn retirer_canal(&mut self, fiche: FicheId, cle_canal: &str) {
        if let Some(propres) = self.canaux.get_mut(&fiche) {
            propres.declares.retain(|c| c.cle() != cle_canal);
            propres.notes.remove(cle_canal);
            propres.pro.retain(|c| c != cle_canal);
            propres.perso.retain(|c| c != cle_canal);
        }
        self.canaux.retain(|_, propres| !propres.est_vide());
    }

    /// Attache une note libre à un canal de la fiche ; une note vide la
    /// retire.
    ///
    /// # Errors
    ///
    /// [`ErreurCanal::FicheInconnue`].
    // [impl->req~note-par-canal~1]
    pub fn annoter_canal(
        &mut self,
        fiche: FicheId,
        cle_canal: &str,
        note: &str,
    ) -> Result<(), ErreurCanal> {
        if !self.fiches.contains_key(&fiche) {
            return Err(ErreurCanal::FicheInconnue(fiche));
        }
        let propres = self.canaux.entry(fiche).or_default();
        if note.trim().is_empty() {
            propres.notes.remove(cle_canal);
        } else {
            propres
                .notes
                .insert(cle_canal.to_owned(), note.trim().to_owned());
        }
        self.canaux.retain(|_, propres| !propres.est_vide());
        Ok(())
    }

    /// Fixe le classement d'un contexte, par clés de canal, du préféré au
    /// moins préféré ; vide : le classement de l'autre contexte vaudra.
    ///
    /// # Errors
    ///
    /// [`ErreurCanal::FicheInconnue`].
    // [impl->req~classements-pro-perso~2]
    pub fn classer_canaux(
        &mut self,
        fiche: FicheId,
        contexte: Contexte,
        cles: Vec<String>,
    ) -> Result<(), ErreurCanal> {
        if !self.fiches.contains_key(&fiche) {
            return Err(ErreurCanal::FicheInconnue(fiche));
        }
        let mut uniques: Vec<String> = Vec::new();
        for c in cles {
            if !uniques.contains(&c) {
                uniques.push(c);
            }
        }
        let propres = self.canaux.entry(fiche).or_default();
        match contexte {
            Contexte::Pro => propres.pro = uniques,
            Contexte::Perso => propres.perso = uniques,
        }
        self.canaux.retain(|_, propres| !propres.est_vide());
        Ok(())
    }

    /// Classement effectif d'un contexte : le sien s'il est renseigné,
    /// sinon celui de l'autre contexte ; seuls les canaux disponibles y
    /// figurent.
    // [impl->req~classements-pro-perso~2]
    #[must_use]
    pub fn classement(&self, fiche: FicheId, contexte: Contexte) -> Vec<String> {
        let Some(propres) = self.canaux.get(&fiche) else {
            return Vec::new();
        };
        let (sien, autre) = match contexte {
            Contexte::Pro => (&propres.pro, &propres.perso),
            Contexte::Perso => (&propres.perso, &propres.pro),
        };
        let disponibles: Vec<String> = self.canaux(fiche).iter().map(|d| d.canal.cle()).collect();
        let retenus = |classement: &Vec<String>| -> Vec<String> {
            classement
                .iter()
                .filter(|c| disponibles.contains(c))
                .cloned()
                .collect()
        };
        let sien = retenus(sien);
        if sien.is_empty() {
            retenus(autre)
        } else {
            sien
        }
    }

    /// Canal préféré d'une fiche pour un contexte, s'il y en a un.
    // [impl->req~classements-pro-perso~2]
    #[must_use]
    pub fn canal_prefere(&self, fiche: FicheId, contexte: Contexte) -> Option<Canal> {
        let premier = self.classement(fiche, contexte).into_iter().next()?;
        self.canaux(fiche)
            .into_iter()
            .map(|d| d.canal)
            .find(|c| c.cle() == premier)
    }

    /// Canaux propres d'une fiche, tels qu'enregistrés.
    #[must_use]
    pub fn canaux_de(&self, fiche: FicheId) -> Option<&CanauxDeFiche> {
        self.canaux.get(&fiche)
    }

    /// Vrai si la fiche dispose d'un canal de ce type.
    // [impl->req~filtre-par-canal~1]
    #[must_use]
    pub fn dispose_de(&self, fiche: FicheId, type_canal: &TypeCanal) -> bool {
        self.canaux(fiche)
            .iter()
            .any(|d| d.canal.type_canal == *type_canal)
    }
}

#[cfg(test)]
mod tests;

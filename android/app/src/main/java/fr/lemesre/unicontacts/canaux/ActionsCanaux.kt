// Action d'un canal : l'adresse qu'ouvre son bouton, vers l'application
// concernée. Code pur, éprouvé sur la JVM.
// [impl->req~actions-triees-par-preference~2]
package fr.lemesre.unicontacts.canaux

object ActionsCanaux {
    /** Numéro international, chiffres seuls précédés de `+` (indicatif +33 par défaut). */
    fun numero(texte: String): String {
        val chiffres = texte.filter { it.isDigit() }
        return when {
            texte.trim().startsWith("+") -> "+$chiffres"
            chiffres.startsWith("00") -> "+" + chiffres.drop(2)
            chiffres.startsWith("0") -> "+33" + chiffres.drop(1)
            else -> "+$chiffres"
        }
    }

    /**
     * Encodage en pourcentage (UTF-8) de tout caractère hors des non
     * réservés et de ceux admis : un identifiant venu d'une source ne peut
     * ainsi ajouter ni paramètre (`?bcc=…`), ni fragment, ni destinataire.
     */
    fun encoder(texte: String, admis: String = ""): String = buildString {
        for (octet in texte.toByteArray(Charsets.UTF_8)) {
            val c = (octet.toInt() and 0xff).toChar()
            if (c.isLetterOrDigit() && c.code < 0x80 || c in "-._~" || c in admis) {
                append(c)
            } else {
                append('%').append("%02X".format(octet.toInt() and 0xff))
            }
        }
    }

    /** Pseudonyme Telegram valide : lettres, chiffres et soulignés, 5 à 32. */
    private val PSEUDO_TELEGRAM = Regex("[A-Za-z0-9_]{5,32}")

    /**
     * Adresse qu'ouvre le canal, ou null pour un canal sans action connue ou
     * un identifiant invalide : composeur, SMS, messagerie électronique, ou
     * lien de la messagerie. Les identifiants sont encodés, jamais recopiés
     * tels quels.
     */
    fun uri(code: String, identifiant: String): String? {
        val brut = identifiant.trim()
        return when (code) {
            "appel" -> "tel:${numero(brut)}"
            "sms" -> "smsto:${numero(brut)}"
            "email" -> "mailto:${encoder(brut, "@")}"
            "whatsapp" -> "https://wa.me/${numero(brut).drop(1)}"
            "signal" -> "https://signal.me/#p/${numero(brut)}"
            "telegram" -> if (brut.startsWith("@")) {
                brut.drop(1).takeIf { PSEUDO_TELEGRAM.matches(it) }?.let { "https://t.me/$it" }
            } else {
                "https://t.me/${numero(brut)}"
            }
            "matrix" -> "https://matrix.to/#/${encoder(brut, "@:")}"
            else -> null
        }
    }
}

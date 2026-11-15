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
     * Adresse qu'ouvre le canal, ou null pour un canal sans action connue :
     * composeur, SMS, messagerie électronique, ou lien de la messagerie.
     */
    fun uri(code: String, identifiant: String): String? {
        val brut = identifiant.trim()
        return when (code) {
            "appel" -> "tel:${numero(brut)}"
            "sms" -> "smsto:${numero(brut)}"
            "email" -> "mailto:$brut"
            "whatsapp" -> "https://wa.me/${numero(brut).drop(1)}"
            "signal" -> "https://signal.me/#p/${numero(brut)}"
            "telegram" -> if (brut.startsWith("@")) "https://t.me/${brut.drop(1)}" else "https://t.me/${numero(brut)}"
            "matrix" -> "https://matrix.to/#/$brut"
            else -> null
        }
    }
}

// Détection assistée des messageries : les messageries installées déclarent
// leurs contacts dans Android ; on rapproche leurs numéros de ceux des
// fiches pour suggérer un canal, que l'utilisateur confirme. Le
// rapprochement est du code pur, éprouvé sur la JVM.
// [impl->req~detection-assistee-canaux~1]
package fr.lemesre.unicontacts.canaux

import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.FicheFfi

/** Contact déclaré par une messagerie : son code de canal et ses numéros. */
data class Declaration(val code: String, val numeros: List<String>)

/** Canal suggéré pour une fiche, à confirmer. */
data class Suggestion(val code: String, val libelle: String, val numero: String)

object DetectionCanaux {
    /** Type MIME des lignes de contact des messageries reconnues, et leur code de canal. */
    val MESSAGERIES = mapOf(
        "vnd.android.cursor.item/vnd.com.whatsapp.profile" to "whatsapp",
        "vnd.android.cursor.item/vnd.org.thoughtcrime.securesms.contact" to "signal",
        "vnd.android.cursor.item/vnd.org.telegram.messenger.android.profile" to "telegram",
    )

    private val LIBELLES = mapOf("whatsapp" to "WhatsApp", "signal" to "Signal", "telegram" to "Telegram")

    /**
     * Suggestions pour une fiche : chaque messagerie qui a déclaré un de ses
     * numéros, et qu'elle n'a pas encore pour ce numéro.
     */
    fun suggestions(fiche: FicheFfi, declarations: List<Declaration>): List<Suggestion> {
        val numeros = fiche.champs
            .filter { it.champ == ChampFfi.TELEPHONE }
            .flatMap { it.valeurs }
            .map { it.texte }
        val deja = fiche.canaux.map { it.cle }.toSet()
        return declarations.flatMap { declaration ->
            numeros.filter { numero ->
                declaration.numeros.any { ActionsCanaux.numero(it) == ActionsCanaux.numero(numero) }
            }.map { numero ->
                Suggestion(declaration.code, LIBELLES[declaration.code] ?: declaration.code, numero)
            }
        }.filter { "${it.code}:${ActionsCanaux.numero(it.numero)}" !in deja }.distinct()
    }

    /** Numéro porté par l'identifiant WhatsApp (`33612345678@s.whatsapp.net`), s'il y en a un. */
    fun numeroWhatsApp(identifiant: String): String? =
        identifiant.substringBefore('@').filter { it.isDigit() }.takeIf { it.length >= 6 }?.let { "+$it" }
}

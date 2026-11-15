// Lecture des contacts que les messageries installées ont déclarés dans
// Android : pour chacun, le code de la messagerie et ses numéros.
// [impl->req~detection-assistee-canaux~1]
package fr.lemesre.unicontacts.canaux

import android.content.ContentResolver
import android.provider.ContactsContract

class LecteurMessageries(private val resolveur: ContentResolver) {
    /** Contacts déclarés par les messageries reconnues ; vide sans accès aux contacts. */
    fun declarations(): List<Declaration> {
        val parContact = mutableMapOf<Long, MutableSet<String>>()
        val numeros = mutableMapOf<Long, MutableList<String>>()
        val types = DetectionCanaux.MESSAGERIES.keys.toList()
        val marques = types.joinToString(",") { "?" }
        try {
            resolveur.query(
                ContactsContract.Data.CONTENT_URI,
                arrayOf(ContactsContract.Data.RAW_CONTACT_ID, ContactsContract.Data.MIMETYPE, ContactsContract.Data.DATA1),
                "${ContactsContract.Data.MIMETYPE} IN ($marques)",
                types.toTypedArray(),
                null,
            )?.use { curseur ->
                while (curseur.moveToNext()) {
                    val brut = curseur.getLong(0)
                    val code = DetectionCanaux.MESSAGERIES[curseur.getString(1)] ?: continue
                    parContact.getOrPut(brut) { mutableSetOf() } += code
                    val donnee = curseur.getString(2).orEmpty()
                    val numero = if (code == "whatsapp") DetectionCanaux.numeroWhatsApp(donnee) else donnee.takeIf { it.startsWith("+") }
                    numero?.let { numeros.getOrPut(brut) { mutableListOf() } += it }
                }
            }
            if (parContact.isNotEmpty()) {
                val ids = parContact.keys.toList()
                resolveur.query(
                    ContactsContract.Data.CONTENT_URI,
                    arrayOf(ContactsContract.Data.RAW_CONTACT_ID, ContactsContract.CommonDataKinds.Phone.NUMBER),
                    "${ContactsContract.Data.MIMETYPE} = ? AND ${ContactsContract.Data.RAW_CONTACT_ID} IN (${ids.joinToString(",") { "?" }})",
                    arrayOf(ContactsContract.CommonDataKinds.Phone.CONTENT_ITEM_TYPE) + ids.map { it.toString() },
                    null,
                )?.use { curseur ->
                    while (curseur.moveToNext()) {
                        numeros.getOrPut(curseur.getLong(0)) { mutableListOf() } += curseur.getString(1).orEmpty()
                    }
                }
            }
        } catch (_: SecurityException) {
            // Accès aux contacts non accordé : aucune suggestion.
            return emptyList()
        }
        return parContact.flatMap { (brut, codes) ->
            codes.map { Declaration(it, numeros[brut].orEmpty()) }
        }
    }
}

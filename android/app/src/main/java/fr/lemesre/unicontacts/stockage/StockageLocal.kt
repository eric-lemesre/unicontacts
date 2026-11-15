// Lecture et retrait des contacts du stockage local Android : contacts
// bruts sans compte, ou du compte local que déclare le système (ADR-0015).
// Le retrait ne vise que les contacts bruts encore locaux.
// [impl->req~import-stockage-local~1]
package fr.lemesre.unicontacts.stockage

import android.content.ContentResolver
import android.content.Context
import android.os.Build
import android.provider.ContactsContract.Data
import android.provider.ContactsContract.RawContacts
import fr.lemesre.unicontacts.coeur.ContactLuFfi
import fr.lemesre.unicontacts.synchro.Ligne

class StockageLocal(private val context: Context) {
    private val resolveur: ContentResolver = context.contentResolver

    /** Contacts du stockage local, non supprimés. */
    fun lire(): List<ContactLuFfi> {
        val (filtre, arguments) = filtreLocal()
        val noms = linkedMapOf<Long, String?>()
        resolveur.query(
            RawContacts.CONTENT_URI,
            arrayOf(RawContacts._ID, RawContacts.DISPLAY_NAME_PRIMARY),
            "$filtre AND ${RawContacts.DELETED} = 0",
            arguments,
            RawContacts._ID,
        )?.use { curseur ->
            while (curseur.moveToNext()) noms[curseur.getLong(0)] = curseur.getString(1)
        }
        val lignes = lignes(noms.keys)
        return noms.map { (id, nom) -> ContactsLocaux.contact(id, nom, lignes[id].orEmpty()) }
    }

    /**
     * Retire du stockage local les contacts bruts désignés par leurs
     * identifiants d'import ; un contact qui n'est plus local est épargné.
     * Rend le nombre de contacts retirés.
     */
    fun retirer(uids: Collection<String>): Int {
        val (filtre, arguments) = filtreLocal()
        return uids.mapNotNull(ContactsLocaux::idBrut).chunked(LOT).sumOf { ids ->
            resolveur.delete(
                RawContacts.CONTENT_URI,
                "${RawContacts._ID} IN (${ids.joinToString(",") { "?" }}) AND $filtre",
                (ids.map(Long::toString) + arguments.orEmpty()).toTypedArray(),
            )
        }
    }

    /** Lignes de données des contacts bruts donnés, par contact. */
    private fun lignes(ids: Collection<Long>): Map<Long, List<Ligne>> {
        val colonnes = (1..10).map { "data$it" }
        val parContact = mutableMapOf<Long, MutableList<Ligne>>()
        ids.chunked(LOT).forEach { lot ->
            resolveur.query(
                Data.CONTENT_URI,
                arrayOf(Data.RAW_CONTACT_ID, Data.MIMETYPE) + colonnes,
                "${Data.RAW_CONTACT_ID} IN (${lot.joinToString(",") { "?" }})",
                lot.map(Long::toString).toTypedArray(),
                "${Data.RAW_CONTACT_ID}, ${Data._ID}",
            )?.use { curseur ->
                while (curseur.moveToNext()) {
                    val valeurs = colonnes.mapIndexedNotNull { i, nom ->
                        curseur.getString(i + 2)?.let { nom to it }
                    }.toMap()
                    parContact.getOrPut(curseur.getLong(0)) { mutableListOf() } += Ligne(curseur.getString(1), valeurs)
                }
            }
        }
        return parContact
    }

    /** Filtre des contacts bruts locaux : sans compte, ou du compte local du système. */
    private fun filtreLocal(): Pair<String, Array<String>?> {
        val type = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            RawContacts.getLocalAccountType(context)
        } else {
            null
        }
        return if (type == null) {
            "${RawContacts.ACCOUNT_TYPE} IS NULL" to null
        } else {
            "(${RawContacts.ACCOUNT_TYPE} IS NULL OR ${RawContacts.ACCOUNT_TYPE} = ?)" to arrayOf(type)
        }
    }

    private companion object {
        /** Taille des lots d'identifiants, sous la limite des paramètres SQLite. */
        const val LOT = 500
    }
}

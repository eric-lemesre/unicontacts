// Écriture du référentiel dans les contacts Android : un contact brut par
// fiche, sous le seul compte d'unicontacts, en lecture seule pour le reste
// du téléphone. Un contact inchangé n'est pas réécrit (empreinte en SYNC1) ;
// un contact dont la fiche a disparu est retiré.
// [impl->req~ecriture-sous-compte-unique~1]
package fr.lemesre.unicontacts.synchro

import android.accounts.Account
import android.content.ContentProviderClient
import android.content.ContentProviderOperation
import android.net.Uri
import android.provider.ContactsContract
import android.provider.ContactsContract.Data
import android.provider.ContactsContract.RawContacts
import fr.lemesre.unicontacts.coeur.FicheFfi

class EcrivainContacts(private val client: ContentProviderClient, private val compte: Account) {
    private fun commeSynchroniseur(uri: Uri): Uri = uri.buildUpon()
        .appendQueryParameter(ContactsContract.CALLER_IS_SYNCADAPTER, "true")
        .appendQueryParameter(RawContacts.ACCOUNT_NAME, compte.name)
        .appendQueryParameter(RawContacts.ACCOUNT_TYPE, compte.type)
        .build()

    private val contactsBruts = commeSynchroniseur(RawContacts.CONTENT_URI)
    private val donnees = commeSynchroniseur(Data.CONTENT_URI)

    /** Contact brut existant : identifiant et empreinte. */
    private data class Existant(val id: Long, val empreinte: String?)

    private fun existants(): Map<String, Existant> {
        val existants = mutableMapOf<String, Existant>()
        client.query(
            contactsBruts,
            arrayOf(RawContacts._ID, RawContacts.SOURCE_ID, RawContacts.SYNC1),
            "${RawContacts.ACCOUNT_TYPE} = ? AND ${RawContacts.ACCOUNT_NAME} = ?",
            arrayOf(compte.type, compte.name),
            null,
        )?.use { curseur ->
            while (curseur.moveToNext()) {
                val source = curseur.getString(1) ?: continue
                existants[source] = Existant(curseur.getLong(0), curseur.getString(2))
            }
        }
        return existants
    }

    /** Publie les fiches ; rend le nombre de contacts écrits et retirés. */
    fun publier(fiches: List<FicheFfi>): Pair<Int, Int> {
        val existants = existants()
        var ecrits = 0
        val operations = ArrayList<ContentProviderOperation>()
        for (fiche in fiches) {
            val lignes = LignesContacts.lignes(fiche)
            val empreinte = LignesContacts.empreinte(lignes)
            val existant = existants[fiche.id]
            if (existant?.empreinte == empreinte) continue
            ecrits++
            if (existant == null) {
                val debut = operations.size
                operations += ContentProviderOperation.newInsert(contactsBruts)
                    .withValue(RawContacts.ACCOUNT_NAME, compte.name)
                    .withValue(RawContacts.ACCOUNT_TYPE, compte.type)
                    .withValue(RawContacts.SOURCE_ID, fiche.id)
                    .withValue(RawContacts.SYNC1, empreinte)
                    .withValue(RawContacts.RAW_CONTACT_IS_READ_ONLY, 1)
                    .build()
                lignes.forEach { operations += insertion(it).withValueBackReference(Data.RAW_CONTACT_ID, debut).build() }
            } else {
                operations += ContentProviderOperation.newDelete(donnees)
                    .withSelection("${Data.RAW_CONTACT_ID} = ?", arrayOf(existant.id.toString()))
                    .build()
                lignes.forEach { operations += insertion(it).withValue(Data.RAW_CONTACT_ID, existant.id).build() }
                operations += ContentProviderOperation.newUpdate(contactsBruts)
                    .withSelection("${RawContacts._ID} = ?", arrayOf(existant.id.toString()))
                    .withValue(RawContacts.SYNC1, empreinte)
                    .build()
            }
            if (operations.size >= LOT) appliquer(operations)
        }
        val gardes = fiches.map { it.id }.toSet()
        val retires = existants.filterKeys { it !in gardes }.values
        retires.forEach {
            operations += ContentProviderOperation.newDelete(contactsBruts)
                .withSelection("${RawContacts._ID} = ?", arrayOf(it.id.toString()))
                .build()
        }
        appliquer(operations)
        return ecrits to retires.size
    }

    private fun insertion(ligne: Ligne): ContentProviderOperation.Builder {
        val operation = ContentProviderOperation.newInsert(donnees).withValue(Data.MIMETYPE, ligne.mimetype)
        ligne.colonnes.forEach { (colonne, valeur) -> operation.withValue(colonne, valeur) }
        return operation
    }

    private fun appliquer(operations: ArrayList<ContentProviderOperation>) {
        if (operations.isEmpty()) return
        client.applyBatch(ArrayList(operations))
        operations.clear()
    }

    private companion object {
        /** Opérations par lot : un lot reste sous la limite de transaction Binder. */
        const val LOT = 300
    }
}

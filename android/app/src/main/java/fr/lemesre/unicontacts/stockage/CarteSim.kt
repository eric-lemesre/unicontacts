// Lecture de la carte SIM par SimPhonebookContract (ADR-0016) : nom et
// numéro de chaque enregistrement ADN. Rien n'y est jamais écrit.
// [impl->req~source-sim-lecture-seule~1]
package fr.lemesre.unicontacts.stockage

import android.content.Context
import android.provider.SimPhonebookContract.ElementaryFiles
import android.provider.SimPhonebookContract.SimRecords
import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.ContactLuFfi
import fr.lemesre.unicontacts.coeur.ValeurLueFfi

/** Traduction d'un enregistrement SIM, sans classe Android, éprouvée sur la JVM. */
object EntreesSim {
    /** Contact à importer : nom et numéro seulement ; null pour un enregistrement vide. */
    fun contact(abonnement: Int, enregistrement: Int, nom: String?, numero: String?): ContactLuFfi? {
        val valeurs = listOfNotNull(
            nom?.trim()?.takeIf { it.isNotEmpty() }?.let { ValeurLueFfi(ChampFfi.NOM_AFFICHE, it, emptyList()) },
            numero?.trim()?.takeIf { it.isNotEmpty() }?.let { ValeurLueFfi(ChampFfi.TELEPHONE, it, emptyList()) },
        )
        if (valeurs.isEmpty()) return null
        return ContactLuFfi("sim-$abonnement-$enregistrement", valeurs, emptyList())
    }
}

class CarteSim(context: Context) {
    private val resolveur = context.contentResolver

    /**
     * Entrées de toutes les SIM présentes ; null si aucun annuaire ADN n'est
     * lisible (SIM retirée ou verrouillée) : une lecture vide ne vaut pas
     * disparition des entrées.
     */
    fun lire(): List<ContactLuFfi>? {
        val abonnements = mutableListOf<Int>()
        resolveur.query(
            ElementaryFiles.CONTENT_URI,
            arrayOf(ElementaryFiles.SUBSCRIPTION_ID, ElementaryFiles.EF_TYPE),
            null,
            null,
            null,
        )?.use { curseur ->
            while (curseur.moveToNext()) {
                if (curseur.getInt(1) == ElementaryFiles.EF_ADN) abonnements += curseur.getInt(0)
            }
        }
        if (abonnements.isEmpty()) return null
        return abonnements.flatMap { abonnement ->
            val entrees = mutableListOf<ContactLuFfi>()
            resolveur.query(
                SimRecords.getContentUri(abonnement, ElementaryFiles.EF_ADN),
                arrayOf(SimRecords.RECORD_NUMBER, SimRecords.NAME, SimRecords.PHONE_NUMBER),
                null,
                null,
                null,
            )?.use { curseur ->
                while (curseur.moveToNext()) {
                    EntreesSim.contact(abonnement, curseur.getInt(0), curseur.getString(1), curseur.getString(2))
                        ?.let { entrees += it }
                }
            }
            entrees
        }
    }
}

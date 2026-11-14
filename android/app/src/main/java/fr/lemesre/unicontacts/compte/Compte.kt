// Compte système propre à l'application (ADR-0004) : les contacts du
// référentiel s'écrivent sous lui seul.
// [impl->req~compte-systeme-dedie~1]
package fr.lemesre.unicontacts.compte

import android.accounts.Account
import android.accounts.AccountManager
import android.content.ContentResolver
import android.content.Context
import android.os.Bundle
import android.provider.ContactsContract

object Compte {
    /** Type de compte ; provisoire tant que OQ-007 (nom) n'est pas tranchée. */
    const val TYPE = "fr.lemesre.unicontacts"
    const val NOM = "unicontacts"

    /** Synchronisation périodique, en secondes. */
    private const val PERIODE = 60L * 60L

    val compte: Account get() = Account(NOM, TYPE)

    /**
     * Crée le compte s'il n'existe pas, et le rend synchronisable pour les
     * contacts, automatiquement et périodiquement.
     */
    fun assurer(context: Context): Account {
        val gestionnaire = AccountManager.get(context)
        val compte = compte
        if (gestionnaire.getAccountsByType(TYPE).none { it.name == NOM }) {
            gestionnaire.addAccountExplicitly(compte, null, null)
        }
        val autorite = ContactsContract.AUTHORITY
        ContentResolver.setIsSyncable(compte, autorite, 1)
        ContentResolver.setSyncAutomatically(compte, autorite, true)
        ContentResolver.addPeriodicSync(compte, autorite, Bundle.EMPTY, PERIODE)
        return compte
    }

    /** Demande une synchronisation immédiate. */
    fun synchroniser() {
        val extras = Bundle().apply {
            putBoolean(ContentResolver.SYNC_EXTRAS_MANUAL, true)
            putBoolean(ContentResolver.SYNC_EXTRAS_EXPEDITED, true)
        }
        ContentResolver.requestSync(compte, ContactsContract.AUTHORITY, extras)
    }
}

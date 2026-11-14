// Adaptateur de synchronisation du compte d'unicontacts : chaque source est
// synchronisée directement avec son serveur, puis le référentiel est publié
// dans les contacts Android sous le seul compte de l'application.
// [impl->req~synchronisation-directe~1]
package fr.lemesre.unicontacts.synchro

import android.accounts.Account
import android.app.Service
import android.content.AbstractThreadedSyncAdapter
import android.content.ContentProviderClient
import android.content.Context
import android.content.Intent
import android.content.OperationApplicationException
import android.content.SyncResult
import android.os.Bundle
import android.os.IBinder
import android.os.RemoteException
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ErreurFfi
import java.security.GeneralSecurityException

class AdaptateurSynchro(context: Context) : AbstractThreadedSyncAdapter(context, true, false) {
    override fun onPerformSync(
        account: Account,
        extras: Bundle,
        authority: String,
        provider: ContentProviderClient,
        syncResult: SyncResult,
    ) {
        val app = context.applicationContext as UnicontactsApp
        val coeur = app.coeur
        for (source in app.configurations.toutes(coeur.sources())) {
            try {
                val bilan = coeur.synchroniserCarddav(source.id, source.carnet, source.utilisateur, source.motDePasse())
                syncResult.stats.numUpdates += bilan.ecrites.toLong()
                syncResult.stats.numDeletes += bilan.supprimees.toLong()
                syncResult.stats.numSkippedEntries += bilan.refusees.toLong()
            } catch (erreur: ErreurFfi) {
                // L'échec d'une source n'arrête pas les autres ; le message
                // ne porte ni identifiant ni mot de passe.
                syncResult.stats.numIoExceptions++
                app.signaler(source.nom, erreur.message.orEmpty())
            } catch (erreur: GeneralSecurityException) {
                // Clé du coffre perdue (réinstallation, ADR-0010) : le mot de
                // passe est à ressaisir.
                syncResult.stats.numAuthExceptions++
                app.signaler(source.nom, "mot de passe à ressaisir (${erreur.javaClass.simpleName})")
            }
        }
        try {
            val (ecrits, retires) = EcrivainContacts(provider, account).publier(coeur.fichesCompletes())
            syncResult.stats.numInserts += ecrits.toLong()
            syncResult.stats.numDeletes += retires.toLong()
        } catch (erreur: SecurityException) {
            // Accès aux contacts non accordé : la publication attend la
            // permission, le référentiel reste synchronisé.
            syncResult.databaseError = true
            app.signaler("Contacts du téléphone", "accès non accordé (${erreur.javaClass.simpleName})")
        } catch (erreur: OperationApplicationException) {
            syncResult.databaseError = true
            app.signaler("Contacts du téléphone", erreur.message.orEmpty())
        } catch (erreur: RemoteException) {
            syncResult.databaseError = true
            app.signaler("Contacts du téléphone", erreur.message.orEmpty())
        }
    }
}

class ServiceSynchro : Service() {
    override fun onBind(intent: Intent?): IBinder? = adaptateur(this).syncAdapterBinder

    private companion object {
        private val verrou = Any()
        private var instance: AdaptateurSynchro? = null

        fun adaptateur(context: Context): AdaptateurSynchro = synchronized(verrou) {
            instance ?: AdaptateurSynchro(context.applicationContext).also { instance = it }
        }
    }
}

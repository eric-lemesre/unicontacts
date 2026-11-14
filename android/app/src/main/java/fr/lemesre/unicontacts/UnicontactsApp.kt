// Application : un seul cœur, ouvert sur sa base, partagé par les écrans et
// l'adaptateur de synchronisation.
// [impl->req~compte-systeme-dedie~1]
package fr.lemesre.unicontacts

import android.app.Application
import fr.lemesre.unicontacts.coeur.Coeur
import fr.lemesre.unicontacts.compte.Compte
import fr.lemesre.unicontacts.sources.ConfigurationsSources
import java.io.File

class UnicontactsApp : Application() {
    val coeur: Coeur by lazy { Coeur.ouvrir(File(filesDir, "referentiel.sqlite").path) }
    val configurations: ConfigurationsSources by lazy { ConfigurationsSources(this) }

    /** Dernier échec de synchronisation par source, pour l'écran des sources. */
    val echecs: MutableMap<String, String> = java.util.concurrent.ConcurrentHashMap()

    fun signaler(source: String, message: String) {
        echecs[source] = message
    }

    override fun onCreate() {
        super.onCreate()
        Compte.assurer(this)
    }
}

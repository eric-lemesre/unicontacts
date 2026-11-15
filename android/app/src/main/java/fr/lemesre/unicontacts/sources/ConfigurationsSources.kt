// Paramètres de connexion des sources CardDAV, gardés côté Android
// (ADR-0010) : URL du carnet et identifiant en clair, mot de passe chiffré
// par le coffre. Le cœur ne connaît que l'identifiant de la source.
// [impl->req~synchronisation-directe~1]
package fr.lemesre.unicontacts.sources

import android.content.Context
import androidx.core.content.edit
import fr.lemesre.unicontacts.coeur.SourceFfi

/** Source CardDAV prête à synchroniser. */
class SourceCardDav(
    val id: String,
    val nom: String,
    val carnet: String,
    val utilisateur: String,
    private val chiffre: Chiffre,
) {
    /** Mot de passe déchiffré, le temps d'un appel. */
    fun motDePasse(): String = Coffre.dechiffrer(chiffre)
}

class ConfigurationsSources(context: Context) {
    private val preferences = context.getSharedPreferences("sources", Context.MODE_PRIVATE)

    fun enregistrer(id: String, carnet: String, utilisateur: String, motDePasse: String) {
        val chiffre = Coffre.chiffrer(motDePasse)
        preferences.edit {
            putString("$id.carnet", carnet)
            putString("$id.utilisateur", utilisateur)
            putString("$id.iv", chiffre.iv)
            putString("$id.mdp", chiffre.donnees)
        }
    }

    /** Sources configurées dont les paramètres de connexion sont connus. */
    fun toutes(sources: List<SourceFfi>): List<SourceCardDav> = sources.mapNotNull { source ->
        val carnet = preferences.getString("${source.id}.carnet", null) ?: return@mapNotNull null
        val utilisateur = preferences.getString("${source.id}.utilisateur", null) ?: return@mapNotNull null
        val iv = preferences.getString("${source.id}.iv", null) ?: return@mapNotNull null
        val mdp = preferences.getString("${source.id}.mdp", null) ?: return@mapNotNull null
        SourceCardDav(source.id, source.nom, carnet, utilisateur, Chiffre(iv, mdp))
    }

    fun carnet(id: String): String? = preferences.getString("$id.carnet", null)

    /** Source d'import du stockage local, une fois créée (ADR-0015). */
    var sourceStockageLocal: String?
        get() = preferences.getString("stockage-local.source", null)
        set(id) = preferences.edit { putString("stockage-local.source", id) }

    companion object {
        /**
         * Motif du refus d'une URL de carnet, ou null si elle convient :
         * https seulement (http pour un serveur local de test), sans
         * identifiants dans l'URL.
         */
        fun refusCarnet(url: String): String? {
            val adresse = url.trim()
            val reste = when {
                adresse.startsWith("https://") -> adresse.removePrefix("https://")
                adresse.startsWith("http://localhost") || adresse.startsWith("http://127.0.0.1") ->
                    adresse.removePrefix("http://")
                else -> return "L'adresse doit commencer par https://"
            }
            val hote = reste.substringBefore('/')
            return when {
                hote.isEmpty() -> "L'adresse doit nommer un serveur"
                '@' in hote -> "L'identifiant se saisit à part, pas dans l'adresse"
                else -> null
            }
        }
    }
}

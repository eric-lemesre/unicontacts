// Paramètres de connexion des sources CardDAV et Google, gardés côté
// Android (ADR-0010, ADR-0017) : URL du carnet, identifiant, adresse et
// identifiant client en clair ; mot de passe et jeton de renouvellement
// chiffrés par le coffre. Le cœur ne connaît que l'identifiant de la source.
// [impl->req~synchronisation-directe~1]
// [impl->req~source-google~1]
package fr.lemesre.unicontacts.sources

import android.content.Context
import androidx.core.content.edit
import fr.lemesre.unicontacts.coeur.SourceFfi

/** Source Google prête à synchroniser (ADR-0017). */
class SourceGoogle(
    val id: String,
    val nom: String,
    val adresse: String,
    val client: String,
    private val chiffre: Chiffre,
) {
    /** Jeton de renouvellement déchiffré, le temps d'un appel. */
    fun renouvellement(): String = Coffre.dechiffrer(chiffre)
}

/** Autorisation Google en attente du retour du navigateur. */
class AttenteGoogle(val adresse: String, val client: String, private val chiffre: Chiffre) {
    /** Vérificateur PKCE déchiffré. */
    fun verificateur(): String = Coffre.dechiffrer(chiffre)
}

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

    fun carnet(id: String): String? =
        preferences.getString("$id.carnet", null) ?: preferences.getString("$id.adresse", null)?.let { "Google : $it" }

    /**
     * Enregistre une source Google : adresse et identifiant client en clair,
     * jeton de renouvellement chiffré par le coffre (ADR-0010, ADR-0017).
     */
    fun enregistrerGoogle(id: String, adresse: String, client: String, renouvellement: String) {
        val chiffre = Coffre.chiffrer(renouvellement)
        preferences.edit {
            putString("$id.adresse", adresse)
            putString("$id.client", client)
            putString("$id.iv", chiffre.iv)
            putString("$id.jeton", chiffre.donnees)
        }
    }

    /** Sources Google configurées dont l'autorisation est connue. */
    fun toutesGoogle(sources: List<SourceFfi>): List<SourceGoogle> = sources.mapNotNull { source ->
        val adresse = preferences.getString("${source.id}.adresse", null) ?: return@mapNotNull null
        val client = preferences.getString("${source.id}.client", null) ?: return@mapNotNull null
        val iv = preferences.getString("${source.id}.iv", null) ?: return@mapNotNull null
        val jeton = preferences.getString("${source.id}.jeton", null) ?: return@mapNotNull null
        SourceGoogle(source.id, source.nom, adresse, client, Chiffre(iv, jeton))
    }

    /** Source Google déjà configurée pour une adresse, s'il en est une. */
    fun sourceGoogle(sources: List<SourceFfi>, adresse: String): String? =
        sources.firstOrNull { preferences.getString("${it.id}.adresse", null).equals(adresse, ignoreCase = true) }?.id

    /** Dernier identifiant client OAuth saisi, pour préremplir le formulaire. */
    var clientGoogle: String?
        get() = preferences.getString("google.client", null)
        set(valeur) = preferences.edit { putString("google.client", valeur) }

    /** Garde une autorisation en attente, le vérificateur chiffré. */
    fun attendreGoogle(etat: String, adresse: String, client: String, verificateur: String) {
        val chiffre = Coffre.chiffrer(verificateur)
        preferences.edit {
            putString("google.attente.etat", etat)
            putString("google.attente.adresse", adresse)
            putString("google.attente.client", client)
            putString("google.attente.iv", chiffre.iv)
            putString("google.attente.verificateur", chiffre.donnees)
        }
    }

    /**
     * Autorisation en attente dont l'état est celui reçu, retirée des
     * préférences ; null si l'état ne correspond pas (retour forgé ou
     * périmé).
     */
    fun reprendreGoogle(etat: String?): AttenteGoogle? {
        val attendu = preferences.getString("google.attente.etat", null)
        val attente = if (attendu != null && attendu == etat) {
            val adresse = preferences.getString("google.attente.adresse", null)
            val client = preferences.getString("google.attente.client", null)
            val iv = preferences.getString("google.attente.iv", null)
            val verificateur = preferences.getString("google.attente.verificateur", null)
            if (adresse != null && client != null && iv != null && verificateur != null) {
                AttenteGoogle(adresse, client, Chiffre(iv, verificateur))
            } else {
                null
            }
        } else {
            null
        }
        preferences.edit {
            listOf("etat", "adresse", "client", "iv", "verificateur").forEach { remove("google.attente.$it") }
        }
        return attente
    }

    /** Source d'import du stockage local, une fois créée (ADR-0015). */
    var sourceStockageLocal: String?
        get() = preferences.getString("stockage-local.source", null)
        set(id) = preferences.edit { putString("stockage-local.source", id) }

    /** Source de la carte SIM, une fois ajoutée (ADR-0016). */
    var sourceSim: String?
        get() = preferences.getString("sim.source", null)
        set(id) = preferences.edit { putString("sim.source", id) }

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

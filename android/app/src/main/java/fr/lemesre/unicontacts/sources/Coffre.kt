// Coffre des mots de passe des sources (ADR-0010) : chiffrés en AES-GCM par
// une clé du Keystore Android, propre à l'application et non exportable.
// [impl->req~synchronisation-directe~1]
package fr.lemesre.unicontacts.sources

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyStore
import java.util.Base64
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Texte chiffré et son vecteur d'initialisation, en base64. */
data class Chiffre(val iv: String, val donnees: String)

object Coffre {
    private const val MAGASIN = "AndroidKeyStore"
    private const val ALIAS = "unicontacts-sources"
    private const val TRANSFORMATION = "AES/GCM/NoPadding"
    private const val ETIQUETTE_BITS = 128

    private fun cle(): SecretKey {
        val magasin = KeyStore.getInstance(MAGASIN).apply { load(null) }
        (magasin.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        val generateur = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, MAGASIN)
        generateur.init(
            KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return generateur.generateKey()
    }

    fun chiffrer(clair: String): Chiffre {
        val chiffreur = Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.ENCRYPT_MODE, cle()) }
        val donnees = chiffreur.doFinal(clair.toByteArray(Charsets.UTF_8))
        val encodeur = Base64.getEncoder()
        return Chiffre(encodeur.encodeToString(chiffreur.iv), encodeur.encodeToString(donnees))
    }

    fun dechiffrer(chiffre: Chiffre): String {
        val decodeur = Base64.getDecoder()
        val dechiffreur = Cipher.getInstance(TRANSFORMATION).apply {
            init(Cipher.DECRYPT_MODE, cle(), GCMParameterSpec(ETIQUETTE_BITS, decodeur.decode(chiffre.iv)))
        }
        return String(dechiffreur.doFinal(decodeur.decode(chiffre.donnees)), Charsets.UTF_8)
    }
}

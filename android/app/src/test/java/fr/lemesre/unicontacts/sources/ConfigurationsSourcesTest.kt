// Contrôle de l'adresse d'un carnet saisie par l'utilisateur.
// [impl->req~synchronisation-directe~1]
package fr.lemesre.unicontacts.sources

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ConfigurationsSourcesTest {
    @Test
    fun uneAdresseHttpsEstAdmise() {
        assertNull(ConfigurationsSources.refusCarnet("https://murena.io/remote.php/dav/addressbooks/users/c/contacts/"))
    }

    @Test
    fun uneAdresseHttpVersUnServeurDistantEstRefusee() {
        assertEquals("L'adresse doit commencer par https://", ConfigurationsSources.refusCarnet("http://exemple.org/carnet/"))
    }

    @Test
    fun unServeurLocalDeTestPeutEtreEnHttp() {
        assertNull(ConfigurationsSources.refusCarnet("http://127.0.0.1:5232/essai/carnet/"))
    }

    @Test
    fun desIdentifiantsDansLAdresseSontRefuses() {
        assertEquals(
            "L'identifiant se saisit à part, pas dans l'adresse",
            ConfigurationsSources.refusCarnet("https://camille:secret@exemple.org/carnet/"),
        )
    }
}

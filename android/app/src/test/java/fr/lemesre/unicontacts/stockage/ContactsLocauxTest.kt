// Traduction des contacts du stockage local en contacts à importer,
// éprouvée sur la JVM.
// [utest->req~import-stockage-local~1]
package fr.lemesre.unicontacts.stockage

import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.DateLueFfi
import fr.lemesre.unicontacts.coeur.ValeurLueFfi
import fr.lemesre.unicontacts.synchro.Ligne
import fr.lemesre.unicontacts.synchro.Mime
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ContactsLocauxTest {
    private fun ligne(mime: String, vararg colonnes: Pair<String, String>) = Ligne(mime, colonnes.toMap())

    @Test
    fun `un contact local donne ses noms, numeros types et adresses`() {
        val contact = ContactsLocaux.contact(
            42,
            "Alice Martin",
            listOf(
                ligne(Mime.NOM, "data1" to "Alice Martin", "data2" to "Alice", "data3" to "Martin"),
                ligne(Mime.TELEPHONE, "data1" to "06 12 34 56 78", "data2" to "2"),
                ligne(Mime.EMAIL, "data1" to "alice@exemple.fr", "data2" to "2"),
                ligne(Mime.ADRESSE, "data4" to "3 rue des Lilas", "data7" to "Lyon", "data9" to "69003", "data2" to "1"),
            ),
        )
        assertEquals(
            listOf(
                ValeurLueFfi(ChampFfi.NOM_AFFICHE, "Alice Martin", emptyList()),
                ValeurLueFfi(ChampFfi.NOM_STRUCTURE, "Martin;Alice", emptyList()),
                ValeurLueFfi(ChampFfi.TELEPHONE, "06 12 34 56 78", listOf("cell")),
                ValeurLueFfi(ChampFfi.EMAIL, "alice@exemple.fr", listOf("work")),
                ValeurLueFfi(ChampFfi.ADRESSE, ";;3 rue des Lilas;Lyon;;69003", listOf("home")),
            ),
            contact.valeurs,
        )
    }

    @Test
    fun `l'identifiant d'import designe le contact brut`() {
        assertEquals(42L, ContactsLocaux.idBrut(ContactsLocaux.uid(42)))
    }

    @Test
    fun `un identifiant etranger ne designe aucun contact brut`() {
        assertNull(ContactsLocaux.idBrut("carddav-42"))
    }

    @Test
    fun `sans ligne de nom, le nom affiche du contact brut s'impose`() {
        val contact = ContactsLocaux.contact(7, "Plombier", listOf(ligne(Mime.TELEPHONE, "data1" to "0102030405")))
        assertEquals(ValeurLueFfi(ChampFfi.NOM_AFFICHE, "Plombier", emptyList()), contact.valeurs.last())
    }

    @Test
    fun `les evenements donnent naissance, mariage et dates libellees`() {
        val contact = ContactsLocaux.contact(
            7,
            null,
            listOf(
                ligne(Mime.EVENEMENT, "data1" to "1980-05-17", "data2" to "3"),
                ligne(Mime.EVENEMENT, "data1" to "--06-21", "data2" to "1"),
                ligne(Mime.EVENEMENT, "data1" to "2015-09-01", "data2" to "0", "data3" to "Rencontre"),
            ),
        )
        assertEquals(
            listOf(
                DateLueFfi("naissance", "1980-05-17"),
                DateLueFfi("mariage", "--06-21"),
                DateLueFfi("Rencontre", "2015-09-01"),
            ),
            contact.dates,
        )
    }

    @Test
    fun `l'organisation garde son service et la fonction va a part`() {
        val contact = ContactsLocaux.contact(
            7,
            null,
            listOf(ligne(Mime.ORGANISATION, "data1" to "Acme; SA", "data5" to "Achats", "data4" to "Acheteuse")),
        )
        assertEquals(
            listOf(
                ValeurLueFfi(ChampFfi.ORGANISATION, "Acme\\; SA;Achats", emptyList()),
                ValeurLueFfi(ChampFfi.FONCTION, "Acheteuse", emptyList()),
            ),
            contact.valeurs,
        )
    }

    @Test
    fun `les relations du stockage local ne se lisent pas`() {
        val contact = ContactsLocaux.contact(7, "Alice", listOf(ligne(Mime.RELATION, "data1" to "Bruno", "data2" to "14")))
        assertEquals(listOf(ValeurLueFfi(ChampFfi.NOM_AFFICHE, "Alice", emptyList())), contact.valeurs)
    }
}

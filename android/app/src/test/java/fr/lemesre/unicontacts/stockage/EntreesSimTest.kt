// Traduction des enregistrements SIM, éprouvée sur la JVM.
// [utest->req~source-sim-lecture-seule~1]
package fr.lemesre.unicontacts.stockage

import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.ContactLuFfi
import fr.lemesre.unicontacts.coeur.ValeurLueFfi
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class EntreesSimTest {
    @Test
    fun `un enregistrement SIM donne son nom et son numero`() {
        assertEquals(
            ContactLuFfi(
                "sim-1-3",
                listOf(
                    ValeurLueFfi(ChampFfi.NOM_AFFICHE, "Garage", emptyList()),
                    ValeurLueFfi(ChampFfi.TELEPHONE, "+33478000000", emptyList()),
                ),
                emptyList(),
            ),
            EntreesSim.contact(1, 3, " Garage ", "+33478000000"),
        )
    }

    @Test
    fun `un enregistrement SIM vide est ignore`() {
        assertNull(EntreesSim.contact(1, 4, "", null))
    }
}

// Traduction des dates en événements d'agenda, éprouvée sur la JVM.
// [impl->req~dates-dans-l-agenda~1]
package fr.lemesre.unicontacts.agenda

import fr.lemesre.unicontacts.coeur.DateFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import org.junit.Assert.assertEquals
import org.junit.Test

class EvenementsAgendaTest {
    private fun fiche(nom: String, vararg dates: DateFfi) = FicheFfi(
        id = "id-$nom",
        nom = nom,
        champs = emptyList(),
        sources = emptyList(),
        relations = emptyList(),
        estMoi = false,
        groupes = emptyList(),
        dates = dates.toList(),
    )

    private val naissance = DateFfi("naissance", "Anniversaire", "--05-17", null, 5u, 17u)
    private val mariage = DateFfi("mariage", "Anniversaire de mariage", "2005-06-11", 2005, 6u, 11u)

    @Test
    fun chaqueDateDonneUnEvenementTitreParSonType() {
        val titres = EvenementsAgenda.attendus(listOf(fiche("Camille", naissance, mariage))).map { it.titre }

        assertEquals(listOf("Anniversaire de Camille", "Anniversaire de mariage de Camille"), titres)
    }

    @Test
    fun uneDateAjouteeSeCreeEtUneDateRetireeSeSupprime() {
        val avant = EvenementsAgenda.attendus(listOf(fiche("Camille", naissance)))
        val connus = avant.associate { it.cle to 41L }

        val plan = EvenementsAgenda.plan(EvenementsAgenda.attendus(listOf(fiche("Camille", mariage))), connus)

        assertEquals(listOf("Anniversaire de mariage de Camille") to listOf(41L), plan.aCreer.map { it.titre } to plan.aSupprimer)
    }

    @Test
    fun unRenommageRemplaceLEvenement() {
        val connus = EvenementsAgenda.attendus(listOf(fiche("Camille", naissance))).associate { it.cle to 7L }

        val plan = EvenementsAgenda.plan(EvenementsAgenda.attendus(listOf(fiche("Camille E.", naissance))), connus)

        assertEquals(1 to listOf(7L), plan.aCreer.size to plan.aSupprimer)
    }

    @Test
    fun uneDateSansAnneeCommenceLAnneeCourante() {
        val evenement = EvenementsAgenda.attendus(listOf(fiche("Camille", naissance))).first()

        assertEquals(2026, EvenementsAgenda.anneeDeDepart(evenement, 2026))
    }
}

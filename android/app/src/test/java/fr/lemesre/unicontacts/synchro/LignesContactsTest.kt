// Traduction des fiches en lignes de contacts Android, éprouvée sur la JVM.
// [impl->req~ecriture-sous-compte-unique~1]
// [utest->req~synchronisation-relations~1]
// [utest->req~synchronisation-dates~1]
package fr.lemesre.unicontacts.synchro

import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.ChampValeursFfi
import fr.lemesre.unicontacts.coeur.DateFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import fr.lemesre.unicontacts.coeur.RelationFfi
import fr.lemesre.unicontacts.coeur.ValeurFfi
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class LignesContactsTest {
    private fun valeur(texte: String, vararg types: String) = ValeurFfi(texte, types.toList(), "Nextcloud", 0L)

    private fun fiche(vararg champs: Pair<ChampFfi, List<ValeurFfi>>) = FicheFfi(
        id = "f1",
        nom = "Camille Exemple",
        champs = champs.map { ChampValeursFfi(it.first, it.second) },
        sources = listOf("Nextcloud", "Google"),
        relations = emptyList(),
        estMoi = false,
        groupes = emptyList(),
        dates = emptyList(),
    )

    @Test
    fun unConjointSEcritParSonNomEtLeTypeAndroidConjoint() {
        val lignes = LignesContacts.lignes(
            fiche().copy(relations = listOf(RelationFfi("conjoint", "Conjoint", "f2", "Dominique Fictif"))),
        )

        assertEquals(
            mapOf("data1" to "Dominique Fictif", "data2" to "14"),
            lignes.first { it.mimetype == Mime.RELATION }.colonnes,
        )
    }

    @Test
    fun lesDatesDeviennentDesEvenementsSelonLeurType() {
        val lignes = LignesContacts.lignes(
            fiche().copy(
                dates = listOf(
                    DateFfi("naissance", "Anniversaire", "--05-17", null, 5u, 17u),
                    DateFfi("mariage", "Anniversaire de mariage", "2005-06-11", 2005, 6u, 11u),
                    DateFfi("fete", "Fête du prénom", "--05-18", null, 5u, 18u),
                ),
            ),
        )

        assertEquals(
            listOf(
                mapOf("data1" to "--05-17", "data2" to "3"),
                mapOf("data1" to "2005-06-11", "data2" to "1"),
                mapOf("data1" to "--05-18", "data2" to "0", "data3" to "Fête du prénom"),
            ),
            lignes.filter { it.mimetype == Mime.EVENEMENT }.map { it.colonnes },
        )
    }

    @Test
    fun unMedecinSEcritEnTypePersonnaliseAvecSonLibelle() {
        val colonnes = LignesContacts.relation(RelationFfi("medecin", "Médecin", "f3", "Docteur Martin")).colonnes

        assertEquals(mapOf("data1" to "Docteur Martin", "data2" to "0", "data3" to "Médecin"), colonnes)
    }

    @Test
    fun uneFicheDeDeuxSourcesNaQuUneLigneDeNom() {
        val lignes = LignesContacts.lignes(fiche(ChampFfi.EMAIL to listOf(valeur("a@example.org"))))

        assertEquals(1, lignes.count { it.mimetype == Mime.NOM })
    }

    @Test
    fun leNomStructureRemplitNomEtPrenom() {
        val lignes = LignesContacts.lignes(fiche(ChampFfi.NOM_STRUCTURE to listOf(valeur("Exemple;Camille;;;"))))

        val nom = lignes.first { it.mimetype == Mime.NOM }.colonnes
        assertEquals(mapOf("data1" to "Camille Exemple", "data3" to "Exemple", "data2" to "Camille"), nom)
    }

    @Test
    fun unMobileEstDeTypeMobile() {
        val lignes = LignesContacts.lignes(fiche(ChampFfi.TELEPHONE to listOf(valeur("+33639980000", "cell"))))

        assertEquals("2", lignes.first { it.mimetype == Mime.TELEPHONE }.colonnes["data2"])
    }

    @Test
    fun uneAdresseSeDecoupeEnComposantes() {
        val adresse = valeur(";;12 rue de l'Exemple;Exempleville;;99999;France", "home")

        val colonnes = LignesContacts.lignes(fiche(ChampFfi.ADRESSE to listOf(adresse)))
            .first { it.mimetype == Mime.ADRESSE }.colonnes

        assertEquals(
            listOf("12 rue de l'Exemple", "Exempleville", "99999", "France", "1"),
            listOf(colonnes["data4"], colonnes["data7"], colonnes["data9"], colonnes["data10"], colonnes["data2"]),
        )
    }

    @Test
    fun uneDateVcardDevientUneDateAndroid() {
        assertEquals(listOf("1980-05-17", "1980-05-17", "--05-17", null),
            listOf("19800517", "1980-05-17", "--0517", "bientôt").map { LignesContacts.date(it) })
    }

    @Test
    fun unPointVirguleEchappeNeSeparePasLesComposantes() {
        assertEquals(listOf("A;B", "C"), LignesContacts.composantes("A\\;B;C"))
    }

    @Test
    fun lEmpreinteChangeQuandUneValeurChange() {
        val avant = LignesContacts.lignes(fiche(ChampFfi.NOTE to listOf(valeur("une"))))
        val apres = LignesContacts.lignes(fiche(ChampFfi.NOTE to listOf(valeur("deux"))))

        assertNotEquals(LignesContacts.empreinte(avant), LignesContacts.empreinte(apres))
    }

    @Test
    fun lEmpreinteNeDependPasDeLOrdreDesColonnes() {
        val a = listOf(Ligne(Mime.NOTE, linkedMapOf("data1" to "x", "data2" to "y")))
        val b = listOf(Ligne(Mime.NOTE, linkedMapOf("data2" to "y", "data1" to "x")))

        assertEquals(LignesContacts.empreinte(a), LignesContacts.empreinte(b))
    }
}

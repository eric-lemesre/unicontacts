// Actions et détection des canaux, éprouvées sur la JVM.
// [impl->req~actions-triees-par-preference~2]
// [impl->req~detection-assistee-canaux~1]
package fr.lemesre.unicontacts.canaux

import fr.lemesre.unicontacts.coeur.CanalFfi
import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.ChampValeursFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import fr.lemesre.unicontacts.coeur.ValeurFfi
import org.junit.Assert.assertEquals
import org.junit.Test

class CanauxTest {
    private fun fiche(numeros: List<String>, canaux: List<CanalFfi> = emptyList()) = FicheFfi(
        id = "f1",
        nom = "Camille",
        champs = listOf(ChampValeursFfi(ChampFfi.TELEPHONE, numeros.map { ValeurFfi(it, emptyList(), "Nextcloud", 0L) })),
        sources = emptyList(),
        relations = emptyList(),
        estMoi = false,
        groupes = emptyList(),
        dates = emptyList(),
        canaux = canaux,
        classementPro = emptyList(),
        classementPerso = emptyList(),
    )

    @Test
    fun chaqueCanalOuvreSonApplication() {
        val uris = listOf(
            "appel" to "06 39 98 00 00",
            "sms" to "+33639980000",
            "email" to "camille@example.org",
            "whatsapp" to "06 39 98 00 00",
            "signal" to "06 39 98 00 00",
            "telegram" to "@camille",
            "matrix" to "@camille:example.org",
        ).map { (code, identifiant) -> ActionsCanaux.uri(code, identifiant) }

        assertEquals(
            listOf(
                "tel:+33639980000",
                "smsto:+33639980000",
                "mailto:camille@example.org",
                "https://wa.me/33639980000",
                "https://signal.me/#p/+33639980000",
                "https://t.me/camille",
                "https://matrix.to/#/@camille:example.org",
            ),
            uris,
        )
    }

    @Test
    fun unIdentifiantNeGlisseNiParametreNiFragmentDansLAdresse() {
        val uris = listOf(
            ActionsCanaux.uri("email", "camille@example.org?bcc=espion@example.org&body=x"),
            ActionsCanaux.uri("matrix", "@camille:example.org#/!salle"),
            ActionsCanaux.uri("telegram", "@camille/../?start=x"),
        )

        assertEquals(
            listOf(
                "mailto:camille@example.org%3Fbcc%3Despion@example.org%26body%3Dx",
                "https://matrix.to/#/@camille:example.org%23%2F%21salle",
                null,
            ),
            uris,
        )
    }

    @Test
    fun uneMessagerieQuiADeclareUnNumeroDeLaFicheEstSuggeree() {
        val suggestions = DetectionCanaux.suggestions(
            fiche(listOf("06 39 98 00 00")),
            listOf(Declaration("signal", listOf("+33 6 39 98 00 00")), Declaration("whatsapp", listOf("+33611111111"))),
        )

        assertEquals(listOf(Suggestion("signal", "Signal", "06 39 98 00 00")), suggestions)
    }

    @Test
    fun unCanalDejaDeclareNEstPlusSuggere() {
        val signal = CanalFfi("signal:+33639980000", "signal", "Signal", "+33639980000", null, true, false, false)

        val suggestions = DetectionCanaux.suggestions(
            fiche(listOf("06 39 98 00 00"), listOf(signal)),
            listOf(Declaration("signal", listOf("+33639980000"))),
        )

        assertEquals(emptyList<Suggestion>(), suggestions)
    }

    @Test
    fun leNumeroDUnIdentifiantWhatsAppSeRetrouve() {
        assertEquals("+33639980000", DetectionCanaux.numeroWhatsApp("33639980000@s.whatsapp.net"))
    }
}

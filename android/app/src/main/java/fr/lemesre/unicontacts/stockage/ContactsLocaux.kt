// Traduction d'un contact brut du stockage local Android en contact à
// importer : réciproque de LignesContacts (ADR-0015). Code pur, sans classe
// Android, éprouvé sur la JVM.
// [impl->req~import-stockage-local~1]
package fr.lemesre.unicontacts.stockage

import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.ContactLuFfi
import fr.lemesre.unicontacts.coeur.DateLueFfi
import fr.lemesre.unicontacts.coeur.ValeurLueFfi
import fr.lemesre.unicontacts.synchro.Ligne
import fr.lemesre.unicontacts.synchro.Mime

object ContactsLocaux {
    private const val PREFIXE = "local-"

    /** Identifiant d'import d'un contact brut. */
    fun uid(idBrut: Long): String = "$PREFIXE$idBrut"

    /** Contact brut désigné par un identifiant d'import, s'il en est un. */
    fun idBrut(uid: String): Long? = uid.takeIf { it.startsWith(PREFIXE) }?.removePrefix(PREFIXE)?.toLongOrNull()

    /**
     * Contact à importer, d'après les lignes de données du contact brut et
     * son nom affiché. Les relations ne se lisent pas : elles désignent une
     * autre fiche, que le stockage local ne connaît que par son nom.
     */
    fun contact(idBrut: Long, nomAffiche: String?, lignes: List<Ligne>): ContactLuFfi {
        val valeurs = mutableListOf<ValeurLueFfi>()
        val dates = mutableListOf<DateLueFfi>()
        fun ajouter(champ: ChampFfi, texte: String?, types: List<String> = emptyList()) {
            if (!texte.isNullOrBlank()) valeurs += ValeurLueFfi(champ, texte.trim(), types)
        }
        var nomVu = false
        for (ligne in lignes) {
            val c = ligne.colonnes
            when (ligne.mimetype) {
                Mime.NOM -> {
                    nomVu = nomVu || !c["data1"].isNullOrBlank()
                    ajouter(ChampFfi.NOM_AFFICHE, c["data1"])
                    ajouter(ChampFfi.NOM_STRUCTURE, structure(listOf("data3", "data2", "data5", "data4", "data6").map { c[it] }))
                }
                Mime.TELEPHONE -> ajouter(ChampFfi.TELEPHONE, c["data1"], typesTelephone(c["data2"]))
                Mime.EMAIL -> ajouter(ChampFfi.EMAIL, c["data1"], typesAdresse(c["data2"]))
                Mime.ADRESSE -> {
                    val parties = listOf("data5", "data6", "data4", "data7", "data8", "data9", "data10").map { c[it] }
                    val texte = structure(parties)
                        ?: c["data1"]?.takeIf { it.isNotBlank() }?.let { structure(listOf(null, null, it)) }
                    ajouter(ChampFfi.ADRESSE, texte, typesAdresse(c["data2"]))
                }
                Mime.ORGANISATION -> {
                    ajouter(ChampFfi.ORGANISATION, structure(listOf(c["data1"], c["data5"])))
                    ajouter(ChampFfi.FONCTION, c["data4"])
                }
                Mime.NOTE -> ajouter(ChampFfi.NOTE, c["data1"])
                Mime.SITE -> ajouter(ChampFfi.URL, c["data1"])
                Mime.EVENEMENT -> c["data1"]?.takeIf { it.isNotBlank() }?.let { dates += DateLueFfi(codeEvenement(c), it.trim()) }
            }
        }
        if (!nomVu) ajouter(ChampFfi.NOM_AFFICHE, nomAffiche)
        return ContactLuFfi(uid(idBrut), valeurs, dates)
    }

    /** Type d'événement (ADR-0011) : 3 anniversaire, 1 mariage, 0 libellé, 2 autre. */
    private fun codeEvenement(colonnes: Map<String, String>): String = when (colonnes["data2"]) {
        "3" -> "naissance"
        "1" -> "mariage"
        "0" -> colonnes["data3"]?.takeIf { it.isNotBlank() } ?: "Autre"
        else -> "Autre"
    }

    /** Types de téléphone : 1 domicile, 2 mobile, 3 travail, 4 et 5 fax, 17 mobile pro. */
    fun typesTelephone(type: String?): List<String> = when (type) {
        "1" -> listOf("home")
        "2" -> listOf("cell")
        "3" -> listOf("work")
        "4" -> listOf("fax", "work")
        "5" -> listOf("fax", "home")
        "17" -> listOf("cell", "work")
        else -> emptyList()
    }

    /** Types d'e-mail et d'adresse : 1 domicile, 2 travail. */
    fun typesAdresse(type: String?): List<String> = when (type) {
        "1" -> listOf("home")
        "2" -> listOf("work")
        else -> emptyList()
    }

    /**
     * Texte structuré : composantes échappées comme en vCard et séparées par
     * `;`, sans les composantes vides de fin ; null si toutes sont vides.
     */
    fun structure(parties: List<String?>): String? {
        val echappees = parties.map { partie ->
            partie.orEmpty().trim()
                .replace("\\", "\\\\")
                .replace(";", "\\;")
                .replace(",", "\\,")
                .replace("\n", "\\n")
        }
        if (echappees.all { it.isEmpty() }) return null
        return echappees.dropLastWhile { it.isEmpty() }.joinToString(";")
    }
}

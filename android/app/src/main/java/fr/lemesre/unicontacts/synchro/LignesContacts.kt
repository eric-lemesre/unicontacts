// Traduction d'une fiche du référentiel en lignes de données des contacts
// Android : un contact brut par fiche, quel que soit le nombre de ses
// sources. Code pur, sans classe Android, éprouvé sur la JVM.
// [impl->req~ecriture-sous-compte-unique~1]
package fr.lemesre.unicontacts.synchro

import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import fr.lemesre.unicontacts.coeur.ValeurFfi
import java.security.MessageDigest

/** Une ligne de la table `Data` : son type MIME et ses colonnes. */
data class Ligne(val mimetype: String, val colonnes: Map<String, String>)

/** Types MIME et colonnes de `ContactsContract.CommonDataKinds`. */
object Mime {
    const val NOM = "vnd.android.cursor.item/name"
    const val TELEPHONE = "vnd.android.cursor.item/phone_v2"
    const val EMAIL = "vnd.android.cursor.item/email_v2"
    const val ADRESSE = "vnd.android.cursor.item/postal-address_v2"
    const val ORGANISATION = "vnd.android.cursor.item/organization"
    const val NOTE = "vnd.android.cursor.item/note"
    const val SITE = "vnd.android.cursor.item/website"
    const val EVENEMENT = "vnd.android.cursor.item/contact_event"
}

object LignesContacts {
    /** Lignes de données d'une fiche, dans un ordre stable. */
    fun lignes(fiche: FicheFfi): List<Ligne> {
        val valeurs = fiche.champs.associate { it.champ to it.valeurs }
        fun de(champ: ChampFfi): List<ValeurFfi> = valeurs[champ].orEmpty()

        val lignes = mutableListOf<Ligne>()
        lignes += nom(fiche.nom, de(ChampFfi.NOM_STRUCTURE).firstOrNull()?.texte)
        de(ChampFfi.TELEPHONE).forEach {
            lignes += Ligne(Mime.TELEPHONE, mapOf("data1" to it.texte, "data2" to typeTelephone(it.types)))
        }
        de(ChampFfi.EMAIL).forEach {
            lignes += Ligne(Mime.EMAIL, mapOf("data1" to it.texte, "data2" to typeAdresse(it.types)))
        }
        de(ChampFfi.ADRESSE).forEach { lignes += adresse(it) }
        val organisation = de(ChampFfi.ORGANISATION).firstOrNull()?.texte
        val fonction = de(ChampFfi.FONCTION).firstOrNull()?.texte
        if (organisation != null || fonction != null) {
            val parties = composantes(organisation.orEmpty())
            lignes += Ligne(
                Mime.ORGANISATION,
                listOfNotNull(
                    parties.getOrNull(0)?.takeIf { it.isNotBlank() }?.let { "data1" to it },
                    fonction?.let { "data4" to it },
                    parties.getOrNull(1)?.takeIf { it.isNotBlank() }?.let { "data5" to it },
                ).toMap(),
            )
        }
        de(ChampFfi.NOTE).firstOrNull()?.let { lignes += Ligne(Mime.NOTE, mapOf("data1" to it.texte)) }
        de(ChampFfi.URL).forEach { lignes += Ligne(Mime.SITE, mapOf("data1" to it.texte, "data2" to "7")) }
        de(ChampFfi.ANNIVERSAIRE).firstOrNull()?.let { date(it.texte) }?.let {
            lignes += Ligne(Mime.EVENEMENT, mapOf("data1" to it, "data2" to "3"))
        }
        return lignes
    }

    /**
     * Empreinte des lignes : gardée dans `SYNC1` du contact brut, elle évite
     * de réécrire un contact inchangé.
     */
    fun empreinte(lignes: List<Ligne>): String {
        val texte = lignes.joinToString("\n") { ligne ->
            ligne.mimetype + "|" + ligne.colonnes.toSortedMap().entries.joinToString("|") { "${it.key}=${it.value}" }
        }
        val octets = MessageDigest.getInstance("SHA-256").digest(texte.toByteArray(Charsets.UTF_8))
        return octets.joinToString("") { "%02x".format(it) }
    }

    /** Nom structuré : nom affiché, puis nom, prénom, autres prénoms, préfixe, suffixe. */
    private fun nom(affiche: String, structure: String?): Ligne {
        val parties = composantes(structure.orEmpty())
        val colonnes = mutableMapOf("data1" to affiche)
        listOf("data3", "data2", "data5", "data4", "data6").forEachIndexed { i, colonne ->
            parties.getOrNull(i)?.takeIf { it.isNotBlank() }?.let { colonnes[colonne] = it }
        }
        return Ligne(Mime.NOM, colonnes)
    }

    /** Adresse : boîte postale, complément, rue, ville, région, code postal, pays. */
    private fun adresse(valeur: ValeurFfi): Ligne {
        val parties = composantes(valeur.texte)
        val colonnes = mutableMapOf("data2" to typeAdresse(valeur.types))
        val ordre = listOf("data5", "data6", "data4", "data7", "data8", "data9", "data10")
        ordre.forEachIndexed { i, colonne ->
            parties.getOrNull(i)?.takeIf { it.isNotBlank() }?.let { colonnes[colonne] = it }
        }
        val formatee = listOfNotNull(
            parties.getOrNull(2),
            listOfNotNull(parties.getOrNull(5), parties.getOrNull(3)).filter { it.isNotBlank() }.joinToString(" "),
            parties.getOrNull(6),
        ).filter { it.isNotBlank() }.joinToString("\n")
        if (formatee.isNotEmpty()) colonnes["data1"] = formatee
        return Ligne(Mime.ADRESSE, colonnes)
    }

    /** Types de téléphone : 2 mobile, 3 travail, 1 domicile, 7 autre. */
    fun typeTelephone(types: List<String>): String = when {
        "cell" in types -> "2"
        "work" in types -> "3"
        "home" in types -> "1"
        else -> "7"
    }

    /** Types d'e-mail et d'adresse : 2 travail, 1 domicile, 3 autre. */
    fun typeAdresse(types: List<String>): String = when {
        "work" in types -> "2"
        "home" in types -> "1"
        else -> "3"
    }

    /** Date vCard (`19800517`, `1980-05-17`, `--0517`) au format Android. */
    fun date(texte: String): String? {
        val sansAnnee = texte.startsWith("--")
        val chiffres = texte.substringBefore('T').filter { it.isDigit() }
        return when {
            sansAnnee && chiffres.length == 4 -> "--${chiffres.substring(0, 2)}-${chiffres.substring(2)}"
            chiffres.length == 8 -> "${chiffres.substring(0, 4)}-${chiffres.substring(4, 6)}-${chiffres.substring(6)}"
            else -> null
        }
    }

    /** Composantes d'une valeur structurée, échappements vCard retirés. */
    fun composantes(texte: String): List<String> {
        val resultat = mutableListOf<String>()
        val courante = StringBuilder()
        var echappe = false
        for (c in texte) {
            when {
                echappe -> {
                    courante.append(if (c == 'n' || c == 'N') '\n' else c)
                    echappe = false
                }
                c == '\\' -> echappe = true
                c == ';' -> {
                    resultat += courante.toString().trim()
                    courante.clear()
                }
                else -> courante.append(c)
            }
        }
        resultat += courante.toString().trim()
        return resultat
    }
}

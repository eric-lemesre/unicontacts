// Dates des fiches traduites en événements annuels d'agenda, et plan des
// changements à reporter. Code pur, sans classe Android, éprouvé sur la JVM.
// [impl->req~dates-dans-l-agenda~1]
package fr.lemesre.unicontacts.agenda

import fr.lemesre.unicontacts.coeur.DateFfi
import fr.lemesre.unicontacts.coeur.FicheFfi

/** Événement annuel attendu dans l'agenda, désigné par sa clé. */
data class Evenement(val cle: String, val titre: String, val annee: Int?, val mois: Int, val jour: Int)

/** Changements à reporter dans l'agenda. */
data class Plan(val aCreer: List<Evenement>, val aSupprimer: List<Long>)

object EvenementsAgenda {
    /** Titre de l'événement d'une date. */
    fun titre(nom: String, date: DateFfi): String = when (date.code) {
        "naissance" -> "Anniversaire de $nom"
        "mariage" -> "Anniversaire de mariage de $nom"
        "fete" -> "Fête de $nom"
        else -> "${date.libelle} — $nom"
    }

    /**
     * Événements attendus : un par date de fiche. La clé porte la fiche, le
     * type, la date et le titre : changer l'un d'eux remplace l'événement.
     */
    fun attendus(fiches: List<FicheFfi>): List<Evenement> = fiches.flatMap { fiche ->
        fiche.dates.map { date ->
            val titre = titre(fiche.nom, date)
            Evenement(
                cle = listOf(fiche.id, date.code, date.texte, titre).joinToString("\u001f"),
                titre = titre,
                annee = date.annee,
                mois = date.mois.toInt(),
                jour = date.jour.toInt(),
            )
        }
    }

    /**
     * Plan de report : créer les événements attendus qui n'existent pas,
     * supprimer ceux qui ne sont plus attendus.
     */
    fun plan(attendus: List<Evenement>, connus: Map<String, Long>): Plan {
        val cles = attendus.map { it.cle }.toSet()
        return Plan(
            aCreer = attendus.filter { it.cle !in connus }.distinctBy { it.cle },
            aSupprimer = connus.filterKeys { it !in cles }.values.toList(),
        )
    }

    /**
     * Année du premier événement de la récurrence : l'année de la date si
     * elle est connue, sinon l'année donnée (l'année en cours).
     */
    fun anneeDeDepart(evenement: Evenement, anneeCourante: Int): Int = evenement.annee ?: anneeCourante
}

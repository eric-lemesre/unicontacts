// Report des dates des fiches dans un agenda du téléphone choisi par
// l'utilisateur, en événements annuels récurrents : l'agenda émet les
// rappels (OQ-004). L'application ne garde que la correspondance entre ses
// clés et les événements qu'elle a créés.
// [impl->req~dates-dans-l-agenda~1]
package fr.lemesre.unicontacts.agenda

import android.Manifest
import android.content.ContentValues
import android.content.Context
import android.content.pm.PackageManager
import android.provider.CalendarContract
import androidx.core.content.edit
import fr.lemesre.unicontacts.coeur.FicheFfi
import java.util.Calendar
import java.util.TimeZone

/** Agenda proposé : identifiant et nom affiché. */
data class Agenda(val id: Long, val nom: String)

class AgendaDates(private val context: Context) {
    private val preferences = context.getSharedPreferences("agenda", Context.MODE_PRIVATE)

    /** Agenda choisi, s'il y en a un. */
    var choisi: Long?
        get() = preferences.getLong(CLE_AGENDA, AUCUN).takeIf { it != AUCUN }
        set(valeur) {
            preferences.edit { putLong(CLE_AGENDA, valeur ?: AUCUN) }
        }

    private fun autorise(): Boolean = listOf(Manifest.permission.READ_CALENDAR, Manifest.permission.WRITE_CALENDAR)
        .all { context.checkSelfPermission(it) == PackageManager.PERMISSION_GRANTED }

    /** Agendas où l'application peut écrire. */
    fun agendas(): List<Agenda> {
        if (!autorise()) return emptyList()
        val agendas = mutableListOf<Agenda>()
        context.contentResolver.query(
            CalendarContract.Calendars.CONTENT_URI,
            arrayOf(CalendarContract.Calendars._ID, CalendarContract.Calendars.CALENDAR_DISPLAY_NAME),
            "${CalendarContract.Calendars.CALENDAR_ACCESS_LEVEL} >= ?",
            arrayOf(CalendarContract.Calendars.CAL_ACCESS_CONTRIBUTOR.toString()),
            null,
        )?.use { curseur ->
            while (curseur.moveToNext()) agendas += Agenda(curseur.getLong(0), curseur.getString(1).orEmpty())
        }
        return agendas
    }

    /**
     * Reporte les dates des fiches dans l'agenda choisi : crée les
     * événements nouveaux, supprime ceux qui ne sont plus attendus. Un
     * changement d'agenda retire les événements de l'ancien. Sans agenda
     * choisi ni permission, ne fait rien.
     */
    fun reporter(fiches: List<FicheFfi>) {
        if (!autorise()) return
        val agenda = choisi
        val connus = connus()
        val ancien = preferences.getLong(CLE_AGENDA_DES_EVENEMENTS, AUCUN)
        val attendus = if (agenda == null) emptyList() else EvenementsAgenda.attendus(fiches)
        val plan = if (ancien != (agenda ?: AUCUN)) {
            Plan(attendus, connus.values.toList())
        } else {
            EvenementsAgenda.plan(attendus, connus)
        }
        val restants = if (ancien != (agenda ?: AUCUN)) mutableMapOf() else connus.toMutableMap()
        plan.aSupprimer.forEach { id ->
            context.contentResolver.delete(
                CalendarContract.Events.CONTENT_URI,
                "${CalendarContract.Events._ID} = ?",
                arrayOf(id.toString()),
            )
            restants.values.remove(id)
        }
        if (agenda != null) {
            val anneeCourante = Calendar.getInstance().get(Calendar.YEAR)
            plan.aCreer.forEach { evenement ->
                creer(agenda, evenement, anneeCourante)?.let { restants[evenement.cle] = it }
            }
        }
        enregistrer(restants, agenda ?: AUCUN)
    }

    private fun creer(agenda: Long, evenement: Evenement, anneeCourante: Int): Long? {
        val debut = Calendar.getInstance(TimeZone.getTimeZone("UTC")).apply {
            clear()
            set(EvenementsAgenda.anneeDeDepart(evenement, anneeCourante), evenement.mois - 1, evenement.jour)
        }.timeInMillis
        val valeurs = ContentValues().apply {
            put(CalendarContract.Events.CALENDAR_ID, agenda)
            put(CalendarContract.Events.TITLE, evenement.titre)
            put(CalendarContract.Events.DTSTART, debut)
            put(CalendarContract.Events.DURATION, "P1D")
            put(CalendarContract.Events.ALL_DAY, 1)
            put(CalendarContract.Events.EVENT_TIMEZONE, "UTC")
            put(CalendarContract.Events.RRULE, "FREQ=YEARLY")
            put(CalendarContract.Events.DESCRIPTION, "unicontacts")
        }
        return context.contentResolver.insert(CalendarContract.Events.CONTENT_URI, valeurs)
            ?.lastPathSegment?.toLongOrNull()
    }

    private fun connus(): Map<String, Long> = preferences.getStringSet(CLE_EVENEMENTS, emptySet()).orEmpty()
        .mapNotNull { ligne ->
            val separateur = ligne.lastIndexOf('\t')
            if (separateur < 0) return@mapNotNull null
            ligne.substring(separateur + 1).toLongOrNull()?.let { ligne.substring(0, separateur) to it }
        }.toMap()

    private fun enregistrer(evenements: Map<String, Long>, agenda: Long) {
        preferences.edit {
            putStringSet(CLE_EVENEMENTS, evenements.map { (cle, id) -> "$cle\t$id" }.toSet())
            putLong(CLE_AGENDA_DES_EVENEMENTS, agenda)
        }
    }

    private companion object {
        const val AUCUN = -1L
        const val CLE_AGENDA = "agenda"
        const val CLE_AGENDA_DES_EVENEMENTS = "agenda-des-evenements"
        const val CLE_EVENEMENTS = "evenements"
    }
}

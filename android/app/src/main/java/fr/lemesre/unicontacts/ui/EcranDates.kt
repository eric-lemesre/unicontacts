// Dates : section de la fiche (dates, fête proposée, ajout), vue « À
// venir » filtrable par groupe, et choix de l'agenda où les reporter.
// [impl->req~types-de-dates~1]
// [impl->req~fete-du-prenom-proposee~1]
// [impl->req~vue-a-venir~1]
// [impl->req~dates-dans-l-agenda~1]
package fr.lemesre.unicontacts.ui

import android.Manifest
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.agenda.Agenda
import fr.lemesre.unicontacts.agenda.AgendaDates
import fr.lemesre.unicontacts.coeur.DateAVenirFfi
import fr.lemesre.unicontacts.coeur.DateFfi
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import fr.lemesre.unicontacts.compte.Compte
import kotlinx.coroutines.launch

/** Mois en toutes lettres. */
private val MOIS = listOf(
    "janvier", "février", "mars", "avril", "mai", "juin",
    "juillet", "août", "septembre", "octobre", "novembre", "décembre",
)

/** Date lisible : « 17 mai 1980 », ou « 17 mai » sans année. */
fun dateLisible(date: DateFfi): String {
    val jour = if (date.jour.toInt() == 1) "1er" else date.jour.toString()
    val sansAnnee = "$jour ${MOIS.getOrElse(date.mois.toInt() - 1) { "?" }}"
    return date.annee?.let { "$sansAnnee $it" } ?: sansAnnee
}

/** Date lisible d'un texte `--MM-JJ` proposé par le cœur. */
private fun jourLisible(texte: String): String {
    val mois = texte.substring(2, 4).toIntOrNull() ?: return texte
    val jour = texte.substring(5).toIntOrNull() ?: return texte
    return dateLisible(DateFfi("", "", texte, null, mois.toUByte(), jour.toUByte()))
}

/** Dates d'une fiche, sur son écran. */
@Composable
fun SectionDates(app: UnicontactsApp, fiche: FicheFfi, recharger: suspend () -> Unit) {
    val portee = rememberCoroutineScope()
    var proposees by remember { mutableStateOf(emptyList<String>()) }
    var ajout by remember { mutableStateOf(false) }
    var message by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(fiche) { proposees = horsInterface { app.coeur.fetesProposees(fiche.id) } }

    fun modifier(appel: () -> Unit) {
        portee.launch {
            message = try {
                horsInterface(appel)
                null
            } catch (erreur: ErreurFfi) {
                erreur.message
            }
            recharger()
            if (message == null) Compte.synchroniser()
        }
    }

    Column {
        Text("Dates", style = MaterialTheme.typography.titleMedium)
        fiche.dates.forEach { date ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("${date.libelle} : ${dateLisible(date)}", Modifier.weight(1f))
                TextButton(onClick = { modifier { app.coeur.retirerDate(fiche.id, date.code, date.texte) } }) {
                    Text("Retirer")
                }
            }
        }
        proposees.forEach { jour ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Fête proposée : ${jourLisible(jour)}", Modifier.weight(1f))
                TextButton(onClick = { modifier { app.coeur.ajouterDate(fiche.id, "fete", jour) } }) {
                    Text("Accepter")
                }
            }
        }
        message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        TextButton(onClick = { ajout = true }) { Text("Ajouter une date") }
    }

    if (ajout) {
        DialogueDate(onAnnuler = { ajout = false }) { code, texte ->
            ajout = false
            modifier { app.coeur.ajouterDate(fiche.id, code, texte) }
        }
    }
}

/** Ajout d'une date : type, libellé d'une date personnalisée, date. */
@Composable
private fun DialogueDate(onAnnuler: () -> Unit, onConfirmer: (code: String, date: String) -> Unit) {
    val types = listOf("naissance" to "Anniversaire", "mariage" to "Anniversaire de mariage", "fete" to "Fête du prénom")
    var code by remember { mutableStateOf("naissance") }
    var libelle by remember { mutableStateOf("") }
    var texte by remember { mutableStateOf("") }
    val autre = code == AUTRE_DATE
    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Ajouter une date") },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                (types + (AUTRE_DATE to "Autre…")).forEach { (valeur, nom) ->
                    Row(Modifier.fillMaxWidth().clickable { code = valeur }, verticalAlignment = Alignment.CenterVertically) {
                        RadioButton(selected = code == valeur, onClick = { code = valeur })
                        Text(nom)
                    }
                }
                if (autre) {
                    OutlinedTextField(libelle, { libelle = it }, label = { Text("Libellé") }, singleLine = true)
                }
                OutlinedTextField(
                    texte,
                    { texte = it },
                    label = { Text("Date (JJ/MM/AAAA, ou JJ/MM sans année)") },
                    singleLine = true,
                )
            }
        },
        confirmButton = {
            TextButton(
                enabled = texte.isNotBlank() && (!autre || libelle.isNotBlank()),
                onClick = { onConfirmer(if (autre) libelle.trim() else code, texte.trim()) },
            ) { Text("Ajouter") }
        },
        dismissButton = { TextButton(onClick = onAnnuler) { Text("Annuler") } },
    )
}

/** Marqueur du choix « Autre… » : un libellé saisi. */
private const val AUTRE_DATE = "\u0000autre"

/** Quand tombe une date à venir. */
private fun quand(date: DateAVenirFfi): String = when (date.dansJours) {
    0u -> "aujourd'hui"
    1u -> "demain"
    else -> "dans ${date.dansJours} jours"
}

/** Dates à venir, de la plus proche à la plus lointaine, filtrables par groupe. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EcranAVenir(app: UnicontactsApp, ouvrir: (Ecran) -> Unit, retour: () -> Unit) {
    var dates by remember { mutableStateOf(emptyList<DateAVenirFfi>()) }
    var filtre by remember { mutableStateOf(FILTRE_VIDE) }
    var choixFiltre by remember { mutableStateOf(false) }
    LaunchedEffect(filtre) { dates = horsInterface { app.coeur.aVenir(filtre) } }
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("À venir") },
                navigationIcon = { TextButton(onClick = retour) { Text("Retour") } },
                actions = {
                    TextButton(onClick = { choixFiltre = true }) {
                        Text(if (filtre.groupes.isEmpty()) "Groupes…" else "Groupes (${filtre.groupes.size})")
                    }
                },
            )
        },
    ) { marges ->
        LazyColumn(Modifier.padding(marges).fillMaxSize()) {
            items(dates, key = { it.ficheId + it.date.code + it.date.texte }) { date ->
                Column(
                    Modifier.fillMaxWidth().clickable { ouvrir(Ecran.Fiche(date.ficheId)) }.padding(16.dp),
                ) {
                    Text(date.nom, style = MaterialTheme.typography.titleMedium)
                    val annees = date.annees?.let { " — $it ans" }.orEmpty()
                    Text(
                        "${date.date.libelle} : ${dateLisible(date.date.copy(annee = null))}, ${quand(date)}$annees",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
                HorizontalDivider()
            }
        }
    }
    if (choixFiltre) {
        DialogueFiltre(app, filtre, onAnnuler = { choixFiltre = false }) {
            filtre = it
            choixFiltre = false
        }
    }
}

/** Choix de l'agenda où reporter les dates, sur l'écran des sources. */
@Composable
fun SectionAgenda() {
    val contexte = LocalContext.current
    val agendaDates = remember { AgendaDates(contexte.applicationContext) }
    var agendas by remember { mutableStateOf(emptyList<Agenda>()) }
    var choisi by remember { mutableStateOf(agendaDates.choisi) }
    val lanceur = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {
        agendas = agendaDates.agendas()
    }
    LaunchedEffect(Unit) {
        lanceur.launch(arrayOf(Manifest.permission.READ_CALENDAR, Manifest.permission.WRITE_CALENDAR))
    }
    fun choisir(id: Long?) {
        choisi = id
        agendaDates.choisi = id
        Compte.synchroniser()
    }
    Column {
        Text("Agenda des dates", style = MaterialTheme.typography.titleMedium)
        Text(
            "Les dates des fiches y deviennent des événements annuels ; l'agenda en émet les rappels.",
            style = MaterialTheme.typography.bodySmall,
        )
        Row(Modifier.fillMaxWidth().clickable { choisir(null) }, verticalAlignment = Alignment.CenterVertically) {
            RadioButton(selected = choisi == null, onClick = { choisir(null) })
            Text("Aucun")
        }
        agendas.forEach { agenda ->
            Row(Modifier.fillMaxWidth().clickable { choisir(agenda.id) }, verticalAlignment = Alignment.CenterVertically) {
                RadioButton(selected = choisi == agenda.id, onClick = { choisir(agenda.id) })
                Text(agenda.nom)
            }
        }
    }
}

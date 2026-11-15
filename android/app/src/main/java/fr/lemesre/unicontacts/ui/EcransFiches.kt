// Liste des fiches, avec le compteur de décisions et la section « Proches »,
// et écran d'une fiche : valeurs avec leur provenance, édition, relations,
// suppression choisie par source.
// [impl->req~boite-de-decisions~1]
// [impl->req~section-proches~1]
package fr.lemesre.unicontacts.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import fr.lemesre.unicontacts.coeur.ResumeFicheFfi
import fr.lemesre.unicontacts.coeur.SourceFfi
import fr.lemesre.unicontacts.compte.Compte
import kotlinx.coroutines.launch

/** Libellé d'un champ. */
fun libelle(champ: ChampFfi): String = when (champ) {
    ChampFfi.NOM_AFFICHE -> "Nom affiché"
    ChampFfi.NOM_STRUCTURE -> "Nom ; prénom ; autres prénoms ; préfixe ; suffixe"
    ChampFfi.ORGANISATION -> "Organisation ; service"
    ChampFfi.FONCTION -> "Fonction"
    ChampFfi.NOTE -> "Note"
    ChampFfi.ANNIVERSAIRE -> "Date de naissance (AAAA-MM-JJ)"
    ChampFfi.TELEPHONE -> "Téléphones (un par ligne)"
    ChampFfi.EMAIL -> "E-mails (un par ligne)"
    ChampFfi.ADRESSE -> "Adresses (une par ligne, champs séparés par ;)"
    ChampFfi.URL -> "Sites web (un par ligne)"
    ChampFfi.RELATION -> "Relations"
}

/** Champs édités comme du texte : tous, sauf les relations. */
val champsEditables: List<ChampFfi> = ChampFfi.entries.filter { it != ChampFfi.RELATION }

/** Vrai pour un champ multivalué : une valeur par ligne. */
fun multivalue(champ: ChampFfi): Boolean =
    champ in setOf(ChampFfi.TELEPHONE, ChampFfi.EMAIL, ChampFfi.ADRESSE, ChampFfi.URL)

/** Date lisible d'un horodatage en millisecondes. */
fun dateLisible(millis: Long): String =
    java.text.DateFormat.getDateTimeInstance(java.text.DateFormat.SHORT, java.text.DateFormat.SHORT)
        .format(java.util.Date(millis))

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EcranListe(app: UnicontactsApp, ouvrir: (Ecran) -> Unit) {
    var fiches by remember { mutableStateOf(emptyList<ResumeFicheFfi>()) }
    var proches by remember { mutableStateOf(emptyList<ResumeFicheFfi>()) }
    var decisions by remember { mutableStateOf(0u) }
    val portee = rememberCoroutineScope()
    LaunchedEffect(Unit) {
        fiches = horsInterface { app.coeur.fiches() }
        proches = horsInterface { app.coeur.proches() }
        decisions = horsInterface { app.coeur.nombreDecisions() }
    }
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("unicontacts") },
                actions = {
                    if (decisions > 0u) {
                        TextButton(onClick = { ouvrir(Ecran.Decisions) }) {
                            BadgedBox(badge = { Badge { Text(decisions.toString()) } }) { Text("Décisions") }
                        }
                    }
                    TextButton(onClick = { ouvrir(Ecran.Sources) }) { Text("Sources") }
                },
            )
        },
        floatingActionButton = {
            FloatingActionButton(onClick = {
                portee.launch { ouvrir(Ecran.Fiche(horsInterface { app.coeur.creerFiche() })) }
            }) { Text("+") }
        },
    ) { marges ->
        LazyColumn(Modifier.padding(marges).fillMaxSize()) {
            if (proches.isNotEmpty()) {
                item(key = "titre-proches") { TitreSection("Proches") }
                items(proches, key = { "proche-" + it.id }) { LigneFiche(it, ouvrir) }
                item(key = "titre-tous") { TitreSection("Tous les contacts") }
            }
            items(fiches, key = { it.id }) { LigneFiche(it, ouvrir) }
        }
    }
}

@Composable
private fun TitreSection(titre: String) {
    Text(
        titre,
        style = MaterialTheme.typography.titleSmall,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(start = 16.dp, top = 16.dp, end = 16.dp),
    )
}

@Composable
private fun LigneFiche(fiche: ResumeFicheFfi, ouvrir: (Ecran) -> Unit) {
    Column(
        Modifier
            .fillMaxWidth()
            .clickable { ouvrir(Ecran.Fiche(fiche.id)) }
            .padding(16.dp),
    ) {
        Text(fiche.nom, style = MaterialTheme.typography.titleMedium)
        listOfNotNull(fiche.telephone, fiche.email).forEach {
            Text(it, style = MaterialTheme.typography.bodySmall)
        }
    }
    HorizontalDivider()
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EcranFiche(app: UnicontactsApp, id: String, ouvrir: (Ecran) -> Unit, retour: () -> Unit) {
    var fiche by remember { mutableStateOf<FicheFfi?>(null) }
    val saisies = remember { mutableStateMapOf<ChampFfi, String>() }
    var message by remember { mutableStateOf<String?>(null) }
    var suppression by remember { mutableStateOf<List<SourceFfi>?>(null) }
    val portee = rememberCoroutineScope()

    suspend fun charger() {
        val lue = horsInterface { app.coeur.fiche(id) }
        fiche = lue
        saisies.clear()
        champsEditables.forEach { champ ->
            saisies[champ] = lue.champs.firstOrNull { it.champ == champ }?.valeurs.orEmpty()
                .joinToString("\n") { it.texte }
        }
    }
    LaunchedEffect(id) { charger() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(fiche?.nom ?: "") },
                navigationIcon = { TextButton(onClick = retour) { Text("Retour") } },
            )
        },
    ) { marges ->
        val lue = fiche ?: return@Scaffold
        Column(
            Modifier.padding(marges).padding(16.dp).verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (lue.sources.isNotEmpty()) {
                Text("Sources : " + lue.sources.joinToString(", "), style = MaterialTheme.typography.bodySmall)
            }
            SectionRelations(app, lue, ouvrir, recharger = { charger() })
            champsEditables.forEach { champ ->
                OutlinedTextField(
                    value = saisies[champ].orEmpty(),
                    onValueChange = { saisies[champ] = it },
                    label = { Text(libelle(champ)) },
                    singleLine = !multivalue(champ) && champ != ChampFfi.NOTE,
                    modifier = Modifier.fillMaxWidth(),
                )
                lue.champs.firstOrNull { it.champ == champ }?.valeurs?.forEach { valeur ->
                    Text(
                        "${valeur.texte} — ${valeur.origine}, ${dateLisible(valeur.dateMillis)}",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
            message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = {
                    portee.launch {
                        message = enregistrer(app, lue, saisies)
                        if (message == null) {
                            Compte.synchroniser()
                            charger()
                        }
                    }
                }) { Text("Enregistrer") }
                TextButton(onClick = {
                    portee.launch { suppression = horsInterface { app.coeur.sourcesDeSuppression(id) } }
                }) { Text("Supprimer") }
            }
        }
    }

    suppression?.let { possibles ->
        DialogueSuppression(possibles, onAnnuler = { suppression = null }) { choisies ->
            portee.launch {
                horsInterface { app.coeur.supprimerFiche(id, choisies) }
                Compte.synchroniser()
                suppression = null
                retour()
            }
        }
    }
}

/** Enregistre les champs modifiés ; rend le motif d'un refus, ou null. */
private suspend fun enregistrer(app: UnicontactsApp, fiche: FicheFfi, saisies: Map<ChampFfi, String>): String? {
    for (champ in champsEditables) {
        val avant = fiche.champs.firstOrNull { it.champ == champ }?.valeurs.orEmpty().map { it.texte }
        val saisi = saisies[champ].orEmpty()
        val apres = if (multivalue(champ)) saisi.lines() else listOf(saisi)
        val nettes = apres.map { it.trim() }.filter { it.isNotEmpty() }
        if (nettes == avant) continue
        try {
            horsInterface { app.coeur.modifierFiche(fiche.id, champ, nettes) }
        } catch (erreur: ErreurFfi) {
            return "${libelle(champ)} : ${erreur.message}"
        }
    }
    return null
}

/**
 * Suppression : l'utilisateur choisit explicitement les sources où elle
 * s'applique, parmi celles que leurs réglages autorisent en écriture.
 */
@Composable
private fun DialogueSuppression(possibles: List<SourceFfi>, onAnnuler: () -> Unit, onConfirmer: (List<String>) -> Unit) {
    val choisies = remember { mutableStateListOf<String>() }
    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Supprimer la fiche") },
        text = {
            Column {
                Text("Supprimer aussi de :")
                possibles.forEach { source ->
                    Row {
                        Checkbox(
                            checked = source.id in choisies,
                            onCheckedChange = { if (it) choisies += source.id else choisies -= source.id },
                        )
                        Text(source.nom, Modifier.padding(top = 12.dp))
                    }
                }
            }
        },
        confirmButton = { TextButton(onClick = { onConfirmer(choisies.toList()) }) { Text("Supprimer") } },
        dismissButton = { TextButton(onClick = onAnnuler) { Text("Annuler") } },
    )
}

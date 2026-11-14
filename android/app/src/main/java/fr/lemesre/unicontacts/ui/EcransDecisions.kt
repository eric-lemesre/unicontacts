// Liste des décisions et vue comparée : les valeurs en présence côte à côte,
// chacune avec sa source et sa date, et les choix offerts par la décision.
// [impl->req~boite-de-decisions~1]
// [impl->req~vue-comparee~1]
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
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ChoixFfi
import fr.lemesre.unicontacts.coeur.DecisionFfi
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.coeur.NatureFfi
import fr.lemesre.unicontacts.coeur.ResolutionFfi
import fr.lemesre.unicontacts.coeur.StatutFfi
import fr.lemesre.unicontacts.coeur.ValeurFfi
import fr.lemesre.unicontacts.compte.Compte
import kotlinx.coroutines.launch

/** Intitulé d'une décision. */
fun intitule(decision: DecisionFfi): String = when (val nature = decision.nature) {
    is NatureFfi.Divergence -> "Divergence : ${libelle(nature.champ)}"
    NatureFfi.Suppression -> "Supprimée de ${decision.sourceNom}"
    is NatureFfi.DoublonProbable -> "Doublon probable de ${nature.autreNom}"
    NatureFfi.NouvellePhoto -> "Nouvelle photo"
}

private fun libelle(choix: ChoixFfi): String = when (choix) {
    ChoixFfi.GARDER_REFERENTIEL -> "Garder unicontacts"
    ChoixFfi.GARDER_SOURCE -> "Garder la source"
    ChoixFfi.GARDER_LES_DEUX -> "Garder les deux"
    ChoixFfi.SAISIR -> "Saisir"
    ChoixFfi.PROPAGER -> "Supprimer partout"
    ChoixFfi.RESTAURER -> "Restaurer sur la source"
    ChoixFfi.DETACHER -> "Détacher de la source"
    ChoixFfi.FUSIONNER -> "Réunir les fiches"
    ChoixFfi.DISTINGUER -> "Garder deux fiches"
    ChoixFfi.REPORTER -> "Reporter"
}

private fun resolution(choix: ChoixFfi, saisie: String): ResolutionFfi? = when (choix) {
    ChoixFfi.GARDER_REFERENTIEL -> ResolutionFfi.GarderReferentiel
    ChoixFfi.GARDER_SOURCE -> ResolutionFfi.GarderSource
    ChoixFfi.GARDER_LES_DEUX -> ResolutionFfi.GarderLesDeux
    ChoixFfi.SAISIR -> ResolutionFfi.Saisir(saisie.lines().map { it.trim() }.filter { it.isNotEmpty() })
    ChoixFfi.PROPAGER -> ResolutionFfi.Propager
    ChoixFfi.RESTAURER -> ResolutionFfi.Restaurer
    ChoixFfi.DETACHER -> ResolutionFfi.Detacher
    ChoixFfi.FUSIONNER -> ResolutionFfi.Fusionner
    ChoixFfi.DISTINGUER -> ResolutionFfi.Distinguer
    ChoixFfi.REPORTER -> null
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EcranDecisions(app: UnicontactsApp, ouvrir: (Ecran) -> Unit, retour: () -> Unit) {
    var decisions by remember { mutableStateOf(emptyList<DecisionFfi>()) }
    LaunchedEffect(Unit) { decisions = horsInterface { app.coeur.decisions() } }
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Décisions (${decisions.size})") },
                navigationIcon = { TextButton(onClick = retour) { Text("Retour") } },
            )
        },
    ) { marges ->
        LazyColumn(Modifier.padding(marges).fillMaxSize()) {
            items(decisions, key = { it.id }) { decision ->
                Column(
                    Modifier
                        .fillMaxWidth()
                        .clickable { ouvrir(Ecran.Decision(decision.id)) }
                        .padding(16.dp),
                ) {
                    Text(decision.ficheNom, style = MaterialTheme.typography.titleMedium)
                    Text(intitule(decision))
                    val reportee = if (decision.statut == StatutFfi.REPORTEE) " — reportée" else ""
                    Text(
                        "${decision.sourceNom}, ${dateLisible(decision.creeeLeMillis)}$reportee",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
                HorizontalDivider()
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EcranDecision(app: UnicontactsApp, id: String, retour: () -> Unit) {
    var decision by remember { mutableStateOf<DecisionFfi?>(null) }
    var similaires by remember { mutableStateOf(0u) }
    var enLot by remember { mutableStateOf(false) }
    var saisie by remember { mutableStateOf("") }
    var message by remember { mutableStateOf<String?>(null) }
    val portee = rememberCoroutineScope()
    LaunchedEffect(id) {
        decision = horsInterface { app.coeur.decisions().firstOrNull { it.id == id } }
        similaires = horsInterface { app.coeur.casSimilaires(id) }
    }
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(decision?.ficheNom ?: "") },
                navigationIcon = { TextButton(onClick = retour) { Text("Retour") } },
            )
        },
    ) { marges ->
        val lue = decision ?: return@Scaffold
        Column(
            Modifier.padding(marges).padding(16.dp).verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(intitule(lue), style = MaterialTheme.typography.titleMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Colonne("unicontacts", lue.referentiel, Modifier.weight(1f))
                Colonne(lue.sourceNom, lue.source, Modifier.weight(1f))
            }
            if (lue.base.isNotEmpty()) {
                Text("Dernier état synchronisé : " + lue.base.joinToString(", "), style = MaterialTheme.typography.bodySmall)
            }
            if (ChoixFfi.SAISIR in lue.choix) {
                OutlinedTextField(
                    value = saisie,
                    onValueChange = { saisie = it },
                    label = { Text("Valeur saisie") },
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            if (similaires > 0u) {
                Row {
                    Checkbox(checked = enLot, onCheckedChange = { enLot = it })
                    Text("Appliquer aussi aux $similaires cas similaires", Modifier.padding(top = 12.dp))
                }
            }
            message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
            lue.choix.forEach { choix ->
                OutlinedButton(
                    onClick = {
                        portee.launch {
                            try {
                                val resolution = resolution(choix, saisie)
                                if (resolution == null) {
                                    horsInterface { app.coeur.reporter(id) }
                                } else {
                                    horsInterface { app.coeur.resoudre(id, resolution, enLot) }
                                    Compte.synchroniser()
                                }
                                retour()
                            } catch (erreur: ErreurFfi) {
                                message = erreur.message
                            }
                        }
                    },
                    modifier = Modifier.fillMaxWidth(),
                ) { Text(libelle(choix)) }
            }
        }
    }
}

@Composable
private fun Colonne(titre: String, valeurs: List<ValeurFfi>, modifier: Modifier) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(titre, style = MaterialTheme.typography.labelLarge)
        if (valeurs.isEmpty()) Text("(vide)")
        valeurs.forEach { valeur ->
            Text(valeur.texte)
            Text("${valeur.origine}, ${dateLisible(valeur.dateMillis)}", style = MaterialTheme.typography.bodySmall)
        }
    }
}

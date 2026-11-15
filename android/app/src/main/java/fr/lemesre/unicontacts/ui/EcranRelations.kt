// Relations d'une fiche : qualité de fiche « moi », liens typés vers
// d'autres fiches, ajout et retrait. Les réciproques et les inverses sont
// l'affaire du cœur : cet écran ne fait que demander.
// [impl->req~fiche-moi~1]
// [impl->req~liens-types~1]
// [impl->req~reciproques-automatiques~1]
package fr.lemesre.unicontacts.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import fr.lemesre.unicontacts.coeur.ResumeFicheFfi
import fr.lemesre.unicontacts.coeur.TypeRelationFfi
import fr.lemesre.unicontacts.compte.Compte
import kotlinx.coroutines.launch

/** Nombre de fiches proposées à la fois dans le choix de la fiche liée. */
private const val FICHES_PROPOSEES = 8

@Composable
fun SectionRelations(app: UnicontactsApp, fiche: FicheFfi, ouvrir: (Ecran) -> Unit, recharger: suspend () -> Unit) {
    val portee = rememberCoroutineScope()
    var ajout by remember { mutableStateOf(false) }
    var message by remember { mutableStateOf<String?>(null) }

    /** Appel au cœur qui modifie des relations, puis rechargement. */
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
        Row(verticalAlignment = Alignment.CenterVertically) {
            Checkbox(
                checked = fiche.estMoi,
                onCheckedChange = { coche -> modifier { app.coeur.designerMoi(if (coche) fiche.id else null) } },
            )
            Text("C'est moi")
        }
        Text("Relations", style = MaterialTheme.typography.titleMedium)
        fiche.relations.forEach { relation ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    "${relation.libelle} : ${relation.nom}",
                    Modifier.weight(1f).clickable { ouvrir(Ecran.Fiche(relation.ficheId)) },
                )
                TextButton(onClick = { modifier { app.coeur.delier(fiche.id, relation.code, relation.ficheId) } }) {
                    Text("Retirer")
                }
            }
        }
        message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        TextButton(onClick = { ajout = true }) { Text("Ajouter une relation") }
    }

    if (ajout) {
        DialogueRelation(app, fiche.id, onAnnuler = { ajout = false }) { type, inverse, liee ->
            ajout = false
            modifier {
                if (!inverse.isNullOrBlank()) app.coeur.declarerInverse(type, inverse.trim())
                app.coeur.lier(fiche.id, type, liee)
            }
        }
    }
}

/**
 * Choix d'une relation : un type proposé, ou un libellé nouveau avec son
 * inverse facultatif ; puis la fiche liée, cherchée par son nom.
 */
@Composable
private fun DialogueRelation(
    app: UnicontactsApp,
    porteuse: String,
    onAnnuler: () -> Unit,
    onConfirmer: (type: String, inverse: String?, liee: String) -> Unit,
) {
    var types by remember { mutableStateOf(emptyList<TypeRelationFfi>()) }
    var fiches by remember { mutableStateOf(emptyList<ResumeFicheFfi>()) }
    var type by remember { mutableStateOf<String?>(null) }
    var autre by remember { mutableStateOf("") }
    var inverse by remember { mutableStateOf("") }
    var recherche by remember { mutableStateOf("") }
    var liee by remember { mutableStateOf<ResumeFicheFfi?>(null) }
    LaunchedEffect(porteuse) {
        types = horsInterface { app.coeur.typesRelation() }
        fiches = horsInterface { app.coeur.fiches() }.filter { it.id != porteuse }
    }
    val autreChoisi = type == AUTRE
    val typeRetenu = if (autreChoisi) autre.trim().takeIf { it.isNotEmpty() } else type

    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Ajouter une relation") },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                types.forEach { propose ->
                    LigneChoix(
                        propose.libelle + (propose.inverse?.let { " (inverse : $it)" } ?: ""),
                        type == propose.code,
                    ) { type = propose.code }
                }
                LigneChoix("Autre…", autreChoisi) { type = AUTRE }
                if (autreChoisi) {
                    OutlinedTextField(autre, { autre = it }, label = { Text("Libellé") }, singleLine = true)
                    OutlinedTextField(
                        inverse,
                        { inverse = it },
                        label = { Text("Inverse (facultatif)") },
                        singleLine = true,
                    )
                }
                OutlinedTextField(
                    recherche,
                    { recherche = it },
                    label = { Text("Fiche liée") },
                    singleLine = true,
                    modifier = Modifier.padding(top = 12.dp).fillMaxWidth(),
                )
                fiches
                    .filter { recherche.isBlank() || it.nom.contains(recherche.trim(), ignoreCase = true) }
                    .take(FICHES_PROPOSEES)
                    .forEach { proposee ->
                        Text(
                            proposee.nom,
                            fontWeight = if (liee?.id == proposee.id) FontWeight.Bold else FontWeight.Normal,
                            modifier = Modifier.fillMaxWidth().clickable { liee = proposee }.padding(vertical = 8.dp),
                        )
                    }
            }
        },
        confirmButton = {
            TextButton(
                enabled = typeRetenu != null && liee != null,
                onClick = {
                    val choisi = typeRetenu ?: return@TextButton
                    val fiche = liee ?: return@TextButton
                    onConfirmer(choisi, if (autreChoisi) inverse else null, fiche.id)
                },
            ) { Text("Lier") }
        },
        dismissButton = { TextButton(onClick = onAnnuler) { Text("Annuler") } },
    )
}

/** Marqueur du choix « Autre… » : un libellé saisi. */
private const val AUTRE = "\u0000autre"

@Composable
private fun LigneChoix(texte: String, choisi: Boolean, choisir: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = choisir), verticalAlignment = Alignment.CenterVertically) {
        RadioButton(selected = choisi, onClick = choisir)
        Text(texte)
    }
}

// Canaux d'une fiche : un bouton par canal disponible, préférés pro et
// perso marqués, ouvrant l'application concernée ; notes, messageries
// déclarées, suggestions des messageries installées, classements.
// [impl->req~actions-triees-par-preference~2]
// [impl->req~detection-assistee-canaux~1]
// [impl->req~note-par-canal~1]
// [impl->req~classements-pro-perso~2]
package fr.lemesre.unicontacts.ui

import android.content.ActivityNotFoundException
import android.content.Intent
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.core.net.toUri
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.canaux.ActionsCanaux
import fr.lemesre.unicontacts.canaux.DetectionCanaux
import fr.lemesre.unicontacts.canaux.LecteurMessageries
import fr.lemesre.unicontacts.canaux.Suggestion
import fr.lemesre.unicontacts.coeur.CanalFfi
import fr.lemesre.unicontacts.coeur.ChampFfi
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import kotlinx.coroutines.launch

/** Messageries proposées à la déclaration : code et libellé. */
private val MESSAGERIES = listOf("whatsapp" to "WhatsApp", "signal" to "Signal", "telegram" to "Telegram", "matrix" to "Matrix")

/** Canaux filtrables : code et libellé. */
val CANAUX_FILTRABLES = listOf("appel" to "Appel", "sms" to "SMS", "email" to "E-mail") + MESSAGERIES

@Composable
fun SectionCanaux(app: UnicontactsApp, fiche: FicheFfi, recharger: suspend () -> Unit) {
    val contexte = LocalContext.current
    val portee = rememberCoroutineScope()
    var suggestions by remember { mutableStateOf(emptyList<Suggestion>()) }
    val ignorees = remember { mutableStateListOf<Suggestion>() }
    var ajout by remember { mutableStateOf(false) }
    var classement by remember { mutableStateOf(false) }
    var note by remember { mutableStateOf<CanalFfi?>(null) }
    var message by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(fiche) {
        suggestions = horsInterface {
            DetectionCanaux.suggestions(fiche, LecteurMessageries(contexte.contentResolver).declarations())
        }
    }

    fun modifier(appel: () -> Unit) {
        portee.launch {
            message = try {
                horsInterface(appel)
                null
            } catch (erreur: ErreurFfi) {
                erreur.message
            }
            recharger()
        }
    }

    fun ouvrir(canal: CanalFfi) {
        val uri = ActionsCanaux.uri(canal.code, canal.identifiant) ?: return
        val action = when {
            uri.startsWith("tel:") -> Intent.ACTION_DIAL
            uri.startsWith("smsto:") || uri.startsWith("mailto:") -> Intent.ACTION_SENDTO
            else -> Intent.ACTION_VIEW
        }
        try {
            contexte.startActivity(Intent(action, uri.toUri()))
        } catch (_: ActivityNotFoundException) {
            message = "Aucune application pour ${canal.libelle}"
        }
    }

    Column {
        Text("Joindre", style = MaterialTheme.typography.titleMedium)
        fiche.canaux.forEach { canal ->
            val marques = listOfNotNull("★ pro".takeIf { canal.preferePro }, "★ perso".takeIf { canal.preferePerso })
            Row(verticalAlignment = Alignment.CenterVertically) {
                OutlinedButton(onClick = { ouvrir(canal) }, modifier = Modifier.weight(1f)) {
                    Text((marques + "${canal.libelle} ${canal.identifiant}").joinToString("  "))
                }
                TextButton(onClick = { note = canal }) { Text("Note") }
                if (canal.declare) {
                    TextButton(onClick = { modifier { app.coeur.retirerCanal(fiche.id, canal.cle) } }) { Text("Retirer") }
                }
            }
            canal.note?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
        }
        suggestions.filter { it !in ignorees }.forEach { suggestion ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("${suggestion.libelle} détecté pour ${suggestion.numero}", Modifier.weight(1f))
                TextButton(onClick = {
                    modifier { app.coeur.declarerCanal(fiche.id, suggestion.code, suggestion.numero) }
                }) { Text("Ajouter") }
                TextButton(onClick = { ignorees += suggestion }) { Text("Ignorer") }
            }
        }
        message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        Row {
            TextButton(onClick = { ajout = true }) { Text("Ajouter une messagerie") }
            TextButton(onClick = { classement = true }, enabled = fiche.canaux.isNotEmpty()) { Text("Préférences") }
        }
    }

    if (ajout) {
        val numeros = fiche.champs.filter { it.champ == ChampFfi.TELEPHONE }.flatMap { it.valeurs }.map { it.texte }
        DialogueMessagerie(numeros, onAnnuler = { ajout = false }) { code, identifiant ->
            ajout = false
            modifier { app.coeur.declarerCanal(fiche.id, code, identifiant) }
        }
    }
    note?.let { canal ->
        DialogueNote(canal, onAnnuler = { note = null }) { texte ->
            note = null
            modifier { app.coeur.annoterCanal(fiche.id, canal.cle, texte) }
        }
    }
    if (classement) {
        DialogueClassements(fiche, onAnnuler = { classement = false }) { pro, perso ->
            classement = false
            modifier {
                app.coeur.classerCanaux(fiche.id, true, pro)
                app.coeur.classerCanaux(fiche.id, false, perso)
            }
        }
    }
}

/** Déclaration d'une messagerie : type, puis numéro de la fiche ou pseudonyme. */
@Composable
private fun DialogueMessagerie(numeros: List<String>, onAnnuler: () -> Unit, onConfirmer: (String, String) -> Unit) {
    var code by remember { mutableStateOf("signal") }
    var autre by remember { mutableStateOf("") }
    var identifiant by remember { mutableStateOf(numeros.firstOrNull().orEmpty()) }
    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Ajouter une messagerie") },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                (MESSAGERIES + ("" to "Autre…")).forEach { (valeur, nom) ->
                    Row(Modifier.fillMaxWidth().clickable { code = valeur }, verticalAlignment = Alignment.CenterVertically) {
                        RadioButton(selected = code == valeur, onClick = { code = valeur })
                        Text(nom)
                    }
                }
                if (code.isEmpty()) {
                    OutlinedTextField(autre, { autre = it }, label = { Text("Nom de la messagerie") }, singleLine = true)
                }
                Text("Numéro de la fiche, ou pseudonyme", style = MaterialTheme.typography.titleSmall)
                numeros.forEach { numero ->
                    Row(Modifier.fillMaxWidth().clickable { identifiant = numero }, verticalAlignment = Alignment.CenterVertically) {
                        RadioButton(selected = identifiant == numero, onClick = { identifiant = numero })
                        Text(numero)
                    }
                }
                OutlinedTextField(identifiant, { identifiant = it }, label = { Text("Identifiant (@pseudo…)") }, singleLine = true)
            }
        },
        confirmButton = {
            val type = if (code.isEmpty()) autre.trim() else code
            TextButton(enabled = type.isNotEmpty() && identifiant.isNotBlank(), onClick = { onConfirmer(type, identifiant.trim()) }) {
                Text("Ajouter")
            }
        },
        dismissButton = { TextButton(onClick = onAnnuler) { Text("Annuler") } },
    )
}

@Composable
private fun DialogueNote(canal: CanalFfi, onAnnuler: () -> Unit, onConfirmer: (String) -> Unit) {
    var texte by remember { mutableStateOf(canal.note.orEmpty()) }
    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Note — ${canal.libelle} ${canal.identifiant}") },
        text = { OutlinedTextField(texte, { texte = it }, label = { Text("Note (vide : aucune)") }) },
        confirmButton = { TextButton(onClick = { onConfirmer(texte) }) { Text("Enregistrer") } },
        dismissButton = { TextButton(onClick = onAnnuler) { Text("Annuler") } },
    )
}

/**
 * Classements pro et perso : on touche les canaux dans l'ordre de
 * préférence ; un classement vide laisse l'autre valoir pour les deux.
 */
@Composable
private fun DialogueClassements(fiche: FicheFfi, onAnnuler: () -> Unit, onConfirmer: (List<String>, List<String>) -> Unit) {
    val pro = remember { mutableStateListOf<String>().apply { addAll(fiche.classementPro) } }
    val perso = remember { mutableStateListOf<String>().apply { addAll(fiche.classementPerso) } }
    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Préférences") },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                listOf("Pro" to pro, "Perso" to perso).forEach { (titre, ordre) ->
                    Text("$titre — toucher dans l'ordre de préférence", style = MaterialTheme.typography.titleSmall)
                    fiche.canaux.forEach { canal ->
                        val rang = ordre.indexOf(canal.cle)
                        Text(
                            (if (rang >= 0) "${rang + 1}. " else "—  ") + "${canal.libelle} ${canal.identifiant}",
                            modifier = Modifier.fillMaxWidth().clickable {
                                if (rang >= 0) ordre.remove(canal.cle) else ordre.add(canal.cle)
                            },
                        )
                    }
                }
            }
        },
        confirmButton = { TextButton(onClick = { onConfirmer(pro.toList(), perso.toList()) }) { Text("Enregistrer") } },
        dismissButton = { TextButton(onClick = onAnnuler) { Text("Annuler") } },
    )
}

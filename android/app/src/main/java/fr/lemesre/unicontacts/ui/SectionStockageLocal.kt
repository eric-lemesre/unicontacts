// Stockage local : import des contacts du téléphone à la demande, puis
// retrait des originaux importés après confirmation explicite (ADR-0015) ;
// carte SIM ajoutée comme source en lecture seule (ADR-0016).
// [impl->req~import-stockage-local~1]
// [impl->req~source-sim-lecture-seule~1]
package fr.lemesre.unicontacts.ui

import android.Manifest
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.coeur.ImportFfi
import fr.lemesre.unicontacts.compte.Compte
import fr.lemesre.unicontacts.stockage.StockageLocal
import kotlinx.coroutines.launch

/** Compte rendu lisible d'un import. */
fun compteRendu(bilan: ImportFfi): String = buildList {
    add("${bilan.creees} fiche(s) créée(s)")
    add("${bilan.rattachees} rattachée(s) à une fiche existante")
    if (bilan.decisions > 0u) add("${bilan.decisions} décision(s) à trancher")
    if (bilan.illisibles > 0u) add("${bilan.illisibles} élément(s) illisible(s) laissé(s) de côté")
}.joinToString(", ")

@Composable
fun SectionStockageLocal(app: UnicontactsApp, recharger: suspend () -> Unit) {
    val contexte = LocalContext.current
    val stockage = remember { StockageLocal(contexte.applicationContext) }
    val portee = rememberCoroutineScope()
    var message by remember { mutableStateOf<String?>(null) }
    var aRetirer by remember { mutableStateOf<List<String>?>(null) }
    var action by remember { mutableStateOf<(() -> Unit)?>(null) }

    /** Importe tout le stockage local et rend le bilan. */
    fun importer(): ImportFfi {
        val source = app.configurations.sourceStockageLocal
            ?: app.coeur.ajouterSourceImport("Stockage local").also { app.configurations.sourceStockageLocal = it }
        return app.coeur.importer(source, stockage.lire())
    }

    fun lancer(travail: suspend () -> String) {
        portee.launch {
            message = try {
                travail()
            } catch (erreur: ErreurFfi) {
                erreur.message
            } catch (erreur: SecurityException) {
                "Accès aux contacts non accordé (${erreur.javaClass.simpleName})"
            }
            recharger()
        }
    }

    val permissions = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { accordees ->
        if (accordees.values.all { it }) action?.invoke() else message = "Accès aux contacts non accordé"
        action = null
    }
    fun avecPermissions(suite: () -> Unit) {
        action = suite
        permissions.launch(arrayOf(Manifest.permission.READ_CONTACTS, Manifest.permission.WRITE_CONTACTS))
    }

    Column {
        Text("Contacts du téléphone", style = MaterialTheme.typography.titleMedium)
        Text(
            "L'import verse au référentiel les contacts rangés hors de tout compte, sans y toucher ; " +
                "les originaux ne se retirent qu'à votre demande.",
            style = MaterialTheme.typography.bodySmall,
        )
        Row {
            TextButton(onClick = {
                avecPermissions {
                    lancer {
                        val bilan = horsInterface { importer() }
                        Compte.synchroniser()
                        "Import : ${compteRendu(bilan)}."
                    }
                }
            }) { Text("Importer les contacts du téléphone") }
        }
        Row {
            TextButton(onClick = {
                avecPermissions {
                    lancer {
                        // Un import d'abord : la dernière modification locale
                        // rejoint sa fiche avant que l'original parte.
                        horsInterface { importer() }
                        val source = app.configurations.sourceStockageLocal
                        val uids = source?.let { horsInterface { app.coeur.copiesLiees(it) } }.orEmpty()
                        if (uids.isEmpty()) return@lancer "Aucun original importé à retirer."
                        aRetirer = uids
                        "${uids.size} original(aux) importé(s) dans le stockage local."
                    }
                }
            }) { Text("Retirer les originaux importés") }
        }
        message?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
    }

    aRetirer?.let { uids ->
        AlertDialog(
            onDismissRequest = { aRetirer = null },
            title = { Text("Retirer ${uids.size} contact(s) du téléphone ?") },
            text = {
                Text(
                    "Ces contacts quittent le stockage local du téléphone. Leurs fiches restent dans unicontacts, " +
                        "qui les publie dans les contacts sous son propre compte.",
                )
            },
            confirmButton = {
                TextButton(onClick = {
                    aRetirer = null
                    lancer {
                        val retires = horsInterface { stockage.retirer(uids) }
                        // Un dernier import détache les liens devenus sans objet.
                        horsInterface { importer() }
                        Compte.synchroniser()
                        "$retires contact(s) retiré(s) du stockage local."
                    }
                }) { Text("Retirer") }
            },
            dismissButton = { TextButton(onClick = { aRetirer = null }) { Text("Annuler") } },
        )
    }
}

@Composable
fun SectionCarteSim(app: UnicontactsApp, recharger: suspend () -> Unit) {
    val portee = rememberCoroutineScope()
    var configuree by remember { mutableStateOf(app.configurations.sourceSim != null) }
    var message by remember { mutableStateOf<String?>(null) }
    val permission = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { accordee ->
        if (!accordee) {
            message = "Accès aux contacts non accordé"
            return@rememberLauncherForActivityResult
        }
        portee.launch {
            message = try {
                val id = horsInterface { app.coeur.ajouterSourceImport("Carte SIM") }
                app.configurations.sourceSim = id
                configuree = true
                Compte.synchroniser()
                null
            } catch (erreur: ErreurFfi) {
                erreur.message
            }
            recharger()
        }
    }
    Column {
        Text("Carte SIM", style = MaterialTheme.typography.titleMedium)
        if (configuree) {
            Text(
                "Relue à chaque synchronisation : nom et numéro de chaque entrée. Rien n'y est jamais écrit.",
                style = MaterialTheme.typography.bodySmall,
            )
        } else {
            TextButton(onClick = { permission.launch(Manifest.permission.READ_CONTACTS) }) {
                Text("Ajouter la carte SIM (lecture seule)")
            }
        }
        message?.let { Text(it, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall) }
    }
}

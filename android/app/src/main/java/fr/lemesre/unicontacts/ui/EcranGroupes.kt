// Groupes : gestion (création, renommage, déplacement, suppression), choix
// des groupes d'une fiche, et filtre de la liste. Un groupe se désigne par
// son chemin (ADR-0012) ; toute règle est celle du cœur.
// [impl->req~appartenance-multiple~1]
// [impl->req~groupes-hierarchiques~1]
// [impl->req~filtre-par-groupes~2]
// [impl->req~renommage-groupe-operation-unique~1]
// [impl->req~filtre-par-canal~1]
package fr.lemesre.unicontacts.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
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
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.coeur.FicheFfi
import fr.lemesre.unicontacts.coeur.FiltreFfi
import fr.lemesre.unicontacts.compte.Compte
import kotlinx.coroutines.launch

/** Filtre vide : toutes les fiches. */
val FILTRE_VIDE = FiltreFfi(groupes = emptyList(), intersection = false, sousGroupes = true, recherche = "", canal = null)

/** Vrai si le filtre restreint la liste. */
fun FiltreFfi.estActif(): Boolean = groupes.isNotEmpty() || recherche.isNotBlank() || canal != null

/** Retrait d'un groupe selon sa profondeur, pour montrer la hiérarchie. */
private fun retrait(chemin: String): Int = chemin.count { it == '/' } * 16

/** Nom propre d'un groupe : le dernier nom de son chemin. */
private fun nomPropre(chemin: String): String = chemin.substringAfterLast('/')

/** Gestion des groupes. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EcranGroupes(app: UnicontactsApp, retour: () -> Unit) {
    var groupes by remember { mutableStateOf(emptyList<String>()) }
    var nouveau by remember { mutableStateOf("") }
    var message by remember { mutableStateOf<String?>(null) }
    var edition by remember { mutableStateOf<String?>(null) }
    val portee = rememberCoroutineScope()

    suspend fun charger() {
        groupes = horsInterface { app.coeur.groupes() }
    }
    fun modifier(appel: () -> Unit) {
        portee.launch {
            message = try {
                horsInterface(appel)
                null
            } catch (erreur: ErreurFfi) {
                erreur.message
            }
            charger()
            if (message == null) Compte.synchroniser()
        }
    }
    LaunchedEffect(Unit) { charger() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Groupes") },
                navigationIcon = { TextButton(onClick = retour) { Text("Retour") } },
            )
        },
    ) { marges ->
        Column(
            Modifier.padding(marges).padding(16.dp).verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            groupes.forEach { chemin ->
                Text(
                    nomPropre(chemin),
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.fillMaxWidth().clickable { edition = chemin }.padding(start = retrait(chemin).dp),
                )
            }
            if (groupes.isEmpty()) Text("Aucun groupe.")
            HorizontalDivider()
            OutlinedTextField(
                nouveau,
                { nouveau = it },
                label = { Text("Nouveau groupe (Travail/Clients…)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
            Button(onClick = {
                val chemin = nouveau
                nouveau = ""
                modifier { app.coeur.creerGroupe(chemin) }
            }, enabled = nouveau.isNotBlank()) { Text("Créer") }
        }
    }

    edition?.let { chemin ->
        DialogueGroupe(
            chemin,
            groupes.filter { it != chemin && !it.startsWith("$chemin/") },
            onFermer = { edition = null },
            onRenommer = { nom ->
                edition = null
                val parent = chemin.substringBeforeLast('/', "")
                modifier { app.coeur.renommerGroupe(chemin, if (parent.isEmpty()) nom else "$parent/$nom") }
            },
            onDeplacer = { parent ->
                edition = null
                modifier { app.coeur.deplacerGroupe(chemin, parent) }
            },
            onSupprimer = {
                edition = null
                modifier { app.coeur.supprimerGroupe(chemin) }
            },
        )
    }
}

/** Renommage, choix du parent ou suppression d'un groupe. */
@Composable
private fun DialogueGroupe(
    chemin: String,
    parentsPossibles: List<String>,
    onFermer: () -> Unit,
    onRenommer: (String) -> Unit,
    onDeplacer: (String?) -> Unit,
    onSupprimer: () -> Unit,
) {
    var nom by remember { mutableStateOf(nomPropre(chemin)) }
    val parentActuel = chemin.substringBeforeLast('/', "").ifEmpty { null }
    var parent by remember { mutableStateOf(parentActuel) }
    AlertDialog(
        onDismissRequest = onFermer,
        title = { Text(chemin) },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                OutlinedTextField(nom, { nom = it }, label = { Text("Nom") }, singleLine = true)
                TextButton(onClick = { onRenommer(nom.trim()) }, enabled = nom.isNotBlank()) { Text("Renommer") }
                Text("Parent", style = MaterialTheme.typography.titleSmall)
                LigneParent("(aucun)", parent == null) { parent = null }
                parentsPossibles.forEach { possible -> LigneParent(possible, parent == possible) { parent = possible } }
                TextButton(onClick = { onDeplacer(parent) }, enabled = parent != parentActuel) { Text("Déplacer") }
                TextButton(onClick = onSupprimer) {
                    Text("Supprimer le groupe (les fiches restent)", color = MaterialTheme.colorScheme.error)
                }
            }
        },
        confirmButton = { TextButton(onClick = onFermer) { Text("Fermer") } },
    )
}

@Composable
private fun LigneParent(texte: String, choisi: Boolean, choisir: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = choisir), verticalAlignment = Alignment.CenterVertically) {
        RadioButton(selected = choisi, onClick = choisir)
        Text(texte)
    }
}

/** Groupes d'une fiche, sur son écran. */
@Composable
fun SectionGroupes(app: UnicontactsApp, fiche: FicheFfi, recharger: suspend () -> Unit) {
    val portee = rememberCoroutineScope()
    var choix by remember { mutableStateOf(false) }
    Column {
        Text("Groupes", style = MaterialTheme.typography.titleMedium)
        Text(fiche.groupes.joinToString(", ").ifEmpty { "Aucun" })
        TextButton(onClick = { choix = true }) { Text("Choisir les groupes") }
    }
    if (choix) {
        DialogueChoixGroupes(app, fiche.groupes, onAnnuler = { choix = false }) { retenus ->
            choix = false
            portee.launch {
                horsInterface { app.coeur.ranger(fiche.id, retenus) }
                recharger()
                Compte.synchroniser()
            }
        }
    }
}

@Composable
private fun DialogueChoixGroupes(
    app: UnicontactsApp,
    actuels: List<String>,
    onAnnuler: () -> Unit,
    onConfirmer: (List<String>) -> Unit,
) {
    var groupes by remember { mutableStateOf(emptyList<String>()) }
    val retenus = remember { mutableStateListOf<String>().apply { addAll(actuels) } }
    var nouveau by remember { mutableStateOf("") }
    LaunchedEffect(Unit) { groupes = horsInterface { app.coeur.groupes() } }
    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Groupes de la fiche") },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                (groupes + retenus.filter { it !in groupes }).forEach { chemin ->
                    LigneCase(chemin, chemin in retenus) { coche ->
                        if (coche) retenus += chemin else retenus -= chemin
                    }
                }
                OutlinedTextField(
                    nouveau,
                    { nouveau = it },
                    label = { Text("Autre groupe (Travail/Clients…)") },
                    singleLine = true,
                )
                TextButton(onClick = {
                    if (nouveau.isNotBlank()) retenus += nouveau.trim()
                    nouveau = ""
                }) { Text("Ajouter") }
            }
        },
        confirmButton = { TextButton(onClick = { onConfirmer(retenus.toList()) }) { Text("Enregistrer") } },
        dismissButton = { TextButton(onClick = onAnnuler) { Text("Annuler") } },
    )
}

/** Choix des groupes du filtre de la liste, et de leur combinaison. */
@Composable
fun DialogueFiltre(app: UnicontactsApp, filtre: FiltreFfi, onAnnuler: () -> Unit, onAppliquer: (FiltreFfi) -> Unit) {
    var groupes by remember { mutableStateOf(emptyList<String>()) }
    val retenus = remember { mutableStateListOf<String>().apply { addAll(filtre.groupes) } }
    var intersection by remember { mutableStateOf(filtre.intersection) }
    var sousGroupes by remember { mutableStateOf(filtre.sousGroupes) }
    var canal by remember { mutableStateOf(filtre.canal) }
    LaunchedEffect(Unit) { groupes = horsInterface { app.coeur.groupes() } }
    AlertDialog(
        onDismissRequest = onAnnuler,
        title = { Text("Filtrer") },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                groupes.forEach { chemin ->
                    LigneCase(chemin, chemin in retenus) { coche ->
                        if (coche) retenus += chemin else retenus -= chemin
                    }
                }
                if (groupes.isEmpty()) Text("Aucun groupe.")
                HorizontalDivider()
                LigneParent("L'un des groupes", !intersection) { intersection = false }
                LigneParent("Tous les groupes à la fois", intersection) { intersection = true }
                LigneCase("Sous-groupes compris", sousGroupes) { sousGroupes = it }
                HorizontalDivider()
                Text("Canal disponible", style = MaterialTheme.typography.titleSmall)
                LigneParent("Tous", canal == null) { canal = null }
                CANAUX_FILTRABLES.forEach { (code, nom) -> LigneParent(nom, canal == code) { canal = code } }
            }
        },
        confirmButton = {
            TextButton(onClick = {
                onAppliquer(
                    filtre.copy(groupes = retenus.toList(), intersection = intersection, sousGroupes = sousGroupes, canal = canal),
                )
            }) { Text("Appliquer") }
        },
        dismissButton = {
            TextButton(onClick = { onAppliquer(filtre.copy(groupes = emptyList(), canal = null)) }) { Text("Aucun filtre") }
        },
    )
}

@Composable
private fun LigneCase(texte: String, coche: Boolean, changer: (Boolean) -> Unit) {
    Row(Modifier.fillMaxWidth().clickable { changer(!coche) }, verticalAlignment = Alignment.CenterVertically) {
        Checkbox(checked = coche, onCheckedChange = changer)
        Text(texte)
    }
}

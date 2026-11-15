// Sources : ajout d'un carnet CardDAV (URL, identifiant, mot de passe
// d'application chiffré au coffre), synchronisation à la demande, et import
// des contacts du téléphone, carte SIM en lecture seule.
// [impl->req~synchronisation-directe~1]
// [impl->req~source-sim-lecture-seule~1]
package fr.lemesre.unicontacts.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.ui.unit.dp
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.SourceFfi
import fr.lemesre.unicontacts.compte.Compte
import fr.lemesre.unicontacts.sources.ConfigurationsSources
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EcranSources(app: UnicontactsApp, retour: () -> Unit) {
    var sources by remember { mutableStateOf(emptyList<SourceFfi>()) }
    var nom by remember { mutableStateOf("") }
    var carnet by remember { mutableStateOf("") }
    var utilisateur by remember { mutableStateOf("") }
    var motDePasse by remember { mutableStateOf("") }
    var vcard4 by remember { mutableStateOf(false) }
    var message by remember { mutableStateOf<String?>(null) }
    val portee = rememberCoroutineScope()

    suspend fun charger() {
        sources = horsInterface { app.coeur.sources() }
    }
    LaunchedEffect(Unit) { charger() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Sources") },
                navigationIcon = { TextButton(onClick = retour) { Text("Retour") } },
            )
        },
    ) { marges ->
        Column(
            Modifier.padding(marges).padding(16.dp).verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            sources.forEach { source ->
                Column {
                    Text(source.nom, style = MaterialTheme.typography.titleMedium)
                    Text(app.configurations.carnet(source.id) ?: "(sans paramètres de connexion)", style = MaterialTheme.typography.bodySmall)
                    app.echecs[source.nom]?.let {
                        Text("Dernier échec : $it", color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
                    }
                }
                HorizontalDivider()
            }
            Button(onClick = { Compte.synchroniser() }, enabled = sources.isNotEmpty()) { Text("Synchroniser maintenant") }
            SectionAgenda()
            HorizontalDivider()
            SectionStockageLocal(app) { charger() }
            HorizontalDivider()
            SectionCarteSim(app) { charger() }
            HorizontalDivider()

            Text("Ajouter un carnet CardDAV", style = MaterialTheme.typography.titleMedium)
            OutlinedTextField(nom, { nom = it }, label = { Text("Nom (Murena, Nextcloud…)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            OutlinedTextField(
                carnet,
                { carnet = it },
                label = { Text("Adresse du carnet (https://…)") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri),
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(utilisateur, { utilisateur = it }, label = { Text("Identifiant") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            OutlinedTextField(
                motDePasse,
                { motDePasse = it },
                label = { Text("Mot de passe d'application") },
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                modifier = Modifier.fillMaxWidth(),
            )
            Row {
                Checkbox(checked = vcard4, onCheckedChange = { vcard4 = it })
                Text("Le serveur sert du vCard 4.0", Modifier.padding(top = 12.dp))
            }
            message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
            Button(onClick = {
                val refus = ConfigurationsSources.refusCarnet(carnet)
                    ?: if (nom.isBlank() || utilisateur.isBlank() || motDePasse.isEmpty()) "Tous les champs sont requis" else null
                if (refus != null) {
                    message = refus
                    return@Button
                }
                portee.launch {
                    val id = horsInterface { app.coeur.ajouterSource(nom.trim(), vcard4) }
                    horsInterface { app.configurations.enregistrer(id, carnet.trim(), utilisateur.trim(), motDePasse) }
                    nom = ""; carnet = ""; utilisateur = ""; motDePasse = ""; message = null
                    charger()
                    Compte.synchroniser()
                }
            }) { Text("Ajouter et synchroniser") }
        }
    }
}

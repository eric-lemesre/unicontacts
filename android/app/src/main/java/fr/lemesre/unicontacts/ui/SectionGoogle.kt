// Ajout ou réautorisation d'un compte Google (ADR-0017) : adresse et
// identifiant client OAuth de l'utilisateur, puis page d'autorisation de
// Google ouverte dans le navigateur ; le retour arrive dans RetourGoogle.
// [impl->req~source-google~1]
package fr.lemesre.unicontacts.ui

import android.content.ActivityNotFoundException
import android.content.Intent
import androidx.core.net.toUri
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardType
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ErreurFfi
import kotlinx.coroutines.launch
import java.security.GeneralSecurityException

@Composable
fun SectionGoogle(app: UnicontactsApp) {
    val contexte = LocalContext.current
    val portee = rememberCoroutineScope()
    var adresse by remember { mutableStateOf("") }
    var client by remember { mutableStateOf(app.configurations.clientGoogle.orEmpty()) }
    var message by remember { mutableStateOf<String?>(null) }

    Column {
        Text("Ajouter ou reconnecter un compte Google", style = MaterialTheme.typography.titleMedium)
        Text(
            "L'identifiant client est celui du client OAuth « Android » que vous avez créé dans la console " +
                "Google Cloud. Google affichera « application non vérifiée » : c'est la vôtre.",
            style = MaterialTheme.typography.bodySmall,
        )
        OutlinedTextField(
            adresse,
            { adresse = it },
            label = { Text("Adresse du compte (…@gmail.com)") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            client,
            { client = it },
            label = { Text("Identifiant client OAuth (….apps.googleusercontent.com)") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        message?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        Button(
            enabled = adresse.isNotBlank() && client.isNotBlank(),
            onClick = {
                portee.launch {
                    message = try {
                        val demande = horsInterface {
                            val demande = app.coeur.googleDemande(client.trim(), adresse.trim())
                            app.configurations.clientGoogle = client.trim()
                            app.configurations.attendreGoogle(demande.etat, adresse.trim(), client.trim(), demande.verificateur)
                            demande
                        }
                        contexte.startActivity(Intent(Intent.ACTION_VIEW, demande.url.toUri()))
                        null
                    } catch (erreur: ErreurFfi) {
                        erreur.message
                    } catch (erreur: GeneralSecurityException) {
                        "Coffre indisponible (${erreur.javaClass.simpleName})"
                    } catch (erreur: ActivityNotFoundException) {
                        "Aucun navigateur pour ouvrir la page d'autorisation"
                    }
                }
            },
        ) { Text("Autoriser dans le navigateur") }
    }
}

// Retour de l'autorisation Google (ADR-0017) : le navigateur rend la main
// par `fr.lemesre.unicontacts:/oauth2redirect?code=…&state=…`. L'état doit
// être celui de la demande en attente ; le code s'échange alors contre le
// jeton de renouvellement, gardé chiffré, et la source Google est créée ou
// réautorisée.
// [impl->req~source-google~1]
package fr.lemesre.unicontacts.sources

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import fr.lemesre.unicontacts.UnicontactsApp
import fr.lemesre.unicontacts.coeur.ErreurFfi
import fr.lemesre.unicontacts.compte.Compte
import fr.lemesre.unicontacts.ui.ActivitePrincipale
import fr.lemesre.unicontacts.ui.horsInterface
import java.security.GeneralSecurityException

class RetourGoogle : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val app = application as UnicontactsApp
        val donnees = intent?.data
        val code = donnees?.getQueryParameter("code")
        val refus = donnees?.getQueryParameter("error")
        // L'état se consomme une seule fois, qu'il corresponde ou non.
        val attente = app.configurations.reprendreGoogle(donnees?.getQueryParameter("state"))
        setContent {
            MaterialTheme {
                var message by remember { mutableStateOf("Autorisation en cours…") }
                LaunchedEffect(Unit) {
                    message = when {
                        refus != null -> "Autorisation refusée par Google ($refus)."
                        attente == null || code == null -> "Retour d'autorisation inattendu : rien n'a été enregistré."
                        else -> try {
                            horsInterface { enregistrer(app, attente, code) }
                        } catch (erreur: ErreurFfi) {
                            erreur.message.orEmpty()
                        } catch (erreur: GeneralSecurityException) {
                            "Coffre indisponible (${erreur.javaClass.simpleName})"
                        }
                    }
                }
                Column(Modifier.padding(24.dp)) {
                    Text(message)
                    Button(onClick = {
                        startActivity(Intent(this@RetourGoogle, ActivitePrincipale::class.java))
                        finish()
                    }) { Text("Retour à unicontacts") }
                }
            }
        }
    }

    /** Échange le code, garde le jeton, crée ou réautorise la source. */
    private fun enregistrer(app: UnicontactsApp, attente: AttenteGoogle, code: String): String {
        val renouvellement = app.coeur.googleEchanger(attente.client, code, attente.verificateur())
        val sources = app.coeur.sources()
        val existante = app.configurations.sourceGoogle(sources, attente.adresse)
        val id = existante ?: app.coeur.ajouterSource("Google (${attente.adresse})", false)
        app.configurations.enregistrerGoogle(id, attente.adresse, attente.client, renouvellement)
        app.echecs.remove("Google (${attente.adresse})")
        Compte.synchroniser()
        return if (existante == null) "Compte Google ajouté ; synchronisation lancée." else "Compte Google réautorisé ; synchronisation lancée."
    }
}

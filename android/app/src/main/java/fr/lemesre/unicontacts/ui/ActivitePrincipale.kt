// Activité unique : écrans de la liste, d'une fiche, des sources et des
// décisions. Les appels au cœur se font hors du fil de l'interface.
// [impl->req~boite-de-decisions~1]
package fr.lemesre.unicontacts.ui

import android.Manifest
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import fr.lemesre.unicontacts.UnicontactsApp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** Écran affiché. */
sealed interface Ecran {
    data object Liste : Ecran
    data class Fiche(val id: String) : Ecran
    data object Sources : Ecran
    data object Groupes : Ecran
    data object AVenir : Ecran
    data object Decisions : Ecran
    data class Decision(val id: String) : Ecran
}

/** Exécute un appel au cœur hors du fil de l'interface. */
suspend fun <T> horsInterface(appel: () -> T): T = withContext(Dispatchers.IO) { appel() }

class ActivitePrincipale : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val app = application as UnicontactsApp
        setContent {
            MaterialTheme {
                DemandePermissions()
                Navigation(app)
            }
        }
    }
}

@Composable
private fun DemandePermissions() {
    val lanceur = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {}
    LaunchedEffect(Unit) {
        lanceur.launch(arrayOf(Manifest.permission.READ_CONTACTS, Manifest.permission.WRITE_CONTACTS))
    }
}

@Composable
private fun Navigation(app: UnicontactsApp) {
    var pile by remember { mutableStateOf(listOf<Ecran>(Ecran.Liste)) }
    val ouvrir: (Ecran) -> Unit = { pile = pile + it }
    val retour: () -> Unit = { if (pile.size > 1) pile = pile.dropLast(1) }
    BackHandler(enabled = pile.size > 1, onBack = retour)
    when (val ecran = pile.last()) {
        Ecran.Liste -> EcranListe(app, ouvrir)
        is Ecran.Fiche -> EcranFiche(app, ecran.id, ouvrir, retour)
        Ecran.Sources -> EcranSources(app, retour)
        Ecran.Groupes -> EcranGroupes(app, retour)
        Ecran.AVenir -> EcranAVenir(app, ouvrir, retour)
        Ecran.Decisions -> EcranDecisions(app, ouvrir, retour)
        is Ecran.Decision -> EcranDecision(app, ecran.id, retour)
    }
}

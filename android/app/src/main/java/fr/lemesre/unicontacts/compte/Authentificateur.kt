// Authentificateur du compte système : le compte d'unicontacts ne porte
// aucun secret (les identifiants des sources sont au coffre, ADR-0010) ; il
// se crée depuis l'application, jamais depuis les réglages du téléphone.
// [impl->req~compte-systeme-dedie~1]
package fr.lemesre.unicontacts.compte

import android.accounts.AbstractAccountAuthenticator
import android.accounts.Account
import android.accounts.AccountAuthenticatorResponse
import android.accounts.AccountManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.IBinder
import fr.lemesre.unicontacts.ui.ActivitePrincipale

class Authentificateur(private val context: Context) : AbstractAccountAuthenticator(context) {
    override fun addAccount(
        response: AccountAuthenticatorResponse?,
        accountType: String?,
        authTokenType: String?,
        requiredFeatures: Array<out String>?,
        options: Bundle?,
    ): Bundle = Bundle().apply {
        putParcelable(AccountManager.KEY_INTENT, Intent(context, ActivitePrincipale::class.java))
    }

    override fun editProperties(response: AccountAuthenticatorResponse?, accountType: String?): Bundle? = null

    override fun confirmCredentials(response: AccountAuthenticatorResponse?, account: Account?, options: Bundle?): Bundle? = null

    override fun getAuthToken(
        response: AccountAuthenticatorResponse?,
        account: Account?,
        authTokenType: String?,
        options: Bundle?,
    ): Bundle? = null

    override fun getAuthTokenLabel(authTokenType: String?): String? = null

    override fun updateCredentials(
        response: AccountAuthenticatorResponse?,
        account: Account?,
        authTokenType: String?,
        options: Bundle?,
    ): Bundle? = null

    override fun hasFeatures(
        response: AccountAuthenticatorResponse?,
        account: Account?,
        features: Array<out String>?,
    ): Bundle = Bundle().apply { putBoolean(AccountManager.KEY_BOOLEAN_RESULT, false) }
}

class ServiceAuthentificateur : Service() {
    private lateinit var authentificateur: Authentificateur

    override fun onCreate() {
        super.onCreate()
        authentificateur = Authentificateur(this)
    }

    override fun onBind(intent: Intent?): IBinder? = authentificateur.iBinder
}

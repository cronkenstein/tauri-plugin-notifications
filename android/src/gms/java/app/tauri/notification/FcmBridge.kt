package app.tauri.notification

import android.content.Context
import com.google.firebase.messaging.FirebaseMessaging

object FcmBridge {
  const val AUTOMATIC = true

  fun isAvailable(context: Context): Boolean =
    runCatching { com.google.android.gms.common.GoogleApiAvailabilityLight.getInstance()
      .isGooglePlayServicesAvailable(context) == com.google.android.gms.common.ConnectionResult.SUCCESS
    }.getOrDefault(false)

  /**
   * Fetches the FCM registration token. [onError] is invoked when the Firebase
   * runtime is not initialised (e.g. unit tests, or a missing
   * `google-services.json` at the app level).
   */
  fun getToken(onToken: (String) -> Unit, onError: (String) -> Unit) {
    try {
      FirebaseMessaging.getInstance().token.addOnCompleteListener { task ->
        if (task.isSuccessful) {
          onToken(task.result ?: "")
        } else {
          onError(task.exception?.message ?: "Failed to get FCM token")
        }
      }
    } catch (error: Exception) {
      onError("Failed to get FCM token: ${error.message}")
    }
  }

  /**
   * Deletes the FCM registration token. [onDone] receives `false` when the
   * Firebase runtime is not initialised; callers should treat that as
   * "nothing to delete".
   */
  fun deleteToken(onDone: (Boolean) -> Unit) {
    try {
      FirebaseMessaging.getInstance().deleteToken().addOnCompleteListener { task ->
        onDone(task.isSuccessful)
      }
    } catch (error: Exception) {
      onDone(false)
    }
  }
}

package app.tauri.notification

import android.content.Context

object FcmBridge {
  const val AUTOMATIC = false

  fun isAvailable(context: Context): Boolean =
    runCatching { context.packageManager.getApplicationInfo("com.google.android.gms", 0).enabled }
      .getOrDefault(false)

  // FOSS builds carry no Google libraries, so FCM registration is unavailable.
  fun getToken(onToken: (String) -> Unit, onError: (String) -> Unit) {
    onError("FCM is not available in this build")
  }

  fun deleteToken(onDone: (Boolean) -> Unit) {
    onDone(true)
  }
}

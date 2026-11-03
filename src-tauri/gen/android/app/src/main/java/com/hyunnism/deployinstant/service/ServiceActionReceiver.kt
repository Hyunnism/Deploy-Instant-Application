package com.hyunnism.deployinstant.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

class ServiceActionReceiver : BroadcastReceiver() {

    companion object {
        const val ACTION_STOP_ALL = "com.hyunnism.deployinstant.ACTION_STOP_ALL"
        private const val TAG = "DeployInstantReceiver"

        @JvmStatic
        external fun nativeStopAllFromService()
    }

    override fun onReceive(context: Context, intent: Intent?) {
        Log.i(TAG, "onReceive dipanggil: action=${intent?.action}")
        if (intent?.action == ACTION_STOP_ALL) {
            // Hentikan foreground service segera
            DeployInstantService.stop(context)

            // Panggil Rust backend via JNI untuk menghentikan PM2 & tunnel
            try {
                nativeStopAllFromService()
                Log.i(TAG, "nativeStopAllFromService sukses dipanggil")
            } catch (e: Throwable) {
                Log.e(TAG, "Gagal memanggil nativeStopAllFromService", e)
            }
        }
    }
}


package com.hyunnism.deployinstant.service

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationCompat
import com.hyunnism.deployinstant.MainActivity

class DeployInstantService : Service() {

    private var wakeLock: PowerManager.WakeLock? = null

    companion object {
        const val CHANNEL_ID = "deploy_instant_server_channel"
        const val NOTIFICATION_ID = 9101
        const val ACTION_START_OR_UPDATE = "com.hyunnism.deployinstant.ACTION_START_OR_UPDATE"
        const val ACTION_STOP_SERVICE = "com.hyunnism.deployinstant.ACTION_STOP_SERVICE"
        const val EXTRA_ACTIVE_COUNT = "extra_active_count"
        const val EXTRA_DESC = "extra_desc"

        fun startOrUpdate(context: Context, activeCount: Int, desc: String = "") {
            val intent = Intent(context, DeployInstantService::class.java).apply {
                action = ACTION_START_OR_UPDATE
                putExtra(EXTRA_ACTIVE_COUNT, activeCount)
                putExtra(EXTRA_DESC, desc)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            val intent = Intent(context, DeployInstantService::class.java).apply {
                action = ACTION_STOP_SERVICE
            }
            context.startService(intent)
        }
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent == null || intent.action == ACTION_STOP_SERVICE) {
            releaseWakeLock()
            stopSelf()
            return START_NOT_STICKY
        }

        val activeCount = intent.getIntExtra(EXTRA_ACTIVE_COUNT, 0)
        if (activeCount <= 0) {
            releaseWakeLock()
            stopSelf()
            return START_NOT_STICKY
        }

        acquireWakeLock()
        val desc = intent.getStringExtra(EXTRA_DESC) ?: "$activeCount project sedang berjalan"
        val notification = buildNotification(activeCount, desc)
        startForeground(NOTIFICATION_ID, notification)
        return START_STICKY
    }

    private fun buildNotification(activeCount: Int, desc: String): android.app.Notification {
        val openAppIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        val openAppPendingIntent = PendingIntent.getActivity(
            this,
            0,
            openAppIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or (if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0)
        )

        val stopAllIntent = Intent(this, ServiceActionReceiver::class.java).apply {
            action = ServiceActionReceiver.ACTION_STOP_ALL
        }
        val stopAllPendingIntent = PendingIntent.getBroadcast(
            this,
            1,
            stopAllIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or (if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0)
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_notify_sync)
            .setContentTitle("Deploy Instant Aktif")
            .setContentText(desc)
            .setContentIntent(openAppPendingIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .addAction(
                android.R.drawable.ic_menu_close_clear_cancel,
                "Stop Semua",
                stopAllPendingIntent
            )
            .build()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Deploy Instant Background Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Menjaga proses PM2 dan server Deploy Instant tetap aktif di latar belakang"
                setShowBadge(false)
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager?.createNotificationChannel(channel)
        }
    }

    private fun acquireWakeLock() {
        if (wakeLock == null) {
            val pm = getSystemService(Context.POWER_SERVICE) as? PowerManager
            wakeLock = pm?.newWakeLock(
                PowerManager.PARTIAL_WAKE_LOCK,
                "DeployInstant::ServerWakeLock"
            )?.apply {
                setReferenceCounted(false)
                // ponytail: tahan wake lock maksimal 24 jam per sesi aktif
                acquire(24 * 60 * 60 * 1000L)
            }
        }
    }

    private fun releaseWakeLock() {
        try {
            if (wakeLock?.isHeld == true) {
                wakeLock?.release()
            }
        } catch (_: Exception) {}
        wakeLock = null
    }

    override fun onDestroy() {
        releaseWakeLock()
        stopForeground(true)
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}


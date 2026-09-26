package com.corvofeng.fakeatv

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.net.wifi.WifiManager
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.util.Log
import androidx.core.app.NotificationCompat

class AtvService : Service(), AtvNative.Callback {

    companion object {
        private const val TAG = "AtvService"
        const val CHANNEL_ID = "fake_atv_service_channel"
        const val NOTIFICATION_ID = 1001

        const val ACTION_START = "com.corvofeng.fakeatv.ACTION_START"
        const val ACTION_STOP = "com.corvofeng.fakeatv.ACTION_STOP"
        const val BROADCAST_EVENT = "com.corvofeng.fakeatv.EVENT_BROADCAST"

        const val EXTRA_NAME = "extra_name"
        const val EXTRA_PIN = "extra_pin"
        const val EXTRA_EVENT_TYPE = "extra_event_type"
        const val EXTRA_EVENT_DETAIL = "extra_event_detail"

        var isRunning = false
            private set
    }

    private var multicastLock: WifiManager.MulticastLock? = null
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_STOP -> {
                stopServer()
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
            }
            else -> {
                val name = intent?.getStringExtra(EXTRA_NAME) ?: "Android TV"
                val pin = intent?.getIntExtra(EXTRA_PIN, 1111) ?: 1111
                startServer(name, pin)
            }
        }
        return START_STICKY
    }

    private fun startServer(name: String, pin: Int) {
        if (isRunning) return

        startForeground(NOTIFICATION_ID, buildNotification(name, pin))

        // Acquire MulticastLock to allow Bonjour mDNS discovery on Android TV Wi-Fi
        val wm = applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
        multicastLock = wm?.createMulticastLock("FakeAtvMulticastLock")?.apply {
            setReferenceCounted(true)
            acquire()
        }

        // Acquire WakeLock to keep networking responsive
        val pm = getSystemService(Context.POWER_SERVICE) as? PowerManager
        wakeLock = pm?.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "FakeAtv:WakeLock")?.apply {
            acquire(24 * 60 * 60 * 1000L) // 24 hours
        }

        Log.i(TAG, "Starting native server: $name (PIN $pin)")
        Thread {
            val started = AtvNative.nativeStartServer(name, pin, this)
            isRunning = started
            broadcast("ServerStatus", if (started) "running" else "failed")
        }.start()
    }

    private fun stopServer() {
        if (!isRunning) return
        Log.i(TAG, "Stopping native server")
        AtvNative.nativeStopServer()
        isRunning = false

        multicastLock?.let {
            if (it.isHeld) it.release()
        }
        wakeLock?.let {
            if (it.isHeld) it.release()
        }
        broadcast("ServerStatus", "stopped")
    }

    override fun onButton(name: String) {
        Log.i(TAG, "onButton: $name")
        AtvAccessibilityService.instance?.handleButton(name)
        broadcast("Button", name)
    }

    override fun onTouch(dx: Double, dy: Double, phase: Int) {
        AtvAccessibilityService.instance?.handleTouch(dx, dy, phase)
    }

    override fun onAudio(volume: Double, muted: Boolean) {
        broadcast("Audio", "vol=$volume muted=$muted")
    }

    override fun onEvent(kind: String, detail: String) {
        Log.i(TAG, "onEvent: $kind $detail")
        if (kind == "MouseClick") {
            AtvAccessibilityService.instance?.handleMouseClick()
        }
        broadcast("Event", "$kind: $detail")
    }

    private fun broadcast(type: String, detail: String) {
        val intent = Intent(BROADCAST_EVENT).apply {
            putExtra(EXTRA_EVENT_TYPE, type)
            putExtra(EXTRA_EVENT_DETAIL, detail)
            setPackage(packageName)
        }
        sendBroadcast(intent)
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Apple TV Remote Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Runs the fake Apple TV receiver on Android TV"
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager?.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(name: String, pin: Int): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Apple TV Remote Receiver")
            .setContentText("Listening as \"$name\" (PIN: $pin)")
            .setSmallIcon(android.R.drawable.ic_menu_compass)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        stopServer()
    }
}

package com.corvofeng.fakeatv

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log

class BootReceiver : BroadcastReceiver() {
    companion object {
        private const val TAG = "BootReceiver"
    }

    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action ?: return
        Log.i(TAG, "onReceive boot/wake broadcast: $action")

        val deviceName = DeviceInfo.getDeviceName(context)
        val serviceIntent = Intent(context, AtvService::class.java).apply {
            this.action = AtvService.ACTION_START
            putExtra(AtvService.EXTRA_NAME, deviceName)
            putExtra(AtvService.EXTRA_PIN, 1111)
        }
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(serviceIntent)
            } else {
                context.startService(serviceIntent)
            }
            Log.i(TAG, "AtvService started successfully from boot broadcast: $action")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start AtvService on broadcast ($action)", e)
        }
    }
}

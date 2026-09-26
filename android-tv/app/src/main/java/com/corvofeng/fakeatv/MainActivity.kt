package com.corvofeng.fakeatv

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.text.format.Formatter
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    private lateinit var tvStatus: TextView
    private lateinit var tvIp: TextView
    private lateinit var tvPin: TextView
    private lateinit var tvAccessibility: TextView
    private lateinit var tvLog: TextView
    private lateinit var btnToggle: Button
    private lateinit var btnAccessibility: Button

    private val logEntries = mutableListOf<String>()

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            val type = intent?.getStringExtra(AtvService.EXTRA_EVENT_TYPE) ?: return
            val detail = intent.getStringExtra(AtvService.EXTRA_EVENT_DETAIL) ?: ""
            appendLog("[$type] $detail")
            updateUi()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        tvStatus = findViewById(R.id.tvStatus)
        tvIp = findViewById(R.id.tvIp)
        tvPin = findViewById(R.id.tvPin)
        tvAccessibility = findViewById(R.id.tvAccessibility)
        tvLog = findViewById(R.id.tvLog)
        btnToggle = findViewById(R.id.btnToggle)
        btnAccessibility = findViewById(R.id.btnAccessibility)

        btnToggle.setOnClickListener {
            if (AtvService.isRunning) {
                val intent = Intent(this, AtvService::class.java).apply {
                    action = AtvService.ACTION_STOP
                }
                startService(intent)
            } else {
                startAtvService()
            }
            btnToggle.postDelayed({ updateUi() }, 500)
        }

        btnAccessibility.setOnClickListener {
            val intent = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
            startActivity(intent)
        }

        // Auto-start service on app launch
        if (!AtvService.isRunning) {
            startAtvService()
        }
    }

    override fun onResume() {
        super.onResume()
        val filter = IntentFilter(AtvService.BROADCAST_EVENT)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            registerReceiver(receiver, filter, Context.RECEIVER_NOT_EXPORTED)
        } else {
            registerReceiver(receiver, filter)
        }
        updateUi()
    }

    override fun onPause() {
        super.onPause()
        unregisterReceiver(receiver)
    }

    private fun startAtvService() {
        val intent = Intent(this, AtvService::class.java).apply {
            action = AtvService.ACTION_START
            putExtra(AtvService.EXTRA_NAME, "Android TV")
            putExtra(AtvService.EXTRA_PIN, 1111)
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }

    private fun updateUi() {
        val running = AtvService.isRunning
        tvStatus.text = if (running) "Status: Running" else "Status: Stopped"
        tvStatus.setTextColor(if (running) 0xFF4CAF50.toInt() else 0xFFF44336.toInt())
        btnToggle.text = if (running) "Stop Server" else "Start Server"

        tvIp.text = "IP Address: ${getWifiIpAddress()}"
        tvPin.text = "Pairing PIN: 1111"

        val accessOk = AtvAccessibilityService.isServiceRunning
        tvAccessibility.text = "Accessibility Service: " + if (accessOk) "Enabled (Full Control)" else "Disabled (Tap below to enable)"
        tvAccessibility.setTextColor(if (accessOk) 0xFF4CAF50.toInt() else 0xFFFF9800.toInt())
    }

    private fun appendLog(line: String) {
        logEntries.add(0, line)
        if (logEntries.size > 15) {
            logEntries.removeLast()
        }
        tvLog.text = logEntries.joinToString("\n")
    }

    @Suppress("DEPRECATION")
    private fun getWifiIpAddress(): String {
        return try {
            val wm = applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
            val ip = wm?.connectionInfo?.ipAddress ?: 0
            if (ip != 0) {
                Formatter.formatIpAddress(ip)
            } else {
                "Unknown"
            }
        } catch (e: Exception) {
            "Unknown"
        }
    }
}

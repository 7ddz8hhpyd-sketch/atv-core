package com.corvofeng.fakeatv

import android.content.ActivityNotFoundException
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
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
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
            if (RemoteController.isHardwareAvailable) {
                Toast.makeText(this, "Hardware remote driver (/dev/input/event0) is active!\nNo accessibility service needed.", Toast.LENGTH_SHORT).show()
                return@setOnClickListener
            }
            if (AtvAccessibilityService.isServiceRunning) {
                Toast.makeText(this, "Accessibility Service is already enabled", Toast.LENGTH_SHORT).show()
                return@setOnClickListener
            }
            try {
                val intent = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
                startActivity(intent)
            } catch (e: ActivityNotFoundException) {
                AlertDialog.Builder(this)
                    .setTitle("Accessibility Settings")
                    .setMessage(
                        "This device does not have a system Accessibility Settings page.\n\n" +
                        "Enable via ADB:\n\n" +
                        "adb shell settings put secure enabled_accessibility_services com.corvofeng.fakeatv/.AtvAccessibilityService\n" +
                        "adb shell settings put secure accessibility_enabled 1"
                    )
                    .setPositiveButton("OK", null)
                    .show()
            }
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
        val deviceName = DeviceInfo.getDeviceName(this)
        val intent = Intent(this, AtvService::class.java).apply {
            action = AtvService.ACTION_START
            putExtra(AtvService.EXTRA_NAME, deviceName)
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
        val deviceName = DeviceInfo.getDeviceName(this)
        val deviceId = DeviceInfo.getDeviceId(this)
        tvStatus.text = if (running) "$deviceName: Running" else "$deviceName: Stopped"
        tvStatus.setTextColor(if (running) 0xFF4CAF50.toInt() else 0xFFF44336.toInt())
        btnToggle.text = if (running) "Stop Server" else "Start Server"

        tvIp.text = "IP: ${getWifiIpAddress()}   ID: $deviceId"
        tvPin.text = "Pairing PIN: 1111"

        val hwOk = RemoteController.isHardwareAvailable
        val accessOk = AtvAccessibilityService.isServiceRunning
        if (hwOk) {
            tvAccessibility.text = "Control Mode: Hardware Remote Driver (/dev/input/event0) Active"
            tvAccessibility.setTextColor(0xFF4CAF50.toInt())
            btnAccessibility.text = "Hardware Driver Active"
        } else {
            tvAccessibility.text = "Accessibility Service: " + if (accessOk) "Enabled (Full Control)" else "Disabled (Tap below to enable)"
            tvAccessibility.setTextColor(if (accessOk) 0xFF4CAF50.toInt() else 0xFFFF9800.toInt())
            btnAccessibility.text = "Accessibility Settings"
        }
    }

    private fun appendLog(line: String) {
        logEntries.add(0, line)
        if (logEntries.size > 15) {
            logEntries.removeLast()
        }
        tvLog.text = logEntries.joinToString("\n")
    }

    private fun getWifiIpAddress(): String {
        return DeviceInfo.getLocalIpAddress(this) ?: "Unknown"
    }
}

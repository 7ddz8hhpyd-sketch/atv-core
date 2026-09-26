package com.corvofeng.fakeatv

import android.content.Context
import android.util.Log
import dadb.AdbKeyPair
import dadb.AdbShellStream
import dadb.Dadb
import java.io.File
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

object LocalAdbDriver {
    private const val TAG = "LocalAdbDriver"
    private const val ADB_HOST = "127.0.0.1"
    private const val ADB_PORT = 5555

    private val executor = Executors.newSingleThreadExecutor()
    private var appContext: Context? = null
    private var dadbInstance: Dadb? = null
    private var shellStream: AdbShellStream? = null

    val isConnected = AtomicBoolean(false)

    fun init(context: Context) {
        appContext = context.applicationContext
        initAsync()
    }

    fun initAsync(callback: ((Boolean) -> Unit)? = null) {
        executor.execute {
            val ok = connectInternal()
            callback?.invoke(ok)
        }
    }

    @Synchronized
    fun connectInternal(): Boolean {
        if (isConnected.get() && shellStream != null) {
            return true
        }
        closeInternal()
        try {
            Log.i(TAG, "Attempting to connect to local ADB at $ADB_HOST:$ADB_PORT...")
            val context = appContext
            val d = if (context != null) {
                val keyDir = File(context.filesDir, "adb")
                keyDir.mkdirs()
                val privFile = File(keyDir, "adbkey")
                val pubFile = File(keyDir, "adbkey.pub")
                if (!privFile.exists() || !pubFile.exists()) {
                    Log.i(TAG, "Generating internal ADB keypair at $keyDir...")
                    AdbKeyPair.generate(privFile, pubFile)
                }
                val keyPair = AdbKeyPair.read(privFile, pubFile)
                Dadb.create(ADB_HOST, ADB_PORT, keyPair)
            } else {
                Dadb.create(ADB_HOST, ADB_PORT)
            }

            val stream = d.openShell()
            dadbInstance = d
            shellStream = stream
            isConnected.set(true)
            Log.i(TAG, "Successfully connected to local ADB daemon!")
            return true
        } catch (e: Exception) {
            Log.w(TAG, "Failed to connect to local ADB ($ADB_HOST:$ADB_PORT): ${e.message}")
            closeInternal()
            return false
        }
    }

    @Synchronized
    fun closeInternal() {
        try { shellStream?.close() } catch (_: Exception) {}
        try { dadbInstance?.close() } catch (_: Exception) {}
        shellStream = null
        dadbInstance = null
        isConnected.set(false)
    }

    private val KEY_MAP = mapOf(
        "up" to "KEYCODE_DPAD_UP",
        "down" to "KEYCODE_DPAD_DOWN",
        "left" to "KEYCODE_DPAD_LEFT",
        "right" to "KEYCODE_DPAD_RIGHT",
        "select" to "KEYCODE_DPAD_CENTER",
        "menu" to "KEYCODE_BACK",
        "home" to "KEYCODE_HOME",
        "play_pause" to "KEYCODE_MEDIA_PLAY_PAUSE",
        "volume_up" to "KEYCODE_VOLUME_UP",
        "volume_down" to "KEYCODE_VOLUME_DOWN",
        "mute" to "KEYCODE_VOLUME_MUTE",
        "power" to "KEYCODE_POWER",
        "siri" to "KEYCODE_SEARCH"
    )

    fun sendKey(buttonName: String, isMappedToMenu: Boolean = false): Boolean {
        val keycode = if (isMappedToMenu) {
            "KEYCODE_MENU"
        } else {
            KEY_MAP[buttonName.lowercase()] ?: "KEYCODE_${buttonName.uppercase()}"
        }

        if (!isConnected.get()) {
            if (!connectInternal()) {
                return false
            }
        }

        return synchronized(this) {
            try {
                val stream = shellStream ?: return false
                val cmd = "input keyevent $keycode\n"
                stream.write(cmd)
                Log.d(TAG, "ADB sent keyevent: $keycode")
                true
            } catch (e: Exception) {
                Log.w(TAG, "Error writing to ADB shell stream: ${e.message}, will retry reconnection")
                closeInternal()
                if (connectInternal()) {
                    try {
                        shellStream?.write("input keyevent $keycode\n")
                        return true
                    } catch (_: Exception) {}
                }
                false
            }
        }
    }
}

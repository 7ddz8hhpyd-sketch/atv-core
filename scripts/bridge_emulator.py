#!/usr/bin/env python3
"""
Apple TV Remote - Android Emulator Bridge for macOS
===================================================
Bridges Apple TV Remote protocol traffic from your physical Wi-Fi network
into the Android TV emulator via ADB port forwarding and macOS mDNS (Bonjour).

Usage:
    python3 scripts/bridge_emulator.py
"""

import asyncio
import os
import signal
import subprocess
import sys

DEVICE_NAME = "Android TV Emulator"
MRP_PORT = 49152
COMPANION_PORT = 49153
AIRPLAY_PORT = 49154

LOCAL_FORWARD_MRP = 59152
LOCAL_FORWARD_COMPANION = 59153

DEVICE_ID = "AA:BB:CC:DD:EE:01"
SERVER_IDENTIFIER = "2E468249-2F22-4416-86C8-50BF22D4F24D"
UNIQUE_ID = SERVER_IDENTIFIER.replace("-", "")
DEVICE_MODEL = "AppleTV5,3"
SOURCE_VERSION = "715.2"

dns_processes = []

async def pipe(reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
    try:
        while not reader.at_eof():
            data = await reader.read(4096)
            if not data:
                break
            writer.write(data)
            await writer.drain()
    except Exception:
        pass
    finally:
        try:
            writer.close()
            await writer.wait_closed()
        except Exception:
            pass

def create_proxy(target_port: int, service_name: str):
    async def handler(local_reader: asyncio.StreamReader, local_writer: asyncio.StreamWriter):
        peer = local_writer.get_extra_info("peername")
        print(f"[{service_name}] Connection from {peer} -> forwarding to emulator:{target_port}")
        try:
            remote_reader, remote_writer = await asyncio.open_connection("127.0.0.1", target_port)
            asyncio.create_task(pipe(local_reader, remote_writer))
            asyncio.create_task(pipe(remote_reader, local_writer))
        except Exception as e:
            print(f"[{service_name}] Failed to connect to emulator ({target_port}): {e}")
            local_writer.close()
    return handler

async def handle_airplay(reader: asyncio.StreamReader, writer: asyncio.StreamWriter):
    peer = writer.get_extra_info("peername")
    print(f"[AirPlay] Ping from {peer}")
    try:
        await reader.read(4096)
        writer.write(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK")
        await writer.drain()
    except Exception:
        pass
    finally:
        try:
            writer.close()
            await writer.wait_closed()
        except Exception:
            pass

def start_mdns_service(service_type: str, port: int, txt_records: list):
    cmd = ["/usr/bin/dns-sd", "-R", DEVICE_NAME, service_type, "local.", str(port)] + txt_records
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    dns_processes.append(proc)

def cleanup():
    print("\nStopping bridge and cleaning up mDNS...")
    for p in dns_processes:
        try:
            p.terminate()
            p.wait(timeout=1)
        except Exception:
            try:
                p.kill()
            except Exception:
                pass
    try:
        subprocess.run(["adb", "forward", "--remove", f"tcp:{LOCAL_FORWARD_MRP}"], capture_output=True)
        subprocess.run(["adb", "forward", "--remove", f"tcp:{LOCAL_FORWARD_COMPANION}"], capture_output=True)
    except Exception:
        pass
    print("Cleanup done.")

async def main():
    print("=== Apple TV Remote Emulator Bridge ===")
    
    # 1. Setup adb port forwards
    print("1. Configuring ADB port forwards to Android TV emulator...")
    res1 = subprocess.run(["adb", "forward", f"tcp:{LOCAL_FORWARD_MRP}", f"tcp:{MRP_PORT}"], capture_output=True, text=True)
    res2 = subprocess.run(["adb", "forward", f"tcp:{LOCAL_FORWARD_COMPANION}", f"tcp:{COMPANION_PORT}"], capture_output=True, text=True)
    if res1.returncode != 0 or res2.returncode != 0:
        print(f"Error configuring ADB forward: {res1.stderr} {res2.stderr}")
        print("Please make sure the Android emulator is running and 'adb devices' sees it.")
        return

    # 2. Start TCP relay servers on 0.0.0.0
    print("2. Starting TCP relays on Mac network interfaces...")
    try:
        server_mrp = await asyncio.start_server(
            create_proxy(LOCAL_FORWARD_MRP, "MRP"),
            "0.0.0.0",
            MRP_PORT
        )
        server_companion = await asyncio.start_server(
            create_proxy(LOCAL_FORWARD_COMPANION, "Companion"),
            "0.0.0.0",
            COMPANION_PORT
        )
        server_airplay = await asyncio.start_server(
            handle_airplay,
            "0.0.0.0",
            AIRPLAY_PORT
        )
    except OSError as e:
        print(f"Error binding port: {e}")
        print("Make sure no other instance of fake_atv or bridge is occupying ports 49152/49153.")
        return

    # 3. Register mDNS / Bonjour services via macOS mDNSResponder
    print("3. Registering Bonjour mDNS services on local Wi-Fi...")
    start_mdns_service(
        "_mediaremotetv._tcp",
        MRP_PORT,
        [
            f"Name={DEVICE_NAME}",
            f"UniqueIdentifier={UNIQUE_ID}",
            "SystemBuildVersion=22K160",
            "LocalAirPlayReceiverPairingIdentity=9C4F2B8A1D3E5F60",
            "ModelName=Apple TV",
            "AllowPairing=YES",
        ]
    )
    start_mdns_service(
        "_companion-link._tcp",
        COMPANION_PORT,
        [
            "rpMac=1",
            "rpHA=D851F0A4E5C9",
            "rpHN=B7359A9BCBAB",
            f"rpVr={SOURCE_VERSION}",
            f"rpMd={DEVICE_MODEL}",
            "rpFl=0x36782",
            "rpAD=F0E18C86DB60",
            "rpHI=40DB206B32FA",
            f"rpBA={DEVICE_ID}",
        ]
    )
    start_mdns_service(
        "_airplay._tcp",
        AIRPLAY_PORT,
        [
            f"deviceid={DEVICE_ID}",
            "features=0x5A7FFFF7,0x1E",
            "flags=0x44",
            f"model={DEVICE_MODEL}",
            f"srcvers={SOURCE_VERSION}",
            "vv=2",
            f"pi={SERVER_IDENTIFIER}",
            "pk=6b8b4567f85b7f54a3e1c0a93f0a9e2c",
            f"name={DEVICE_NAME}",
        ]
    )

    print("\n" + "=" * 60)
    print(f"  ✓ Bridge is RUNNING successfully!")
    print(f"  • Device Name : {DEVICE_NAME}")
    print(f"  • Pairing PIN : 1111 (Default)")
    print(f"  • Target      : Android Emulator (Ports 49152 & 49153)")
    print("=" * 60)
    print("Instructions:")
    print("1. On iPhone / iPad, open Control Center -> tap Apple TV Remote.")
    print(f"2. You should now see \"{DEVICE_NAME}\" appear in the device list!")
    print("3. Tap it and enter PIN: 1111")
    print("4. Press buttons or swipe on iPhone -> watch the Android TV emulator react!")
    print("=" * 60)
    print("Press Ctrl+C to exit.\n")

    loop = asyncio.get_running_loop()
    stop_event = asyncio.Event()

    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, stop_event.set)

    try:
        await stop_event.wait()
    finally:
        cleanup()

if __name__ == "__main__":
    asyncio.run(main())

#!/usr/bin/env python3
import argparse
import base64
import json
import socket
import ssl
import sys
import time
from urllib.parse import urlencode

try:
    from websocket import create_connection
except Exception as e:
    print("ERROR: Missing dependency 'websocket-client'. Install with:", file=sys.stderr)
    print("  python3 -m pip install --user websocket-client", file=sys.stderr)
    raise

def b64_name(name: str) -> str:
    return base64.b64encode(name.encode("utf-8")).decode("ascii")

def build_url(host: str, port: int, app_name: str, token: str | None) -> str:
    # Samsung TV websocket remote endpoint:
    # /api/v2/channels/samsung.remote.control?name=<base64(appname)>&token=<optional>
    # (Token use is optional; some TVs will show an "Allow" prompt on first connect.)
    params = {"name": b64_name(app_name)}
    if token:
        params["token"] = token
    return f"ws://{host}:{port}/api/v2/channels/samsung.remote.control?{urlencode(params)}"

def send_key(ws, key: str) -> None:
    # Common payload format used for key sending:
    payload = {
        "method": "ms.remote.control",
        "params": {
            "Cmd": "Click",
            "DataOfCmd": key,
            "Option": "false",
            "TypeOfRemote": "SendRemoteKey",
        },
    }
    ws.send(json.dumps(payload))

def main() -> int:
    parser = argparse.ArgumentParser(description="Send volume keys to Samsung TV over websocket.")
    parser.add_argument("--host", required=True, help="TV IP address (e.g., 192.168.1.50)")
    parser.add_argument("--port", type=int, default=8001, help="WebSocket port (default: 8001)")
    parser.add_argument("--app", default="MacVolumeRemote", help="App name shown on TV (default: MacVolumeRemote)")
    parser.add_argument("--token", default=None, help="Optional token (if your TV provides one)")
    parser.add_argument("action", choices=["up", "down", "mute"], help="Volume action")
    parser.add_argument("--count", type=int, default=1, help="Number of times to send (default: 1)")
    parser.add_argument("--delay", type=float, default=0.25, help="Delay between sends in seconds (default: 0.25)")
    args = parser.parse_args()

    key_map = {
        "up": "KEY_VOLUP",
        "down": "KEY_VOLDOWN",
        "mute": "KEY_MUTE",
    }
    key = key_map[args.action]

    url = build_url(args.host, args.port, args.app, args.token)

    # Basic safety: short socket timeout so we don't hang forever on a dead IP.
    socket.setdefaulttimeout(3.0)

    try:
        ws = create_connection(url, timeout=3.0)
    except Exception as e:
        print(f"ERROR: Could not connect to {url}", file=sys.stderr)
        print(f"  {e}", file=sys.stderr)
        print("", file=sys.stderr)
        print("Notes:", file=sys.stderr)
        print("  - TV must be on and on the same network.", file=sys.stderr)
        print("  - Some models prompt on the TV to allow the connection the first time.", file=sys.stderr)
        print("  - If port 8001 fails, your model might require a different port or secure websocket.", file=sys.stderr)
        return 2

    try:
        # Some TVs like a tiny pause after connect.
        time.sleep(0.15)

        for i in range(args.count):
            send_key(ws, key)
            if i + 1 < args.count:
                time.sleep(args.delay)

    finally:
        try:
            ws.close()
        except Exception:
            pass

    return 0

if __name__ == "__main__":
    raise SystemExit(main())

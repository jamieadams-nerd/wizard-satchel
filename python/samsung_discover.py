#!/usr/bin/env python3
import argparse
import base64
import json
import socket
import sys
import time
import urllib.request
import urllib.error
import xml.etree.ElementTree as ET
from urllib.parse import urlencode, urlparse

try:
    from websocket import create_connection
except Exception:
    print("ERROR: Missing dependency 'websocket-client'. Install with:", file=sys.stderr)
    print("  python3 -m pip install --user websocket-client", file=sys.stderr)
    raise

SSDP_ADDR = ("239.255.255.250", 1900)

# A few ST values to try. The Samsung legacy PDF describes Samsung-specific STs.  [oai_citation:2‡CloudFront](https://d3unf4s5rp9dfh.cloudfront.net/SmartTV_Legacy/convergence/convergence/Device%2BDiscovery%2C%2BAuthentication%2C%2Band%2BPairing.pdf?utm_source=chatgpt.com)
ST_CANDIDATES = [
    "urn:samsung.com:service:MultiScreenService:1",
    "urn:samsung.com:device:RemoteControlReceiver:1",
    "upnp:rootdevice",
    "ssdp:all",
]

def http_get(url: str, timeout: float = 2.0) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "MacSamsungDiscover/1.0"})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read()

def parse_ssdp_response(data: bytes) -> dict:
    # SSDP replies are HTTP-like headers over UDP.
    text = data.decode("utf-8", errors="replace")
    lines = [ln.strip() for ln in text.split("\n") if ln.strip()]
    headers = {}
    for ln in lines[1:]:
        if ":" in ln:
            k, v = ln.split(":", 1)
            headers[k.strip().upper()] = v.strip()
    return headers

def ssdp_discover(timeout_s: float = 2.0, mx: int = 1) -> list[dict]:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    try:
        sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)
        sock.settimeout(0.25)

        results = []
        seen = set()

        for st in ST_CANDIDATES:
            msg = "\r\n".join([
                "M-SEARCH * HTTP/1.1",
                f"HOST: {SSDP_ADDR[0]}:{SSDP_ADDR[1]}",
                'MAN: "ssdp:discover"',
                f"MX: {mx}",
                f"ST: {st}",
                "",
                "",
            ]).encode("utf-8")

            try:
                sock.sendto(msg, SSDP_ADDR)
            except OSError:
                continue

        end = time.time() + timeout_s
        while time.time() < end:
            try:
                data, addr = sock.recvfrom(65535)
            except socket.timeout:
                continue
            except OSError:
                break

            ip = addr[0]
            headers = parse_ssdp_response(data)
            location = headers.get("LOCATION", "")
            usn = headers.get("USN", "")
            st = headers.get("ST", headers.get("NT", ""))

            key = (ip, location, usn, st)
            if key in seen:
                continue
            seen.add(key)

            results.append({
                "ip": ip,
                "server": headers.get("SERVER", ""),
                "st": st,
                "usn": usn,
                "location": location,
            })

        return results
    finally:
        try:
            sock.close()
        except Exception:
            pass

def looks_like_samsung_from_upnp(location_url: str) -> tuple[bool, str]:
    if not location_url:
        return (False, "")
    try:
        xml_bytes = http_get(location_url, timeout=2.0)
    except Exception:
        return (False, "")

    try:
        root = ET.fromstring(xml_bytes)
    except ET.ParseError:
        return (False, "")

    # Find <manufacturer>, <modelName> in any namespace.
    manufacturer = ""
    model = ""
    for el in root.iter():
        tag = el.tag.lower()
        if tag.endswith("manufacturer") and el.text:
            manufacturer = el.text.strip()
        if tag.endswith("modelname") and el.text:
            model = el.text.strip()

    info = " / ".join([x for x in [manufacturer, model] if x])
    is_samsung = ("samsung" in manufacturer.lower()) or ("samsung" in model.lower())
    return (is_samsung, info)

def tv_api_v2_info(ip: str) -> tuple[bool, str]:
    # Many 2016+ TVs expose http://<ip>:8001/api/v2/ with JSON device info.  [oai_citation:3‡Reddit](https://www.reddit.com/r/crestron/comments/vx9ak4/samsung_tv_via_websockets_api/?utm_source=chatgpt.com)
    url = f"http://{ip}:8001/api/v2/"
    try:
        data = http_get(url, timeout=1.5)
        j = json.loads(data.decode("utf-8", errors="replace"))
        name = j.get("device", {}).get("name") or j.get("name") or ""
        model = j.get("device", {}).get("modelName") or ""
        info = " / ".join([x for x in [name, model] if x])
        return (True, info if info else "api/v2 responded")
    except Exception:
        return (False, "")

def b64_name(name: str) -> str:
    return base64.b64encode(name.encode("utf-8")).decode("ascii")

def ws_url(ip: str, app_name: str, token: str | None) -> str:
    params = {"name": b64_name(app_name)}
    if token:
        params["token"] = token
    return f"ws://{ip}:8001/api/v2/channels/samsung.remote.control?{urlencode(params)}"

def send_volume_key(ip: str, action: str, app_name: str, token: str | None, count: int, delay: float) -> None:
    key_map = {"up": "KEY_VOLUP", "down": "KEY_VOLDOWN", "mute": "KEY_MUTE"}
    key = key_map[action]
    url = ws_url(ip, app_name, token)

    socket.setdefaulttimeout(3.0)
    ws = create_connection(url, timeout=3.0)
    try:
        time.sleep(0.15)
        payload = {
            "method": "ms.remote.control",
            "params": {
                "Cmd": "Click",
                "DataOfCmd": key,
                "Option": "false",
                "TypeOfRemote": "SendRemoteKey",
            },
        }
        msg = json.dumps(payload)
        for i in range(count):
            ws.send(msg)
            if i + 1 < count:
                time.sleep(delay)
    finally:
        try:
            ws.close()
        except Exception:
            pass

def main() -> int:
    ap = argparse.ArgumentParser(description="Discover Samsung TVs via SSDP and send volume keys.")
    ap.add_argument("--timeout", type=float, default=2.5, help="Discovery listen time in seconds (default: 2.5)")
    ap.add_argument("--app", default="MacVolumeRemote", help="App name shown on TV (default: MacVolumeRemote)")
    ap.add_argument("--token", default=None, help="Optional token (if you have one)")
    ap.add_argument("action", choices=["list", "up", "down", "mute"], help="list devices, or send a volume action")
    ap.add_argument("--count", type=int, default=1, help="Times to send key (default: 1)")
    ap.add_argument("--delay", type=float, default=0.25, help="Delay between sends (default: 0.25)")
    args = ap.parse_args()

    raw = ssdp_discover(timeout_s=args.timeout)
    # Build a “device list” with best-effort identification.
    devices = []
    for r in raw:
        ip = r["ip"]
        loc = r["location"]

        is_samsung, upnp_info = looks_like_samsung_from_upnp(loc)
        api_ok, api_info = tv_api_v2_info(ip)

        score = 0
        info_parts = []
        if is_samsung:
            score += 2
            if upnp_info:
                info_parts.append(upnp_info)
        if api_ok:
            score += 2
            if api_info:
                info_parts.append(api_info)
        if "samsung" in (r.get("server", "").lower()):
            score += 1

        # Keep everything, but sort with likely Samsungs first.
        devices.append({
            "ip": ip,
            "score": score,
            "st": r.get("st", ""),
            "server": r.get("server", ""),
            "location": loc,
            "info": " | ".join([p for p in info_parts if p]) or "",
        })

    devices.sort(key=lambda d: (d["score"], d["ip"]), reverse=True)

    if args.action == "list":
        if not devices:
            print("No SSDP devices found. Make sure TV is on and on the same LAN.")
            return 1
        for i, d in enumerate(devices, 1):
            tag = "LIKELY-SAMSUNG" if d["score"] >= 3 else "maybe"
            print(f"{i:2d}. {d['ip']}  [{tag}]  {d['info']}")
        print("")
        print("Run one of:")
        print("  python3 samsung_discover_and_volume.py up")
        print("  python3 samsung_discover_and_volume.py down --count 5")
        print("  python3 samsung_discover_and_volume.py mute")
        return 0

    # For actions, pick the top-scoring device.
    likely = [d for d in devices if d["score"] >= 3]
    if not likely:
        # Fall back to any device found (user can still try).
        if not devices:
            print("No devices found. Try: python3 samsung_discover_and_volume.py list")
            return 1
        target = devices[0]
        print("WARNING: No strongly-identified Samsung TV found; trying best candidate:", target["ip"], file=sys.stderr)
    else:
        target = likely[0]

    print(f"Using TV at {target['ip']} ({target['info']})")
    print("If your TV prompts 'Allow this device', approve it with your physical remote.")
    send_volume_key(target["ip"], args.action, args.app, args.token, args.count, args.delay)
    print("Done.")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

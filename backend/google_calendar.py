"""Google Calendar direct integration for Notely Desktop.
Provides OAuth loopback authentication, token refresh, and direct event synchronization
without requiring manual browser tabs for each deadline.
"""
from __future__ import annotations
import base64
import datetime
import json
import os
import threading
import time
import urllib.parse
import urllib.request
import urllib.error
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path

CONFIG_FILE = 'google_calendar_config.json'
TOKENS_FILE = 'google_calendar_tokens.json'
LOOPBACK_PORT = 18240
REDIRECT_URI = f'http://127.0.0.1:{LOOPBACK_PORT}/callback'

# Default fallback client credentials embedded for Notely Desktop App (RFC 8252 public client)
_K = 0x5C
_CB = [101, 104, 100, 111, 107, 101, 111, 101, 100, 100, 111, 106, 113, 41, 108, 101, 106, 54, 55, 55, 104, 45, 101, 107, 61, 54, 51, 108, 106, 109, 108, 58, 46, 57, 109, 46, 57, 105, 47, 63, 42, 100, 110, 54, 55, 114, 61, 44, 44, 47, 114, 59, 51, 51, 59, 48, 57, 41, 47, 57, 46, 63, 51, 50, 40, 57, 50, 40, 114, 63, 51, 49]
_SB = [27, 19, 31, 15, 12, 4, 113, 49, 31, 41, 36, 24, 56, 38, 13, 52, 45, 58, 9, 37, 111, 38, 44, 4, 58, 10, 54, 113, 44, 41, 4, 63, 51, 8, 42]

_EMBEDDED_CID = bytes(b ^ _K for b in _CB).decode('utf-8')
_EMBEDDED_SEC = bytes(b ^ _K for b in _SB).decode('utf-8')

DEFAULT_CLIENT_ID = os.environ.get('NOTELY_GOOGLE_CLIENT_ID') or _EMBEDDED_CID
DEFAULT_CLIENT_SECRET = os.environ.get('NOTELY_GOOGLE_CLIENT_SECRET') or _EMBEDDED_SEC

_loopback_server = None
_loopback_thread = None
_auth_state_lock = threading.Lock()

def _get_path(data_root: str | Path, filename: str) -> Path:
    p = Path(data_root)
    p.mkdir(parents=True, exist_ok=True)
    return p / filename

def get_config(data_root: str | Path) -> dict:
    path = _get_path(data_root, CONFIG_FILE)
    if path.exists():
        try:
            cfg = json.loads(path.read_text(encoding='utf-8'))
            if not cfg.get('clientId'):
                cfg['clientId'] = DEFAULT_CLIENT_ID
            if not cfg.get('clientSecret'):
                cfg['clientSecret'] = DEFAULT_CLIENT_SECRET
            return cfg
        except Exception:
            pass
    return {
        'clientId': DEFAULT_CLIENT_ID,
        'clientSecret': DEFAULT_CLIENT_SECRET,
        'autoSync': True,
        'preferredMode': 'web',
        'dismissPrompt': False
    }

def save_config(data_root: str | Path, config: dict):
    path = _get_path(data_root, CONFIG_FILE)
    path.write_text(json.dumps(config, indent=2, ensure_ascii=False), encoding='utf-8')

def get_tokens(data_root: str | Path) -> dict:
    path = _get_path(data_root, TOKENS_FILE)
    if path.exists():
        try:
            return json.loads(path.read_text(encoding='utf-8'))
        except Exception:
            pass
    return {}

def save_tokens(data_root: str | Path, tokens: dict):
    path = _get_path(data_root, TOKENS_FILE)
    path.write_text(json.dumps(tokens, indent=2, ensure_ascii=False), encoding='utf-8')

def get_status(data_root: str | Path) -> dict:
    config = get_config(data_root)
    tokens = get_tokens(data_root)
    # Real connection requires actual OAuth tokens
    connected = bool(tokens.get('refresh_token') or (tokens.get('mode') == 'oauth' and tokens.get('access_token')))
    effective_client_id = config.get('clientId') or DEFAULT_CLIENT_ID
    effective_secret = config.get('clientSecret') or DEFAULT_CLIENT_SECRET
    has_credentials = bool(effective_client_id and effective_secret)
    pref_mode = 'oauth' if connected else config.get('preferredMode', 'web')
    return {
        'connected': connected,
        'email': tokens.get('email', ''),
        'mode': 'oauth' if connected else (pref_mode if pref_mode == 'web' else 'none'),
        'preferredMode': pref_mode,
        'autoSync': config.get('autoSync', True),
        'hasCredentials': has_credentials,
        'clientId': effective_client_id,
        'dismissPrompt': config.get('dismissPrompt', False)
    }

def disconnect(data_root: str | Path):
    path = _get_path(data_root, TOKENS_FILE)
    if path.exists():
        path.unlink(missing_ok=True)

def refresh_access_token_if_needed(data_root: str | Path) -> str:
    tokens = get_tokens(data_root)
    config = get_config(data_root)
    access_token = tokens.get('access_token')
    expires_at = tokens.get('expires_at', 0)
    refresh_token = tokens.get('refresh_token')

    if access_token and time.time() < (expires_at - 60):
        return access_token

    if not refresh_token:
        raise RuntimeError("Google Calendar belum terhubung. Sambungkan akun terlebih dahulu.")

    client_id = config.get('clientId') or DEFAULT_CLIENT_ID
    client_secret = config.get('clientSecret') or DEFAULT_CLIENT_SECRET
    if not client_id or not client_secret:
        raise RuntimeError("Client ID atau Secret Google belum dikonfigurasi.")

    payload = urllib.parse.urlencode({
        'client_id': client_id,
        'client_secret': client_secret,
        'refresh_token': refresh_token,
        'grant_type': 'refresh_token'
    }).encode('utf-8')

    req = urllib.request.Request(
        'https://oauth2.googleapis.com/token',
        data=payload,
        headers={'Content-Type': 'application/x-www-form-urlencoded'}
    )
    with urllib.request.urlopen(req, timeout=15) as resp:
        data = json.loads(resp.read().decode('utf-8'))

    new_access_token = data['access_token']
    tokens['access_token'] = new_access_token
    tokens['expires_at'] = int(time.time()) + int(data.get('expires_in', 3600))
    save_tokens(data_root, tokens)
    return new_access_token

def _fetch_user_email(access_token: str) -> str:
    try:
        req = urllib.request.Request(
            'https://www.googleapis.com/oauth2/v2/userinfo',
            headers={'Authorization': f'Bearer {access_token}'}
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read().decode('utf-8'))
            return data.get('email', '')
    except Exception:
        return ''

def import_credentials_json(data_root: str | Path, content: str | dict) -> dict:
    """Import and save Google OAuth credentials from a downloaded client_secret_*.json file."""
    if isinstance(content, str):
        data = json.loads(content)
    else:
        data = content
    creds = data.get('installed') or data.get('web') or data
    client_id = str(creds.get('client_id', '')).strip()
    client_secret = str(creds.get('client_secret', '')).strip()
    if not client_id:
        raise ValueError("File JSON tidak memuat 'client_id' Google OAuth yang valid.")

    config = get_config(data_root)
    config['clientId'] = client_id
    if client_secret:
        config['clientSecret'] = client_secret
    save_config(data_root, config)
    return {
        'success': True,
        'clientId': client_id,
        'hasSecret': bool(client_secret)
    }

def start_oauth_flow(data_root: str | Path, client_id: str = '', client_secret: str = '') -> str:
    """Start local loopback server and return Google OAuth authorization URL."""
    global _loopback_server, _loopback_thread

    config = get_config(data_root)
    if client_id:
        config['clientId'] = client_id.strip()
    if client_secret:
        config['clientSecret'] = client_secret.strip()
    save_config(data_root, config)

    c_id = config.get('clientId') or DEFAULT_CLIENT_ID
    c_sec = config.get('clientSecret') or DEFAULT_CLIENT_SECRET
    if not c_id:
        raise ValueError("Google OAuth Client ID belum dikonfigurasi. Masukkan Client ID & Secret Google Cloud Anda terlebih dahulu.")

    with _auth_state_lock:
        if _loopback_server:
            try:
                _loopback_server.shutdown()
                _loopback_server.server_close()
            except Exception:
                pass
            _loopback_server = None

        class OAuthCallbackHandler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_GET(self):
                parsed = urllib.parse.urlparse(self.path)
                qs = urllib.parse.parse_qs(parsed.query)
                code = qs.get('code', [''])[0]
                error = qs.get('error', [''])[0]

                if code:
                    try:
                        token_payload = urllib.parse.urlencode({
                            'code': code,
                            'client_id': c_id,
                            'client_secret': c_sec,
                            'redirect_uri': REDIRECT_URI,
                            'grant_type': 'authorization_code'
                        }).encode('utf-8')

                        t_req = urllib.request.Request(
                            'https://oauth2.googleapis.com/token',
                            data=token_payload,
                            headers={'Content-Type': 'application/x-www-form-urlencoded'}
                        )
                        with urllib.request.urlopen(t_req, timeout=15) as t_resp:
                            token_data = json.loads(t_resp.read().decode('utf-8'))

                        access_token = token_data.get('access_token', '')
                        refresh_token = token_data.get('refresh_token', '')
                        existing = get_tokens(data_root)
                        if not refresh_token and existing.get('refresh_token'):
                            refresh_token = existing['refresh_token']
                        email = _fetch_user_email(access_token) if access_token else ''

                        save_tokens(data_root, {
                            'access_token': access_token,
                            'refresh_token': refresh_token,
                            'expires_at': int(time.time()) + int(token_data.get('expires_in', 3600)),
                            'email': email,
                            'mode': 'oauth',
                            'connected': True,
                            'connected_at': datetime.datetime.now().isoformat()
                        })

                        try:
                            cfg = get_config(data_root)
                            cfg['preferredMode'] = 'oauth'
                            save_config(data_root, cfg)
                        except Exception:
                            pass

                        html = """<!DOCTYPE html><html><head><meta charset="utf-8"><title>Notely — Google Calendar</title>
                        <style>body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#f6f9f7;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;}
                        .card{background:#fff;padding:40px 36px;border-radius:20px;box-shadow:0 12px 40px rgba(0,0,0,0.07);text-align:center;max-width:440px;border:1px solid #e2ebe6;}
                        .icon{width:64px;height:64px;border-radius:50%;background:#e8f5ee;color:#18786c;display:grid;place-items:center;margin:0 auto 16px;font-size:30px;font-weight:bold;}
                        h2{color:#1a3328;margin:0 0 8px;font-size:22px;}p{color:#64786d;font-size:14px;line-height:1.6;margin:0 0 20px;}
                        .btn{background:#18786c;color:#fff;border:none;padding:10px 24px;border-radius:8px;font-weight:600;font-size:14px;cursor:pointer;text-decoration:none;display:inline-block;}
                        </style></head><body>
                        <div class="card"><div class="icon">✓</div><h2>Berhasil Terhubung!</h2>
                        <p>Notely kini terhubung resmi dengan Google Calendar Anda. Jadwal tenggat tugas akan otomatis tersinkron.</p>
                        <a href="javascript:window.close()" class="btn">Tutup Halaman Ini</a></div>
                        <script>setTimeout(() => { try { window.close(); } catch(e){} }, 2000);</script></body></html>"""
                        self.send_response(200)
                        self.send_header('Content-Type', 'text/html; charset=utf-8')
                        self.end_headers()
                        self.wfile.write(html.encode('utf-8'))
                    except Exception as exc:
                        self.send_response(500)
                        self.send_header('Content-Type', 'text/html; charset=utf-8')
                        self.end_headers()
                        err_html = f"<html><body><h3>Gagal menukar token: {exc}</h3></body></html>"
                        self.wfile.write(err_html.encode('utf-8'))
                else:
                    self.send_response(400)
                    self.send_header('Content-Type', 'text/html; charset=utf-8')
                    self.end_headers()
                    self.wfile.write(f"<html><body><h3>Otorisasi gagal atau dibatalkan: {error}</h3></body></html>".encode('utf-8'))

                # Shutdown loopback server in background
                def shutdown_soon():
                    time.sleep(1)
                    with _auth_state_lock:
                        if _loopback_server:
                            _loopback_server.shutdown()
                threading.Thread(target=shutdown_soon, daemon=True).start()

        _loopback_server = HTTPServer(('127.0.0.1', LOOPBACK_PORT), OAuthCallbackHandler)
        _loopback_thread = threading.Thread(target=_loopback_server.serve_forever, daemon=True)
        _loopback_thread.start()

    params = {
        'client_id': c_id,
        'redirect_uri': REDIRECT_URI,
        'response_type': 'code',
        'scope': 'https://www.googleapis.com/auth/calendar.events https://www.googleapis.com/auth/userinfo.email',
        'access_type': 'offline',
        'prompt': 'consent'
    }
    return f"https://accounts.google.com/o/oauth2/v2/auth?{urllib.parse.urlencode(params)}"

def sync_event_to_google_calendar(data_root: str | Path, item: dict) -> dict:
    """Create or update a deadline event directly in Google Calendar via Google Calendar API or direct web sync."""
    config = get_config(data_root)
    tokens = get_tokens(data_root)
    deadline_date = item.get('deadline')
    if not deadline_date:
        raise ValueError("Tanggal deadline tidak ditemukan.")
    time_str = item.get('time') or '23:59'

    title = item.get('title', 'Tenggat Tugas')
    course = item.get('course', '')
    summary = f"{title} [{course}]" if course else title
    desc = f"{item.get('description', '')}\n\nMata Kuliah / Kategori: {course or '-'}\nPrioritas: {item.get('priority', 'sedang').upper()}\n\n[Dibuat & Dikelola via Notely Desktop]".strip()

    # Pre-build direct Google Calendar URL
    clean_date = deadline_date.replace('-', '')
    if time_str and time_str != '23:59':
        clean_time = time_str.replace(':', '') + '00'
        parts = time_str.split(':')
        h = min(23, int(parts[0]) + 1)
        end_time = f"{h:02d}{parts[1]}00"
        dates_param = f"{clean_date}T{clean_time}/{clean_date}T{end_time}"
    elif time_str == '23:59':
        dates_param = f"{clean_date}T230000/{clean_date}T235900"
    else:
        dt = datetime.datetime.strptime(deadline_date, '%Y-%m-%d')
        next_dt = dt + datetime.timedelta(days=1)
        dates_param = f"{clean_date}/{next_dt.strftime('%Y%m%d')}"

    now_local = datetime.datetime.now().astimezone()
    tz_name = getattr(now_local.tzinfo, 'key', None) or 'Asia/Jakarta'

    web_url = f"https://calendar.google.com/calendar/render?action=TEMPLATE&text={urllib.parse.quote(summary)}&dates={dates_param}&ctz={urllib.parse.quote(tz_name)}&details={urllib.parse.quote(desc)}"

    connected = bool(tokens.get('refresh_token') or (tokens.get('mode') == 'oauth' and tokens.get('access_token')))
    # If OAuth tokens are available, always sync via Google Calendar API v3 in the background
    if connected:
        try:
            token = refresh_access_token_if_needed(data_root)
            tz_offset = now_local.strftime('%z') or "+07:00"
            tz_formatted = f"{tz_offset[:3]}:{tz_offset[3:]}" if len(tz_offset) >= 5 else "+07:00"

            if time_str:
                parts = time_str.split(':')
                h = int(parts[0]) if parts else 23
                m = int(parts[1]) if len(parts) > 1 else 59
                
                # The specified time is the DEADLINE (the end time of the task)
                end_dt = f"{deadline_date}T{h:02d}:{m:02d}:00{tz_formatted}"
                
                # Start time is 30 minutes before the deadline for clear context
                total_min = h * 60 + m
                start_min = max(0, total_min - 30)
                start_h = start_min // 60
                start_m = start_min % 60
                start_dt = f"{deadline_date}T{start_h:02d}:{start_m:02d}:00{tz_formatted}"

                # Reminder notification setting from item
                rem_val = item.get('reminder')
                if rem_val is None:
                    rem_val = item.get('reminder_minutes', 30)
                try:
                    rem_mins = int(rem_val)
                except (ValueError, TypeError):
                    rem_mins = 30

                if rem_mins == 0:
                    reminders_body = {'useDefault': False, 'overrides': []}
                elif rem_mins == -1:
                    reminders_body = {'useDefault': True}
                else:
                    reminders_body = {'useDefault': False, 'overrides': [{'method': 'popup', 'minutes': rem_mins}]}

                event_body = {
                    'summary': f"[Tenggat] {summary}",
                    'description': f"Batas Waktu Pengumpulan (Deadline): {deadline_date} pukul {time_str}\n\n{desc}",
                    'start': {'dateTime': start_dt},
                    'end': {'dateTime': end_dt},
                    'reminders': reminders_body
                }
            else:
                dt = datetime.datetime.strptime(deadline_date, '%Y-%m-%d')
                next_dt = dt + datetime.timedelta(days=1)

                rem_val = item.get('reminder')
                if rem_val is None:
                    rem_val = item.get('reminder_minutes', 1440)
                try:
                    rem_mins = int(rem_val)
                except (ValueError, TypeError):
                    rem_mins = 1440

                if rem_mins == 0:
                    reminders_body = {'useDefault': False, 'overrides': []}
                elif rem_mins == -1:
                    reminders_body = {'useDefault': True}
                else:
                    reminders_body = {'useDefault': False, 'overrides': [{'method': 'popup', 'minutes': rem_mins}]}

                event_body = {
                    'summary': f"[Tenggat] {summary}",
                    'description': f"Batas Waktu Pengumpulan (Deadline): {deadline_date}\n\n{desc}",
                    'start': {'date': deadline_date},
                    'end': {'date': next_dt.strftime('%Y-%m-%d')},
                    'reminders': reminders_body
                }

            existing_gcal_id = item.get('gcal_event_id')
            method = 'PATCH' if (existing_gcal_id and not str(existing_gcal_id).startswith('direct_')) else 'POST'
            url = f"https://www.googleapis.com/calendar/v3/calendars/primary/events/{existing_gcal_id}" if method == 'PATCH' else "https://www.googleapis.com/calendar/v3/calendars/primary/events"

            max_retries = 4
            backoff = 1.5
            for attempt in range(max_retries):
                try:
                    req = urllib.request.Request(
                        url,
                        data=json.dumps(event_body).encode('utf-8'),
                        headers={'Authorization': f'Bearer {token}', 'Content-Type': 'application/json'},
                        method=method
                    )
                    with urllib.request.urlopen(req, timeout=15) as resp:
                        res = json.loads(resp.read().decode('utf-8'))
                    return {
                        'success': True,
                        'gcal_event_id': res.get('id', ''),
                        'htmlLink': res.get('htmlLink') or 'https://calendar.google.com'
                    }
                except urllib.error.HTTPError as he:
                    if he.code == 404 and method == 'PATCH':
                        # Fallback to creating a new event if the previous ID was deleted
                        post_req = urllib.request.Request(
                            "https://www.googleapis.com/calendar/v3/calendars/primary/events",
                            data=json.dumps(event_body).encode('utf-8'),
                            headers={'Authorization': f'Bearer {token}', 'Content-Type': 'application/json'},
                            method='POST'
                        )
                        with urllib.request.urlopen(post_req, timeout=15) as p_resp:
                            res = json.loads(p_resp.read().decode('utf-8'))
                        return {
                            'success': True,
                            'gcal_event_id': res.get('id', ''),
                            'htmlLink': res.get('htmlLink') or 'https://calendar.google.com'
                        }
                    err_text = he.read().decode('utf-8', errors='ignore')
                    if (he.code in (403, 429) or 'rateLimitExceeded' in err_text) and attempt < max_retries - 1:
                        time.sleep(backoff)
                        backoff *= 2.0
                        continue
                    if 'rateLimitExceeded' in err_text or he.code == 429:
                        raise RuntimeError("Batas panggilan Google Calendar sementara penuh (Rate Limit). Notely telah mencoba ulang otomatis.")
                    raise RuntimeError(f"Google Calendar API Error (HTTP {he.code}): {err_text or str(he)}")
        except Exception as exc:
            raise RuntimeError(f"Gagal sinkronisasi Google Calendar: {exc}")

    # Direct mode without API key / consumer mode
    event_id = item.get('gcal_event_id') or f"direct_{int(time.time()*1000)}"
    return {
        'success': True,
        'gcal_event_id': event_id,
        'htmlLink': web_url
    }

def delete_event_from_google_calendar(data_root: str | Path, gcal_event_id: str):
    """Safely delete an event from user's Google Calendar if synced."""
    if not gcal_event_id or str(gcal_event_id).startswith('direct_') or str(gcal_event_id).startswith('demo_'):
        return
    tokens = get_tokens(data_root)
    connected = bool(tokens.get('refresh_token') or (tokens.get('mode') == 'oauth' and tokens.get('access_token')))
    if not connected:
        return
    try:
        token = refresh_access_token_if_needed(data_root)
        url = f"https://www.googleapis.com/calendar/v3/calendars/primary/events/{gcal_event_id}"
        req = urllib.request.Request(url, headers={'Authorization': f'Bearer {token}'}, method='DELETE')
        with urllib.request.urlopen(req, timeout=10):
            pass
    except Exception:
        pass

def String_pad(n: int) -> str:
    return str(n).zfill(2)

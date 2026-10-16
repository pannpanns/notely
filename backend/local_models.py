"""Ollama catalog and byte-based, multi-layer download telemetry."""
import json
import time
import urllib.request
import urllib.error
from providers import ollama_base
from concurrent.futures import ThreadPoolExecutor

BASE = 'http://127.0.0.1:11434/api'
CATALOG = [
    {'id': 'qwen3.5:4b', 'name': 'Qwen 3.5', 'tier': '4B', 'estimatedBytes': 3_400_000_000, 'description': 'Pilihan utama untuk merangkum materi dan rekaman.'},
    {'id': 'qwen3.5:2b', 'name': 'Qwen 3.5 (2B)', 'tier': '2B', 'estimatedBytes': 2_700_000_000, 'description': 'Lebih ringan untuk perangkat dengan memori terbatas.'},
    {'id': 'qwen2.5:3b', 'name': 'Qwen 2.5', 'tier': '3B', 'estimatedBytes': 1_900_000_000, 'description': 'Alternatif kompatibel untuk instalasi sebelumnya.'},
    {'id': 'gemma3:4b', 'name': 'Gemma 3', 'tier': '4B', 'estimatedBytes': 3_300_000_000, 'description': 'Alternatif model lokal dari Google.'},
    {'id': 'qwen2.5:1.5b', 'name': 'Qwen 2.5 (1.5B)', 'tier': '1.5B', 'estimatedBytes': 1_000_000_000, 'description': 'Model kecil untuk kebutuhan ringkasan sederhana.'},
]
_cache = {}


def read_json(url, timeout=8):
    with urllib.request.urlopen(url, timeout=timeout) as response:
        return json.load(response)


def manifest(model):
    # Catalog is allowlisted. Never construct a remote URL from arbitrary input.
    if model not in {m['id'] for m in CATALOG}:
        return {}
    if model in _cache and time.monotonic() - _cache[model][0] < 3600:
        return _cache[model][1]
    name, tag = model.split(':', 1)
    url = f'https://registry.ollama.ai/v2/library/{name}/manifests/{tag}'
    req = urllib.request.Request(url, headers={'Accept': 'application/vnd.docker.distribution.manifest.v2+json'})
    with urllib.request.urlopen(req, timeout=6) as response:
        data = json.load(response)
    layers = {x['digest']: int(x['size']) for x in data.get('layers', []) + [data.get('config', {})] if x.get('digest') and x.get('size')}
    _cache[model] = (time.monotonic(), layers)
    return layers


def catalog():
    connected, error, installed = True, '', {}
    try:
        installed = {x['name']: x for x in read_json(ollama_base() + '/api/tags').get('models', [])}
    except Exception:
        connected, error = False, 'Ollama belum terhubung. Pasang dan jalankan Ollama, lalu periksa kembali.'
    def entry(model):
        result = dict(model, installed=model['id'] in installed, sizeBytes=model['estimatedBytes'], sizeSource='estimate')
        try:
            layers = manifest(model['id'])
            if layers:
                result.update(sizeBytes=sum(layers.values()), sizeSource='registry')
        except Exception:
            pass  # Offline catalog stays usable and estimates remain labelled.
        if model['id'] in installed:
            result.update(sizeBytes=installed[model['id']].get('size', result['sizeBytes']), sizeSource='installed')
        return result
    with ThreadPoolExecutor(max_workers=5) as pool:
        entries = list(pool.map(entry, CATALOG))
    for name, item in installed.items():
        if name not in {x['id'] for x in entries}:
            entries.append(dict(id=name, name=name, tier='Lokal', description='Model terpasang di Ollama.', installed=True, sizeBytes=item.get('size', 0), sizeSource='installed'))
    return {'connected': connected, 'message': error, 'models': entries}


class DownloadProgress:
    def __init__(self, model, layers=None, clock=time.monotonic):
        self.model, self.layers, self.done = model, dict(layers or {}), {}
        self.clock = clock
        self.last_time = clock()
        self.last_done, self.speed = 0, 0

    def feed(self, event):
        digest = event.get('digest')
        if digest and event.get('total'):
            self.layers[digest] = int(event['total'])
            value = max(0, int(event.get('completed', 0)))
            self.done[digest] = min(value, self.layers[digest])
        completed, total = sum(self.done.values()), sum(self.layers.values())
        now = self.clock()
        elapsed = now - self.last_time
        # First event of each layer may represent a resumed/cached blob, not traffic.
        if digest and digest == getattr(self, 'last_digest', None) and elapsed >= .2:
            self.speed = max(0, completed - self.last_done) / elapsed
        elif digest != getattr(self, 'last_digest', None):
            self.speed = 0
        if elapsed >= .2 or digest != getattr(self, 'last_digest', None):
            self.last_time, self.last_done = now, completed
        self.last_digest = digest
        success = event.get('status') == 'success'
        if success:
            completed = total
            self.speed = 0
        return {'message': event.get('status', 'Mengunduh model'), 'model': self.model,
                'downloadedBytes': completed, 'totalBytes': total,
                'bytesPerSecond': self.speed, 'sampledAt': time.time(),
                'percent': (100 if success else min(99.9, completed * 100 / total)) if total else None,
                'etaSeconds': max(0, total-completed) / self.speed if self.speed else None}


def pull_model(model, update):
    if model not in {x['id'] for x in CATALOG}:
        raise ValueError('Pilih model dari katalog unduhan lokal.')
    try:
        layers = manifest(model)
    except Exception:
        layers = {}
    tracker = DownloadProgress(model, layers)
    update(tracker.feed({'status': 'Menghubungkan ke Ollama'}))
    req = urllib.request.Request(ollama_base() + '/api/pull', data=json.dumps({'model': model, 'stream': True}).encode(), headers={'Content-Type': 'application/json'})
    success = False
    try:
        with urllib.request.urlopen(req, timeout=3600) as response:
            for line in response:
                if not line.strip():
                    continue
                event = json.loads(line)
                if event.get('error'):
                    raise RuntimeError(str(event['error'])[:250])
                update(tracker.feed(event))
                success = success or event.get('status') == 'success'
    except urllib.error.HTTPError as exc:
        code=exc.code;exc.close()
        if code==404:raise RuntimeError('Model tidak tersedia pada registry Ollama. Pilih model lain dari katalog.') from None
        raise RuntimeError(f'Ollama gagal mengunduh model (HTTP {code}). Periksa versi Ollama, koneksi dan ruang disk.') from None
    except (urllib.error.URLError, TimeoutError) as exc:
        raise RuntimeError('Unduhan terputus atau Ollama belum berjalan. Periksa koneksi dan coba lagi; Ollama dapat menggunakan data parsial.') from exc
    if not success:
        raise RuntimeError('Aliran unduhan terputus sebelum model selesai diverifikasi. Coba unduh kembali.')
    return {'model': model}


def delete_model(model):
    installed=read_json(ollama_base()+'/api/tags').get('models',[])
    if model not in {x.get('name') for x in installed}:raise ValueError('Model tidak ditemukan di penyimpanan aktif Notely.')
    req=urllib.request.Request(ollama_base()+'/api/delete',data=json.dumps({'model':model}).encode(),headers={'Content-Type':'application/json'},method='DELETE')
    with urllib.request.urlopen(req,timeout=60) as response:response.read()
    return {'deleted':model}

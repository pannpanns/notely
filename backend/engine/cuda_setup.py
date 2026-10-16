"""Optional on-demand NVIDIA CUDA DLL downloader (Windows x64, no pip/runtime Python required).

Packages are retrieved from PyPI's official JSON metadata, SHA-256 checked against
that metadata, unpacked into a staging directory and atomically committed. This
is intentionally separate from the installer and from the Python environment.
"""
from __future__ import annotations
import hashlib
import json
import os
import platform
import shutil
import sys
import tempfile
import urllib.request
import zipfile
from pathlib import Path

CUDA_PACKAGES = (
    ('nvidia-cublas-cu12', '12.4.5.8'),
    ('nvidia-cudnn-cu12', '9.5.1.17'),
    ('nvidia-cuda-runtime-cu12', '12.4.127'),
)
MARKER = 'notely_cuda_manifest.json'


def cuda_root() -> Path:
    if os.environ.get('NOTELY_MODEL_ROOT'):return Path(os.environ['NOTELY_MODEL_ROOT'])/'cuda'
    base = os.environ.get('LOCALAPPDATA') if sys.platform == 'win32' else None
    return Path(base or Path.home() / '.local' / 'share') / 'Notely' / 'cuda'


def cuda_installed(root: Path | None = None) -> bool:
    root = Path(root or cuda_root())
    marker = root / MARKER
    try:
        meta = json.loads(marker.read_text(encoding='utf-8'))
        return meta['packages'] == list(map(list, CUDA_PACKAGES)) and all(
            (root / relative).is_file() for relative in (
                'nvidia/cublas/bin/cublas64_12.dll',
                'nvidia/cublas/bin/cublasLt64_12.dll',
                'nvidia/cudnn/bin/cudnn64_9.dll',
                'nvidia/cuda_runtime/bin/cudart64_12.dll',
            )
        )
    except (OSError, ValueError, KeyError, TypeError):
        return False


def resolve_wheel(metadata: dict) -> dict:
    wheels = [f for f in metadata.get('urls', [])
              if f.get('filename', '').endswith('-py3-none-win_amd64.whl')]
    if len(wheels) != 1:
        raise RuntimeError('Paket CUDA x64 resmi tidak ditemukan untuk versi ini.')
    wheel = wheels[0]
    if not wheel['url'].startswith('https://files.pythonhosted.org/'):
        raise RuntimeError('Sumber paket NVIDIA tidak terpercaya.')
    digest = wheel.get('digests', {}).get('sha256', '')
    if len(digest) != 64:
        raise RuntimeError('Checksum paket NVIDIA tidak tersedia.')
    return wheel


def safe_extract_nvidia_wheel(wheel_path: Path, destination: Path):
    total = 0
    with zipfile.ZipFile(wheel_path) as z:
        for item in z.infolist():
            if item.is_dir():
                continue
            # Need only runtime DLL files, not headers or metadata.
            pieces = Path(item.filename.replace('\\', '/')).parts
            if (len(pieces) != 4 or pieces[0] != 'nvidia' or pieces[2] != 'bin'
                    or not pieces[3].lower().endswith('.dll')):
                continue
            if any(x in ('.', '..') for x in pieces):
                raise RuntimeError('Arsip CUDA memiliki nama file tidak aman.')
            total += item.file_size
            if total > 4 * 1024**3:
                raise RuntimeError('Ukuran ekstraksi CUDA tidak wajar.')
            out = destination.joinpath(*pieces)
            out.parent.mkdir(parents=True, exist_ok=True)
            with z.open(item) as src, out.open('wb') as dst:
                shutil.copyfileobj(src, dst)
    if not total:
        raise RuntimeError('Arsip CUDA tidak memuat file DLL Windows.')


def install_cuda(progress=None, root: Path | None = None,
                 platform_name: str | None = None) -> dict:
    platform_name = platform_name or sys.platform
    if platform_name != 'win32' or platform.machine().lower() not in ('amd64', 'x86_64'):
        raise RuntimeError('Akselerasi CUDA ini hanya tersedia untuk Windows x64.')
    root = Path(root or cuda_root())
    if cuda_installed(root):
        return {'message': 'Pustaka NVIDIA sudah ada.', 'path': str(root)}
    root.parent.mkdir(parents=True, exist_ok=True)
    required_space = 4 * 1024**3
    if shutil.disk_usage(root.parent).free < required_space:
        raise RuntimeError('Ruang kosong kurang dari 4 GB untuk proses unduh/ekstraksi CUDA.')
    staging = Path(tempfile.mkdtemp(prefix='cuda-download-', dir=root.parent))
    try:
        for index, (pkg, version) in enumerate(CUDA_PACKAGES, 1):
            if progress:progress(f'Komponen NVIDIA {index}/3: memeriksa {pkg}…')
            with urllib.request.urlopen(f'https://pypi.org/pypi/{pkg}/{version}/json', timeout=40) as response:
                info = json.load(response)
            wheel = resolve_wheel(info)
            expected_size = wheel.get('size', 0)
            if not (0 < expected_size < 2 * 1024**3):
                raise RuntimeError('Ukuran unduhan NVIDIA tidak wajar.')
            tmp = staging / f'{pkg}.whl'
            digest = hashlib.sha256()
            downloaded = 0
            req = urllib.request.Request(wheel['url'], headers={'User-Agent': 'Notely-Setup/2.3'})
            with urllib.request.urlopen(req, timeout=90) as response, tmp.open('wb') as output:
                while True:
                    block = response.read(2 * 1024 * 1024)
                    if not block:break
                    downloaded += len(block)
                    if downloaded > expected_size:
                        raise RuntimeError('Ukuran paket NVIDIA melebihi metadata resmi.')
                    digest.update(block); output.write(block)
                    if progress:progress(f'NVIDIA {index}/3 · {downloaded/1048576:.0f}/{expected_size/1048576:.0f} MB')
            if downloaded != expected_size or digest.hexdigest().lower() != wheel['digests']['sha256'].lower():
                raise RuntimeError('Checksum unduhan NVIDIA tidak cocok. Tidak akan memasang paket.')
            safe_extract_nvidia_wheel(tmp, staging)
            tmp.unlink()
        (staging / MARKER).write_text(json.dumps({'packages': list(map(list, CUDA_PACKAGES))}), encoding='utf-8')
        if not cuda_installed(staging):
            raise RuntimeError('Paket CUDA terunduh tetapi ada DLL yang tidak lengkap.')
        if root.exists():
            backup = root.with_name('cuda-previous')
            if backup.exists(): shutil.rmtree(backup)
            root.rename(backup)
            try:staging.rename(root)
            except Exception:
                backup.rename(root)
                raise
            shutil.rmtree(backup, ignore_errors=True)
        else:staging.rename(root)
        if progress:progress('Pustaka NVIDIA siap. Menguji GPU saat memuat model…')
        return {'message':'Pustaka CUDA terpasang dan diverifikasi. Aktivasi GPU menunggu uji inferensi.', 'path':str(root)}
    finally:
        if staging.exists():shutil.rmtree(staging,ignore_errors=True)

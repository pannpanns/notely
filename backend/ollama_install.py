"""Verified, resumable installation from pinned official release assets."""
import hashlib
import http.client
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import tempfile
import time
import urllib.error
import urllib.request
import zipfile
from ollama_release import RELEASE


def download_archive(url, archive, expected, progress):
    """Keep partial bytes on transient failures; accept resume only at exact offset."""
    archive=Path(archive)
    for attempt in range(4):
        offset=archive.stat().st_size if archive.exists() else 0
        if offset==expected:return
        if offset>expected:archive.unlink();offset=0
        req=urllib.request.Request(url,headers={'User-Agent':'Notely/2.7','Accept-Encoding':'identity',**({'Range':f'bytes={offset}-'} if offset else {})})
        try:
            with urllib.request.urlopen(req,timeout=60) as response:
                status=getattr(response,'status',200)
                if status==206:
                    content_range=response.headers.get('Content-Range','')
                    match=re.fullmatch(r'bytes (\d+)-(\d+)/(\d+)',content_range)
                    if not match or int(match[1])!=offset or int(match[3])!=expected:
                        raise RuntimeError('Server mengembalikan rentang unduhan tidak valid. Coba lagi nanti.')
                elif status==200:offset=0
                else:raise RuntimeError(f'Server unduhan memberikan HTTP {status}.')
                done=offset;last=time.monotonic();last_done=done
                with archive.open('ab' if offset else 'wb') as fp:
                    while True:
                        block=response.read(1024*1024)
                        if not block:break
                        if done+len(block)>expected:
                            raise RuntimeError('Ukuran komponen tidak sesuai metadata resmi.')
                        fp.write(block);done+=len(block)
                        now=time.monotonic();dt=now-last
                        speed=(done-last_done)/dt if dt>0 else 0
                        progress({'message':'Mengunduh mesin AI lokal otomatis','downloadedBytes':done,'totalBytes':expected,'percent':min(99,done*100/expected),'bytesPerSecond':speed,'etaSeconds':(expected-done)/speed if speed else None,'sampledAt':time.time()})
                        last=now;last_done=done
                if done!=expected:raise OSError('Unduhan terputus sebelum selesai.')
                return
        except (urllib.error.URLError, OSError, http.client.HTTPException) as exc:
            code=getattr(exc,'code',None)
            if code==416:
                archive.unlink(missing_ok=True)
            elif code and code not in (403,408,429,500,502,503,504):
                raise RuntimeError(f'Unduhan komponen gagal (HTTP {code}). Coba lagi nanti.') from None
            if attempt==3:
                raise RuntimeError('Unduhan komponen belum selesai. Data parsial disimpan; klik Coba lagi. Periksa koneksi atau akses ke GitHub Releases.') from None
            retry_after=getattr(exc,'headers',{}).get('Retry-After','') if getattr(exc,'headers',None) else ''
            wait=min(30,int(retry_after)) if retry_after.isdigit() else 2**(attempt+1)
            progress(f'Koneksi unduhan tertunda. Melanjutkan otomatis dalam {wait} detik ({attempt+1}/3)…')
            time.sleep(wait)


def install_runtime(root, progress=lambda value:None):
    if platform.system()!='Windows' or platform.machine().lower() not in ('amd64','x86_64'):
        raise RuntimeError('Unduhan otomatis AI lokal memerlukan Windows x64.')
    root=Path(root);target=root/'runtime'/'ollama'
    if (target/'ollama.exe').is_file():return str(target/'ollama.exe')
    target.parent.mkdir(parents=True,exist_ok=True)
    release=RELEASE;url=release['url'];digest=release['sha256'];expected=release['size']
    if not url.startswith('https://github.com/ollama/ollama/releases/download/') or not re.fullmatch('[a-f0-9]{64}',digest) or not isinstance(expected,int) or expected<=0:
        raise RuntimeError('Metadata komponen tidak valid. Gunakan paket Notely resmi terbaru.')
    archive=target.parent/f"ollama-{release['version']}.zip.part"
    progress('Menyiapkan komponen AI lokal Notely…')
    if shutil.disk_usage(root).free<expected*3-(archive.stat().st_size if archive.exists() else 0):
        raise RuntimeError('Ruang disk tidak cukup untuk mengunduh dan memasang komponen AI lokal.')
    download_archive(url,archive,expected,progress)
    progress('Memverifikasi checksum komponen…')
    sha=hashlib.sha256()
    with archive.open('rb') as fp:
        for block in iter(lambda:fp.read(1024*1024),b''):sha.update(block)
    if archive.stat().st_size!=expected or sha.hexdigest()!=digest:
        archive.unlink(missing_ok=True)
        raise RuntimeError('Verifikasi unduhan gagal. Klik Unduh untuk mencoba kembali.')
    with tempfile.TemporaryDirectory(prefix='.ollama-install-',dir=target.parent) as tmp:
        unpack=Path(tmp)/'unpacked';unpack.mkdir()
        progress('Memasang komponen AI lokal…')
        with zipfile.ZipFile(archive) as z:
            if shutil.disk_usage(root).free<sum(x.file_size for x in z.infolist())+100_000_000:
                raise RuntimeError('Ruang disk tidak cukup untuk mengekstrak komponen.')
            for entry in z.infolist():
                name=PurePosixPath(entry.filename.replace('\\','/'))
                if name.is_absolute() or '..' in name.parts or ':' in str(name) or (entry.external_attr>>16)&0o170000==0o120000:
                    raise RuntimeError('Arsip komponen berisi jalur tidak aman.')
            z.extractall(unpack)
        if not (unpack/'ollama.exe').is_file():raise RuntimeError('Komponen Ollama tidak lengkap.')
        (unpack/'notely-release.json').write_text(json.dumps({'version':release['version'],'sha256':digest}))
        if target.exists():shutil.rmtree(target)
        unpack.replace(target)
    archive.unlink(missing_ok=True)
    return str(target/'ollama.exe')

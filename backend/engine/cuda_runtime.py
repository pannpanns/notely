"""Register official NVIDIA pip-wheel DLLs before importing CTranslate2 on Windows.

This does not claim GPU availability: it only makes CUDA 12/cuDNN 9 libraries
loadable. A genuine Whisper inference is still required to confirm activation.
"""
from __future__ import annotations

import ctypes
import os
import subprocess
import sys
import sysconfig
from pathlib import Path

# Known-compatible CUDA-12.4 and cuDNN-9 wheels with win_amd64 builds.
CUDA_WHEELS = [
    'nvidia-cublas-cu12==12.4.5.8',
    'nvidia-cudnn-cu12==9.5.1.17',
    'nvidia-cuda-runtime-cu12==12.4.127',
]
DLL_NAMES = ('cudart64_12.dll', 'cublas64_12.dll', 'cublasLt64_12.dll',
             'cudnn64_9.dll', 'cudnn_ops64_9.dll', 'cudnn_cnn64_9.dll')
_dll_dir_handles = []
_dll_handles = []


def find_cuda_bin_dirs(site_packages=None):
    roots = [Path(site_packages or sysconfig.get_paths()['purelib'])]
    external = os.environ.get('NOTELY_CUDA_ROOT')
    if external:
        roots.insert(0, Path(external))
    if getattr(sys, 'frozen', False):
        roots.insert(0, Path(getattr(sys, '_MEIPASS', Path(sys.executable).parent)))
    paths = []
    for root in roots:
        nvidia = root / 'nvidia'
        if nvidia.is_dir():
            paths.extend(path for path in sorted(nvidia.glob('*/bin')) if path.is_dir())
    return list(dict.fromkeys(paths))


def register_cuda_dlls(site_packages=None, platform=None):
    """Idempotently add installed NVIDIA wheel directories to Windows loader.

    Returns (directories, missing_file_names). No global DLL copying and no
    changes to the machine/system PATH. On non-Windows, this is a safe no-op.
    """
    platform = platform or sys.platform
    if platform != 'win32':
        return [], []
    paths = find_cuda_bin_dirs(site_packages)
    found = {f.name.lower(): f for path in paths for f in path.glob('*.dll')}
    # Dynamic CT2 LoadLibrary uses PATH; Python extension dependencies use
    # add_dll_directory. Both paths must be configured before import.
    existing = os.environ.get('PATH', '').split(os.pathsep)
    prefix = [str(p) for p in paths if str(p) not in existing]
    if prefix:
        os.environ['PATH'] = os.pathsep.join(prefix + existing)
    for path in paths:
        try:
            if hasattr(os, 'add_dll_directory') and str(path) not in [p for p, _ in _dll_dir_handles]:
                _dll_dir_handles.append((str(path), os.add_dll_directory(str(path))))
        except OSError:
            pass
    # Preload in dependency order. CT2 may use native LoadLibrary outside
    # Python's own loader search rules.
    already = {name for name, _ in _dll_handles}
    for name in DLL_NAMES:
        dll = found.get(name.lower())
        if dll and name not in already:
            try:
                _dll_handles.append((name, ctypes.WinDLL(str(dll))))
            except OSError:
                # This is diagnostic, not proof of failure; inference decides.
                pass
    missing = [name for name in DLL_NAMES if name.lower() not in found]
    return [str(p) for p in paths], missing


def dll_report():
    if sys.platform != 'win32':
        return 'CUDA DLL Windows hanya berlaku pada Windows.'
    dirs, missing = register_cuda_dlls()
    return ('Folder CUDA pip: ' + ('; '.join(dirs) or 'belum ditemukan') + '\n'
            + ('DLL NVIDIA belum ditemukan dalam lingkungan Python: ' + ', '.join(missing)
               if missing else 'DLL NVIDIA tersedia dalam folder pip (inferensi CUDA tetap harus diuji).'))


def install_cuda_wheels(on_line=None):
    """Explicit opt-in installation; call from a background thread only."""
    if sys.platform != 'win32':
        raise RuntimeError('Paket CUDA RTX ini ditujukan untuk Windows 64-bit.')
    cmd = [sys.executable, '-m', 'pip', 'install', '--prefer-binary', *CUDA_WHEELS]
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                            text=True, encoding='utf-8', errors='replace', bufsize=1,
                            creationflags=subprocess.CREATE_NO_WINDOW if sys.platform=='win32' else 0)
    tail = []
    try:
        for line in proc.stdout:
            line = line.rstrip()
            if line:
                tail.append(line)
                if len(tail) > 40:
                    tail.pop(0)
                if on_line:
                    on_line(line)
        result = proc.wait()
    finally:
        if proc.poll() is None:
            proc.kill()
    if result:
        raise RuntimeError('Instalasi CUDA gagal (kode %d):\n%s' % (result, '\n'.join(tail[-9:])))
    register_cuda_dlls()
    return '\n'.join(tail[-9:])

"""Launch an isolated Ollama service; never reconfigure another app's Ollama."""
import atexit
import ctypes
from ctypes import wintypes
import os
from pathlib import Path
import shutil
import socket
import subprocess
import threading
import time
import urllib.request

_lock=threading.RLock()
_child=None
_job=None

def _own_process(process):
    # Closing this engine's job handle also stops Ollama and its inference workers,
    # including forced termination of the desktop/engine (no atexit required).
    global _job
    if os.name!='nt':return
    class Basic(ctypes.Structure):
        _fields_=[('process_time',ctypes.c_longlong),('job_time',ctypes.c_longlong),('flags',wintypes.DWORD),('minimum',ctypes.c_size_t),('maximum',ctypes.c_size_t),('active',wintypes.DWORD),('affinity',ctypes.c_size_t),('priority',wintypes.DWORD),('scheduling',wintypes.DWORD)]
    class IO(ctypes.Structure):
        _fields_=[(n,ctypes.c_ulonglong) for n in ('read','write','other','read_bytes','write_bytes','other_bytes')]
    class Extended(ctypes.Structure):
        _fields_=[('basic',Basic),('io',IO),('process_memory',ctypes.c_size_t),('job_memory',ctypes.c_size_t),('peak_process',ctypes.c_size_t),('peak_job',ctypes.c_size_t)]
    k=ctypes.WinDLL('kernel32',use_last_error=True)
    k.CreateJobObjectW.argtypes=[ctypes.c_void_p,wintypes.LPCWSTR];k.CreateJobObjectW.restype=wintypes.HANDLE
    k.SetInformationJobObject.argtypes=[wintypes.HANDLE,ctypes.c_int,ctypes.c_void_p,wintypes.DWORD];k.SetInformationJobObject.restype=wintypes.BOOL
    k.AssignProcessToJobObject.argtypes=[wintypes.HANDLE,wintypes.HANDLE];k.AssignProcessToJobObject.restype=wintypes.BOOL
    k.CloseHandle.argtypes=[wintypes.HANDLE];k.CloseHandle.restype=wintypes.BOOL
    handle=k.CreateJobObjectW(None,None);limits=Extended();limits.basic.flags=0x2000
    if not handle or not k.SetInformationJobObject(handle,9,ctypes.byref(limits),ctypes.sizeof(limits)) or not k.AssignProcessToJobObject(handle,int(process._handle)):
        if handle:k.CloseHandle(handle)
        process.terminate();raise RuntimeError('Tidak dapat mengelola proses Ollama dengan aman. Mulai ulang Notely.')
    _job=handle

def stop():
    global _child,_job
    with _lock:
        if _child is not None and _child.poll() is None:
            _child.terminate()
            try:_child.wait(timeout=10)
            except subprocess.TimeoutExpired:_child.kill();_child.wait(timeout=5)
        _child=None
        if _job is not None:
            k=ctypes.WinDLL('kernel32');k.CloseHandle.argtypes=[wintypes.HANDLE];k.CloseHandle(_job);_job=None
        os.environ.pop('NOTELY_OLLAMA_URL',None)

atexit.register(stop)
def ensure(progress=lambda value:None, install=False):
    global _child
    with _lock:
        if _child is not None and _child.poll() is None:return
        stop()
        root=os.environ.get('NOTELY_MODEL_ROOT')
        if not root:raise RuntimeError('Pilih folder penyimpanan model sebelum menggunakan AI lokal.')
        executable=str(Path(root)/'runtime'/'ollama'/'ollama.exe')
        if not Path(executable).is_file():
            if not install:raise RuntimeError('Klik Unduh. Notely akan memasang mesin AI lokal secara otomatis.')
            from ollama_install import install_runtime
            executable=install_runtime(root,progress)
        progress('Menjalankan mesin AI lokal…')
        with socket.socket() as s:
            s.bind(('127.0.0.1',0));port=s.getsockname()[1]
        env=dict(os.environ,OLLAMA_HOST=f'127.0.0.1:{port}',OLLAMA_MODELS=str(Path(root)/'ollama'),OLLAMA_NO_CLOUD='1')
        _child=subprocess.Popen([executable,'serve'],env=env,stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
        _own_process(_child)
        base=f'http://127.0.0.1:{port}'
        opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
        for _ in range(80):
            if _child.poll() is not None:break
            try:
                with opener.open(base+'/api/version',timeout=.5):pass
                os.environ['NOTELY_OLLAMA_URL']=base;return
            except OSError:time.sleep(.25)
        stop();raise RuntimeError('Ollama gagal dimulai. Periksa instalasi Ollama dan izin folder model.')

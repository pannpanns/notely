"""Launch an isolated Ollama service; never reconfigure another app's Ollama."""
import atexit
import csv
import ctypes
import json
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
_log_handle=None

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

def _log_path():
    root=os.environ.get('NOTELY_MODEL_ROOT')
    return Path(root)/'runtime'/'ollama'/'ollama.log' if root else None


def _request_json(url, body=None, timeout=5):
    data=json.dumps(body,ensure_ascii=False).encode('utf-8') if body is not None else None
    req=urllib.request.Request(url,data=data,headers={'Content-Type':'application/json'},method='POST' if body is not None else 'GET')
    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with opener.open(req,timeout=timeout) as response:
        return json.loads(response.read(2_000_000))


def nvidia_status():
    """Return NVIDIA driver telemetry when nvidia-smi is available."""
    candidates=[]
    found=shutil.which('nvidia-smi')
    if found:candidates.append(found)
    candidates.append(r'C:\Program Files\NVIDIA Corporation\NVSMI\nvidia-smi.exe')
    executable=next((item for item in candidates if Path(item).is_file()),None)
    if not executable:
        return {'detected':False,'message':'nvidia-smi tidak ditemukan. Ini belum membuktikan GPU tidak didukung.'}
    flags=getattr(subprocess,'CREATE_NO_WINDOW',0) if os.name=='nt' else 0
    try:
        result=subprocess.run([executable,'--query-gpu=name,driver_version,utilization.gpu,memory.used,memory.total,temperature.gpu','--format=csv,noheader,nounits'],capture_output=True,text=True,timeout=4,creationflags=flags,check=False)
        if result.returncode!=0:
            return {'detected':False,'message':(result.stderr or 'nvidia-smi gagal membaca GPU.').strip()[:180]}
        devices=[]
        for row in csv.reader(result.stdout.splitlines(),skipinitialspace=True):
            if len(row)<6:continue
            def number(value):
                try:return int(value.strip())
                except (TypeError,ValueError):return None
            devices.append({'name':row[0].strip(),'driver':row[1].strip(),'utilizationPercent':number(row[2]),'memoryUsedMiB':number(row[3]),'memoryTotalMiB':number(row[4]),'temperatureC':number(row[5])})
        return {'detected':bool(devices),'devices':devices,'message':'NVIDIA terdeteksi.' if devices else 'nvidia-smi tidak mengembalikan perangkat.'}
    except (OSError,subprocess.SubprocessError) as exc:
        return {'detected':False,'message':str(exc)[:180]}


def _loaded_models(base):
    data=_request_json(base+'/api/ps',timeout=5)
    models=[]
    for item in data.get('models',[]):
        size=int(item.get('size') or 0)
        vram=int(item.get('size_vram') or 0)
        if size and vram >= size*0.95:processor='GPU'
        elif vram>0:processor='CPU + GPU'
        else:processor='CPU'
        models.append({'name':item.get('name') or item.get('model') or 'Model tidak diketahui','sizeBytes':size,'vramBytes':vram,'processor':processor,'contextLength':item.get('context_length')})
    return models


def diagnostics():
    """Report actual Ollama model placement separately from Whisper CUDA status."""
    with _lock:
        child=_child
        running=child is not None and child.poll() is None
        base=os.environ.get('NOTELY_OLLAMA_URL','')
    gpu=nvidia_status()
    path=_log_path()
    result={'running':running,'nvidia':gpu,'logPath':str(path) if path else None,'models':[],'processor':'idle','version':None}
    if not running or not base:
        result['message']='Mesin Ollama Notely belum berjalan. Jalankan tes GPU atau buat ringkasan lokal terlebih dahulu.'
        return result
    try:
        version=_request_json(base+'/api/version',timeout=3)
        models=_loaded_models(base)
        result['version']=version.get('version')
        result['models']=models
        if not models:result['processor']='idle';result['message']='Ollama berjalan, tetapi tidak ada model yang sedang dimuat.'
        elif all(item['processor']=='GPU' for item in models):result['processor']='GPU';result['message']='Semua model yang aktif dimuat sepenuhnya di VRAM.'
        elif any(item['processor'] in ('GPU','CPU + GPU') for item in models):result['processor']='CPU + GPU';result['message']='Ollama memakai GPU, tetapi ada bagian model yang menggunakan RAM/CPU.'
        else:result['processor']='CPU';result['message']='Model aktif tidak memakai VRAM menurut laporan Ollama.'
    except Exception as exc:
        result['processor']='unknown'
        result['message']='Tidak dapat membaca status runtime Ollama: '+str(exc)[:180]
    return result


def test_gpu(model):
    """Run a tiny real inference, then inspect Ollama's own VRAM allocation."""
    model=str(model or '').strip()
    if not model or len(model)>160:raise ValueError('Pilih model Ollama yang valid.')
    ensure(install=False)
    base=os.environ.get('NOTELY_OLLAMA_URL','')
    if not base:raise RuntimeError('Alamat runtime Ollama Notely belum tersedia.')
    tags=_request_json(base+'/api/tags',timeout=8)
    available={item.get('name','') for item in tags.get('models',[]) if item.get('name')}
    if model not in available and model+':latest' not in available:
        raise RuntimeError('Model '+model+' belum terpasang. Klik Unduh model Ollama terlebih dahulu.')
    response=_request_json(base+'/api/chat',{
        'model':model,
        'messages':[{'role':'user','content':'Balas tepat satu kata: OK.'}],
        'stream':False,'keep_alive':'10m',
        'options':{'temperature':0,'num_ctx':4096,'num_predict':16}
    },timeout=240)
    reply=(response.get('message') or {}).get('content','').strip()
    if not reply:raise RuntimeError('Tes inferensi selesai tanpa teks. Periksa runtime Ollama dan log GPU.')
    result=diagnostics()
    result['test']={'ok':True,'reply':reply}
    return result


def stop():
    global _child,_job,_log_handle
    with _lock:
        if _child is not None and _child.poll() is None:
            _child.terminate()
            try:_child.wait(timeout=10)
            except subprocess.TimeoutExpired:_child.kill();_child.wait(timeout=5)
        _child=None
        if _job is not None:
            k=ctypes.WinDLL('kernel32');k.CloseHandle.argtypes=[wintypes.HANDLE];k.CloseHandle(_job);_job=None
        if _log_handle is not None:
            try:_log_handle.close()
            except OSError:pass
            _log_handle=None
        os.environ.pop('NOTELY_OLLAMA_URL',None)


def ensure(progress=lambda value:None, install=False):
    global _child,_log_handle
    with _lock:
        if _child is not None and _child.poll() is None:return
        stop()
        root=os.environ.get('NOTELY_MODEL_ROOT')
        if not root:raise RuntimeError('Pilih folder penyimpanan model sebelum menggunakan AI lokal.')
        executable=str(Path(root)/'runtime'/'ollama'/'ollama.exe')
        if not Path(executable).is_file():
            if not install:raise RuntimeError('Mesin AI lokal belum terpasang. Klik Unduh model Ollama untuk memasangnya.')
            from ollama_install import install_runtime
            executable=install_runtime(root,progress)
        progress('Menjalankan mesin AI lokal…')
        with socket.socket() as s:
            s.bind(('127.0.0.1',0));port=s.getsockname()[1]
        env=dict(os.environ,OLLAMA_HOST=f'127.0.0.1:{port}',OLLAMA_MODELS=str(Path(root)/'ollama'),OLLAMA_NO_CLOUD='1',OLLAMA_DEBUG='1')
        log_path=Path(executable).parent/'ollama.log'
        log_path.parent.mkdir(parents=True,exist_ok=True)
        try:
            if log_path.exists() and log_path.stat().st_size>20*1024*1024:
                previous=log_path.with_suffix('.log.1')
                previous.unlink(missing_ok=True)
                log_path.replace(previous)
        except OSError:pass
        _log_handle=log_path.open('ab',buffering=0)
        try:
            _child=subprocess.Popen([executable,'serve'],env=env,stdin=subprocess.DEVNULL,stdout=_log_handle,stderr=subprocess.STDOUT,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
            _own_process(_child)
        except Exception:
            if _log_handle is not None:
                _log_handle.close();_log_handle=None
            raise
        base=f'http://127.0.0.1:{port}'
        opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
        for _ in range(80):
            if _child.poll() is not None:break
            try:
                with opener.open(base+'/api/version',timeout=.5):pass
                os.environ['NOTELY_OLLAMA_URL']=base;return
            except OSError:time.sleep(.25)
        stop();raise RuntimeError('Ollama gagal dimulai. Periksa runtime/ollama/ollama.log untuk detail, termasuk deteksi CUDA NVIDIA.')

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

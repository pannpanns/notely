"""Private per-user settings. Windows API keys are protected with DPAPI."""
import base64
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import threading
import shutil
import hashlib
import tempfile
import stat

LOCK = threading.RLock()
def config_dir():
    return Path(os.environ.get('LOCALAPPDATA',str(Path.home()/'.local/share')))/'Notely'
def read():
    with LOCK:
        p=config_dir()/'preferences.json'
        return json.loads(p.read_text(encoding='utf-8')) if p.exists() else {}
def write(data):
    with LOCK:
        p=config_dir();p.mkdir(parents=True,exist_ok=True)
        temp=p/'preferences.tmp';temp.write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
        temp.replace(p/'preferences.json')

def protect(value, decrypt=False):
    if os.name!='nt':raise RuntimeError('Penyimpanan API key terenkripsi memerlukan Windows.')
    class Blob(ctypes.Structure):
        _fields_=[('size',wintypes.DWORD),('data',ctypes.POINTER(ctypes.c_ubyte))]
    buf=ctypes.create_string_buffer(value)
    source=Blob(len(value),ctypes.cast(buf,ctypes.POINTER(ctypes.c_ubyte)));dest=Blob()
    crypt=ctypes.WinDLL('crypt32',use_last_error=True)
    fn=crypt.CryptUnprotectData if decrypt else crypt.CryptProtectData
    fn.argtypes=[ctypes.POINTER(Blob),ctypes.c_void_p,ctypes.c_void_p,ctypes.c_void_p,ctypes.c_void_p,wintypes.DWORD,ctypes.POINTER(Blob)]
    fn.restype=wintypes.BOOL
    if not fn(ctypes.byref(source),None,None,None,None,1,ctypes.byref(dest)):
        raise RuntimeError('Windows tidak dapat membuka/menyimpan key untuk akun ini. Masukkan ulang key.')
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel.LocalFree.argtypes=[ctypes.c_void_p];kernel.LocalFree.restype=ctypes.c_void_p
    try:return ctypes.string_at(dest.data,dest.size)
    finally:kernel.LocalFree(dest.data)

def save_key(provider,key):
    if provider not in ('gemini','openai','claude','groq','openrouter','custom'):raise ValueError('Penyedia tidak valid.')
    with LOCK:
        data=read();keys=data.setdefault('protectedKeys',{})
        if key.strip():keys[provider]=base64.b64encode(protect(key.strip().encode())).decode()
        else:keys.pop(provider,None)
        write(data)

def public_settings():
    data=read();keys={}
    for provider,value in data.get('protectedKeys',{}).items():
        try:keys[provider]=protect(base64.b64decode(value),True).decode()
        except Exception:pass
    root=data.get('modelRoot','')
    if root and not (Path(root)/'.notely-owned').is_file():root=''
    return {'keys':keys,'modelRoot':root}

def select_folder(folder):
    path=Path(folder)
    if not path.is_absolute() or not path.is_dir():raise ValueError('Pilih folder yang ada dengan lokasi absolut.')
    root=path.resolve()/'NotelyModels'
    if root.is_symlink():raise ValueError('Folder model tidak boleh berupa tautan.')
    if root.exists() and not (root/'.notely-owned').is_file():raise ValueError('Folder NotelyModels sudah ada tanpa penanda kepemilikan. Pilih folder lain.')
    root.mkdir(exist_ok=True)
    (root/'.notely-owned').write_text('app.notely.desktop',encoding='utf-8')
    with LOCK:
        data=read();data['modelRoot']=str(root)
        data['modelRoots']=list(dict.fromkeys([*data.get('modelRoots',[]),str(root)]));write(data)
    activate_storage(root)
    return {'modelRoot':str(root)}

def activate_storage(root=None):
    root=root or read().get('modelRoot')
    if not root:
        base=Path(os.environ.get('NOTELY_INSTALL_DIR') or config_dir());base.mkdir(parents=True,exist_ok=True)
        return select_folder(str(base))
    if root:
        path=Path(root)
        if path.is_symlink() or not (path/'.notely-owned').is_file():raise ValueError('Folder model tidak tersedia. Sambungkan drive atau pilih folder kembali.')
        os.environ['NOTELY_MODEL_ROOT']=str(path)
        os.environ['NOTELY_HF_CACHE']=str(path/'transcription')
        os.environ['HF_HOME']=str(path/'huggingface')
        os.environ['HF_HUB_CACHE']=str(path/'transcription')
        os.environ['HF_XET_CACHE']=str(path/'huggingface'/'xet')
        os.environ['NOTELY_CUDA_ROOT']=str(path/'cuda')


def _link(path):
    return path.is_symlink() or bool(getattr(path.lstat(),"st_file_attributes",0)&getattr(stat,"FILE_ATTRIBUTE_REPARSE_POINT",1024)) if path.exists() or path.is_symlink() else False

def _owned(path):
    return (not _link(path) and (path/'.notely-owned').is_file()
            and (path/'.notely-owned').read_text(encoding='utf-8')=='app.notely.desktop')

def _digest(path):
    h=hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda:f.read(4*1024*1024),b''):h.update(b)
    return h.digest()

def migrate_storage(folder, progress=lambda message:None):
    """Copy and verify app-owned data before switching; delete only verified originals.

    Staging is always on the destination drive. Interrupted copies are retried safely.
    User files and external Ollama/HuggingFace caches are never removed.
    """
    base=Path(folder).resolve()
    if not base.is_dir():raise ValueError('Folder instalasi/model tidak tersedia.')
    destination=base/'NotelyModels'
    old=read().get('modelRoot')
    source=Path(old) if old else None
    if source and source.resolve()==destination.resolve():
        if not _owned(source):raise ValueError('Folder model tidak dimiliki Notely.')
        source=None
    if destination.exists() and not _owned(destination):raise ValueError('Folder tujuan bukan milik Notely. Pilih folder lain.')
    if source and source.exists() and not _owned(source):raise ValueError('Kepemilikan folder model lama tidak valid; pemindahan dibatalkan.')
    if source and not source.exists():raise ValueError('Drive model lama tidak tersedia. Sambungkan drive sebelum memindahkan model.')
    if source and (source.resolve() in destination.parents or destination.resolve() in source.resolve().parents):
        raise ValueError('Folder tujuan tidak boleh berada di dalam folder model lama, atau sebaliknya.')
    files=[]
    if source:
        for path in source.rglob('*'):
            # No traversal through junctions or external symbolic links.
            if _link(path):
                raise ValueError('Folder model mengandung tautan. Pemindahan aman memerlukan folder biasa.')
            if path.is_file() and path.name!='.notely-owned':files.append((path,path.relative_to(source)))
    legacy=config_dir()/'cuda'
    cleanup_roots=[source] if source else []
    manifest=legacy/'notely_cuda_manifest.json'
    if manifest.is_file() and legacy.resolve()!= (destination/'cuda').resolve():
        from engine.cuda_setup import CUDA_PACKAGES
        if _link(legacy) or json.loads(manifest.read_text()).get('packages')!=list(map(list,CUDA_PACKAGES)):
            raise ValueError('Paket CUDA lama tidak dapat diverifikasi; pemindahan dibatalkan.')
        for path in legacy.rglob('*'):
            if _link(path):raise ValueError('Paket CUDA lama mengandung tautan; pemindahan dibatalkan.')
            if path.is_file():files.append((path,Path('cuda')/path.relative_to(legacy)))
        cleanup_roots.append(legacy)
    total=sum(p.stat().st_size for p,_ in files)
    if shutil.disk_usage(base).free<total+64*1024*1024:raise ValueError('Ruang drive tujuan belum cukup untuk menyalin dan memverifikasi model.')
    destination.mkdir(exist_ok=True)
    (destination/'.notely-owned').write_text('app.notely.desktop',encoding='utf-8')
    copied=0
    for path,rel in files:
        target=destination/rel
        # Validate ancestors before writing into an existing destination.
        for ancestor in [target,*target.parents]:
            if ancestor==base:break
            if _link(ancestor):raise ValueError('Tujuan model mengandung tautan.')
        target.parent.mkdir(parents=True,exist_ok=True)
        progress({'message':f'Memindahkan model: {rel.name}', 'downloadedBytes':copied,'totalBytes':total})
        digest=_digest(path)
        if target.exists():
            if not target.is_file() or _digest(target)!=digest:raise ValueError('Ada berkas model berbeda di tujuan. Pilih folder kosong agar data tidak tertimpa.')
        else:
            fd,tmp=tempfile.mkstemp(prefix='.notely-copy-',dir=target.parent);os.close(fd)
            tmp=Path(tmp)
            try:
                shutil.copyfile(path,tmp)
                if _digest(tmp)!=digest:raise ValueError('Verifikasi salinan model gagal. Model lama tetap aman.')
                tmp.replace(target)
            finally:tmp.unlink(missing_ok=True)
        copied+=path.stat().st_size
    # Commit settings only after every file is verified. No destructive whole-tree removal.
    result=select_folder(str(base))
    if files:
        failures=False
        for path,rel in files:
            try:
                if _digest(path)==_digest(destination/rel):path.unlink()
                else:failures=True
            except OSError:failures=True
        for old_root in cleanup_roots:
            for path in sorted(old_root.rglob('*'),key=lambda p:len(p.parts),reverse=True):
                if path.is_dir():
                    try:path.rmdir()
                    except OSError:pass
            if not failures:
                try:
                    (old_root/'.notely-owned').unlink(missing_ok=True)
                    old_root.rmdir()
                except OSError:pass
        if failures:result['warning']='Lokasi model sudah diganti; sebagian berkas lama masih dipakai dan belum dapat dihapus.'
    return result

def use_install_storage(progress=lambda message:None):
    install=os.environ.get('NOTELY_INSTALL_DIR')
    if install and read().get('storagePolicy')!='custom':return migrate_storage(install,progress)
    activate_storage()
    return {'modelRoot':os.environ.get('NOTELY_MODEL_ROOT','')}

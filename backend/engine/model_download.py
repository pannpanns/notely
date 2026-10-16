"""Whisper model metadata and download reporting through the Hub's public API."""
import fnmatch
import os
import threading
import time
from pathlib import Path

REPOSITORIES = {
 'maleo-ai/whisper-small-id': ('maleo-ai/whisper-small-id', ['ct2-int8/*'], 'ct2-int8'),
 'maleo-ai/whisper-large-v3-turbo-id': ('maleo-ai/whisper-large-v3-turbo-id', ['ct2-int8/*'], 'ct2-int8'),
 'small': ('Systran/faster-whisper-small', None, ''),
 'medium': ('Systran/faster-whisper-medium', None, ''),
 'turbo': ('mobiuslabsgmbh/faster-whisper-large-v3-turbo', None, ''),
 'large-v3': ('Systran/faster-whisper-large-v3', None, ''),
}
PATTERNS = ['config.json','preprocessor_config.json','model.bin','tokenizer.json','vocabulary.*']
_metadata = {}


def metadata(model):
    if model in _metadata:
        return _metadata[model]
    from huggingface_hub import HfApi
    repo, patterns, subdir = REPOSITORIES[model]
    info = HfApi().model_info(repo, files_metadata=True, timeout=8)
    files = [f for f in info.siblings if any(fnmatch.fnmatch(f.rfilename,p) for p in (patterns or PATTERNS))]
    total = sum(f.size or 0 for f in files)
    _metadata[model] = {'sizeBytes':total, 'revision':info.sha}
    return _metadata[model]


def download(model, progress=None):
    from huggingface_hub import snapshot_download
    from tqdm.auto import tqdm
    repo, patterns, subdir = REPOSITORIES[model]
    cache=os.environ.get('NOTELY_HF_CACHE')
    patterns = patterns or PATTERNS
    try:
        folder=Path(snapshot_download(repo, cache_dir=cache, allow_patterns=patterns, local_files_only=True))/subdir
        if all((folder/name).exists() for name in ['model.bin','config.json','tokenizer.json']):
            return str(folder)
    except Exception:
        pass
    try:info=metadata(model)
    except Exception:info={'sizeBytes':0,'revision':None}
    lock=threading.RLock()
    report={'downloaded':0,'transfer':0,'lastTransfer':0,'time':time.monotonic(),'speed':0}
    class Progress(tqdm):
        def __init__(self,*args,**kwargs):
            self.byte_bar=kwargs.get('unit')=='B'
            self.transfer_bar='transfer' in kwargs.get('name','') or kwargs.get('desc')=='Downloading bytes'
            self.observed=kwargs.get('initial',0)
            kwargs.pop('name',None)
            kwargs['disable']=True
            super().__init__(*args,**kwargs)
        def update(self,n=1):
            self.observed+=n or 0
            if self.byte_bar and progress:
                with lock:
                    report['transfer' if self.transfer_bar else 'downloaded']=max(0,self.observed)
                    now=time.monotonic()
                    dt=now-report['time']
                    if dt>=.3:
                        report['speed']=max(0,report['transfer']-report['lastTransfer'])/dt
                        report['time'],report['lastTransfer']=now,report['transfer']
                        done=min(report['downloaded'],info['sizeBytes'])
                        total=info['sizeBytes'];speed=report['speed']
                        progress({'message':'Mengunduh model transkripsi','model':repo.split('/')[-1],
                                  'downloadedBytes':done,'totalBytes':total,'bytesPerSecond':speed,
                                  'percent':min(99.9,done*100/total) if total else None,
                                  'etaSeconds':(total-done)/speed if speed else None,'sampledAt':time.time()})
            return super().update(n)
    if progress:
        progress({'message':'Menyiapkan unduhan transkripsi','model':repo.split('/')[-1],
                  'downloadedBytes':0,'totalBytes':info['sizeBytes'],'percent':0})
    folder=Path(snapshot_download(repo,cache_dir=cache,revision=info['revision'],allow_patterns=patterns,tqdm_class=Progress))/subdir
    if not (folder/'model.bin').exists():
        raise RuntimeError('Unduhan model belum lengkap.')
    if progress:
        progress({'message':'Unduhan selesai. Menguji model…','model':repo.split('/')[-1],
                  'downloadedBytes':info['sizeBytes'],'totalBytes':info['sizeBytes'],'percent':100})
    return str(folder)

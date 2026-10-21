"""Loopback-only engine bridge for the Tauri UI. Shared bearer token is mandatory."""
from __future__ import annotations
import argparse, base64, hmac, json, os, queue, re, secrets, shutil, sys, threading, time, traceback, urllib.request, wave
from collections import deque
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE/'engine'))  # v1.8 engine modules use absolute local imports.
from engine.folder_store import Collections
from engine.audio_import import AUDIO_EXTENSIONS, normalize_audio
from engine.recording import RecordingEngine
from engine.audio_capture import enumerate_devices, AudioDevice, diagnose
from engine.model_runtime import ModelManager, MODEL_CHOICES, runtime_report, discover_nvidia
from engine.cuda_setup import install_cuda, cuda_root, cuda_installed
from engine.core import new_session, default_output_root, read_json, export_transcript, nice_seconds
from providers import PROVIDERS, models_for, summarize, test_connection
import preferences
import managed_ollama
from local_models import catalog, pull_model

class Service:
    def __init__(self,root):
        self.root=Path(root); self.root.mkdir(parents=True,exist_ok=True)
        self.folders=Collections(self.root)
        self.storage_ready=False;self.storage_error=''
        self.transcriber=ModelManager(); self.recording=None; self.lock=threading.RLock()
        self.events=deque(maxlen=1000); self.serial=0
        self.spectra={}; self.jobs={}; self.state='idle'; self.active_session=None
        self.preview={}; self.backend='Belum diuji'; self.queue_status=(0,0);self.transcript_count=0
        self.job('storage',lambda progress:self.prepare_storage(progress))
    def prepare_storage(self,progress,folder=None):
        self.storage_ready=False;self.storage_error=''
        try:
            managed_ollama.stop()
            result=preferences.migrate_storage(folder,progress) if folder else preferences.use_install_storage(progress)
            if folder:
                data=preferences.read();data['storagePolicy']='custom';preferences.write(data)
            self.transcriber=ModelManager();self.storage_ready=True
            return result
        except Exception as exc:
            self.storage_error=str(exc)
            raise
    def require_storage(self):
        if not self.storage_ready:
            raise ValueError(self.storage_error or 'Model sedang disiapkan/dipindahkan. Lihat progres Penyimpanan model di Pengaturan.')
    def send(self,event,value):
        with self.lock:
            if event=='spectrum':
                self.spectra[value['source']]=value
                return
            if event=='backend':self.backend=str(value)
            if event=='queue':self.queue_status=tuple(value)
            if event=='finished':
                self.state='idle'; self.active_session=str(value[0]);
                value={'session':value[0].name,'warnings':value[2]}
            if event=='preview':
                label,pid,start,txt=value
                self.preview[label]={'id':pid,'start':start,'text':txt}
            if event=='preview_clear':
                label,pid=value
                if label in self.preview and self.preview[label]['id']<=pid:self.preview.pop(label,None)
            if event=='transcript':
                raw=value
                value=[{'start':v.get('start',0),'end':v.get('end',0),'text':v.get('text','')} for v in raw[self.transcript_count:]]
                self.transcript_count=len(raw)
            if event=='volume':value={'source':value[0],'rms':value[1]}
            self.serial+=1
            self.events.append({'seq':self.serial,'event':event,'value':value})
    def sessions(self):
        results=[]
        for d in self.root.iterdir():
            if d.is_symlink() or d.resolve().parent != self.root.resolve() or not d.is_dir() or not (d/'metadata.json').exists():continue
            try:
                meta=read_json(d/'metadata.json',{})
                results.append({'id':d.name,'title':meta.get('judul','Rekaman'), 'date':meta.get('tanggal',''),
                                'status':meta.get('status',''), 'folder':meta.get('folder',''), 'audio':meta.get('audio',[]), **self.session_sizes(d)})
            except Exception:continue
        return sorted(results,key=lambda x:x['id'],reverse=True)
    @staticmethod
    def session_sizes(folder):
        total=audio=0
        for base, dirs, files in os.walk(folder, followlinks=False):
            dirs[:]=[d for d in dirs if not (Path(base)/d).is_symlink()]
            for name in files:
                p=Path(base)/name
                if p.is_symlink():continue
                try:size=p.stat().st_size
                except OSError:continue
                total+=size
                if p.suffix.lower() in ('.wav','.flac','.mp3','.m4a','.ogg'):audio+=size
        return {'sizeBytes':total,'audioBytes':audio}
    def delete_session(self,id,audio_only=False):
        with self.lock:
            if self.state!='idle' or any(j['status']=='running' for j in self.jobs.values()):
                raise ValueError('Selesaikan rekaman dan proses aktif sebelum menghapus.')
            p=self.session_dir(id)
            before=self.session_sizes(p)['sizeBytes']
            if audio_only:
                for audio in p.glob('audio_*.*'):
                    if audio.suffix.lower() in ('.wav','.flac','.mp3','.m4a','.ogg'):
                        audio.unlink()
                meta=read_json(p/'metadata.json',{});meta['audio']=[]
                (p/'metadata.json').write_text(json.dumps(meta,ensure_ascii=False),encoding='utf-8')
                freed=max(0,before-self.session_sizes(p)['sizeBytes'])
            else:
                shutil.rmtree(p);freed=before
                if self.active_session and Path(self.active_session).name==id:self.active_session=None
            return {'deleted':id,'audioOnly':audio_only,'freedBytes':freed}
    def delete_folder(self, folder):
        with self.lock:
            if self.state!='idle' or any(j['status']=='running' for j in self.jobs.values()):
                raise ValueError('Selesaikan proses aktif sebelum menghapus folder.')
            return self.folders.delete(folder)
    def update_session(self,args):
        from engine.core import atomic_json
        with self.lock:
            if self.state!='idle' or any(j['status']=='running' for j in self.jobs.values()):
                raise ValueError('Selesaikan proses aktif sebelum mengubah rekaman.')
            d=self.session_dir(args.get('id',''))
            meta=read_json(d/'metadata.json',{})
            if 'title' in args:meta['judul']=Collections.name(args['title'])
            if 'folder' in args:meta['folder']=self.folders.validate(args['folder'])
            atomic_json(d/'metadata.json',meta)
            return {'id':d.name,'title':meta.get('judul',''),'folder':meta.get('folder','')}
    def session_dir(self,id):
        if not id or '/' in id or '\\' in id or id in ('.','..'):raise ValueError('ID rekaman tidak valid.')
        p=self.root/id
        if p.is_symlink() or p.resolve().parent != self.root.resolve() or not p.is_dir() or not (p/'metadata.json').exists():raise ValueError('Rekaman tidak ditemukan.')
        return p
    def session(self,id):
        p=self.session_dir(id)
        meta=read_json(p/'metadata.json',{})
        seg=read_json(p/'segments.json',[]) or []
        edited=p/'transkrip_edit.txt'
        return {'id':id,'meta':meta,'segments':seg,**self.session_sizes(p),
                'transcript':edited.read_text(encoding='utf-8') if edited.exists() else (p/'transkrip.txt').read_text(encoding='utf-8') if (p/'transkrip.txt').exists() else '',
                'summaryOptions':read_json(p/'summary-options.json',{}),
                'summary':(p/'ringkasan_ai.md').read_text(encoding='utf-8') if (p/'ringkasan_ai.md').exists() else '',
                'audio':[x.name for x in p.glob('audio_*.*') if x.suffix in ('.wav','.flac')], 'edited':edited.exists()}
    def status(self,since=0):
        with self.lock:
            return {'state':self.state,'spectra':list(self.spectra.values()) if self.state=='recording' else [],'backend':self.backend,'session':Path(self.active_session).name if self.active_session else None,
                    'elapsed':int(time.monotonic()-self.recording.started) if self.state=='recording' and self.recording else None,
                    'queue':self.queue_status,'events':[e for e in self.events if e['seq']>since][-150:], 'latest':self.serial,
                    'preview':list(self.preview.values()),'jobs':{key:{'kind':j['kind'],'status':j['status'],'message':j.get('message',''), 'progress':j.get('progress'), 'result':j.get('result') if j['kind'] in ('model','audio_test','storage') else None} for key,j in list(self.jobs.items())[-70:]}}
    def start(self,args):
        with self.lock:
            if self.state!='idle':raise ValueError('Perekaman masih berlangsung atau sedang diselesaikan.')
            speakers,mics,_=enumerate_devices()
            device_id=int(args.get('speaker',-1))
            speaker=next((d for d in speakers if d.index==device_id),None)
            if speaker is None:raise ValueError('Pilih output speaker yang tersedia.')
            chosen=[('Peserta (komputer)',speaker)]
            if args.get('microphone') is not None:
                mic=next((d for d in mics if d.index==int(args['microphone'])),None)
                if mic is None:raise ValueError('Mikrofon tidak ditemukan.')
                chosen.append(('Mikrofon',mic))
            model=args.get('model',MODEL_CHOICES[1]); gpu=args.get('gpu','CUDA')
            if model not in MODEL_CHOICES or gpu not in ('CUDA','CPU','Otomatis'):raise ValueError('Model/akselerasi tidak valid.')
            folder=self.folders.validate(args.get('folder',''))
            session=new_session(self.root,str(args.get('title','Catatan transkripsi'))[:70])
            from engine.core import refresh_metadata
            refresh_metadata(session,folder=folder)
            engine=RecordingEngine(self,session,chosen,model,str(args.get('language','Indonesia')),gpu)
            self.recording=engine;self.active_session=str(session);self.state='recording';self.spectra={};self.preview={};self.transcript_count=0
            engine.start()
            return {'session':session.name}
    def stop(self):
        with self.lock:
            if self.state!='recording':raise ValueError('Belum ada rekaman aktif.')
            self.state='stopping';self.recording.stop()
        return {'message':'Perekaman dihentikan; transkrip sedang diselesaikan.'}
    def finish_partial(self):
        with self.lock:
            if self.state!='stopping':raise ValueError('Tidak ada proses penyelesaian rekaman.')
            ok=self.recording.finish_now()
        if not ok:raise ValueError('Tunggu hingga audio selesai ditutup.')
        return {'message':'Menyimpan transkrip parsial; audio asli dipertahankan.'}
    def job(self,kind,fn):
        jid=secrets.token_hex(7)
        if len(self.jobs)>80:
            for k in [k for k,j in self.jobs.items() if j['status']!='running'][:20]: self.jobs.pop(k,None)
        self.jobs[jid]={'kind':kind,'status':'running','message':'Memulai...'}
        def run():
            def update(x):
                with self.lock:
                    if isinstance(x, dict):
                        self.jobs[jid].update(message=str(x.get('message', 'Mengunduh...'))[:180], progress=x)
                    else:
                        self.jobs[jid]['message']=str(x)[:180]
            try:
                result=fn(update)
                self.jobs[jid].update(status='done',message='Selesai',result=result,progress=None)
            except Exception as e:
                self.jobs[jid].update(status='error',message=str(e)[:300])
        threading.Thread(target=run,daemon=True).start()
        return {'job':jid}
    def transcribe_model(self,choice,gpu):
        if self.state!='idle':raise ValueError('Hentikan dan selesaikan rekaman terlebih dahulu.')
        return self.job('model',lambda progress:self._prep_model(choice,gpu,progress))
    def _prep_model(self,choice,gpu,progress):
        progress('Mengunduh (jika perlu) dan menguji model...')
        model,backend,warning=self.transcriber.get(choice,gpu,progress)
        self.backend=backend
        return {'backend':backend,'warning':warning}
    def summarize(self,args):
        with self.lock:return self._summarize_locked(args)
    def _summarize_locked(self,args):
        if self.state!='idle':raise ValueError('Tunggu hingga rekaman dan transkripsi selesai.')
        if any(j['kind']=='summary' and j['status']=='running' for j in self.jobs.values()):
            raise ValueError('Ringkasan lain masih berjalan. Tunggu hingga selesai.')
        id=args.get('session'); p=self.session_dir(id)
        provider=args.get('provider','ollama'); model=str(args.get('model','')).strip()
        if provider not in PROVIDERS:raise ValueError('Penyedia tidak tersedia.')
        key=str(args.get('apiKey',''));context=str(args.get('context',''))
        instructions=str(args.get('instructions',''))
        (p/'summary-options.json').write_text(json.dumps({'instructions':instructions,'context':context},ensure_ascii=False),encoding='utf-8')
        endpoint=str(args.get('endpoint',''))
        transcript=self.session(id)['transcript']
        title=read_json(p/'metadata.json',{}).get('judul','Rekaman')
        def work(progress):
            if provider=='ollama':
                self.require_storage()
                managed_ollama.ensure(progress, install=True)
                if model not in models_for('ollama'):pull_model(model,progress)
            result=summarize(title,transcript,provider,model,key,endpoint,context,progress, bool(args.get('geminiFallback',True)),instructions=instructions,economy=args.get('economy') is True)
            (p/'ringkasan_ai.md').write_text(result,encoding='utf-8')
            return {'session':id,'summary':result}
        return self.job('summary',work)
    def import_audio(self,tmp,title,folder=''):
        with self.lock:
            if self.state!='idle':raise ValueError('Hentikan rekaman terlebih dahulu.')
            if Path(tmp).suffix.lower() not in AUDIO_EXTENSIONS:raise ValueError('Format impor tidak didukung.')
            folder=self.folders.validate(folder)
            self.state='importing'
        session=None
        try:
            session=new_session(self.root,title)
            info=normalize_audio(tmp,session/'audio_komputer.flac')
            meta=read_json(session/'metadata.json',{})
            meta.update(status='diimpor',audio=['audio_komputer.flac'],folder=folder,**info)
            (session/'metadata.json').write_text(json.dumps(meta,ensure_ascii=False),encoding='utf-8')
            return {'session':session.name,'message':'Audio berhasil diimpor.',**info}
        except Exception:
            if session is not None:shutil.rmtree(session,ignore_errors=True)
            raise
        finally:
            with self.lock:self.state='idle'
    def reprocess(self,args):
        with self.lock:return self._reprocess_locked(args)
    def _reprocess_locked(self,args):
        if self.state!='idle':raise ValueError('Hentikan perekaman dahulu.')
        d=self.session_dir(args['session'])
        model=args.get('model',MODEL_CHOICES[1]);gpu=args.get('gpu','CUDA')
        if model not in MODEL_CHOICES or gpu not in ('CUDA','CPU','Otomatis'):raise ValueError('Model/akselerasi tidak valid.')
        if any(j['kind']=='transcribe' and j['status']=='running' for j in self.jobs.values()):raise ValueError('Transkripsi lain masih berjalan.')
        return self.job('transcribe',lambda progress:self._reprocess(d,model,gpu,progress))
    def _reprocess(self,d,model,gpu,progress):
        import soundfile as sf
        import numpy as np
        from engine.core import to_mono_16k, export_transcript
        from engine.model_runtime import transcribe_options
        from engine.text_quality import sanitize_hypothesis, accept_speech_segment
        files=list(d.glob('audio_komputer.*')) or list(d.glob('audio_mikrofon.*'))
        if not files:raise ValueError('Audio tidak ditemukan.')
        progress('Menyiapkan model...')
        whisper,backend,_=self.transcriber.get(model,gpu)
        segments=[]
        with sf.SoundFile(str(files[0])) as f:
            sr=f.samplerate; block=max(1,int(sr*12)); pos=0
            while True:
                arr=f.read(block,dtype='float32',always_2d=True)
                if not len(arr):break
                audio=to_mono_16k(arr,sr)
                results,_=whisper.transcribe(audio,**transcribe_options('id'))
                for s in results:
                    if not accept_speech_segment(s):continue
                    txt,_=sanitize_hypothesis(s.text.strip())
                    if txt:segments.append({'start':round(pos/sr+s.start,2),'end':round(pos/sr+s.end,2),'source':'Peserta (komputer)','text':txt})
                pos+=len(arr)
                progress(f'Memproses audio {nice_seconds(pos/sr)} / {nice_seconds(len(f)/sr)}')
        export_transcript(d,segments)
        return {'session':d.name,'count':len(segments)}

class Handler(BaseHTTPRequestHandler):
    def log_message(self,*args):pass
    def _headers(self,status=200):
        self.send_response(status)
        origin=self.headers.get('Origin','')
        if origin in ('http://tauri.localhost','tauri://localhost','http://localhost:1420','http://127.0.0.1:1420'):
            self.send_header('Access-Control-Allow-Origin',origin)
            self.send_header('Vary','Origin')
        self.send_header('Access-Control-Allow-Headers','Content-Type,Authorization,X-File-Name,X-Folder-Id')
        self.send_header('Access-Control-Allow-Methods','GET,POST,OPTIONS')
        self.send_header('Cache-Control','no-store')
        self.send_header('Content-Type','application/json; charset=utf-8')
        self.end_headers()
    def _json(self,obj,status=200):
        self._headers(status)
        self.wfile.write(json.dumps(obj,ensure_ascii=False,default=str).encode('utf-8'))
    def _auth(self):
        header=self.headers.get('Authorization','')
        return hmac.compare_digest(header,'Bearer '+self.server.secret)
    def do_OPTIONS(self):self._headers(204)
    def do_GET(self):
        if not self._auth():return self._json({'error':'Akses ditolak.'},401)
        try:
            from urllib.parse import parse_qs,urlsplit
            path=urlsplit(self.path).path;q=parse_qs(urlsplit(self.path).query)
            svc=self.server.service
            if path=='/status':out=svc.status(int(q.get('since',[0])[0]))
            elif path=='/devices':
                a,b,default=enumerate_devices()
                out={'speakers':[vars(x) for x in a],'microphones':[vars(x) for x in b],'default':default}
            elif path=='/sessions':out=svc.sessions()
            elif path=='/folders':out=svc.folders.list()
            elif path=='/session':out=svc.session(q.get('id',[''])[0])
            elif path=='/models':out={'transcribe':MODEL_CHOICES,'summary':PROVIDERS}
            elif path=='/gpu':out={'report':runtime_report(), 'nvidia': discover_nvidia(), 'cudaInstalled': cuda_installed()}
            elif path=='/ollama/status':
                svc.require_storage()
                try:managed_ollama.ensure()
                except RuntimeError as exc:out={'running':False,'processor':'unknown','message':str(exc),'nvidia':managed_ollama.nvidia_status(),'models':[]}
                else:out=managed_ollama.diagnostics()
            else:return self._json({'error':'Rute tidak ditemukan.'},404)
            return self._json(out)
        except Exception as e:self._json({'error':str(e)[:300]},400)
    def do_POST(self):
        if not self._auth():return self._json({'error':'Akses ditolak.'},401)
        try:
            from urllib.parse import urlsplit
            path=urlsplit(self.path).path
            n=int(self.headers.get('Content-Length',0))
            if n<0 or n>1024*1024*1024:return self._json({'error':'Ukuran permintaan tidak diizinkan.'},413)
            svc=self.server.service
            if path=='/import':
                import tempfile
                from urllib.parse import unquote
                name=unquote(self.headers.get('X-File-Name','rekaman.wav'))
                suffix=Path(name).suffix.lower()
                if suffix not in AUDIO_EXTENSIONS:raise ValueError('Format impor tidak didukung.')
                if n==0:raise ValueError('File audio kosong.')
                tmp=None
                try:
                    with tempfile.NamedTemporaryFile(delete=False,suffix=suffix,dir=svc.root) as fp:
                        tmp=fp.name
                        remaining=n
                        while remaining:
                            data=self.rfile.read(min(1024*1024,remaining))
                            if not data:raise ValueError('Unggahan terputus.')
                            fp.write(data);remaining-=len(data)
                    result=svc.import_audio(tmp,Path(name).stem[:60],self.headers.get('X-Folder-Id',''))
                finally:
                    if tmp:Path(tmp).unlink(missing_ok=True)
                return self._json(result)
            if n>2*1024*1024:return self._json({'error':'Permintaan JSON terlalu besar.'},413)
            args=json.loads(self.rfile.read(n) or b'{}')
            if path in ('/start','/model/prepare','/cuda/prepare','/reprocess','/ollama/pull','/ollama/catalog','/ollama/delete','/ollama/status','/ollama/gpu-test','/provider/test'):
                svc.require_storage()
            if path=='/start':out=svc.start(args)
            elif path=='/stop':out=svc.stop()
            elif path=='/finish-partial':out=svc.finish_partial()
            elif path=='/model/prepare':out=svc.transcribe_model(args.get('model',MODEL_CHOICES[1]),args.get('gpu','Otomatis'))
            elif path=='/cuda/prepare':
                if svc.state!='idle':raise ValueError('Hentikan perekaman dahulu.')
                if not discover_nvidia():raise ValueError('GPU NVIDIA tidak terdeteksi. Gunakan mode CPU.')
                out=svc.job('cuda',lambda update:install_cuda(update))
            elif path=='/test/audio':
                if svc.state!='idle':raise ValueError('Tes audio dilakukan sebelum merekam.')
                outputs,_,_=enumerate_devices()
                dev=next((d for d in outputs if d.index==int(args.get('speaker',-1))),None)
                if dev is None:raise ValueError('Speaker tidak ditemukan.')
                def test(progress):
                    progress('Memeriksa suara selama 5 detik...')
                    rms=diagnose(dev,5.)
                    return {'rms':rms,'message': 'Suara terdeteksi dengan baik.' if rms>0.0006 else 'Audio hening. Periksa speaker dan perangkat rekaman.'}
                out=svc.job('audio_test',test)
            elif path=='/summary':out=svc.summarize(args)
            elif path=='/reprocess':out=svc.reprocess(args)
            elif path=='/preferences':out={**preferences.public_settings(),'storageReady':svc.storage_ready,'storageError':svc.storage_error}
            elif path=='/credentials/save':
                preferences.save_key(args.get('provider',''),str(args.get('apiKey','')));out={'saved':True}
            elif path=='/storage/select':
                with svc.lock:
                    if svc.state!='idle' or any(j['status']=='running' for j in svc.jobs.values()):raise ValueError('Selesaikan proses aktif sebelum mengganti folder model.')
                    managed_ollama.stop()
                    folder=str(args.get('folder',''))
                    svc.storage_ready=False
                    out=svc.job('storage',lambda progress:svc.prepare_storage(progress,folder))
            elif path=='/provider/test':
                out=test_connection(args.get('provider',''),str(args.get('model','')),str(args.get('apiKey','')),str(args.get('endpoint','')))
            elif path=='/provider/models':out={'models':models_for(args.get('provider','ollama'),args.get('apiKey',''),args.get('endpoint',''))}
            elif path=='/transcription/catalog':
                from engine.model_runtime import MODEL_IDS
                from engine.model_download import metadata
                from concurrent.futures import ThreadPoolExecutor
                def describe(item):
                    name,model=item
                    try:return {'name':name,**metadata(model)}
                    except Exception:return {'name':name,'sizeBytes':None}
                with ThreadPoolExecutor(max_workers=6) as pool:
                    out={'models':list(pool.map(describe,MODEL_IDS.items()))}
            elif path=='/ollama/gpu-test':
                if svc.state!='idle':raise ValueError('Selesaikan rekaman sebelum menjalankan tes GPU Ollama.')
                out=managed_ollama.test_gpu(str(args.get('model','')))
            elif path=='/ollama/catalog':
                try:managed_ollama.ensure();out=catalog()
                except RuntimeError as exc:out={'connected':False,'message':str(exc),'models':[dict(m,installed=False,sizeBytes=m['estimatedBytes'],sizeSource='estimate') for m in __import__('local_models').CATALOG]}
            elif path=='/ollama/delete':
                from local_models import delete_model
                with svc.lock:
                    if any(j['status']=='running' for j in svc.jobs.values()):raise ValueError('Selesaikan proses aktif sebelum menghapus model.')
                    managed_ollama.ensure();out=delete_model(str(args.get('model','')))
            elif path=='/ollama/pull':
                name=args.get('model','qwen3.5:4b')
                if name not in {m['id'] for m in __import__('local_models').CATALOG}:raise ValueError('Pilih model dari katalog lokal.')
                with svc.lock:
                    active=next((k for k,v in svc.jobs.items() if v['kind']=='ollama' and v['status']=='running'),None)
                    if active:raise ValueError('Tunggu unduhan model yang sedang berjalan selesai.')
                    def pull(update):
                        def progress(value):
                            update({**(value if isinstance(value,dict) else {'message':str(value)}),'model':name})
                        managed_ollama.ensure(progress, install=True)
                        return pull_model(name,progress)
                    out=svc.job('ollama',pull)
            elif path=='/folders/delete':out=svc.delete_folder(args.get('id',''))
            elif path=='/folders/create':out=svc.folders.create(args.get('name',''))
            elif path=='/folders/rename':out=svc.folders.rename(args.get('id',''),args.get('name',''))
            elif path=='/session/update':out=svc.update_session(args)
            elif path=='/session/delete':out=svc.delete_session(args.get('id',''),args.get('audioOnly') is True)
            elif path=='/session/edit':
                d=svc.session_dir(args.get('id',''))
                txt=str(args.get('text',''))
                if len(txt)>2_000_000:raise ValueError('Teks terlalu panjang.')
                (d/'transkrip_edit.txt').write_text(txt,encoding='utf-8')
                out={'saved':True}
            else:return self._json({'error':'Rute tidak ditemukan.'},404)
            self._json(out)
        except Exception as e:self._json({'error':str(e)[:330]},400)

def main():
    if len(sys.argv) == 3 and sys.argv[1] == '--self-test':
        from engine.packaging_check import run
        raise SystemExit(run(sys.argv[2]))
    p=argparse.ArgumentParser();p.add_argument('--port',type=int,required=True);p.add_argument('--token',required=True);p.add_argument('--data-root',default=str(default_output_root()))
    p.add_argument('--parent-pid', type=int)
    p.add_argument('--install-dir')
    args=p.parse_args()
    if args.install_dir:os.environ['NOTELY_INSTALL_DIR']=args.install_dir
    from engine.parent_watch import watch_parent
    watch_parent(args.parent_pid)
    if not 1024<args.port<65536 or len(args.token)<24:raise SystemExit('Port/token invalid')
    server=ThreadingHTTPServer(('127.0.0.1',args.port),Handler)
    server.daemon_threads=True
    server.secret=args.token;server.service=Service(args.data_root)
    if sys.stdout is not None: print('ENGINE_READY',flush=True)
    try:server.serve_forever(poll_interval=.25)
    except KeyboardInterrupt:pass
    finally:
        if server.service.state=='recording':server.service.stop()
        server.server_close()
if __name__=='__main__':main()

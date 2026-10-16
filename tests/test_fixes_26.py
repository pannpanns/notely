import ast,io,json,os,sys,tempfile,threading,unittest,urllib.error
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'backend'))
import providers,preferences

# Exercise actual service file operations without loading Windows audio hardware.
source=ast.parse((ROOT/'backend/server.py').read_text())
service=next(n for n in source.body if isinstance(n,ast.ClassDef) and n.name=='Service')
service.body=[n for n in service.body if isinstance(n,ast.FunctionDef) and n.name in ('sessions','session_sizes','delete_session','session_dir')]
import shutil
scope=dict(Path=Path,os=os,json=json,shutil=shutil,read_json=lambda p,default:json.loads(p.read_text()) if p.exists() else default)
exec(compile(ast.Module(body=[service],type_ignores=[]),'<service>','exec'),scope)
Service=scope['Service']
class Fix26(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name);self.svc=Service();self.svc.root=self.root
        self.svc.lock=threading.RLock();self.svc.state='idle';self.svc.jobs={};self.svc.active_session=None
        self.folder=self.root/'session';self.folder.mkdir()
        (self.folder/'metadata.json').write_text('{"audio":["audio_komputer.wav"]}')
        (self.folder/'audio_komputer.wav').write_bytes(b'a'*10000)
        (self.folder/'ringkasan_ai.md').write_text('Summary')
    def test_size_and_audio_delete_keeps_notes(self):
        self.assertEqual(self.svc.sessions()[0]['audioBytes'],10000)
        self.svc.delete_session('session',True)
        self.assertEqual(self.svc.sessions()[0]['audioBytes'],0)
        self.assertTrue((self.folder/'ringkasan_ai.md').exists())
        self.assertEqual(json.loads((self.folder/'metadata.json').read_text())['audio'],[])
    def test_delete_all(self):
        self.svc.delete_session('session');self.assertFalse(self.folder.exists())
        self.assertEqual(self.svc.sessions(),[])
    def test_block_busy_and_traversal(self):
        for state in ['recording','stopping']:
            self.svc.state=state
            with self.assertRaises(ValueError):self.svc.delete_session('session')
        self.svc.state='idle';self.svc.jobs={'job':{'status':'running'}}
        with self.assertRaises(ValueError):self.svc.delete_session('session')
        self.svc.jobs={}
        for path in ['../session','..','/tmp','']:
            with self.assertRaises(ValueError):self.svc.delete_session(path)
        self.assertTrue(self.folder.exists())
    @unittest.skipIf(os.name=='nt','Symlink permission may require admin')
    def test_no_follow_symlinks(self):
        alias=self.root/'alias';alias.symlink_to(self.folder,target_is_directory=True)
        with self.assertRaises(ValueError):self.svc.delete_session('alias')
        self.assertEqual(len(self.svc.sessions()),1)
    def test_default_storage_reused(self):
        with patch('preferences.config_dir',return_value=self.root/'Notely'),patch.dict(os.environ,{},clear=False):
            preferences.activate_storage();first=os.environ['NOTELY_MODEL_ROOT']
            # Windows TEMP may use RUNNER~1 while select_folder resolves runneradmin.
            # Compare actual filesystem identity, not the spelling of the path.
            self.assertTrue(Path(first).samefile(self.root/'Notely'/'NotelyModels'))
            preferences.activate_storage();self.assertEqual(os.environ['NOTELY_MODEL_ROOT'],first)
    def test_rate_limit_recovers(self):
        error=urllib.error.HTTPError('https://example.test',429,'busy',{'Retry-After':'1'},io.BytesIO(b'{}'))
        with patch('providers.urllib.request.urlopen',side_effect=[error,io.BytesIO(b'{"ok":true}')]),patch('providers.time.sleep') as wait:
            self.assertTrue(providers._fetch('https://example.test')['ok']);wait.assert_called_once()
    def test_network_retry_recovers(self):
        with patch('providers.urllib.request.urlopen',side_effect=[TimeoutError(),io.BytesIO(b'{"ok":true}')]),patch('providers.time.sleep'):
            self.assertTrue(providers._fetch('https://example.test')['ok'])
    def test_summary_cleanup(self):
        s=providers.clean_summary('Berikut adalah ringkasan yang ringkas dan akurat berdasarkan catatan yang diberikan:\n\n---\n\n#### Topik\n- Materi')
        self.assertEqual(s,'#### Topik\n- Materi')
    def test_uninstall_keep_default(self):
        s=(ROOT/'src-tauri/windows/hooks-models.nsh').read_text()
        self.assertIn('/SD IDYES IDYES notely_keep_models',s)
    def test_blocked_and_truncated_gemini(self):
        for response in [{'promptFeedback':{'blockReason':'SAFETY'}},{'candidates':[{'finishReason':'MAX_TOKENS'}]}]:
            with patch('providers.urllib.request.urlopen',return_value=io.BytesIO(('data: '+json.dumps(response)+'\n\n').encode())):
                with self.assertRaises(RuntimeError):providers._ask('gemini','model','key','text')

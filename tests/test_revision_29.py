import io,json,os,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import Mock,patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import preferences,providers
class Revision29(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name).resolve()
  self.cfg=patch('preferences.config_dir',return_value=self.root/'config');self.cfg.start();self.addCleanup(self.cfg.stop)
  self.env=patch.dict(os.environ,{},clear=False);self.env.start();self.addCleanup(self.env.stop)
 def old_models(self):
  base=self.root/'old';base.mkdir();preferences.select_folder(str(base));old=base/'NotelyModels'
  for name in ('ollama/blobs/model','transcription/model.bin','runtime/ollama/ollama.exe'):
   f=old/name;f.parent.mkdir(parents=True,exist_ok=True);f.write_bytes(b'model data')
  return old
 def test_install_migration_and_reuse(self):
  old=self.old_models();install=self.root/'D_Notely';install.mkdir()
  with patch.dict(os.environ,{'NOTELY_INSTALL_DIR':str(install)}):
   out=preferences.use_install_storage();preferences.use_install_storage()
   self.assertEqual(os.environ['NOTELY_HF_CACHE'],str(install/'NotelyModels/transcription'))
   self.assertEqual(os.environ['NOTELY_CUDA_ROOT'],str(install/'NotelyModels/cuda'))
  self.assertEqual(Path(out['modelRoot']),install/'NotelyModels');self.assertFalse(old.exists())
  self.assertEqual((install/'NotelyModels/transcription/model.bin').read_bytes(),b'model data')
 def test_failed_copy_keeps_original_and_settings(self):
  old=self.old_models();dest=self.root/'dest';dest.mkdir()
  with patch('preferences.shutil.copyfile',side_effect=OSError('disk failure')):
   with self.assertRaises(OSError):preferences.migrate_storage(dest)
  self.assertEqual(Path(preferences.read()['modelRoot']),old);self.assertTrue((old/'ollama/blobs/model').exists())
  preferences.migrate_storage(dest);self.assertFalse(old.exists())
 def test_conflict_does_not_overwrite(self):
  old=self.old_models();dest=self.root/'dest';dest.mkdir();target=dest/'NotelyModels';target.mkdir();(target/'.notely-owned').write_text('app.notely.desktop')
  (target/'ollama/blobs').mkdir(parents=True);(target/'ollama/blobs/model').write_bytes(b'different')
  with self.assertRaises(ValueError):preferences.migrate_storage(dest)
  self.assertEqual((target/'ollama/blobs/model').read_bytes(),b'different');self.assertTrue(old.exists())
 def test_insufficient_disk(self):
  old=self.old_models();dest=self.root/'dest';dest.mkdir()
  with patch('preferences.shutil.disk_usage',return_value=Mock(free=0)),self.assertRaises(ValueError):preferences.migrate_storage(dest)
  self.assertTrue(old.exists())
 def test_custom_location_stays_selected(self):
  old=self.old_models();data=preferences.read();data['storagePolicy']='custom';preferences.write(data)
  with patch.dict(os.environ,{'NOTELY_INSTALL_DIR':str(self.root/'unused')}):preferences.use_install_storage()
  self.assertEqual(Path(os.environ['NOTELY_MODEL_ROOT']),old)
 def test_length_limit_preserves_preview_without_replay(self):
  opener=Mock();opener.open.return_value=io.BytesIO(json.dumps({'message':{'content':'Awal '},'done':True,'done_reason':'length'}).encode()+b'\n')
  events=[]
  with patch('providers.urllib.request.build_opener',return_value=opener),self.assertRaisesRegex(RuntimeError,'tidak diulang otomatis'):
   providers._stream_local('model','prompt',events.append)
  self.assertEqual(opener.open.call_count,1);self.assertEqual(events[-1]['preview'],'Awal ')
 def test_local_summary_uses_entire_transcript_once(self):
  text='awal '+('materi '*3500)+' akhir'
  with patch('providers._ask',return_value='Catatan') as ask:
   providers.summarize('title',text,'ollama','model',instructions='Jelaskan dalam paragraf dengan analogi')
  self.assertEqual(ask.call_count,1)
  self.assertIn(text,ask.call_args.args[3]);self.assertIn('Jelaskan dalam paragraf dengan analogi',ask.call_args.args[3])
 def test_oversized_local_summary_never_calls_model(self):
  with patch('providers._ask') as ask,self.assertRaisesRegex(ValueError,'terlalu panjang'):
   providers.summarize('title','x'*40000,'ollama','model')
  ask.assert_not_called()
 def test_fresh_install_root(self):
  dest=self.root/'D_Notely';dest.mkdir()
  with patch.dict(os.environ,{'NOTELY_INSTALL_DIR':str(dest)}):preferences.use_install_storage()
  self.assertEqual(Path(preferences.read()['modelRoot']),dest/'NotelyModels')

 def test_legacy_cuda_moves_with_models(self):
  from engine.cuda_setup import CUDA_PACKAGES
  old=self.old_models();legacy=self.root/'config/cuda';legacy.mkdir()
  (legacy/'notely_cuda_manifest.json').write_text(json.dumps({'packages':list(map(list,CUDA_PACKAGES))}))
  (legacy/'nvidia').mkdir();(legacy/'nvidia/test.dll').write_bytes(b'cuda')
  dest=self.root/'D_Notely';dest.mkdir();preferences.migrate_storage(dest)
  self.assertFalse(legacy.exists());self.assertEqual((dest/'NotelyModels/cuda/nvidia/test.dll').read_bytes(),b'cuda')

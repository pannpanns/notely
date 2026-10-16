import sys, tempfile, unittest, subprocess, shutil
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend/engine'))
from engine.audio_import import normalize_audio
from engine.folder_store import Collections

class ImportFormats(unittest.TestCase):
 def test_decode_real_formats(self):
  if not shutil.which('ffmpeg'):self.skipTest('Fixture generation requires ffmpeg')
  import soundfile as sf
  with tempfile.TemporaryDirectory() as tmp:
   for ext,codec in [('wav','pcm_s16le'),('flac','flac'),('mp3','libmp3lame'),('ogg','libvorbis'),('opus','libopus'),('m4a','aac'),('aac','aac'),('aiff','pcm_s16be'),('wma','wmav2'),('webm','libopus'),('mp4','aac'),('mkv','flac'),('ac3','ac3'),('au','pcm_s16be'),('caf','pcm_s16le'),('wv','wavpack')]:
    with self.subTest(format=ext):
     src=Path(tmp)/('input.'+ext);dst=Path(tmp)/'out.flac'
     subprocess.run(['ffmpeg','-v','error','-y','-f','lavfi','-i','sine=frequency=440:duration=0.3','-ac','2','-c:a',codec,str(src)],check=True,capture_output=True)
     info=normalize_audio(src,dst);audio,rate=sf.read(dst)
     self.assertEqual(rate,16000);self.assertEqual(audio.ndim,1);self.assertGreater(info['duration'],.25)
     self.assertGreater(abs(audio).max(),.01)
 def test_corrupt_rolls_back(self):
  with tempfile.TemporaryDirectory() as tmp:
   src=Path(tmp)/'bad.mp3';dst=Path(tmp)/'out.flac';src.write_bytes(b'not audio')
   with self.assertRaises(Exception):normalize_audio(src,dst)
   self.assertFalse(dst.exists())
 def test_folder_persistence_and_validation(self):
  with tempfile.TemporaryDirectory() as tmp:
   folders=Collections(tmp);ipa=folders.create(' IPA ')
   self.assertEqual(Collections(tmp).list()[0]['name'],'IPA')
   with self.assertRaises(ValueError):folders.create('ipa')
   with self.assertRaises(ValueError):folders.create('  ')
   with self.assertRaises(ValueError):folders.validate('../escape')
   folders.rename(ipa['id'],'Biologi');self.assertEqual(folders.list()[0]['id'],ipa['id'])
   self.assertEqual(folders.list()[0]['name'],'Biologi')

class SessionOrganization(unittest.TestCase):
 def test_rename_move_and_import_rollback(self):
  import threading
  from server import Service
  from engine.core import new_session, read_json
  from unittest.mock import patch
  with tempfile.TemporaryDirectory() as tmp:
   service=Service.__new__(Service);service.root=Path(tmp);service.lock=threading.RLock();service.state='idle';service.jobs={};service.folders=Collections(tmp)
   folder=service.folders.create('IPA');session=new_session(service.root,'Awal')
   (session/'audio_komputer.flac').write_bytes(b'original')
   (session/'transkrip.txt').write_text('hasil transkrip')
   service.update_session({'id':session.name,'title':'Pelajaran IPA','folder':folder['id']})
   self.assertEqual(read_json(session/'metadata.json',{})['judul'],'Pelajaran IPA')
   self.assertEqual(service.sessions()[0]['folder'],folder['id'])
   self.assertEqual((session/'audio_komputer.flac').read_bytes(),b'original')
   self.assertEqual((session/'transkrip.txt').read_text(),'hasil transkrip')
   service.update_session({'id':session.name,'folder':''})
   self.assertEqual(service.sessions()[0]['folder'],'')
   with self.assertRaises(ValueError):service.update_session({'id':session.name,'folder':'../escape'})
   with patch('server.normalize_audio',side_effect=ValueError('bad audio')):
    with self.assertRaises(ValueError):service.import_audio('bad.mp3','Invalid')
   self.assertEqual(len(service.sessions()),1);self.assertEqual(service.state,'idle')
   service.state='recording'
   with self.assertRaises(ValueError):service.update_session({'id':session.name,'title':'Busy'})

if __name__=='__main__':unittest.main()

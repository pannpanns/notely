import io
import json
import sys
import unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'backend'))
from local_models import DownloadProgress, pull_model, catalog

class LocalModelTests(unittest.TestCase):
    def test_multilayer_progress_and_speed(self):
        now=[0]
        tracker=DownloadProgress('test', {'a':100, 'b':200}, clock=lambda:now[0])
        tracker.feed({'digest':'a','total':100,'completed':50})
        now[0]=1
        p=tracker.feed({'digest':'a','total':100,'completed':100})
        self.assertEqual(p['bytesPerSecond'],50)
        now[0]=2
        p=tracker.feed({'digest':'b','total':200,'completed':100})
        self.assertEqual(p['downloadedBytes'],200)
        self.assertEqual(p['totalBytes'],300)
        self.assertEqual(p['bytesPerSecond'],0) # cached/resumed prefix isn't network traffic
        now[0]=3
        p=tracker.feed({'digest':'b','total':200,'completed':150})
        self.assertEqual(p['bytesPerSecond'],50)
        self.assertEqual(p['etaSeconds'],1)
        self.assertEqual(tracker.feed({'status':'success'})['percent'],100)

    def test_stream_must_finish_successfully(self):
        raw=b'{"status":"pulling manifest"}\n'
        with patch('local_models.manifest',return_value={}), patch('local_models.urllib.request.urlopen',return_value=io.BytesIO(raw)):
            with self.assertRaisesRegex(RuntimeError,'terputus'):
                pull_model('qwen3.5:2b',lambda x:None)

    def test_stream_errors_and_allowlist(self):
        with self.assertRaises(ValueError):pull_model('../../bad',lambda x:None)
        with patch('local_models.manifest',return_value={}), patch('local_models.urllib.request.urlopen',return_value=io.BytesIO(b'{"error":"disk full"}\n')):
            with self.assertRaisesRegex(RuntimeError,'disk full'):pull_model('qwen3.5:2b',lambda x:None)

    def test_success_and_installed_size(self):
        events=[]
        with patch('local_models.manifest',return_value={'a':100}), patch('local_models.urllib.request.urlopen',return_value=io.BytesIO(b'{"status":"success"}\n')):
            self.assertEqual(pull_model('qwen3.5:2b',events.append)['model'],'qwen3.5:2b')
        self.assertEqual(events[-1]['percent'],100)
        with patch('local_models.read_json',return_value={'models':[{'name':'qwen3.5:2b','size':1234}]}), patch('local_models.manifest',return_value={'a':999}):
            data=catalog();m=next(x for x in data['models'] if x['id']=='qwen3.5:2b')
            self.assertTrue(m['installed']);self.assertEqual(m['sizeBytes'],1234)
            self.assertEqual(m['sizeSource'],'installed')

    def test_offline_catalog_keeps_estimates(self):
        with patch('local_models.read_json',side_effect=OSError), patch('local_models.manifest',side_effect=OSError):
            data=catalog()
            self.assertFalse(data['connected'])
            self.assertTrue(all(m['sizeSource']=='estimate' for m in data['models']))

if __name__=='__main__':unittest.main()

import io
import sys
import unittest
import urllib.error
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'backend'))
sys.path.insert(0,str(ROOT/'backend/engine'))
import providers
import model_runtime

class FixTests(unittest.TestCase):
    def error(self,code):
        return urllib.error.HTTPError('https://example.test',code,'error',{},io.BytesIO(b'{}'))
    def test_busy_service_retries_then_succeeds(self):
        messages=[]
        with patch('providers.urllib.request.urlopen',side_effect=[self.error(503),self.error(503),io.BytesIO(b'{"ok":true}')]) as call,patch('providers.time.sleep') as sleep:
            self.assertTrue(providers._fetch('https://example.test',progress=messages.append)['ok'])
            self.assertEqual(call.call_count,3);self.assertEqual(sleep.call_count,2);self.assertEqual(len(messages),2)
    def test_retry_is_bounded(self):
        with patch('providers.urllib.request.urlopen',side_effect=lambda *a,**k:(_ for _ in ()).throw(self.error(503))) as call,patch('providers.time.sleep'):
            with self.assertRaisesRegex(RuntimeError,'bukan indikasi API key salah'):providers._fetch('https://example.test')
            self.assertEqual(call.call_count,4)
    def test_invalid_requests_are_not_retried(self):
        for code in [400,401,403]:
            with patch('providers.urllib.request.urlopen',side_effect=self.error(code)) as call,patch('providers.time.sleep') as sleep:
                with self.assertRaises(RuntimeError):providers._fetch('https://example.test')
                self.assertEqual(call.call_count,1);sleep.assert_not_called()
    def test_model_list_uses_header_not_url_for_key(self):
        with patch('providers._fetch',return_value={'models':[]}) as fetch:
            providers.models_for('gemini',' test-key ')
            self.assertNotIn('test-key',fetch.call_args.args[0])
            self.assertEqual(fetch.call_args.kwargs['headers']['x-goog-api-key'],'test-key')
    def test_gpu_probe_hides_windows_console(self):
        with patch('model_runtime.sys.platform','win32'),patch('model_runtime.subprocess.CREATE_NO_WINDOW',0x08000000,create=True),patch('model_runtime.subprocess.run') as run:
            run.return_value.stdout='NVIDIA'
            self.assertEqual(model_runtime.discover_nvidia(),'NVIDIA')
            self.assertEqual(run.call_args.kwargs['creationflags'],0x08000000)

if __name__=='__main__':unittest.main()

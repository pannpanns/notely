import hashlib,io,json,sys,tempfile,unittest,urllib.error,zipfile
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import ollama_install
class Response(io.BytesIO):
    def __init__(self,body,status=200,headers=None):super().__init__(body);self.status=status;self.headers=headers or {}
class InstallationTests(unittest.TestCase):
    def archive(self,name='ollama.exe'):
        buf=io.BytesIO()
        with zipfile.ZipFile(buf,'w') as z:z.writestr(name,b'fake executable')
        return buf.getvalue()
    def run_install(self,root,body,digest=None):
        metadata={'version':'v-test','size':len(body),'sha256':digest or hashlib.sha256(body).hexdigest(),'url':'https://github.com/ollama/ollama/releases/download/v-test/ollama-windows-amd64.zip'}
        with patch('ollama_install.RELEASE',metadata),patch('ollama_install.platform.system',return_value='Windows'),patch('ollama_install.platform.machine',return_value='AMD64'),patch('ollama_install.urllib.request.urlopen',return_value=Response(body)) as fetch:
            result=ollama_install.install_runtime(root)
            # The only network request is to the archive, never GitHub API.
            self.assertEqual(fetch.call_count,1)
            self.assertEqual(fetch.call_args.args[0].full_url,metadata['url'])
            return result
    def test_verified_install_and_reuse(self):
        with tempfile.TemporaryDirectory() as root:
            result=self.run_install(root,self.archive());self.assertTrue(Path(result).exists())
            with patch('ollama_install.platform.system',return_value='Windows'),patch('ollama_install.platform.machine',return_value='AMD64'),patch('ollama_install.urllib.request.urlopen') as network:
                self.assertEqual(ollama_install.install_runtime(root),result);network.assert_not_called()
    def test_corrupt_download_never_published(self):
        with tempfile.TemporaryDirectory() as root:
            with self.assertRaisesRegex(RuntimeError,'Verifikasi'):self.run_install(root,self.archive(),'0'*64)
            self.assertFalse((Path(root)/'runtime/ollama').exists())
    def test_archive_traversal_rejected(self):
        with tempfile.TemporaryDirectory() as root:
            with self.assertRaisesRegex(RuntimeError,'tidak aman'):self.run_install(root,self.archive('../evil.exe'))
            self.assertFalse((Path(root)/'runtime/ollama').exists())
    def test_resumes_partial_at_exact_offset(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'partial';p.write_bytes(b'abc')
            with patch('ollama_install.urllib.request.urlopen',return_value=Response(b'def',206,{'Content-Range':'bytes 3-5/6'})) as fetch:
                ollama_install.download_archive('https://example.test/file',p,6,lambda x:None)
                self.assertEqual(fetch.call_args.args[0].get_header('Range'),'bytes=3-')
            self.assertEqual(p.read_bytes(),b'abcdef')
    def test_server_ignoring_range_restarts_instead_of_appending(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'partial';p.write_bytes(b'abc')
            with patch('ollama_install.urllib.request.urlopen',return_value=Response(b'abcdef')):
                ollama_install.download_archive('https://example.test/file',p,6,lambda x:None)
            self.assertEqual(p.read_bytes(),b'abcdef')
    def test_wrong_range_rejected(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'partial';p.write_bytes(b'abc')
            with patch('ollama_install.urllib.request.urlopen',return_value=Response(b'def',206,{'Content-Range':'bytes 1-3/6'})):
                with self.assertRaisesRegex(RuntimeError,'rentang'):ollama_install.download_archive('https://example.test/file',p,6,lambda x:None)
            self.assertEqual(p.read_bytes(),b'abc')
    def test_transient_403_retry_succeeds(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'partial'
            err=urllib.error.HTTPError('https://example.test/file',403,'rate limited',{},io.BytesIO())
            with patch('ollama_install.urllib.request.urlopen',side_effect=[err,Response(b'done')]),patch('ollama_install.time.sleep') as sleep:
                ollama_install.download_archive('https://example.test/file',p,4,lambda x:None)
                sleep.assert_called_once();self.assertEqual(p.read_bytes(),b'done')
    def test_failed_download_preserves_partial_for_retry(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'partial';p.write_bytes(b'abc')
            with patch('ollama_install.urllib.request.urlopen',side_effect=TimeoutError()),patch('ollama_install.time.sleep') as sleep:
                with self.assertRaisesRegex(RuntimeError,'Data parsial'):ollama_install.download_archive('https://example.test/file',p,6,lambda x:None)
                self.assertEqual(sleep.call_count,3);self.assertEqual(p.read_bytes(),b'abc')

import io,json,os,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import providers,preferences,local_models

class RevisionTests(unittest.TestCase):
    def test_model_pages_and_text_filter(self):
        pages=[{'models':[{'name':'models/gemini-flash','supportedGenerationMethods':['generateContent']}],'nextPageToken':'next'}, {'models':[{'name':'models/gemini-image','supportedGenerationMethods':['generateContent']},{'name':'models/gemini-lite','supportedGenerationMethods':['generateContent']}]}]
        with patch('providers._fetch',side_effect=pages) as fetch:
            self.assertEqual(providers.models_for('gemini','secret'),['gemini-flash','gemini-lite'])
            self.assertIn('pageToken=next',fetch.call_args.args[0])
    def test_503_switches_and_records_actual_model(self):
        with patch('providers.models_for',return_value=['gemini-flash','gemini-flash-lite']),patch('providers._ask',side_effect=[providers.ProviderError(503,'busy'),'Ringkasan']) as ask:
            result=providers.summarize('Audio','Teks','gemini','gemini-flash','key')
            self.assertNotIn('Dibuat oleh',result)
            self.assertIn('Ringkasan',result)
            self.assertEqual(ask.call_args.args[1],'gemini-flash-lite')
    def test_no_switch_for_auth_quota_or_disabled(self):
        for code,enabled in [(403,True),(429,True),(503,False)]:
            with patch('providers.models_for',return_value=['gemini-flash','gemini-flash-lite']),patch('providers._ask',side_effect=providers.ProviderError(code,'error')) as ask:
                with self.assertRaises(providers.ProviderError):providers.summarize('Audio','Teks','gemini','gemini-flash','key',gemini_fallback=enabled)
                self.assertEqual(ask.call_count,1)
    def test_test_connection_generates_response(self):
        with patch('providers._ask',return_value='OK') as ask:
            self.assertTrue(providers.test_connection('gemini','gemini-flash','key')['ok'])
            self.assertEqual(ask.call_args.args[3],'Jawab dengan satu kata: OK')
    def test_storage_is_owned_and_tracked(self):
        with tempfile.TemporaryDirectory() as tmp,patch.dict(os.environ,{},clear=False),patch('preferences.config_dir',return_value=Path(tmp)/'config'):
            a=Path(tmp)/'a';b=Path(tmp)/'b';a.mkdir();b.mkdir()
            first=preferences.select_folder(str(a));second=preferences.select_folder(str(b))
            self.assertTrue((Path(first['modelRoot'])/'.notely-owned').exists())
            self.assertEqual(len(preferences.read()['modelRoots']),2)
            self.assertEqual(os.environ['NOTELY_HF_CACHE'],str(Path(second['modelRoot'])/'transcription'))
    def test_refuses_unowned_existing_folder(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp)/'NotelyModels').mkdir()
            with self.assertRaises(ValueError):preferences.select_folder(tmp)
    def test_key_ciphertext_roundtrip_without_plaintext_file(self):
        with tempfile.TemporaryDirectory() as tmp,patch('preferences.config_dir',return_value=Path(tmp)),patch('preferences.protect',side_effect=lambda value,decrypt=False: value[::-1]):
            preferences.save_key('gemini','secret-key')
            self.assertNotIn('secret-key',(Path(tmp)/'preferences.json').read_text())
            self.assertEqual(preferences.public_settings()['keys']['gemini'],'secret-key')
            preferences.save_key('gemini','')
            self.assertNotIn('gemini',preferences.public_settings()['keys'])
    def test_delete_calls_managed_server_delete(self):
        with patch.dict(os.environ,{'NOTELY_OLLAMA_URL':'http://127.0.0.1:32123'}),patch('local_models.read_json',return_value={'models':[{'name':'qwen2.5:3b'}]}),patch('local_models.urllib.request.urlopen',return_value=io.BytesIO(b'')) as send:
            self.assertEqual(local_models.delete_model('qwen2.5:3b')['deleted'],'qwen2.5:3b')
            req=send.call_args.args[0];self.assertEqual(req.method,'DELETE');self.assertIn(':32123/api/delete',req.full_url)
    def test_selected_model_does_not_require_list_endpoint(self):
        with patch('providers.models_for',side_effect=RuntimeError('list offline')) as listing,patch('providers._ask',return_value='Ringkasan'):
            self.assertIn('Ringkasan',providers.summarize('Audio','Teks','gemini','gemini-flash','key'))
            listing.assert_not_called()
    def test_uninstall_only_managed_models(self):
        root=Path(__file__).resolve().parents[1]
        hooks=(root/'src-tauri/windows/hooks-models.nsh').read_text()
        cleanup=(root/'src-tauri/windows/cleanup-models.ps1').read_text()
        self.assertIn('NOTELY_STOP_ENGINE "-RemoveModels"',hooks)
        self.assertIn("'.notely-owned'",cleanup)
        self.assertIn('ReparsePoint',cleanup)
        self.assertNotIn('.ollama/models',cleanup)
if __name__=='__main__':unittest.main()

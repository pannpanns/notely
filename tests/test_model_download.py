import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend/engine'))
from model_download import download

class ModelDownloadTests(unittest.TestCase):
    def test_cached_model_works_without_network(self):
        with tempfile.TemporaryDirectory() as tmp:
            for name in ['model.bin','config.json','tokenizer.json']:
                (Path(tmp)/name).write_text('cached')
            with patch('huggingface_hub.snapshot_download',return_value=tmp), patch('model_download.metadata',side_effect=AssertionError('network not allowed')):
                self.assertEqual(download('small'),tmp)

    def test_download_byte_progress_is_forwarded(self):
        events=[]
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp)/'model.bin').write_text('model')
            def snapshot(repo,**kwargs):
                if kwargs.get('local_files_only'):raise FileNotFoundError()
                cls=kwargs['tqdm_class']
                transfer=cls(unit='B',name='huggingface_hub.snapshot_download.transfer',total=100)
                reconstructed=cls(unit='B',name='huggingface_hub.snapshot_download',total=100)
                transfer.update(50);reconstructed.update(50)
                return tmp
            with patch('huggingface_hub.snapshot_download',side_effect=snapshot),patch('model_download.metadata',return_value={'sizeBytes':100,'revision':'commit'}),patch('model_download.time.monotonic',side_effect=[0,1,2]):
                self.assertEqual(download('small',events.append),tmp)
            self.assertTrue(any(x.get('downloadedBytes')==50 for x in events))
            self.assertEqual(events[-1]['percent'],100)

if __name__=='__main__':unittest.main()

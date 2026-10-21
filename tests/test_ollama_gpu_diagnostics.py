import os
import sys
import unittest
from pathlib import Path
from unittest.mock import patch, Mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend'))
import managed_ollama


class OllamaGpuDiagnosticsTests(unittest.TestCase):
    def test_loaded_models_classify_full_partial_and_cpu(self):
        payload = {'models': [
            {'name': 'full', 'size': 1000, 'size_vram': 1000, 'context_length': 4096},
            {'name': 'partial', 'size': 2000, 'size_vram': 600},
            {'name': 'cpu', 'size': 2000, 'size_vram': 0},
        ]}
        with patch.object(managed_ollama, '_request_json', return_value=payload):
            models = managed_ollama._loaded_models('http://127.0.0.1:12345')
        self.assertEqual([x['processor'] for x in models], ['GPU', 'CPU + GPU', 'CPU'])
        self.assertEqual(models[0]['contextLength'], 4096)

    def test_gpu_test_runs_real_request_and_returns_vram_diagnostics(self):
        replies = [
            {'models': [{'name': 'qwen3.5:4b'}]},
            {'message': {'content': 'OK'}},
            {'version': 'test-version'},
            {'models': [{'name': 'qwen3.5:4b', 'size': 4000, 'size_vram': 3900}]},
        ]
        with patch.object(managed_ollama, 'ensure'), \
             patch.dict(os.environ, {'NOTELY_OLLAMA_URL': 'http://127.0.0.1:12345'}), \
             patch.object(managed_ollama, '_request_json', side_effect=replies) as request, \
             patch.object(managed_ollama, 'nvidia_status', return_value={'detected': True, 'devices': []}), \
             patch.object(managed_ollama, '_child', Mock(poll=Mock(return_value=None))):
            result = managed_ollama.test_gpu('qwen3.5:4b')
        self.assertTrue(result['test']['ok'])
        self.assertEqual(result['processor'], 'GPU')
        self.assertEqual(request.call_count, 4)

    def test_gpu_test_requires_model_download(self):
        with patch.object(managed_ollama, 'ensure'), \
             patch.dict(os.environ, {'NOTELY_OLLAMA_URL': 'http://127.0.0.1:12345'}), \
             patch.object(managed_ollama, '_request_json', return_value={'models': []}):
            with self.assertRaisesRegex(RuntimeError, 'belum terpasang'):
                managed_ollama.test_gpu('qwen3.5:4b')


if __name__ == '__main__':
    unittest.main()

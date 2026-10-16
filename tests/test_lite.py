import hashlib
import io
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch
from sys import path
root=Path(__file__).resolve().parents[1]
path.insert(0,str(root/'backend'/'engine'))
from cuda_setup import cuda_installed, resolve_wheel, safe_extract_nvidia_wheel, MARKER, CUDA_PACKAGES


class NotelyLiteContract(unittest.TestCase):
    def test_name_and_installer(self):
        conf=json.loads((root/'src-tauri/tauri.conf.json').read_text(encoding='utf-8'))
        self.assertEqual(conf['productName'],'Notely')
        self.assertEqual(conf['bundle']['targets'],['nsis'])
        self.assertIn('notely-engine',conf['bundle']['externalBin'][0])
        self.assertIn('serde_json', (root/'src-tauri/Cargo.toml').read_text(encoding='utf-8'))

    def test_workflow_no_cuda_bundled(self):
        workflow_dir = root/'.github/workflows'
        candidates = sorted(workflow_dir.glob('*.yml')) + sorted(workflow_dir.glob('*.yaml'))
        workflows = [p.read_text(encoding='utf-8') for p in candidates]
        builders = [text for text in workflows if 'scripts/build_engine.py' in text]
        self.assertTrue(builders, 'Workflow build Notely tidak ditemukan. Upload .github/workflows dari ZIP ke root repo.')
        y = '\n'.join(builders)
        self.assertNotIn('python -m pip install \'nvidia-', y)
        self.assertNotIn('--collect-binaries nvidia.', y)
        self.assertIn('--onedir',(root/'scripts/build_engine.py').read_text(encoding='utf-8'))
        self.assertIn('Notely-Lite-Windows-Installer',y)

    def test_cuda_optional_endpoint(self):
        api=(root/'backend/server.py').read_text(encoding='utf-8')
        self.assertIn("path=='/cuda/prepare'",api)
        self.assertIn('cuda_installed()',api)
        ui=(root/'frontend/desktop.js').read_text(encoding='utf-8')
        self.assertIn("api('/cuda/prepare'",ui)
        self.assertIn('wizard-welcome',ui)
        self.assertNotIn('rapatnote',ui.lower())

    def test_security_reject_unofficial_urls(self):
        info={'urls':[{'filename':'nvidia_test-1-py3-none-win_amd64.whl',
                       'url':'https://evil.example/wheel','digests':{'sha256':'a'*64},'size':512}]}
        with self.assertRaises(RuntimeError):resolve_wheel(info)

    def test_extract_safe(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp=Path(tmp)
            wheel=tmp/'cuda.whl'
            with zipfile.ZipFile(wheel,'w') as archive:
                archive.writestr('nvidia/cublas/bin/cublas64_12.dll',b'ok')
                archive.writestr('../../escape.dll',b'unsafe')
                archive.writestr('nvidia/cublas/include/cublas.h',b'not-needed')
            safe_extract_nvidia_wheel(wheel,tmp/'result')
            self.assertEqual((tmp/'result/nvidia/cublas/bin/cublas64_12.dll').read_bytes(),b'ok')
            self.assertFalse((tmp/'escape.dll').exists())

    def test_cuda_marker_required(self):
        with tempfile.TemporaryDirectory() as temp:
            root_dir=Path(temp)
            self.assertFalse(cuda_installed(root_dir))
            for part in ('cublas/bin/cublas64_12.dll','cublas/bin/cublasLt64_12.dll',
                         'cudnn/bin/cudnn64_9.dll','cuda_runtime/bin/cudart64_12.dll'):
                f=root_dir/'nvidia'/part
                f.parent.mkdir(parents=True,exist_ok=True)
                f.write_bytes(b'placeholder')
            (root_dir/MARKER).write_text(json.dumps({'packages':list(map(list,CUDA_PACKAGES))}))
            self.assertTrue(cuda_installed(root_dir))


if __name__=='__main__':unittest.main()

import json
import re
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

class InstallerContract(unittest.TestCase):
    def test_config_json(self):
        config=json.loads((ROOT/'src-tauri/tauri.conf.json').read_text(encoding='utf-8'))
        self.assertEqual(config['bundle']['targets'], ['nsis'])
        self.assertEqual(config['build']['frontendDist'], '../frontend')
    def test_no_inline_script_blocked_by_csp(self):
        html=(ROOT/'frontend/index.html').read_text(encoding='utf-8')
        self.assertNotIn('<script>\n', html)
        self.assertIn('<script src="ui.js"></script>', html)
        self.assertIn('<script src="desktop.js"></script>', html)
    def test_real_engine_routes(self):
        src=(ROOT/'backend/server.py').read_text(encoding='utf-8')
        for part in ("'/start'", "'/stop'", "'/finish-partial'", "'/status'", "'/gpu'", "'/summary'", "'/model/prepare'"):
            self.assertIn(part,src)
    def test_user_no_manual_cuda_command(self):
        html=(ROOT/'frontend/index.html').read_text(encoding='utf-8')
        self.assertNotIn('Cek GPU (tersedia',html)
        adapter=(ROOT/'frontend/desktop.js').read_text(encoding='utf-8')
        self.assertIn("await api('/gpu')",adapter)
        self.assertIn("gpu:engine.preferCuda?'Otomatis':'CPU'",adapter)
    def test_compact_builder(self):
        builder=(ROOT/'scripts/build_engine.py').read_text(encoding='utf-8')
        self.assertIn("'--noconsole'", builder)
        self.assertIn("'--exclude-module'", builder)
        self.assertIn("'nvidia'", builder)
        self.assertIn("'scipy'", builder)
    def test_provider_summary_exists(self):
        adapter=(ROOT/'frontend/desktop.js').read_text(encoding='utf-8')
        self.assertIn("api('/provider/models'",adapter)
        self.assertIn("api('/summary'",adapter)
        self.assertIn('Simpan audio & teks sementara',adapter)
    def test_no_sample_sessions_as_real_history(self):
        src=(ROOT/'frontend/ui.js').read_text(encoding='utf-8')
        self.assertIn('const SEED=[];',src)
    def test_privacy_no_api_key_storage(self):
        src=(ROOT/'frontend/desktop.js').read_text(encoding='utf-8')
        self.assertNotIn('safeStore.set(\'api',src)
if __name__=='__main__':unittest.main()

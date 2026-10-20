import importlib.util,json,os,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
def module(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/f'{name}.py')
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
class ReleaseTests(unittest.TestCase):
    def test_config_and_manifest(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);d=root/'src-tauri';d.mkdir()
            (d/'tauri.conf.json').write_text(json.dumps({'version':'1.1.0'}))
            config=module('configure_updates');manifest=module('release_manifest')
            env={'GITHUB_REPOSITORY':'owner/notely','NOTELY_UPDATER_PUBLIC_KEY':'test-public-key','GITHUB_REF_TYPE':'tag','GITHUB_REF_NAME':'v1.1.0'}
            with patch.dict(os.environ,env),patch.object(config,'ROOT',root),patch.object(manifest,'ROOT',root):
                config.configure()
                actual=json.loads((d/'tauri.conf.json').read_text())
                self.assertEqual(actual['plugins']['updater']['endpoints'],['https://github.com/owner/notely/releases/latest/download/latest.json'])
                bundle=d/'target/release/bundle/nsis';bundle.mkdir(parents=True)
                (bundle/'Notely Setup.exe').write_bytes(b'fixture')
                (bundle/'Notely Setup.exe.sig').write_text('fixture-signature')
                (root/'RELEASE_NOTES.md').write_text('Perubahan baru')
                manifest.manifest()
                result=json.loads((bundle/'latest.json').read_text())
                self.assertEqual(result['version'],'1.1.0')
                self.assertEqual(result['notes'],'Perubahan baru')
                self.assertTrue(result['platforms']['windows-x86_64']['url'].endswith('Notely%20Setup.exe'))
                with patch.dict(os.environ,{'GITHUB_REF_NAME':'v9.0.0'}):
                    with self.assertRaises(ValueError):config.configure()

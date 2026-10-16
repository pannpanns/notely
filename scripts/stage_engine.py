"""Stage a flat onedir runtime; map only supporting resources to install root."""
import json
import shutil
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]

def main():
    source = ROOT / 'dist/notely-engine'
    stage = ROOT / 'src-tauri/binaries'
    stage.mkdir(exist_ok=True)
    target = stage / 'runtime'
    if target.exists():
        shutil.rmtree(target)
    target.mkdir()
    shutil.copy2(source / 'notely-engine.exe', stage / 'notely-engine-x86_64-pc-windows-msvc.exe')
    for path in source.iterdir():
        if path.name == 'notely-engine.exe':
            continue
        if path.is_dir():
            shutil.copytree(path, target / path.name)
        else:
            shutil.copy2(path, target / path.name)
    # The sidecar is supplied once through externalBin, never duplicated as a resource.
    config_path = ROOT / 'src-tauri/tauri.conf.json'
    config = json.loads(config_path.read_text(encoding='utf-8'))
    config['bundle']['resources'] = {'binaries/runtime/': './'}
    config_path.write_text(json.dumps(config, indent=2)+'\n', encoding='utf-8')

if __name__ == '__main__':
    main()

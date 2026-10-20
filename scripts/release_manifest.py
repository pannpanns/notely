"""Produce the signed Windows updater manifest from actual build artifacts."""
import datetime, json, os
from pathlib import Path
from urllib.parse import quote
ROOT=Path(__file__).resolve().parents[1]
def manifest():
    config=json.loads((ROOT/'src-tauri/tauri.conf.json').read_text())
    version=config['version']; tag='v'+version
    folder=ROOT/'src-tauri/target/release/bundle/nsis'
    installers=list(folder.glob('*.exe'))
    if len(installers)!=1: raise ValueError('Harus tepat satu installer NSIS')
    installer=installers[0]
    signature=Path(str(installer)+'.sig').read_text().strip()
    if not signature: raise ValueError('Signature kosong')
    notes=(ROOT/'RELEASE_NOTES.md').read_text(encoding='utf-8')
    data={'version':version,'notes':notes,'pub_date':datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'platforms':{'windows-x86_64':{'signature':signature,
        'url':f"https://github.com/{os.environ['GITHUB_REPOSITORY']}/releases/download/{tag}/{quote(installer.name)}"}}}
    (folder/'latest.json').write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
if __name__=='__main__': manifest()

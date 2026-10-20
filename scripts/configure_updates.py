"""Generate release config from GitHub repository and public signing key."""
import json, os, re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def configure():
    repository=os.environ.get('GITHUB_REPOSITORY','')
    public_key=os.environ.get('NOTELY_UPDATER_PUBLIC_KEY','').strip()
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repository):
        raise ValueError('GITHUB_REPOSITORY harus owner/repository')
    if not public_key: raise ValueError('Isi repository variable NOTELY_UPDATER_PUBLIC_KEY dengan isi file public key Tauri.')
    path=ROOT/'src-tauri/tauri.conf.json'
    config=json.loads(path.read_text())
    tag=os.environ.get('GITHUB_REF_NAME','')
    if os.environ.get('GITHUB_REF_TYPE')=='tag' and tag != 'v'+config['version']:
        raise ValueError('Tag harus sama dengan versi tauri.conf.json: v'+config['version'])
    config.setdefault('bundle',{})['createUpdaterArtifacts']=True
    config.setdefault('plugins',{})['updater']={
        'pubkey':public_key,
        'endpoints':[f'https://github.com/{repository}/releases/latest/download/latest.json']}
    path.write_text(json.dumps(config,indent=2)+'\n')
if __name__=='__main__': configure()

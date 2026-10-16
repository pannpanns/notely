"""Build onedir sidecar in the clean CI venv; record native payload sizes."""
import ast
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
EXCLUDES = ('scipy', 'torch', 'torchaudio', 'torchvision', 'tensorflow',
            'transformers', 'pandas', 'matplotlib', 'IPython', 'pytest', 'nvidia')


def main():
    args = [sys.executable, '-m', 'PyInstaller', '--noconfirm', '--clean',
            '--onedir', '--contents-directory', '.', '--noconsole', '--noupx', '--name', 'notely-engine',
            '--paths', str(ROOT / 'backend'), '--paths', str(ROOT / 'backend/engine')]
    # Let PyInstaller hooks discover binary dependencies. Avoid collect-all:
    # it pulls package sources, tests and unrelated submodules into the payload.
    # Keep FW's Silero model and metadata used at runtime (VAD module is imported
    # by faster-whisper even when vad_filter=False).
    args += ['--collect-data', 'faster_whisper', '--copy-metadata', 'faster-whisper',
             '--collect-submodules', 'av', '--collect-binaries', 'ctranslate2',
             '--collect-binaries', 'pyaudiowpatch', '--collect-all', '_soundfile_data',
             '--copy-metadata', 'huggingface-hub', '--copy-metadata', 'ctranslate2']
    for package in EXCLUDES:
        args += ['--exclude-module', package]
    args.append(str(ROOT / 'backend/server.py'))
    subprocess.run(args, cwd=ROOT, check=True)
    report = ROOT / 'build_reports'
    report.mkdir(exist_ok=True)
    toc = ast.literal_eval((ROOT / 'build/notely-engine/Analysis-00.toc').read_text(encoding='utf-8'))
    entries = {}

    def walk(value):
        if isinstance(value, (list, tuple)):
            if len(value) == 3 and all(isinstance(x, str) for x in value):
                dest, source, kind = value
                p = Path(source)
                if p.is_file():
                    entries[(dest, source)] = {'destination': dest, 'bytes': p.stat().st_size, 'kind': kind}
            else:
                for item in value:
                    walk(item)
    walk(toc)
    payload = sorted(entries.values(), key=lambda x: x['bytes'], reverse=True)
    (report / 'engine-payload.json').write_text(json.dumps(payload, indent=2), encoding='utf-8')
    # Fail if forbidden scientific/training/GPU packages accidentally return.
    for item in payload:
        dest = item['destination'].lower().replace('\\', '/').replace('.', '/')
        if any(dest == x or dest.startswith(x + '/') for x in EXCLUDES):
            raise RuntimeError(f"Unexpected package: {item['destination']}")
    folder = ROOT / 'dist/notely-engine'
    exe = folder / 'notely-engine.exe'
    size = sum(p.stat().st_size for p in folder.rglob('*') if p.is_file())
    lines = ['# Engine payload', '', f'Engine folder (uncompressed): {size / 1048576:.2f} MiB', '',
             '| Largest collected entries | MiB |', '| --- | ---: |']
    lines += [f"| {x['destination']} | {x['bytes']/1048576:.2f} |" for x in payload[:40]]
    (report / 'engine-size.md').write_text('\n'.join(lines)+'\n', encoding='utf-8')
    print('\n'.join(lines))
    if size > 600 * 1048576:
        raise RuntimeError('Engine exceeds the 600 MiB uncompressed regression budget. Inspect build_reports.')


if __name__ == '__main__':
    main()

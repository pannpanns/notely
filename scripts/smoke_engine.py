"""Exercise the packaged engine, including native imports, without downloading models."""
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request


def main():
    exe = Path(sys.argv[1]).resolve()
    with tempfile.TemporaryDirectory(prefix='notely-smoke-') as tmp:
        # Run native dependency self-test from the frozen executable, not host Python.
        subprocess.run([str(exe), '--self-test', str(Path(tmp)/'self-test.json')], check=True, timeout=120)
        report = json.loads((Path(tmp)/'self-test.json').read_text(encoding='utf-8'))
        assert report['ok'], report
        with socket.socket() as s:
            s.bind(('127.0.0.1', 0))
            port = s.getsockname()[1]
        token = secrets.token_hex(32)
        owner = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(180)'])
        child = subprocess.Popen([str(exe), '--parent-pid', str(owner.pid),
                                  '--port', str(port), '--token', token, '--data-root', tmp])
        base = f'http://127.0.0.1:{port}'
        try:
            deadline = time.monotonic()+90
            while True:
                if child.poll() is not None:
                    raise RuntimeError(f'Engine exited: {child.returncode}')
                try:
                    request = urllib.request.Request(base+'/models', headers={'Authorization': 'Bearer '+token})
                    with urllib.request.urlopen(request, timeout=3) as response:
                        assert json.load(response)['transcribe']
                    break
                except (OSError, urllib.error.URLError):
                    if time.monotonic() >= deadline: raise
                    time.sleep(0.5)
            try:
                urllib.request.urlopen(base+'/status', timeout=3)
                raise AssertionError('Unauthenticated request accepted')
            except urllib.error.HTTPError as exc:
                assert exc.code == 401
            owner.terminate()
            owner.wait(timeout=10)
            child.wait(timeout=10)
            assert child.returncode == 0, child.returncode
            print('Native imports, authenticated API, and parent-death exit: PASS')
        finally:
            for process in (child, owner):
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=15)
        # Exercise the actual uninstall cleanup with an orphan engine. The
        # canonical dist basename matches the installed name (staged one differs).
        if exe.name.lower() == 'notely-engine.exe':
            orphan = subprocess.Popen([str(exe), '--port', str(port), '--token', token, '--data-root', tmp])
            try:
                deadline = time.monotonic()+30
                while True:
                    if orphan.poll() is not None:
                        raise RuntimeError('Orphan test engine exited before cleanup')
                    try:
                        request = urllib.request.Request(base+'/models', headers={'Authorization': 'Bearer '+token})
                        with urllib.request.urlopen(request, timeout=2):
                            break
                    except OSError:
                        if time.monotonic() >= deadline: raise
                        time.sleep(0.2)
                script = Path(__file__).resolve().parents[1]/'src-tauri/windows/cleanup.ps1'
                subprocess.run([str(Path(os.environ['WINDIR'])/'SysWOW64/WindowsPowerShell/v1.0/powershell.exe'), '-NoProfile', '-NonInteractive',
                                '-ExecutionPolicy', 'Bypass', '-File', str(script),
                                '-InstallDir', str(exe.parent)], check=True, timeout=40)
                orphan.wait(timeout=10)
                print('32-bit PowerShell entry -> native path-scoped cleanup: PASS')
            finally:
                if orphan.poll() is None: orphan.kill()
                orphan.wait(timeout=15)


if __name__ == '__main__': main()

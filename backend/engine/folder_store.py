"""Named collections use stable IDs, never user-supplied filesystem paths."""
import threading
import uuid
from pathlib import Path
from engine.core import atomic_json, read_json

class Collections:
    def __init__(self, root):
        self.path = Path(root) / 'folders.json'
        self.lock = threading.RLock()

    @staticmethod
    def name(value):
        if not isinstance(value, str):
            raise ValueError('Nama harus berupa teks.')
        value = value.strip()
        if not value or len(value) > 100 or any(ord(c) < 32 for c in value):
            raise ValueError('Nama wajib diisi, maksimal 100 karakter, tanpa karakter kontrol.')
        return value

    def list(self):
        with self.lock:
            return read_json(self.path, [])

    def validate(self, folder):
        if folder == '':return ''
        if not isinstance(folder,str) or not any(f['id']==folder for f in self.list()):
            raise ValueError('Folder tidak ditemukan.')
        return folder

    def create(self, name):
        name = self.name(name)
        with self.lock:
            folders = self.list()
            if any(f['name'].casefold()==name.casefold() for f in folders):
                raise ValueError('Nama folder sudah digunakan.')
            folder = {'id':uuid.uuid4().hex, 'name':name}
            atomic_json(self.path, folders+[folder])
            return folder

    def rename(self, folder, name):
        name = self.name(name)
        with self.lock:
            self.validate(folder)
            folders = self.list()
            if any(f['id']!=folder and f['name'].casefold()==name.casefold() for f in folders):
                raise ValueError('Nama folder sudah digunakan.')
            for f in folders:
                if f['id']==folder:f['name']=name
            atomic_json(self.path, folders)
        return {'renamed':folder}

    def delete(self, folder):
        with self.lock:
            if not folder:
                raise ValueError('Folder tidak ditemukan.')
            self.validate(folder)
            # Clear only collection references. Never remove recording files.
            updates = []
            for directory in self.path.parent.iterdir():
                if directory.is_symlink() or not directory.is_dir():
                    continue
                metadata = directory / 'metadata.json'
                if not metadata.is_file() or metadata.is_symlink():
                    continue
                value = read_json(metadata, {})
                if value.get('folder') == folder:
                    updates.append((metadata, value))
            for metadata, value in updates:
                atomic_json(metadata, {**value, 'folder': ''})
            atomic_json(self.path, [f for f in self.list() if f['id'] != folder])
            return {'deleted': folder, 'moved': len(updates)}

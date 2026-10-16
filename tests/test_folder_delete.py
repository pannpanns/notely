import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend'))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend/engine'))
from engine.folder_store import Collections

class FolderDeletion(unittest.TestCase):
    def test_preserves_recordings_and_other_folders(self):
        with tempfile.TemporaryDirectory() as tmp:
            folders = Collections(tmp)
            first = folders.create('Pertama')['id']
            other = folders.create('Lainnya')['id']
            for name, folder in [('a', first), ('b', first), ('c', other)]:
                directory = Path(tmp) / name
                directory.mkdir()
                (directory / 'metadata.json').write_text(json.dumps({'folder': folder, 'judul': name}))
                (directory / 'audio.flac').write_bytes(b'audio original')
                (directory / 'transkrip.txt').write_text('transkrip original')
            result = folders.delete(first)
            self.assertEqual(result['moved'], 2)
            self.assertEqual(folders.list(), [{'id': other, 'name': 'Lainnya'}])
            for name in ('a', 'b', 'c'):
                directory = Path(tmp) / name
                meta = json.loads((directory / 'metadata.json').read_text())
                self.assertEqual(meta['folder'], other if name == 'c' else '')
                self.assertEqual(meta['judul'], name)
                self.assertEqual((directory / 'audio.flac').read_bytes(), b'audio original')
                self.assertEqual((directory / 'transkrip.txt').read_text(), 'transkrip original')
            for bad in ('', first, '../escape'):
                with self.assertRaises(ValueError):
                    folders.delete(bad)

    def test_metadata_failure_keeps_folder(self):
        with tempfile.TemporaryDirectory() as tmp:
            folders = Collections(tmp)
            folder = folders.create('Tetap ada')['id']
            directory = Path(tmp) / 'recording'
            directory.mkdir()
            (directory / 'metadata.json').write_text(json.dumps({'folder': folder}))
            with patch('engine.folder_store.atomic_json', side_effect=OSError('disk error')):
                with self.assertRaises(OSError):
                    folders.delete(folder)
            self.assertEqual(folders.validate(folder), folder)

if __name__ == '__main__':
    unittest.main()

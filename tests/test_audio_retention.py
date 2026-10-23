import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend'))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend' / 'engine'))
from engine.core import new_session, read_json
from engine.recording import RecordingEngine


class FakeUI:
    def __init__(self):
        self.events = []

    def send(self, event, value):
        self.events.append((event, value))


class AudioRetentionTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.session = new_session(self.root, 'Tes retensi audio')
        self.ui = FakeUI()
        (self.session / 'audio_komputer.flac').write_bytes(b'computer audio')
        (self.session / 'audio_mikrofon.flac').write_bytes(b'microphone audio')

    def engine(self):
        engine = RecordingEngine(self.ui, self.session, [], 'Whisper small (umum)', 'Indonesia')
        engine.capture_done.set()
        engine.recorded_blocks = 1
        engine.processed_blocks = 1
        engine.segments = [{'start': 0.0, 'end': 2.0, 'source': 'Peserta (komputer)', 'text': 'Contoh transkrip.'}]
        return engine

    def test_computer_audio_removed_before_compression_after_successful_transcription(self):
        engine = self.engine()

        def compress_remaining_audio(session, progress=None):
            self.assertFalse((session / 'audio_komputer.flac').exists())
            return ['audio_mikrofon.flac'], []

        with patch('engine.recording.compress_session_audio', side_effect=compress_remaining_audio):
            engine._complete()

        self.assertFalse((self.session / 'audio_komputer.flac').exists())
        self.assertTrue((self.session / 'audio_mikrofon.flac').exists())
        meta = read_json(self.session / 'metadata.json', {})
        self.assertTrue(meta['audio_komputer_dihapus'])
        self.assertEqual(meta['audio'], ['audio_mikrofon.flac'])
        self.assertTrue((self.session / 'transkrip.txt').exists())

    def test_computer_audio_preserved_for_partial_transcription(self):
        engine = self.engine()
        with patch('engine.recording.compress_session_audio', return_value=(
                ['audio_komputer.flac', 'audio_mikrofon.flac'], [])):
            engine._complete(partial=True)

        self.assertTrue((self.session / 'audio_komputer.flac').exists())
        meta = read_json(self.session / 'metadata.json', {})
        self.assertFalse(meta['audio_komputer_dihapus'])
        self.assertIn('audio_komputer.flac', meta['audio'])

    def test_computer_audio_preserved_when_transcription_failed(self):
        engine = self.engine()
        engine.failures.append('Model gagal dimuat: test failure')
        with patch('engine.recording.compress_session_audio', return_value=(
                ['audio_komputer.flac', 'audio_mikrofon.flac'], [])):
            engine._complete()

        self.assertTrue((self.session / 'audio_komputer.flac').exists())
        meta = read_json(self.session / 'metadata.json', {})
        self.assertFalse(meta['audio_komputer_dihapus'])


if __name__ == '__main__':
    unittest.main()

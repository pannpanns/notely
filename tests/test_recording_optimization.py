import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend'))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend/engine'))
from core import StreamingMono16kResampler, to_mono_16k
from model_runtime import transcribe_options
from speech_chunking import PauseChunker
from engine.recording import RecordingEngine


class _FakeFrame:
    def __init__(self, array):
        self.array = np.asarray(array, dtype=np.float32)

    @classmethod
    def from_ndarray(cls, array, format, layout):
        return cls(array)

    def to_ndarray(self):
        return self.array


class _FakeAudioResampler:
    instances = 0

    def __init__(self, format, layout, rate):
        type(self).instances += 1
        self.flushed = False

    def resample(self, frame):
        if frame is None:
            self.flushed = True
            return []
        return [frame]


class RecordingOptimizationTests(unittest.TestCase):
    def test_streaming_resampler_reuses_one_filter(self):
        fake_av = types.SimpleNamespace(AudioFrame=_FakeFrame, AudioResampler=_FakeAudioResampler)
        _FakeAudioResampler.instances = 0
        with patch.dict(sys.modules, {'av': fake_av}):
            resampler = StreamingMono16kResampler(48000)
            first = resampler.process(np.ones(16, dtype=np.float32))
            second = resampler.process(np.full(16, 0.5, dtype=np.float32))
            tail = resampler.flush()
        self.assertEqual(_FakeAudioResampler.instances, 1)
        self.assertEqual(len(first), 16)
        self.assertEqual(len(second), 16)
        self.assertEqual(len(tail), 0)
        with self.assertRaises(RuntimeError):
            resampler.process(np.ones(1, dtype=np.float32))

    def test_16khz_path_does_not_need_pyav(self):
        audio = np.linspace(-1, 1, 32, dtype=np.float32)
        with patch.dict(sys.modules, {'av': None}):
            result = to_mono_16k(audio, 16000)
        np.testing.assert_array_equal(result, audio)

    def test_transcription_uses_smaller_beam_for_lower_load(self):
        self.assertEqual(transcribe_options('id', preview=True)['beam_size'], 1)
        self.assertEqual(transcribe_options('id')['beam_size'], 2)

    def test_low_resource_recording_is_default_and_queue_is_bounded(self):
        with patch.dict('os.environ', {}, clear=True):
            engine = RecordingEngine(object(), Path('.'), [], 'model', 'Indonesia')
        self.assertFalse(engine.live_preview_enabled)
        self.assertEqual(engine.audio_tasks.maxsize, 96)

    def test_live_preview_can_be_enabled_explicitly(self):
        with patch.dict('os.environ', {}, clear=True):
            engine = RecordingEngine(object(), Path('.'), [], 'model', 'Indonesia', live_preview=True)
        self.assertTrue(engine.live_preview_enabled)

    def test_pause_chunker_can_reuse_precomputed_rms(self):
        chunker = PauseChunker(rate=16000, silence_seconds=0.1, max_seconds=0.5)
        speech = np.full(2048, 0.1, dtype=np.float32)
        self.assertEqual(chunker.feed(speech, rms=0.1), [])
        self.assertTrue(chunker.active)


if __name__ == '__main__':
    unittest.main()

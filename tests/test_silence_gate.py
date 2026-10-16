"""Silence regression: actual bundled VAD plus decoder confidence guard."""
import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend/engine'))
from model_runtime import transcribe_options
from text_quality import accept_speech_segment, sanitize_hypothesis

class SilenceGateTests(unittest.TestCase):
    def test_silence_and_low_noise_are_not_speech(self):
        from faster_whisper.vad import get_speech_timestamps, VadOptions
        for preview in (False, True):
            options = transcribe_options('id', preview=preview)
            self.assertTrue(options['vad_filter'])
            for audio in (np.zeros(32000, dtype=np.float32),
                          np.random.default_rng(7).normal(0, .004, 32000).astype(np.float32)):
                self.assertEqual(get_speech_timestamps(audio, VadOptions(**options['vad_parameters'])), [])

    def test_genuine_thanks_is_preserved(self):
        segment = SimpleNamespace(text='Terima kasih.', no_speech_prob=.02, avg_logprob=-.2)
        self.assertTrue(accept_speech_segment(segment))
        self.assertEqual(sanitize_hypothesis(segment.text)[0], 'Terima kasih.')

    def test_uncertain_silence_hypothesis_is_rejected(self):
        self.assertFalse(accept_speech_segment(SimpleNamespace(
            text='Terima kasih.', no_speech_prob=.92, avg_logprob=-1.4)))
        self.assertTrue(accept_speech_segment(SimpleNamespace(
            text='Pembicaraan pelan', no_speech_prob=.05, avg_logprob=-1.1)))

if __name__ == '__main__':
    unittest.main()

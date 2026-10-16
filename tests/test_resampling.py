import sys
import unittest
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend/engine'))
from core import to_mono_16k


class ResamplingTests(unittest.TestCase):
    def test_rates_duration_dtype_and_pitch(self):
        for rate in (8000, 16000, 22050, 44100, 48000):
            with self.subTest(rate=rate):
                t = np.arange(rate) / rate
                audio = np.sin(2*np.pi*1000*t).astype(np.float32)
                out = to_mono_16k(audio, rate)
                self.assertLessEqual(abs(len(out)-16000), 1)
                self.assertEqual(out.dtype, np.float32)
                peak = np.argmax(abs(np.fft.rfft(out))) * 16000 / len(out)
                self.assertLess(abs(peak-1000), 2)
                self.assertGreater(float(np.std(out[100:-100])), 0.65)

    def test_antialiasing(self):
        t = np.arange(48000)/48000
        out = to_mono_16k(np.sin(2*np.pi*12000*t), 48000)
        self.assertLess(float(np.std(out[100:-100])), 0.01)

    def test_stereo_empty_short_and_bad_input(self):
        audio = np.column_stack((np.ones(48000), -np.ones(48000)))
        self.assertTrue(np.allclose(to_mono_16k(audio), 0))
        self.assertEqual(len(to_mono_16k([])), 0)
        self.assertEqual(to_mono_16k([0.0], 48000).dtype, np.float32)
        for rate in (0, -1, 1.5):
            with self.assertRaises(ValueError): to_mono_16k([0], rate)
        with self.assertRaises(ValueError): to_mono_16k(np.zeros((2,2,2)))

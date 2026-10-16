import sys,unittest
from pathlib import Path
import numpy as np
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend/engine'))
from audio_spectrum import spectrum_bands
class SpectrumTests(unittest.TestCase):
    def test_silence_is_flat(self):self.assertEqual(spectrum_bands(np.zeros(2048)),[0.]*18)
    def test_tone_peak_matches_frequency(self):
        samples=.4*np.sin(2*np.pi*1000*np.arange(2048)/16000)
        values=spectrum_bands(samples);edges=np.geomspace(60,7600,19)
        peak=int(np.argmax(values));self.assertLessEqual(edges[peak],1000);self.assertGreater(edges[peak+1],1000)
        self.assertGreater(max(values),.7)
    def test_louder_input_has_larger_peak(self):
        samples=np.sin(2*np.pi*2000*np.arange(683)/16000)
        self.assertGreater(max(spectrum_bands(samples*.5)),max(spectrum_bands(samples*.005)))
    def test_empty_nonfinite_and_bounds(self):
        self.assertEqual(spectrum_bands([]),[0.]*18)
        self.assertTrue(all(0<=x<=1 for x in spectrum_bands([float('nan'),float('inf')]*500)))

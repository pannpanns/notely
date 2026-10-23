"""Compact log-frequency spectrum from captured PCM, never synthetic bars."""
import numpy as np

_NFFT = 2048
_WINDOW = np.hanning(_NFFT).astype(np.float32)
_WINDOW_SUM = max(float(_WINDOW.sum()), 1.0)
_DEFAULT_FREQUENCIES = np.fft.rfftfreq(_NFFT, 1 / 16000)
_DEFAULT_EDGES = np.geomspace(60, 7600, 19)
_DEFAULT_GROUPS = [
    np.flatnonzero((_DEFAULT_FREQUENCIES >= lo) & (_DEFAULT_FREQUENCIES < hi))
    for lo, hi in zip(_DEFAULT_EDGES[:-1], _DEFAULT_EDGES[1:])
]

def spectrum_bands(samples, sample_rate=16000, bands=18):
    audio = np.nan_to_num(np.asarray(samples, dtype=np.float32).reshape(-1), nan=0., posinf=0., neginf=0.)
    if len(audio) < 8:
        return [0.] * bands
    audio = audio[-_NFFT:]
    if len(audio) < _NFFT:
        audio = np.pad(audio, (_NFFT - len(audio), 0))
    window = _WINDOW if sample_rate == 16000 else np.hanning(_NFFT).astype(np.float32)
    magnitude = np.abs(np.fft.rfft(audio * window, n=_NFFT)) * 2 / (float(window.sum()) if sample_rate != 16000 else _WINDOW_SUM)
    if sample_rate == 16000 and bands == 18:
        groups = _DEFAULT_GROUPS
    else:
        frequencies = np.fft.rfftfreq(_NFFT, 1 / sample_rate)
        edges = np.geomspace(60, min(7600, sample_rate / 2), bands + 1)
        groups = [np.flatnonzero((frequencies >= lo) & (frequencies < hi)) for lo, hi in zip(edges[:-1], edges[1:])]
    values = []
    for group in groups:
        amplitude = float(magnitude[group].max()) if len(group) else 0.
        values.append(round(float(np.clip((20 * np.log10(max(amplitude, 1e-8)) + 65) / 65, 0, 1)), 4))
    return values

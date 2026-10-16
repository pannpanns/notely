"""Compact log-frequency spectrum from captured PCM, never synthetic bars."""
import numpy as np


def spectrum_bands(samples, sample_rate=16000, bands=18):
    audio=np.nan_to_num(np.asarray(samples,dtype=np.float32).reshape(-1),nan=0.,posinf=0.,neginf=0.)
    if len(audio)<8:return [0.]*bands
    audio=audio[-2048:]
    window=np.hanning(len(audio))
    magnitude=np.abs(np.fft.rfft(audio*window,n=2048))*2/max(float(window.sum()),1.)
    frequencies=np.fft.rfftfreq(2048,1/sample_rate)
    edges=np.geomspace(60,min(7600,sample_rate/2),bands+1)
    values=[]
    for lower,upper in zip(edges[:-1],edges[1:]):
        group=magnitude[(frequencies>=lower)&(frequencies<upper)]
        amplitude=float(group.max()) if len(group) else 0.
        values.append(round(float(np.clip((20*np.log10(max(amplitude,1e-8))+65)/65,0,1)),4))
    return values

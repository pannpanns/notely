"""Windows WASAPI loopback/microphone input via PyAudioWPatch.

Loopback inputs are listed directly, rather than guessing a match between two
unrelated speaker and microphone identifiers. Import of PyAudioWPatch is lazy
so offline unit tests and module-level imports can run on non-Windows systems.
"""
from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass


@dataclass(frozen=True)
class AudioDevice:
    name: str
    index: int
    rate: int
    channels: int
    kind: str  # loopback | mic


def enumerate_devices():
    """Return (loopback devices, mic devices, default loopback index)."""
    import pyaudiowpatch as pa
    with pa.PyAudio() as p:
        raw = list(p.get_loopback_device_info_generator())
        outputs = [AudioDevice(str(d['name']), int(d['index']),
                               int(round(float(d['defaultSampleRate']))),
                               min(2, int(d['maxInputChannels'])), 'loopback')
                   for d in raw if int(d.get('maxInputChannels', 0)) > 0]
        wasapi_index = None
        try:
            wasapi_index = int(p.get_host_api_info_by_type(pa.paWASAPI)['index'])
        except (OSError, KeyError, ValueError):
            pass
        all_inputs = list(p.get_device_info_generator())
        microphones = [AudioDevice(str(d['name']), int(d['index']),
                                   int(round(float(d['defaultSampleRate']))),
                                   1, 'mic')
                       for d in all_inputs if int(d.get('maxInputChannels', 0)) > 0
                       and not d.get('isLoopbackDevice', False)
                       and (wasapi_index is None or int(d.get('hostApi', -1)) == wasapi_index)]
        # Some Bluetooth/USB microphone drivers do not advertise a WASAPI input.
        if not microphones:
            microphones = [AudioDevice(str(d['name']), int(d['index']),
                                       int(round(float(d['defaultSampleRate']))),
                                       1, 'mic')
                           for d in all_inputs if int(d.get('maxInputChannels', 0)) > 0
                           and not d.get('isLoopbackDevice', False)]
        default_index = outputs[0].index if outputs else None
        try:
            selected = p.get_default_wasapi_loopback()
            if any(d.index == int(selected['index']) for d in outputs):
                default_index = int(selected['index'])
        except (OSError, KeyError, ValueError, AttributeError):
            pass
    return outputs, microphones, default_index


@contextmanager
def open_input(device: AudioDevice, frames_per_buffer: int = 2048):
    """Open native-rate PCM16 capture. The caller reads via stream.read()."""
    import pyaudiowpatch as pa
    p = pa.PyAudio()
    stream = None
    try:
        stream = p.open(format=pa.paInt16, channels=device.channels, rate=device.rate,
                        input=True, input_device_index=device.index,
                        frames_per_buffer=frames_per_buffer)
        yield stream
    finally:
        if stream is not None:
            try:
                stream.stop_stream()
            except Exception:
                pass
            try:
                stream.close()
            except Exception:
                pass
        p.terminate()


def int16_mono(raw: bytes, channels: int):
    """Convert interleaved PCM16 into mono float32 [-1, 1] for diagnostics."""
    import numpy as np
    audio = np.frombuffer(raw, dtype='<i2')
    if channels > 1:
        audio = audio[:len(audio) // channels * channels].reshape(-1, channels)
        return audio.astype(np.float32).mean(axis=1) / 32768.0
    return audio.astype(np.float32) / 32768.0


def diagnose(device: AudioDevice, duration: float = 5.0):
    """Measure sound from a selected output while the user plays meeting audio."""
    import numpy as np
    import time
    rms_peak = 0.0
    count = 0
    with open_input(device) as stream:
        end_at = time.monotonic() + duration
        while time.monotonic() < end_at:
            buf = stream.read(2048, exception_on_overflow=False)
            samples = int16_mono(buf, device.channels)
            if len(samples):
                count += 1
                rms_peak = max(rms_peak, float(np.sqrt(np.mean(samples * samples))))
    if not count:
        raise RuntimeError('Driver tidak memberikan data audio. Coba pilih output lain.')
    return rms_peak

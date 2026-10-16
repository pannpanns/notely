"""Crash-conscious compact meeting audio storage.

Never create transcription chunks on disk; only one continuous WAV per source.
Convert losslessly after the final transcript, verify FLAC before deleting WAV.
"""
from pathlib import Path


class StorageError(RuntimeError):
    pass


def compress_recording(wav_path: Path, *, sf_module=None):
    """Returns final path. Failure leaves WAV intact; transient *.part removed."""
    wav_path = Path(wav_path)
    if not wav_path.exists():
        return None
    if sf_module is None:
        import soundfile as sf_module
    sf = sf_module
    target = wav_path.with_suffix('.flac')
    temporary = wav_path.with_suffix('.flac.part')
    temporary.unlink(missing_ok=True)
    try:
        with sf.SoundFile(str(wav_path), 'r') as reader:
            if reader.channels != 1 or reader.samplerate != 16000:
                raise StorageError('Format rekaman tidak didukung untuk kompresi.')
            frames = reader.frames
            with sf.SoundFile(str(temporary), 'w', samplerate=reader.samplerate,
                              channels=1, format='FLAC', subtype='PCM_16') as writer:
                while True:
                    block = reader.read(65536, dtype='int16', always_2d=True)
                    if len(block) == 0:
                        break
                    writer.write(block)
        info = sf.info(str(temporary))
        if info.frames != frames or info.samplerate != 16000 or info.channels != 1:
            raise StorageError('Verifikasi FLAC gagal. WAV asli dipertahankan.')
        # Verify PCM bytes also decode identically, not merely the frame count.
        import numpy as np
        with sf.SoundFile(str(wav_path), 'r') as original, sf.SoundFile(str(temporary), 'r') as packed:
            while True:
                a = original.read(65536, dtype='int16', always_2d=True)
                b = packed.read(65536, dtype='int16', always_2d=True)
                if not np.array_equal(a, b):
                    raise StorageError('Isi FLAC berbeda dari WAV. File asli dipertahankan.')
                if len(a) == 0:
                    break
        # Avoid replacing good audio with a larger file after failed compression.
        if temporary.stat().st_size >= wav_path.stat().st_size:
            temporary.unlink(missing_ok=True)
            return wav_path
        temporary.replace(target)
        wav_path.unlink()
        return target
    except Exception:
        temporary.unlink(missing_ok=True)
        raise


def compress_session_audio(session: Path, progress=None):
    results, errors = [], []
    for src in ('komputer', 'mikrofon'):
        wav = Path(session) / f'audio_{src}.wav'
        if not wav.exists():
            continue
        try:
            if progress: progress(f'Mengompres rekaman {src} tanpa mengurangi kualitas...')
            result = compress_recording(wav)
            if result is not None:
                results.append(result.name)
        except Exception as error:
            errors.append(f'{wav.name}: kompresi gagal ({error}); WAV asli tetap aman.')
    return results, errors

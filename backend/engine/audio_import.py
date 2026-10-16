"""Decode local media incrementally using bundled PyAV, into playback-safe FLAC."""
from pathlib import Path

AUDIO_EXTENSIONS = frozenset('.wav .wave .flac .mp3 .mp2 .ogg .oga .opus .m4a .m4b .aac .aiff .aif .aifc .wma .amr .ac3 .eac3 .au .snd .caf .ape .wv .mka .webm .mp4 .mkv .mov .avi .3gp .3g2 .ts .mts .m2ts .mpeg .mpg .weba'.split())
MAX_SAMPLES = 16000 * 60 * 60 * 24  # Bound decoded output, including compressed inputs.


def normalize_audio(source, destination):
    import av
    import soundfile as sf
    total = 0
    try:
        # Disable playlists/network protocols: imported media must remain local.
        with av.open(str(source), options={'protocol_whitelist': 'file,pipe'}) as container:
            if not container.streams.audio:
                raise ValueError('File tidak memiliki trek audio.')
            stream = container.streams.audio[0]
            resampler = av.AudioResampler(format='s16', layout='mono', rate=16000)
            with sf.SoundFile(str(destination), 'w', samplerate=16000, channels=1,
                              format='FLAC', subtype='PCM_16') as output:
                def write(frames):
                    nonlocal total
                    for frame in frames:
                        total += frame.samples
                        if total > MAX_SAMPLES:
                            raise ValueError('Durasi audio maksimal 24 jam.')
                        output.write(frame.to_ndarray().reshape(-1))
                for frame in container.decode(stream):
                    write(resampler.resample(frame))
                write(resampler.resample(None))
        if total == 0:
            raise ValueError('Audio kosong atau tidak dapat dibaca.')
        return {'duration': total / 16000, 'sampleRate': 16000}
    except Exception as exc:
        Path(destination).unlink(missing_ok=True)
        if isinstance(exc, (ValueError, OSError)):
            raise
        raise ValueError('Audio tidak dapat dibaca. File mungkin rusak atau codec tidak didukung.') from exc

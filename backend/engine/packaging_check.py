"""Offline checks executed inside the frozen sidecar by CI."""
import json
from pathlib import Path
import traceback


def run(destination):
    try:
        import av
        import ctranslate2
        import huggingface_hub
        import numpy as np
        import onnxruntime
        import pyaudiowpatch
        import soundfile as sf
        import tokenizers
        from faster_whisper import WhisperModel
        from faster_whisper.vad import get_vad_model, get_speech_timestamps, VadOptions
        from core import to_mono_16k
        import io
        samples = to_mono_16k(np.zeros(4800, dtype=np.float32), 48000)
        assert len(samples) == 1600
        buf = io.BytesIO()
        sf.write(buf, samples, 16000, format='FLAC')
        buf.seek(0)
        decoded, rate = sf.read(buf)
        assert rate == 16000 and len(decoded) == 1600
        get_vad_model()  # bundled ONNX model and native runtime must both work
        from model_runtime import transcribe_options
        options = VadOptions(**transcribe_options('id')['vad_parameters'])
        assert get_speech_timestamps(np.zeros(32000, dtype=np.float32), options) == []
        result = {'ok': True, 'av': av.__version__, 'ct2': ctranslate2.__version__}
    except Exception:
        result = {'ok': False, 'error': traceback.format_exc()}
    Path(destination).write_text(json.dumps(result, indent=2), encoding='utf-8')
    return 0 if result['ok'] else 1

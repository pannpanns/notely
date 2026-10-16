"""Model loading and NVIDIA CUDA selection for streaming transcription.

External Indonesian fine-tunes are opt-in, downloaded on first use. No local
meeting recording is uploaded by this module; only model files downloaded.
"""
from __future__ import annotations
import os
import subprocess
import sys
import threading
from pathlib import Path
from cuda_runtime import register_cuda_dlls, dll_report

MODEL_CHOICES = (
    'Indonesia Small (terlatih)',
    'Indonesia Turbo (terlatih, GPU)',
    'Whisper small (umum)',
    'Whisper medium (umum)',
    'Whisper turbo (umum)',
    'Whisper large-v3 (umum)',
)
MODEL_IDS = {
    MODEL_CHOICES[0]: 'maleo-ai/whisper-small-id',
    MODEL_CHOICES[1]: 'maleo-ai/whisper-large-v3-turbo-id',
    MODEL_CHOICES[2]: 'small',
    MODEL_CHOICES[3]: 'medium',
    MODEL_CHOICES[4]: 'turbo',
    MODEL_CHOICES[5]: 'large-v3',
}
MODEL_LANGUAGE = {'id': 'id', 'en': 'en', 'Otomatis': None}


def discover_nvidia():
    """Find GPU name using nvidia-smi (informational; not a proof of usable CUDA)."""
    try:
        result = subprocess.run(['nvidia-smi', '--query-gpu=name,memory.total', '--format=csv,noheader'],
                                timeout=7, capture_output=True, text=True, check=True,
                                creationflags=subprocess.CREATE_NO_WINDOW if sys.platform=='win32' else 0)
        return result.stdout.strip() or 'NVIDIA GPU terdeteksi'
    except (OSError, subprocess.SubprocessError):
        return ''


def cuda_device_count():
    register_cuda_dlls()
    import ctranslate2
    return ctranslate2.get_cuda_device_count()


def runtime_report():
    name = discover_nvidia()
    try:
        devices = cuda_device_count()
        ct = f'CTranslate2: {devices} GPU CUDA terdeteksi.'
    except Exception as exc:
        devices = 0
        ct = f'CTranslate2 gagal memeriksa CUDA: {exc}'
    return (f'GPU NVIDIA: {name or "tidak ditemukan melalui nvidia-smi"}\n{ct}\n'
            + dll_report() + '\nDeteksi GPU belum membuktikan CUDA siap. Inferensi model akan mengujinya.')


def _model_folder(model_choice, progress=None):
    from model_download import download
    return download(MODEL_IDS.get(model_choice, model_choice), progress)


def transcribe_options(language: str | None, *, preview=False):
    """Natural Indonesian decoding, no hotwords or hard-coded corrections."""
    return dict(language=language, task='transcribe', beam_size=1 if preview else 3,
                best_of=1, temperature=0.0, condition_on_previous_text=False,
                # CT2 decoding discourages repeated loops; default Whisper
                # thresholds reject highly compressed/hallucinated hypotheses.
                repetition_penalty=1.12 if preview else 1.08,
                no_repeat_ngram_size=3, compression_ratio_threshold=2.2,
                log_prob_threshold=-1.0, no_speech_threshold=.60,
                # Energy chunking alone also admits fan/hiss/music. Silero gates
                # both provisional and final decoding without editing saved audio.
                vad_filter=True,
                vad_parameters=dict(threshold=.5, min_speech_duration_ms=160,
                                    min_silence_duration_ms=400, speech_pad_ms=200),
                without_timestamps=False)


class ModelManager:
    def __init__(self):
        self.lock = threading.Lock()
        self.model = None
        self.identity = None
        self.backend = None

    def get(self, choice, gpu_choice='Otomatis', progress=None):
        """Returns (model, description, optional_warning). Auto only falls back visibly.

        For explicit 'CUDA', errors are raised rather than silently using CPU.
        """
        with self.lock:
            key = (choice, gpu_choice)
            if self.model is not None and self.identity == key:
                return self.model, self.backend, None
            register_cuda_dlls()  # Important: BEFORE importing CTranslate2/faster-whisper.
            from faster_whisper import WhisperModel
            from numpy import zeros
            selected = 'cpu' if gpu_choice == 'CPU' else 'cuda'
            warning = None
            if selected == 'cuda':
                try:
                    if cuda_device_count() <= 0:
                        raise RuntimeError('CTranslate2 tidak melihat GPU CUDA.')
                except Exception as exc:
                    if gpu_choice == 'CUDA':
                        raise RuntimeError(f'CUDA dipilih, tetapi tidak siap: {exc}. Komponen GPU harus disiapkan otomatis pada pengaturan awal. Jika gagal, gunakan CPU.') from exc
                    warning = f'CUDA tidak tersedia ({exc}); beralih ke CPU.'
                    selected = 'cpu'
            if choice == 'Indonesia Turbo (terlatih, GPU)' and selected == 'cpu':
                if gpu_choice == 'CUDA':
                    raise RuntimeError('Model Indonesia Turbo membutuhkan CUDA NVIDIA yang berfungsi.')
                warning = (warning or 'Tidak ada CUDA.') + ' Menggunakan model Indonesia Small di CPU.'
            model_path = _model_folder('Indonesia Small (terlatih)' if
                                       choice == 'Indonesia Turbo (terlatih, GPU)' and selected == 'cpu'
                                       else choice, progress)
            def load_and_check(device):
                mode = 'int8_float16' if device == 'cuda' else 'int8'
                loaded = WhisperModel(model_path, device=device, compute_type=mode, num_workers=2)
                # Consume lazy generator to force cuBLAS/cuDNN kernel execution now.
                # A detected CUDA device is not enough to claim GPU actually works.
                warmup = transcribe_options('id', preview=True)
                warmup['vad_filter'] = False  # force decoder/CUDA kernels on silence
                gen, _ = loaded.transcribe(zeros(16000 * 2, dtype='float32'), **warmup)
                list(gen)
                return loaded, mode
            try:
                instance, precision = load_and_check(selected)
            except Exception as exc:
                if selected == 'cpu' or gpu_choice == 'CUDA':
                    raise RuntimeError(f'Gagal memuat model di {selected.upper()}: {exc}. Jika CUDA, klik Pasang CUDA RTX lalu restart; cek HASIL_CEK_GPU.txt.') from exc
                warning = f'CUDA gagal saat uji inferensi ({exc}). CPU digunakan; periksa CUDA/cuDNN.'
                selected = 'cpu'
                if choice == 'Indonesia Turbo (terlatih, GPU)':
                    model_path = _model_folder('Indonesia Small (terlatih)', progress)
                    warning += ' Model diganti Indonesia Small.'
                instance, precision = load_and_check('cpu')
            self.model = instance
            self.identity = key
            self.backend = f'{selected.upper()} ({precision})'
            return instance, self.backend, warning

    def release(self):
        """Release the cached Whisper model before Ollama uses a 6 GB RTX card.

        Do not call while a recording or live transcription worker is active.
        """
        import gc
        with self.lock:
            self.model = None
            self.identity = None
            self.backend = None
        gc.collect()

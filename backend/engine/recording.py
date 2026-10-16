"""Reusable Notely live capture engine migrated from v1.8 (Tkinter UI removed)."""
from __future__ import annotations
import os, queue, shutil, threading, time, wave, math
from pathlib import Path
from .audio_spectrum import spectrum_bands
from .audio_capture import enumerate_devices, open_input, int16_mono, diagnose
from .model_runtime import MODEL_CHOICES, ModelManager, transcribe_options, runtime_report
from .training_data import save_verified_example
from .speech_chunking import PauseChunker
from .transcript_format import timeline_text
from .text_quality import sanitize_hypothesis, preview_visible, accept_speech_segment
from .storage import compress_session_audio
from .core import (SAMPLE_RATE, default_output_root, export_transcript, new_session,
    nice_seconds, offline_summary, read_json, refresh_metadata, sort_segments, to_mono_16k, write_wav)
CHUNK_SECONDS=8.0
OVERLAP_SECONDS=0.0
PREVIEW_MIN_SECONDS=1.25
PREVIEW_INTERVAL=.85
PREVIEW_MIN_ADVANCE=.55

class RecordingEngine:
    def __init__(self, ui, session, devices, model_name, language, gpu_choice="Otomatis"):
        self.ui = ui
        self.session = session
        self.devices = devices  # list of (friendly label, Windows WASAPI AudioDevice)
        self.model_name = model_name
        self.language = None if language == 'Otomatis' else ('id' if language == 'Indonesia' else 'en')
        self.stop_event = threading.Event()
        self.cancel_transcription = threading.Event()
        self.capture_done = threading.Event()
        self.finalized = False
        self.gpu_choice = gpu_choice
        self.live_lock = threading.Lock()
        self.live_audio = {}  # source -> (piece_id, start, preview audio, core_start)
        self.preview_progress = {}  # source -> (piece_id, frames, completed_at)
        self.preview_thread = None
        self.last_final_piece = {}
        self._stop_lock = threading.Lock()
        self._stop_started = False
        self._all_capture_started = threading.Event()
        # Max ~96 * 8s * 16k * 4 bytes = 49MB, even if GPU processing stops.
        self.audio_tasks = queue.Queue(maxsize=96)
        self.skipped_blocks = 0
        self.threads = []
        self.started = time.monotonic()
        self.segments = []
        self.segments_lock = threading.Lock()
        self.failures = []
        self.recorded_blocks = 0
        self.processed_blocks = 0

    def _emit(self, event, value):
        # Old workers must not change a new session's UI after partial finish.
        if not self.finalized:
            self.ui.send(event, value)

    def start(self):
        t = threading.Thread(target=self._transcription_worker, daemon=True, name='Transcription')
        self.transcription_thread = t
        t.start()
        try:
            for label, device in self.devices:
                t = threading.Thread(target=self._capture, args=(label, device), daemon=True, name=f'Record {label}')
                self.threads.append(t)
                t.start()
        finally:
            self._all_capture_started.set()

    def stop(self):
        with self._stop_lock:
            if self._stop_started:
                return
            self._stop_started = True
            self.stop_event.set()
        threading.Thread(target=self._join_capture, daemon=True).start()

    def _join_capture(self):
        self._all_capture_started.wait(timeout=8)
        for t in self.threads:
            if t.ident is None:
                self.failures.append('Thread perekaman tidak dapat dimulai.')
                continue
            t.join(timeout=12)
            if t.is_alive():
                self.failures.append('Capture audio tidak berhasil berhenti sepenuhnya; periksa rekaman.')
        complete = not any(t.is_alive() for t in self.threads)
        self.capture_done.set()
        self._emit('capture_stopped', (self.recorded_blocks, self.processed_blocks, complete))
        # Signal end without blocking the capture-stop/UI thread indefinitely.
        while not self.cancel_transcription.is_set():
            try:
                self.audio_tasks.put(None, timeout=.5)
                break
            except queue.Full:
                continue

    def finish_now(self):
        """Save partial results and release UI without waiting for slow Whisper."""
        if not self.capture_done.is_set():
            return False
        self.cancel_transcription.set()
        threading.Thread(target=self._complete, args=(True,), daemon=True, name='Finish partial').start()
        return True

    def _complete(self, partial=False):
        # The final worker calls this in background; never block the UI thread.
        with self.segments_lock:
            if self.finalized:
                return
            self.finalized = True
            warnings = list(self.failures)
            if self.skipped_blocks:
                warnings.append(f'{self.skipped_blocks} potongan teks dilewati akibat antrean penuh. Audio lengkap tetap tersedia untuk ditranskripsikan ulang.')
            if partial:
                warnings.append('Transkripsi belum lengkap: gunakan Impor Audio untuk memulihkannya.')
            # Summary is deliberately AI-only in Studio 2. Legacy extractive summary removed.
            summ = ''
            try:
                export_transcript(self.session, self.segments)
            except Exception as exc:
                warnings.append(f'Gagal menyimpan teks: {exc}. Periksa ruang disk.')
                summ = ''
        if self.capture_done.is_set() and not any(t.is_alive() for t in self.threads):
            audio_paths, compression_errors = compress_session_audio(
                self.session, progress=lambda msg: self.ui.send('storage', msg))
            warnings.extend(compression_errors)
        else:
            audio_paths = [p.name for p in self.session.glob('audio_*.wav')]
            warnings.append('Perekaman belum tertutup sepenuhnya; WAV tetap disimpan tanpa kompresi.')
        try:
            refresh_metadata(self.session, status='sebagian' if partial else ('selesai' if not warnings else 'selesai_dengan_peringatan'),
                             peringatan=warnings, audio=audio_paths, potongan_diproses=self.processed_blocks,
                             potongan_terekam=self.recorded_blocks, potongan_terlewat=self.skipped_blocks,
                             selesai=time.strftime('%Y-%m-%d %H:%M:%S'))
        except Exception as exc:
            warnings.append(f'Metadata tidak berhasil disimpan: {exc}')
        self.ui.send('finished', (self.session, summ, warnings))

    def _capture(self, label, device):
        """Persist all audio; finalize transcription candidates on speech pauses.

        No silence trimming is applied to the permanent WAV. A pause chunker
        only determines the boundaries for Whisper and the readable timeline.
        """
        import numpy as np
        suffix = 'komputer' if label == 'Peserta (komputer)' else 'mikrofon'
        permanent_wav = self.session / f'audio_{suffix}.wav'
        chunker = PauseChunker(rate=SAMPLE_RATE, silence_seconds=.80,
                               max_seconds=CHUNK_SECONDS)
        sample_count = 0
        chunk_index = 0
        paragraph_id = 0
        max_rms = 0.0
        rms_acc, rms_blocks = 0.0, 0
        sample_offset = max(0.0, time.monotonic() - self.started)
        last_preview_publish = 0.0

        def submit_piece(start_sample, current, flush_reason):
            nonlocal chunk_index, paragraph_id
            # PauseChunker already adds 240 ms of pre-roll. The previous
            # version added another 650 ms, duplicating words at boundaries.
            core_start = sample_offset + start_sample / SAMPLE_RATE
            chunk_start = core_start
            paragraph_key = f'{suffix}_{paragraph_id}'
            # In-memory clip, never create a file for each transcribed phrase.
            # If AI cannot keep up, preserve full recording and skip only text jobs.
            try:
                self.audio_tasks.put_nowait((label, chunk_start, current, core_start, chunk_index, paragraph_key))
            except queue.Full:
                self.skipped_blocks += 1
                if self.skipped_blocks == 1:
                    self._emit('notice', 'Antrean AI penuh. Audio tetap lengkap; beberapa teks bisa dipulihkan dari rekaman.')
            with self.live_lock:
                self.live_audio.pop(label, None)
            self.recorded_blocks += 1
            self._emit('queue', (self.recorded_blocks, self.processed_blocks))
            chunk_index += 1
            # A chunk forced by maximum length belongs to the same paragraph;
            # a chunk closed by speech silence ends that paragraph.
            if flush_reason == 'pause':
                paragraph_id += 1

        try:
            with open_input(device) as reader, open(permanent_wav, 'wb') as raw_file, wave.open(raw_file, 'wb') as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(SAMPLE_RATE)
                self._emit('status', f'Merekam suara dari {device.name} ...')
                last_disk_check = last_sync = time.monotonic()
                while not self.stop_event.is_set():
                    if time.monotonic() - last_disk_check > 12:
                        last_disk_check = time.monotonic()
                        if shutil.disk_usage(self.session).free < 50 * 1024 * 1024:
                            raise OSError('Ruang disk kurang dari 50 MB; rekaman dihentikan agar file tidak rusak.')
                    raw = reader.read(2048, exception_on_overflow=False)
                    if not raw:
                        continue
                    mono = to_mono_16k(int16_mono(raw, device.channels), input_rate=device.rate)
                    if not len(mono):
                        continue
                    wav.writeframes((np.clip(mono, -1, 1) * 32767).astype('<i2').tobytes())
                    if time.monotonic() - last_sync >= 15:
                        raw_file.flush()
                        os.fsync(raw_file.fileno())
                        last_sync = time.monotonic()
                    sample_count += len(mono)
                    level = float(np.sqrt(np.mean(mono * mono)))
                    max_rms = max(max_rms, level)
                    rms_acc += level
                    rms_blocks += 1
                    if rms_blocks >= 3:
                        self._emit('volume', (label, rms_acc / rms_blocks))
                        self._emit('spectrum', {'source':label,'bands':spectrum_bands(mono),'rms':rms_acc/rms_blocks,'sampledAt':time.time()})
                        rms_acc = rms_blocks = 0
                    ready = chunker.feed(mono)
                    # While speaking, update the latest rolling hypothesis.
                    now = time.monotonic()
                    if not ready and now - last_preview_publish >= .45:
                        preview = chunker.preview(limit_seconds=4.0)
                        if preview is not None and len(preview[1]) >= SAMPLE_RATE * PREVIEW_MIN_SECONDS:
                            preview_start_idx, audio = preview
                            with self.live_lock:
                                self.live_audio[label] = (
                                    chunk_index, sample_offset + preview_start_idx / SAMPLE_RATE,
                                    audio, sample_offset + chunker.start_sample / SAMPLE_RATE)
                            last_preview_publish = now
                    for start_sample, clip in ready:
                        submit_piece(start_sample, clip, chunker.last_flush_reason)
                for start_sample, clip in chunker.finish():
                    submit_piece(start_sample, clip, 'stop')
                self._emit('status', f'Audio tersimpan ({nice_seconds(sample_count / SAMPLE_RATE)}).')
            if suffix == 'komputer' and (sample_count < SAMPLE_RATE or max_rms < .0006):
                self.failures.append('Audio komputer hening/tidak diterima. Periksa output rekaman dan Tes Suara.')
        except Exception as exc:
            err = f'{label}: {type(exc).__name__}: {exc}'
            self.failures.append(err)
            self._emit('error', err)
            self._emit('status', 'Perekaman berhenti akibat masalah perangkat audio.')
            if len(self.devices) == 1:
                self.stop()

    def _preview_candidate(self):
        """Return latest unfinalized audio window, never a growing backlog."""
        if self.stop_event.is_set() or self.cancel_transcription.is_set():
            return None
        now = time.monotonic()
        with self.live_lock:
            items = list(self.live_audio.items())
        # Prefer the source whose latest preview is oldest, for two-source meetings.
        items.sort(key=lambda pair: self.preview_progress.get(pair[0], (-1, 0, 0))[2])
        for label, (piece_id, start, samples, core_start) in items:
            if piece_id <= self.last_final_piece.get(label, -1):
                continue
            if len(samples) < SAMPLE_RATE * PREVIEW_MIN_SECONDS:
                continue
            old_id, frames, last = self.preview_progress.get(label, (-1, 0, 0))
            if old_id == piece_id and (now-last < PREVIEW_INTERVAL or
                                       len(samples)-frames < SAMPLE_RATE * PREVIEW_MIN_ADVANCE):
                continue
            return label, piece_id, start, samples, core_start
        return None

    def _preview_worker(self, model):
        """Low-latency rolling caption loop; never accumulates a transcript backlog.

        Whisper is not a token-streaming recognizer. Each iteration replaces the
        current provisional hypothesis; the final worker commits accurate text.
        """
        import numpy as np
        while not (self.stop_event.is_set() or self.cancel_transcription.is_set() or self.finalized):
            candidate = self._preview_candidate()
            if candidate is None:
                time.sleep(.06)
                continue
            label, piece_id, start, samples, core_start = candidate
            self.preview_progress[label] = (piece_id, len(samples), time.monotonic())
            try:
                if float(np.sqrt(np.mean(samples * samples))) < .001:
                    self._emit('preview', (label, piece_id, core_start, ''))
                    continue
                result, _ = model.transcribe(samples, **transcribe_options(self.language, preview=True))
                parts = []
                for seg in result:
                    if not accept_speech_segment(seg):
                        continue
                    if start + seg.end > core_start + .04 and seg.text.strip():
                        parts.append(seg.text.strip())
                # Do not overwrite a final piece or reappear after Stop.
                if (not self.stop_event.is_set() and not self.cancel_transcription.is_set()
                        and not self.finalized and piece_id > self.last_final_piece.get(label, -1)):
                    # Ignore stale output. A GPU inference may finish AFTER
                    # newer audio has been captured; replace, never queue.
                    preview_text = preview_visible(' '.join(parts))
                    if self.last_final_piece.get(label, -1) < piece_id:
                        self._emit('preview', (label, piece_id, core_start, preview_text))
            except Exception as exc:
                self._emit('notice', f'Teks langsung terganggu: {exc}. Rekaman tetap aman.')
                time.sleep(.3)

    def _transcription_worker(self):
        try:
            self._emit('status', f'Memuat {self.model_name}. Audio tetap direkam meski model belum siap...')
            model, backend, warning = self.ui.transcriber.get(self.model_name, self.gpu_choice)
            self._emit('backend', backend)
            if warning:
                self._emit('notice', warning)
            self._emit('status', f'Model siap: {backend}. Teks SEMENTARA diproses saat bicara, paralel dengan final.')
        except Exception as exc:
            self.failures.append(f'Model gagal dimuat: {exc}')
            self._emit('backend', 'GAGAL (audio tetap direkam)')
            self._emit('error', f'Tidak bisa memuat model: {exc}. Audio akan tetap tersimpan.')
            model = None
        # Independent GPU worker: final segment inference cannot starve live text.
        # CTranslate2 num_workers=2 allows concurrent inference from these threads.
        if model is not None and not self.stop_event.is_set():
            self.preview_thread = threading.Thread(target=self._preview_worker, args=(model,),
                                                   daemon=True, name='Live captions')
            self.preview_thread.start()
        while not self.cancel_transcription.is_set():
            job = self.audio_tasks.get()
            if job is None:
                break
            label, start, arr, core_start, piece_id, paragraph_key = job
            self.last_final_piece[label] = piece_id
            try:
                if model is not None:
                    import numpy as np
                    arr = np.asarray(arr, dtype=np.float32)
                    if len(arr) and float(np.sqrt(np.mean(arr * arr))) > .0008:
                        iterable, _ = model.transcribe(arr, **transcribe_options(self.language))
                        additions = []
                        for seg in iterable:
                            if not accept_speech_segment(seg):
                                continue
                            words, suspicious = sanitize_hypothesis((seg.text or '').strip())
                            if words and (start + seg.end) > core_start + .04:
                                additions.append({'start': round(max(core_start, start + seg.start), 2),
                                                  'end': round(start + seg.end, 2),
                                                  'source': label, 'text': words,
                                                  'paragraph': paragraph_key,
                                                  'perlu_verifikasi': suspicious})
                        if additions:
                            with self.segments_lock:
                                if not self.finalized and not self.cancel_transcription.is_set():
                                    self.segments.extend(additions)
                                    export_transcript(self.session, self.segments)
                                    self._emit('transcript', sort_segments(self.segments))
            except Exception as exc:
                self.failures.append(f'Transkripsi bagian {piece_id}: {exc}')
                self._emit('error', f'Transkripsi bagian {piece_id} gagal: {exc}. Rekaman asli tetap disimpan.')
            self._emit('preview_clear', (label, piece_id))
            self.processed_blocks += 1
            self._emit('queue', (self.recorded_blocks, self.processed_blocks))
        self._complete(partial=self.cancel_transcription.is_set())



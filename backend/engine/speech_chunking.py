"""Energy-based pause chunking, never changes the continuously saved audio.

This is intentionally conservative: a short pause triggers a candidate sentence
boundary. Whisper still decides the words; very quiet voices may go undetected.
"""
from __future__ import annotations
from collections import deque
import numpy as np


class PauseChunker:
    """Low-cost float32 stream chunker; audio is in 16 kHz mono.

    feed(block) returns zero or one (start_sample, chunk_audio) entries.
    finish() flushes an outstanding utterance on Stop.
    """
    def __init__(self, rate=16000, threshold=0.0024, silence_seconds=.80,
                 max_seconds=11.0, preroll_seconds=.24, min_speech_seconds=.16):
        self.rate = rate
        self.threshold = threshold
        self.silence_samples_required = int(silence_seconds * rate)
        self.max_samples = int(max_seconds * rate)
        self.preroll_samples = int(preroll_seconds * rate)
        self.min_speech_samples = int(min_speech_seconds * rate)
        self.position = 0
        self.active = False
        self.parts = []
        self.start_sample = 0
        self.silent_samples = 0
        self.voiced_samples = 0
        self.pre = deque()
        self.pre_len = 0
        self.last_flush_reason = None

    def _add_pre(self, start, block):
        self.pre.append((start, block))
        self.pre_len += len(block)
        # Keep a little leading context for the next spoken phrase.
        while self.pre and self.pre_len > self.preroll_samples + len(self.pre[0][1]):
            _, old = self.pre.popleft()
            self.pre_len -= len(old)

    def _flush(self, reason='stop'):
        self.last_flush_reason = reason
        if not self.parts:
            return []
        combined = np.concatenate(self.parts)
        piece = (self.start_sample, combined) if self.voiced_samples >= self.min_speech_samples else None
        # A short tail is reused as pre-roll, but never alters the original WAV.
        self.pre.clear()
        self.pre_len = 0
        tail_len = min(len(combined), self.preroll_samples)
        if tail_len:
            self._add_pre(self.position - tail_len, combined[-tail_len:].copy())
        self.active = False
        self.parts = []
        self.silent_samples = 0
        self.voiced_samples = 0
        return [piece] if piece else []

    def feed(self, samples, rms=None):
        samples = np.asarray(samples, dtype=np.float32)
        if not len(samples):
            return []
        start = self.position
        self.position += len(samples)
        if rms is None:
            rms = float(np.sqrt(np.mean(samples * samples)))
        else:
            rms = float(rms)
        voiced = rms >= self.threshold
        if not self.active:
            self._add_pre(start, samples.copy())
            if not voiced:
                return []
            self.active = True
            self.start_sample = self.pre[0][0]
            self.parts = [block for _, block in self.pre]
            self.pre.clear()
            self.pre_len = 0
            self.voiced_samples = len(samples)
            self.silent_samples = 0
        else:
            self.parts.append(samples.copy())
            if voiced:
                self.voiced_samples += len(samples)
                self.silent_samples = 0
            else:
                self.silent_samples += len(samples)
        length = self.position - self.start_sample
        if self.silent_samples >= self.silence_samples_required:
            return self._flush(reason='pause')
        if length >= self.max_samples:
            return self._flush(reason='max')
        return []

    def preview(self, limit_seconds=5.2):
        """Return (start sample, recent utterance samples) for rolling preview."""
        if not self.active or not self.parts:
            return None
        n = int(limit_seconds * self.rate)
        # Only concatenate the blocks needed to meet the preview cap.
        chosen, count = [], 0
        for item in reversed(self.parts):
            chosen.append(item)
            count += len(item)
            if count >= n:
                break
        clip = np.concatenate(chosen[::-1])[-n:]
        return self.position - len(clip), clip

    def finish(self):
        return self._flush() if self.active else []

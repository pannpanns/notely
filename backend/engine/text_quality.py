"""Conservative prevention of repetitive Whisper hallucinations.

Whisper can invent loops at the ends of unclear/short audio. We *only* remove
mechanical repetitions (at least four repeated words, or three identical
phrases), preserving the original audio and flagging affected segments.
This is NOT a spelling correction or proof of the speaker's actual words.
"""
from __future__ import annotations
import re

_TOKEN = re.compile(r'[^\W_]+(?:[\'-][^\W_]+)*|[^\s]', flags=re.UNICODE)


def _norm(token: str) -> str:
    return re.sub(r'[^\w]+', '', token.casefold(), flags=re.UNICODE)


def sanitize_hypothesis(text: str) -> tuple[str, bool]:
    """Return (safe text, removed hallucination-like repetition).

    A true speaker repetition is possible; keep up to two repetitions. Ignore
    purely punctuation tokens when detecting spans.
    """
    text = re.sub(r'\s+', ' ', text or '').strip()
    if not text:
        return '', False
    words = text.split()
    norm = [_norm(w) for w in words]
    if len(words) < 4:
        return text, False
    out = []
    i = 0
    changed = False
    while i < len(words):
        found = False
        # Long phrases first, then single-word loops. Never suppress ordinary
        # 'iya iya' or 'sangat sangat' spoken naturally.
        for span in [1, *range(min(5, (len(words) - i) // 3), 1, -1)]:
            token_group = norm[i:i+span]
            if any(not w for w in token_group):
                continue
            runs = 1
            while norm[i + runs*span:i + (runs+1)*span] == token_group:
                runs += 1
            minimum = 4 if span == 1 else 3
            if runs >= minimum:
                out.extend(words[i:i+span*2])
                i += runs*span
                changed = True
                found = True
                break
        if not found:
            out.append(words[i]); i += 1
    clean = ' '.join(out)
    clean = re.sub(r'\s+([,.!?;:])', r'\1', clean)
    return clean, changed


def preview_visible(text: str) -> str:
    """Avoid displaying hallucinated filler; previews remain uncommitted."""
    return sanitize_hypothesis(text)[0]


def accept_speech_segment(segment) -> bool:
    """Reject weak no-speech hypotheses, never blacklist legitimate phrases.

    VAD is the primary gate. This catches low-confidence decoder output that
    slips through it; missing scores retain compatibility with other decoders.
    """
    no_speech = getattr(segment, 'no_speech_prob', 0.0) or 0.0
    log_prob = getattr(segment, 'avg_logprob', 0.0) or 0.0
    return not (no_speech >= .6 and log_prob <= -1.0)

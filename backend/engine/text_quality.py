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


# Known Whisper hallucination phrases trained from YouTube/subtitle outros.
_ALWAYS_HALLUCINATION = (
    'terima kasih telah menonton',
    'terima kasih sudah menonton',
    'terima kasih sudah menyaksikan',
    'terima kasih telah menyaksikan',
    'jangan lupa subscribe',
    'jangan lupa like',
    'like comment dan subscribe',
    'like komen dan subscribe',
    'selamat menonton',
    'thanks for watching',
    'thank you for watching',
    'please subscribe',
    'subtitles by',
    'subtitle by',
    'amara org',
    'silakan subscribe',
)

_WEAK_STANDALONE_HALLUCINATIONS = {
    'terima kasih',
    'terimakasih',
    'terima kasih banyak',
    'terimakasih banyak',
    'terima kasih atas perhatiannya',
    'sampai jumpa',
    'sampai jumpa lagi',
    'sampai bertemu lagi',
    'thank you',
    'thank you very much',
    'thanks',
    'bye',
    'bye bye',
    'selamat tinggal',
    'you',
}


def is_whisper_hallucination(text: str, segment=None, *, audio_rms: float | None = None, preview: bool = False) -> bool:
    """Detect phantom Whisper outro phrases (e.g. 'Terima kasih.') on pauses/noise.

    Always drops unmistakable YouTube subtitle artifacts. Drops standalone
    'Terima kasih' / 'Sampai jumpa' when confidence is weak, audio energy is
    low, or the live preview clip is very short/uncertain, while preserving
    genuine clearly spoken gratitude.
    """
    cleaned = re.sub(r'[^\w\s]+', ' ', (text or '').casefold(), flags=re.UNICODE)
    cleaned = re.sub(r'\s+', ' ', cleaned).strip()
    if not cleaned:
        return True
    for phrase in _ALWAYS_HALLUCINATION:
        if phrase in cleaned:
            return True
    if cleaned in _WEAK_STANDALONE_HALLUCINATIONS:
        no_speech = getattr(segment, 'no_speech_prob', 0.0) or 0.0
        log_prob = getattr(segment, 'avg_logprob', 0.0) or 0.0
        seg_start = getattr(segment, 'start', 0.0) or 0.0
        seg_end = getattr(segment, 'end', 0.0) or 0.0
        duration = max(0.0, seg_end - seg_start)
        if no_speech >= 0.22 or log_prob <= -0.50:
            return True
        if audio_rms is not None and audio_rms < 0.009:
            return True
        if preview and (duration < 1.4 or no_speech >= 0.12):
            return True
    return False


def accept_speech_segment(segment, *, audio_rms: float | None = None, preview: bool = False) -> bool:
    """Reject weak no-speech hypotheses and known Whisper outro hallucinations.

    VAD is the primary gate. This catches low-confidence decoder output and
    phantom 'Terima kasih.' hallucinations on pauses/breaths.
    """
    no_speech = getattr(segment, 'no_speech_prob', 0.0) or 0.0
    log_prob = getattr(segment, 'avg_logprob', 0.0) or 0.0
    if no_speech >= 0.6 and log_prob <= -1.0:
        return False
    if no_speech >= 0.78:
        return False
    text = getattr(segment, 'text', '') or ''
    if text and is_whisper_hallucination(text, segment, audio_rms=audio_rms, preview=preview):
        return False
    return True

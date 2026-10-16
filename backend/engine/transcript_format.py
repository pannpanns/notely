"""Readable display/export of transcription while retaining raw timed segments."""
from __future__ import annotations
from text_quality import sanitize_hypothesis
import re


def clean_text(value):
    value = re.sub(r'\s+', ' ', str(value or '')).strip()
    return re.sub(r'\s+([,.;!?])', r'\1', value)


def join_words(left, right):
    """Deduplicate a small repeated word overlap at audio boundaries."""
    left, right = clean_text(left), clean_text(right)
    if not left: return right
    if not right: return left
    lhs, rhs = left.split(), right.split()
    overlap = 0
    for n in range(min(len(lhs), len(rhs), 8), 0, -1):
        def norm(words):
            return [w.casefold().strip('.,!?;:') for w in words]
        if norm(lhs[-n:]) == norm(rhs[:n]):
            overlap = n
            break
    addition = ' '.join(rhs[overlap:])
    if not addition: return left
    if addition[:1] in ',.;!?': return left + addition
    return left + ' ' + addition


def utterances(segments, pause_seconds=.82, max_chars=420):
    """Group consecutive segments by pause, not by Whisper's arbitrary lines."""
    ordered = sorted(segments, key=lambda s: (float(s.get('start', 0)), s.get('source','')))
    result = []
    for s in ordered:
        text = clean_text(s.get('text', ''))
        if not text: continue
        start = float(s.get('start', 0))
        end = max(start, float(s.get('end', start)))
        source = s.get('source', '')
        paragraph = s.get('paragraph')
        if result:
            prev = result[-1]
            gap = start - prev['end']
            paragraph_matches = (paragraph is None or prev.get('paragraph') is None
                                 or paragraph == prev['paragraph'])
            can_merge = (source == prev['source'] and paragraph_matches
                         and gap <= pause_seconds
                         and len(prev['text']) + len(text) < max_chars)
            if can_merge:
                prev['text'] = join_words(prev['text'], text)
                prev['end'] = max(prev['end'], end)
                continue
        result.append(dict(start=start, end=end, text=text, source=source, paragraph=paragraph))
    return result


def timeline_lines(segments, nice_seconds):
    return [f'[{nice_seconds(u["start"])}] {u["text"]}' for u in utterances(segments)]


def timeline_text(segments, nice_seconds, empty=''):
    lines = timeline_lines(segments, nice_seconds)
    return '\n\n'.join(lines) if lines else empty

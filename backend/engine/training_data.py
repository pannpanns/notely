"""Export user-reviewed audio/text pairs for FUTURE training; never train on-device.

Only explicitly confirmed corrections enter the local dataset. The session
metadata identifies device labels (not actual speaker identities).
"""
from __future__ import annotations
import json
import re
import wave
from pathlib import Path
from core import SAMPLE_RATE, read_json, export_transcript


def save_verified_example(session: Path, index: int, new_text: str):
    """Correct a final segment and save corresponding 16k WAV clip + JSONL manifest.

    Requires an existing complete WAV. Editing changes the transcript but does
    not alter the original recording. Caller confirms correspondence by ear.
    """
    new_text = re.sub(r'\s+', ' ', new_text or '').strip()
    if not new_text or len(new_text) > 2000:
        raise ValueError('Teks koreksi harus berisi 1–2000 karakter.')
    items = read_json(session/'segments.json', [])
    if not isinstance(index, int) or index < 0 or index >= len(items):
        raise IndexError('Bagian transkrip tidak ditemukan.')
    seg = items[index]
    source = seg.get('source','')
    if source == 'Peserta (komputer)':
        source_file = session/'audio_komputer.wav'
    elif source == 'Anda (mikrofon)':
        source_file = session/'audio_mikrofon.wav'
    else:
        raise ValueError('Contoh dari impor audio tidak diekspor sebagai dataset otomatis. Butuh pemotongan dan verifikasi format audio.')
    if not source_file.is_file():
        compressed = source_file.with_suffix('.flac')
        if compressed.is_file():
            source_file = compressed
        else:
            raise FileNotFoundError(f'Rekaman asli tidak ditemukan: {source_file.name}')
    start, end = float(seg['start']), float(seg['end'])
    if end-start < .30 or end-start > 30:
        raise ValueError('Contoh terlalu singkat atau lebih dari 30 detik. Pilih segmen lain.')
    if source_file.suffix.lower() == '.flac':
        import soundfile as sf
        import numpy as np
        with sf.SoundFile(str(source_file), 'r') as reader:
            rate = reader.samplerate
            if rate != SAMPLE_RATE or reader.channels != 1:
                raise ValueError('Dataset memerlukan FLAC/WAV mono 16 kHz.')
            first=max(0,min(reader.frames,int(start*rate)))
            last=min(reader.frames,int(end*rate))
            if last-first < rate*.30:
                raise ValueError('Batas timestamp tidak sesuai dengan durasi rekaman.')
            reader.seek(first)
            data=reader.read(last-first,dtype='int16').astype('<i2').tobytes()
    else:
        with wave.open(str(source_file),'rb') as reader:
            rate = reader.getframerate()
            if rate != SAMPLE_RATE or reader.getnchannels()!=1 or reader.getsampwidth()!=2:
                raise ValueError('Dataset memerlukan WAV mono 16 kHz PCM16.')
            first=max(0,min(reader.getnframes(),int(start*rate)))
            last=min(reader.getnframes(),int(end*rate))
            if last-first < rate*.30:
                raise ValueError('Batas timestamp tidak sesuai dengan durasi rekaman.')
            reader.setpos(first)
            data=reader.readframes(last-first)
    folder=session/'dataset_latih_id'
    folder.mkdir(parents=True,exist_ok=True)
    label = 'komputer' if source=='Peserta (komputer)' else 'mikrofon'
    filename=f'{label}_{index:05d}.wav'
    with wave.open(str(folder/filename),'wb') as out:
        out.setnchannels(1); out.setsampwidth(2); out.setframerate(SAMPLE_RATE)
        out.writeframes(data)
    original=seg['text']
    items[index]['text']=new_text
    export_transcript(session,items)
    # Rebuild full manifest from the verified examples stored on disk. No duplicate
    # examples when editing an already-reviewed segment.
    path=folder/'manifest.jsonl'
    existing={}
    if path.exists():
        for line in path.read_text(encoding='utf-8').splitlines():
            if line.strip():
                try:
                    rec=json.loads(line)
                    existing[rec['audio']]=rec
                except (ValueError,KeyError): pass
    existing[filename]={'audio':filename,'text':new_text,'source':source,
                        'start':start,'end':end,'reviewed':True}
    path.write_text(''.join(json.dumps(x,ensure_ascii=False)+'\n' for x in existing.values()),encoding='utf-8')
    return folder, len(existing), original

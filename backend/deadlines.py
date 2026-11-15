"""Deadline notes management and AI-assisted extraction from text and screenshots."""
from __future__ import annotations
import base64
import datetime
import json
import os
import re
import time
import uuid
from pathlib import Path
try:
    from backend.ocr import extract_text_from_image
    from backend.providers import _ask, unload_local_model
except ImportError:
    from ocr import extract_text_from_image
    from providers import _ask, unload_local_model

DEADLINES_FILE = 'deadlines.json'

def _get_file_path(data_root: str | Path) -> Path:
    p = Path(data_root)
    p.mkdir(parents=True, exist_ok=True)
    return p / DEADLINES_FILE

def load_deadlines(data_root: str | Path) -> list[dict]:
    path = _get_file_path(data_root)
    if not path.exists():
        return []
    try:
        data = json.loads(path.read_text(encoding='utf-8'))
        return data if isinstance(data, list) else []
    except Exception:
        return []

def save_deadlines(data_root: str | Path, deadlines: list[dict]) -> bool:
    path = _get_file_path(data_root)
    temp_path = path.with_suffix('.tmp')
    try:
        temp_path.write_text(json.dumps(deadlines, ensure_ascii=False, indent=2), encoding='utf-8')
        temp_path.replace(path)
        return True
    except Exception as e:
        if temp_path.exists():
            temp_path.unlink(missing_ok=True)
        raise e

def add_or_update_deadline(data_root: str | Path, item: dict) -> dict:
    deadlines = load_deadlines(data_root)
    item_id = item.get('id') or f"dl_{int(time.time()*1000)}_{uuid.uuid4().hex[:6]}"
    item['id'] = item_id
    if 'created_at' not in item:
        item['created_at'] = datetime.datetime.now().isoformat()
    if 'status' not in item:
        item['status'] = 'pending'
    if not item.get('priority'):
        item['priority'] = 'sedang'
    if not item.get('time'):
        item['time'] = '23:59'

    existing_idx = next((i for i, d in enumerate(deadlines) if d.get('id') == item_id), None)
    if existing_idx is not None:
        deadlines[existing_idx].update(item)
    else:
        deadlines.insert(0, item)
    save_deadlines(data_root, deadlines)
    return item

def delete_deadline(data_root: str | Path, item_id: str) -> bool:
    deadlines = load_deadlines(data_root)
    new_list = [d for d in deadlines if d.get('id') != item_id]
    save_deadlines(data_root, new_list)
    return True

def toggle_deadline_status(data_root: str | Path, item_id: str) -> dict | None:
    deadlines = load_deadlines(data_root)
    for d in deadlines:
        if d.get('id') == item_id:
            d['status'] = 'completed' if d.get('status') != 'completed' else 'pending'
            save_deadlines(data_root, deadlines)
            return d
    return None

def build_extraction_prompt(source_text: str, user_prompt: str = "") -> str:
    now = datetime.datetime.now()
    hari_indo = ['Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu', 'Minggu'][now.weekday()]
    bulan_indo = ['Januari', 'Februari', 'Maret', 'April', 'Mei', 'Juni', 'Juli', 'Agustus', 'September', 'Oktober', 'November', 'Desember'][now.month - 1]
    ref_date_str = f"{hari_indo}, {now.day} {bulan_indo} {now.year} ({now.strftime('%Y-%m-%d')})"

    return f"""Kamu adalah asisten pengatur jadwal akademik dan manajemen tugas (Deadline Manager).
Tugasmu adalah menganalisis teks sumber dan instruksi pengguna berikut, lalu mengekstrak semua daftar tugas, proyek, kuis, atau deadline ke dalam format JSON yang terstruktur.

[REFERENSI WAKTU SAAT INI]
Hari ini adalah: {ref_date_str}
Tahun saat ini: {now.year}

[PETUNJUK PENENTUAN TANGGAL]
- Jika teks menyebutkan tanggal tanpa tahun (misal "28 November" atau "tgl 5 depan"), asumsikan tahun {now.year}.
- Jika menyebutkan hari relatif seperti "besok", "lusa", "jumat depan", "senin ini", hitung tanggal pastinya berdasarkan referensi hari ini ({ref_date_str}).
- Format tanggal HARUS selalu 'YYYY-MM-DD' (contoh: '2026-11-28').
- Jika jam pengumpulan tidak disebutkan secara eksplisit, gunakan default jam '23:59'. Format jam HARUS 'HH:MM' (24 jam).
- Tentukan prioritas: 'tinggi' (urgent/bobot besar/ujian), 'sedang' (tugas reguler), atau 'rendah' (tugas bacaan/opsional).

[DATA SUMBER DARI PENGGUNA / SCREENSHOT]
{source_text}

[INSTRUKSI TAMBAHAN PENGGUNA]
{user_prompt or 'Ekstrak semua deadline tugas yang tercantum pada teks sumber di atas.'}

[FORMAT KELUARAN WAJIB]
Keluarkan HANYA array JSON murni tanpa pengantar, tanpa penutup, tanpa markdown ```json.
Contoh format:
[
  {{
    "title": "Nama Tugas atau Judul Kegiatan",
    "course": "Mata Kuliah / Mata Pelajaran / Kategori",
    "deadline": "YYYY-MM-DD",
    "time": "HH:MM",
    "description": "Rincian tugas, format file yang diminta, atau catatan penting",
    "priority": "tinggi" | "sedang" | "rendah"
  }}
]
Jika tidak ditemukan tugas atau deadline apapun dalam teks, keluarkan array kosong: []"""

def _clean_json_output(raw_text: str) -> list[dict]:
    text = raw_text.strip()
    # Strip markdown code blocks if wrapped
    if text.startswith('```'):
        text = re.sub(r'^```(?:json)?\s*', '', text)
        text = re.sub(r'\s*```$', '', text)
    text = text.strip()

    # Search for json array bracket
    match = re.search(r'\[.*\]', text, re.DOTALL)
    if match:
        text = match.group(0)

    try:
        parsed = json.loads(text)
        if isinstance(parsed, list):
            valid_items = []
            for item in parsed:
                if not isinstance(item, dict):
                    continue
                title = str(item.get('title') or '').strip()
                if not title:
                    continue
                deadline = str(item.get('deadline') or '').strip()
                # Verify date format YYYY-MM-DD
                if not re.match(r'^\d{4}-\d{2}-\d{2}$', deadline):
                    # Fallback to tomorrow if invalid
                    tomorrow = datetime.date.today() + datetime.timedelta(days=1)
                    deadline = tomorrow.strftime('%Y-%m-%d')

                time_str = str(item.get('time') or '23:59').strip()
                if not re.match(r'^\d{2}:\d{2}$', time_str):
                    time_str = '23:59'

                priority = str(item.get('priority') or 'sedang').lower()
                if priority not in ('tinggi', 'sedang', 'rendah'):
                    priority = 'sedang'

                valid_items.append({
                    'id': f"dl_{int(time.time()*1000)}_{uuid.uuid4().hex[:6]}",
                    'title': title,
                    'course': str(item.get('course') or 'Umum').strip(),
                    'deadline': deadline,
                    'time': time_str,
                    'description': str(item.get('description') or '').strip(),
                    'priority': priority,
                    'status': 'pending',
                    'created_at': datetime.datetime.now().isoformat()
                })
            return valid_items
    except Exception as e:
        print("JSON parse error:", e, "Raw output:", text[:200])
    return []

def extract_deadlines(
    prompt: str = "",
    image_base64: str = "",
    provider: str = "ollama",
    model: str = "",
    api_key: str = "",
    endpoint: str = "",
    progress=None
) -> dict:
    ocr_text = ""
    if image_base64:
        # Strip data URL prefix if present
        if ',' in image_base64:
            image_base64 = image_base64.split(',', 1)[1]
        try:
            img_bytes = base64.b64decode(image_base64)
            if progress:
                progress("Membaca teks dari gambar (OCR Windows)...")
            ocr_text = extract_text_from_image(img_bytes)
        except Exception as e:
            print("Failed to decode or OCR image:", e)

    source_text = ""
    if ocr_text:
        source_text += f"[Teks dari Tangkapan Layar]:\n{ocr_text}\n"
    if prompt:
        source_text += f"[Catatan / Teks Pengguna]:\n{prompt}\n"

    if not source_text.strip():
        raise ValueError("Teks maupun gambar tidak memuat konten yang bisa dianalisis.")

    full_prompt = build_extraction_prompt(source_text, prompt)

    if progress:
        progress("AI sedang menganalisis tenggat dan menyusun jadwal...")

    raw_response = ""
    try:
        raw_response = _ask(provider, model, api_key, full_prompt, endpoint, progress)
    finally:
        if provider == 'ollama' and model:
            unload_local_model(model)

    items = _clean_json_output(raw_response)
    return {
        'deadlines': items,
        'ocrText': ocr_text,
        'count': len(items)
    }

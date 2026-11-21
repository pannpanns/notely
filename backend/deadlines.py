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
    if 'reminder' not in item:
        item['reminder'] = 30

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

    return f"""Kamu adalah asisten pengatur agenda akademik, jadwal kuliah, dan manajemen tugas (Notely Agenda & Deadline Manager).
Tugasmu adalah menganalisis teks sumber (bisa berasal dari teks langsung atau tangkapan layar pengumuman/chat grup kuliah/screenshot web) dan instruksi tambahan pengguna, lalu mengekstrak semua agenda akademik: tugas kuliah, proyek, kuis, ujian, jadwal perkuliahan daring (Google Meet, Zoom, Teams), praktikum, maupun pertemuan ke dalam format JSON yang terstruktur.

[PENTING - ATURAN EKSTRAKSI AGENDA & JADWAL KULIAH/MEET]:
1. Jika teks memuat jadwal perkuliahan, kelas daring, seminar, atau pertemuan (seperti link Google Meet 'meet.google.com' atau Zoom dan jam pelaksanaan seperti 'jam 16.30-19.29' atau '16:30'):
   - Wajib diekstrak sebagai agenda jadwal perkuliahan/meeting!
   - Untuk rentang jam seperti '16.30-19.29', gunakan jam mulai yaitu '16:30'.
   - Judul tugas/agenda harus jelas, ringkas, dan rapi (misal: "Perkuliahan Google Meet", "Kuliah Daring [Nama Matkul]", atau "Praktikum"). Jangan membuat judul dari potongan kalimat chat seperti "Teman Mohon Izin...".
   - Sertakan link Google Meet/Zoom di dalam 'description' jika ada.
2. Jika pengguna mengajukan pertanyaan atau instruksi (contoh: "Jam berapa meet", "tolong catat tugas ini", "kapan deadline"), gunakan pertanyaan tersebut sebagai petunjuk untuk mencari agenda yang relevan dari teks sumber. JANGAN menjawabnya sebagai obrolan santai, tetapi TETAP keluarkan format JSON.
3. Penentuan Tanggal:
   - Jika teks menyebutkan "hari ini", gunakan tanggal hari ini: {now.strftime('%Y-%m-%d')}.
   - Jika menyebutkan "besok", hitung +1 hari ({(now + datetime.timedelta(days=1)).strftime('%Y-%m-%d')}).
   - Jika menyebutkan "lusa", hitung +2 hari.
   - Jika menyebutkan nama hari (misal "senin depan"), hitung tanggal pastinya berdasarkan referensi hari ini: {ref_date_str}.
   - Format tanggal HARUS selalu 'YYYY-MM-DD'.
4. Penentuan Jam:
   - Gunakan format 'HH:MM' (24 jam). Jika format di teks menggunakan titik seperti '16.30', ubah menjadi '16:30'.
   - Jika untuk pengumpulan tugas dan jam tidak disebutkan eksplisit, gunakan default '23:59'.
5. Prioritas:
   - 'tinggi': Ujian, UTS, UAS, kuis, pertemuan mendesak, atau tugas penting.
   - 'sedang': Perkuliahan/kuliah Google Meet reguler, tugas mingguan, laporan.
   - 'rendah': Bacaan opsional atau materi santai.

[REFERENSI WAKTU SAAT INI]
Hari ini adalah: {ref_date_str}
Tahun saat ini: {now.year}

[DATA SUMBER DARI PENGGUNA / SCREENSHOT]
{source_text}

[INSTRUKSI TAMBAHAN PENGGUNA]
{user_prompt or 'Ekstrak semua jadwal perkuliahan, pertemuan, dan deadline tugas yang tercantum pada teks sumber di atas.'}

[FORMAT KELUARAN WAJIB]
Keluarkan HANYA array JSON murni tanpa pengantar dan tanpa penutup markdown:
[
  {{
    "title": "Perkuliahan Google Meet",
    "course": "Kuliah",
    "deadline": "{now.strftime('%Y-%m-%d')}",
    "time": "16:30",
    "description": "Kuliah daring via Google Meet: https://meet.google.com/...",
    "priority": "sedang"
  }}
]
Jika benar-benar tidak ada agenda tugas atau jadwal apapun yang bisa dijadwalkan, keluarkan: []"""

INDONESIAN_MONTHS = {
    'januari': 1, 'jan': 1, 'februari': 2, 'feb': 2, 'maret': 3, 'mar': 3,
    'april': 4, 'apr': 4, 'mei': 5, 'juni': 6, 'jun': 6, 'juli': 7, 'jul': 7,
    'agustus': 8, 'agu': 8, 'agst': 8, 'september': 9, 'sep': 9, 'sept': 9,
    'oktober': 10, 'okt': 10, 'november': 11, 'nov': 11, 'desember': 12, 'des': 12
}

INDONESIAN_DAYS = {
    'senin': 0, 'selasa': 1, 'rabu': 2, 'kamis': 3,
    'jumat': 4, "jum'at": 4, 'sabtu': 5, 'minggu': 6, 'ahad': 6
}

def extract_deadlines_heuristically(text: str, ref_date: datetime.date = None) -> list[dict]:
    """Deterministic fallback parser for Indonesian task, lecture & deadline expressions."""
    if not text or not text.strip():
        return []
    if ref_date is None:
        ref_date = datetime.date.today()

    normalized = text.replace('\r\n', '\n')
    
    # Check if text contains a Google Meet / Zoom link or lecture meeting announcement
    meet_match = re.search(r'(?:https?://)?(?:meet\.google\.com/[a-z0-9-]+|[a-zA-Z0-9.-]*zoom\.us/[^\s\)\],]+)', normalized, re.IGNORECASE)
    has_meet_context = bool(meet_match or re.search(r'\b(google meet|gmeet|meet|zoom|kuliah|perkuliahan|tatap muka|daring|kelas|webinar|rapat|pertemuan)\b|sesuai\s+(?:di\s+)?gambar', normalized, re.IGNORECASE))

    results = []

    # If it's a meeting / lecture announcement with a link or specific time digits, extract it directly
    has_time_or_link = bool(meet_match or re.search(r'\b\d{1,2}[.:]\d{2}\b', normalized))
    if has_meet_context and has_time_or_link:
        # Date detection
        target_date = ref_date
        if re.search(r'\bbesok\b', normalized, re.IGNORECASE):
            target_date = ref_date + datetime.timedelta(days=1)
        elif re.search(r'\blusa\b', normalized, re.IGNORECASE):
            target_date = ref_date + datetime.timedelta(days=2)
        elif re.search(r'\bhari\s*ini\b', normalized, re.IGNORECASE):
            target_date = ref_date
        else:
            for day_name, day_idx in INDONESIAN_DAYS.items():
                if re.search(rf'\b{day_name}\b', normalized, re.IGNORECASE):
                    cur_idx = ref_date.weekday()
                    days_ahead = (day_idx - cur_idx) % 7
                    if days_ahead == 0 and not re.search(r'\bhari\s*ini\b', normalized, re.IGNORECASE):
                        days_ahead = 7
                    target_date = ref_date + datetime.timedelta(days=days_ahead)
                    break

        # Time detection
        target_time = '16:30'  # reasonable fallback for afternoon class if not found
        m_range = re.search(r'(\d{1,2})[.:](\d{2})\s*[-–]\s*(\d{1,2})[.:](\d{2})', normalized)
        if m_range:
            target_time = f'{int(m_range.group(1)):02d}:{int(m_range.group(2)):02d}'
        else:
            m_single = re.search(r'(?:jam|pukul)?\s*(\d{1,2})[.:](\d{2})', normalized, re.IGNORECASE)
            if m_single:
                target_time = f'{int(m_single.group(1)):02d}:{int(m_single.group(2)):02d}'

        # Course detection
        course = 'Kuliah'
        m_sender = re.search(r'\b([A-Z][a-zA-Z]+)\s+([A-Z]{2,5})\b', normalized)
        if m_sender and m_sender.group(2).upper() in ('PTI', 'TI', 'SI', 'IF', 'DKV', 'TE', 'TS', 'TM'):
            course = m_sender.group(2).upper()
        else:
            for kw in ['fisika', 'matematika', 'kalkulus', 'basis data', 'pbo', 'ai', 'kecerdasan buatan', 'pemrograman', 'jaringan', 'algoritma', 'alpro', 'kimia', 'biologi', 'akuntansi', 'manajemen', 'pti', 'ti', 'si', 'if', 'dkv']:
                if re.search(rf'\b{kw}\b', normalized, re.IGNORECASE):
                    course = kw.upper() if len(kw) <= 4 or kw in ('pbo', 'ai', 'pti', 'dkv') else kw.title()
                    break

        if course != 'Kuliah':
            title = f"Perkuliahan Google Meet ({course})" if ('meet' in normalized.lower() or meet_match) else f"Kuliah {course}"
        else:
            title = 'Perkuliahan Google Meet' if ('meet' in normalized.lower() or meet_match) else 'Kuliah Daring'

        desc_link = f"Link: {meet_match.group(0)}" if meet_match else ""
        desc_time = f"Jadwal: {target_time} WIB" if target_time else ""
        desc = f"Jadwal perkuliahan daring. {desc_time} {desc_link}".strip()

        results.append({
            'id': f'dl_{int(time.time()*1000)}_{uuid.uuid4().hex[:6]}',
            'title': title,
            'course': course,
            'deadline': target_date.strftime('%Y-%m-%d'),
            'time': target_time,
            'description': desc,
            'priority': 'sedang',
            'status': 'pending',
            'created_at': datetime.datetime.now().isoformat()
        })
        return results

    lines = re.split(r'[\n;]+|(?<=[^0-9])\s*[-*•]\s*', normalized)
    chunks = []
    for line in lines:
        line = line.strip()
        if not line:
            continue
        parts = re.split(r',\s*(?=(?:tugas|kuis|revisi|laporan|ujian|pr|presentasi|proyek|makalah|resume|uas|uts|kuliah|perkuliahan)\b)|(?<=[a-zA-Z0-9])\s+dan\s+(?=(?:tugas|kuis|revisi|laporan|ujian|pr|presentasi|proyek|makalah|resume|uas|uts|kuliah|perkuliahan)\b)', line, flags=re.IGNORECASE)
        for p in parts:
            p = p.strip()
            if p:
                chunks.append(p)

    for chunk in chunks:
        c_lower = chunk.lower()

        # Ignore conversational questions from being added as dummy tasks
        if re.search(r'^(?:jam berapa|kapan|apakah|adakah|tolong|jadwalkan|cek)\b', c_lower) or c_lower.endswith('?'):
            continue

        # Check date cues
        has_time_cue = bool(re.search(r'\b(deadline|tenggat|dikumpul|kumpul|jam|pukul|hari|besok|lusa|minggu|senin|selasa|rabu|kamis|jumat|sabtu|tanggal|tgl|\d{1,2}/\d{1,2})\b', c_lower))
        if not has_time_cue and len(chunks) > 1:
            if not re.match(r'^(tugas|kuis|revisi|laporan|ujian|pr|presentasi|proyek|makalah|resume|kuliah|perkuliahan)\b', c_lower):
                continue

        # 1. Parse Date
        target_date = None
        if re.search(r'\bhari\s*ini\b', c_lower):
            target_date = ref_date

        if not target_date:
            m_rel_day = re.search(r'(\d+)\s*hari\s*lagi', c_lower)
            if m_rel_day:
                target_date = ref_date + datetime.timedelta(days=int(m_rel_day.group(1)))

        if not target_date:
            m_rel_week = re.search(r'(\d+)\s*minggu\s*lagi', c_lower)
            if m_rel_week:
                target_date = ref_date + datetime.timedelta(weeks=int(m_rel_week.group(1)))

        if not target_date:
            if re.search(r'\bbesok\b', c_lower):
                target_date = ref_date + datetime.timedelta(days=1)
            elif re.search(r'\blusa\b', c_lower):
                target_date = ref_date + datetime.timedelta(days=2)
            elif re.search(r'\bminggu\s*depan\b', c_lower) and not any(d in c_lower for d in INDONESIAN_DAYS):
                target_date = ref_date + datetime.timedelta(days=7)

        if not target_date:
            for day_name, day_idx in INDONESIAN_DAYS.items():
                if re.search(rf'\b{day_name}\b', c_lower):
                    cur_idx = ref_date.weekday()
                    days_ahead = (day_idx - cur_idx) % 7
                    if days_ahead == 0:
                        days_ahead = 7
                    elif 'depan' in c_lower and days_ahead < 3:
                        days_ahead += 7
                    target_date = ref_date + datetime.timedelta(days=days_ahead)
                    break

        if not target_date:
            m_date = re.search(r'(?:tanggal|tgl)?\s*(\d{1,2})\s*([a-zA-Z]+)?(?:\s*(\d{4}))?', c_lower)
            if m_date and (re.search(r'(?:tanggal|tgl)', c_lower) or m_date.group(2) in INDONESIAN_MONTHS):
                d = int(m_date.group(1))
                m_name = m_date.group(2)
                month = INDONESIAN_MONTHS.get(m_name, ref_date.month) if m_name else ref_date.month
                year = int(m_date.group(3)) if m_date.group(3) else ref_date.year
                try:
                    target_date = datetime.date(year, month, d)
                except Exception:
                    pass

        if not target_date:
            m_slash = re.search(r'(\d{1,2})[/-](\d{1,2})(?:[/-](\d{2,4}))?', c_lower)
            if m_slash:
                d = int(m_slash.group(1))
                m = int(m_slash.group(2))
                y = int(m_slash.group(3)) if m_slash.group(3) else ref_date.year
                if y < 100:
                    y += 2000
                try:
                    target_date = datetime.date(y, m, d)
                except Exception:
                    pass

        if not target_date:
            target_date = ref_date + datetime.timedelta(days=1)

        # 2. Parse Time
        target_time = '23:59'
        m_range = re.search(r'(\d{1,2})[.:](\d{2})\s*[-–]\s*(\d{1,2})[.:](\d{2})', c_lower)
        if m_range:
            h1 = int(m_range.group(1))
            mi1 = int(m_range.group(2))
            if 0 <= h1 <= 23 and 0 <= mi1 <= 59:
                target_time = f'{h1:02d}:{mi1:02d}'
        else:
            m_time = re.search(r'(?:jam|pukul)?\s*(\d{1,2})[:.](\d{2})', c_lower)
            if m_time:
                h = int(m_time.group(1))
                mi = int(m_time.group(2))
                if 0 <= h <= 23 and 0 <= mi <= 59:
                    target_time = f'{h:02d}:{mi:02d}'
            else:
                m_hour = re.search(r'(?:jam|pukul)\s*(\d{1,2})\s*(malam|siang|pagi|sore)?', c_lower)
                if m_hour:
                    h = int(m_hour.group(1))
                    period = m_hour.group(2) or ''
                    if 'malam' in period and h < 12:
                        h += 12
                    elif 'siang' in period and h < 12 and h != 12:
                        h += 12
                    elif 'sore' in period and h < 12:
                        h += 12
                    target_time = f'{h:02d}:00'

        # 3. Clean Title & Course
        clean_title = re.sub(r'(?i)\b(deadline|tenggat|dikumpul|kumpul|jam|pukul|hari|lagi|besok|lusa|depan|senin|selasa|rabu|kamis|jumat|sabtu|minggu|tanggal|tgl|\d{1,2}[:.]\d{2}|\d+\s*hari)\b.*$', '', chunk).strip()
        clean_title = re.sub(r'[,:\-–]+$', '', clean_title).strip()
        if not clean_title or len(clean_title) < 3:
            clean_title = chunk[:40].strip()

        title_formatted = ' '.join(w.capitalize() if not w.isupper() else w for w in clean_title.split())

        course = 'Umum'
        for kw in ['fisika', 'matematika', 'kalkulus', 'basis data', 'pbo', 'ai', 'kecerdasan buatan', 'pemrograman', 'jaringan', 'algoritma', 'alpro', 'kimia', 'biologi', 'akuntansi', 'manajemen', 'pti', 'ti', 'si', 'if', 'dkv']:
            if kw in c_lower:
                course = kw.upper() if len(kw) <= 4 or kw in ('pbo', 'ai', 'pr', 'pti', 'dkv') else kw.title()
                break

        priority = 'sedang'
        if any(w in c_lower for w in ['kuis', 'ujian', 'uts', 'uas', 'proyek', 'final', 'penting', 'urgent', 'mendesak']):
            priority = 'tinggi'
        elif any(w in c_lower for w in ['bacaan', 'opsional', 'santai', 'ringan']):
            priority = 'rendah'

        results.append({
            'id': f'dl_{int(time.time()*1000)}_{uuid.uuid4().hex[:6]}',
            'title': title_formatted,
            'course': course,
            'deadline': target_date.strftime('%Y-%m-%d'),
            'time': target_time,
            'description': chunk.strip(),
            'priority': priority,
            'status': 'pending',
            'created_at': datetime.datetime.now().isoformat()
        })

    return results

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

    parsed = None
    try:
        parsed = json.loads(text)
    except Exception:
        # Try extracting JSON array using regex if embedded in prose
        m_list = re.search(r'\[\s*\{.*\}\s*\]', text, re.DOTALL)
        if m_list:
            try:
                parsed = json.loads(m_list.group(0))
            except Exception:
                pass
        if parsed is None:
            # Try extracting single JSON object using regex
            m_obj = re.search(r'\{\s*"title".*?\}', text, re.DOTALL)
            if m_obj:
                try:
                    parsed = json.loads(m_obj.group(0))
                except Exception:
                    pass

    if isinstance(parsed, dict):
        for k in ('deadlines', 'tasks', 'agenda', 'items', 'events'):
            if isinstance(parsed.get(k), list):
                parsed = parsed[k]
                break
        else:
            if parsed.get('title') or parsed.get('deadline'):
                parsed = [parsed]
            else:
                parsed = []

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
    except Exception as e:
        print(f"AI Provider query failed: {e}. Falling back to heuristic OCR extraction.")
        if progress:
            progress("Menyusun jadwal dari teks & hasil baca gambar...")
    finally:
        if provider == 'ollama' and model:
            try:
                unload_local_model(model)
            except Exception:
                pass

    items = _clean_json_output(raw_response) if raw_response else []
    if not items:
        # Fallback to deterministic heuristic Indonesian deadline extractor
        combined_text = f"{ocr_text}\n{prompt}\n{raw_response}".strip()
        items = extract_deadlines_heuristically(combined_text)

    return {
        'deadlines': items,
        'ocrText': ocr_text,
        'count': len(items)
    }

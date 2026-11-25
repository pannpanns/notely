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
6. Multi-Gambar: Jika terdapat lebih dari satu bagian `[Teks dari Tangkapan Layar 1]`, `[Teks dari Tangkapan Layar 2]`, dst., wajib mengekstrak SEMUA jadwal dan tugas dari setiap gambar ke dalam satu array JSON.

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
    "course": "Akademik",
    "deadline": "{now.strftime('%Y-%m-%d')}",
    "time": "16:30",
    "description": "Kuliah daring via Google Meet: https://meet.google.com/...",
    "priority": "sedang",
    "reminder": 60
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
        end_time = ''
        m_range = re.search(r'(\d{1,2})[.:](\d{2})\s*[-–]\s*(\d{1,2})[.:](\d{2})', normalized)
        if m_range:
            target_time = f'{int(m_range.group(1)):02d}:{int(m_range.group(2)):02d}'
            end_time = f'{int(m_range.group(3)):02d}:{int(m_range.group(4)):02d}'
        else:
            m_single = re.search(r'(?:jam|pukul)?\s*(\d{1,2})[.:](\d{2})', normalized, re.IGNORECASE)
            if m_single:
                target_time = f'{int(m_single.group(1)):02d}:{int(m_single.group(2)):02d}'

        # Category / Course detection (flexible and all-round)
        course = 'Akademik'
        m_sender = re.search(r'\b([A-Z][a-zA-Z]+)\s+([A-Z]{2,5})\b', normalized)
        if m_sender and m_sender.group(2).upper() in ('PTI', 'TI', 'SI', 'IF', 'DKV', 'TE', 'TS', 'TM'):
            course = m_sender.group(2).upper()
        else:
            for kw in ['fisika', 'matematika', 'kalkulus', 'basis data', 'pbo', 'ai', 'kecerdasan buatan', 'pemrograman', 'jaringan', 'algoritma', 'alpro', 'kimia', 'biologi', 'akuntansi', 'manajemen', 'pti', 'ti', 'si', 'if', 'dkv']:
                if re.search(rf'\b{kw}\b', normalized, re.IGNORECASE):
                    course = kw.upper() if len(kw) <= 4 or kw in ('pbo', 'ai', 'pti', 'dkv') else kw.title()
                    break

        if course not in ('Akademik', 'Umum'):
            title = f"Perkuliahan Google Meet ({course})" if ('meet' in normalized.lower() or meet_match) else f"Kuliah {course}"
        else:
            title = 'Perkuliahan Google Meet' if ('meet' in normalized.lower() or meet_match) else 'Kuliah Daring'

        desc_time = f"Jadwal: {target_time}" + (f" - {end_time} WIB" if end_time else " WIB")
        desc_link = f"Link: {meet_match.group(0)}" if meet_match else ""
        desc = f"Jadwal perkuliahan daring. {desc_time} {desc_link}".strip()

        item_dict = {
            'id': f'dl_{int(time.time()*1000)}_{uuid.uuid4().hex[:6]}',
            'title': title,
            'course': course,
            'deadline': target_date.strftime('%Y-%m-%d'),
            'time': target_time,
            'description': desc,
            'priority': 'sedang',
            'reminder': 60,
            'status': 'pending',
            'created_at': datetime.datetime.now().isoformat()
        }
        if end_time:
            item_dict['end_time'] = end_time
        results.append(item_dict)
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

        # Ignore pure conversational questions with NO concrete time digits or deadline info (e.g. "Jam berapa meet?")
        is_question = bool(re.search(r'^(?:jam berapa|kapan|apakah|adakah)\b', c_lower) or c_lower.endswith('?'))
        has_concrete_time = bool(re.search(r'(?:\b(?:jam|pukul)\s*\d{1,2}|\b\d{1,2}[:.]\d{2}\b|\b(?:deadline|tenggat|dikumpul|kumpul)\b)', c_lower))
        if is_question and not has_concrete_time:
            continue

        has_task_cue = bool(re.search(r'\b(tugas|uts|uas|kuis|ujian|pr|revisi|laporan|presentasi|proyek|makalah|resume|kuliah|perkuliahan|kelas|webinar|rapat|pertemuan)\b', c_lower))
        has_time_cue = bool(re.search(r'\b(deadline|tenggat|dikumpul|kumpul|jam|pukul|hari|besok|lusa|minggu|senin|selasa|rabu|kamis|jumat|sabtu|tanggal|tgl|\d{1,2}/\d{1,2}|\d{1,2}[:.]\d{2})\b', c_lower))
        if not has_time_cue and not has_task_cue and len(chunks) > 1:
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
                    elif not period and h in (7, 8, 9, 10, 11) and ('deadline' in c_lower or 'tenggat' in c_lower or 'kumpul' in c_lower):
                        h += 12
                    target_time = f'{h:02d}:00'

        # 3. Clean Title & Course
        m_task = re.search(r'\b((?:tugas|kuis|revisi|laporan|ujian|pr|presentasi|proyek|makalah|resume|uas|uts|kuliah|perkuliahan)\b(?:\s+(?!deadline|tenggat|jam|pukul|hari|besok|lusa|minggu|senin|selasa|rabu|kamis|jumat|sabtu|tanggal|tgl|tolong|dikumpul|kumpul|nya)[a-zA-Z0-9]+){0,5})', chunk, re.IGNORECASE)
        if m_task:
            clean_title = m_task.group(1).strip()
        else:
            clean_title = re.sub(r'(?i)\b(deadline|tenggat|dikumpul|kumpul|jam|pukul|hari|lagi|besok|lusa|depan|senin|selasa|rabu|kamis|jumat|sabtu|minggu|tanggal|tgl|\d{1,2}[:.]\d{2}|\d+\s*hari)\b.*$', '', chunk).strip()
            clean_title = re.sub(r'^(?:halo|hai|tolong|mohon|buatkan|jadwalkan|catat|ingatkan|masukkan)?\s*(?:saya\s+ada|ada|terdapat)?\s*', '', clean_title, flags=re.IGNORECASE).strip()
            clean_title = re.sub(r'[,:\-–]+$', '', clean_title).strip()
            if not clean_title or len(clean_title) < 3:
                clean_title = chunk[:40].strip()

        title_formatted = ' '.join(w.upper() if w.upper() in ('UTS', 'UAS', 'AI', 'PR', 'TI', 'SI', 'DKV', 'PTI', 'PBO') else w.capitalize() for w in clean_title.split())

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
            'reminder': 60 if any(w in c_lower for w in ['meet', 'kuliah', 'zoom', 'tatap muka']) else 30,
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

            course_val = str(item.get('course') or 'Umum').strip()
            if course_val.lower() == 'kuliah':
                course_val = 'Akademik'

            valid_items.append({
                'id': f"dl_{int(time.time()*1000)}_{uuid.uuid4().hex[:6]}",
                'title': title,
                'course': course_val,
                'deadline': deadline,
                'time': time_str,
                'description': str(item.get('description') or '').strip(),
                'priority': priority,
                'reminder': int(item.get('reminder', 60 if any(w in title.lower() for w in ['meet', 'kuliah', 'zoom', 'tatap muka']) else 30)),
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
    progress=None,
    images: list | None = None
) -> dict:
    if provider == 'ollama':
        try:
            try:
                import backend.managed_ollama as managed_ollama
                from backend.providers import models_for
            except ImportError:
                import managed_ollama
                from providers import models_for
            managed_ollama.ensure(progress if progress else lambda m: None)
            avail = models_for('ollama')
            if avail and (not model or model not in avail):
                model = avail[0]
            elif not model:
                model = 'gemma4:latest'
        except Exception as e:
            print("managed_ollama auto-ensure failed in extract_deadlines:", e)

    img_list = []
    if isinstance(images, list):
        for im in images:
            if isinstance(im, str) and im.strip():
                img_list.append(im.strip())
    if isinstance(image_base64, str) and image_base64.strip():
        if image_base64.strip() not in img_list:
            img_list.insert(0, image_base64.strip())

    ocr_parts = []
    total_imgs = len(img_list)
    for idx, raw_img in enumerate(img_list, 1):
        b64 = raw_img.split(',', 1)[1] if ',' in raw_img else raw_img
        try:
            img_bytes = base64.b64decode(b64)
            if progress:
                if total_imgs > 1:
                    progress(f"Membaca teks dari gambar {idx}/{total_imgs} (OCR Windows)...")
                else:
                    progress("Membaca teks dari gambar (OCR Windows)...")
            part_text = extract_text_from_image(img_bytes)
            if part_text and part_text.strip():
                ocr_parts.append(part_text.strip())
        except Exception as e:
            print(f"Failed to decode or OCR image #{idx}:", e)

    ocr_text = "\n\n".join(ocr_parts)
    source_text = ""
    if len(ocr_parts) == 1:
        source_text += f"[Teks dari Tangkapan Layar]:\n{ocr_parts[0]}\n"
    elif len(ocr_parts) > 1:
        for idx, pt in enumerate(ocr_parts, 1):
            source_text += f"[Teks dari Tangkapan Layar {idx}]:\n{pt}\n\n"
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
        # Fallback to deterministic heuristic Indonesian deadline extractor per screenshot & prompt
        seen_keys = set()
        collected = []

        def _add_unique(found_items):
            for it in found_items:
                key = (it.get('title', '').strip().lower(), it.get('deadline', ''), it.get('time', ''))
                if key not in seen_keys:
                    seen_keys.add(key)
                    collected.append(it)

        if ocr_parts:
            for pt in ocr_parts:
                found = extract_deadlines_heuristically(pt)
                if not found and prompt:
                    found = extract_deadlines_heuristically(f"{pt}\n{prompt}")
                _add_unique(found)
            if prompt and not re.search(r'sesuai\s+(?:di\s+)?gambar|dari\s+gambar|di\s+screenshot', prompt, re.IGNORECASE):
                _add_unique(extract_deadlines_heuristically(prompt))
        else:
            _add_unique(extract_deadlines_heuristically(prompt))

        items = collected

    return {
        'deadlines': items,
        'ocrText': ocr_text,
        'count': len(items)
    }

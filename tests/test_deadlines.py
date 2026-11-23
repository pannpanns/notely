import os
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'backend'))

from deadlines import (
    load_deadlines,
    save_deadlines,
    add_or_update_deadline,
    delete_deadline,
    toggle_deadline_status,
    _clean_json_output,
    build_extraction_prompt,
    extract_deadlines_heuristically
)

class DeadlinesTests(unittest.TestCase):
    def test_crud_operations(self):
        with tempfile.TemporaryDirectory() as td:
            # Initial state is empty
            self.assertEqual(load_deadlines(td), [])

            # Add deadline
            item = add_or_update_deadline(td, {
                'title': 'Laporan Fisika Dasar',
                'course': 'Fisika 1',
                'deadline': '2026-11-20',
                'time': '23:59',
                'priority': 'tinggi'
            })
            self.assertTrue(item['id'].startswith('dl_'))
            self.assertEqual(item['status'], 'pending')
            self.assertEqual(item['reminder'], 30)
            loaded = load_deadlines(td)
            self.assertEqual(len(loaded), 1)
            self.assertEqual(loaded[0]['title'], 'Laporan Fisika Dasar')
            self.assertEqual(loaded[0]['reminder'], 30)

            # Toggle status
            toggled = toggle_deadline_status(td, item['id'])
            self.assertEqual(toggled['status'], 'completed')
            self.assertEqual(load_deadlines(td)[0]['status'], 'completed')

            # Update existing
            item['title'] = 'Laporan Fisika Lanjutan'
            add_or_update_deadline(td, item)
            self.assertEqual(load_deadlines(td)[0]['title'], 'Laporan Fisika Lanjutan')

            # Delete
            delete_deadline(td, item['id'])
            self.assertEqual(load_deadlines(td), [])

    def test_clean_json_output(self):
        sample = '''
        Berikut hasil ekstraksi tugas yang ditemukan:
        ```json
        [
            {
                "title": "Tugas 1 Pemrograman Web",
                "course": "Pemrograman Web",
                "deadline": "2026-11-25",
                "time": "14:00",
                "description": "Kirim link github",
                "priority": "tinggi"
            },
            {
                "title": "Kuis Basis Data",
                "course": "Basis Data",
                "deadline": "2026-11-30",
                "time": "10:00",
                "description": "Bab Normalisasi",
                "priority": "sedang"
            }
        ]
        ```
        Semoga membantu!
        '''
        results = _clean_json_output(sample)
        self.assertEqual(len(results), 2)
        self.assertEqual(results[0]['title'], 'Tugas 1 Pemrograman Web')
        self.assertEqual(results[0]['deadline'], '2026-11-25')
        self.assertEqual(results[0]['priority'], 'tinggi')
        self.assertEqual(results[1]['title'], 'Kuis Basis Data')

    def test_clean_json_invalid_fallback(self):
        invalid = "Tidak ditemukan tugas apapun."
        results = _clean_json_output(invalid)
        self.assertEqual(results, [])

    def test_build_extraction_prompt(self):
        prompt = build_extraction_prompt("Tugas dikumpul besok", "Cari tugas fisika")
        self.assertIn("Tugas dikumpul besok", prompt)
        self.assertIn("Cari tugas fisika", prompt)
        self.assertIn("YYYY-MM-DD", prompt)

    def test_heuristic_extractor_user_prompt(self):
        import datetime
        ref = datetime.date(2026, 11, 15)
        text = "Laporan praktikum fisika deadline 3 hari lagi jam 23:59, revisi proposal senin depan"
        items = extract_deadlines_heuristically(text, ref_date=ref)
        self.assertEqual(len(items), 2)
        self.assertEqual(items[0]['title'], 'Laporan Praktikum Fisika')
        self.assertEqual(items[0]['course'], 'Fisika')
        self.assertEqual(items[0]['deadline'], '2026-11-18')
        self.assertEqual(items[0]['time'], '23:59')
        self.assertEqual(items[1]['title'], 'Revisi Proposal')
        self.assertEqual(items[1]['deadline'], '2026-11-23')

    def test_heuristic_extractor_dates_and_course(self):
        import datetime
        ref = datetime.date(2026, 11, 15)
        text = "Tugas makalah Kecerdasan Buatan dikumpul jumat depan jam 12:00, kuis Basis Data tanggal 25 jam 10:00"
        items = extract_deadlines_heuristically(text, ref_date=ref)
        self.assertEqual(len(items), 2)
        self.assertIn('Kecerdasan Buatan', items[0]['title'])
        self.assertEqual(items[0]['time'], '12:00')
        self.assertEqual(items[0]['deadline'], '2026-11-20')
        self.assertIn('Basis Data', items[1]['title'])
        self.assertEqual(items[1]['time'], '10:00')
        self.assertEqual(items[1]['deadline'], '2026-11-25')

    def test_heuristic_extractor_google_meet_lecture(self):
        import datetime
        ref = datetime.date(2026, 11, 21)
        text = "Selamat siang, Bapak & teman-teman 🙏 Mohon izin mengirimkan link google meet untuk perkuliahan hari ini yang dijadwalkan pada jam 16.30-19.29 Link dapat diakses dibawah ini. https://meet.google.com/bfd-ghon-ght"
        items = extract_deadlines_heuristically(text, ref_date=ref)
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]['title'], 'Perkuliahan Google Meet')
        self.assertEqual(items[0]['deadline'], '2026-11-21')
        self.assertEqual(items[0]['time'], '16:30')
        self.assertIn('meet.google.com', items[0]['description'])

    def test_conversational_questions_ignored(self):
        import datetime
        ref = datetime.date(2026, 11, 21)
        text = "Jam berapa meet?"
        items = extract_deadlines_heuristically(text, ref_date=ref)
        self.assertEqual(len(items), 0)

    def test_heuristic_conversational_task_prompt(self):
        import datetime
        ref = datetime.date(2026, 11, 23)
        text = "hari ini saya ada tugas uts deadline nya jam 8 tolong buatkan"
        items = extract_deadlines_heuristically(text, ref_date=ref)
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]['title'], 'Tugas UTS')
        self.assertEqual(items[0]['deadline'], '2026-11-23')
        self.assertEqual(items[0]['time'], '20:00')
        self.assertEqual(items[0]['priority'], 'tinggi')

if __name__ == '__main__':
    unittest.main()

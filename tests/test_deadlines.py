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
    build_extraction_prompt
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
            loaded = load_deadlines(td)
            self.assertEqual(len(loaded), 1)
            self.assertEqual(loaded[0]['title'], 'Laporan Fisika Dasar')

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

if __name__ == '__main__':
    unittest.main()

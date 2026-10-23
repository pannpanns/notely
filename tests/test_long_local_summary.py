import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'backend'))
import providers


class LongLocalTests(unittest.TestCase):
    def test_utf8_lossless(self):
        text = ('Indonesia 中文 🙂\n' * 4000)
        chunks = providers._utf8_chunks(text, 14000)
        self.assertEqual(''.join(chunks), text)
        self.assertTrue(all(len(c.encode('utf-8')) <= 14000 for c in chunks))

    def test_long_transcript_is_processed_losslessly_without_total_byte_limit(self):
        text = ('AWAL-UNIK\n' + ('materi-中文🙂 pembahasan dan contoh.\n' * 2500) +
                'TENGAH-UNIK\n' + ('topik tambahan dan penjelasan.\n' * 2500) + 'AKHIR-UNIK')
        source_chunks = []
        calls = []
        progress = []

        def fake_ask(provider, model, key, prompt, endpoint, progress_callback):
            providers._local_context_size([{'content': prompt}])
            calls.append(prompt)
            if '<DATA>\n' in prompt:
                source_chunks.append(prompt.split('<DATA>\n', 1)[1].split('\n</DATA>', 1)[0])
                return f'Catatan bagian {len(source_chunks)} dengan fakta unik {len(source_chunks)}.'
            if '<CATATAN TERKUMPUL>' in prompt:
                return 'Ringkasan final lengkap dari seluruh transkrip.'
            return 'Catatan gabungan.'

        with patch.object(providers, '_ask', side_effect=fake_ask):
            result = providers.summarize('Uji', text, 'ollama', 'model', progress=progress.append)

        self.assertGreater(len(source_chunks), 1)
        self.assertEqual(''.join(source_chunks), text)
        self.assertIn('AWAL-UNIK', ''.join(source_chunks))
        self.assertIn('TENGAH-UNIK', ''.join(source_chunks))
        self.assertIn('AKHIR-UNIK', ''.join(source_chunks))
        self.assertIn('Ringkasan final lengkap', result)
        self.assertNotIn('Catatan pemrosesan', result)
        self.assertTrue(any(isinstance(event, dict) and event.get('stage') == 'Mode transkrip panjang' for event in progress))
        self.assertTrue(all(len(str(event.get('preview', ''))) <= providers.LOCAL_PREVIEW_CHARS for event in progress if isinstance(event, dict)))

    def test_failure_not_returned_as_final(self):
        with patch.object(providers, '_ask', side_effect=RuntimeError('putus')):
            with self.assertRaisesRegex(RuntimeError, 'putus'):
                providers.summarize('Uji', 'a' * 140000, 'ollama', 'model')

    def test_cloud_transcript_chunks_are_lossless_and_final_is_one_summary(self):
        text = 'awal ' + ('materi-中文🙂 ' * 2600) + ' akhir'
        source_chunks = []
        calls = []
        def fake_ask(provider, model, key, prompt, endpoint, progress):
            calls.append(prompt)
            if '<TRANSKRIP>\n' in prompt:
                source_chunks.append(prompt.split('<TRANSKRIP>\n', 1)[1].split('\n</TRANSKRIP>', 1)[0])
                return f'Catatan bagian {len(source_chunks)} dengan fakta unik {len(source_chunks)}.'
            if 'Satukan seluruh catatan' in prompt:
                return 'Satu ringkasan final lengkap.'
            return 'Catatan gabungan dengan fakta unik.'
        with patch.object(providers, '_ask', side_effect=fake_ask):
            result = providers.summarize('Uji', text, 'gemini', 'model', 'key')
        self.assertEqual(''.join(source_chunks), text)
        self.assertIn('Satu ringkasan final', result)
        self.assertTrue(any('fakta unik' in p for p in calls))

    def test_prompt_and_context_are_not_silently_truncated(self):
        instruction = 'INSTRUKSI-PANJANG-' + ('x' * 5000)
        context = 'KONTEKS-PANJANG-' + ('y' * 2000)
        with patch.object(providers, '_ask', return_value='Hasil lengkap') as ask:
            providers.summarize('Uji', 'Transkrip pendek.', 'gemini', 'model', 'key', context=context, instructions=instruction)
        prompt = ask.call_args.args[3]
        self.assertIn(instruction, prompt)
        self.assertIn(context, prompt)


if __name__ == '__main__':
    unittest.main()

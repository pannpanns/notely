import sys, unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import providers
class LongLocalTests(unittest.TestCase):
    def test_utf8_lossless(self):
        text=('Indonesia 中文 🙂\n'*4000)
        chunks=providers._utf8_chunks(text,14000)
        self.assertEqual(''.join(chunks),text)
        self.assertTrue(all(len(c.encode())<=14000 for c in chunks))
    def test_long_transcript_reaches_all_chunks_and_final(self):
        text='awal '+('abc 中文 🙂 '*9000)+' akhir'
        calls=[]
        def ask(provider,model,key,prompt,endpoint,progress):
            providers._local_context_size([{'content':prompt}])
            calls.append(prompt)
            return 'Catatan penting.'
        with patch.object(providers,'_ask',side_effect=ask):
            result=providers.summarize('Uji',text,'ollama','model')
        source=[x.split('<DATA>\n',1)[1].split('\n</DATA>',1)[0] for x in calls if '<DATA>' in x]
        self.assertEqual(''.join(source),text)
        self.assertIn('Catatan penting.',result)
        self.assertIn('Satukan catatan',calls[-1])
    def test_failure_not_returned_as_final(self):
        with patch.object(providers,'_ask',side_effect=RuntimeError('putus')):
            with self.assertRaisesRegex(RuntimeError,'putus'):
                providers.summarize('Uji','a'*80000,'ollama','model')
if __name__=='__main__':unittest.main()

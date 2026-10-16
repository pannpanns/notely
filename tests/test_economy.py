import sys, unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import providers
class EconomyTests(unittest.TestCase):
 def test_economy_preserves_content_and_prompt(self):
  with patch.object(providers,'_ask',return_value='Hasil') as ask:
   providers.summarize('Title','  [00:09] Fakta satu\n\n [00:12] Fakta dua ','gemini','model','key',instructions='Bahasa Inggris',economy=True)
  prompt=ask.call_args.args[3]
  for text in ('[00:09] Fakta satu','[00:12] Fakta dua','Bahasa Inggris','120–250'):self.assertIn(text,prompt)
  self.assertEqual(ask.call_count,1)
 def test_normal_style_unchanged(self):
  with patch.object(providers,'_ask',return_value='Hasil') as ask:
   providers.summarize('Title','Fakta','gemini','model','key')
  self.assertIn('300–600',ask.call_args.args[3])

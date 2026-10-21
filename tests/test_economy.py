import sys, unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import providers
class EconomyTests(unittest.TestCase):
 def test_legacy_economy_flag_does_not_force_short_summary(self):
  with patch.object(providers,'_ask',return_value='Hasil') as ask:
   providers.summarize('Title','  [00:09] Fakta satu\n\n [00:12] Fakta dua ','gemini','model','key',instructions='Bahasa Inggris',economy=True)
  prompt=ask.call_args.args[3]
  for text in ('[00:09] Fakta satu','[00:12] Fakta dua','Bahasa Inggris'):self.assertIn(text,prompt)
  self.assertNotIn('120–250',prompt)
  self.assertEqual(ask.call_count,1)
 def test_normal_style_requests_complete_summary(self):
  with patch.object(providers,'_ask',return_value='Hasil') as ask:
   providers.summarize('Title','Fakta','gemini','model','key')
  self.assertIn('ringkasan lengkap',ask.call_args.args[3])
  self.assertNotIn('300–600',ask.call_args.args[3])
if __name__=='__main__':unittest.main()

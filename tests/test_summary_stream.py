import io,json,sys,unittest
from pathlib import Path
from unittest.mock import patch,Mock
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import providers
class StreamTests(unittest.TestCase):
 def stream(self,items):
  opener=Mock();opener.open.return_value=io.BytesIO(b''.join(json.dumps(i).encode()+b'\n' for i in items));events=[]
  with patch.object(providers.urllib.request,'build_opener',return_value=opener):
   result=providers._ask('ollama','local','','prompt',progress=events.append)
  return result,events,opener
 def test_incremental_and_final(self):
  result,events,opener=self.stream([{'message':{'content':'Halo '}},{'message':{'content':'dunia'},'done':True}])
  self.assertEqual(result,'Halo dunia');self.assertEqual(events[0]['preview'],'')
  self.assertIn('Halo ',[e['preview'] for e in events]);self.assertEqual(events[-1]['preview'],result)
  body=json.loads(opener.open.call_args.args[0].data);self.assertTrue(body['stream']);self.assertEqual(body['options']['num_predict'],4096)
  self.assertEqual(opener.open.call_count,1)
 def test_incomplete_never_success(self):
  for items in ([{'message':{'content':'partial'}}],[{'error':'out of memory'}],[{'done':True}],[{'message':{'content':'partial'},'done':True,'done_reason':'length'}]):
   with self.subTest(items=items),self.assertRaises(RuntimeError):self.stream(items)
 def test_timeout_no_replay(self):
  opener=Mock();opener.open.side_effect=TimeoutError()
  with patch.object(providers.urllib.request,'build_opener',return_value=opener),self.assertRaisesRegex(RuntimeError,'tidak merespons atau koneksinya terputus'):providers._stream_local('model','prompt')
  self.assertEqual(opener.open.call_count,1)
 def test_broken_json(self):
  opener=Mock();opener.open.return_value=io.BytesIO(b'not json\n')
  with patch.object(providers.urllib.request,'build_opener',return_value=opener),self.assertRaisesRegex(RuntimeError,'Format respons'):providers._stream_local('model','prompt')
 def test_stage_forwarded(self):
  events=[]
  def fake(model,prompt,progress):progress({'preview':'result','message':'writing'});return 'result'
  with patch.object(providers,'_stream_local',side_effect=fake):providers.summarize('Title','transcript','ollama','local',progress=events.append)
  self.assertEqual([x for x in events if isinstance(x,dict)][-1]['stage'],'Menyusun ringkasan')

 def test_local_context_is_bounded_and_scales_with_input(self):
  self.assertEqual(providers._local_context_size([{'content':'short'}]),8192)
  self.assertEqual(providers._local_context_size([{'content':'x'*10000}]),16384)
  self.assertEqual(providers._local_context_size([{'content':'x'*24000}]),32768)
  with self.assertRaises(ValueError):providers._local_context_size([{'content':'x'*40000}])

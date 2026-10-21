import io,json,sys,unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'backend'))
import providers

def frames(items):
 return b''.join(b'data: '+(x.encode() if isinstance(x,str) else json.dumps(x,ensure_ascii=False).encode())+b'\r\n\r\n' for x in items)
class CloudStreamTests(unittest.TestCase):
 def run_stream(self,provider,items):
  events=[]
  with patch.object(providers.urllib.request,'urlopen',return_value=io.BytesIO(frames(items))) as request:
   result=providers._ask(provider,'model','key','prompt','https://example.com/v1',events.append)
  return result,events,request.call_args.args[0]
 def test_openai_compatible(self):
  for provider in ('openai','groq','openrouter','custom'):
   with self.subTest(provider=provider):
    result,events,req=self.run_stream(provider,[{'choices':[{'delta':{'content':'Halo '}}]},{'choices':[{'delta':{'content':'世界'},'finish_reason':'stop'}]},'[DONE]'])
    self.assertEqual(result,'Halo 世界');self.assertTrue(json.loads(req.data)['stream'])
    self.assertIn('Halo ',[x['preview'] for x in events]);self.assertEqual(events[-1]['preview'],result)
 def test_gemini(self):
  result,events,req=self.run_stream('gemini',[{'candidates':[{'content':{'parts':[{'text':'rahasia','thought':True},{'text':'Halo'}]}}]},{'candidates':[{'content':{'parts':[{'text':' dunia'}]},'finishReason':'STOP'}]}])
  self.assertEqual(result,'Halo dunia');self.assertIn(':streamGenerateContent?alt=sse',req.full_url)
 def test_claude(self):
  result,events,req=self.run_stream('claude',[{'type':'ping'},{'type':'content_block_delta','delta':{'type':'text_delta','text':'Halo'}},{'type':'message_delta','delta':{'stop_reason':'end_turn'}},{'type':'message_stop'}])
  self.assertEqual(result,'Halo');self.assertTrue(json.loads(req.data)['stream'])
 def test_length_limit_continues_automatically(self):
  first=io.BytesIO(frames([{'choices':[{'delta':{'content':'Awal '},'finish_reason':'length'}]},'[DONE]']))
  second=io.BytesIO(frames([{'choices':[{'delta':{'content':'lanjutan'},'finish_reason':'stop'}]},'[DONE]']))
  events=[]
  with patch.object(providers.urllib.request,'urlopen',side_effect=[first,second]) as request:
   result=providers._ask('openai','model','key','prompt','',events.append)
  self.assertEqual(result,'Awal lanjutan');self.assertEqual(request.call_count,2)
  continuation=json.loads(request.call_args.args[0].data)
  self.assertIn('Lanjutkan jawaban',continuation['messages'][-1]['content'])
 def test_response_over_100k_has_no_application_length_cap(self):
  items=[]
  for i in range(11):
   items.append({'choices':[{'delta':{'content':('x'*10000)},'finish_reason':'stop' if i==10 else None}]})
  events=[]
  with patch.object(providers.urllib.request,'urlopen',return_value=io.BytesIO(frames(items+['[DONE]']))):
   result=providers._ask('openai','model','key','prompt',progress=events.append)
  self.assertEqual(len(result),110000)
 def test_incomplete_and_length_never_succeed(self):
  for items in ([{'choices':[{'delta':{'content':'partial'}}]}],[{'choices':[{'delta':{'content':'partial'},'finish_reason':'length'}]}],['[DONE]'],[{'error':{'message':'secret'}}]):
   with self.subTest(items=items),self.assertRaises(RuntimeError):self.run_stream('openai',items)
 def test_sse_comments_and_multiline(self):
  result=list(providers._sse_events(io.BytesIO(b': heartbeat\r\nevent: message\r\ndata: {"a":\r\ndata: 1}\r\n\r\n')))
  self.assertEqual(result,[{'a':1}])
 def test_broken_frame(self):
  with self.assertRaises(RuntimeError):list(providers._sse_events(io.BytesIO(b'data: {}')))
 def test_partial_not_retried(self):
  events=[]
  with patch.object(providers.urllib.request,'urlopen',return_value=io.BytesIO(frames([{'choices':[{'delta':{'content':'partial'}}]}]))) as request:
   with self.assertRaises(RuntimeError):providers._ask('openai','model','key','prompt',progress=events.append)
  self.assertEqual(request.call_count,1);self.assertEqual(events[-1]['preview'],'partial')
if __name__=='__main__':unittest.main()

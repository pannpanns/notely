function updateSummaryStream(job){
 engine.summaryStream=job;
 const p=job.progress||{};
 if(typeof p.preview==='string'&&p.preview!==engine.summaryPreview){engine.summaryPreview=p.preview;engine.summaryLastText=Date.now();}
 paintSummaryStream();
}
function paintSummaryStream(){
 if(state.selected!==engine.summarySession||!engine.summaryStream)return;
 const job=engine.summaryStream,p=job.progress||{},status=document.getElementById('summary-progress');
 const seconds=Math.max(0,Math.floor((Date.now()-engine.summaryStarted)/1000));
 const quiet=Math.max(0,Math.floor((Date.now()-engine.summaryLastText)/1000));
 if(status)status.textContent=[p.stage,engine.summaryPreview?'Menyusun ringkasan…':'Membaca dan merangkum transkrip…'].filter(Boolean).join(' · ');
 let live=document.getElementById('summary-live');
 if(!live&&engine.summaryPreview&&state.summaryError){live=document.createElement('div');live.id='summary-live';live.className='summary-paper real-summary';document.getElementById('summary-scroll')?.prepend(live);}
 if(live){
  const value=engine.summaryPreview||'';
  document.getElementById('summary-scroll')?.classList.toggle('has-preview',Boolean(value));
  if(live._streamText!==value){
   const scroll=document.getElementById('summary-scroll'),bottom=scroll&&scroll.scrollHeight-scroll.scrollTop-scroll.clientHeight<70;
   live.innerHTML=value?'<p class="muted-block">Pratinjau sementara — belum disimpan sebagai ringkasan final.</p>'+renderSummaryMarkdown(value):'';
   live._streamText=value;if(bottom)scroll.scrollTop=scroll.scrollHeight;
  }
 }
}
const refreshBeforeStream=refreshPage;
refreshPage=function(){refreshBeforeStream();paintSummaryStream();};
setInterval(paintSummaryStream,1000);

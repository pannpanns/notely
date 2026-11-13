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
 if(status){
  const stage=p.stage||p.message||job.message||'Memproses transkrip…';
  const activity=quiet>=30?'Menyusun ringkasan…':p.phase==='final'?'Menyatukan hasil akhir…':p.phase==='reducing'?'Menggabungkan catatan…':p.phase==='collecting'?'Membaca bagian transkrip…':'Menyusun ringkasan…';
  status.textContent=[stage,activity].filter(Boolean).join(' · ');
 }
 let live=document.getElementById('summary-live');
 if(!live&&engine.summaryPreview&&state.summaryError){
  live=document.createElement('div');
  live.id='summary-live';
  live.className='summary-paper real-summary';
  document.getElementById('summary-scroll')?.prepend(live);
 }
 if(live){
  const value=engine.summaryPreview||'';
  if(state.summaryError){
   live.style.display='';
   document.getElementById('summary-scroll')?.classList.add('has-preview');
   if(live._streamText!==value){
    live.innerHTML=value?'<p class="muted-block">Pratinjau sementara — proses terhenti sebelum selesai.</p>'+renderSummaryMarkdown(value):'';
    live._streamText=value;
   }
  }else{
   if(live._streamText!==value){
    const scroll=document.getElementById('summary-scroll'),bottom=scroll&&scroll.scrollHeight-scroll.scrollTop-scroll.clientHeight<70;
    live.innerHTML=value?renderSummaryMarkdown(value):'';
    live._streamText=value;
    if(bottom)scroll.scrollTop=scroll.scrollHeight;
   }
  }
 }
}
const refreshBeforeStream=refreshPage;
refreshPage=function(){refreshBeforeStream();paintSummaryStream();};
setInterval(paintSummaryStream,1000);

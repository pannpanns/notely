/* Notely 2.2 desktop adapter. No CDN, credentials protected locally with Windows DPAPI. */
'use strict';
const engine = {base:'',token:'',ready:false, gpu:false, gpuReport:'', selectedSpeaker:null, selectedMic:null, state:'idle', since:0, polling:false, currentJob:'',summaryJob:'', transcribeJob:'',ollamaJob:'',modelReady:false,previews:[],previewSignature:'', lastSession:null, cudaJob:'', cudaDownloaded:false, hasNvidia:false, preferCuda:true};
const choiceIds = {
 'maleo-turbo':'Indonesia Turbo (terlatih, GPU)', 'maleo-small':'Indonesia Small (terlatih)',
 'whisper-small':'Whisper small (umum)', 'whisper-medium':'Whisper medium (umum)',
 'whisper-turbo':'Whisper turbo (umum)', 'whisper-large':'Whisper large-v3 (umum)'
};
function native(){return window.__TAURI__?.core?.invoke;}
async function api(path,payload){
 if(!engine.ready)throw Error('Mesin transkripsi belum siap. Tutup dan buka Notely kembali.');
 const res=await fetch(engine.base+path,{method:payload===undefined?'GET':'POST',headers:{Authorization:'Bearer '+engine.token,...(payload===undefined?{}:{'Content-Type':'application/json'})},...((payload===undefined)?{}:{body:JSON.stringify(payload)})});
 const obj=await res.json(); if(!res.ok||obj.error)throw Error(obj.error||'Server tidak merespons.'); return obj;
}
function alertError(error){toast(String(error?.message||error).slice(0,200));}
async function startup(){
 if(!native()) {window.__notelyStatus='Mode pratinjau — backend desktop tidak tersedia';render();return;}
 let data;for(let attempt=0;attempt<30;attempt++){
  try{data=await native()('bridge_connection');if(data?.url&&data?.token)break;}catch{}
  await new Promise(r=>setTimeout(r,400));
 }
 if(!data?.url) {window.__notelyStatus='Backend tidak terhubung';render();return;}
 engine.base=data.url;engine.token=data.token;engine.ready=true;
 for(let attempt=0;attempt<50;attempt++){
  try{await api('/status');break;}catch(e){if(attempt===49){engine.ready=false;window.__notelyStatus='Backend gagal memulai';render();return;}await new Promise(r=>setTimeout(r,500));}
 }
 window.__notelyStatus='Mesin lokal terhubung';
 try{const saved=await api('/preferences',{});state.savedKeys=saved.keys||{};state.apiKey=state.savedKeys[state.settings.provider]||'';state.modelStorage=saved.modelRoot||'';}catch(e){alertError(e);}
 if(!safeStore.get('notelySetupComplete'))showWizard();
 await Promise.allSettled([loadSessions(),findDevices(),detectGpu()]);engine.deviceChecked=true;render();
 const welcomeStart=document.getElementById('wizard-start');if(welcomeStart){welcomeStart.disabled=false;welcomeStart.textContent='Mulai persiapan →';}
 const gpuChoice=document.getElementById('wizard-use-cuda');if(gpuChoice){gpuChoice.disabled=!engine.hasNvidia;gpuChoice.checked=engine.hasNvidia;}
 const gpuNote=document.getElementById('wizard-gpu-note');if(gpuNote)gpuNote.textContent=window.__notelyGpuStatus||'';
 if(typeof loadLocalCatalog==='function')loadLocalCatalog();
 if(typeof loadTranscriptionSizes==='function')loadTranscriptionSizes();
 setInterval(poll,200);
 if(!safeStore.get('notelySetupComplete')) showWizard();else activateSavedModel();
}
async function activateSavedModel(){
 try{if(!await ensureModelStorage())return;const j=await api('/model/prepare',{model:choiceIds[state.settings.model],gpu:engine.preferCuda?'Otomatis':'CPU'});engine.currentJob=j.job;
  window.__notelyStatus='Memuat model pilihan terakhir…';
 }catch(e){window.__notelyStatus='Model perlu disiapkan · '+e.message;}
}
async function loadSessions(){
 state.folders=await api('/folders');
 const result=await api('/sessions');let found=Array.isArray(result)?result:[];
 meetings.splice(0,meetings.length);
 for(const x of found.sort((a,b)=>String(a.id).localeCompare(String(b.id)))){
  meetings.push({id:x.id,folder:x.folder||'',title:x.title||'Rekaman',when:(x.date||x.id).replace('T',' ').replaceAll('_',' '),duration:'',type:'Lokal',sizeBytes:x.sizeBytes||0,audioBytes:x.audioBytes||0,summary:null,context:'',rows:[]});
 }
 if(!meetings.some(m=>m.id===state.selected))state.selected=meetings.at(-1)?.id||null;
}
async function findDevices(){
 const response=await api('/devices');const speakers=response.speakers||[];
 engine.speakers=speakers;engine.microphones=response.microphones||[];
 engine.selectedSpeaker=speakers.find(d=>d.index===response.default)||speakers[0]||null;
 if(!engine.selectedSpeaker) window.__notelyStatus='Speaker belum ditemukan · periksa output Windows';
}
async function detectGpu(){
 try{const x=await api('/gpu');engine.gpuReport=String(x.report||'');
 engine.hasNvidia=Boolean(x.nvidia);
 engine.cudaDownloaded=Boolean(x.cudaInstalled);
 const match=engine.gpuReport.match(/CTranslate2:\s*(\d+)\s*GPU/);
 engine.gpu=Boolean(match&&Number(match[1])>0&&engine.cudaDownloaded);
 window.__notelyGpuStatus=engine.hasNvidia?'NVIDIA terdeteksi: '+x.nvidia+' · '+(engine.cudaDownloaded?'Paket GPU tersedia.':'Percepatan CUDA dapat diunduh saat persiapan.'):'Tidak ada NVIDIA yang terdeteksi. CPU digunakan otomatis.';
 }catch(err){window.__notelyGpuStatus='Pemeriksaan GPU gagal: '+err.message;engine.gpu=false;}
}
async function loadSession(id){
 state.summaryError='';
 if(!id||id==='empty')return;
 const data=await api('/session?id='+encodeURIComponent(id));
 const m=meetings.find(x=>x.id===id);if(!m)return;
 m.title=data.meta?.judul||m.title;
 m.rows=(data.segments||[]).map(s=>[Number(s.start||0),String(s.text||'')]);
 if(data.edited&&data.transcript){m.rows=[[0,data.transcript]];}
 m.realSummary=data.summary||'';m.sizeBytes=data.sizeBytes||0;m.audioBytes=data.audioBytes||0;
 m.summary=Boolean(data.summary);m.instructions=m.instructions||'';m.context='';
 m.when=data.meta?.tanggal||m.when;
 state.selected=id;state.page='notes';render();
}
let pollBusy=false;
async function poll(){
 if(!engine.ready||pollBusy)return;pollBusy=true;
 try{
 const s=await api('/status?since='+engine.since);engine.since=s.latest||engine.since;
 engine.state=s.state;
 if(typeof updateRecordingDock==='function')updateRecordingDock(s.spectra||[],s.state,s.elapsed);
 const nextPreview=JSON.stringify(s.preview||[]);
 if(nextPreview!==engine.previewSignature){
   engine.previewSignature=nextPreview;engine.previews=s.preview||[];
   if(state.recording&&state.page==='notes')updateLiveTranscript(getMeeting());
 }
 window.__notelyStatus=s.state==='recording'?'● Sedang merekam · '+(s.backend||'AI diproses'):s.state==='stopping'?'Menyimpan dan menyelesaikan transkrip…':s.backend==='Belum diuji'?'Model belum disiapkan':(s.backend||'Siap');
 for(const e of s.events||[]){
  if(e.event==='transcript'){
   const m=getMeeting();if(!m||m.id==='empty')continue;
   for(const seg of e.value||[])m.rows.push([Number(seg.start||0),String(seg.text||'')]);
   if(state.page==='notes')updateLiveTranscript(m);
  }
  if(e.event==='finished'){
   engine.state='idle';state.recording=false;clearInterval(state.recordTimer);
   await native()('set_busy_guard',{value:false}).catch(()=>{});
   await loadSessions();const session=e.value?.session;
   if(session)await loadSession(session); else render();
   toast('Rekaman disimpan. Transkrip sudah tersedia.');
  }
  if(e.event==='notice')toast(e.value);
 }
 if(engine.transcribeJob&&s.jobs?.[engine.transcribeJob]?.status==='done'){
  engine.transcribeJob='';if(state.selected)await loadSession(state.selected);toast('Transkripsi audio selesai.');
 }
 if(engine.transcribeJob&&s.jobs?.[engine.transcribeJob]?.status==='error'){
  const error=s.jobs[engine.transcribeJob].message;engine.transcribeJob='';alertError(error);
 }
 if(engine.summaryJob&&s.jobs?.[engine.summaryJob]?.status==='running'){
  updateSummaryStream(s.jobs[engine.summaryJob]);
 }
 if(engine.summaryJob&&s.jobs?.[engine.summaryJob]?.status==='done'){
  engine.summaryJob='';state.summaryLoading=false;
  if(state.selected===engine.summarySession)await loadSession(engine.summarySession);
  else refreshPage();
  engine.summaryPreview='';engine.summaryStream=null;toast('Ringkasan selesai.');
 }
 updateStorageProgress(s.jobs||{});
 if(typeof updateDownloadCenter==='function')updateDownloadCenter(s.jobs||{});
 if(engine.ollamaJob&&s.jobs?.[engine.ollamaJob]?.status==='done'){
  engine.ollamaJob='';if(typeof loadLocalCatalog==='function')loadLocalCatalog();toast('Model lokal Ollama berhasil diunduh.');
 }
 if(engine.ollamaJob&&s.jobs?.[engine.ollamaJob]?.status==='error'){
  const err=s.jobs[engine.ollamaJob].message;engine.ollamaJob='';alertError('Ollama: '+err);
  if(state.page==='settings'&&state.tab==='summary')refreshPage();
 }
 if(engine.summaryJob&&s.jobs?.[engine.summaryJob]?.status==='error'){
  updateSummaryStream(s.jobs[engine.summaryJob]);const error=s.jobs[engine.summaryJob].message;engine.summaryJob='';state.summaryLoading=false;state.summaryError=error;refreshPage();alertError(error);
 }
 if(engine.cudaJob&&s.jobs?.[engine.cudaJob]){
  const j=s.jobs[engine.cudaJob]; const statusEl=document.getElementById('wizard-status');
  if(statusEl&&j.status==='running')statusEl.textContent='Menyiapkan NVIDIA: '+(j.message||'Mengunduh komponen…');
  if(j.status==='done'){
    engine.cudaJob='';engine.cudaDownloaded=true;
    if(statusEl)statusEl.textContent='CUDA terunduh. Sekarang menyiapkan model bahasa…';
    try{const res=await api('/model/prepare',{model:choiceIds[state.settings.model],gpu:'CUDA'});engine.currentJob=res.job;}
    catch(err){if(statusEl)statusEl.textContent='Gagal menyiapkan model GPU: '+err.message; const btn=document.getElementById('wizard-install');if(btn)btn.disabled=false;}
  } else if(j.status==='error'){
    engine.cudaJob=''; if(statusEl)statusEl.textContent='CUDA gagal: '+j.message+'. Anda bisa mencoba mode CPU.';
    const btn=document.getElementById('wizard-install');if(btn)btn.disabled=false;
    const cb=document.getElementById('wizard-use-cuda');if(cb)cb.checked=false;
  }
 }
 if(engine.currentJob&&s.jobs?.[engine.currentJob]){
  const j=s.jobs[engine.currentJob];const el=document.getElementById('wizard-status');
  if(el)el.textContent=(j.status==='running'?'Menyiapkan model: ':'')+(j.message||'');
  if(j.status==='done'){
   engine.currentJob='';engine.modelReady=true;
   const el=document.getElementById('wizard-status');
   if(el)el.textContent='Model siap · '+(j.result?.backend||'');
   const button=document.getElementById('wizard-finish');if(button){button.disabled=false;button.textContent='Selesai & buka Notely';}
   window.__notelyStatus='Model siap · '+(j.result?.backend||'');
  }
  if(j.status==='error'){
   engine.currentJob='';const el=document.getElementById('wizard-status');if(el)el.textContent='Persiapan gagal: '+j.message;
   const button=document.getElementById('wizard-install');if(button)button.disabled=false;
  }
 }
 if(state.recording){const timer=document.getElementById('demo-timer');if(timer)timer.textContent=ts(Number(s.elapsed||0));}
 }catch(e){ /* Device may be temporarily busy; next polling cycle retries. */ }
 finally{pollBusy=false;}
}
function updateLiveTranscript(m){
 const list=document.getElementById('transcript-list');
 if(!list)return;
 const bottom=list.scrollHeight-list.scrollTop-list.clientHeight<60;
 list.innerHTML=transcriptList(m);
 if(state.recording&&engine.previews?.length){
  const draft=engine.previews.filter(p=>p.text).map(p=>`<div class="transcript-row transcript-draft"><span class="timestamp">[${ts(Number(p.start)||0)}]</span><p>${esc(p.text)} <small class="draft-label">· Diketik sementara</small></p></div>`).join('');
  list.insertAdjacentHTML('beforeend',draft);
 }
 if(bottom)list.scrollTop=list.scrollHeight;
 const head=document.querySelector('.transcript-pane .pane-head .small');if(head)head.textContent=m.rows.length+' bagian';
}
async function startRecording(){
 if(!engine.ready)return alertError('Aplikasi belum siap');
 if(engine.state!=='idle'||state.recordStarting)return toast('Perekaman sebelumnya belum selesai.');
 if(!engine.modelReady)return showWizard();
 if(!engine.selectedSpeaker)return toast('Speaker belum tersedia. Buka Pengaturan → Rekaman.');
 state.recordStarting=true;
 try{
  const title='Rekaman '+new Date().toLocaleString('id-ID',{day:'numeric',month:'short',hour:'2-digit',minute:'2-digit'});
  const opts={title,folder:state.folderFilter&&state.folderFilter!=='all'?state.folderFilter:'',model:choiceIds[state.settings.model]||choiceIds['maleo-turbo'],gpu:engine.preferCuda?'Otomatis':'CPU',language:state.settings.language==='en'?'Inggris':'Indonesia',speaker:engine.selectedSpeaker.index};
  if(engine.selectedMic!==null)opts.microphone=engine.selectedMic;
  const res=await api('/start',opts);
  state.recording=true;engine.state='recording';state.started=Date.now();state.recordSeconds=0;
  const m={id:res.session,title,when:new Date().toLocaleString('id-ID'),rows:[],summary:null,context:'',type:'Lokal'};
  meetings.push(m);state.selected=m.id;state.page='notes';state.noteTab='transcript';engine.previews=[];render();
  if(native())await native()('set_busy_guard',{value:true}).catch(()=>{});
  state.recordTimer=setInterval(()=>{const t=document.getElementById('demo-timer');if(t)t.textContent=ts(Math.floor((Date.now()-state.started)/1000));},1000);
  toast('Perekaman dimulai. Suara speaker sedang direkam.');
 }catch(e){alertError(e);}finally{state.recordStarting=false;}
}
const originalStrip=appStrip;
appStrip=function(){const base=originalStrip();
 if(engine.state!=='stopping')return base;
 return base.replace('</div>',' <button class="button small" data-action="finish-partial" title="Simpan audio lengkap meskipun transkrip belum selesai">Simpan audio & teks sementara</button></div>');
};
async function savePartial(){
 try{await api('/finish-partial',{});toast('Menyimpan audio asli dan teks yang telah selesai.');}
 catch(e){alertError(e);}
}
async function stopRecording(){
 if(!state.recording)return;
 try{await api('/stop',{});state.recording=false;clearInterval(state.recordTimer);engine.state='stopping';render();
 toast('Audio dihentikan. Menyelesaikan transkrip, mohon tunggu.');}
 catch(e){alertError(e);}
}
async function generateRealSummary(){
 if(state.summaryLoading||state.summarySubmitting||document.querySelector('[data-action="confirm-real-summary"]'))return;
 if(engine.state!=='idle')return toast('Selesaikan perekaman terlebih dahulu.');
 const m=getMeeting();if(m.id==='empty'||!m.rows.length)return toast('Transkrip masih kosong.');
 const provider=state.settings.provider;const online=provider!=='ollama';
 if(online&&!state.apiKey.trim()){state.page='settings';state.tab='summary';render();return toast('Masukkan API key di Pengaturan → Ringkasan AI.');}
 if(online){modal('Kirim transkrip ke '+providers[provider].name+'?',
  'Untuk membuat ringkasan, isi transkrip akan dikirim ke penyedia AI yang Anda pilih. Audio asli tidak dikirim.',
  [['Batal','close-modal',''],['Setuju & ringkas','confirm-real-summary','accent']]);
 document.querySelector('[data-action="confirm-real-summary"]')?.focus();return;}
 await confirmRealSummary();
}
async function reprocessAudio(){
 if(!engine.modelReady)return showWizard();
 const m=getMeeting();if(!m?.id||m.id==='empty')return;
 try{const j=await api('/reprocess',{session:m.id,model:choiceIds[state.settings.model],gpu:engine.preferCuda?'Otomatis':'CPU'});engine.transcribeJob=j.job;toast('Memproses audio dengan model yang dipilih…');}
 catch(e){alertError(e);}
}
async function downloadOllama(){
 if(state.settings.provider!=='ollama')return;
 const model=state.settings.summaryModel;
 if(engine.ollamaJob)return toast('Satu model sedang diunduh.');
 try{if(!await ensureModelStorage())return;const j=await api('/ollama/pull',{model});engine.ollamaJob=j.job;
  if(typeof beginDownload==='function')beginDownload(j.job,model);
  toast('Notely menyiapkan mesin AI dan model '+model+' secara otomatis…');}
 catch(e){alertError('Persiapan AI lokal gagal: '+e.message);}
}
async function refreshProviderModels(){
 try{
  const which=state.settings.provider;
  if(which==='ollama'&&typeof loadLocalCatalog==='function')return loadLocalCatalog();
  if(which!=='ollama'&&!state.apiKey.trim())return toast('Masukkan API key terlebih dahulu.');
  const data=await api('/provider/models',{provider:which,apiKey:state.apiKey,endpoint:state.settings.endpoint||''});
  if(!data.models?.length)throw Error('Model tidak ditemukan di akun ini.');
  providers[which].options=data.models.map(s=>[s,s]);
  state.settings.providerModels={...(state.settings.providerModels||{}),[which]:data.models};
  if(!data.models.includes(state.settings.summaryModel))state.settings.summaryModel=data.models[0];
  persistSettings();refreshPage();toast(data.models.length+' model tersedia dari '+providers[which].name+'.');
 }catch(e){alertError(e);}
}
async function confirmRealSummary(){
 if(state.summaryLoading||state.summarySubmitting)return;
 state.summarySubmitting=true;
 closeModal();const m=getMeeting();const sentInstructions=m.instructions||'';
 const model=state.settings.customModel?.trim()&&state.settings.summaryModel==='custom'?state.settings.customModel:state.settings.summaryModel;
 try{const result=await api('/summary',{session:m.id,provider:state.settings.provider,model,apiKey:state.apiKey,geminiFallback:state.settings.geminiFallback!==false,context:'',instructions:sentInstructions,endpoint:state.settings.endpoint||''});
  if(m.instructions===sentInstructions)m.instructions='';
  engine.summaryJob=result.job;engine.summarySession=m.id;engine.summaryStarted=Date.now();engine.summaryLastText=Date.now();engine.summaryPreview='';engine.summaryStream=null;state.summaryLoading=true;state.noteTab='summary';state.promptOpen=false;state.summaryError='';toast('AI sedang membuat ringkasan…');refreshPage();
 }catch(e){alertError(e);}finally{state.summarySubmitting=false;}
}
/* Safe read-only markdown renderer (keeps escaped HTML). */
function summaryPaperReal(m){
 if(!m.realSummary){return `<div class="summary-empty"><div class="empty-icon">${icon('file')}</div><h3>Belum ada ringkasan</h3><p>Pilih AI lokal atau layanan API untuk merangkum transkrip ini.</p><button class="button accent" data-action="generate-summary" ${state.recording?'disabled':''}>${icon('spark')} Buat ringkasan AI</button></div>`;}
 return '<article class="summary-paper real-summary">'+renderSummaryMarkdown(m.realSummary)+'</article>';
}
summaryPaper=summaryPaperReal;
/* Match approved v2.1 layout while wiring the real engine. */
/* Add re-transcribe action while preserving approved split-panel UI. */
const originalSummaryTab=summaryTab;
summaryTab=function(){const html=originalSummaryTab();
 if(state.settings.provider!=='ollama')return html;
 return html.replace('<div class="space-between" style="margin-top:20px">',
 `<div style="margin-top:12px;display:flex;gap:8px;flex-wrap:wrap"><button class="button" data-action="download-ollama-model">${icon('download')} Unduh model Ollama</button><button class="button" data-action="ollama-gpu-test">${icon('cpu')} Tes GPU Ollama</button></div><p class="field-help" style="margin-top:7px">Tes menjalankan inferensi singkat dengan model pilihan, lalu memeriksa alokasi VRAM dari runtime Ollama. Model harus sudah diunduh.</p><div class="space-between" style="margin-top:20px">`);
};
let ollamaGpuTestRunning=false;
async function testOllamaGpu(){
 if(ollamaGpuTestRunning)return;
 if(engine.state!=='idle')return toast('Selesaikan perekaman atau proses aktif terlebih dahulu.');
 const model=state.settings.summaryModel;
 ollamaGpuTestRunning=true;toast('Menjalankan tes singkat Ollama dan memeriksa alokasi GPU…');
 try{
  const r=await api('/ollama/gpu-test',{model});
  const fmt=n=>{if(!Number.isFinite(Number(n))||Number(n)<=0)return '0 B';const u=['B','KB','MB','GB'];let v=Number(n),i=0;while(v>=1024&&i<u.length-1){v/=1024;i++;}return v.toFixed(i?1:0)+' '+u[i];};
  const models=(r.models||[]).map(m=>`<li><strong>${esc(m.name)}</strong>: ${esc(m.processor)}; VRAM ${esc(fmt(m.vramBytes))} dari ukuran model ${esc(fmt(m.sizeBytes))}${m.contextLength?`; konteks ${esc(m.contextLength)} token`:''}.</li>`).join('')||'<li>Tidak ada model aktif yang dilaporkan.</li>';
  const devices=r.nvidia?.devices||[];
  const deviceHtml=devices.length?devices.map(g=>`<li>${esc(g.name)}; driver ${esc(g.driver)}; GPU ${g.utilizationPercent===null?'tidak diketahui':esc(g.utilizationPercent+'%')}; VRAM ${g.memoryUsedMiB===null?'?':esc(g.memoryUsedMiB+' MiB')} / ${g.memoryTotalMiB===null?'?':esc(g.memoryTotalMiB+' MiB')}; suhu ${g.temperatureC===null?'?':esc(g.temperatureC+'°C')}.</li>`).join(''):`<li>${esc(r.nvidia?.message||'Telemetri NVIDIA tidak tersedia.')}</li>`;
  const labels={GPU:'GPU aktif: model dimuat hampir seluruhnya di VRAM.', 'CPU + GPU':'GPU aktif sebagian: model memakai VRAM dan RAM/CPU.', CPU:'Model terdeteksi berjalan di CPU, bukan VRAM.', idle:'Tes selesai, tetapi tidak ada model aktif.', unknown:'Status GPU belum dapat dipastikan.'};
  const message=labels[r.processor]||r.message||'Status tidak diketahui.';
  const html=`<p><strong>${esc(message)}</strong></p><p>Versi Ollama Notely: ${esc(r.version||'tidak diketahui')}</p><h3>Penempatan model menurut Ollama</h3><ul>${models}</ul><h3>Telemetri NVIDIA</h3><ul>${deviceHtml}</ul><p class="field-help">Jika model tercatat CPU, periksa driver NVIDIA dan buka file log untuk melihat alasan backend CUDA gagal dimuat. Grafik Task Manager harus menggunakan CUDA/Compute, bukan hanya 3D.</p><p class="field-help">Log: ${esc(r.logPath||'tidak tersedia')}</p>`;
  modal('Hasil tes GPU Ollama',html,[['Tutup','close-modal','']]);
 }catch(e){modal('Tes GPU Ollama gagal',`<p>${esc(e.message||e)}</p><p>Pastikan mesin Ollama terpasang, model sudah diunduh, dan driver NVIDIA kompatibel.</p>`,[['Tutup','close-modal','']]);}
 finally{ollamaGpuTestRunning=false;}
}
const prototypeClick=handleClick;
handleClick=function(action,id){
 if(action==='ask-demo'||action==='start-demo')return startRecording();
 if(action==='stop-demo')return stopRecording();
 if(action==='finish-partial')return savePartial();
 if(action==='generate-summary')return generateRealSummary();
 if(action==='reprocess-audio')return reprocessAudio();
 if(action==='choose-storage')return chooseModelStorage().catch(alertError);
 if(action==='test-provider')return testProvider();
 if(action==='save-ai')return saveApiKey().then(()=>{persistSettings();toast('API key dan pengaturan disimpan di komputer ini.');}).catch(alertError);
 if(action==='delete-local')return modal('Hapus model lokal?',esc(id)+' akan dihapus dari penyimpanan Notely.',['Batal','Hapus'].map((v,i)=>[v,i?'confirm-delete-local':'close-modal',i?'accent':''])),state.deleteModel=id;
 if(action==='confirm-delete-local'){closeModal();return api('/ollama/delete',{model:state.deleteModel}).then(()=>loadLocalCatalog()).then(()=>toast('Model dihapus.')).catch(alertError);}
 if(action==='ai-connection')return refreshProviderModels();
 if(action==='download-ollama-model')return downloadOllama();
 if(action==='ollama-gpu-test')return testOllamaGpu();
 if(action==='confirm-real-summary')return confirmRealSummary();
 if(action==='open-note')return loadSession(id).catch(alertError);
 if(action==='import-audio')return showAudioImport();
 if(action==='desktop-only')return native()('open_storage').catch(alertError);
 if(action==='recording-info')return native()('open_storage').catch(alertError);
 if(action==='choose-model'){state.settings.model=id;persistSettings();engine.modelReady=false;return showWizard();}
 if(action==='about')return modal('Tentang Notely Desktop','Notely adalah aplikasi transkripsi dan ringkasan rekaman lokal. Pilih model saat penyiapan awal. Pastikan Anda memiliki izin perekaman dari pihak yang relevan.',[['Tutup','close-modal','']]);
 if(action==='quick-menu')return showWizard();
 return prototypeClick(action,id);
};
/* Overwrite prototype setting text, not user data. */
let oldRender=render;
render=function(){oldRender();applyDeviceDropdown();};
function applyDeviceDropdown(){
 const list=document.querySelector('select[data-setting="device"]');if(list&&engine.speakers?.length){
  list.innerHTML=engine.speakers.map(x=>`<option value="${x.index}">${esc(x.name)}</option>`).join('');
  list.value=String(engine.selectedSpeaker?.index??engine.speakers[0].index);
 }
 const mic=document.querySelector('select[data-setting="mic"]');if(mic&&engine.microphones?.length){mic.innerHTML='<option value="none">Tidak merekam mikrofon</option>'+engine.microphones.map(x=>`<option value="${x.index}">${esc(x.name)}</option>`).join('');mic.value=engine.selectedMic===null?'none':String(engine.selectedMic);}
}
document.addEventListener('change',e=>{
 if(e.target.dataset.setting==='device'&&engine.speakers){engine.selectedSpeaker=engine.speakers.find(x=>x.index===Number(e.target.value))||engine.selectedSpeaker;}
 if(e.target.dataset.setting==='mic'){engine.selectedMic=e.target.value==='none'?null:Number(e.target.value);}
});
/* Override previous preview-only audio input action before it executes. */
document.getElementById('audio-file').replaceWith(document.getElementById('audio-file').cloneNode(true));
function showWizard(){
 if(document.getElementById('setup-wizard'))return;
 if(engine.deviceChecked&&!engine.hasNvidia&&state.settings.model==='maleo-turbo')state.settings.model='maleo-small';
 const root=document.createElement('div');root.id='setup-wizard';root.innerHTML=`<div class="wizard-card" role="dialog" aria-modal="true" aria-label="Persiapan awal Notely">
 <div id="wizard-welcome" class="wizard-stage"><img class="wizard-logo" src="assets/notely.svg" alt="Notely"><span class="wizard-overline">NOTELY / SELAMAT DATANG</span><h2>Semua hal penting, dalam satu catatan.</h2><p>Rekam, transkripsikan, dan buat ringkasan dengan AI pilihan Anda.</p>
 <div class="wizard-benefits"><div>◈ &nbsp; Rekaman dan transkrip tersimpan di komputer Anda</div><div>◈ &nbsp; Model bahasa Indonesia yang bisa dipakai offline</div><div>◈ &nbsp; Unduh hanya komponen yang diperlukan</div></div>
 <button class="button accent wizard-primary" id="wizard-start" ${engine.deviceChecked?'':'disabled'}>${engine.deviceChecked?'Mulai persiapan':'Memeriksa perangkat…'} <span aria-hidden="true">→</span></button></div>
 <div id="wizard-settings" class="wizard-stage" hidden><img class="wizard-logo" src="assets/notely.svg" alt="Notely"><span class="wizard-overline">LANGKAH 1 DARI 2 / PILIH MODEL</span><h2>Sesuaikan Notely</h2><p>Komponen akan diunduh sekali. Notely akan menggunakan kembali yang sudah tersedia.</p>
 <div class="wizard-model-grid"><div class="wizard-field-group"><label for="wizard-model">Model untuk transkripsi</label><select id="wizard-model">${models.map(m=>`<option value="${m.id}" ${m.id===state.settings.model?'selected':''}>${esc(m.name)} · ${esc(m.size)}</option>`).join('')}</select>
 <p class="wizard-note" id="wizard-gpu-note">${esc(window.__notelyGpuStatus||'Mendeteksi GPU NVIDIA secara otomatis…')}</p>
 <label class="wizard-gpu-choice"><input id="wizard-use-cuda" type="checkbox" ${engine.hasNvidia?'checked':'disabled'}> Gunakan percepatan NVIDIA CUDA (jika tersedia)</label><p class="wizard-note">Komponen CUDA membutuhkan unduhan dan ruang penyimpanan tambahan. Jika tidak tersedia, mode CPU tetap bisa dipilih.</p>
 </div><div class="wizard-field-group"><label for="wizard-provider">Ringkasan AI</label><select id="wizard-provider">${Object.entries(providers).map(([id,p])=>`<option value="${id}" ${id===state.settings.provider?'selected':''}>${esc(p.name)} — ${id==='ollama'?'Lokal':'Online (API key)'}</option>`).join('')}</select>
 <label>Model untuk ringkasan</label><select id="wizard-summary-model"></select>
 <div id="wizard-key-area"><label>API key (bisa diisi nanti)</label><input id="wizard-api-key" class="control-select" type="password" placeholder="Kunci pribadi penyedia AI" autocomplete="off"></div>
 <p class="wizard-note">AI online memerlukan API key.</p></div></div>
 
 <div class="wizard-actions"><button class="button accent" id="wizard-install">Unduh & siapkan model</button></div></div>
 <div id="wizard-download" class="wizard-stage" hidden><img class="wizard-logo" src="assets/notely.svg" alt="Notely"><span class="wizard-overline">LANGKAH 2 DARI 2 / PERSIAPAN KOMPONEN</span><h2>Menyiapkan Notely</h2><p>Unduhan diproses secara lokal. Jangan tutup aplikasi sampai persiapan selesai.</p>
 <div class="wizard-download-item"><strong>Mesin transkripsi</strong><span>Maleo / Whisper</span></div>
 <div class="wizard-download-item"><strong>Percepatan NVIDIA</strong><span>Hanya saat diperlukan</span></div>
 <div class="wizard-progress-track"><div id="wizard-progress-bar"></div></div>
 <div id="wizard-status" class="wizard-status" role="status" aria-live="polite">Menunggu persiapan…</div>
 <div class="wizard-actions"><button class="button" id="wizard-back" type="button">Kembali</button><button class="button accent" id="wizard-finish" disabled>Selesai & buka Notely</button></div></div></div>`;
 document.body.appendChild(root);
 if(typeof loadTranscriptionSizes==='function')loadTranscriptionSizes();
 const stages=['wizard-welcome','wizard-settings','wizard-download'];
 const stage=(name)=>stages.forEach(x=>root.querySelector('#'+x).hidden=(x!==name));
 root.querySelector('#wizard-start').addEventListener('click',()=>stage('wizard-settings'));
 root.querySelector('#wizard-back').addEventListener('click',()=>{
   if(engine.currentJob||engine.cudaJob)return;
   root.querySelector('#wizard-install').disabled=false;stage('wizard-settings');
 });
 root.querySelector('#wizard-model').addEventListener('change',e=>{state.settings.model=e.target.value;persistSettings();engine.modelReady=false;});
 function renderWizardProvider(){
  const id=state.settings.provider;
  state.apiKey=state.savedKeys?.[id]||'';root.querySelector('#wizard-api-key').value=state.apiKey;
  root.querySelector('#wizard-summary-model').innerHTML=providers[id].options.map(([key,label])=>`<option value="${esc(key)}" ${key===state.settings.summaryModel?'selected':''}>${esc(label)}</option>`).join('');
  root.querySelector('#wizard-key-area').hidden=id==='ollama';
 }
 root.querySelector('#wizard-provider').addEventListener('change',e=>{state.settings.provider=e.target.value;state.settings.summaryModel=providers[e.target.value].options[0][0];persistSettings();renderWizardProvider();});
 root.querySelector('#wizard-summary-model').addEventListener('change',e=>{state.settings.summaryModel=e.target.value;persistSettings();});
 root.querySelector('#wizard-api-key').value=state.apiKey;
 root.querySelector('#wizard-api-key').addEventListener('input',e=>{state.apiKey=e.target.value;});
 renderWizardProvider();
 root.querySelector('#wizard-install').addEventListener('click',async ()=>{
  engine.preferCuda=Boolean(root.querySelector('#wizard-use-cuda')?.checked)&&engine.hasNvidia;
  if(!engine.preferCuda&&state.settings.model==='maleo-turbo'){
    state.settings.model='maleo-small';root.querySelector('#wizard-model').value='maleo-small';persistSettings();
    root.querySelector('#wizard-status').textContent='CPU dipilih. Model dialihkan ke Maleo Indonesian Small untuk performa lebih sesuai.';
  }
  if(!await ensureModelStorage())return;
  await saveApiKey();
  const b=root.querySelector('#wizard-install');b.disabled=true;stage('wizard-download');
  root.querySelector('#wizard-status').textContent='Menyiapkan model. Jangan tutup aplikasi hingga selesai…';
  try{
   if(engine.preferCuda&&!engine.cudaDownloaded){
     const job=await api('/cuda/prepare',{});engine.cudaJob=job.job;
     root.querySelector('#wizard-status').textContent='Mengunduh pustaka resmi NVIDIA dan memverifikasi checksum…';
   }else{
     const response=await api('/model/prepare',{model:choiceIds[state.settings.model],gpu:engine.preferCuda?'Otomatis':'CPU'});
     engine.currentJob=response.job;
   }
  }
  catch(e){root.querySelector('#wizard-status').textContent='Gagal: '+e.message;b.disabled=false;}
 });
 root.querySelector('#wizard-finish').addEventListener('click',()=>{
  if(!engine.modelReady)return;
  safeStore.set('notelySetupComplete',true);root.remove();state.page='home';render();toast('Notely siap. Pilih Mulai Rekam untuk memulai.');
 });
}
if(native()){startup().catch(alertError);}else{window.__notelyStatus='Pratinjau desain (tanpa mesin audio)';render();}

async function saveApiKey(){
 if(state.settings.provider==='ollama')return;
 await api('/credentials/save',{provider:state.settings.provider,apiKey:state.apiKey});
 state.savedKeys={...(state.savedKeys||{}),[state.settings.provider]:state.apiKey};
}
async function chooseModelStorage(){
 if(!native())throw Error('Pemilihan folder tersedia dalam aplikasi desktop.');
 const folder=await native()('choose_model_folder');if(!folder)return false;
 const data=await api('/storage/select',{folder});state.storageMessage='Memindahkan model…';refreshPage();return true;
}
async function ensureModelStorage(){
 const data=await api('/preferences',{});state.modelStorage=data.modelRoot||'';
 if(data.storageError)throw Error(data.storageError);
 if(data.storageReady===false){toast('Model sedang dipindahkan/disiapkan otomatis. Lihat progres di Pengaturan.');return false;}
 if(!state.modelStorage)throw Error('Folder model belum siap. Periksa Penyimpanan model di Pengaturan.');
 return true;
}
async function testProvider(){
 const model=state.settings.summaryModel==='custom'?state.settings.customModel:state.settings.summaryModel;
 toast('Menguji respons model dengan pesan singkat…');
 try{await saveApiKey();const r=await api('/provider/test',{provider:state.settings.provider,model,apiKey:state.apiKey,endpoint:state.settings.endpoint||''});toast(r.message);}catch(e){modal('Tes AI belum berhasil',esc(e.message),[['Tutup','close-modal','']]);}
}

const reportedStorageJobs=new Set();
function updateStorageProgress(jobs){
 for(const [id,job] of Object.entries(jobs)){
  if(job.kind!=='storage')continue;
  if(job.status==='running')state.storageMessage=job.message;
  else if(!reportedStorageJobs.has(id)){
   reportedStorageJobs.add(id);
   if(job.status==='done'){
    state.modelStorage=job.result?.modelRoot||state.modelStorage;
    state.storageMessage=job.result?.warning||'Model siap di '+state.modelStorage;
   }else{state.storageMessage=job.message;alertError(job.message);}
   if(state.page==='settings')refreshPage();
  }
 }
 const el=document.getElementById('storage-progress');if(el)el.textContent=state.storageMessage||'';
}
document.addEventListener('input',e=>{if(e.target.id==='summary-instructions')getMeeting().instructions=e.target.value;});

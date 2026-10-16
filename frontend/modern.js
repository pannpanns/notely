/* Focused desktop views. All download metrics originate in the engine. */
'use strict';
state.noteTab='transcript';
const localCatalog={loading:false,loaded:false,connected:false,message:'Periksa model yang tersedia.',models:[
 {id:'qwen3.5:4b',name:'Qwen 3.5',tier:'4B',sizeBytes:3400000000,sizeSource:'estimate',description:'Pilihan utama untuk materi dan rekaman.'},
 {id:'qwen3.5:2b',name:'Qwen 3.5 (2B)',tier:'2B',sizeBytes:2700000000,sizeSource:'estimate',description:'Lebih ringan untuk memori terbatas.'},
 {id:'qwen2.5:3b',name:'Qwen 2.5',tier:'3B',sizeBytes:1900000000,sizeSource:'estimate',description:'Alternatif untuk instalasi sebelumnya.'},
 {id:'gemma3:4b',name:'Gemma 3',tier:'4B',sizeBytes:3300000000,sizeSource:'estimate',description:'Alternatif model lokal dari Google.'},
 {id:'qwen2.5:1.5b',name:'Qwen 2.5 (1.5B)',tier:'1.5B',sizeBytes:1000000000,sizeSource:'estimate',description:'Untuk ringkasan sederhana.'}
]};
const transfers=new Map();
const transferExpiry=new Map();
const dismissedTransfers=new Set();
let downloadsOpen=true;
function bytes(n){if(!Number.isFinite(n)||n<=0)return '0 B';const i=Math.min(3,Math.floor(Math.log(n)/Math.log(1000)));return (n/1000**i).toLocaleString('id-ID',{maximumFractionDigits:i?2:0})+' '+['B','KB','MB','GB'][i];}
function sizeLabel(m){return m.sizeBytes?`${m.sizeSource==='estimate'?'≈ ':''}${bytes(m.sizeBytes)}`:'Ukuran belum tersedia';}
async function loadLocalCatalog(){
 if(localCatalog.loading)return;
 localCatalog.loading=true;
 if(state.page==='settings'&&state.tab==='summary')refreshPage();
 try{const data=await api('/ollama/catalog',{});Object.assign(localCatalog,data,{loaded:true});
 providers.ollama.options=data.models.map(m=>[m.id,`${m.name} · ${sizeLabel(m)}`]);
 }catch(e){localCatalog.message=String(e.message);localCatalog.connected=false;}
 finally{localCatalog.loading=false;if(state.page==='settings'&&state.tab==='summary')refreshPage();}
}
function beginDownload(id,model){transfers.set(id,{kind:'ollama',status:'running',message:'Menyiapkan unduhan',progress:{model}});downloadsOpen=true;renderDownloads();if(state.page==='settings'&&state.tab==='summary')refreshPage();}
function updateDownloadCenter(jobs){
 for(const [id,j] of Object.entries(jobs)){
  if(!['ollama','model','cuda'].includes(j.kind)||dismissedTransfers.has(id))continue;
  if(j.status!=='running'&&transfers.has(id)&&!transferExpiry.has(id)){
   transferExpiry.set(id,setTimeout(()=>{transfers.delete(id);dismissedTransfers.add(id);transferExpiry.delete(id);renderDownloads();},j.status==='error'?12000:5000));
  }
  if(j.status==='running'||transfers.has(id))transfers.set(id,j);
 }
 renderDownloads();
}
function renderDownloads(){
 let el=document.getElementById('download-center');
 if(!el){el=document.createElement('aside');el.id='download-center';el.setAttribute('aria-label','Aktivitas unduhan');document.body.appendChild(el);}
 const entries=[...transfers.entries()];if(!entries.length){el.hidden=true;return;}el.hidden=false;
 const active=entries.filter(([,j])=>j.status==='running').length;
 el.innerHTML=`<button class="download-heading" data-action="toggle-downloads" aria-expanded="${downloadsOpen}">${icon('download')}<strong>${active?`${active} proses berjalan`:'Aktivitas selesai'}</strong>${icon('chevron')}</button>${downloadsOpen?`<div class="download-items">${entries.map(([id,j])=>{
 const p=j.progress||{};const fresh=Date.now()/1000-(p.sampledAt||0)<5;const speed=fresh?p.bytesPerSecond:0;
 return `<div class="transfer"><div class="space-between"><strong>${esc(p.model||({model:'Model transkripsi',cuda:'Komponen NVIDIA',ollama:'AI lokal'}[j.kind]))}</strong>${j.status!=='running'?`<button class="dismiss" data-action="dismiss-transfer" data-id="${id}" aria-label="Tutup aktivitas">×</button>`:''}</div><span class="transfer-status ${j.status==='error'?'error-text':''}">${esc(j.status==='done'?'Siap digunakan':j.message)}</span>${j.status==='running'?`<progress ${Number.isFinite(p.percent)?`value="${p.percent}"`:''} max="100" aria-label="Progres unduhan"></progress><div class="transfer-metrics"><span>${p.totalBytes?`${bytes(p.downloadedBytes||0)} / ${bytes(p.totalBytes)}`:'Menyiapkan komponen…'}</span><strong>${Number.isFinite(p.percent)?Math.floor(p.percent)+'%':''}</strong></div>${p.totalBytes?`<div class="transfer-metrics"><span>${speed?bytes(speed)+'/s':'Menunggu data kecepatan'}</span><span>${speed&&p.etaSeconds?`≈ ${Math.ceil(p.etaSeconds/60)} menit`:''}</span></div>`:''}`:''}${j.status==='error'&&j.kind==='ollama'?`<button class="button small" data-action="retry-transfer" data-id="${esc(p.model||'')}">Coba lagi</button>`:''}</div>`;
 }).join('')}</div>`:''}`;
}
const baseHomePage=homePage;
homePage=function(){return `<div class="calm-home"><div class="home-greeting"><span class="eyebrow">RUANG CATATAN ANDA</span><h1>${state.recording?'Fokus pada percakapan.':'Ide penting, tersimpan rapi.'}</h1><p>${state.recording?'Notely sedang mencatat. Anda bisa membuka transkrip langsung.':'Mulai rekaman baru atau lanjutkan catatan terakhir Anda.'}</p></div><div class="home-choices"><button class="home-choice featured" data-action="${state.recording?'open-note':'ask-demo'}" data-id="${esc(state.selected||'')}"><span class="choice-icon">${icon('mic')}</span><strong>${state.recording?'Buka rekaman':'Rekam percakapan'}</strong><span>Audio komputer dan mikrofon</span><b aria-hidden="true">↗</b></button><button class="home-choice" data-action="import-audio"><span class="choice-icon">${icon('upload')}</span><strong>Impor rekaman</strong><span>Ubah file audio menjadi catatan</span><b aria-hidden="true">↗</b></button></div><div class="recent-heading"><h2>Catatan terakhir</h2><button class="button ghost small" data-action="page" data-id="recordings">Lihat semua ${icon('chevron')}</button></div><div class="recent-notes">${meetings.length?meetings.slice(-3).reverse().map(m=>`<button class="recent-note" data-action="open-note" data-id="${esc(m.id)}"><span class="recent-icon">${icon('file')}</span><span><strong>${esc(m.title)}</strong><small>${esc(m.when)}</small></span>${icon('chevron')}</button>`).join(''):`<div class="quiet-empty">${icon('notes')}<p>Belum ada catatan. Mulai dengan rekaman pertama Anda.</p></div>`}</div><div class="privacy-foot">${icon('shield')} Transkripsi lokal · Catatan tersimpan di komputer</div></div>`;};
summaryTab=function(){const local=state.settings.provider==='ollama';const p=providers[state.settings.provider]||providers.ollama;return `<div class="section-intro"><span class="eyebrow">MODEL & PRIVASI</span><h2>Ringkasan AI</h2><p>Pilih tempat catatan Anda diproses.</p></div><div class="ai-modes"><button class="ai-mode ${local?'active':''}" data-action="choose-provider" data-id="ollama">${icon('cpu')}<span><strong>AI lokal</strong><small>Di komputer Anda, tanpa API key</small></span><span class="tag">Utama</span></button><button class="ai-mode ${!local?'active':''}" data-action="choose-provider" data-id="gemini">${icon('globe')}<span><strong>Layanan API</strong><small>Hubungkan penyedia pilihan Anda</small></span></button></div>${local?`
 <div class="setting-card"><strong>Penyimpanan model</strong><p id="storage-progress" class="field-help">${esc(state.storageMessage||'')}</p><p class="field-help">${esc(state.modelStorage||'Folder data Notely')}</p><button class="button" data-action="choose-storage">Pilih folder</button></div>
 <div class="catalog-header"><div><h3>Model lokal</h3><p>${localCatalog.loading?'Memeriksa model dan ukuran…':localCatalog.connected?'Siap digunakan':esc(localCatalog.message)}</p></div><button class="button small" data-action="refresh-local" ${localCatalog.loading?'disabled':''}>${icon('refresh')} Periksa</button></div>
 <div class="local-model-grid">${localCatalog.models.map(localModelCard).join('')}</div>
`:`
 <div class="setting-card cloud-settings"><div class="field"><label class="field-label" for="cloud-provider">Penyedia API</label><select id="cloud-provider" class="control-select" data-change="cloud-provider">${Object.entries(providers).filter(([id])=>id!=='ollama').map(([id,v])=>`<option value="${id}" ${id===state.settings.provider?'selected':''}>${esc(v.name)}</option>`).join('')}</select></div>
 ${providerKeyGuide(state.settings.provider)}<div class="field"><label class="field-label" for="cloud-model">Model ${esc(p.name)}</label><select id="cloud-model" class="control-select" data-change="summary-model">${p.options.map(([id,label])=>`<option value="${esc(id)}" ${id===state.settings.summaryModel?'selected':''}>${esc(label)}</option>`).join('')}</select></div>
 ${state.settings.summaryModel==='custom'?`<div class="field"><label class="field-label" for="custom-model">ID model</label><input id="custom-model" class="textbox" value="${esc(state.settings.customModel||'')}" placeholder="penyedia/model"></div>`:''}
 ${state.settings.provider==='custom'?`<div class="field"><label class="field-label" for="custom-endpoint">Endpoint</label><input id="custom-endpoint" class="textbox" value="${esc(state.settings.endpoint||'')}" placeholder="https://server.example/v1"></div>`:''}
 ${state.settings.provider==='gemini'?`<label class="field-help"><input type="checkbox" data-setting="geminiFallback" ${state.settings.geminiFallback!==false?'checked':''}> Jika model sibuk, coba hingga 2 model Gemini Flash lain yang tersedia. Penggunaan mengikuti kuota/tarif akun.</label>`:''}<div class="field"><label class="field-label" for="api-key">API key</label><div class="keyinput"><input id="api-key" class="textbox" type="${state.showKey?'text':'password'}" autocomplete="off" value="${esc(state.apiKey)}" placeholder="Masukkan API key"><button data-action="toggle-key" aria-label="Tampilkan atau sembunyikan key">${icon(state.showKey?'eyeoff':'eye')}</button></div><span class="field-help">Key disimpan terenkripsi pada akun Windows ini. Tidak dikirim ke server pembuat Notely.</span></div><div class="space-between"><button class="button" data-action="ai-connection">${icon('refresh')} Muat model</button><button class="button" data-action="test-provider">Tes respons</button><button class="button accent" data-action="save-ai">${icon('check')} Simpan</button></div></div>`}`;};
function localModelCard(m){const chosen=state.settings.summaryModel===m.id;const downloading=[...transfers.values()].some(j=>j.status==='running'&&j.progress?.model===m.id);return `<article class="local-model ${chosen?'chosen':''}"><div class="space-between"><span class="model-glyph">${icon('cpu')}</span><span class="tag ${m.installed?'':'neutral'}">${m.installed?'Terpasang':downloading?'Mengunduh':'Belum diunduh'}</span></div><h3>${esc(m.name)}</h3><p>${esc(m.description)}</p><div class="model-facts"><div><small>UKURAN MODEL</small><strong>${sizeLabel(m)}</strong></div><div><small>PARAMETER</small><strong>${esc(m.tier)}</strong></div></div><div class="local-model-actions"><button class="button ${chosen?'selected':'accent'}" data-action="select-local" data-id="${esc(m.id)}">${chosen?icon('check')+' Dipilih':'Pilih model'}</button>${m.installed?`<button class="button" data-action="delete-local" data-id="${esc(m.id)}">Hapus model</button>`:''}${!m.installed?`<button class="button" data-action="pull-local" data-id="${esc(m.id)}" ${engine.ollamaJob?'disabled':''}>${icon('download')} ${downloading?'Mengunduh':'Unduh'}</button>`:''}</div></article>`;}
notesPage=function(){const m=getMeeting();const summaryHere=state.selected===engine.summarySession;return `<div class="notes-page focused-notes tabbed-notes ${state.noteTab==='summary'?'summary-view':''}">${state.recording?`<div class="notes-preview-banner"><span class="pulse"></span> Mendengarkan <span id="demo-timer">${ts(state.recordSeconds)}</span></div>`:''}<div class="focus-toolbar"><div class="segmented" role="tablist" aria-label="Tampilan catatan"><button id="tab-transcript" role="tab" aria-controls="panel-transcript" aria-selected="${state.noteTab!=='summary'}" tabindex="${state.noteTab!=='summary'?0:-1}" class="${state.noteTab!=='summary'?'active':''}" data-action="note-tab" data-id="transcript">Transkrip</button><button id="tab-summary" role="tab" aria-controls="panel-summary" aria-selected="${state.noteTab==='summary'}" tabindex="${state.noteTab==='summary'?0:-1}" class="${state.noteTab==='summary'?'active':''}" data-action="note-tab" data-id="summary">Ringkasan</button></div><button class="button small" data-action="recording-info">${icon('headphones')} Audio</button></div><div class="notes-content"><section id="panel-transcript" role="tabpanel" aria-labelledby="tab-transcript" class="transcript-pane" ${state.noteTab==='summary'?'hidden':''}><div class="pane-head"><h2>Transkrip</h2><div class="toolbar-group"><button class="button small" data-action="copy-transcript">${icon('copy')} Salin</button>${m.id!=='empty'&&!state.recording?`<button class="button small" data-action="edit-transcript">${icon('notes')} Edit</button>`:''}</div></div><div class="transcript-list" id="transcript-list">${transcriptList(m)}</div></section><section id="panel-summary" role="tabpanel" aria-labelledby="tab-summary" class="summary-pane" ${state.noteTab!=='summary'?'hidden':''}><div class="summary-model-bar"><span>${esc(providers[state.settings.provider]?.name||'AI Lokal')} · ${esc(state.settings.summaryModel)}</span><button class="button small" data-action="copy-summary" ${m.summary?'':'disabled'}>${icon('copy')} Salin</button><button class="button small ghost" data-action="go-summary" aria-label="Pilih model AI" title="Pilih model AI">${icon('settings')}</button><button class="button accent small" data-action="generate-summary" ${state.summaryLoading||state.recording?'disabled':''}>${icon('spark')} ${state.summaryLoading?'Meringkas…':m.summary?'Buat ulang':'Ringkas'}</button></div>${state.summaryError?`<div class="summary-error" role="alert"><strong>Ringkasan belum berhasil</strong><p>${esc(state.summaryError)}</p><button class="button small" data-action="generate-summary">Coba lagi</button></div>`:''}<div class="summary-scroll" id="summary-scroll">${state.summaryLoading&&summaryHere?`<div class="summary-empty"><span class="working-orbit">${icon('spark')}</span><h3>Sedang merangkum catatan</h3><p id="summary-progress" role="status" aria-live="polite">Membaca transkrip…</p></div><div id="summary-live" class="summary-paper real-summary" aria-label="Pratinjau ringkasan"></div>`:summaryPaper(m)}</div><div class="context-disclosure prompt-panel"><label class="field-label prompt-label" for="summary-instructions">Prompt AI</label><div class="prompt-composer"><textarea id="summary-instructions" maxlength="4000" placeholder="Tulis instruksi untuk AI">${esc(m.instructions||'')}</textarea><button class="prompt-send" data-action="generate-summary" aria-label="Kirim prompt AI" title="Kirim prompt AI" ${state.summaryLoading||state.recording?'disabled':''}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="m21 3-7 18-4-7-7-4 18-7Z" stroke-linejoin="round"/><path d="m10 14 11-11"/></svg></button></div></div></section></div></div>`;};
const previousClick=handleClick;
handleClick=function(action,id){
 if(action==='edit-transcript'){
  const m=getMeeting();modal('Edit transkrip','<textarea id="transcript-editor" class="textbox" style="width:100%;min-height:240px">'+esc(transcriptText(m))+'</textarea>',[['Batal','close-modal',''],['Simpan','save-transcript','accent']]);return;
 }
 if(action==='save-transcript'){
  const text=document.getElementById('transcript-editor')?.value||'';
  return api('/session/edit',{id:getMeeting().id,text}).then(()=>{closeModal();return loadSession(state.selected);}).then(()=>toast('Transkrip disimpan.')).catch(alertError);
 }
 if(action==='note-tab'){return selectNoteTab(id);}
 if(action==='refresh-local')return loadLocalCatalog();
 if(action==='toggle-downloads'){downloadsOpen=!downloadsOpen;return renderDownloads();}
 if(action==='dismiss-transfer'){transfers.delete(id);dismissedTransfers.add(id);clearTimeout(transferExpiry.get(id));transferExpiry.delete(id);return renderDownloads();}
 if(['select-local','pull-local','retry-transfer'].includes(action)){
  id=id||state.settings.summaryModel;
  state.settings.provider='ollama';state.settings.summaryModel=id;persistSettings();refreshPage();
  if(action!=='select-local')return downloadOllama();return;
 }
 if(action==='generate-summary')state.noteTab='summary';
 const result=previousClick(action,id);
 if((action==='tab'&&id==='summary')||action==='go-summary'||(action==='choose-provider'&&id==='ollama')){
  if(!localCatalog.loaded&&!localCatalog.loading)loadLocalCatalog();
 }
 return result;
};
document.addEventListener('change',e=>{if(e.target.dataset.change==='cloud-provider')handleClick('choose-provider',e.target.value);});
document.addEventListener('input',e=>{if(e.target.id==='custom-endpoint'){state.settings.endpoint=e.target.value;persistSettings();}});
render();

let transcriptionCatalogLoaded=false;
async function loadTranscriptionSizes(){
 if(!engine.ready||transcriptionCatalogLoaded)return;
 transcriptionCatalogLoaded=true;
 try{const data=await api('/transcription/catalog',{});
  for(const m of models){const info=data.models.find(x=>x.name===choiceIds[m.id]);if(info?.sizeBytes)m.size=bytes(info.sizeBytes);}
  const select=document.getElementById('wizard-model');
  if(select)for(const option of select.options){const m=models.find(m=>m.id===option.value);if(m)option.textContent=m.name+' · '+m.size;}
  if(state.page==='settings'&&state.tab==='transcription')refreshPage();
 }catch{transcriptionCatalogLoaded=false;}
}
// Offline estimates are shown immediately; live metadata replaces them when available.
const beforeTranscriptionTab=transcriptionTab;
transcriptionTab=function(){loadTranscriptionSizes();return beforeTranscriptionTab();};

// Switch visibility in place: preserve scroll, prompt drafts and live generation.
function selectNoteTab(id){
 if(!['transcript','summary'].includes(id))return;
 if(state.noteTab!==id)clearInstructionDrafts();
 state.noteTab=id;
 document.querySelector('.tabbed-notes')?.classList.toggle('summary-view',id==='summary');
 if(typeof applyDockScale==='function')applyDockScale();
 for(const name of ['transcript','summary']){
  const tab=document.getElementById('tab-'+name),panel=document.getElementById('panel-'+name),active=name===id;
  if(tab){tab.classList.toggle('active',active);tab.setAttribute('aria-selected',String(active));tab.tabIndex=active?0:-1;}
  if(panel)panel.hidden=!active;
 }
}
document.addEventListener('keydown',e=>{
 if(!e.target.matches('.tabbed-notes [role="tab"]'))return;
 if(!['ArrowLeft','ArrowRight','Home','End'].includes(e.key))return;
 e.preventDefault();const id=e.key==='Home'?'transcript':e.key==='End'?'summary':state.noteTab==='summary'?'transcript':'summary';
 selectNoteTab(id);document.getElementById('tab-'+id)?.focus();
});

// Keep the prompt disclosure state across page refreshes.
document.addEventListener("toggle",e=>{if(e.target.matches?.(".context-disclosure"))state.promptOpen=e.target.open;},true);

// Instruction drafts belong only to the current view, never to saved summaries.
function clearInstructionDrafts(){
 for(const m of meetings)m.instructions='';
 const input=document.getElementById('summary-instructions');if(input)input.value='';
 state.promptOpen=false;
}
let instructionView=state.page+':'+state.selected;
function syncInstructionView(){
 const next=state.page+':'+state.selected;
 if(next!==instructionView){clearInstructionDrafts();instructionView=next;}
}
const renderBeforeDraftReset=render;
render=function(){syncInstructionView();renderBeforeDraftReset();};
const refreshBeforeDraftReset=refreshPage;
refreshPage=function(){syncInstructionView();refreshBeforeDraftReset();};
document.addEventListener('keydown',e=>{
 if(document.querySelector('#modal-root .modal'))return;
 if(e.target.id!=='summary-instructions'||e.key!=='Enter'||e.shiftKey||e.isComposing)return;
 e.preventDefault();
 if(e.repeat||state.summaryLoading||state.summarySubmitting||state.recording)return;
 getMeeting().instructions=e.target.value;
 handleClick('generate-summary');
});

// Keep keyboard events inside the consent dialog, including held Enter keys.
document.addEventListener('keydown',e=>{
 const confirm=document.querySelector('#modal-root [data-action="confirm-real-summary"]');
 if(!confirm)return;
 const dialog=confirm.closest('.modal');
 if(e.key==='Enter'&&!e.isComposing){
  e.preventDefault();e.stopImmediatePropagation();
  if(!e.repeat){
   if(e.target.closest?.('[data-action="close-modal"]'))closeModal();
   else confirmRealSummary();
  }
 }else if(e.key==='Tab'){
  const buttons=[...dialog.querySelectorAll('button')];
  const i=buttons.indexOf(document.activeElement);
  e.preventDefault();buttons[(i+(e.shiftKey?-1:1)+buttons.length)%buttons.length].focus();
 }
},true);
const summaryBeforeEconomy=summaryTab;
summaryTab=function(){if(state.settings.provider==='ollama')return summaryBeforeEconomy();return summaryBeforeEconomy()+`<section class="setting-card token-settings" aria-labelledby="token-settings-title"><h3 id="token-settings-title">Penggunaan token</h3><div class="token-option"><label class="token-choice" for="summary-economy"><input id="summary-economy" type="checkbox" data-setting="summaryEconomy" aria-describedby="token-help token-note" ${state.settings.summaryEconomy===true?'checked':''}><span>Ringkasan hemat <span class="token-target">(target 120–250 kata)</span></span></label><div class="token-description"><p id="token-help">Meminta jawaban dan catatan per bagian yang lebih pendek. Seluruh transkrip tetap diproses; instruksi panjang khusus Anda tetap diutamakan.</p><p id="token-note">Target ini bukan batas token keras atau jaminan biaya. Permintaan diproses berurutan; pemrosesan paralel tidak otomatis menghemat token.</p></div></div></section>`;};

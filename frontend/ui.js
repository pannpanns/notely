
'use strict';
/* Notely browser UI prototype. No microphone, system-audio capture, external API,
   model download, or backend operations occur in this file. */
const paths={
  home:'<path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1z"/>',
  notes:'<path d="M8 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9l-6-6H8z"/><path d="M14 3v7h7M7 14h10M7 18h7"/>',
  settings:'<path d="M12 2l1.7 2.4 2.9-.2.8 2.7 2.8 1-.6 2.9L22 12l-2.4 1.2.6 2.9-2.8 1-.8 2.7-2.9-.2L12 22l-1.7-2.4-2.9.2-.8-2.7-2.8-1 .6-2.9L2 12l2.4-1.2-.6-2.9 2.8-1 .8-2.7 2.9.2z"/><circle cx="12" cy="12" r="3"/>',
  search:'<circle cx="11" cy="11" r="7"/><path d="m16 16 5 5"/>',
  mic:'<rect x="9" y="2" width="6" height="13" rx="3"/><path d="M5 10a7 7 0 0 0 14 0M12 17v5m-4 0h8"/>',
  stop:'<rect x="5" y="5" width="14" height="14" rx="2"/>',
  upload:'<path d="M12 16V3m-5 5 5-5 5 5M4 16v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3"/>',
  info:'<circle cx="12" cy="12" r="10"/><path d="M12 11v6m0-10h.01"/>',
  globe:'<circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 0 1 0 20M12 2a15 15 0 0 0 0 20"/>',
  chevron:'<path d="m6 9 6 6 6-6"/>',
  back:'<path d="m14 7-5 5 5 5M9 12h12"/>',
  copy:'<rect x="8" y="8" width="13" height="13" rx="2"/><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/>',
  folder:'<path d="M3 7a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  spark:'<path d="m12 3 1.8 6.2L20 11l-6.2 1.8L12 19l-1.8-6.2L4 11l6.2-1.8zM20 17v4m-2-2h4"/>',
  file:'<path d="M7 3h7l5 5v13H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM14 3v6h5"/>',
  monitor:'<rect x="3" y="3" width="18" height="14" rx="2"/><path d="M8 21h8m-4-4v4"/>',
  check:'<path d="m5 12 4 4L19 6"/>',
  audio:'<path d="M8 7v10m4-14v18m4-14v10M4 10v4m16-4v4"/>',
  more:'<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
  clock:'<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  bolt:'<path d="m13 2-9 12h7l-1 8 10-12h-7z"/>',
  cpu:'<rect x="5" y="5" width="14" height="14" rx="2"/><rect x="9" y="9" width="6" height="6" rx="1"/><path d="M9 1v4m6-4v4M9 19v4m6-4v4M1 9h4m-4 6h4m14-6h4m-4 6h4"/>',
  eye:'<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
  eyeoff:'<path d="m3 3 18 18M10 5.3A12.6 12.6 0 0 1 12 5c6 0 10 7 10 7a16 16 0 0 1-3.6 4.2M6.3 6.3C3.5 8.1 2 12 2 12s4 7 10 7a9 9 0 0 0 4.3-1.1"/>',
  download:'<path d="M12 3v12m-5-5 5 5 5-5M4 17v3h16v-3"/>',
  shield:'<path d="M12 2 4 6v6c0 5 3.6 8.2 8 10 4.4-1.8 8-5 8-10V6z"/>',
  language:'<path d="M3 5h12m-6-3v3m4 0c-.7 5.8-4 9.6-9 12m2-9c1 3 3.4 5.3 7 7.5M15 10l5 12m-8 0 6-14"/>',
  refresh:'<path d="M20 11a8 8 0 0 0-14-5L3 9m0-6v6h6M4 13a8 8 0 0 0 14 5l3-3m0 6v-6h-6"/>',
  trash:'<path d="M3 6h18M8 6V4h8v2m3 0-1 15H6L5 6m5 4v7m4-7v7"/>',
  ellipsis:'<path d="M4 12h.01M12 12h.01M20 12h.01"/>',
  sliders:'<path d="M4 7h16M4 17h16M8 4v6m8 4v6"/>',
  headphones:'<path d="M4 14v-3a8 8 0 0 1 16 0v3M4 14v5h5v-6H4m16 1v5h-5v-6h5"/>',
};
const icon=(name)=>`<svg aria-hidden="true" viewBox="0 0 24 24">${paths[name]||paths.file}</svg>`;
const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const SEED=[];
const models=[
 {id:'maleo-turbo',name:'Maleo Indonesian Turbo',group:'id',size:'≈ 818 MB',speed:'Seimbang',quality:'Fokus bahasa Indonesia',hint:'Untuk kuliah dan rekaman bahasa Indonesia.'},
 {id:'maleo-small',name:'Maleo Indonesian Small',group:'id',size:'≈ 248 MB',speed:'Lebih cepat',quality:'Fokus bahasa Indonesia',hint:'Lebih hemat memori. Cocok untuk laptop dengan kemampuan sedang.'},
 {id:'whisper-small',name:'Whisper Small',group:'multi',size:'≈ 486 MB',speed:'Cepat',quality:'Multibahasa',hint:'Pilihan ringan untuk percakapan dengan beberapa bahasa.'},
 {id:'whisper-medium',name:'Whisper Medium',group:'multi',size:'≈ 1,53 GB',speed:'Seimbang',quality:'Multibahasa',hint:'Lebih banyak kemampuan dibanding model kecil, tetapi lebih berat.'},
 {id:'whisper-turbo',name:'Whisper Large-v3 Turbo',group:'multi',size:'≈ 1,62 GB',speed:'Relatif cepat',quality:'Multibahasa',hint:'Model multibahasa dengan optimasi kecepatan inferensi.'},
 {id:'whisper-large',name:'Whisper Large-v3',group:'multi',size:'≈ 3,09 GB',speed:'Lebih lambat',quality:'Multibahasa',hint:'Untuk kualitas transkrip akhir saat perangkat cukup kuat.'}
];
const providers={
 ollama:{name:'AI Lokal',desc:'Privat · Ollama',options:[['qwen3.5:4b','Qwen 3.5 · 4B · ≈3.4 GB'],['qwen3.5:2b','Qwen 3.5 (2B) · ≈2.7 GB'],['qwen2.5:3b','Qwen 2.5 · 3B · ≈1.9 GB'],['gemma3:4b','Gemma 3 · 4B · ≈3.3 GB'],['qwen2.5:1.5b','Qwen 2.5 · 1.5B · ≈1 GB']]},
 gemini:{name:'Gemini',desc:'Google · Online',options:[['gemini-2.5-flash','Gemini 2.5 Flash — cepat dan seimbang'],['gemini-2.5-pro','Gemini 2.5 Pro — untuk pembahasan kompleks'],['gemini-2.5-flash-lite','Gemini 2.5 Flash-Lite — ringan']]},
 openai:{name:'OpenAI',desc:'API · Online',options:[['gpt-4.1-mini','GPT-4.1 mini'],['gpt-4.1','GPT-4.1'],['gpt-4o-mini','GPT-4o mini']]},
 claude:{name:'Claude',desc:'Anthropic · Online',options:[['claude-sonnet','Claude Sonnet'],['claude-haiku','Claude Haiku'],['claude-opus','Claude Opus']]},
 groq:{name:'Groq',desc:'API · Online',options:[['llama-3.3-70b-versatile','Llama 3.3 70B'],['custom','Masukkan nama model sendiri']]},
 openrouter:{name:'OpenRouter',desc:'Beragam model',options:[['custom','Masukkan ID model OpenRouter']]},
 custom:{name:'Server sendiri',desc:'Kompatibel OpenAI',options:[['custom','Masukkan ID model sendiri']]}
};
const presets={model:'maleo-turbo',provider:'ollama',summaryModel:'qwen3.5:4b',language:'id',template:'materi',autoSummary:false,saveAudio:true,notify:true,device:'default',mic:'none'};
const safeStore={get(k){try{return JSON.parse(localStorage.getItem(k));}catch{return null;}},set(k,v){try{localStorage.setItem(k,JSON.stringify(v));}catch{}}};
const settings={...presets,...(safeStore.get('notelyDesktopSettings')||{})};
for(const [id,list] of Object.entries(settings.providerModels||{})){if(providers[id]&&Array.isArray(list))providers[id].options=list.map(m=>[m,m]);}
let savedDemo=null;
const meetings=[...SEED.map(m=>({...m,rows:m.rows.map(r=>[...r])})),...(Array.isArray(savedDemo)?savedDemo:[])];
const state={page:'home',tab:'general',selected:null,search:'',modelGroup:'id',settings,recording:false,started:0,recordTimer:null,recChunkTimer:null,liveIndex:0,recordSeconds:0,liveMeeting:null,contextText:'',showKey:false,apiKey:'',summaryLoading:false};
const persistSettings=()=>safeStore.set('notelyDesktopSettings',state.settings);
const liveSamples=['Baik, kita mulai diskusi hari ini mengenai langkah-langkah menyusun tugas akhir.','Pertama, tentukan tujuan penelitian supaya arah pembahasannya jelas.','Selanjutnya, perhatikan data yang diperlukan dan cara mengumpulkannya.','Untuk minggu depan, silakan siapkan gambaran awal dari topik yang dipilih.','Kalau ada pertanyaan, catat dulu agar bisa kita bahas bersama.'];
const getMeeting=()=>meetings.find(m=>m.id===state.selected)||meetings[0]||{id:'empty',title:'Belum ada catatan transkripsi',when:'',duration:'',type:'Kosong',context:'',summary:null,rows:[]};
const ts=sec=>`${String(Math.floor(sec/60)).padStart(2,'0')}:${String(Math.floor(sec%60)).padStart(2,'0')}`;
const transcriptText=m=>m.rows.map(r=>r[1]).join('\n\n');
const saveDemos=()=>safeStore.set('notelyPreviewSessions',meetings.filter(m=>!SEED.some(x=>x.id===m.id)));
function sidebar(){const active=state.page;return `<aside class="sidebar">
 <div class="brand"><img class="brand-logo" width="36" height="36" src="assets/notely.svg" alt="">Notely</div>
 <label class="search">${icon('search')}<input id="sidebar-search" placeholder="Cari catatan transkripsi..." value="${esc(state.search)}" autocomplete="off" aria-label="Cari catatan transkripsi"></label>
 <div class="sidebar-scroll"><button class="navlink ${active==='home'?'active':''}" data-action="page" data-id="home">${icon('home')} Beranda</button>
 <button class="navlink ${active==='notes'?'active':''}" data-action="page" data-id="notes">${icon('notes')} Catatan Rekaman</button>
 <button class="navlink ${active==='recordings'?'active':''}" data-action="page" data-id="recordings">${icon('folder')} Semua rekaman</button>
 <div class="list-label">Riwayat rekaman</div><div id="meeting-list">${meetingList()}</div></div>
 <div class="sidebar-bottom"><button class="sidebtn primary" data-action="${state.recording?'stop-demo':'ask-demo'}">${icon(state.recording?'stop':'mic')} ${state.recording?'Hentikan & simpan':'Mulai rekam'}</button>
 <button class="sidebtn secondary" data-action="import-audio">${icon('upload')} Impor audio</button>
 <button class="sidebtn neutral tiny" data-action="page" data-id="settings">${icon('settings')} Pengaturan</button>
 <button class="sidebtn neutral tiny" data-action="about">${icon('info')} Tentang</button>
 <div class="version">Notely · Desktop 1.0.0</div></div></aside>`;}
function meetingList(){const q=state.search.trim().toLowerCase();const filtered=meetings.filter(m=>(m.title+' '+m.rows.map(r=>r[1]).join(' ')).toLowerCase().includes(q));return filtered.length?filtered.slice().reverse().map(m=>`<button class="meeting-link ${state.page==='notes'&&state.selected===m.id?'active':''}" data-action="open-note" data-id="${esc(m.id)}">${icon('file')}<div><strong>${esc(m.title)}</strong><span>${esc(m.when)}${m.type==='Simulasi'?' · Demo':''}</span></div></button>`).join(''):`<div class="no-search">Tidak ada catatan yang cocok.</div>`;}
function appStrip(){return `<div class="app-strip"><span><strong>Notely</strong> &nbsp; / &nbsp; Desktop</span><span class="preview-note"><span class="dot"></span>${window.__notelyStatus||'Menghubungkan mesin transkripsi…'}</span></div>`;}
function render(){document.getElementById('root').innerHTML=`<div class="app">${sidebar()}<main class="workspace">${appStrip()}<div class="page" id="main-page">${renderPage()}</div></main></div>`;if(state.recording)updateTimer();}
function renderPage(){if(state.page==='recordings')return recordingsPage();if(state.page==='home')return homePage();if(state.page==='notes')return notesPage();return settingsPage();}
function homePage(){return `<div class="home"><div class="home-tools"><button class="button" data-action="change-home-language">${icon('globe')} ${state.settings.language==='id'?'Indonesia':'English'} ${icon('chevron')}</button></div>
<div class="home-center"><h1>${state.recording?'Transkripsi sedang berjalan':'Selamat datang di Notely'}</h1><p>${state.recording?'Buka Catatan Rekaman untuk membaca transkrip saat rekaman berlangsung.':'Rekam pembicaraan, baca transkrip langsung, dan rangkum hal penting.'}</p>${state.recording?'<div class="live-pill"><span class="pulse"></span> REKAMAN BERLANGSUNG</div>':''}</div>
<div class="home-action"><button class="circle-rec" data-action="${state.recording?'stop-demo':'ask-demo'}" aria-label="${state.recording?'Hentikan & simpan':'Mulai rekaman'}">${icon(state.recording?'stop':'mic')}</button><button class="morebtn" data-action="quick-menu" aria-label="Pilihan cepat">${icon('more')}</button></div></div>`;}
function titleSettings(){return `<div class="top-heading"><div class="heading-left"><button class="backlink" data-action="page" data-id="home">${icon('back')} Kembali</button><h1>Pengaturan</h1></div><span class="subtle-actions"><button class="button" data-action="about">${icon('info')} Panduan</button></span></div>`;}
const tabs=[['general','Umum','sliders'],['recordings','Rekaman','mic'],['transcription','Transkripsi','audio'],['summary','Ringkasan AI','spark']];
function settingsPage(){return `<section class="settings" data-settings-tab="${esc(state.tab)}">${titleSettings()}<nav class="tabs">${tabs.map(([id,label,ico])=>`<button class="tab ${state.tab===id?'active':''}" data-action="tab" data-id="${id}">${icon(ico)}${label}</button>`).join('')}</nav><div class="settings-scroll"><div class="settings-body">${renderTab()}</div></div></section>`;}
function toggle(id,label,description,checked){return `<div class="setting-card"><div class="setting-row"><div><h3>${label}</h3><p>${description}</p></div><input type="checkbox" class="switch" data-setting="${id}" ${checked?'checked':''} aria-label="${label}"></div></div>`;}
function generalTab(){return `<h2 class="section-title">Preferensi aplikasi</h2><p class="section-description">Sesuaikan preferensi Anda.</p>
${toggle('notify','Notifikasi perekaman','Tampilkan pemberitahuan ketika perekaman dimulai atau selesai.',state.settings.notify)}
<div class="setting-card"><h3 class="section-title">Bahasa aplikasi</h3><p class="section-description">Bahasa antarmuka Notely.</p><div class="field" style="max-width:350px"><select class="control-select" data-setting="language"><option value="id" ${state.settings.language==='id'?'selected':''}>Indonesia</option><option value="en" ${state.settings.language==='en'?'selected':''}>English (pratinjau belum diterjemahkan)</option></select></div></div>
<div class="setting-card"><h3 class="section-title">Lokasi penyimpanan</h3><p class="section-description">Rekaman dan transkrip disimpan di Documents/Notely.</p><button class="button" data-action="desktop-only">${icon('folder')} Lihat pengaturan folder</button></div>`;}
function recordingTab(){return `<h2 class="section-title">Pengaturan perekaman</h2><p class="section-description">Perangkat audio dan pengelolaan file rekaman.</p>
${toggle('saveAudio','Simpan rekaman suara','Simpan audio bersama catatan.',state.settings.saveAudio)}
<div class="setting-card"><div class="split-field"><div class="field"><span class="field-label">Suara komputer</span><select class="control-select" data-setting="device"><option value="default">Speaker / headset utama (Windows)</option></select></div><div class="field"><span class="field-label">Mikrofon Anda</span><select class="control-select" data-setting="mic"><option value="none">Tidak direkam</option><option value="default" ${state.settings.mic==='default'?'selected':''}>Mikrofon utama</option></select><span class="field-help">Opsional, jika Anda juga ingin menyimpan suara sendiri.</span></div></div></div>
`;}
function modelCard(m){const chosen=state.settings.model===m.id;return `<article class="model-card ${chosen?'active':''}"><div class="model-symbol">${icon(m.group==='id'?'audio':'cpu')}</div><div class="model-info"><div class="model-name">${esc(m.name)}${m.id==='maleo-turbo'?'<span class="tag">Pilihan awal</span>':''}</div><p class="model-description">${esc(m.hint)}</p><div class="model-stats"><span>${icon('cpu')} ${esc(m.size)}</span><span>${icon('bolt')} ${esc(m.speed)}</span><span>${icon('globe')} ${esc(m.quality)}</span></div></div><div class="model-actions">${chosen?'<span class="status-text">Dipilih</span>':''}<button class="button ${chosen?'selected':''} small" data-action="choose-model" data-id="${m.id}">${chosen?'Terpilih':'Pilih'}</button><button class="button small ghost" data-action="model-info" data-id="${m.id}" title="Informasi pemasangan">${icon('info')}</button></div></article>`;}
function transcriptionTab(){const group=state.modelGroup;return `<div class="select-intro"><div><h2 class="section-title">Model transkripsi</h2><p class="section-description" style="margin-bottom:0">Pilih AI yang mengubah suara menjadi teks. Model lokal diproses di komputer.</p></div><select class="control-select" data-change="model-group" aria-label="Kelompok model"><option value="id" ${group==='id'?'selected':''}>Khusus Bahasa Indonesia</option><option value="multi" ${group==='multi'?'selected':''}>Whisper multibahasa</option></select></div>

<div class="model-group">${models.filter(m=>m.group===group).map(modelCard).join('')}</div>`;}
const providerCard=(id,obj)=>`<button class="provider-card ${state.settings.provider===id?'active':''}" data-action="choose-provider" data-id="${id}"><strong>${esc(obj.name)} ${state.settings.provider===id?'✓':''}</strong><small>${esc(obj.desc)}</small></button>`;
function summaryTab(){const p=state.settings.provider;const provider=providers[p]||providers.gemini;const currently=state.settings.summaryModel;const keyRequired=p!=='ollama';const needsCustom=currently==='custom';return `<h2 class="section-title">Ringkasan berbantuan AI</h2><p class="section-description">Pilih layanan AI dan model yang akan merangkum isi transkrip. Transkrip hanya dikirim ke penyedia online setelah Anda menyetujui permintaan.</p>
${toggle('autoSummary','Ringkas otomatis setelah rekaman','Jalankan ringkasan ketika rekaman selesai (pada versi aplikasi yang terhubung).',state.settings.autoSummary)}
<div class="setting-card"><h3 class="section-title">Penyedia AI</h3><p class="section-description">Gunakan AI online dengan API key, atau AI lokal di komputer.</p><div class="provider-group">${Object.entries(providers).map(([id,v])=>providerCard(id,v)).join('')}</div><div class="hr"></div>
<div class="split-field"><div class="field"><span class="field-label">Model ${esc(provider.name)}</span><select class="control-select" data-change="summary-model">${provider.options.map(([id,name])=>`<option value="${esc(id)}" ${id===currently?'selected':''}>${esc(name)}</option>`).join('')}</select><span class="field-help">Model yang benar-benar tersedia ditentukan oleh akun dan layanan penyedia.</span></div><div class="field"><span class="field-label">Bahasa ringkasan</span><select class="control-select" data-change="summary-language"><option value="id" ${state.settings.summaryLanguage!=='en'?'selected':''}>Bahasa Indonesia</option><option value="en" ${state.settings.summaryLanguage==='en'?'selected':''}>English</option></select><span class="field-help">Bahasa hasil ringkasan tidak harus sama dengan bahasa sumber audio.</span></div></div>
${needsCustom?'<div class="field" style="margin-top:17px"><label class="field-label" for="custom-model">ID model khusus</label><input class="textbox" id="custom-model" placeholder="Contoh: penyedia/nama-model" value="'+esc(state.settings.customModel||'')+'"></div>':''}
${keyRequired?`<div class="field" style="margin-top:17px"><label class="field-label" for="api-key">API key ${esc(provider.name)}</label><div class="keyinput"><input class="textbox" id="api-key" type="${state.showKey?'text':'password'}" placeholder="Masukkan API key untuk penggunaan online" autocomplete="off" value="${esc(state.apiKey)}"><button data-action="toggle-key" aria-label="Tampilkan atau sembunyikan API key">${icon(state.showKey?'eyeoff':'eye')}</button></div><span class="field-help">API key disimpan terenkripsi untuk akun Windows ini.</span></div>`:`<p class="provider-hint">Ollama menjalankan model di komputer Anda. Ollama perlu dipasang dan dijalankan secara terpisah sebelum mengunduh model lokal melalui Notely.</p>`}
<div class="space-between" style="margin-top:20px"><button class="button" data-action="ai-connection">${icon('refresh')} Periksa koneksi</button><button class="button accent" data-action="save-ai">${icon('check')} Simpan pilihan</button></div></div>
<div class="setting-card"><h3 class="section-title">Format ringkasan</h3><p class="section-description">Sesuaikan susunan hasil yang dibuat AI.</p><select class="control-select" style="max-width:430px" data-change="template"><option value="materi" ${state.settings.template==='materi'?'selected':''}>Ringkasan materi — poin-poin penting</option><option value="rekaman" ${state.settings.template==='rekaman'?'selected':''}>Notulen rekaman — keputusan dan tindak lanjut</option><option value="singkat" ${state.settings.template==='singkat'?'selected':''}>Ringkasan cepat — inti pembahasan</option></select></div>`;}
function renderTab(){return state.tab==='general'?generalTab():state.tab==='recordings'?recordingTab():state.tab==='transcription'?transcriptionTab():state.tab==='summary'?summaryTab():generalTab();}
function transcriptList(m){const rows=m.rows||[];return rows.length?rows.map(([sec,t])=>`<div class="transcript-row"><span class="timestamp">[${ts(Number(sec)||0)}]</span><p>${esc(t)}</p></div>`).join(''):`<p class="muted-block">Belum ada transkrip untuk sesi ini.</p>`;}
function summaryPaper(m){const isRap=state.settings.template==='rekaman';const isQuick=state.settings.template==='singkat';if(!m.summary)return `<div class="summary-empty"><div class="empty-icon">${icon('notes')}</div><h3>Belum ada ringkasan</h3><p>Ringkasan materi atau keputusan rekaman akan muncul di sini setelah diproses AI.</p><button class="button accent" data-action="generate-summary">${icon('spark')} Lihat contoh ringkasan</button></div>`;
return `<article class="summary-paper"><h2>${isRap?'Notulen Rekaman':isQuick?'Inti Pembahasan':'Ringkasan Materi'} — ${esc(m.title)}</h2><p>Ini contoh tampilan ringkasan untuk menguji desain. Pada aplikasi lengkap, teks akan dibuat dari transkrip menggunakan model yang dipilih.</p>
<h3>${isRap?'Pokok Pembahasan':'Poin Penting'}</h3><ul>${(m.id==='kuliah'?['Pahami masalah dan tujuan penelitian sebelum menentukan metode.','Pastikan rumusan masalah, tujuan, dan data saling berkaitan.','Pilih cara pengumpulan data sesuai kebutuhan penelitian.']:m.rows.slice(0,3).map(r=>r[1])).map(x=>`<li>${esc(x)}</li>`).join('')}</ul>
${!isQuick?`<h3>${isRap?'Tindak Lanjut':'Hal yang Perlu Diingat'}</h3><ul><li>${esc(m.id==='kuliah'?'Menyiapkan rancangan judul, masalah, dan tujuan penelitian untuk pertemuan berikutnya.':'Periksa kembali bagian tindak lanjut berdasarkan rekaman dan konteks rekaman.')}</li></ul>`:''}
<h3>Catatan</h3><p>Verifikasi hasil penting dengan audio asli, terutama nama, angka, dan istilah khusus.</p></article>`;}
function notesPage(){const m=getMeeting();const rec=state.recording;return `<div class="notes-page"><div class="notes-toolbar"><div class="toolbar-group"><span class="note-title">${icon('file')} ${esc(m.title)} <span style="color:#b3bdc3">/</span> ${esc(m.when)}</span><button class="button small" data-action="copy-transcript">${icon('copy')} Salin</button><button class="button small" data-action="recording-info">${icon('headphones')} Rekaman</button></div><div class="toolbar-group"><button class="button ${m.summary?'selected':''} small" data-action="generate-summary">${icon('spark')} ${m.summary?'Buat ulang ringkasan':'Buat ringkasan'}</button><select class="tiny-select" data-change="summary-language" title="Bahasa ringkasan"><option value="id" ${state.settings.summaryLanguage!=='en'?'selected':''}>Indonesia</option><option value="en" ${state.settings.summaryLanguage==='en'?'selected':''}>English</option></select><button class="button small" data-action="go-summary">${icon('settings')} Model AI</button></div></div>
${rec?`<div class="notes-preview-banner"><span class="pulse" style="display:inline-block;margin-right:8px"></span>Perekaman berlangsung. Teks sementara dapat berubah saat AI mendengar konteks tambahan.</div>`:''}
<div class="notes-content"><section class="transcript-pane"><div class="pane-head"><h2>Transkrip ${rec?'<span class="small"> · sedang diperbarui</span>':''}</h2><span class="small">${m.rows.length} bagian</span></div><div class="transcript-list" id="transcript-list">${transcriptList(m)}</div><div class="transcript-bottom"><label class="field-label" for="meeting-context">Konteks untuk ringkasan (opsional)</label><textarea id="meeting-context" placeholder="Contoh: mata kuliah, tujuan rekaman, atau materi yang dibahas...">${esc(m.context||'')}</textarea></div></section>
<section class="summary-pane"><div class="pane-head"><h2>Ringkasan AI</h2><div class="toolbar-group"><span class="small">${esc(providers[state.settings.provider]?.name||'Gemini')}</span><button class="button small" data-action="template-menu">${icon('sliders')} Templat</button></div></div><div class="summary-scroll" id="summary-scroll">${summaryPaper(m)}</div><div class="summary-actions"><span class="muted-block">${rec?'Perekaman berlangsung':'Ringkasan dapat disalin setelah dibuat'}</span><button class="button small" data-action="copy-summary" ${!m.summary?'disabled':''}>${icon('copy')} Salin ringkasan</button></div></section></div></div>`;}
function refreshPage(){
 const p=document.getElementById('main-page');
 // Re-rendering a setting must not behave like navigating to another tab.
 const previous=p?.querySelector('.settings');
 const sameTab=state.page==='settings'&&previous?.dataset.settingsTab===state.tab;
 const scroll=sameTab?previous.querySelector('.settings-scroll'):null;
 const scrollTop=scroll?.scrollTop||0,scrollLeft=scroll?.scrollLeft||0;
 const active=scroll?.contains(document.activeElement)?document.activeElement:null;
 const focusId=active?.id;
 const selection=active&&typeof active.selectionStart==='number'?[active.selectionStart,active.selectionEnd]:null;
 const guide=scroll?.querySelector('.api-guide');
 const guideOpen=guide?.tagName==='DETAILS'?guide.open:null;
 if(p)p.innerHTML=renderPage();
 if(sameTab){
  const next=p?.querySelector('.settings-scroll');
  const nextGuide=next?.querySelector('.api-guide');
  if(guideOpen!==null&&nextGuide?.tagName==='DETAILS')nextGuide.open=guideOpen;
  const focus=focusId?document.getElementById(focusId):null;
  if(focus&&next?.contains(focus)){
   focus.focus({preventScroll:true});
   if(selection&&typeof focus.setSelectionRange==='function')focus.setSelectionRange(...selection);
  }
  if(next){next.scrollTop=scrollTop;next.scrollLeft=scrollLeft;}
 }
 const list=document.getElementById('meeting-list');if(list)list.innerHTML=meetingList();
 const recBtn=document.querySelector('.sidebar-bottom .sidebtn.primary');
 if(recBtn){recBtn.setAttribute('data-action',state.recording?'stop-demo':'ask-demo');recBtn.innerHTML=icon(state.recording?'stop':'mic')+' '+(state.recording?'Hentikan & simpan':'Mulai rekam');}
 if(state.recording)updateTimer();
}
function selectPage(page){state.page=page;render();}
let toastTimeout;
function toast(s){const t=document.getElementById('toast');t.textContent=s;t.classList.add('show');clearTimeout(toastTimeout);toastTimeout=setTimeout(()=>t.classList.remove('show'),3500);}
function modal(title,desc,actions=[['Tutup','close-modal','']]){const root=document.getElementById('modal-root');root.innerHTML=`<div class="overlay" role="presentation"><div class="modal" role="dialog" aria-modal="true" aria-label="${esc(title)}"><h2>${esc(title)}</h2><p>${desc}</p><div class="modal-actions">${actions.map(([label,action,kind])=>`<button class="button ${kind||''}" data-action="${action}">${esc(label)}</button>`).join('')}</div></div></div>`;}
const closeModal=()=>{document.getElementById('modal-root').innerHTML='';};
function beginDemo(){closeModal();if(state.recording)return;state.recording=true;state.started=Date.now();state.liveIndex=0;state.recordSeconds=0;const item={id:'demo-'+Date.now(),title:'Simulasi Transkripsi',when:'Sesi demo · hari ini',duration:'00:00',type:'Simulasi',context:'',summary:null,rows:[]};meetings.push(item);state.selected=item.id;state.page='notes';render();state.recordTimer=setInterval(updateTimer,1000);state.recChunkTimer=setInterval(()=>{if(!state.recording)return;const m=getMeeting();const sec=Math.floor((Date.now()-state.started)/1000);m.rows.push([sec,liveSamples[state.liveIndex%liveSamples.length]]);state.liveIndex++;if(state.page==='notes'){const list=document.getElementById('transcript-list');if(list){const atBottom=list.scrollTop+list.clientHeight>=list.scrollHeight-80;list.insertAdjacentHTML('beforeend',`<div class="transcript-row"><span class="timestamp">[${ts(sec)}]</span><p>${esc(liveSamples[(state.liveIndex-1)%liveSamples.length])}</p></div>`);if(atBottom)list.scrollTop=list.scrollHeight;}const h=document.querySelector('.transcript-pane .pane-head .small');if(h)h.textContent=` · ${m.rows.length} bagian`;} },2100);toast('Simulasi dimulai. Suara asli tidak direkam.');}
function updateTimer(){if(!state.recording)return;state.recordSeconds=Math.floor((Date.now()-state.started)/1000);const s=document.getElementById('demo-timer');if(s)s.textContent=ts(state.recordSeconds);}
function stopDemo(){if(!state.recording)return;state.recording=false;clearInterval(state.recordTimer);clearInterval(state.recChunkTimer);const m=getMeeting();m.duration=ts(state.recordSeconds);if(!m.rows.length)m.rows.push([0,'Sesi simulasi berakhir sebelum teks contoh pertama muncul.']);saveDemos();refreshPage();toast('Simulasi selesai. Sesi contoh tersimpan di browser ini.');}
function generateSummary(){const m=getMeeting();if(!m.rows.length){toast('Belum ada transkrip untuk dicontohkan.');return;}m.summary=true;if(m.type==='Simulasi')saveDemos();refreshPage();toast('Contoh ringkasan ditampilkan. Tidak ada permintaan API yang dikirim.');}
function copyText(s){if(navigator.clipboard?.writeText){navigator.clipboard.writeText(s).then(()=>toast('Teks disalin ke clipboard.')).catch(()=>fallbackCopy(s));}else fallbackCopy(s);}
function fallbackCopy(s){const t=document.createElement('textarea');t.value=s;t.style.position='fixed';t.style.left='-9999px';document.body.appendChild(t);t.select();const ok=document.execCommand('copy');t.remove();toast(ok?'Teks disalin.':'Salin tidak didukung browser ini.');}
function openAudioPicker(){document.getElementById('audio-file').click();}
function modelInfo(id){const m=models.find(x=>x.id===id);if(!m)return;modal('Tentang '+m.name,`<strong>${esc(m.hint)}</strong><br><br>Unduh model ini saat persiapan awal atau dari Pengaturan → Transkripsi. CUDA dipilih secara otomatis bila GPU kompatibel.`,[['Kembali','close-modal','']]);}
function handleClick(action,id){switch(action){
case 'page':selectPage(id);break;
case 'open-note':state.selected=id;selectPage('notes');break;
case 'tab':state.tab=id;refreshPage();break;
case 'choose-model':state.settings.model=id;persistSettings();refreshPage();toast('Pilihan model disimpan.');break;
case 'model-info':modelInfo(id);break;
case 'choose-provider':{state.settings.provider=id;state.settings.summaryModel=providers[id].options[0][0];state.apiKey=state.savedKeys?.[id]||'';state.showKey=false;persistSettings();refreshPage();break;}
case 'save-ai':persistSettings();toast('Pilihan AI disimpan.');break;
case 'toggle-key':{const v=document.getElementById('api-key');if(v)state.apiKey=v.value;state.showKey=!state.showKey;refreshPage();break;}
case 'ai-connection':modal('Pemeriksaan koneksi AI','Tombol ini belum terhubung ke API pada pratinjau browser. Untuk aplikasi final, uji koneksi dilakukan dengan token pribadi dan informasi dari provider.',[['Mengerti','close-modal','']]);break;
case 'generate-summary':generateSummary();break;
case 'copy-transcript':copyText(transcriptText(getMeeting()));break;
case 'copy-summary':copyText(document.querySelector('.summary-paper')?.innerText||'');break;
case 'go-summary':state.page='settings';state.tab='summary';render();break;
case 'template-menu':modal('Templat ringkasan','Gunakan Pengaturan → Ringkasan AI untuk memilih ringkasan materi, catatan transkripsi rekaman, atau ringkasan cepat.',[['Ke pengaturan','go-summary','accent'],['Kembali','close-modal','']]);break;
case 'about':modal('Notely Desktop 1.0.0','Antarmuka ini bisa diuji secara offline. Transkrip, rekaman, dan ringkasan dalam contoh adalah data simulasi. Perekaman suara Windows, CUDA, unduhan model, dan permintaan API tidak aktif dalam HTML ini.',[['Tutup','close-modal','']]);break;
case 'quick-menu':modal('Aksi cepat','Coba simulasi transkrip, buka catatan, atau atur model AI. Semua fitur mesin suara memerlukan backend pada tahap berikutnya.',[['Coba simulasi','start-demo','accent'],['Tutup','close-modal','']]);break;
case 'ask-demo':modal('Coba tampilan saat merekam','Browser pratinjau ini tidak menangkap suara speaker, mikrofon, atau rekaman Anda. Untuk menguji tata letak, kita dapat menjalankan transkrip contoh yang muncul bertahap selama simulasi.',[['Batal','close-modal',''],['Mulai simulasi','start-demo','accent']]);break;
case 'start-demo':beginDemo();break;
case 'stop-demo':stopDemo();break;
case 'close-modal':closeModal();break;
case 'recording-info':modal('Audio rekaman','Sesi contoh tidak memiliki rekaman audio sungguhan. Pada aplikasi Windows nantinya, bagian ini akan menyediakan pemutar rekaman utama dan tautan ke file yang disimpan.',[['Tutup','close-modal','']]);break;
case 'import-audio':openAudioPicker();break;
case 'desktop-only':modal('Fitur untuk aplikasi Windows','Pengaturan folder, perangkat audio, dan GPU membutuhkan akses sistem yang tidak tersedia pada file HTML pratinjau ini. Fitur tersebut akan dihubungkan ke backend Python dalam tahap berikutnya.',[['Mengerti','close-modal','']]);break;
case 'change-home-language':state.page='settings';state.tab='general';render();break;
}}
document.addEventListener('click',e=>{const b=e.target.closest('[data-action]');if(b){e.preventDefault();handleClick(b.dataset.action,b.dataset.id);return;}if(e.target.classList.contains('overlay'))closeModal();});
document.addEventListener('change',e=>{const el=e.target;const id=el.dataset.setting;if(id){state.settings[id]=el.type==='checkbox'?el.checked:el.value;persistSettings();if(id==='language')toast('Pilihan bahasa tersimpan. Prototipe ini ditulis dalam bahasa Indonesia.');return;}const ch=el.dataset.change;switch(ch){case 'model-group':state.modelGroup=el.value;refreshPage();break;case 'summary-model':state.settings.summaryModel=el.value;persistSettings();refreshPage();break;case 'summary-language':state.settings.summaryLanguage=el.value;persistSettings();break;case 'template':state.settings.template=el.value;persistSettings();break;}});
document.addEventListener('input',e=>{const el=e.target;if(el.id==='sidebar-search'){state.search=el.value;document.getElementById('meeting-list').innerHTML=meetingList();}else if(el.id==='meeting-context'){getMeeting().context=el.value;if(getMeeting().type==='Simulasi')saveDemos();}else if(el.id==='api-key'){state.apiKey=el.value;}else if(el.id==='custom-model'){state.settings.customModel=el.value;persistSettings();}});
document.addEventListener('keydown',e=>{if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'){e.preventDefault();document.getElementById('sidebar-search')?.focus();}if(e.key==='Escape')closeModal();});
document.getElementById('audio-file').addEventListener('change',e=>{const f=e.target.files[0];if(f){const u=URL.createObjectURL(f);modal('Audio lokal dipilih',`Berkas: <strong>${esc(f.name)}</strong><br><br>Pratinjau browser dapat memilih audio, tetapi belum melakukan transkripsi. Pemrosesan WAV/FLAC dengan Maleo AI akan terhubung melalui backend pada tahap berikutnya.`,[['Mengerti','close-modal','']]);setTimeout(()=>URL.revokeObjectURL(u),60000);}e.target.value='';});
render();


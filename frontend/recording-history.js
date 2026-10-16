function folderOptions(selected=''){
 return `<option value="" ${!selected||selected==='all'?'selected':''}>Semua rekaman (halaman utama)</option>`+(state.folders||[]).map(f=>`<option value="${esc(f.id)}" ${selected===f.id?'selected':''}>${esc(f.name)}</option>`).join('');
}
function recordingMB(n){return ((n||0)/1000000).toLocaleString('id-ID',{minimumFractionDigits:2,maximumFractionDigits:2})+' MB';}
meetingList=function(){
 const q=state.search.trim().toLowerCase();
 const filtered=meetings.filter(m=>(m.title+' '+m.rows.map(r=>r[1]).join(' ')).toLowerCase().includes(q));
 return (filtered.length?filtered.slice().reverse().map(m=>`<div class="meeting-row"><button class="meeting-link ${state.selected===m.id&&state.page==='notes'?'active':''}" data-action="open-note" data-id="${esc(m.id)}">${icon('file')}<div><strong>${esc(m.title)}</strong><span>${esc(m.when)}</span><span>${recordingMB(m.audioBytes)} audio</span></div></button><button class="delete-recording" data-action="delete-recording" data-id="${esc(m.id)}" aria-label="Hapus ${esc(m.title)}">Hapus</button></div>`).join(''):'<div class="no-search">Belum ada rekaman yang cocok.</div>');
};
const beforeDeleteClick=handleClick;
handleClick=function(action,id){
 if(action==='open-folder'){
  const folder=(state.folders||[]).find(f=>f.id===id);
  state.libraryFolder=folder?folder.id:'all';state.folderFilter=state.libraryFolder;state.librarySearch='';state.page='recordings';render();
  document.querySelector('.library-breadcrumb button')?.focus();return;
 }
 if(action==='page'&&id==='recordings'){state.libraryFolder='all';state.folderFilter='all';state.librarySearch='';}
 if(action==='delete-folder'){
  const folder=(state.folders||[]).find(f=>f.id===(id||state.libraryFolder));if(!folder)return;
  state.pendingFolderDelete=folder.id;
  return modal('Hapus folder',`Folder <strong>${esc(folder.name)}</strong> akan dihapus. Semua rekaman di dalamnya dipindahkan ke <strong>halaman utama Semua rekaman</strong>. Audio dan catatan tetap tersimpan.`,[['Batal','close-modal',''],['Hapus folder','confirm-delete-folder','danger']]);
 }
 if(action==='confirm-delete-folder'){
  if(state.organizeBusy||!state.pendingFolderDelete)return;
  state.organizeBusy=true;
  return api('/folders/delete',{id:state.pendingFolderDelete}).then(async()=>{
   state.folderFilter='all';state.libraryFolder='all';state.pendingFolderDelete=null;closeModal();await loadSessions();
   if(state.page==='notes'&&state.selected)await loadSession(state.selected);else render();
   toast('Folder dihapus. Rekaman tetap tersimpan.');
  }).catch(alertError).finally(()=>{state.organizeBusy=false;});
 }

 if(['rename-recording','move-recording','new-folder','rename-folder'].includes(action)){
  const m=meetings.find(x=>x.id===id);
  if(state.organizeBusy)return;
  state.organizeTarget=id;state.organizeAction=action;
  const folder=(state.folders||[]).find(f=>f.id===(id||state.libraryFolder));
  if(action==='rename-folder')state.organizeTarget=folder?.id;
  const moving=action==='move-recording';
  const title=moving?'Pindahkan rekaman':action==='rename-recording'?'Ubah nama rekaman':action==='new-folder'?'Buat folder':'Ubah nama folder';
  modal(title,moving?`<label for="organize-value">Folder tujuan</label><select id="organize-value">${folderOptions(m?.folder||'')}</select>`:`<label for="organize-value">Nama</label><input id="organize-value" maxlength="100" value="${esc(action==='rename-recording'?m?.title:action==='rename-folder'?folder?.name:'')}" placeholder="">`,[['Batal','close-modal',''],['Simpan','save-organization','accent']]);
  document.getElementById('organize-value')?.closest('.modal')?.classList.add('organize-dialog');
  document.getElementById('organize-value')?.focus();return;
 }
 if(action==='save-organization'){
  if(state.organizeBusy)return;
  const value=document.getElementById('organize-value')?.value.trim();
  if(state.organizeAction!=='move-recording'&&!value){document.getElementById('organize-value')?.focus();return toast('Nama perlu diisi.');}
  const kind=state.organizeAction;
  const route=kind==='new-folder'?'/folders/create':kind==='rename-folder'?'/folders/rename':'/session/update';
  const payload=kind==='new-folder'?{name:value}:kind==='rename-folder'?{id:state.organizeTarget,name:value}:kind==='move-recording'?{id:state.organizeTarget,folder:value}:{id:state.organizeTarget,title:value};
  state.organizeBusy=true;
  return api(route,payload).then(async result=>{
   if(kind==='new-folder'){state.folderFilter='all';state.libraryFolder='all';state.librarySearch='';state.page='recordings';}
   closeModal();await loadSessions();
   if(state.page==='notes'&&state.selected)await loadSession(state.selected);else render();
   toast('Perubahan disimpan.');
  }).catch(alertError).finally(()=>{state.organizeBusy=false;});
 }
 if(action==='delete-recording'){
  const m=meetings.find(m=>m.id===id);if(!m)return;
  if(state.recording||engine.summaryJob||engine.transcribeJob)return toast('Selesaikan proses aktif terlebih dahulu.');
  state.pendingDelete=id;
  return modal('Hapus rekaman?',`<strong>${esc(m.title)}</strong><p>Audio: ${recordingMB(m.audioBytes)} · Total: ${recordingMB(m.sizeBytes)}</p><p>Hapus audio saja untuk mempertahankan transkrip dan ringkasan, atau hapus seluruh catatan. Penghapusan bersifat permanen.</p>`,[['Batal','close-modal',''],['Audio saja','delete-audio-confirm',''],['Hapus semua','delete-all-confirm','accent']]);
 }
 if(action==='delete-audio-confirm'||action==='delete-all-confirm'){
  if(state.deletingRecording)return;
  const target=state.pendingDelete;if(!target)return;
  state.deletingRecording=true;
  return api('/session/delete',{id:target,audioOnly:action==='delete-audio-confirm'}).then(async result=>{
   closeModal();state.pendingDelete=null;await loadSessions();
   if(state.page!=='recordings'){if(state.selected)await loadSession(state.selected);else state.page='home';}
   render();toast('Berhasil dihapus. Ruang dibebaskan: '+recordingMB(result.freedBytes));
  }).catch(alertError).finally(()=>{state.deletingRecording=false;});
 }
 return beforeDeleteClick(action,id);
};
render();

let recordingMenuOrigin=null;
function closeRecordingMenu(restore=false){
 document.getElementById('recording-context-menu')?.remove();
 if(restore)recordingMenuOrigin?.focus();
}
function openRecordingMenu(target,x,y){
 closeRecordingMenu();recordingMenuOrigin=target;
 const menu=document.createElement('div');menu.id='recording-context-menu';menu.className='recording-context-menu';menu.setAttribute('role','menu');
 menu.innerHTML=['rename-recording','move-recording'].map((action,i)=>`<button role="menuitem" data-action="${action}" data-id="${esc(target.dataset.id)}">${i?'Pindahkan ke folder':'Ubah nama'}</button>`).join('');
 document.body.appendChild(menu);
 menu.style.left=Math.max(8,Math.min(x,innerWidth-menu.offsetWidth-8))+'px';
 menu.style.top=Math.max(8,Math.min(y,innerHeight-menu.offsetHeight-8))+'px';
 menu.querySelector('button').focus();
}
document.addEventListener('contextmenu',e=>{const target=e.target.closest('[data-action="open-note"]');if(!target)return;e.preventDefault();openRecordingMenu(target,e.clientX,e.clientY);});
document.addEventListener('pointerdown',e=>{if(!e.target.closest('#recording-context-menu'))closeRecordingMenu();});
document.addEventListener('click',e=>{if(e.target.closest('#recording-context-menu'))closeRecordingMenu();});
document.addEventListener('keydown',e=>{
 const target=e.target.closest('[data-action="open-note"]');
 if(target&&(e.key==='ContextMenu'||(e.shiftKey&&e.key==='F10'))){e.preventDefault();const r=target.getBoundingClientRect();openRecordingMenu(target,r.left,r.bottom);return;}
 const menu=document.getElementById('recording-context-menu');if(!menu)return;
 if(e.key==='Escape'){e.preventDefault();closeRecordingMenu(true);}
 if(['ArrowDown','ArrowUp','Tab'].includes(e.key)){e.preventDefault();const items=[...menu.querySelectorAll('button')];items[(items.indexOf(document.activeElement)+1)%items.length].focus();}
});
window.addEventListener('resize',()=>closeRecordingMenu());
document.addEventListener('scroll',()=>closeRecordingMenu(),true);

function currentLibraryFolder(){return (state.folders||[]).find(f=>f.id===state.libraryFolder)||null;}
function libraryRecordings(){
 const folder=currentLibraryFolder();
 return meetings.filter(m=>folder?m.folder===folder.id:!m.folder||!(state.folders||[]).some(f=>f.id===m.folder));
}
function recordingCards(){
 const query=(state.librarySearch||'').trim().toLowerCase();
 const items=libraryRecordings().filter(m=>m.title.toLowerCase().includes(query)).slice().reverse();
 return items.length?items.map(m=>`<article class="library-recording"><div class="library-recording-icon">${icon('file')}</div><div class="library-recording-info"><button class="recording-title" data-action="open-note" data-id="${esc(m.id)}">${esc(m.title)}</button><div class="recording-meta"><span>${esc(m.when)}</span><span>${recordingMB(m.audioBytes)} audio</span></div></div><div class="library-recording-actions"><button class="button small" data-action="open-note" data-id="${esc(m.id)}">Buka</button><button class="button small" data-action="rename-recording" data-id="${esc(m.id)}">Ubah nama</button><button class="button small" data-action="move-recording" data-id="${esc(m.id)}">Pindahkan</button><button class="button small danger" data-action="delete-recording" data-id="${esc(m.id)}">Hapus</button></div></article>`).join(''):`<div class="library-empty">${query?'Tidak ada rekaman yang cocok di lokasi ini.':currentLibraryFolder()?'Folder ini masih kosong. Pindahkan rekaman ke sini atau impor audio.':'Rekaman yang belum dipindahkan ke folder akan tampil di sini.'}</div>`;
}
function libraryFolderCards(){
 if(currentLibraryFolder())return '';
 const query=(state.librarySearch||'').trim().toLowerCase();
 const folders=(state.folders||[]).filter(f=>f.name.toLowerCase().includes(query));
 return `<section class="library-folders" aria-label="Folder"><h2>Folder <span>${folders.length}</span></h2>${folders.length?`<div class="folder-grid">${folders.map(f=>`<article class="folder-card"><button class="folder-open" data-action="open-folder" data-id="${esc(f.id)}" aria-label="Buka folder ${esc(f.name)}"><span class="folder-glyph">${icon('folder')}</span><strong>${esc(f.name)}</strong><small>${meetings.filter(m=>m.folder===f.id).length} rekaman</small><span class="folder-arrow" aria-hidden="true">↗</span></button><div class="folder-card-actions"><button class="button small ghost" data-action="rename-folder" data-id="${esc(f.id)}" aria-label="Ubah nama folder ${esc(f.name)}">Ubah nama</button><button class="button small ghost" data-action="delete-folder" data-id="${esc(f.id)}" aria-label="Hapus folder ${esc(f.name)}">${icon('trash')} Hapus</button></div></article>`).join('')}</div>`:`<p class="folder-empty">${query?'Tidak ada folder yang cocok.':'Buat folder untuk mengelompokkan rekaman Anda.'}</p>`}</section>`;
}
function recordingsPage(){
 const folder=currentLibraryFolder(),items=libraryRecordings();
 if(!folder){state.libraryFolder='all';state.folderFilter='all';}
 return `<section class="recordings-page explorer-page"><nav class="library-breadcrumb" aria-label="Lokasi rekaman"><button data-action="open-folder" data-id="all" ${folder?'':'aria-current="page"'}>${icon('folder')} Semua folder</button>${folder?`<span aria-hidden="true">/</span><span aria-current="page">${esc(folder.name)}</span>`:''}</nav><header class="recordings-heading"><div><h1>${folder?esc(folder.name):'Semua rekaman'}</h1><p>${folder?'':(state.folders||[]).length+' folder · '}${items.length} rekaman di sini · ${recordingMB(items.reduce((n,m)=>n+(m.audioBytes||0),0))} audio</p></div><div class="toolbar-group"><button class="button" data-action="import-audio">${icon('upload')} Impor audio</button>${folder?'':'<button class="button accent" data-action="new-folder">+ Folder baru</button>'}</div></header>${folder?`<div class="library-folder-actions"><button class="button" data-action="open-folder" data-id="all">← Kembali</button><button class="button" data-action="rename-folder" data-id="${esc(folder.id)}">Ubah nama folder</button><button class="button danger" data-action="delete-folder" data-id="${esc(folder.id)}">${icon('trash')} Hapus folder</button></div>`:''}<div class="recordings-filters"><label>Cari di ${folder?'folder ini':'halaman utama'}<input id="library-search" type="search" placeholder="${folder?'Cari judul rekaman':'Cari folder atau rekaman di sini'}" value="${esc(state.librarySearch||'')}"></label></div><div id="library-folders">${libraryFolderCards()}</div><section class="library-files"><h2>Rekaman</h2><div id="library-recordings">${recordingCards()}</div></section></section>`;
}
document.addEventListener('input',e=>{if(e.target.id==='library-search'){
 state.librarySearch=e.target.value;
 document.getElementById('library-folders').innerHTML=libraryFolderCards();
 document.getElementById('library-recordings').innerHTML=recordingCards();
}});

/* A single import route for picker and external file drops. */
'use strict';
const audioImport={file:null,busy:false,depth:0,returnFocus:null,model:null,gpu:null};
function importAllowed(){
 if(state.recording||state.recordStarting||engine.state!=='idle'){toast('Selesaikan rekaman sebelum mengimpor audio.');return false;}
 return true;
}
function showAudioImport(file=null){
 if(audioImport.busy||!importAllowed())return;
 if(!document.getElementById('audio-import-dialog'))audioImport.returnFocus=document.activeElement;
 if(file)audioImport.file=file;
 audioImport.model=audioImport.model||state.settings.model;
 audioImport.gpu=audioImport.gpu||(engine.preferCuda?'Otomatis':'CPU');
 const chosen=audioImport.file;
 let root=document.getElementById('audio-import-root');
 if(!root){root=document.createElement('div');root.id='audio-import-root';document.body.appendChild(root);}
 root.innerHTML=`<div class="import-backdrop"><section class="import-dialog" id="audio-import-dialog" role="dialog" aria-modal="true" aria-labelledby="import-title"><button class="import-close" data-import="close" aria-label="Tutup">×</button><div class="import-heading-icon">${icon('upload')}</div><h2 id="import-title">Impor audio</h2><p class="import-subtitle">Bawa percakapan Anda menjadi catatan.</p>${chosen?`<div class="import-file"><span>${icon('headphones')}</span><div><strong>${esc(chosen.name)}</strong><small>${(chosen.size/1048576).toLocaleString('id-ID',{maximumFractionDigits:2})} MB · ${esc(chosen.name.split('.').pop().toUpperCase())}</small></div><span class="import-check">${icon('check')}</span></div><button class="button import-choose" data-import="choose">Pilih file lain</button>`:`<button class="import-dropzone" data-import="choose"><span class="import-upload-icon">${icon('upload')}</span><strong>Tarik file audio ke sini</strong><span>atau klik untuk memilih file</span><small>MP3, OGG, OPUS, M4A, WAV, FLAC dan lainnya · Maksimal 1 GB</small></button>`}<div class="import-options"><label for="import-folder">Simpan di folder</label><select id="import-folder">${folderOptions(state.folderFilter==='all'?'':state.folderFilter||'')}</select><label for="import-model">Model transkripsi</label><select id="import-model">${models.map(m=>`<option value="${esc(m.id)}" ${m.id===audioImport.model?'selected':''}>${esc(m.name)} · ${esc(m.size)} · ${esc(m.speed)}</option>`).join('')}</select><label for="import-gpu">Pemrosesan</label><select id="import-gpu"><option value="Otomatis" ${audioImport.gpu==='Otomatis'?'selected':''}>Otomatis (GPU jika tersedia)</option><option value="CPU" ${audioImport.gpu==='CPU'?'selected':''}>CPU</option></select><label class="import-auto"><input id="import-transcribe" type="checkbox" checked> Langsung transkripsikan setelah impor</label><small>Model yang belum tersedia akan diunduh saat transkripsi pertama.</small></div><p id="import-feedback" class="import-feedback" role="status" aria-live="polite">${chosen?'Siap diimpor.':'Audio diproses di komputer Anda.'}</p><progress id="import-progress" hidden aria-label="Progres impor"></progress><div class="import-actions"><button class="button" data-import="close">Batal</button><button class="button accent" data-import="submit" ${chosen?'':'disabled'}>${icon('upload')} Impor audio</button></div></section></div>`;
 root.querySelector('#import-model').addEventListener('change',e=>audioImport.model=e.target.value);
 root.querySelector('#import-gpu').addEventListener('change',e=>audioImport.gpu=e.target.value);
 root.querySelector(chosen?'[data-import="submit"]':'[data-import="choose"]').focus();
}
function closeAudioImport(){
 if(audioImport.busy)return;
 document.getElementById('audio-import-root')?.remove();audioImport.file=null;audioImport.model=null;audioImport.gpu=null;
 if(audioImport.returnFocus?.isConnected)audioImport.returnFocus.focus();
}
function selectImportFiles(files){
 if(audioImport.busy||!importAllowed())return;
 if(files.length!==1)return toast('Pilih satu file audio untuk setiap impor.');
 const file=files[0];
 if(!/\.(wav|wave|flac|mp3|mp2|ogg|oga|opus|m4a|m4b|aac|aiff|aif|aifc|wma|amr|ac3|eac3|au|snd|caf|ape|wv|mka|webm|mp4|mkv|mov|avi|3gp|3g2|ts|mts|m2ts|mpeg|mpg|weba)$/i.test(file.name))return toast('Format belum didukung. Pilih file audio atau video yang tercantum di pemilih file.');
 if(!file.size||file.size>1024**3)return toast('Ukuran audio harus lebih dari 0 dan maksimal 1 GB.');
 showAudioImport(file);
}
function setImportBusy(busy){
 audioImport.busy=busy;
 const root=document.getElementById('audio-import-dialog');if(!root)return;
 root.setAttribute('aria-busy',String(busy));
 root.querySelectorAll('button,select,input').forEach(button=>button.disabled=busy);
 root.querySelector('#import-progress').hidden=!busy;
 root.querySelector('[data-import="submit"]').innerHTML=busy?'Mengimpor…':icon('upload')+' Impor audio';
}
async function submitAudioImport(){
 if(audioImport.busy||!audioImport.file||!importAllowed())return;
 const feedback=document.getElementById('import-feedback');
 if(!engine.ready){feedback.textContent='Mesin belum siap. Tunggu sebentar lalu coba lagi.';return;}
 setImportBusy(true);feedback.textContent='Mengimpor audio ke catatan Anda…';
 try{
  const file=audioImport.file;
  const model=choiceIds[audioImport.model],gpu=audioImport.gpu;
  const transcribe=document.getElementById('import-transcribe').checked;
  const response=await fetch(engine.base+'/import',{method:'POST',headers:{Authorization:'Bearer '+engine.token,'X-File-Name':encodeURIComponent(file.name),'X-Folder-Id':document.getElementById('import-folder').value},body:file});
  const result=await response.json();if(!response.ok)throw Error(result.error||'Impor gagal.');
  // The import is committed: close before refreshing so a refresh error cannot
  // invite a duplicate upload. Existing records remain accessible in history.
  setImportBusy(false);closeAudioImport();state.noteTab='transcript';
  try{await loadSessions();await loadSession(result.session);toast('Audio berhasil diimpor.');}
  catch(error){toast('Audio sudah diimpor. Buka kembali Catatan Rekaman untuk memuatnya.');}
  if(transcribe){
   try{const job=await api('/reprocess',{session:result.session,model,gpu});engine.transcribeJob=job.job;toast('Menyiapkan model dan mentranskripsikan audio…');}
   catch(error){toast('Audio tersimpan, tetapi transkripsi belum dimulai: '+String(error.message||error));}
  }
 }catch(error){setImportBusy(false);feedback.textContent=String(error.message||error);}
}
function hideAudioDrop(){audioImport.depth=0;document.getElementById('audio-drop-overlay')?.remove();}
function showAudioDrop(){
 if(audioImport.busy||document.getElementById('setup-wizard')||document.getElementById('audio-drop-overlay'))return;
 const root=document.createElement('div');root.id='audio-drop-overlay';root.setAttribute('role','status');
 root.innerHTML=`<div class="audio-drop-card"><div class="drop-orbit">${icon('upload')}</div><h2>Lepaskan audio untuk mengimpor</h2><p>MP3, OGG, OPUS, M4A, WAV, FLAC dan lainnya · Maksimal 1 GB</p><span>File Anda tetap diproses secara lokal</span></div>`;
 document.body.appendChild(root);
}
const isFileDrag=e=>Array.from(e.dataTransfer?.types||[]).includes('Files');
document.addEventListener('dragenter',e=>{if(!isFileDrag(e))return;e.preventDefault();audioImport.depth++;showAudioDrop();});
document.addEventListener('dragover',e=>{if(!isFileDrag(e))return;e.preventDefault();e.dataTransfer.dropEffect=audioImport.busy?'none':'copy';showAudioDrop();});
document.addEventListener('dragleave',e=>{if(!isFileDrag(e)&&!audioImport.depth)return;audioImport.depth=Math.max(0,audioImport.depth-1);if(!audioImport.depth)hideAudioDrop();});
document.addEventListener('drop',e=>{if(!isFileDrag(e)&&!audioImport.depth)return;e.preventDefault();hideAudioDrop();if(document.getElementById('setup-wizard'))return;selectImportFiles(Array.from(e.dataTransfer?.files||[]));});
window.addEventListener('blur',hideAudioDrop);
document.addEventListener('dragend',hideAudioDrop);
document.getElementById('audio-file').addEventListener('change',e=>{const files=Array.from(e.target.files||[]);e.target.value='';if(files.length)selectImportFiles(files);});
document.addEventListener('click',e=>{
 const button=e.target.closest('[data-import]');if(!button)return;
 switch(button.dataset.import){case 'choose':document.getElementById('audio-file').click();break;case 'close':closeAudioImport();break;case 'submit':submitAudioImport();break;}
});
document.addEventListener('keydown',e=>{
 if(e.key==='Escape'){hideAudioDrop();closeAudioImport();}
 const dialog=document.getElementById('audio-import-dialog');if(!dialog||e.key!=='Tab')return;
 const items=[...dialog.querySelectorAll('button:not(:disabled),select:not(:disabled),input:not(:disabled)')];
 if(!items.length){e.preventDefault();return;}
 const first=items[0],last=items[items.length-1];
 if(e.shiftKey&&document.activeElement===first){e.preventDefault();last.focus();}
 else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first.focus();}
});

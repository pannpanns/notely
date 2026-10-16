/* Recording controls and resizable navigation. Spectrum values come from PCM FFT. */
const sidebarLayout={collapsed:safeStore.get('notelySidebarCollapsed')===true,width:Math.min(380,Math.max(220,Number(safeStore.get('notelySidebarWidth'))||244))};
let dockScale=1;
const dockAudio={spectra:[],status:'idle',elapsed:0};
function applySidebarLayout(){
 const app=document.querySelector('.app');if(!app)return;
 app.classList.toggle('sidebar-collapsed',sidebarLayout.collapsed);
 app.style.setProperty('--sidebar-expanded',sidebarLayout.width+'px');
 const toggle=document.getElementById('sidebar-toggle');
 if(toggle){toggle.setAttribute('aria-expanded',String(!sidebarLayout.collapsed));toggle.title=sidebarLayout.collapsed?'Besarkan sidebar':'Kecilkan sidebar';toggle.setAttribute('aria-label',toggle.title);}
 const grip=document.getElementById('sidebar-resize');if(grip)grip.setAttribute('aria-valuenow',String(sidebarLayout.width));
}
function installRecordingShell(){
 const sidebar=document.querySelector('.sidebar');const workspace=document.querySelector('.workspace');if(!sidebar||!workspace)return;
 // Preserve accessible labels when text is visually collapsed.
 for(const el of sidebar.querySelectorAll('button')){if(!el.title)el.title=el.textContent.trim();if(!el.getAttribute('aria-label'))el.setAttribute('aria-label',el.title);}
 if(!document.getElementById('sidebar-toggle')){
  const toggle=document.createElement('button');toggle.id='sidebar-toggle';toggle.className='sidebar-toggle';toggle.dataset.action='toggle-sidebar';toggle.setAttribute('aria-controls','notely-sidebar');toggle.innerHTML='<svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="3"></rect><path d="M9 4v16M13 9l3 3-3 3"></path></svg>';
  document.querySelector('.app-strip')?.prepend(toggle);
 }
 sidebar.id='notely-sidebar';
 if(!document.getElementById('sidebar-resize')){
  const grip=document.createElement('div');grip.id='sidebar-resize';grip.className='sidebar-resize';grip.tabIndex=0;grip.setAttribute('role','separator');grip.setAttribute('aria-label','Ubah lebar sidebar');grip.setAttribute('aria-orientation','vertical');grip.setAttribute('aria-valuemin','220');grip.setAttribute('aria-valuemax','380');sidebar.appendChild(grip);
 }
 if(!document.getElementById('recording-dock')){
  const dock=document.createElement('div');dock.id='recording-dock';dock.className='recording-dock';dock.setAttribute('role','group');dock.setAttribute('aria-label','Kontrol rekaman');
  dock.innerHTML=`<button id="dock-record" class="dock-record" data-action="ask-demo" aria-label="Mulai rekaman">${icon('mic')}</button><div class="dock-spectrum" aria-label="Spektrum audio rekaman" role="img">${Array.from({length:18},()=>'<i style="height:3px"></i>').join('')}</div><span class="dock-time" id="dock-time"></span><button class="dock-more" data-action="recording-menu" aria-label="Pilihan rekaman">${icon('more')}</button>`;
  workspace.appendChild(dock);
 }
 applySidebarLayout();applyDockScale();paintRecordingDock();
}
function updateRecordingDock(spectra,status,elapsed){
 dockAudio.spectra=Array.isArray(spectra)?spectra:[];dockAudio.status=status;dockAudio.elapsed=Number(elapsed)||0;paintRecordingDock();
}
function spectrumLevels(spectra,now,active){
 const current=active?spectra.filter(s=>Number.isFinite(s.sampledAt)&&now-s.sampledAt<1.5&&now-s.sampledAt>=-2):[];
 return Array.from({length:18},(_,i)=>Math.max(0,...current.map(s=>Math.min(1,Math.max(0,Number(s.bands?.[i])||0)))));
}
function paintRecordingDock(){
 const dock=document.getElementById('recording-dock');if(!dock)return;
 const stopping=engine.state==='stopping';const active=state.recording||engine.state==='recording';
 const record=document.getElementById('dock-record');const action=active?'stop-demo':'ask-demo';
 if(record.dataset.action!==action||record.dataset.drawn!=='yes'){record.dataset.action=action;record.innerHTML=icon(active?'stop':'mic');record.dataset.drawn='yes';}
 record.disabled=stopping;
 record.setAttribute('aria-label',stopping?'Menyimpan rekaman':active?'Hentikan dan simpan rekaman':'Mulai rekaman');record.title=record.getAttribute('aria-label');
 dock.classList.toggle('is-recording',active);dock.classList.toggle('is-stopping',stopping);
 document.getElementById('dock-time').textContent=stopping?'Menyimpan…':active?ts(dockAudio.elapsed||state.recordSeconds||0):'';
 document.getElementById('dock-time').hidden=!(active||stopping);
 const values=spectrumLevels(dockAudio.spectra,Date.now()/1000,active);
 dock.querySelectorAll('.dock-spectrum i').forEach((bar,i)=>{bar.style.height=(3+values[i]*18)+'px';});
 dock.querySelector('.dock-spectrum').setAttribute('aria-label',active?'Spektrum audio komputer dan mikrofon yang direkam':'Spektrum audio, belum merekam');
}
const beforeDockRender=render;
render=function(){beforeDockRender();installRecordingShell();};
const beforeDockRefresh=refreshPage;
refreshPage=function(){beforeDockRefresh();installRecordingShell();};
const beforeDockClick=handleClick;
handleClick=function(action,id){
 if(action==='toggle-sidebar'){
  sidebarLayout.collapsed=!sidebarLayout.collapsed;safeStore.set('notelySidebarCollapsed',sidebarLayout.collapsed);applySidebarLayout();return;
 }
 if(action==='recording-menu')return modal('Pilihan rekaman','Pilih tindakan untuk rekaman Anda.',[['Impor audio','dock-import',''],['Pengaturan rekaman','dock-settings',''],['Tutup','close-modal','']]);
 if(action==='dock-import'){closeModal();return beforeDockClick('import-audio');}
 if(action==='dock-settings'){closeModal();state.page='settings';state.tab='recordings';return render();}
 return beforeDockClick(action,id);
};
let sidebarDrag=null;
function setSidebarWidth(width){sidebarLayout.width=Math.round(Math.max(220,Math.min(380,width)));applySidebarLayout();}
document.addEventListener('pointerdown',e=>{
 if(e.target.id!=='sidebar-resize'||sidebarLayout.collapsed)return;
 sidebarDrag={x:e.clientX,width:sidebarLayout.width,target:e.target};e.target.setPointerCapture?.(e.pointerId);document.body.classList.add('resizing-sidebar');e.preventDefault();
});
document.addEventListener('pointermove',e=>{if(sidebarDrag)setSidebarWidth(sidebarDrag.width+e.clientX-sidebarDrag.x);});
function finishSidebarDrag(){if(!sidebarDrag)return;sidebarDrag=null;document.body.classList.remove('resizing-sidebar');safeStore.set('notelySidebarWidth',sidebarLayout.width);}
document.addEventListener('pointerup',finishSidebarDrag);document.addEventListener('pointercancel',finishSidebarDrag);
window.addEventListener('blur',finishSidebarDrag);
document.addEventListener('keydown',e=>{if(e.target.id!=='sidebar-resize')return;if(['ArrowLeft','ArrowRight','Home','End'].includes(e.key)){e.preventDefault();setSidebarWidth(e.key==='Home'?220:e.key==='End'?380:sidebarLayout.width+(e.key==='ArrowLeft'?-10:10));safeStore.set('notelySidebarWidth',sidebarLayout.width);}});
setInterval(paintRecordingDock,250);
render();

function applyDockScale(){
 const dock=document.getElementById('recording-dock');if(!dock)return;
 const summary=state.page==='notes'&&state.noteTab==='summary';
 const hideDock=summary||state.page==='settings';
 dock.hidden=hideDock;
 if(hideDock){document.querySelector('.page')?.style.setProperty('padding-bottom','0px');return;}
 const width=dock.parentElement.clientWidth;
 const scale=width?Math.min(dockScale,Math.max(.4,(width-28)/dock.offsetWidth)):dockScale;
 dock.style.setProperty('--dock-scale',scale);
 document.querySelector('.page')?.style.setProperty('padding-bottom',Math.ceil(dock.offsetHeight*scale+20)+'px');
 const value=document.getElementById('dock-size-value');if(value)value.textContent=Math.round(dockScale*100)+'%';
}
document.addEventListener('input',e=>{
 if(e.target.id!=='dock-size')return;
 dockScale=Math.min(1.4,Math.max(.7,Number(e.target.value)/100||1));
 safeStore.set('notelyDockScale',dockScale);applyDockScale();
});
window.addEventListener('resize',applyDockScale);

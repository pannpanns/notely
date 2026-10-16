const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const {JSDOM}=require('jsdom');
const root=path.resolve(__dirname,'../frontend');
const dom=new JSDOM(fs.readFileSync(path.join(root,'index.html'),'utf8'),{url:'http://localhost',runScripts:'outside-only'});
const w=dom.window;w.setInterval=()=>0;w.setTimeout=()=>0;
const run=s=>new vm.Script(s).runInContext(dom.getInternalVMContext());
for(const name of ['ui.js','summary-format.js','desktop.js','modern.js','summary-stream.js','recording-history.js','recording-dock.js','audio-import.js'])run(fs.readFileSync(path.join(root,name),'utf8'));
const file=new w.File(['RIFFtest'],'test.wav',{type:'audio/wav'});
function drag(type,files=[file]){const event=new w.Event(type,{bubbles:true,cancelable:true});Object.defineProperty(event,'dataTransfer',{value:{types:['Files'],files,dropEffect:'none'}});w.document.dispatchEvent(event);return event;}
(async()=>{
 run("state.page='notes';state.noteTab='summary';render();");
 assert.ok(drag('dragenter').defaultPrevented);
 assert.ok(w.document.querySelector('#audio-drop-overlay'));
 drag('dragenter');drag('dragleave');assert.ok(w.document.querySelector('#audio-drop-overlay'),'nested enter/leave does not flicker');
 drag('drop');assert.equal(w.document.querySelector('#audio-drop-overlay'),null);
 assert.ok(w.document.querySelector('.import-file').textContent.includes('test.wav'));
 assert.equal(w.document.querySelector('#audio-import-dialog').getAttribute('aria-modal'),'true');
 assert.equal(w.document.querySelectorAll('#import-model option').length,6);
 run('closeAudioImport()');
 for(const ext of ['mp3','ogg','opus','m4a','aac','wma','aiff','webm','mp4']){w.sample=new w.File(['audio'],'sample.'+ext);run('selectImportFiles([window.sample])');assert.ok(w.document.querySelector('.import-file'));run('closeAudioImport()');}

 w.bad=new w.File(['x'],'bad.exe');run('selectImportFiles([window.bad])');
 assert.equal(w.document.querySelector('#audio-import-dialog'),null);
 w.good=file;run('selectImportFiles([window.good,window.good])');
 assert.equal(w.document.querySelector('#audio-import-dialog'),null);
 run("engine.ready=true;selectImportFiles([window.good]);window.uploads=0;fetch=async()=>{window.uploads++;return {ok:false,json:async()=>({error:'Try again'})};};");
 await run('submitAudioImport()');
 assert.ok(w.document.querySelector('#import-feedback').textContent.includes('Try again'));
 assert.equal(run('audioImport.busy'),false);
 run("api=async(path,payload)=>{window.transcribeRequest={path,payload};return {job:'transcribe-1'};};fetch=async()=>{window.uploads++;return {ok:true,json:async()=>({session:'imported'})};};loadSessions=async()=>{};loadSession=async id=>{window.opened=id;};");
 await Promise.all([run('submitAudioImport()'),run('submitAudioImport()')]);
 assert.equal(w.uploads,2,'one failed attempt and one success, no double submission');
 assert.equal(w.transcribeRequest.path,'/reprocess');assert.equal(w.transcribeRequest.payload.session,'imported');
 assert.equal(w.opened,'imported');assert.equal(run('state.noteTab'),'transcript');
 assert.equal(w.document.querySelector('#audio-import-dialog'),null);
 run("state.folders=[{id:'ipa',name:'IPA'},{id:'math',name:'Matematika'}];meetings.splice(0,meetings.length,{id:'one',folder:'ipa',title:'Biologi',when:'today',rows:[]},{id:'two',folder:'math',title:'Aljabar',when:'today',rows:[]});state.folderFilter='ipa';render();");
 assert.ok(w.document.querySelector('#meeting-list').textContent.includes('Biologi'));
 assert.ok(w.document.querySelector('#meeting-list').textContent.includes('Aljabar'),'sidebar history is no longer filtered by the removed folder control');
 run("handleClick('rename-recording','one')");
 assert.equal(w.document.querySelector('#organize-value').value,'Biologi');
 run("closeModal();handleClick('move-recording','one')");
 assert.equal(w.document.querySelector('#organize-value').value,'ipa');
 assert.equal(w.document.querySelectorAll('#organize-value option').length,3);
 run('closeModal()');
 run("state.recording=true;showAudioImport()");assert.equal(w.document.querySelector('#audio-import-dialog'),null);
 console.log('PASS: nested drag/drop, type/count validation, import retry, duplicate guard and transcript navigation');
 dom.window.close();
})().catch(e=>{console.error(e);dom.window.close();process.exitCode=1;});

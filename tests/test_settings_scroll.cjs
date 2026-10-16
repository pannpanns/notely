const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const {JSDOM}=require('jsdom');
const root=path.resolve(__dirname,'../frontend'),html=fs.readFileSync(path.join(root,'index.html'),'utf8');
const dom=new JSDOM(html,{url:'http://localhost',runScripts:'outside-only'}),w=dom.window;
w.setInterval=()=>0;w.setTimeout=()=>0;
const run=s=>new vm.Script(s).runInContext(dom.getInternalVMContext());
for(const [,name] of html.matchAll(/<script src="([^"]+)"/g))run(fs.readFileSync(path.join(root,name),'utf8'));
run("state.page='settings';state.tab='summary';state.settings.provider='gemini';state.settings.summaryModel='gemini-2.5-flash';render();");
function scroll(){return w.document.querySelector('.settings-scroll');}
scroll().scrollTop=620;w.document.querySelector('.api-guide').open=false;
const model=w.document.querySelector('#cloud-model');model.focus();model.value='gemini-2.5-pro';model.dispatchEvent(new w.Event('change',{bubbles:true}));
assert.equal(scroll().scrollTop,620);assert.equal(w.document.activeElement.id,'cloud-model');
assert.equal(w.document.querySelector('#cloud-model').value,'gemini-2.5-pro');
assert.equal(run('state.settings.summaryModel'),'gemini-2.5-pro');
assert.equal(w.document.querySelector('.api-guide').open,false);
assert.equal(JSON.parse(w.localStorage.getItem('notelyDesktopSettings')).summaryModel,'gemini-2.5-pro');
// Provider changes also preserve position, including a custom model field appearing.
run("handleClick('choose-provider','groq')");assert.equal(scroll().scrollTop,620);
const custom=w.document.querySelector('#cloud-model');custom.focus();custom.value='custom';custom.dispatchEvent(new w.Event('change',{bubbles:true}));
assert.ok(w.document.querySelector('#custom-model'));assert.equal(scroll().scrollTop,620);
run("handleClick('choose-provider','gemini')");assert.equal(w.document.querySelector('#custom-model'),null);assert.equal(scroll().scrollTop,620);
run("handleClick('tab','general')");assert.equal(scroll().scrollTop,0,'a different tab starts at the top');
console.log('PASS: model/provider selection preserves scroll, focus and guide state; custom fields update; tab navigation resets scroll');dom.window.close();

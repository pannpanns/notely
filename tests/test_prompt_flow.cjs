const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const desktop=fs.readFileSync(__dirname+'/../frontend/desktop.js','utf8');
const modern=fs.readFileSync(__dirname+'/../frontend/modern.js','utf8');
const ui=fs.readFileSync(__dirname+'/../frontend/ui.js','utf8');
const m={id:'one',rows:[[9,'Hello']],instructions:'Ringkas ini'};
let open=false,focused=false,calls=[],resolveRequest,handler;
const ctx={state:{settings:{provider:'gemini',summaryModel:'test',summaryEconomy:true},apiKey:'key'},engine:{state:'idle'},getMeeting:()=>m,
 providers:{gemini:{name:'Gemini'}},document:{querySelector:()=>open?{focus:()=>focused=true}:null,addEventListener:(type,fn)=>handler=fn},
 modal:()=>open=true,closeModal:()=>open=false,toast:()=>{},refreshPage:()=>{},alertError:()=>{},Date,
 api:(route,payload)=>{calls.push({route,payload});return new Promise(r=>resolveRequest=r);}};
vm.createContext(ctx);
for(const [start,end] of [['async function generateRealSummary()','async function reprocessAudio()'],['async function confirmRealSummary()','/* Safe read-only']])vm.runInContext(desktop.slice(desktop.indexOf(start),desktop.indexOf(end)),ctx);
(async()=>{
 await ctx.generateRealSummary();assert.ok(open&&focused);
 await ctx.generateRealSummary();assert.equal(calls.length,0);
 vm.runInContext(modern.slice(modern.indexOf("document.addEventListener('keydown',e=>{",modern.indexOf('// Keep keyboard events')),modern.indexOf('const instructionView')),ctx);
 const event={key:'Enter',target:{closest:()=>null},preventDefault(){},stopImmediatePropagation(){}};
 // Supply the dialog methods used by the handler.
 ctx.document.querySelector=()=>open?{closest:()=>({})}:null;
 handler({...event,repeat:true});assert.equal(calls.length,0);
 handler(event);assert.equal(calls.length,1);assert.equal(calls[0].payload.instructions,'Ringkas ini');assert.equal(calls[0].payload.economy,undefined);
 await ctx.confirmRealSummary();assert.equal(calls.length,1,'duplicate submits blocked');
 resolveRequest({job:'j1'});await new Promise(setImmediate);
 assert.equal(m.instructions,'');assert.equal(ctx.state.summaryLoading,true);
 ctx.state.summaryLoading=false;m.instructions='Keep on failure';ctx.api=async()=>{throw Error('offline')};
 await ctx.confirmRealSummary();assert.equal(m.instructions,'Keep on failure');assert.equal(ctx.state.summarySubmitting,false);
 ctx.state.summaryLoading=false;ctx.api=()=>new Promise(r=>resolveRequest=r);
 const request=ctx.confirmRealSummary();m.instructions='New draft';resolveRequest({job:'j2'});await request;assert.equal(m.instructions,'New draft');
 ctx.esc=x=>x;ctx.ts=x=>'00:'+String(x).padStart(2,'0');
 vm.runInContext(ui.slice(ui.indexOf('function transcriptList('),ui.indexOf('function summaryPaper(')),ctx);
 assert.match(ctx.transcriptList(m),/\[00:09\]/);
 console.log('PASS: consent Enter/repeat, submission deduplication, success/failure/new drafts, timestamps');
})().catch(e=>{console.error(e);process.exitCode=1});

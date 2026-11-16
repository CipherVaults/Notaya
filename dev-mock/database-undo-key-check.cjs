'use strict';
// Run with one absolute, fresh throwaway output directory outside the repository. Never uses an existing browser profile.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto'),assert=require('node:assert/strict');
const {stripTypeScriptTypes}=require('node:module');
const REPO=path.resolve(__dirname,'..'),SOURCE=path.join(REPO,'src','features','database','undo.ts');
const CASES=[
 {name:'inline-owned',inline:true},{name:'fullpage-owned'},
 ...['input','textarea','select','dialog','editable','pm'].map(control=>({name:'exclude-'+control,inline:true,control,blocked:true})),
 {name:'page-editor-no-db',noDB:true,blocked:true},{name:'nested-wrong-db',inline:true,wrong:true,blocked:true},
 {name:'already-handled',preHandled:true,blocked:true},
 {name:'meta-z',metaKey:true,ctrlKey:false},{name:'ctrl-shift-z',shiftKey:true,redo:true},
 {name:'meta-shift-z',metaKey:true,ctrlKey:false,shiftKey:true,redo:true},{name:'ctrl-y',key:'y',redo:true},
 {name:'meta-y',key:'y',metaKey:true,ctrlKey:false,redo:true},{name:'uppercase-z',key:'Z'},
 {name:'plain-z',ctrlKey:false,blocked:true},{name:'alt-ctrl-z',altKey:true,blocked:true},
 {name:'alt-meta-z',ctrlKey:false,metaKey:true,altKey:true,blocked:true},
 {name:'ctrl-shift-y',key:'y',shiftKey:true,blocked:true},{name:'ctrl-a',key:'a',blocked:true},
 {name:'shift-z-no-modifier',ctrlKey:false,shiftKey:true,blocked:true},
];
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex').toUpperCase();
function noFollow(file){assert(path.isAbsolute(file)&&path.resolve(file)===file,'Canonical absolute path');let at=path.parse(file).root;for(const part of file.slice(at.length).split(path.sep).filter(Boolean)){at=path.join(at,part);assert(!fs.lstatSync(at).isSymbolicLink(),'No symlink descent');}}
function once(text,from,to){assert.equal(text.split(from).length,2,'Exactly one notification import');return text.replace(from,to);}
const warnings=[],network=[],pageErrors=[];process.on('warning',warning=>warnings.push({name:warning.name,code:warning.code??null,message:warning.message}));
let output=null,created=false,context=null,contextClosed=false,passed=false,failure=null,result=null;
(async()=>{try{
 assert.equal(typeof stripTypeScriptTypes,'function','Node with built-in stripTypeScriptTypes required');assert.equal(process.argv.length,3,'Pass one fresh throwaway output directory');
 output=process.argv[2];assert(path.isAbsolute(output)&&path.resolve(output)===output,'Absolute output required');const relative=path.relative(REPO,output);assert(relative.startsWith('..'+path.sep)||relative==='..'||path.isAbsolute(relative),'Throwaway output must be outside repository');
 noFollow(path.dirname(output));assert(!fs.existsSync(output),'Output must be a new leaf');fs.mkdirSync(output);created=true;
 for(const sub of ['profile','temp','home','home/AppData','home/AppData/Local','home/AppData/Roaming','downloads'])fs.mkdirSync(path.join(output,sub));
 noFollow(SOURCE);const stat=fs.lstatSync(SOURCE);assert(stat.isFile()&&stat.size>0&&stat.size<=16777216,'Bounded complete production source');const bytes=fs.readFileSync(SOURCE),sourceSHA256=hash(bytes);
 assert(bytes.equals(Buffer.from(bytes.toString('utf8'))),'Lossless source UTF8');fs.writeFileSync(path.join(output,'undo.input.ts'),bytes,{flag:'wx'});
 const stripped=stripTypeScriptTypes(bytes.toString('utf8'),{mode:'strip'});assert(stripped.length<=16777216,'Bounded full module');
 let module=once(stripped,'import { toast } from "sonner";',`const notifications=[];const toast=Object.assign(message=>notifications.push({kind:'info',message:String(message)}),{error:message=>notifications.push({kind:'error',message:String(message)})});`);
 module=once(module,'import { errorText } from "@/lib/ipc";','const errorText=error=>String(error);');
 module+='\nwindow.__undoModule={retainUndo,pushUndo,dropUndo,undoDepth,undoLast,redoLast,handleUndoKey};window.__undoNotifications=notifications;';
 fs.writeFileSync(path.join(output,'undo.module.js'),module,{flag:'wx'});
 const pw=require(process.env.NOTAYA_PLAYWRIGHT_MODULE||'playwright'),home=path.join(output,'home'),temp=path.join(output,'temp');
 const env={...process.env,TEMP:temp,TMP:temp,HOME:home,USERPROFILE:home,APPDATA:path.join(home,'AppData','Roaming'),LOCALAPPDATA:path.join(home,'AppData','Local')};
 const executablePath=process.env.NOTAYA_CHROMIUM_PATH||undefined;if(executablePath)noFollow(path.resolve(executablePath));
 context=await pw.chromium.launchPersistentContext(path.join(output,'profile'),{executablePath,headless:true,chromiumSandbox:true,timeout:10000,env,offline:true,serviceWorkers:'block',acceptDownloads:false,downloadsPath:path.join(output,'downloads')});
 context.setDefaultTimeout(5000);await context.route('**/*',async route=>{network.push({url:route.request().url(),method:route.request().method()});await route.abort('blockedbyclient');});
 const page=context.pages()[0]||await context.newPage();page.on('pageerror',error=>pageErrors.push(String(error)));
 await page.setContent('<!doctype html><html><head><meta charset="utf-8"></head><body></body></html>');await page.addScriptTag({type:'module',content:module});await page.waitForFunction(()=>!!window.__undoModule);
  const tests=await page.evaluate(async cases=>{
   const m=window.__undoModule,notifications=window.__undoNotifications,tests=[];
   const exported=['dropUndo','handleUndoKey','pushUndo','redoLast','retainUndo','undoDepth','undoLast'];
   if(JSON.stringify(Object.keys(m).sort())!==JSON.stringify(exported)||!exported.every(k=>typeof m[k]==='function'))throw new Error('Full exported production module required');
   for(const [ordinal,spec]of cases.entries()){
    const id='real-dom-'+ordinal,effects=[],facts={};let value=1,check='setup',failure=null,release=null,host=null,route=null,fixture=null;
    const equal=(a,b,label)=>{check=label;if(JSON.stringify(a)!==JSON.stringify(b))throw new Error(label+': '+JSON.stringify({actual:a,expected:b}));};
    try{
     fixture=document.createElement('section');document.body.append(fixture);
     const node=(tag,attrs,parent)=>{const el=document.createElement(tag);for(const[k,v]of Object.entries(attrs||{}))el.setAttribute(k,v);parent.append(el);return el;};
     let parent=fixture;if(spec.inline||spec.noDB){parent=node('div',{class:'ProseMirror',contenteditable:'true'},parent);if(spec.inline)parent=node('div',{'data-database-block':'',contenteditable:'false'},parent);}
     const db=spec.noDB?null:node('div',{'data-database':id},parent);host=db||fixture;parent=db||parent;if(spec.wrong)parent=node('div',{'data-database':'other'},parent);
     parent=node('table',{},parent);parent=node('tbody',{},parent);parent=node('tr',{},parent);let target=node('td',{role:'gridcell',tabindex:'0'},parent);
     if(['input','textarea','select'].includes(spec.control)){target=node(spec.control,{},target);if(spec.control==='select')node('option',{value:'one'},target).textContent='one';}
     if(spec.control==='dialog'){parent=node('dialog',{open:'',role:'dialog'},target);target=node('button',{},parent);}
     if(spec.control==='editable'||spec.control==='pm'){parent=node('div',spec.control==='editable'?{contenteditable:'true'}:{class:'ProseMirror',contenteditable:'false'},target);target=node('span',{tabindex:'0'},parent);}
     target.textContent=target.tagName==='SELECT'?target.textContent:'owned test control';target.focus();equal(document.activeElement===target,true,'real-dom-focus');
     release=m.retainUndo(id);m.pushUndo(id,{label:spec.name,undo:async()=>{value=0;effects.push('undo');},redo:async()=>{value=1;effects.push('redo');}});
     if(spec.redo)await m.undoLast(id);const before=effects.length,beforeValue=value,beforeDepth=m.undoDepth(id),returns=[];
     route=event=>returns.push(m.handleUndoKey(id,event));if(spec.preHandled)host.addEventListener('keydown',event=>event.preventDefault(),{capture:true});host.addEventListener('keydown',route);document.addEventListener('keydown',route);
     const event=new KeyboardEvent('keydown',{key:spec.key||'z',ctrlKey:spec.ctrlKey!==false,metaKey:!!spec.metaKey,altKey:!!spec.altKey,shiftKey:!!spec.shiftKey,bubbles:true,cancelable:true});target.dispatchEvent(event);await Promise.resolve();await Promise.resolve();
     Object.assign(facts,{focusTag:target.tagName,isContentEditable:target.isContentEditable,nearestDatabase:target.closest('[data-database]')?.getAttribute('data-database')??null,insideOuterPM:!!target.closest('.ProseMirror'),effects:effects.slice(before),value,depthBefore:beforeDepth,depthAfter:m.undoDepth(id),defaultPrevented:event.defaultPrevented,returns});
     equal(facts.effects,spec.blocked?[]:[spec.redo?'redo':'undo'],'effects');equal(value,spec.blocked?beforeValue:(spec.redo?1:0),'value');
     equal(facts.depthAfter,spec.blocked?beforeDepth:(spec.redo?{undo:1,redo:0}:{undo:0,redo:1}),'production-stack');
     equal(event.defaultPrevented,!!spec.preHandled||!spec.blocked,'native-default-prevented');equal(returns.filter(Boolean).length,spec.blocked?0:1,'one-owning-handler');
     equal(notifications.filter(n=>n.kind==='error').length,0,'no-production-error-notification');
    }catch(error){failure={check,message:String(error.stack||error)};}finally{if(route){host.removeEventListener('keydown',route);document.removeEventListener('keydown',route);}if(release)release();fixture?.remove();}
    tests.push({name:spec.name,passed:failure===null,failure,facts});
   }return tests;
  },CASES);
 result={source:SOURCE,sourceSHA256,sourceBytes:bytes.length,strippedModuleSHA256:hash(Buffer.from(module)),fullSourceModule:true,notificationImportsOnlyStubbed:true,tests};
 fs.writeFileSync(path.join(output,'results.json'),JSON.stringify(result,null,2)+'\n',{flag:'wx'});
 assert.equal(CASES.length,23,'Exactly23 cases declared');assert.equal(new Set(CASES.map(test=>test.name)).size,23,'Unique case names');
 assert.equal(tests.length,23,'All23 cases must run');assert.deepEqual(tests.map(test=>test.name),CASES.map(test=>test.name),'Every declared case completed in order');
 assert(tests.every(test=>test.passed===true),'Every production regression case must pass');assert.equal(network.length,0,'No network requests');assert.equal(pageErrors.length,0,'No uncaught browser errors');passed=true;
}catch(error){failure=String(error.stack||error);}finally{
 if(context){let timer;try{await Promise.race([context.close(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(new Error('Owned browser close deadline')),5000);})]);contextClosed=true;}catch(error){passed=false;failure=[failure,String(error.stack||error)].filter(Boolean).join('\n');}finally{clearTimeout(timer);}}
 const summary={scope:'database-undo-key-check-v1',passed:passed&&contextClosed,failure,result,warnings,network,pageErrors,contextClosed,profileRetained:true,syntheticRealDOMEvents:true,fullApplicationOrPersistenceQualified:false};
 if(created)fs.writeFileSync(path.join(output,'summary.json'),JSON.stringify(summary,null,2)+'\n',{flag:'wx'});process.stdout.write(JSON.stringify(summary)+'\n');process.exitCode=summary.passed?0:1;
}})();

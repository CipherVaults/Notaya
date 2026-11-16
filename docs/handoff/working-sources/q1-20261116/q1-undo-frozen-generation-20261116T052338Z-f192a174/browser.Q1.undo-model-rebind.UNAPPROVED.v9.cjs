'use strict';
// SOURCE ONLY. One borrowed real-inline mock observation; no product/native/history bypass.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const OBSERVER='EAB97DF82941A1F093A8735DFA3EC465CA440D1CA8D8751D995E6DAFF09374FF';
const ORIGINAL='C082BA9E89B9D3F10E3B0A31E7AA49E1F2F9E1096B164E7C3943B0DFF9327A5D';
const URL='http://127.0.0.1:15317/?s=app&db=1&rows=5000&cover=none&lastKind=page&lastId=p1';
const TITLE='Q1 compact stable row 5001';
const COMMANDS=['vault_status','get_settings','list_notes','list_stickies','list_bin','take_notice','set_backdrop','window_ready','due_reminders_now','ai_get_status','get_database','query_view','save_database','set_cells','editing','row_titles','set_last_place','get_page','save_page','create_row','trash_page','restore_page','history_snapshot'];
const PHASES=['open-real-existing-note','insert-linked-view-through-real-slash-picker','append-new-row-at-end','naturally-name-row-and-enter','first-undo-name-routing','second-undo-row-disappearance','first-redo-same-row','second-redo-same-name','final-visible-readback-after-chain'];
function canonical(value){if(value===undefined)return '["undefined"]';if(value===null)return '["null"]';if(typeof value==='number'){if(!Number.isFinite(value))throw new Error('Finite stable-domain number');return Object.is(value,-0)?'["negative-zero"]':'["number",'+JSON.stringify(value)+']';}if(typeof value==='string'||typeof value==='boolean')return '['+JSON.stringify(typeof value)+','+JSON.stringify(value)+']';if(Array.isArray(value))return '["array",['+value.map(canonical).join(',')+']]';if(value&&typeof value==='object')return '["object",['+Object.keys(value).map(k=>'['+JSON.stringify(k)+','+canonical(value[k])+']').join(',')+']]';throw new Error('Unsupported stable-domain value');}
function encodeData(value,depth=0,seen=new Set(),budget={nodes:0}){if(++budget.nodes>1048576||depth>32)throw new Error('Bounded baseline ordered encoding');if(value===undefined)return{t:'undefined'};if(value===null||typeof value==='string'||typeof value==='boolean')return{t:typeof value,v:value};if(typeof value==='number'){if(!Number.isFinite(value))throw new Error('Finite encoded number');return Object.is(value,-0)?{t:'negative-zero'}:{t:'number',v:value};}if(typeof value!=='object'||seen.has(value))throw new Error('Unsupported/cyclic baseline data');const proto=Object.getPrototypeOf(value);if(proto!==Object.prototype&&proto!==null&&!Array.isArray(value))throw new Error('Plain baseline object');seen.add(value);try{const descriptors=Object.getOwnPropertyDescriptors(value),entries=[];for(const key of Reflect.ownKeys(descriptors)){if(typeof key!=='string')throw new Error('Symbol baseline key');const a=descriptors[key];if(!Object.hasOwn(a,'value')||a.get||a.set)throw new Error('Accessor baseline key');entries.push([key,encodeData(a.value,depth+1,seen,budget)]);}return{t:Array.isArray(value)?'array':'object',e:entries};}finally{seen.delete(value);}}
function semantic(value){if(Array.isArray(value))return '['+value.map(semantic).join(',')+']';if(value&&typeof value==='object')return '{'+Object.keys(value).sort().map(k=>JSON.stringify(k)+':'+semantic(value[k])).join(',')+'}';return JSON.stringify(value);}
function projectCaptured(s) {
 const omit=(v,keys)=>Object.fromEntries(Object.entries(v).filter(([k])=>!keys.includes(k)));
 const dbs=s.databaseItems.filter(x=>x.kind==='database'),rows=s.databaseItems.filter(x=>x.kind==='row');
 const tasks=dbs.filter(d=>d.title==='Tasks'),projects=dbs.filter(d=>d.title==='Projects');
 if(tasks.length!==1||projects.length!==1)throw new Error('One exact synthetic Tasks/Projects database');
 const task=tasks[0],original=rows.filter(r=>r.id!==s.q1AddedId),newRows=s.q1AddedId?rows.filter(r=>r.id===s.q1AddedId):[];
 const p1=s.tree.pages.find(p=>p.id==='p1'),saved=s.savedPages.filter(([id])=>id==='p1');
 return {stable:{
  coreFlags:s.coreFlags,shellPrefs:s.shellPrefs,settings:omit(s.settings,['lastPlace']),
  tree:{...s.tree,pages:s.tree.pages.map(p=>p.id==='p1'?omit(p,['preview','updated']):p)},stickies:s.stickies,pictures:s.pictures,
  binTimes:s.binTimes,searchHistory:s.searchHistory,calcHistory:s.calcHistory,
  databases:dbs.map(d=>d.id===task.id?omit(d,['views','updated','nextId','rev']):d),
  originalRows:original,savedNonP1:s.savedPages.filter(([id])=>id!=='p1'),pageBody:s.pageBody,
  aiState:s.ai.state,aiBridgeErrors:s.ai.bridgeErrors},
 mutable:{lastPlace:s.settings.lastPlace,p1,savedP1:saved.length?saved[0][1]:null,
  tasks:{id:task.id,views:task.views,updated:task.updated,nextId:task.nextId,defaultTemplate:task.defaultTemplate,templates:task.templates,storedWireRevision:{present:Object.hasOwn(task,'rev'),value:task.rev??null},keyOrder:Object.keys(task).filter(key=>key!=='kind')},
  newRow:newRows.length?newRows[0]:null,revs:s.revs,aiCalls:s.ai.calls},
 identities:{tasksId:task.id,projectsId:projects[0].id,projectsRows:rows.filter(r=>r.db===projects[0].id).map(r=>({id:r.id,title:r.title})),
 originalRowIDs:original.map(r=>r.id),originalRowCount:original.length,originalTasksCount:original.filter(r=>r.db===task.id).length,
 maxOriginalOrder:Math.max(...original.map(r=>r.order))}};
}
function installBridge(project,canon,encode,nonce,bindingSHA256) {
 const ack=(value,attempt)=>{if(!value||value.admitted!==true||value.attempt!==attempt||value.bindingSHA256!==bindingSHA256)throw new Error('Exact Q1 host admission acknowledgement');return value;};
 let baseline=null,baselinePromise=null,addedId=null,captureSequence=0;
 const digest=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('').toUpperCase();
 const encodeProjection=async captured=>{
  const copied=structuredClone(captured);if(addedId)copied.q1AddedId=addedId;
  const p=project(copied),domains={};const idsText=canon(p.identities.originalRowIDs);p.identities.originalOrderedIDsSHA256=await digest(idsText);delete p.identities.originalRowIDs;for(const [key,value]of Object.entries(p.stable)){const text=canon(value);if(new TextEncoder().encode(text).length>16777216)throw new Error('Q1 stable domain cap');domains[key]=await digest(text);}
  return {stableDigests:domains,mutable:p.mutable,identities:p.identities};
 };
 Object.defineProperty(window,'__notayaQ1Dispatch',{configurable:false,writable:false,value:async record=>{
  const captureOrdinal=++captureSequence;
  try {
   if(!baselinePromise){baseline=structuredClone(record.captured);baselinePromise=window.__notayaQ1HostLedger({scope:'q1-mock-dispatch-ledger-v1',runNonce:nonce,stage:'baseline',capturedEncoded:encode(baseline)}).then(value=>ack(value,0));}
   const initial=await baselinePromise;if(initial.admitted!==true)throw new Error('Q1 baseline not admitted');
   if(record.captured){
    const after=record.captured.databaseItems.filter(x=>x.kind==='row'),old=new Set(baseline.databaseItems.filter(x=>x.kind==='row').map(x=>x.id));
    const fresh=after.filter(x=>!old.has(x.id));if(fresh.length===1)addedId=fresh[0].id;
   }
   const projection=await encodeProjection(record.captured);projection.captureOrdinal=captureOrdinal;
   const resultText=Object.hasOwn(record,'result')?JSON.stringify(record.result):null;
   const compact={scope:'q1-mock-dispatch-ledger-v1',runNonce:nonce,...record,projection};delete compact.captured;
   if(resultText!==null){if(typeof resultText!=='string')throw new Error('Q1 core typed result JSON');if(new TextEncoder().encode(resultText).length>16777216)throw new Error('Q1 result encoding cap');
    compact.resultEvidence={encoding:'q1-core-typed-json-v1',sha256:await digest(resultText),bytes:new TextEncoder().encode(resultText).length,characters:resultText.length,fullEncoded:resultText.length<=65536?record.result:null};delete compact.result;}
   return ack(await window.__notayaQ1HostLedger(compact),record.attempt);
  }catch(error){try{await window.__notayaQ1HostLedger({scope:'q1-mock-dispatch-ledger-v1',runNonce:nonce,stage:'projection-error',attempt:record.attempt,command:typeof record.command==='string'?record.command.slice(0,128):null,error:String(error)});}catch{}throw error;}
 }});
}
function decode(v,depth=0){
 if(depth>32||!v||typeof v!=='object')throw new Error('Q1 typed data');
 if(v.t==='undefined')return undefined;if(v.t==='negative-zero')return -0;if(['string','boolean','number','object'].includes(v.t)&&Object.hasOwn(v,'v'))return v.v;
 if(v.t!=='object'&&v.t!=='array')throw new Error('Q1 encoded kind');
 if(!Array.isArray(v.e)||v.e.length>1048576)throw new Error('Q1 encoded entries');
 const out=v.t==='array'?[]:Object.create(null),seen=new Set();let arrayLength=null;
 for(const pair of v.e){if(!Array.isArray(pair)||pair.length!==2||typeof pair[0]!=='string'||seen.has(pair[0])||['__proto__','prototype','constructor'].includes(pair[0]))throw new Error('Q1 safe unique key');seen.add(pair[0]);if(v.t==='array'&&pair[0]==='length'){arrayLength=decode(pair[1],depth+1);if(!Number.isInteger(arrayLength)||arrayLength<0||arrayLength>1048576)throw new Error('Q1 array length');continue;}if(v.t==='array'&&!/^(?:0|[1-9][0-9]*)$/.test(pair[0]))throw new Error('Q1 dense array index');Object.defineProperty(out,pair[0],{value:decode(pair[1],depth+1),enumerable:true,writable:true,configurable:true});}
 if(v.t==='array'&&(arrayLength===null||out.length!==arrayLength||Object.keys(out).length!==arrayLength))throw new Error('Q1 exact dense array length');return out;
}
module.exports=async function runQ1(input,m,u,fixture){
 const {need,assert,hash,noFollow,same,inside,bounded,readPinned,saveNew}=u;
 need(input.postApplicationQualified&&input.currentSourceFenceQualified&&input.ownedProcessObserverEstablished,'Current source and owned observer gates');
 need(m.observerSHA256===OBSERVER&&m.observerOriginalSHA256===ORIGINAL&&fixture.origin==='http://127.0.0.1:15317','Exact separate Q1 observer/origin');
 const bindings=JSON.parse(readPinned(m.Q1BindingDataPath,m.Q1BindingDataSHA256,4194304));
 need(bindings.scope==='q1-caller-note-effect-bindings-ready-v1'&&bindings.approved===true&&bindings.ready===true&&bindings.executionApproved===false&&bindings.sourceOnly===false&&bindings.runnableBindingsComplete===true,'Genuine independent complete Q1 bindings, not proposal data');
 need(bindings.noteNormalization.inheritedDefaultsQualified===true&&bindings.noteNormalization.textSerializationQualified===true&&bindings.noteNormalization.runnableBindingsComplete===true&&Array.isArray(bindings.noteNormalization.allowedNoteTextCandidates)&&bindings.noteNormalization.allowedNoteTextByForm&&typeof bindings.noteNormalization.allowedNoteTextByForm==='object'&&!Array.isArray(bindings.noteNormalization.allowedNoteTextByForm),'Exact independently qualified schema defaults and text serialization');
 assert.deepEqual(m.allowedQ1MockCommands,COMMANDS);
 readPinned(m.observerPath,OBSERVER,65536);const observe=require(m.observerPath);need(typeof observe==='function','Borrowed observer export only');
 const result={scope:'q1-inline-browser-produced-v1',passed:false,observer:null,contexts:[],requests:[],responses:[],requestFailures:[],webSockets:[],pageErrors:[],consoleErrors:[],blocked:[],cleanupErrors:[],browserClosed:false,failure:null,ownership:null,
 commandAudit:{events:[],attemptCount:0,successCount:0,errorCount:0,pendingCount:0,pending:[],failure:null,baseline:null,phaseCheckpoints:[],overlapAttribution:'Global invariants at every immutable capture; per-call effects only if no overlap; host acknowledgement adds latency',unobservedPrivateBookkeeping:bindings.effectProjection.unobservedPrivateBookkeeping},
 nativeQualified:false,persistenceQualified:false,productAccepted:false,automaticFollowingStage:false};
 const push=(a,v,max=20000)=>{need(a.length<max,'Bounded Q1 browser observations');a.push(v);};
 let browser=null,context=null,page=null,baseline=null,baseProjection=null,stableDigests=null,currentPhase=null,phaseIndex=-1,addedId=null,linkedId=null,lastProjection=null,noteBaseline=null,eventSequence=0,auditBytes=0,storedWireRevision=null,storedWireRevisionCommitOrdinal=null,databaseBaseline=null,newRowImmutable=null;
 const bidByPath=new Map(),headingAnchorByPath=new Map(),captures=new Map(),qualifiedWrites=[];
 const attempts=new Map(),settled=new Set(),active=new Set(),writeSlots=new Set();
 function reserve(slot){need(!writeSlots.has(slot),'Exact single pre-dispatch write slot: '+slot);writeSlots.add(slot);}
 const admit=attempt=>({admitted:true,attempt,bindingSHA256:m.Q1BindingDataSHA256});
 function ledger(kind,payload,maximum=1048576){need(++eventSequence<=4096,'Q1 host ledger event cap');const bytes=Buffer.from(JSON.stringify({sequence:eventSequence,UTC:new Date().toISOString(),kind,phase:currentPhase,...payload}));
  need(bytes.length<=maximum&&auditBytes+bytes.length<=134217728,'Q1 finite host ledger byte budget');const saved=saveNew(path.join(input.runRoot,'browser-evidence','q1-audit-'+String(eventSequence).padStart(5,'0')+'.json'),bytes,maximum);auditBytes+=bytes.length;push(result.commandAudit.events,saved,4096);return saved;}
 function fail(error){if(!result.commandAudit.failure)result.commandAudit.failure={phase:currentPhase,message:String(error.stack||error)};return error;}
 function keys(a,expected){assert.deepEqual(Object.keys(a).sort(),expected.slice().sort(),'Exact argument own keys');}
 function sameData(a,b,label){assert.equal(canonical(a),canonical(b),label);}
 function originalView(){const v=baseProjection.mutable.tasks.views.filter(x=>x.name==='All tasks'&&x.kind==='table');need(v.length===1,'Exact initial All tasks');return v[0];}
 function validateNote(content,text){
  const normalization=bindings.noteNormalization,anchorPolicy=normalization.generatedHeadingAnchors;
  need(anchorPolicy&&anchorPolicy.sourceSHA256==='54D510C4453388941DFAC57A9CF143CE6DAAF51519C90A225280D45656EE599E'&&anchorPolicy.pattern==='^h[0-9a-z]{8}$','Exact reviewed heading-anchor source policy');
  sameData(anchorPolicy.generatedPaths,['root.0','root.6'],'Exact source-eligible top-level heading paths');sameData(anchorPolicy.defaultNullPaths,['root.8.0'],'Exact nested heading null-default path');
  const pendingHeadingAnchors=new Map(headingAnchorByPath),seenHeadingAnchors=new Set();
  for(const node of baseline.pageBody.content)if(node.type==='heading'&&Object.hasOwn(node.attrs||{},'anchor')&&typeof node.attrs.anchor==='string'&&node.attrs.anchor)seenHeadingAnchors.add(node.attrs.anchor);
  need(typeof content==='string'&&Buffer.byteLength(content)<=262144&&typeof text==='string'&&normalization.allowedNoteTextCandidates.includes(text),'Exact bounded note document/text candidate');
  const doc=JSON.parse(content),bids=new Set(),defaults=normalization.defaultAttrsByType;
  function clean(n,nodePath='root'){
   need(n&&typeof n==='object'&&!Array.isArray(n)&&typeof n.type==='string','Typed note node');const out={...n};
   let reference=baseline.pageBody;for(const index of nodePath.split('.').slice(1))reference=reference?.content?.[Number(index)];const explicit=reference?.type===n.type?reference.attrs||{}:{};
   if(n.attrs){const attrs={...n.attrs};if(Object.hasOwn(attrs,'bid')){
    need(normalization.generatedBlockIdentity.types.includes(n.type),'Only source-eligible block bid');
    need(typeof attrs.bid==='string'&&/^[0-9a-f]{16}$/.test(attrs.bid)&&!bids.has(attrs.bid),'Unique generated block identity');bids.add(attrs.bid);
    const last='root.'+(baseline.pageBody.content.length-1),key=nodePath+'|'+n.type,old=bidByPath.get(key);
    if(old)need(old.id===attrs.bid,'Earlier generated block identities survive setup');
    if(nodePath===last)need(n.type==='paragraph'||n.type==='databaseBlock','Only final paragraph replacement');
    bidByPath.set(key,{id:attrs.bid,type:n.type});delete attrs.bid;
   }
   if(n.type==='heading'){
    if(Object.hasOwn(explicit,'anchor'))need(Object.hasOwn(attrs,'anchor')&&canonical(attrs.anchor)===canonical(explicit.anchor),'Original explicit heading anchor preserved');
    else{
     need(Object.hasOwn(attrs,'anchor'),'Source heading anchor must serialize');
     if(anchorPolicy.generatedPaths.includes(nodePath)){
      need(typeof attrs.anchor==='string'&&/^h[0-9a-z]{8}$/.test(attrs.anchor)&&!seenHeadingAnchors.has(attrs.anchor),'Valid unique top-level generated heading anchor');seenHeadingAnchors.add(attrs.anchor);
      if(pendingHeadingAnchors.has(nodePath))need(pendingHeadingAnchors.get(nodePath)===attrs.anchor,'Stable generated heading anchor by original node path');pendingHeadingAnchors.set(nodePath,attrs.anchor);delete attrs.anchor;
     }else{need(anchorPolicy.defaultNullPaths.includes(nodePath)&&attrs.anchor===null,'Only exact nested heading null default');delete attrs.anchor;}
    }
   }
   const allowed=defaults[n.type]||{};for(const[k,v]of Object.entries(attrs))if(!Object.hasOwn(explicit,k)&&Object.hasOwn(allowed,k)&&canonical(v)===canonical(allowed[k]))delete attrs[k];
    if(Object.keys(attrs).length)out.attrs=attrs;else delete out.attrs;}
   if(normalization.generatedBlockIdentity.types.includes(n.type))need(n.attrs&&typeof n.attrs.bid==='string','Every saved eligible block has its generated identity');
   if(Array.isArray(n.content))out.content=n.content.map((child,i)=>clean(child,nodePath+'.'+i));return out;
  }
  const cleanDoc=clean(doc),raw=baseline.pageBody;keys(cleanDoc,['type','content']);need(cleanDoc.type==='doc'&&raw.type==='doc','Exact note document root');
  const nodes=cleanDoc.content,original=raw.content;need(Array.isArray(nodes)&&Array.isArray(original)&&original.at(-1).type==='paragraph'&&!original.at(-1).attrs&&!original.at(-1).content,'Source final empty paragraph');
  const prefix=original.slice(0,-1),empty=n=>n&&n.type==='paragraph'&&Object.keys(n).length===1;
  need(semantic(nodes.slice(0,prefix.length))===semantic(prefix),'Every earlier node/text/explicit attribute preserved; only exact added defaults and generated bid normalize');
  let form;
  if(nodes.length===original.length&&nodes.at(-1).type==='paragraph'){
   const tail=nodes.at(-1);let slash='';if(!empty(tail)){keys(tail,['type','content']);need(tail.content.length===1&&tail.content[0].type==='text','Only one slash-prefix text node');keys(tail.content[0],['type','text']);slash=tail.content[0].text;}
   need(normalization.allowedSlashPrefixes.includes(slash),'Only exact slash-prefix transition');form='slash:'+slash;
  }else{
   need(nodes.length===original.length+1&&empty(nodes.at(-1)),'Only exact linked block and one trailing empty paragraph');
   const linked=nodes[prefix.length];keys(linked,['type','attrs']);need(linked.type==='databaseBlock','Linked block replaces only final source paragraph');keys(linked.attrs,['databaseId','viewId']);
   need(linked.attrs.databaseId===baseProjection.identities.tasksId&&linked.attrs.viewId===linkedId,'Exact independent linked-view identity');form='linked';
  }
  need(Object.hasOwn(normalization.allowedNoteTextByForm,form)&&text===normalization.allowedNoteTextByForm[form],'Exact source-qualified content-form/text pair');
  if(noteBaseline)sameData({content,text},noteBaseline,'Same parent note bytes after linked baseline');
  headingAnchorByPath.clear();for(const[key,value]of pendingHeadingAnchors)headingAnchorByPath.set(key,value);
  return form==='linked'?1:0;
 }
 function validateProjection(p){
  need(p&&Number.isInteger(p.captureOrdinal)&&p.captureOrdinal>0&&p.captureOrdinal<=4096,'Finite immutable capture order');need(p&&Object.keys(p.stableDigests).length===Object.keys(stableDigests).length,'Exact stable domains');sameData(p.stableDigests,stableDigests,'All unchanged fixture domains including5004 original rows');
  need(p.identities.tasksId===baseProjection.identities.tasksId&&p.identities.projectsId===baseProjection.identities.projectsId&&p.identities.originalRowCount===5004&&p.identities.originalTasksCount===5000,'Exact original dataset identity/count');
  need(p.identities.originalOrderedIDsSHA256===hash(Buffer.from(canonical(baseProjection.identities.originalRowIDs))),'All earlier row IDs and order digest');
  const t=p.mutable.tasks,b=baseProjection.mutable.tasks;sameData(t.keyOrder,t.storedWireRevision.present?[...b.keyOrder,'rev']:b.keyOrder,'Exact raw Tasks own-key order; only stored rev appended');if(storedWireRevision){if(t.storedWireRevision.present)sameData(t.storedWireRevision,storedWireRevision,'Exact admitted stored wire revision');else need(storedWireRevisionCommitOrdinal===null||p.captureOrdinal<storedWireRevisionCommitOrdinal,'Only pending/earlier absent wire revision');}else need(t.storedWireRevision.present===false,'Initially absent stored wire revision');need(t.id===b.id&&[b.nextId,b.nextId+1].includes(t.nextId),'Only one new row nextId step');
  need(t.views.length===b.views.length||t.views.length===b.views.length+1,'Only one independent linked view');
  sameData(t.views.slice(0,b.views.length),b.views,'Earlier views unchanged');if(databaseBaseline&&p.captureOrdinal>=databaseBaseline.captureOrdinal){sameData({views:t.views,updated:t.updated,storedWireRevision:t.storedWireRevision},databaseBaseline.value,'Database/view metadata stable throughout row history');}
  if(t.views.length>b.views.length){const v=t.views.at(-1);need(/^[0-9a-f]{32}$/.test(v.id)&&!b.views.some(x=>x.id===v.id),'Fresh linked view ID');const expected={...originalView(),id:v.id,name:'All tasks (linked)'};sameData(v,expected,'Exact copied linked view');if(linkedId)need(linkedId===v.id,'Stable linked view identity');linkedId=v.id;}
  sameData(p.mutable.lastPlace,{kind:'page',id:'p1'},'Only exact parent-note lastPlace');
  const originalP1=baseline.tree.pages.find(x=>x.id==='p1');need(p.mutable.p1.title==='Morning pages'&&p.mutable.p1.words===originalP1.words,'Parent note identity/words unchanged');
  if(p.mutable.savedP1){keys(p.mutable.savedP1,['title','content','text']);need(p.mutable.savedP1.title==='Morning pages','Exact saved parent title');validateNote(p.mutable.savedP1.content,p.mutable.savedP1.text);need(p.mutable.p1.preview===p.mutable.savedP1.text.slice(0,160),'Exact note preview bookkeeping');}
  const row=p.mutable.newRow;if(row){if(!addedId){need(Array.from(attempts.values()).some(a=>a.command==='create_row'),'Only an admitted pending create may introduce its row');need(/^[0-9a-f]{32}$/.test(row.id)&&!baseProjection.identities.originalRowIDs.includes(row.id),'Fresh pending create identity');addedId=row.id;}need(addedId===row.id&&row.db===t.id&&row.number===5001&&row.order===baseProjection.identities.maxOriginalOrder+1,'Same one appended row identity/position');
   need(['',TITLE].includes(row.title)&&row.content===''&&row.text===''&&row.icon===''&&row.cover===''&&row.favorite===false&&row.createdBy==='You'&&row.editedBy==='You','Only scoped blank/title row fields');const fixed=Object.fromEntries(Object.entries(row).filter(([k])=>!['title','updated','trashedAt'].includes(k)));if(newRowImmutable)sameData(fixed,newRowImmutable,'Same created row immutable fields');else newRowImmutable=fixed;sameData(row.cells,{},'No cell/template/relation effects');need(row.trashedAt===null||Number.isFinite(row.trashedAt),'Soft trash timestamp only');}
  for(const [id,rev]of p.mutable.revs)need(['p1',t.id,addedId].includes(id)&&Number.isInteger(rev)&&rev>=0,'Only named mock revision IDs');
  need(Array.isArray(p.mutable.aiCalls)&&p.mutable.aiCalls.length<=64,'Bounded read-status bookkeeping');for(const call of p.mutable.aiCalls){keys(call,['command','args']);need(call.command==='ai_get_status','Only AI read-status call bookkeeping');keys(call.args,[]);}
  need(!captures.has(p.captureOrdinal),'Unique immutable capture order');captures.set(p.captureOrdinal,p);if(!lastProjection||p.captureOrdinal>lastProjection.captureOrdinal)lastProjection=p;
 }
 function initialHistoryState(p){
  need(baseProjection.mutable.savedP1===null&&p.mutable.savedP1===null,'Initialization-only unsaved p1 history snapshot');
  sameData(p.mutable.p1,baseProjection.mutable.p1,'Full initial p1 metadata including icon/cover');
  sameData(p.mutable.tasks,baseProjection.mutable.tasks,'Initial Tasks metadata/views/counters remain unchanged');
  sameData(p.mutable.revs,baseProjection.mutable.revs,'Initial complete revision map remains unchanged');
  need(p.stableDigests.pageBody===stableDigests.pageBody,'Initial raw lexical p1 body remains unchanged');
 }
 function validateArgs(command,a,p){
  need(a&&typeof a==='object'&&!Array.isArray(a),'Plain argument object');const db=baseProjection.identities.tasksId,phase=currentPhase;
  const reads=['vault_status','get_settings','list_notes','list_stickies','list_bin','take_notice','window_ready','due_reminders_now','ai_get_status'];
  if(command==='history_snapshot'){keys(a,['pageId']);need(phase===PHASES[0]&&phaseIndex===0&&a.pageId==='p1'&&writeSlots.size===0,'Only clean p1 initialization lifecycle snapshot');initialHistoryState(p);return;}
  if(reads.includes(command)){keys(a,[]);if(['vault_status','get_settings','take_notice','window_ready'].includes(command))need(phase===PHASES[0],'Initialization-only command');return;}
  if(command==='set_backdrop'){keys(a,['kind','dark']);sameData(a,{kind:'none',dark:false},'Exact appearance');need(phase===PHASES[0],'Initialization backdrop only');return;}
  if(command==='set_last_place'){keys(a,['lastPlace']);sameData(a.lastPlace,{kind:'page',id:'p1'},'Exact navigation');return;}
  if(command==='get_database'){keys(a,['id']);need(a.id===db,'Only Tasks read');return;}
  if(command==='get_page'){keys(a,['id']);need(a.id==='p1'||(addedId&&a.id===addedId),'Only parent or added row page');return;}
  if(command==='editing'){keys(a,['id','active']);need(['p1',db].includes(a.id)&&typeof a.active==='boolean','Exact note/database editing lifecycle');return;}
  if(command==='row_titles'){keys(a,['ids']);const ids=baseProjection.identities.projectsRows.map(r=>r.id);need(Array.isArray(a.ids)&&a.ids.length>0&&a.ids.length<=5001&&a.ids.every(id=>ids.includes(id)),'Finite four source-bound Projects relation IDs');return;}
  if(command==='query_view'){keys(a,['request']);keys(a.request,['databaseId','viewId','view']);need(a.request.databaseId===db,'Exact query database');const views=[...baseProjection.mutable.tasks.views,...p.mutable.tasks.views];const match=views.filter(v=>v.id===a.request.viewId);need(match.length>0,'Known original or independently copied linked view');sameData(a.request.view,match[0],'Consistent query-generation identity/values');return;}
  if(command==='save_database'){keys(a,['databaseJson','baseRev']);need(phase===PHASES[1]&&a.baseRev===0&&(new Map(p.mutable.revs).get(db)||0)===0,'Linked setup only');const d=JSON.parse(a.databaseJson),before=baseline.databaseItems.find(x=>x.id===db);need(Object.hasOwn(d,'rev')&&d.rev===0,'Exact sole initial wire rev0');sameData(Object.keys(d),[...baseProjection.mutable.tasks.keyOrder,'rev'],'Exact initial key order plus appended wire revision');reserve('linked-database-save');storedWireRevision={present:true,value:0};const omit=(x,ks)=>Object.fromEntries(Object.entries(x).filter(([k])=>!ks.includes(k)));sameData(omit(d,['rev','views','updated']),omit(before,['kind','views','updated']),'No unrelated database schema/metadata write');need(d.views.length===before.views.length+1,'Append exactly one linked view');sameData(d.views.slice(0,before.views.length),before.views,'Earlier view definitions');const v=d.views.at(-1);need(/^[0-9a-f]{32}$/.test(v.id),'Linked view ID');sameData(v,{...originalView(),id:v.id,name:'All tasks (linked)'},'Only exact view copy');if(linkedId)need(v.id===linkedId,'Same linked view');linkedId=v.id;return;}
  if(command==='save_page'){keys(a,['id','title','content','text','baseRev']);need(phase===PHASES[1]&&a.id==='p1'&&a.title==='Morning pages'&&Number.isInteger(a.baseRev)&&a.baseRev===(new Map(p.mutable.revs).get('p1')||0),'Only exact linked setup note save');validateNote(a.content,a.text);return;}
  if(command==='create_row'){keys(a,['databaseId','title','cells','templateId']);need(phase===PHASES[2]&&!addedId&&a.databaseId===db&&a.title===''&&a.cells==='{}'&&a.templateId==='','One blank default row');need(baseProjection.mutable.tasks.defaultTemplate==='','No default template');reserve('append-new-row');return;}
  if(command==='set_cells'){keys(a,['rowId','title','patch']);need(a.rowId===addedId&&a.patch==='{}','Only new row title');const title=phase===PHASES[3]||phase===PHASES[7]?TITLE:phase===PHASES[4]?'':null;need(title!==null&&a.title===title,'Exact name/undo/redo title transition');reserve('title:'+phase);return;}
  if(command==='trash_page'||command==='restore_page'){keys(a,['id']);need(a.id===addedId&&phase===(command==='trash_page'?PHASES[5]:PHASES[6]),'Same added-row undo/redo only');reserve(command+':'+phase);return;}
  throw new Error('Unknown Q1 command held');
 }
 async function audit(record){
  const saved=ledger('mock-dispatch',record,record.stage==='baseline'?16777216:1048576);
  try{
   need(record&&record.scope==='q1-mock-dispatch-ledger-v1'&&record.runNonce===input.nonce&&typeof record.stage==='string','Typed exact-run audit stage');need(!result.commandAudit.failure,'Earlier audit failure holds');
   if(record.stage==='baseline'){
    need(!baseline,'Exactly one immutable baseline');baseline=decode(record.capturedEncoded);baseProjection=projectCaptured(baseline);
    need(baseProjection.identities.originalRowCount===5004&&baseProjection.identities.originalTasksCount===5000&&baseProjection.identities.projectsRows.length===4,'Exact5000+4 synthetic baseline');
    sameData(baseProjection.identities.projectsRows.map(x=>x.title).sort(),['Website relaunch','Mobile app','Research notes','Office move'].sort(),'Exact Projects titles');
    const rawTasks=baseline.databaseItems.find(x=>x.kind==='database'&&x.id===baseProjection.identities.tasksId);need(rawTasks.defaultTemplate===''&&!Object.hasOwn(rawTasks,'automations')&&baseline.ai.calls.length===0,'Exact blank default template, absent automations and empty AI calls');
    stableDigests=Object.fromEntries(Object.entries(baseProjection.stable).map(([k,v])=>[k,hash(Buffer.from(canonical(v)))]));result.commandAudit.baseline=saved;return admit(0);
   }
   need(baseline&&Number.isInteger(record.attempt)&&record.attempt>0&&record.attempt<=2048,'Known finite attempt identity');
   if(record.stage==='projection-error'){
    keys(record,['scope','runNonce','stage','attempt','command','error']);need((record.command===null||typeof record.command==='string'&&record.command.length<=128)&&typeof record.error==='string','Bounded bridge projection diagnostic');
    const pending=attempts.get(record.attempt);result.commandAudit.projectionFailure={attempt:record.attempt,command:record.command,originalBridgeError:record.error,matchingPending:!!pending&&pending.command===record.command&&!settled.has(record.attempt),pendingPhase:pending?.phase??null,hostActive:Array.from(active),settlementQualified:false};
    throw new Error('Bridge projection failure retained: '+record.error);
   }
   need(Array.isArray(record.overlapping)&&record.overlapping.length>0&&record.overlapping.length<=2048&&new Set(record.overlapping).size===record.overlapping.length&&record.overlapping.includes(record.attempt)&&record.overlapping.every(id=>Number.isInteger(id)&&id>0&&id<=2048),'Exact bounded page-active attempt IDs');
   if(record.stage==='before'||record.stage==='invalid-before'){
    need(!attempts.has(record.attempt)&&!settled.has(record.attempt),'Unique attempt');attempts.set(record.attempt,{command:record.command,phase:currentPhase,before:record.projection,args:decode(record.args),event:saved,overlap:record.overlapping.some(id=>id!==record.attempt)});active.add(record.attempt);result.commandAudit.attemptCount++;
    for(const id of active){const pending=attempts.get(id);if(pending&&active.size>1)pending.overlap=true;}
    need(record.stage==='before'&&COMMANDS.includes(record.command),'Unknown or invalid attempted dispatch retained then held');need(decode(record.options)===undefined,'No invoke options/headers');validateProjection(record.projection);validateArgs(record.command,decode(record.args),record.projection);ledger('dispatch-admitted',{attempt:record.attempt,command:record.command,bindingSHA256:m.Q1BindingDataSHA256,reservedWriteSlots:Array.from(writeSlots)});return admit(record.attempt);
   }
   need(['success','error'].includes(record.stage),'Projection/unknown stage failure retained');const pending=attempts.get(record.attempt);need(pending&&pending.command===record.command&&!settled.has(record.attempt),'Exactly one matching settlement');
   if(record.stage==='success'){
    const e=record.resultEvidence;need(e&&typeof e==='object','Exact typed result evidence');keys(e,['encoding','sha256','bytes','characters','fullEncoded']);
    need(e.encoding==='q1-core-typed-json-v1'&&typeof e.sha256==='string'&&/^[0-9A-F]{64}$/.test(e.sha256)&&Number.isSafeInteger(e.bytes)&&e.bytes>0&&e.bytes<=16777216&&Number.isSafeInteger(e.characters)&&e.characters>0&&e.characters<=e.bytes,'Versioned bounded core-typed JSON evidence');
    if(e.fullEncoded!==null){const text=JSON.stringify(e.fullEncoded),bytes=Buffer.from(text);need(text.length<=65536&&text.length===e.characters&&bytes.length===e.bytes&&hash(bytes)===e.sha256,'Exact small typed-result JSON byte/hash readback');}
    else need(e.characters>65536,'Only a large complete typed result uses digest-only evidence');
   }
   if(record.command==='create_row'&&record.stage==='success'){const row=record.projection.mutable.newRow;need(row&&/^[0-9a-f]{32}$/.test(row.id)&&!baseProjection.identities.originalRowIDs.includes(row.id),'One fresh created row');addedId=row.id;const value=decode(record.resultEvidence.fullEncoded);need(value?.row?.id===addedId,'Exact create_row result identity');}
   if(record.command==='history_snapshot'){need(pending.phase===PHASES[0],'Snapshot was admitted only in initialization');initialHistoryState(record.projection);if(record.stage==='success')need(record.resultEvidence&&record.resultEvidence.fullEncoded&&decode(record.resultEvidence.fullEncoded)===null,'Exact history_snapshot null result');}
   if(record.overlapping.some(id=>id!==record.attempt))pending.overlap=true;if(record.command==='save_database'&&record.stage==='success')storedWireRevisionCommitOrdinal=record.projection.captureOrdinal;validateProjection(record.projection);
   if(!pending.overlap&&!['save_page','save_database','create_row','set_cells','trash_page','restore_page','set_last_place','ai_get_status','due_reminders_now','take_notice'].includes(record.command))sameData({...record.projection,captureOrdinal:0},{...pending.before,captureOrdinal:0},'Non-overlap currently observed domains unchanged; declared private bookkeeping remains scoped');
   active.delete(record.attempt);settled.add(record.attempt);attempts.delete(record.attempt);
   if(record.stage==='error'){result.commandAudit.errorCount++;throw new Error('Original mock rejection retained: '+record.error);}
   if(['save_page','save_database','set_cells','trash_page','restore_page'].includes(record.command))qualifiedWrites.push({attempt:record.attempt,command:record.command,id:record.command==='save_database'?baseProjection.identities.tasksId:record.command==='save_page'?'p1':addedId,overlap:pending.overlap});
   result.commandAudit.successCount++;return admit(record.attempt);
  }catch(error){fail(error);ledger('first-or-pending-failure',{attempt:record?.attempt??null,command:record?.command??null,message:String(error),active:Array.from(active)});throw error;}
 }
 function checkpoint(value){
  const terminal=value.kind==='observation-return-candidate'||value.kind==='observation-terminal-failure';
  try{
   const saved=ledger(terminal?'observer-terminal-evidence':'observer-phase',value,terminal?16777216:1048576);
   if(terminal){
    result.commandAudit.terminalEvidence??=[];push(result.commandAudit.terminalEvidence,{kind:value.kind,phase:value.phase,candidatePassed:value.candidatePassed??null,
     finalDeadlineCheckPending:value.finalDeadlineCheckPending??false,fullReportPreserved:!!(value.reportCandidate||value.fullFailedReport),saved},4);
    push(result.commandAudit.phaseCheckpoints,{kind:value.kind,phase:value.phase,atUTC:value.atUTC,saved},32);
    return; // Evidence retention remains allowed after failure; no actor phase or browser action follows here.
   }
   need(!result.commandAudit.failure,'Audit failure stops next observer actor operation');
   if(value.kind==='phase-start'){
    need(PHASES[phaseIndex+1]===value.phase,'Exact nine ordered actor phases');
    sameData(value.completedPhases,PHASES.slice(0,phaseIndex+1),'Only post-checkpoint committed earlier phases may advance');
    if(value.phase===PHASES[2]){
     need(lastProjection?.mutable.savedP1,'Committed linked parent save baseline before row actors');
     noteBaseline={content:lastProjection.mutable.savedP1.content,text:lastProjection.mutable.savedP1.text};
     databaseBaseline={captureOrdinal:lastProjection.captureOrdinal,value:{views:lastProjection.mutable.tasks.views,updated:lastProjection.mutable.tasks.updated,storedWireRevision:lastProjection.mutable.tasks.storedWireRevision}};
    }
    phaseIndex++;currentPhase=value.phase;
   }
   push(result.commandAudit.phaseCheckpoints,value,32);
  }catch(error){
   if(!result.commandAudit.failure)result.commandAudit.failure={phase:value.phase??currentPhase,message:String(error.stack||error),checkpointKind:value.kind};
   throw error;
  }
 }

 try{
  noFollow(m.playwrightModule);const pw=require(m.playwrightModule);
  browser=await bounded(pw.chromium.launch({executablePath:m.chromeImage,headless:true,chromiumSandbox:true,timeout:10000,env:{...input.environment},
   artifactsDir:path.join(input.runRoot,'browser-artifacts'),downloadsPath:path.join(input.runRoot,'downloads'),tracesDir:path.join(input.runRoot,'traces'),
   handleSIGINT:false,handleSIGTERM:false,handleSIGHUP:false,ignoreDefaultArgs:['--hide-scrollbars']}),15000,'owned Q1 browser');
  const ready=path.join(input.runRoot,'browser-owned-attestation.ready'),payload=path.join(input.runRoot,'browser-owned-attestation.complete.json');let ack=null,deadline=Date.now()+5000;
  while(Date.now()<deadline){if(fs.existsSync(ready)){noFollow(ready);need(fs.lstatSync(ready).isFile()&&fs.statSync(ready).size===0,'Ready marker');noFollow(payload);need(fs.lstatSync(payload).isFile()&&fs.statSync(payload).size>0&&fs.statSync(payload).size<=65536,'Complete owned ACK cap');ack=JSON.parse(fs.readFileSync(payload,'utf8'));break;}await new Promise(resolve=>setTimeout(resolve,25));}
  need(ack&&ack.scope==='p6c-b3b-owned-browser-attestation-v1'&&ack.actor==='/root-guard'&&ack.nodePid===process.pid&&Number.isInteger(ack.pid)&&ack.pid>0&&typeof ack.startTimeUtc==='string'&&ack.image===m.chromeImage&&inside(ack.profilePath,path.join(input.runRoot,'temp'))&&path.basename(ack.profilePath).startsWith('playwright_chromiumdev_profile-'),'Exact fresh owned Chrome ACK');result.ownership=ack;
  context=await bounded(browser.newContext({viewport:{width:1400,height:900},acceptDownloads:false,serviceWorkers:'block',permissions:[],locale:'en-US',timezoneId:'UTC'}),5000,'one fresh Q1 context');
  const row={index:0,createdPages:0,closed:false,url:null,storageBefore:null,storageAfter:null};result.contexts.push(row);
  context.on('close',()=>{row.closed=true;});context.on('serviceworker',()=>push(result.blocked,{kind:'serviceworker'}));
  context.on('request',r=>push(result.requests,{url:r.url(),method:r.method(),type:r.resourceType()}));context.on('response',r=>push(result.responses,{url:r.url(),status:r.status(),type:r.request().resourceType()}));
  context.on('requestfailed',r=>push(result.requestFailures,{url:r.url(),method:r.method(),error:r.failure()?.errorText||null}));
  await bounded(context.route('**/*',async route=>{const r=route.request();let good=false;try{const u=new global.URL(r.url());good=r.method()==='GET'&&u.origin===fixture.origin&&!u.hash&&!u.username&&!u.password;}catch{}if(!good){push(result.blocked,{kind:'HTTP',url:r.url()});await route.abort('blockedbyclient');return;}await route.continue();}),5000,'loopback GET only');
  await bounded(context.routeWebSocket('**/*',async socket=>{push(result.webSockets,{url:socket.url()});push(result.blocked,{kind:'WebSocket'});await socket.close({code:1008,reason:'Q1 no sockets'});}),5000,'deny WebSocket');
  await bounded(context.exposeBinding('__notayaQ1HostLedger',async(source,record)=>{if(!(source.context===context&&source.page===page&&source.frame===page.mainFrame())){ledger('held-audit-origin',{attempt:Number.isInteger(record?.attempt)?record.attempt:null,command:typeof record?.command==='string'?record.command.slice(0,128):null});throw fail(new Error('Only one exact Q1 main frame audit'));}return audit(record);}),5000,'durable main-frame audit binding');
  await bounded(context.addInitScript({content:'('+installBridge.toString()+')('+projectCaptured.toString()+','+canonical.toString()+','+encodeData.toString()+','+JSON.stringify(input.nonce)+','+JSON.stringify(m.Q1BindingDataSHA256)+');'}),5000,'reviewed transparent audit initialization');
  row.storageBefore=await bounded(context.storageState(),5000,'fresh Q1 storage');assert.deepEqual(row.storageBefore,{cookies:[],origins:[]});
  context.on('page',p=>{row.createdPages++;if(row.createdPages!==1)push(result.blocked,{kind:'extra-page'});p.on('pageerror',e=>push(result.pageErrors,{error:String(e)}));p.on('console',m=>{if(['error','warning'].includes(m.type()))push(result.consoleErrors,{type:m.type(),text:m.text().slice(0,4096)});});p.on('download',d=>{push(result.blocked,{kind:'download'});void d.cancel();});p.on('dialog',d=>{push(result.blocked,{kind:'dialog'});void d.dismiss();});p.on('framenavigated',f=>{if(f.url()==='about:blank')return;if(f!==p.mainFrame()||f.url()!==URL)push(result.blocked,{kind:'navigation',url:f.url()});else row.url=f.url();});});
  page=await bounded(context.newPage(),5000,'one Q1 page');
  result.observer=await bounded(observe(page,{scope:'p6c-Q1-inline-Root-observation-admission-v1',actor:'/root',approved:true,executionApproved:true,mockOnly:true,
   goalSHA256:m.goalSHA256,finalCommitOID:m.finalCommitOID,sourceModelSHA256:m.sourceModelSHA256,freshOwnedSyntheticContext:true,
   exactEditorSourceAndToolReadinessReviewed:true,unknownCommandAndRuntimeErrorsRemainHolds:true,automaticFollowingStage:false,checkpoint}),280000,'exact nine-phase observer');
  need(result.observer.passed===true&&result.observer.finalDeadlineCheckPassed===true&&result.observer.finalDeadlineCheckPending===false,'Q1 first-failed phase or operational checkpoint deadline retained');need(row.createdPages===1&&page.url()===URL,'One exact final note page');
  row.storageAfter=await bounded(context.storageState(),5000,'owned final Q1 storage');need(row.storageAfter.cookies.length===0&&row.storageAfter.origins.every(x=>x.origin===fixture.origin),'Only owned synthetic storage');
 }catch(error){result.failure=String(error.stack||error);}
 finally{
  result.commandAudit.reservedWriteSlots=Array.from(writeSlots);result.commandAudit.pending=Array.from(attempts,([attempt,p])=>({attempt,command:p.command,phase:p.phase,overlap:p.overlap}));result.commandAudit.pendingCount=result.commandAudit.pending.length;result.commandAudit.bytes=auditBytes;
  try{ledger('terminal-audit',{pending:result.commandAudit.pending,failure:result.commandAudit.failure,observerReturned:!!result.observer});}catch(e){result.cleanupErrors.push(String(e));}
  if(context){try{const row=result.contexts[0];if(row&&row.storageAfter===null){row.storageAfter=await bounded(context.storageState(),5000,'failed-attempt owned storage evidence');need(row.storageAfter.cookies.length===0&&row.storageAfter.origins.every(x=>x.origin===fixture.origin),'Only owned storage even on failure');}if(row&&page)row.url=page.url();}catch(e){result.cleanupErrors.push(String(e));}
   try{await bounded(context.close(),10000,'owned Q1 context close');}catch(e){result.cleanupErrors.push(String(e));}}
  if(browser){try{await bounded(browser.close(),10000,'owned Q1 browser close');result.browserClosed=!browser.isConnected();}catch(e){result.cleanupErrors.push(String(e));}}
  const expectedRevs=new Map();for(const write of qualifiedWrites)expectedRevs.set(write.id,(expectedRevs.get(write.id)||0)+1);
  try{if(result.observer?.passed&&!result.commandAudit.failure){const orderedCaptures=Array.from(captures.values()).sort((a,b)=>a.captureOrdinal-b.captureOrdinal),prior=new Map();for(const capture of orderedCaptures){const values=new Map(capture.mutable.revs);for(const [id,value]of values){need(value>=(prior.get(id)||0)&&value<=(expectedRevs.get(id)||0),'Monotonic bounded source revision receipts');prior.set(id,value);}for(const id of prior.keys())need(values.has(id),'No disappeared revision receipt');}const finalRevs=new Map(lastProjection.mutable.revs);need(finalRevs.size===expectedRevs.size&&Array.from(expectedRevs).every(([id,n])=>finalRevs.get(id)===n),'Exact final write/revision counts; create_row adds no revision');need(lastProjection.mutable.tasks.nextId===baseProjection.mutable.tasks.nextId+1&&lastProjection.mutable.tasks.storedWireRevision.present===true&&lastProjection.mutable.newRow?.id===addedId&&lastProjection.mutable.newRow.title===TITLE&&lastProjection.mutable.newRow.trashedAt===null,'Final exact new-row and wire bookkeeping state');result.commandAudit.revisionQualification={passed:true,successfulWrites:qualifiedWrites,expectedFinal:Array.from(expectedRevs),observedFinal:lastProjection.mutable.revs,orderedCaptures:orderedCaptures.length};}}catch(error){fail(error);result.cleanupErrors.push(String(error));}
  result.commandAudit.pending=Array.from(attempts,([attempt,p])=>({attempt,command:p.command,phase:p.phase,overlap:p.overlap}));result.commandAudit.pendingCount=result.commandAudit.pending.length;result.commandAudit.bytes=auditBytes;
  try{ledger('closed-browser-final-audit',{pending:result.commandAudit.pending,errors:result.commandAudit.errorCount,browserClosed:result.browserClosed,firstFailure:result.commandAudit.failure});}catch(e){result.cleanupErrors.push(String(e));}
  result.passed=!result.failure&&!result.commandAudit.failure&&result.commandAudit.pendingCount===0&&result.commandAudit.errorCount===0&&result.commandAudit.attemptCount>0&&result.commandAudit.successCount===result.commandAudit.attemptCount&&result.observer?.passed===true&&
   result.contexts.length===1&&result.contexts[0].closed&&result.contexts[0].createdPages===1&&result.browserClosed&&result.cleanupErrors.length===0&&[result.requestFailures,result.webSockets,result.pageErrors,result.consoleErrors,result.blocked].every(a=>a.length===0);
 }
 return result;
};

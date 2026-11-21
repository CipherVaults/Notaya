"use strict";
// Run under a bounded process controller with an isolated home and locked dependencies.
// node dev-mock/table-removed-row-focus-check.cjs PW_MODULE PW_SHA256 CHROME_EXE CHROME_SHA256 FRESH_OUT
const fs=require("node:fs"),path=require("node:path"),http=require("node:http"),crypto=require("node:crypto");
const {createRequire}=require("node:module"),{pathToFileURL}=require("node:url");
const assert=require("node:assert/strict");
const FIXTURE="\nimport React, {StrictMode, useCallback, useEffect, useRef, useState} from \"react\";\nimport {createRoot} from \"react-dom/client\";\nimport {flushSync} from \"react-dom\";\nimport TableView from \"__TABLE_MODULE__\";\nimport {DbShellContext} from \"__CONTEXT_MODULE__\";\nimport {handleUndoKey, pushUndo, retainUndo, undoDepth} from \"__UNDO_MODULE__\";\nconst seedRows = Array.from({length:80}, (_,i)=>({id:\"r\"+(i+1),title:\"Row \"+(i+1),icon:\"\",number:i+1,created:1,updated:1,favorite:false,values:{}}));\nconst prop = {id:\"title\",name:\"Name\",kind:\"title\",options:[],numberFormat:\"\",numberDisplay:\"\",numberMax:0,dateFormat:\"\",formula:\"\",relationDb:\"\",relationBack:\"\",relationLimitOne:false,rollupRelation:\"\",rollupProp:\"\",rollupCalc:\"\",idPrefix:\"\",description:\"\",buttonLabel:\"\",actions:[]};\nconst groupBy = {prop:\"title\",sort:\"manual\",dateBy:\"\",numberStep:0,hideEmpty:false,hidden:[],collapsed:[]};\nfunction makeView(cfg:any, id:string) {return {id,name:\"Table\",kind:\"table\",filter:{op:\"and\",conds:[],groups:[]},sorts:cfg.sort?[{prop:\"title\",descending:false}]:[],group:cfg.group?{...groupBy}:null,subGroup:cfg.group===\"subgroup\"?{...groupBy}:null,columns:[{prop:\"title\",visible:true,width:280}],calcs:[],freeze:0,wrap:!!cfg.wrap,rowNumbers:false,openIn:\"full\",cardSize:\"\",cardCover:\"\",cardFit:false,dateProp:\"\",endProp:\"\",calendarMode:\"\",timelineZoom:\"\",showDeps:false,showSubItems:false,chart:{}};}\nfunction groupsFor(kind:string|null, rows:any[]) {\n const ids=rows.map(r=>r.id); const group=(key:string,rowIds:string[],subs:any[]=[])=>({key,label:key,color:\"\",count:rowIds.length,rows:rowIds,subs,calcs:{}});\n if(kind===\"subgroup\") return [group(\"g\",ids,[group(\"s\",ids)])];\n if(kind===\"duplicate\") return [group(\"g1\",ids),group(\"g2\",[\"r2\"])];\n return kind?[group(\"g\",ids)]:null;\n}\nlet serial=0;\nconst root=createRoot(document.getElementById(\"mount\")!);\nfunction App({cfg,token}:{cfg:any;token:number}) {\n const [state,setState]=useState<any>(()=>({rows:seedRows,view:makeView(cfg,\"v1\"),dbId:\"db-\"+token,mounted:true,hidden:false,dialog:null,autofocus:false,tick:0}));\n const stats=useRef({writes:[] as any[],unexpected:[] as string[],undo:0,redo:0,rootKeys:0,probe:{attach:0,detach:0}});\n const probe=useCallback((node:HTMLElement|null)=>{stats.current.probe[node?\"attach\":\"detach\"]++;},[]);\n const db:any={id:state.dbId,sectionId:\"section\",parent:\"\",inline:true,title:\"Fixture\",icon:\"\",description:\"\",order:0,favorite:false,locked:false,props:[prop],views:[state.view],templates:[],defaultTemplate:\"\",nextId:81,subParentProp:\"\",subChildProp:\"\",blockedByProp:\"\",blockingProp:\"\",automations:[],trashedAt:null,created:1,updated:1};\n const result:any={rows:state.rows,total:80,filtered:state.rows.length,truncated:false,groups:groupsFor(cfg.group,state.rows),calcs:{}};\n const unexpected=(name:string)=>{stats.current.unexpected.push(name);throw new Error(\"Unexpected shell operation: \"+name)};\n useEffect(()=>{\n  const release=retainUndo(state.dbId);\n  pushUndo(state.dbId,{label:\"fixture add row\",undo:()=>{stats.current.undo++;setState((s:any)=>({...s,rows:s.rows.filter((r:any)=>r.id!==\"r2\")}));},redo:()=>{stats.current.redo++;setState((s:any)=>({...s,rows:seedRows}));}});\n  return release;\n },[state.dbId]);\n const shell:any={db,view:state.view,databases:[],locked:false,compact:true,\n  editDb:async()=>unexpected(\"editDb\"),editView:(f:any)=>setState((s:any)=>({...s,view:f(s.view)})),\n  trashRows:async()=>unexpected(\"trashRows\"),duplicateRow:async()=>unexpected(\"duplicateRow\"),requery:()=>unexpected(\"requery\"),\n  rowById:new Map(state.rows.map((r:any)=>[r.id,r])),focusRowId:null,clearFocusRow:()=>{},focusRow:()=>unexpected(\"focusRow\"),\n  patchRowLocal:()=>unexpected(\"patchRowLocal\"),openProp:()=>unexpected(\"openProp\"),sampleOf:()=>null};\n const props:any={db,view:state.view,result,rowById:shell.rowById,locked:false,openRow:()=>unexpected(\"openRow\"),\n  setTitle:async(id:string,title:string)=>{stats.current.writes.push({kind:\"title\",id,title})},\n  setCells:async(id:string,patch:any)=>{stats.current.writes.push({kind:\"cells\",id,patch})},\n  createRow:async()=>unexpected(\"createRow\"),\n  updateView:(patch:any)=>setState((s:any)=>({...s,view:{...s.view,...patch}})),\n  reorderRows:async()=>unexpected(\"reorderRows\")};\n (window as any).fixture={\n  ready:true,\n  mutate:(kind:string)=>flushSync(()=>setState((s:any)=>{\n   if(kind===\"remove\"||kind===\"filter\")return {...s,rows:s.rows.filter((r:any)=>r.id!==\"r2\")};\n   if(kind===\"other-remove\")return {...s,rows:s.rows.filter((r:any)=>r.id!==\"r3\")};\n   if(kind===\"collapse\")return {...s,view:{...s.view,group:{...s.view.group,collapsed:[cfg.group===\"duplicate\"?\"g2\":\"g\"]}}};\n   if(kind===\"subgroup\")return {...s,view:{...s.view,group:{...s.view.group,collapsed:[\"g/s\"]}}};\n   if(kind===\"scope-db\")return {...s,rows:s.rows.filter((r:any)=>r.id!==\"r2\"),dbId:s.dbId+\"-new\"};\n   if(kind===\"scope-view\")return {...s,rows:s.rows.filter((r:any)=>r.id!==\"r2\"),view:{...s.view,id:\"v2\"}};\n   if(kind===\"unmount\"||kind===\"navigate\")return {...s,mounted:false};\n   if(kind===\"hidden\")return {...s,rows:s.rows.filter((r:any)=>r.id!==\"r2\"),hidden:true};\n   if(kind===\"wrap\")return {...s,view:{...s.view,wrap:!s.view.wrap}};\n   if(kind===\"sort\")return {...s,view:{...s.view,sorts:s.view.sorts.length?[]:[{prop:\"title\",descending:false}]}};\n   if(kind===\"modal-remove\")return {...s,rows:s.rows.filter((r:any)=>r.id!==\"r2\"),dialog:\"unfocused\"};\n   if(kind===\"autofocus-remove\")return {...s,rows:s.rows.filter((r:any)=>r.id!==\"r2\"),autofocus:true};\n   if(kind===\"dismiss-modal\")return {...s,dialog:null,tick:s.tick+1};\n   return {...s,tick:s.tick+1};\n  })),\n  snapshot:()=>({focusTag:document.activeElement?.tagName,focusId:(document.activeElement as HTMLElement)?.id,\n   inPrimary:!!document.querySelector('[data-test=\"primary\"] .db-scroll')?.contains(document.activeElement),\n   wrapper:document.activeElement===document.querySelector('[data-test=\"primary\"] .db-scroll'),\n   connected:!!document.activeElement?.isConnected,previousConnected:!!(window as any).previousFocus?.isConnected,sourceRowCount:state.rows.length,modelHasRow2:state.rows.some((r:any)=>r.id===\"r2\"),collapsed:state.view.group?.collapsed??[],rows:Array.from(document.querySelectorAll('[data-test=\"primary\"] tr[data-row-id]')).map(x=>x.getAttribute(\"data-row-id\")),\n   depth:undoDepth(state.dbId),stats:stats.current,dbId:state.dbId,viewId:state.view.id}),\n };\n const otherView=makeView({}, \"other-v\");\n const otherDb={...db,id:\"other-\"+token,views:[otherView]};\n const otherShell={...shell,db:otherDb,view:otherView,rowById:new Map(seedRows.map(r=>[r.id,r]))};\n const otherProps={...props,db:otherDb,view:otherView,result:{...result,rows:seedRows,groups:null},rowById:otherShell.rowById};\n return <>\n  <span ref={probe} aria-hidden=\"true\"/>\n  <div data-test=\"primary\" data-database={db.id} hidden={state.hidden} onKeyDown={e=>{if(handleUndoKey(db.id,e))stats.current.rootKeys++}}>\n   {state.mounted&&<DbShellContext.Provider value={shell}><TableView {...props}/></DbShellContext.Provider>}\n  </div>\n  <div data-test=\"other\" data-database={otherDb.id} onKeyDown={e=>handleUndoKey(otherDb.id,e)}><DbShellContext.Provider value={otherShell}><TableView {...otherProps}/></DbShellContext.Provider></div>\n  {state.autofocus&&<input id=\"commit-focus-target\" autoFocus/>}\n  {!state.mounted&&<input id=\"navigation-target\" aria-label=\"New page\" autoFocus={cfg.navigate}/>}\n  {state.dialog&&<div role=\"dialog\" id=\"unfocused-modal\"><input aria-label=\"Modal input\"/></div>}\n </>;\n}\n(window as any).setup=async(cfg:any)=>{\n flushSync(()=>root.render(<StrictMode><App key={++serial} token={serial} cfg={cfg}/></StrictMode>));\n await new Promise<void>(r=>requestAnimationFrame(()=>requestAnimationFrame(()=>r())));\n};\n(window as any).settle=()=>new Promise<void>(r=>requestAnimationFrame(()=>requestAnimationFrame(()=>r())));\n(window as any).dispose=()=>flushSync(()=>root.unmount());\n(window as any).fixtureBooted=true;\n";
const sha=b=>crypto.createHash("sha256").update(b).digest("hex").toUpperCase();
function within(file,root){const r=path.relative(root,file);return r===""||(!r.startsWith(".."+path.sep)&&r!==".."&&!path.isAbsolute(r));}
function ordinary(p,kind){
 p=path.resolve(p);let q=p;
 for(;;){const s=fs.lstatSync(q);if(s.isSymbolicLink())throw Error("Reparse/symlink input refused");if(q===p&&kind==="file"&&!s.isFile())throw Error("Not an ordinary input file");if(q===p&&kind==="dir"&&!s.isDirectory())throw Error("Not an ordinary input directory");const up=path.dirname(q);if(up===q)break;q=up;}return p;
}
function readPin(file,expected){ordinary(file,"file");const b=fs.readFileSync(file);if(expected&&sha(b)!==expected)throw Error("Input SHA256 mismatch: "+path.basename(file));return {file:path.resolve(file),bytes:b.length,sha256:sha(b),body:b};}
function writeNew(file,value){fs.writeFileSync(file,typeof value==="string"?value:JSON.stringify(value,null,2),{flag:"wx",encoding:"utf8"});}
const variants=[
 {name:"sortable-fixed",sort:false,wrap:false},{name:"plain-fixed",sort:true,wrap:false},
 {name:"sortable-wrapped",sort:false,wrap:true},{name:"plain-wrapped",sort:true,wrap:true}
];
const CASES=[
 ...variants.flatMap(v=>["Control+y","Control+Shift+z"].map(key=>({name:"owned-remove-natural-redo/"+v.name+"/"+key,kind:"redo",cfg:v,key}))),
 {name:"filtered-focused-row",kind:"repair",action:"filter",cfg:{}},
 {name:"collapsed-group-focused-row",kind:"repair",action:"collapse",cfg:{group:"group"}},
 {name:"collapsed-subgroup-focused-row",kind:"repair",action:"subgroup",cfg:{group:"subgroup"}},
 {name:"collapsed-duplicate-occurrence-keeps-source-row",kind:"duplicate",cfg:{group:"duplicate"}},
 ...["outside-input","outside-textarea","outside-select","outside-editor","dialog-input","other-table"].map(target=>({name:"preserve-focus/"+target,kind:"external",target,cfg:{}})),
 {name:"preserve-focus/other-row-title-editor",kind:"editor",cfg:{}},
 {name:"owned-input-removal-no-duplicate-commit",kind:"input-remove",cfg:{}},
 {name:"same-commit-external-autofocus-wins",kind:"autofocus",cfg:{}},
 {name:"background-document-declines-repair",kind:"background",cfg:{}},
 {name:"navigation-autofocus",kind:"scope",action:"navigate",cfg:{navigate:true},expected:"navigation-target"},
 {name:"complete-table-unmount",kind:"scope",action:"unmount",cfg:{}},
 {name:"database-identity-change",kind:"scope",action:"scope-db",cfg:{}},
 {name:"view-identity-change",kind:"scope",action:"scope-view",cfg:{}},
 {name:"hidden-navigation-container",kind:"scope",action:"hidden",cfg:{}},
 {name:"same-props-rerender",kind:"keep",action:"rerender",cfg:{}},
 {name:"measurement-ref-churn-wrap",kind:"keep",action:"wrap",cfg:{}},
 {name:"present-key-sort-branch-change",kind:"no-repair",action:"sort",cfg:{}},
 {name:"virtual-scroll-keeps-owned-cell",kind:"scroll",cfg:{}},
 {name:"unfocused-open-dialog-cancels-pending-repair",kind:"modal",cfg:{}},
 {name:"removing-another-row-keeps-focus",kind:"keep",action:"other-remove",cfg:{}}
];

// The automatic JSX transform evaluates token={serial} before key={++serial}.
// Each fresh editor mount therefore uses the zero-based case index in its database ID.
const expectedEditing=CASES.flatMap((c,index)=>["editor","input-remove"].includes(c.kind)
 ? [{id:"db-"+index,active:true},{id:"db-"+index,active:false}] : []);
function installFixtureIpc(expectedEditing){
 const event="items-changed";
 const callbackId=1;
 const listenerId=1;
 const unexpected=[];
 const editing=[];
 let callbackRegistered=false;
 let listening=false;
 let subscriptions=0;
 const exactKeys=(value,keys)=>value!==null&&typeof value==="object"
  &&Object.keys(value).sort().join(",")===keys;
 function deny(operation){
  if(unexpected.length<64)unexpected.push(operation);
  throw Error("Unexpected Tauri fixture operation: "+operation);
 }
 window.__unexpectedIPC=unexpected;
 window.__fixtureIPCReadback=()=>({callbackRegistered,subscriptions,listening,editing:[...editing],unexpected:[...unexpected]});
 window.__TAURI_INTERNALS__={
  transformCallback(callback,once){
   if(callbackRegistered||typeof callback!=="function"||once!==false)return deny("transformCallback");
   callbackRegistered=true;
   return callbackId;
  },
  async invoke(command,args,options){
   if(options!==undefined)return deny(String(command));
   if(command==="plugin:event|listen"&&subscriptions===0&&callbackRegistered
     &&exactKeys(args,"event,handler,target")&&args.event===event&&args.handler===callbackId
     &&exactKeys(args.target,"kind")&&args.target.kind==="Any"){
    subscriptions++;
    listening=true;
    return listenerId;
   }
   if(command==="editing"&&exactKeys(args,"active,id")){
    const expected=expectedEditing[editing.length];
    if(expected&&args.id===expected.id&&args.active===expected.active){
     // Native editing() updates only an in-memory coordination lease. The fixture
     // acknowledges the advisory notice without dispatching, persisting or emitting anything.
     editing.push({id:args.id,active:args.active});
     return;
    }
   }
   return deny(String(command));
  }
 };
 window.__TAURI_EVENT_PLUGIN_INTERNALS__={
  unregisterListener(){return deny("unregisterListener");}
 };
}
function checkFixtureIpc(traffic,editing,label){
 assert.equal(traffic.callbackRegistered,true,label+" callback registration");
 assert.equal(traffic.subscriptions,1,label+" items-changed subscription");
 assert.equal(traffic.listening,true,label+" items-changed subscription remains live until document close");
 assert.deepEqual(traffic.editing,editing,label+" editing notices");
}
async function main(){
 const args=process.argv.slice(2);
 if(args.length!==5)throw Error("Expected PW_MODULE PW_SHA256 CHROME_EXE CHROME_SHA256 FRESH_OUT");
 const [pwArg,pwSHA256,chromeArg,chromeSHA256,outArg]=args;
 if(![pwArg,chromeArg,outArg].every(p=>path.isAbsolute(p)))throw Error("All paths must be absolute");
 if(process.platform==="win32"&&![__filename,pwArg,chromeArg,outArg].every(p=>/^[A-Za-z]:[\\/]/.test(p))){
  throw Error("Windows inputs and output must use local drive paths");
 }
 if(![pwSHA256,chromeSHA256].every(digest=>/^[0-9a-f]{64}$/i.test(digest)))throw Error("Expected exact Playwright and Chrome SHA-256 pins");
 if(process.env.NODE_OPTIONS)throw Error("NODE_OPTIONS is forbidden");
 ordinary(__filename,"file");
 const repo=ordinary(path.resolve(__dirname,".."),"dir");
 const pw=ordinary(pwArg,"file"),chrome=ordinary(chromeArg,"file"),out=path.resolve(outArg);
 if(within(out,repo)||path.dirname(out)!==path.parse(out).root||!/^nq1-row-focus-[A-Za-z0-9-]+$/.test(path.basename(out))||fs.existsSync(out))throw Error("Output must be a fresh short drive-root nq1-row-focus-* directory outside the repository");
 const srcRoot=ordinary(path.join(repo,"src"),"dir"),modulesRoot=ordinary(path.join(repo,"node_modules"),"dir");
 const canonical=path.join(srcRoot,"features/database/views/TableView.tsx");
 const contextFile=path.join(srcRoot,"features/database/context.tsx"),undoFile=path.join(srcRoot,"features/database/undo.ts");
 const table=readPin(canonical),contextPin=readPin(contextFile),undoPin=readPin(undoFile);
 const req=createRequire(path.join(repo,"package.json"));
 function resolveInstalled(name){
  const resolved=ordinary(req.resolve(name),"file");
  if(!within(resolved,modulesRoot))throw Error(`Dependency outside checkout node_modules: ${name}`);
  return resolved;
 }
 const viteEntry=resolveInstalled("vite"),reactPluginEntry=resolveInstalled("@vitejs/plugin-react");
 const pwPin=readPin(pw,pwSHA256.toUpperCase()),chromePin=readPin(chrome,chromeSHA256.toUpperCase());
 const pins=[table,contextPin,undoPin,readPin(viteEntry),readPin(reactPluginEntry),pwPin,chromePin];
 fs.mkdirSync(out);ordinary(out,"dir");
 for(const d of ["temp","cache","downloads","traces","browser-artifacts"])fs.mkdirSync(path.join(out,d));
 // Playwright owns its temporary profile. The caller supplies the remaining isolated environment.
 process.env.TMP=path.join(out,"temp");process.env.TEMP=path.join(out,"temp");
 const result={scope:"table-removed-row-focus-standalone-v1",startedUtc:new Date().toISOString(),nodePID:process.pid,
  tableView:{file:table.file,sha256:table.sha256,bytes:table.bytes},pins:pins.map(({body,...pin})=>pin),
  expectedNames:CASES.map(c=>c.name),expectedCount:CASES.length,cases:[],pageErrors:[],consoleErrors:[],consoleWarnings:[],unexpectedNetwork:[],unexpectedIPC:[],
  moduleGraph:[],tableLoadCount:0,contextCloseObserved:false,browserCloseObserved:false,serverClosed:false,profile:{kind:"playwright-owned-temporary",root:path.join(out,"temp"),path:null,observedInsideOwnedTemp:false,autoCleanupAllowed:true,retentionClaim:false,existsAfterClose:null},
  fixtureIPC:{primary:null,background:null},backgroundFocus:null,focusEmulationDisabled:[],
  fullQ1Qualified:false,nativePersistenceQualified:false,ownerAccepted:false,passed:false};
 let context=null,browser=null,server=null,origin="",page=null;
 try{
  const {build}=await import(pathToFileURL(viteEntry).href),react=(await import(pathToFileURL(reactPluginEntry).href)).default;
  const slash=p=>p.replaceAll("\\","/");
  const entry=FIXTURE.replaceAll("__TABLE_MODULE__",slash(canonical)).replaceAll("__CONTEXT_MODULE__",slash(contextFile)).replaceAll("__UNDO_MODULE__",slash(undoFile));
  writeNew(path.join(out,"fixture.entry.tsx"),entry);
  const virtual="\0q1-removed-row-fixture.tsx",seen=new Set();
  const fixtureModules={name:"bounded-table-fixture",enforce:"pre",
   resolveId(id){if(id==="virtual:q1-removed-row-fixture.tsx")return virtual;},
   load(id){
    if(id===virtual)return entry;
    const clean=id.split("?")[0];
    if(path.isAbsolute(clean)){
     const p=path.resolve(clean);
     if(!within(p,srcRoot)&&!within(p,modulesRoot))throw Error("Module outside approved source/dependency roots");
     ordinary(p,"file");
     if(!seen.has(p)){seen.add(p);if(seen.size>4096)throw Error("Module graph bound exceeded");const s=fs.statSync(p);if(s.size>4*1024*1024)throw Error("Module input size bound exceeded");result.moduleGraph.push({file:p,bytes:s.size});}
     // Load the canonical TableView bytes captured from this checkout.
     if(p.toLowerCase()===canonical.toLowerCase()){result.tableLoadCount++;return table.body.toString("utf8");}
    }
   }};
  const built=await build({configFile:false,envFile:false,mode:"development",define:{"process.env.NODE_ENV":JSON.stringify("development")},root:out,publicDir:false,cacheDir:path.join(out,"cache"),logLevel:"warn",
   plugins:[fixtureModules,react()],resolve:{alias:[
    {find:"@",replacement:srcRoot},
    {find:/^react$/,replacement:resolveInstalled("react")},
    {find:/^react\/jsx-runtime$/,replacement:resolveInstalled("react/jsx-runtime")},
    {find:/^react\/jsx-dev-runtime$/,replacement:resolveInstalled("react/jsx-dev-runtime")},
    {find:/^react-dom$/,replacement:resolveInstalled("react-dom")},
    {find:/^react-dom\/client$/,replacement:resolveInstalled("react-dom/client")}
   ],preserveSymlinks:true},
   build:{write:false,outDir:path.join(out,"bundle"),emptyOutDir:false,target:"chrome120",minify:false,sourcemap:false,
    rollupOptions:{input:"virtual:q1-removed-row-fixture.tsx",output:{entryFileNames:"entry.js",chunkFileNames:"chunk-[hash].js",assetFileNames:"asset-[hash][extname]"}}}});
  assert.equal(result.tableLoadCount,1,"TableView must load at its canonical production module ID");
  const assets=new Map();let assetBytes=0;
  for(const b of Array.isArray(built)?built:[built])for(const item of b.output){
   const data=Buffer.from(item.type==="chunk"?item.code:item.source);assetBytes+=data.length;
   if(assets.has("/"+item.fileName)||data.length>8*1024*1024||assetBytes>16*1024*1024||assets.size>=256)throw Error("Bundle output bound exceeded");
   assets.set("/"+item.fileName,data);
  }
  if(!assets.has("/entry.js"))throw Error("Missing fixture entry output");
  fs.mkdirSync(path.join(out,"bundle"));
  for(const [name,data]of assets){const file=path.join(out,"bundle",name.slice(1));if(!within(file,path.join(out,"bundle")))throw Error("Unsafe bundle path");fs.mkdirSync(path.dirname(file),{recursive:true});fs.writeFileSync(file,data,{flag:"wx"});}
  const html='<!doctype html><html><head><meta charset="utf-8"><link rel="icon" href="data:,"><style>body{font:14px sans-serif}table{border-collapse:collapse}td{padding:3px}.db-scroll{height:240px;max-height:240px!important;width:680px;overflow:auto}tr[data-row-id]{height:35px}thead{height:33px}[data-test=other]{margin-top:12px}#external-dialog{border:1px solid;display:none}</style></head><body><input id="outside-input"><textarea id="outside-textarea"></textarea><select id="outside-select"><option>Fixture</option></select><div id="outside-editor" contenteditable="true" tabindex="0">External editor</div><div id="external-dialog" role="dialog"><input id="dialog-input"></div><div class="ProseMirror" contenteditable="true"><span id="outer-note-marker">Original note marker</span><div id="mount" contenteditable="false"></div></div><script type="module" src="/entry.js"></script></body></html>';
  writeNew(path.join(out,"fixture.html"),html);
  server=http.createServer((request,response)=>{
   const url=new URL(request.url||"/","http://127.0.0.1");
   const data=url.pathname==="/"?Buffer.from(html):assets.get(url.pathname);
   if(request.method!=="GET"||url.search||!data){response.writeHead(404);response.end();return;}
   response.writeHead(200,{"Content-Type":url.pathname==="/"?"text/html; charset=utf-8":url.pathname.endsWith(".css")?"text/css":"text/javascript; charset=utf-8","Cache-Control":"no-store","Content-Security-Policy":"default-src 'none'; script-src 'self'; style-src 'unsafe-inline' 'self'; img-src data:; connect-src 'none'; base-uri 'none'; form-action 'none'"});response.end(data);
  });
  await new Promise((resolve,reject)=>{server.once("error",reject);server.listen(0,"127.0.0.1",resolve);});
  origin="http://127.0.0.1:"+server.address().port;result.loopbackOrigin=origin;
  const {chromium}=require(pw);
  browser=await chromium.launch({executablePath:chrome,headless:true,chromiumSandbox:true,timeout:10000,
   args:["--no-first-run","--no-default-browser-check","--disable-background-networking","--disable-component-update","--disable-sync","--disable-extensions"],
   artifactsDir:path.join(out,"browser-artifacts"),downloadsPath:path.join(out,"downloads"),tracesDir:path.join(out,"traces"),
   handleSIGINT:false,handleSIGTERM:false,handleSIGHUP:false,ignoreDefaultArgs:["--hide-scrollbars"],env:{...process.env}});
  browser.once("disconnected",()=>{result.browserCloseObserved=true;});
  const tempEntries=fs.readdirSync(path.join(out,"temp"),{withFileTypes:true});
  assert.ok(tempEntries.length<=64,"Owned temporary-directory entry bound");
  const profiles=tempEntries.filter(d=>d.isDirectory()&&d.name.startsWith("playwright_chromiumdev_profile-")).map(d=>path.join(out,"temp",d.name));
  assert.equal(profiles.length,1,"Exactly one actual Playwright-owned temporary Chromium profile");
  ordinary(profiles[0],"dir");assert.ok(within(profiles[0],result.profile.root));
  result.profile.path=profiles[0];result.profile.observedInsideOwnedTemp=true;
  context=await browser.newContext({serviceWorkers:"block"});
  context.once("close",()=>{result.contextCloseObserved=true;});
  await context.route("**/*",async route=>{const request=route.request();const u=new URL(request.url());if(u.origin===origin&&request.method()==="GET")return route.continue();if(result.unexpectedNetwork.length<32)result.unexpectedNetwork.push({protocol:u.protocol,host:u.host,method:request.method()});await route.abort();});
  await context.addInitScript(installFixtureIpc,expectedEditing);
  // Playwright 1.65 enables focus emulation for each main frame in this context.
  // Disable it on the loaded page without changing the page's native focus signals.
  async function disableFocusEmulation(target,label){
   const session=await context.newCDPSession(target);
   await session.send("Emulation.setFocusEmulationEnabled",{enabled:false});
   result.focusEmulationDisabled.push(label);
  }
  page=context.pages()[0]||await context.newPage();page.setDefaultTimeout(2500);
  const observePage=p=>{
   p.on("pageerror",e=>{if(result.pageErrors.length<32)result.pageErrors.push(String(e.stack||e));});
   p.on("console",m=>{const a=m.type()==="error"?result.consoleErrors:m.type()==="warning"?result.consoleWarnings:null;if(a&&a.length<64)a.push(m.text());});
  };
  observePage(page);context.on("page",observePage);
  await page.goto(origin+"/",{waitUntil:"load",timeout:15000});
  await disableFocusEmulation(page,"primary");
  await page.waitForFunction(()=>window.fixtureBooted);
  const snap=()=>page.evaluate(()=>window.fixture.snapshot());
  const settle=()=>page.evaluate(()=>window.settle());
  const cell=()=>page.locator('[data-test="primary"] tr[data-row-id="r2"] [role="gridcell"]').first();
  const remember=()=>page.evaluate(()=>{window.previousFocus=document.activeElement;});
  async function focusRow2(){await cell().click({position:{x:80,y:12}});await page.waitForFunction(()=>document.activeElement?.closest('[data-test="primary"] tr[data-row-id="r2"]'));await remember();}
  async function mutate(kind){await page.evaluate(k=>window.fixture.mutate(k),kind);await settle();}
  function common(s){assert.ok(s.stats.probe.attach>=2&&s.stats.probe.detach>=1,"Actual development StrictMode ref lifecycle must be observed");assert.equal(s.stats.writes.length,0,"Focus repair must not submit a cell/title write");assert.equal(s.stats.unexpected.length,0,"Unexpected shell operation");}
  for(const c of CASES){
   const started=Date.now();let before=null,after=null,removed=null;
   try{
    await page.evaluate(async cfg=>{document.getElementById("external-dialog").style.display="none";await window.setup(cfg);},c.cfg);
    if(c.kind==="duplicate"){
     await page.locator('[data-test="primary"] .db-scroll').evaluate(el=>{el.scrollTop=1e6;});await settle();
     await page.locator('[data-test="primary"] tr[data-row-id="r2"] [role="gridcell"]').last().click({position:{x:80,y:12}});await remember();before=await snap();
     await mutate("collapse");after=await snap();assert.equal(after.previousConnected,false);assert.equal(after.wrapper,true);assert.equal(after.modelHasRow2,true);assert.deepEqual(after.collapsed,["g2"]);
    }else{
     await focusRow2();before=await snap();
     if(c.kind==="redo"){
      await page.keyboard.press("Control+z");await page.waitForFunction(()=>!document.querySelector('[data-test="primary"] tr[data-row-id="r2"]'));await settle();
      removed=await snap();
      // Both source versions receive the natural redo key; no focus is supplied after removal.
      await page.keyboard.press(c.key);await settle();after=await snap();
      assert.equal(removed.wrapper,true,"Removed owned row must naturally leave focus on its same table wrapper");
      assert.deepEqual(removed.depth,{undo:0,redo:1});assert.equal(removed.stats.undo,1);common(removed);
      assert.equal(after.modelHasRow2,true);assert.equal(after.stats.redo,1);assert.equal(after.stats.rootKeys,2);assert.deepEqual(after.depth,{undo:1,redo:0});
     }else if(c.kind==="repair"){await mutate(c.action);after=await snap();assert.equal(after.previousConnected,false);assert.equal(after.wrapper,true);}
     else if(c.kind==="external"){
      if(c.target==="dialog-input")await page.locator("#external-dialog").evaluate(el=>{el.style.display="block";});
      const target=c.target==="other-table"?page.locator('[data-test="other"] tr[data-row-id="r1"] [role="gridcell"]').first():page.locator("#"+c.target);
      if(c.target==="other-table")await target.click({position:{x:80,y:12}});else await target.focus();
      await remember();await mutate("remove");after=await snap();assert.equal(after.previousConnected,true);assert.equal(after.wrapper,false);assert.equal(await target.evaluate(el=>el===document.activeElement),true);
     }else if(c.kind==="editor"){
      const r1=page.locator('[data-test="primary"] tr[data-row-id="r1"] [role="gridcell"]').first();await r1.click({position:{x:80,y:12}});await page.keyboard.press("F2");
      const input=page.locator('[data-test="primary"] tr[data-row-id="r1"] input').first();await input.waitFor();await page.waitForFunction(()=>document.activeElement?.closest('[data-row-id="r1"]')?.querySelector("input")===document.activeElement);
      await remember();await mutate("remove");after=await snap();assert.equal(await input.evaluate(el=>el===document.activeElement),true);assert.equal(after.previousConnected,true);assert.equal(after.wrapper,false);
     }else if(c.kind==="input-remove"){
      await page.keyboard.press("F2");await page.waitForFunction(()=>document.activeElement?.closest('[data-row-id="r2"]')?.querySelector("input")===document.activeElement);
      await remember();await mutate("remove");after=await snap();assert.equal(after.previousConnected,false);assert.equal(after.wrapper,true);
     }else if(c.kind==="autofocus"){
      await mutate("autofocus-remove");after=await snap();assert.equal(after.focusId,"commit-focus-target");assert.equal(after.wrapper,false);
     }else if(c.kind==="background"){
      result.backgroundFocus={primaryBefore:await page.evaluate(()=>document.hasFocus()),primaryAfter:null,otherAfter:null};
      const otherPage=await context.newPage();
      try{
       await otherPage.goto(origin+"/",{waitUntil:"load",timeout:15000});
       await disableFocusEmulation(otherPage,"background");
       await otherPage.bringToFront();
       result.backgroundFocus.primaryAfter=await page.evaluate(()=>document.hasFocus());
       result.backgroundFocus.otherAfter=await otherPage.evaluate(()=>document.hasFocus());
       assert.equal(await page.evaluate(()=>document.hasFocus()),false,"Background precondition must be observed");
       assert.equal(result.backgroundFocus.primaryBefore,true,"Primary document must have actual focus before switching");
       assert.equal(result.backgroundFocus.otherAfter,true,"Other document must have actual focus after switching");
       await mutate("remove");after=await snap();assert.equal(after.wrapper,false);
      }finally{
       try{result.fixtureIPC.background=await otherPage.evaluate(()=>window.__fixtureIPCReadback());}
       finally{await otherPage.close();await page.bringToFront();}
      }
     }else if(c.kind==="scope"){
      await mutate(c.action);after=await snap();assert.equal(after.wrapper,false);if(c.expected)assert.equal(after.focusId,c.expected);
     }else if(c.kind==="keep"){
      await mutate(c.action);after=await snap();assert.equal(after.previousConnected,true);assert.equal(await page.evaluate(()=>document.activeElement===window.previousFocus),true);
     }else if(c.kind==="no-repair"){
      await mutate(c.action);after=await snap();assert.equal(after.modelHasRow2,true);assert.equal(after.wrapper,false,"A still-present item key must not claim removal ownership");
     }else if(c.kind==="scroll"){
      await page.locator('[data-test="primary"] .db-scroll').evaluate(el=>{el.scrollTop=1e6;});await settle();after=await snap();
      assert.equal(after.previousConnected,true);assert.equal(await page.evaluate(()=>document.activeElement===window.previousFocus),true);assert.ok(after.rows.length<80,"Actual virtualization remains bounded");
     }else if(c.kind==="modal"){
      await mutate("modal-remove");after=await snap();assert.equal(after.focusTag,"BODY");assert.equal(after.wrapper,false);
      await mutate("dismiss-modal");const later=await snap();assert.equal(later.focusTag,"BODY");assert.equal(later.wrapper,false,"Discarded pending focus must not replay");after=later;
     }else throw Error("Undeclared case kind");
    }
    common(after);assert.equal(await page.locator("#outer-note-marker").textContent(),"Original note marker");
    result.cases.push({name:c.name,passed:true,wallMs:Date.now()-started,before,removed,after});
   }catch(e){result.cases.push({name:c.name,passed:false,wallMs:Date.now()-started,error:String(e.stack||e),before,removed,after:after||await snap().catch(()=>null)});}
  }
  await page.evaluate(()=>window.dispose());await settle();
  result.fixtureIPC.primary=await page.evaluate(()=>window.__fixtureIPCReadback());
  result.unexpectedIPC=[...result.fixtureIPC.primary.unexpected,...(result.fixtureIPC.background?.unexpected??[])];
  assert.equal(new Set(result.cases.map(c=>c.name)).size,CASES.length);
  assert.deepEqual(result.cases.map(c=>c.name),result.expectedNames);
  checkFixtureIpc(result.fixtureIPC.primary,expectedEditing,"Primary");
  assert.ok(result.fixtureIPC.background,"Background fixture IPC audit must be recorded");
  checkFixtureIpc(result.fixtureIPC.background,[],"Background");
  result.passed=result.cases.length===CASES.length&&result.cases.every(c=>c.passed)&&!result.pageErrors.length&&!result.consoleErrors.length&&!result.unexpectedNetwork.length&&!result.unexpectedIPC.length;
 }catch(e){result.failure=String(e.stack||e);result.passed=false;}
 finally{
   if(context)try{await context.close();result.contextCloseObserved=true;}catch(e){result.contextCloseError=String(e);result.passed=false;}
   if(browser)try{await browser.close();result.browserCloseObserved=!browser.isConnected();}catch(e){result.browserCloseError=String(e);result.passed=false;}
   if(result.profile.path)result.profile.existsAfterClose=fs.existsSync(result.profile.path);
  if(server)try{server.closeIdleConnections?.();await new Promise((resolve,reject)=>server.close(e=>e?reject(e):resolve()));result.serverClosed=true;}catch(e){result.serverCloseError=String(e);result.passed=false;}
  for(const p of pins)try{if(readPin(p.file).sha256!==p.sha256)throw Error("Source changed");}catch(e){result.passed=false;(result.postPinFailures??=[]).push(path.basename(p.file));}
  result.finishedUtc=new Date().toISOString();result.completedCount=result.cases.length;result.passCount=result.cases.filter(c=>c.passed).length;result.failCount=result.cases.filter(c=>!c.passed).length;
  if(!result.contextCloseObserved||!result.browserCloseObserved||!result.serverClosed
     ||!result.profile.observedInsideOwnedTemp||result.profile.existsAfterClose!==false)result.passed=false;
  writeNew(path.join(out,"browser-results.json"),result);
  console.log(JSON.stringify({scope:result.scope,passed:result.passed,expected:result.expectedCount,completed:result.completedCount,pass:result.passCount,fail:result.failCount,contextCloseObserved:result.contextCloseObserved,browserCloseObserved:result.browserCloseObserved,serverClosed:result.serverClosed,resultPath:path.join(out,"browser-results.json")}));
  process.exitCode=result.passed?0:1;
 }
}
main().catch(e=>{console.error(String(e.stack||e));process.exitCode=1;});

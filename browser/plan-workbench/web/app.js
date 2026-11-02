const $=id=>document.getElementById(id),encoder=new TextEncoder();
const labels={plan:'RPLAN plan',context:'RCTX context',profile:'Explicit profile',certificate:'Certificate',lineage:'Lineage'};
const texts=Object.fromEntries(Object.keys(labels).map(key=>[key,''])),present=new Set(),originals=new Map();
let wanted=['plan-shape','population','contiguity'],info,worker,ready=false,generation=0,job=0,revision=0,pending,accepted;
let readToken=0,reading=false,readName='',readCancel;
function status(message){$('status').textContent=reading?`Reading ${readName}…`:message;}
function enabled(){ $('run').disabled=!ready||!!pending||reading;$('open-file').disabled=!info; }
function node(tag,text,cls){const result=document.createElement(tag);if(text!==undefined)result.textContent=text;if(cls)result.className=cls;return result;}
function download(name,data){const url=URL.createObjectURL(new Blob([data],{type:'application/octet-stream'})),a=document.createElement('a');a.href=url;a.download=name;document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);}
function requestObject(){
  const mode=$('mode').value,request={mode};
  for(const field of (mode==='audit'?['plan','context','profile','lineage']:['plan','context','certificate']))if(present.has(field))request[field]=texts[field];
  if(mode==='audit'){request.constraints=[...wanted];request.generated_at=$('generated-at').value;}
  return request;
}
function requestText(){return JSON.stringify(requestObject(),null,2);}
function draftState(){ $('draft-state').textContent=accepted?(accepted.request===requestText()?'Draft matches the accepted input snapshot.':'Draft differs from the accepted input snapshot. Check again to replace the accepted report.'):'No accepted report yet. Current inputs remain editable and exportable.';enabled(); }
function cancelRead(message){readToken++;readCancel?.();readCancel=undefined;const previous=reading;reading=false;if(previous&&message)status(message);enabled();}
function changed(message){revision++;cancelRead(message??'Older file read discarded after a newer draft edit.');draftState();if(pending)status('Draft changed; the pending older result will be discarded.');}
function editor(){const field=$('field').value;$('editor-label').textContent=`Edit ${labels[field]}`;$('editor').value=texts[field];$('byte-count').textContent=`${encoder.encode(texts[field]).length} UTF-8 bytes · ${present.has(field)?'input present':'input omitted'}. Editing normalizes line endings; opened originals stay separate.`;}
function originalsView(){
  $('originals').replaceChildren();
  for(const [field,original] of originals){const li=node('li');li.append(node('p',`${labels[field]}: ${original.name} (${original.bytes.byteLength} bytes)`));const button=node('button','Download exact opened bytes');button.addEventListener('click',()=>download(`original-${field}-${original.name.replace(/[\\/]/g,'_')}`,original.bytes));li.append(button);$('originals').append(li);}
  if(!originals.size)$('originals').append(node('li','No user files opened. Synthetic examples are not presented as opened originals.'));
}
function modeView(){
  const audit=$('mode').value==='audit';$('audit-options').hidden=!audit;
  for(const option of $('field').options)option.disabled=audit?option.value==='certificate':['profile','lineage'].includes(option.value);
  if($('field').selectedOptions[0].disabled)$('field').value=audit?'plan':'certificate';editor();
}
function constraintsView(){
  $('constraints').replaceChildren();
  for(const name of info.constraints){const label=node('label'),input=node('input');input.type='checkbox';input.value=name;input.checked=wanted.includes(name);input.addEventListener('change',()=>{wanted=[...$('constraints').querySelectorAll('input:checked')].map(x=>x.value);changed();});label.append(input,document.createTextNode(name));$('constraints').append(label);}
}
function sourceView(){if(!accepted)return;const request=accepted.input;$('accepted-source').textContent=request[$('accepted-field').value]??'';}
function renderAccepted(){
  const report=accepted.report,certificate=report.certificate;
  $('result-summary').textContent=report.mode==='audit'?`Native audit result: ${certificate.result}. This describes the requested checks, not legal approval or a readiness certificate.`:`Native identity verification succeeded. The supplied certificate declares result: ${report.verification.result}. The audit was not rerun. Supplied plan compared: ${report.document?'yes':'no'}; context compared: ${report.context?'yes':'no'}.`;
  $('identities').replaceChildren();
  for(const [name,value] of [[report.document?'Computed supplied-plan hash':'Certificate-declared plan hash (no supplied plan)',report.plan_hash??certificate.plan_hash],['Supplied-context hash',report.context?.context_hash],['Declared unit universe',report.document?.plan.units.unit_universe_hash],['Separately computed unit universe',report.computed_unit_universe_hash]])if(value!==undefined&&value!==null)$('identities').append(node('p',`${name}: ${value}`));
  $('result-checks').replaceChildren();
  $('result-checks').append(node('p',report.mode==='audit'?`Requested constraints: ${report.requested_constraints.join(', ')||'(none beyond mandatory plan-shape)'}`:'Stored certificate checks below are caller content; verification did not execute them.','hint'));
  for(const check of certificate.checks){const row=node('div',undefined,'check');row.dataset.status=check.status;row.append(node('strong',check.name),node('span',check.status,'badge'),node('span',check.severity,'badge'),node('p',check.summary));$('result-checks').append(row);}
  $('result-warnings').replaceChildren();
  if(certificate.warnings.length){const warnings=node('div',undefined,'warnings');warnings.append(node('strong','Native warnings'));for(const warning of certificate.warnings)warnings.append(node('p',`${warning.code}: ${warning.message}`));$('result-warnings').append(warnings);}
  $('certificate-source').textContent=report.certificate_json;$('report-source').textContent=accepted.raw;
  $('accepted-field').replaceChildren();for(const field of Object.keys(labels))if(typeof accepted.input[field]==='string'){const option=node('option',labels[field]);option.value=field;$('accepted-field').append(option);}sourceView();
  for(const id of ['accepted-request','accepted-report','certificate-download','load-certificate'])$(id).disabled=false;
  $('normalized-plan').disabled=typeof report.normalized_plan!=='string';$('normalized-context').disabled=typeof report.normalized_context!=='string';
}
function beginWorker(){
  worker?.terminate();generation++;const mine=generation;ready=false;clearTimeout(pending?.timer);pending=undefined;enabled();status('Starting Rust… Accepted results remain available.');
  return new Promise((resolve,reject)=>{
    let startup;
    function failure(message){if(mine!==generation)return;clearTimeout(startup);clearTimeout(pending?.timer);pending=undefined;ready=false;worker?.terminate();generation++;status(`Engine unavailable: ${message}. Retry preserves current inputs and accepted exports.`);enabled();reject(new Error(message));}
    startup=setTimeout(()=>failure('12-second startup deadline'),12000);
    try{worker=new Worker(new URL('./worker.js',import.meta.url),{type:'module'});}catch(error){failure(String(error));return;}
    worker.onerror=event=>{event.preventDefault();failure(event.message||'worker exception');};
    worker.onmessage=({data})=>{
      if(mine!==generation)return;
      if(data.type==='ready'){
        try{info=JSON.parse(data.info);if(!Array.isArray(info.constraints)||!info.file_limits)throw new Error('Invalid Rust metadata');}
        catch(error){failure(String(error));return;}
        clearTimeout(startup);ready=true;constraintsView();$('limits').textContent='Rust admission: plan/context/certificate 256 KiB each; profile/lineage 32 KiB; 2048 units, 128 districts, 8192 edges; 64 MiB compiled memory. Whole operations are refused, never truncated.';status('Rust ready. Check current inputs.');enabled();resolve();return;
      }
      if(data.type==='failed'){if(data.id!==undefined&&pending?.id!==data.id)return;failure(data.message??'Rust worker failed');return;}
      if(data.type!=='result'||!pending||data.id!==pending.id)return;
      const completed=pending;pending=undefined;clearTimeout(completed.timer);enabled();
      if(completed.revision!==revision){status('Older result discarded after draft changes. Accepted exports remain available.');return;}
      try{
        if(typeof data.report!=='string'||encoder.encode(data.report).length>info.report_bytes)throw new Error('Worker returned an invalid or oversized report');
        const report=JSON.parse(data.report);
        if(report.kind!=='report'){status(`Native ${report.kind} (${report.phase}): ${report.message}. Accepted report retained.`);return;}
        accepted={raw:data.report,report,request:completed.request,input:completed.input};renderAccepted();draftState();status('Rust report accepted. Supplied profiles, provenance and review/release conclusions remain unverified.');
      }catch(error){failure(String(error));}
    };
  });
}
function runCurrent(){
  if(!ready||pending||reading)return;
  const request=requestText();
  if(encoder.encode(request).length>info.request_bytes){status('Current request exceeds 1 MiB. Inputs and accepted exports retained.');return;}
  const id=++job,mine=generation;
  const timer=setTimeout(()=>{if(mine!==generation||pending?.id!==id)return;worker.terminate();generation++;ready=false;pending=undefined;enabled();status('10-second computation deadline. Accepted exports and current drafts retained; retry the engine.');},10000);
  pending={id,timer,revision,request,input:requestObject()};enabled();status('Rust is checking the current input snapshot…');worker.postMessage({id,request});
}
async function openFile(file){
  if(!info)return;const field=$('field').value;cancelRead();
  if(file.size>info.file_limits[field]){status(`File exceeds the Rust ${labels[field]} byte limit. Draft and originals retained.`);return;}
  const token=++readToken;revision++;reading=true;readName=file.name;status('');enabled();
  let timer;
  try{
    const bytes=await Promise.race([file.arrayBuffer(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(new Error('5-second file read deadline')),5000);readCancel=()=>{clearTimeout(timer);reject(new Error('Read superseded'));};})]);
    if(token!==readToken)return;
    if(bytes.byteLength>info.file_limits[field])throw new Error('Read bytes exceed the file limit');
    const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);
    texts[field]=text;present.add(field);originals.set(field,{name:file.name,bytes});revision++;reading=false;readCancel=undefined;editor();originalsView();draftState();status(`Opened ${file.name}. Check current inputs to replace the accepted report.`);
  }catch(error){if(token===readToken){reading=false;readCancel=undefined;status(`${String(error.message??error)}. Draft and originals retained.`);}}
  finally{clearTimeout(timer);if(token===readToken){reading=false;readCancel=undefined;enabled();}}
}
async function loadExample(){
  const name=$('example').value;if(!['example.json','disconnected.json'].includes(name))return;
  cancelRead();const token=++readToken,startRevision=revision,abort=new AbortController();reading=true;readName='synthetic example';status('');enabled();
  const timer=setTimeout(()=>abort.abort(),5000);readCancel=()=>abort.abort();
  try{
    const response=await fetch(new URL(name,import.meta.url),{signal:abort.signal});if(!response.ok)throw new Error(`Example HTTP ${response.status}`);
    const chunks=[],reader=response.body.getReader();let size=0;
    while(true){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>1048576){abort.abort();throw new Error('Example exceeds 1 MiB');}chunks.push(value);}
    const buffer=new Uint8Array(size);let offset=0;for(const chunk of chunks){buffer.set(chunk,offset);offset+=chunk.byteLength;}
    const example=JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(buffer));
    if(token!==readToken||startRevision!==revision)return false;
    for(const field of Object.keys(labels)){if(typeof example[field]==='string'){texts[field]=example[field];present.add(field);}else{texts[field]='';present.delete(field);}}
    $('mode').value='audit';$('generated-at').value=example.generated_at;wanted=example.constraints;revision++;reading=false;readCancel=undefined;modeView();if(info)constraintsView();draftState();status('Synthetic example loaded. Its source/profile assertions are fixture data, not authenticated evidence.');return true;
  }catch(error){if(token===readToken){reading=false;status(`Example unavailable: ${String(error.message??error)}. Current inputs and accepted report retained.`);}return false;}
  finally{clearTimeout(timer);if(token===readToken){reading=false;readCancel=undefined;enabled();}}
}
$('editor').addEventListener('input',()=>{const field=$('field').value;texts[field]=$('editor').value;present.add(field);changed();$('byte-count').textContent=`${encoder.encode(texts[field]).length} UTF-8 bytes · edited text uses LF; original bytes remain separate.`;});
$('editor').addEventListener('keydown',event=>{if(event.key==='Enter'&&(event.ctrlKey||event.metaKey)){event.preventDefault();runCurrent();}});
$('field').addEventListener('change',()=>{cancelRead('Input selection changed; older file read discarded.');editor();});
$('mode').addEventListener('change',()=>{changed();modeView();});$('generated-at').addEventListener('input',()=>changed());
$('open-file').addEventListener('change',event=>{const file=event.target.files[0];event.target.value='';if(file)void openFile(file);});
$('clear-field').addEventListener('click',()=>{const field=$('field').value;present.delete(field);texts[field]='';changed();editor();status('Input omitted from the next request. Opened original retained.');});
$('run').addEventListener('click',runCurrent);
$('retry').addEventListener('click',()=>{void beginWorker().then(runCurrent).catch(()=>{});});
$('load-example').addEventListener('click',()=>{void loadExample().then(loaded=>{if(loaded)runCurrent();});});
$('current-file').addEventListener('click',()=>download(`current-${$('field').value}.json`,texts[$('field').value]));
$('current-request').addEventListener('click',()=>download('current-request.json',requestText()));
$('accepted-request').addEventListener('click',()=>accepted&&download('accepted-request.json',accepted.request));
$('accepted-report').addEventListener('click',()=>accepted&&download('native-report.json',accepted.raw));
$('certificate-download').addEventListener('click',()=>accepted&&download('native-certificate.json',accepted.report.certificate_json));
$('normalized-plan').addEventListener('click',()=>accepted&&download('normalized-plan.rplan',accepted.report.normalized_plan));
$('normalized-context').addEventListener('click',()=>accepted&&download('normalized-context.rctx',accepted.report.normalized_context));
$('load-certificate').addEventListener('click',()=>{texts.certificate=accepted.report.certificate_json;present.add('certificate');$('mode').value='verify';$('field').value='certificate';changed();modeView();status('Accepted certificate loaded into the verification draft. Check it against the current plan/context.');});
$('accepted-field').addEventListener('change',sourceView);
modeView();originalsView();draftState();
const boot=beginWorker().then(()=>true).catch(()=>false),example=loadExample();
void Promise.all([boot,example]).then(([started,loaded])=>{if(started&&loaded)runCurrent();});

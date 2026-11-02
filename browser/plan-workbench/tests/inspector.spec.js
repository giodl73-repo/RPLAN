import {test,expect} from '@playwright/test';
import fs from 'node:fs';
import {open,check,edit,download,accepted} from './helpers.js';
const cases=JSON.parse(fs.readFileSync(new URL('../.cache/native-cases.json',import.meta.url),'utf8'));

test('a supplied plan edit exposes the native contiguity failure and complete witnesses',async({page,request})=>{
  await open(page);await expect(page.locator('#result-summary')).toContainText('Native audit result: pass.');
  const disconnected=await(await request.get('disconnected.json')).json();await edit(page,'plan',disconnected.plan);await check(page);
  await expect(page.locator('#result-summary')).toContainText('Native audit result: fail.');await expect(page.locator('.check[data-status="fail"]').filter({hasText:'contiguity'})).toContainText('contiguity');
  const report=JSON.parse((await download(page,'accepted-report')).toString());expect(report.certificate.checks.some(c=>c.name==='contiguity'&&c.witnesses.length)).toBe(true);expect(report.scope).toContain('not legal approval');
});

test('all complete native response strings match real WASM without JavaScript numeric reserialization',async({page})=>{
  await open(page);
  const result=await page.evaluate(async records=>{
    const worker=new Worker(new URL('./worker.js',location.href),{type:'module'});
    const info=await new Promise((resolve,reject)=>{worker.onerror=reject;worker.onmessage=({data})=>data.type==='ready'?resolve(JSON.parse(data.info)):reject(data);});
    const mismatches=[];
    for(let i=0;i<records.length;i++){
      const response=await new Promise((resolve,reject)=>{worker.onmessage=({data})=>data.type==='result'&&data.id===i?resolve(data.report):reject(data);worker.postMessage({id:i,request:records[i].request});});
      if(response!==records[i].response_text)mismatches.push(i);
    }
    worker.terminate();const {default:init}=await import('./pkg/rplan_plan_workbench.js');const module=await init();module.memory.grow(1024-module.memory.buffer.byteLength/65536);let refused=false;try{module.memory.grow(1);}catch{refused=true;}
    return {cases:records.length,mismatches,maximum:module.memory.buffer.byteLength,refused,constraints:info.constraints};
  },cases);
  expect(result).toEqual({cases:36,mismatches:[],maximum:67108864,refused:true,constraints:['plan-shape','population','contiguity','splits','vra','geometry']});
});

test('accepted certificate moves into an explicit verification draft and retains exact bytes',async({page})=>{
  await open(page);const original=await download(page,'certificate-download');await page.locator('#load-certificate').click();await expect(page.locator('#mode')).toHaveValue('verify');await expect(page.locator('#editor')).toHaveValue(original.toString());
  await check(page);await expect(page.locator('#result-summary')).toContainText('audit was not rerun');expect(await download(page,'certificate-download')).toEqual(original);const input=await accepted(page);expect(input.certificate).toBe(original.toString());expect(input.profile).toBeUndefined();expect(input.constraints).toBeUndefined();
});

test('timestamp changes remain caller labels while stale content hashes refuse without replacing acceptance',async({page})=>{
  await open(page);await page.locator('#load-certificate').click();const certificate=JSON.parse(await page.locator('#editor').inputValue());certificate.generated_at_utc='caller-label-not-a-clock';await page.locator('#editor').fill(JSON.stringify(certificate));await check(page);
  const input=await download(page,'accepted-request');certificate.result='fail';await page.locator('#editor').fill(JSON.stringify(certificate));await page.locator('#run').click();await expect(page.locator('#status')).toContainText('content hash mismatch');expect(await download(page,'accepted-request')).toEqual(input);await expect(page.locator('#editor')).toHaveValue(JSON.stringify(certificate));
});

test('opened CRLF and BOM originals remain exact and editing alone normalizes current text',async({page})=>{
  await open(page);const input=await accepted(page),text=input.plan.replace(/\n/g,'\r\n'),bytes=Buffer.from(text);await page.locator('#open-file').setInputFiles({name:'own.rplan',mimeType:'application/json',buffer:bytes});await expect(page.locator('#status')).toContainText('Opened own.rplan');
  await page.locator('details:has(#originals) > summary').click();const event=page.waitForEvent('download');await page.locator('#originals button').click();expect(fs.readFileSync(await(await event).path())).toEqual(bytes);expect(await download(page,'current-file')).toEqual(bytes);await check(page);expect((await accepted(page)).plan).toBe(text);
  await page.locator('#editor').fill(text+' ');expect((await download(page,'current-file')).toString()).not.toContain('\r');const bom=Buffer.concat([Buffer.from([239,187,191]),bytes]);await page.locator('#open-file').setInputFiles({name:'bom.rplan',mimeType:'application/json',buffer:bom});await expect(page.locator('#status')).toContainText('Opened bom.rplan');expect(await download(page,'current-file')).toEqual(bom);await page.locator('#run').click();await expect(page.locator('#status')).toContainText('Native error (plan)');expect((await accepted(page)).plan).toBe(text);
});

test('invalid UTF-8 and oversized files preserve current draft, opened originals and accepted report',async({page})=>{
  await open(page);const before=await download(page,'accepted-request'),draft=await page.locator('#editor').inputValue();
  await page.locator('#open-file').setInputFiles({name:'kept.rplan',mimeType:'application/json',buffer:Buffer.from(draft)});await expect(page.locator('#status')).toContainText('Opened kept.rplan');
  for(const [name,buffer] of [['too-big.rplan',Buffer.alloc(262145,32)],['invalid.rplan',Buffer.from([255,254])]]){
    await page.locator('#open-file').setInputFiles({name,mimeType:'application/json',buffer});await expect(page.locator('#status')).toContainText('retained');expect(await page.locator('#editor').inputValue()).toBe(draft);expect(await download(page,'accepted-request')).toEqual(before);
  }
  await page.locator('details:has(#originals) > summary').click();await expect(page.locator('#originals')).toContainText('kept.rplan');await expect(page.locator('#originals button')).toHaveCount(1);
});

test('empty present input differs from clearing an optional input and native missing input stays visible',async({page})=>{
  await open(page);await page.locator('#field').selectOption('context');await page.locator('#open-file').setInputFiles({name:'empty.rctx',mimeType:'application/json',buffer:Buffer.alloc(0)});await expect(page.locator('#status')).toContainText('Opened empty.rctx');let current=JSON.parse((await download(page,'current-request')).toString());expect(current.context).toBe('');
  await page.locator('#run').click();await expect(page.locator('#status')).toContainText('Native error (context)');await page.locator('#clear-field').click();current=JSON.parse((await download(page,'current-request')).toString());expect(current.context).toBeUndefined();await check(page);await expect(page.locator('.check[data-status="missing-input"]')).toHaveCount(2);await expect(page.locator('#result-summary')).toContainText('fail');
});

test('not-evaluated checks can coexist with native pass and remain labelled',async({page})=>{
  await open(page);for(const checkbox of await page.locator('#constraints input').all())await checkbox.setChecked(['plan-shape','vra'].includes(await checkbox.inputValue()));await check(page);
  await expect(page.locator('#result-summary')).toContainText('Native audit result: pass.');await expect(page.locator('.check[data-status="not-evaluated"]')).toContainText('vra');await expect(page.locator('.boundary')).toContainText('not legal approval');
});

test('legacy input exports native conversion while complete raw inputs remain distinct',async({page})=>{
  await open(page);const legacy=JSON.parse(cases[27].request);expect(JSON.parse(legacy.plan).rplan_version).toBe('0.1');await edit(page,'plan',legacy.plan);await page.locator('#field').selectOption('context');await page.locator('#clear-field').click();for(const checkbox of await page.locator('#constraints input').all())await checkbox.setChecked((await checkbox.inputValue())==='plan-shape');await check(page);
  const normalized=JSON.parse((await download(page,'normalized-plan')).toString());expect(normalized.rplan_version).toBe('0.2');expect(normalized.plan.assignment).toEqual([0,1]);expect((await accepted(page)).plan).toBe(legacy.plan);await expect(page.locator('#result-summary')).toContainText('pass-with-warnings');
});

test('unknown fields and HTML stay raw input and inert text rather than normalized fields or script',async({page})=>{
  await open(page);const input=await accepted(page),plan=JSON.parse(input.plan);plan.unrecognized='<img src=x onerror="globalThis.injected=true">';plan.metadata.label='<script>globalThis.injected=true</script>';plan.extensions.caller={status:'approved'};await edit(page,'plan',JSON.stringify(plan));await check(page);
  const normalized=JSON.parse((await download(page,'normalized-plan')).toString());expect(normalized.unrecognized).toBeUndefined();expect(normalized.extensions.caller.status).toBe('approved');expect((await accepted(page)).plan).toContain('unrecognized');expect(await page.evaluate(()=>globalThis.injected)).toBeUndefined();expect(await page.locator('#certificate-source').locator('script,img').count()).toBe(0);
});

test('exact certificate and complete report exports preserve native large integer text',async({page})=>{
  await open(page);const lineage=JSON.parse(cases[24].request).lineage;await edit(page,'lineage',lineage);await check(page);const certificate=await download(page,'certificate-download'),raw=await download(page,'accepted-report');expect(certificate.toString()).toContain('18446744073709551615');expect(raw.toString()).toContain('18446744073709551615');
  await page.locator('#load-certificate').click();await check(page);expect(await download(page,'certificate-download')).toEqual(certificate);await expect(page.locator('#result-summary')).toContainText('verification succeeded');
});

test('accepted source inspection does not replace a newer draft and keyboard check uses current inputs',async({page})=>{
  await open(page);const input=await accepted(page);await page.locator('#editor').fill(input.plan+' ');await page.locator('details:has(#accepted-source) > summary').click();await page.locator('#accepted-field').selectOption('plan');expect(await page.locator('#accepted-source').textContent()).toBe(input.plan);await expect(page.locator('#editor')).toHaveValue(input.plan+' ');await page.locator('#editor').press('Control+Enter');await expect(page.locator('#draft-state')).toContainText('Draft matches');expect((await accepted(page)).plan).toBe(input.plan+' ');
});

test('mobile editor and all export actions fit without horizontal overflow',async({page})=>{
  await page.setViewportSize({width:390,height:844});await open(page);const draft=await page.locator('#editor').inputValue();await page.locator('#open-file').setInputFiles({name:'x'.repeat(220)+'.rplan',mimeType:'application/json',buffer:Buffer.from(draft)});await expect(page.locator('#status')).toContainText('Opened');await page.locator('details:has(#originals) > summary').click();expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);for(const id of ['current-file','current-request','accepted-report','certificate-download']){const box=await page.locator('#'+id).boundingBox();expect(box.width).toBeGreaterThan(250);expect(box.x+box.width).toBeLessThanOrEqual(390);}
});

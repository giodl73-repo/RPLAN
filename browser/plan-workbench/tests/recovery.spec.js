import {test,expect} from '@playwright/test';
import {open,check,edit,download,actualWorker} from './helpers.js';

test('newer edits fence a delayed completed report and retain the prior accepted snapshot',async({page,request})=>{
  await page.route('**/worker.js',route=>route.fulfill({contentType:'text/javascript',body:actualWorker("const report=evaluate(data.request);if(count===1)self.postMessage({type:'result',id:data.id,report});else setTimeout(()=>self.postMessage({type:'result',id:data.id,report}),500);")}));
  await open(page);const before=await download(page,'accepted-report'),original=await page.locator('#editor').inputValue();const disconnected=await(await request.get('disconnected.json')).json();await edit(page,'plan',disconnected.plan);await page.locator('#run').click();await page.locator('#editor').fill(original+' ');await expect(page.locator('#status')).toContainText('Older result discarded');expect(await download(page,'accepted-report')).toEqual(before);await expect(page.locator('#editor')).toHaveValue(original+' ');await check(page);
});

test('retry terminates the older generation and checks current inputs with a fresh worker',async({page,request})=>{
  await page.route('**/worker.js',route=>route.fulfill({contentType:'text/javascript',body:actualWorker("const report=evaluate(data.request);if(count===1)self.postMessage({type:'result',id:data.id,report});else setTimeout(()=>self.postMessage({type:'result',id:data.id,report}),800);")}));
  await open(page);const original=await page.locator('#editor').inputValue();const disconnected=await(await request.get('disconnected.json')).json();await edit(page,'plan',disconnected.plan);await page.locator('#run').click();await page.locator('#editor').fill(original+' ');await page.unroute('**/worker.js');await page.locator('#retry').click();await expect(page.locator('#status')).toContainText('Rust report accepted');const fresh=await download(page,'accepted-report');await page.waitForTimeout(900);expect(await download(page,'accepted-report')).toEqual(fresh);await expect(page.locator('#result-summary')).toContainText('Native audit result: pass.');
});

test('real ten-second computation deadline retains complete exports and retry recovers',async({page})=>{
  await page.route('**/worker.js',route=>route.fulfill({contentType:'text/javascript',body:actualWorker("if(count===1)self.postMessage({type:'result',id:data.id,report:evaluate(data.request)});")}));
  await open(page);const before=await download(page,'accepted-report');await page.locator('#editor').fill((await page.locator('#editor').inputValue())+' ');await page.locator('#run').click();await expect(page.locator('#status')).toContainText('10-second computation deadline',{timeout:15000});expect(await download(page,'accepted-report')).toEqual(before);await expect(page.locator('#run')).toBeDisabled();await page.unroute('**/worker.js');await page.locator('#retry').click();await expect(page.locator('#status')).toContainText('Rust report accepted');await expect(page.locator('#draft-state')).toContainText('Draft matches');
});

test('real twelve-second startup deadline preserves the loaded draft and recovers',async({page})=>{
  await page.route('**/worker.js',route=>route.fulfill({contentType:'text/javascript',body:'self.onmessage=()=>{};'}));await page.goto('./');await expect(page.locator('#status')).toContainText('12-second startup deadline',{timeout:16000});const current=await download(page,'current-request');expect(current.length).toBeGreaterThan(1000);await page.unroute('**/worker.js');await page.locator('#retry').click();await expect(page.locator('#status')).toContainText('Rust report accepted');expect(await download(page,'accepted-request')).toEqual(current);
});

test('real five-second file-read deadline commits neither a draft nor an opened original',async({page})=>{
  await open(page);const before=await download(page,'accepted-request'),draft=await page.locator('#editor').inputValue();await page.evaluate(()=>{globalThis.realRead=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){return new Promise(()=>{});};});await page.locator('#open-file').setInputFiles({name:'stalled.rplan',mimeType:'application/json',buffer:Buffer.from('{}')});await expect(page.locator('#status')).toContainText('5-second file read deadline',{timeout:9000});await expect(page.locator('#editor')).toHaveValue(draft);expect(await download(page,'accepted-request')).toEqual(before);await page.locator('details:has(#originals) > summary').click();await expect(page.locator('#originals')).toContainText('No user files opened');await page.evaluate(()=>{File.prototype.arrayBuffer=globalThis.realRead;});await page.locator('#open-file').setInputFiles({name:'recovered.rplan',mimeType:'application/json',buffer:Buffer.from(draft)});await expect(page.locator('#status')).toContainText('Opened recovered.rplan');await check(page);
});

test('a newer edit fences an older file read and original-byte collection',async({page})=>{
  await open(page);const original=await page.locator('#editor').inputValue();await page.evaluate(()=>{const actual=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){return new Promise(resolve=>{globalThis.releaseRead=()=>actual.call(this).then(resolve);});};});await page.locator('#open-file').setInputFiles({name:'older.rplan',mimeType:'application/json',buffer:Buffer.from('{}')});await page.locator('#editor').fill(original+' ');await page.evaluate(()=>globalThis.releaseRead());await expect(page.locator('#editor')).toHaveValue(original+' ');await expect(page.locator('#status')).not.toContainText('Reading');await page.locator('details:has(#originals) > summary').click();await expect(page.locator('#originals')).toContainText('No user files opened');await check(page);
});

test('loading an example fences an older file read before its draft and original bytes can commit',async({page})=>{
  await open(page);await page.evaluate(()=>{const actual=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){return new Promise(resolve=>{globalThis.releaseRead=()=>actual.call(this).then(resolve);});};});await page.locator('#open-file').setInputFiles({name:'older.rplan',mimeType:'application/json',buffer:Buffer.from('{}')});await page.locator('#example').selectOption('disconnected.json');await page.locator('#load-example').click();await expect(page.locator('#result-summary')).toContainText('Native audit result: fail.');await page.evaluate(()=>globalThis.releaseRead());await expect(page.locator('#status')).not.toContainText('Reading');await page.locator('details:has(#originals) > summary').click();await expect(page.locator('#originals')).toContainText('No user files opened');expect((await download(page,'current-file')).toString()).not.toBe('{}');
});

test('worker exception retains accepted data and retry uses the real native worker',async({page})=>{
  await page.route('**/worker.js',route=>route.fulfill({contentType:'text/javascript',body:actualWorker("if(count===1)self.postMessage({type:'result',id:data.id,report:evaluate(data.request)});else throw new Error('injected worker exception');")}));await open(page);const before=await download(page,'accepted-report');await page.locator('#run').click();await expect(page.locator('#status')).toContainText('injected worker exception');expect(await download(page,'accepted-report')).toEqual(before);await page.unroute('**/worker.js');await page.locator('#retry').click();await expect(page.locator('#status')).toContainText('Rust report accepted');
});

test('failed example fetching preserves a newer current draft and the accepted report',async({page})=>{
  await open(page);const before=await download(page,'accepted-report'),draft=(await page.locator('#editor').inputValue())+' ';await page.locator('#editor').fill(draft);await page.route('**/disconnected.json',route=>route.fulfill({status:503,body:'unavailable'}));await page.locator('#example').selectOption('disconnected.json');await page.locator('#load-example').click();await expect(page.locator('#status')).toContainText('Example unavailable');await expect(page.locator('#editor')).toHaveValue(draft);expect(await download(page,'accepted-report')).toEqual(before);await expect(page.locator('#run')).toBeEnabled();
});

test('real five-second example fetch deadline retains current and accepted inputs',async({page})=>{
  await open(page);const before=await download(page,'accepted-report'),draft=(await page.locator('#editor').inputValue())+' ';await page.locator('#editor').fill(draft);
  await page.route('**/disconnected.json',()=>{});await page.locator('#example').selectOption('disconnected.json');await page.locator('#load-example').click();await expect(page.locator('#status')).toContainText('Example unavailable',{timeout:9000});await expect(page.locator('#editor')).toHaveValue(draft);expect(await download(page,'accepted-report')).toEqual(before);await expect(page.locator('#run')).toBeEnabled();
});

test('a late failure for an older job cannot terminate the current job',async({page})=>{
  await page.route('**/worker.js',route=>route.fulfill({contentType:'text/javascript',body:actualWorker("if(count>1)self.postMessage({type:'failed',id:data.id-1,message:'obsolete job error'});self.postMessage({type:'result',id:data.id,report:evaluate(data.request)});")}));
  await open(page);const draft=(await page.locator('#editor').inputValue())+' ';await page.locator('#editor').fill(draft);await check(page);await expect(page.locator('#draft-state')).toContainText('Draft matches');expect(JSON.parse((await download(page,'accepted-request')).toString()).plan).toBe(draft);
});

test('a newer oversized opening cancels an older read before either draft or originals can change',async({page})=>{
  await open(page);const draft=await page.locator('#editor').inputValue(),before=await download(page,'accepted-report');
  await page.evaluate(()=>{const actual=File.prototype.arrayBuffer;File.prototype.arrayBuffer=function(){return new Promise(resolve=>{globalThis.releaseRead=()=>actual.call(this).then(resolve);});};});
  await page.locator('#open-file').setInputFiles({name:'older.rplan',mimeType:'application/json',buffer:Buffer.from('{}')});await page.locator('#open-file').setInputFiles({name:'oversized.rplan',mimeType:'application/json',buffer:Buffer.alloc(262145,32)});await page.evaluate(()=>globalThis.releaseRead());await expect(page.locator('#status')).toContainText('File exceeds');await expect(page.locator('#editor')).toHaveValue(draft);expect(await download(page,'accepted-report')).toEqual(before);await page.locator('details:has(#originals) > summary').click();await expect(page.locator('#originals')).toContainText('No user files opened');
});

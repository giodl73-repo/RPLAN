import {test,expect} from '@playwright/test';
import crypto from 'node:crypto';

test('every served file matches inventory and source/legal routes survive unavailable WASM',async({page,request})=>{
  const inventory=await(await request.get('inventory.json')).json();expect(inventory.runtime_bytes).toBeLessThanOrEqual(1048576);
  for(const file of inventory.files){const response=await request.get(file.path);expect(response.status(),file.path).toBe(200);const bytes=await response.body();expect(bytes.length,file.path).toBe(file.bytes);expect(crypto.createHash('sha256').update(bytes).digest('hex'),file.path).toBe(file.sha256);if(file.path.endsWith('.wasm'))expect(response.headers()['content-type']).toContain('application/wasm');}
  await page.route('**/worker.js',route=>route.abort());await page.goto('./');for(const name of ['Native Rust','Browser source','Corresponding source ZIP','Compiled WASM','Build','Licenses & scope','Build record'])await expect(page.getByRole('link',{name,exact:true})).toBeVisible();
  await page.getByRole('link',{name:'Licenses & scope',exact:true}).click();await expect(page.locator('h1')).toHaveText('RPLAN plan workbench notices');
  for(const link of await page.locator('a').evaluateAll(nodes=>nodes.map(n=>n.getAttribute('href')).filter(h=>!h.startsWith('http'))))expect((await request.get(link)).ok(),link).toBe(true);
});

test('corresponding source includes original tests/fixtures, portable helpers and original MIT grant',async({request})=>{
  const build=await(await request.get('build.json')).json(),files=await(await request.get('source-files.json')).json();
  for(const path of ['Cargo.lock','crates/rplan-core/src/lib.rs','crates/rplan-audit/src/profile.rs','crates/rplan-cli/tests/audit_cli.rs','crates/rplan-cli/tests/pitfall_policy.rs','crates/rplan-audit/fixtures/grid3x3.rctx','docs/authority-boundaries.v1.json','.roles/ROLE.md','browser/plan-workbench/rehearse-consumer.py','browser/plan-workbench/web/app.js'])expect(files.some(f=>f.path===path),path).toBe(true);
  expect(build.full_git_archive).toContain(build.commit);const bytes=await(await request.get('source.zip')).body();expect(crypto.createHash('sha256').update(bytes).digest('hex')).toBe(build.source_zip_sha256);const license=await(await request.get('LICENSE')).text();expect(license).toContain('MIT License');expect(license).toContain('Gio Della-Libera');
  const scope=await(await request.get('SOURCE-SCOPE.md')).text();expect(scope).toContain('every tracked Git blob');expect(scope).toContain('consumer checkout is not mirrored');expect(scope).toContain('No tracked source is silently omitted');
});

import {expect} from '@playwright/test';
import fs from 'node:fs';
export async function open(page){await page.goto('./');await expect(page.locator('#status')).toContainText('Rust report accepted');}
export async function check(page){await page.locator('#run').click();await expect(page.locator('#status')).toContainText('Rust report accepted');}
export async function edit(page,field,text){await page.locator('#field').selectOption(field);await page.locator('#editor').fill(text);}
export async function download(page,id){const event=page.waitForEvent('download');await page.locator('#'+id).click();return fs.readFileSync(await(await event).path());}
export async function accepted(page){return JSON.parse((await download(page,'accepted-request')).toString());}
export const actualWorker=(body)=>`import init,{evaluate,engine_info} from './pkg/rplan_plan_workbench.js';let count=0;self.onmessage=({data})=>{count++;${body}};await init();self.postMessage({type:'ready',info:engine_info()});`;

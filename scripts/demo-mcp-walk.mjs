// Author the animation exclusively through the public stdio MCP tools.
// Playwright launches the app, dismisses first-run prompts and invokes native Save.
import {_electron as electron} from 'playwright';
import {Client} from '@modelcontextprotocol/sdk/client/index.js';
import {StdioClientTransport} from '@modelcontextprotocol/sdk/client/stdio.js';
import {mkdir, writeFile} from 'node:fs/promises';
import {randomBytes} from 'node:crypto';
import path from 'node:path';
import {createServer} from 'node:net';
const root = path.resolve(import.meta.dirname, '..');
const fight = process.argv.includes('--fight');
const frameCount = fight ? 144 : 48;
const basename = fight ? 'stick-fight' : 'blue-walk';
const output = path.resolve(process.env.STYK_DEMO_OUTPUT || path.join(root, 'artifacts/walk'));
const scratch = path.resolve(process.env.STYK_DEMO_WORK || path.join(root, 'artifacts/walk-work'));
await mkdir(output, {recursive: true}); await mkdir(scratch, {recursive: true});
const profile = path.join(scratch, 'profile'); await mkdir(profile, {recursive: true});
await writeFile(path.join(profile, 'telemetry-state.json'), JSON.stringify({decided: true, consent: {analytics: false, diagnostics: false, policyVersion: '2026-09-10', updatedAt: Date.now()}}));
const reserve = createServer(); await new Promise(resolve => reserve.listen(0, '127.0.0.1', resolve));
const port = reserve.address().port; await new Promise(resolve => reserve.close(resolve));
const token = randomBytes(32).toString('hex');
const app = await electron.launch({args: [root], cwd: root, timeout: 60000,
  env: {...process.env, STYK_USER_DATA_DIR: profile, STYK_STARTUP_VIEW: 'editor', STYK_MCP: '1', STYK_MCP_TOKEN: token, STYK_MCP_PORT: String(port)}});
const page = await app.firstWindow();
await page.waitForFunction(() => window.styk?.ready, undefined, {timeout: 60000});
for (const selector of ['.styk-privacy [data-decline]', '.release-splash [aria-label="Close welcome screen"]']) await page.locator(selector).click({timeout: 1500}).catch(() => {});
const client = new Client({name: 'walk-animation-demo', version: '1.0.0'});
await client.connect(new StdioClientTransport({command: process.execPath, args: [path.join(root, 'scripts/mcp-server.mjs')], env: {...process.env, STYK_MCP_TOKEN: token, STYK_MCP_PORT: String(port)}}));
const log = [];
async function call(name, args = {}) {
  const response = await client.callTool({name, arguments: args});
  if (response.isError) throw new Error(JSON.stringify(response));
  log.push({tool: name, arguments: args});
  return name === 'get_frame_preview' ? response.content[0] : JSON.parse(response.content[0].text);
}
const project = await call('get_project'), uid = project.objects[0].uid;
const figure = await call('get_figure', {uid});
const joint = name => figure.joints.find(item => item.name === name).id;
if (fight) {
  const rival = (await call('duplicate_figure', {uid})).uid;
  await call('set_figure_color', {uid: rival, color: '#f45b45'});
  for (const [fighter, facing] of [[uid, 1], [rival, -1]]) {
    for (const name of ['Left lower leg', 'Right lower leg']) await call('configure_ik', {uid: fighter, jointId: joint(name), bendDirection: -facing});
    for (const name of ['Left forearm', 'Right forearm']) await call('configure_ik', {uid: fighter, jointId: joint(name), bendDirection: facing});
  }
  await call('set_fps', {fps: 24});
  const smooth = value => {const t = Math.max(0, Math.min(1, value)); return t * t * (3 - 2 * t);};
  const pulse = (tick, start, peak, end) => tick < peak ? smooth((tick - start) / (peak - start)) : 1 - smooth((tick - peak) / (end - peak));
  for (let tick = 0; tick < frameCount; tick++) {
    await call('insert_keyframe', {uid, tick});
    const approach = smooth(tick / 24) * (1 - smooth((tick - 112) / 31));
    const jab = pulse(tick, 29, 39, 53), dodge = pulse(tick, 31, 42, 57);
    const kick = pulse(tick, 61, 75, 94), recoil = pulse(tick, 73, 81, 103);
    const ax = 350 + 80 * approach - 22 * recoil, bx = 650 - 80 * approach + 8 * dodge;
    const ay = 286 + 3 * Math.sin(tick * Math.PI / 12) ** 2 + 9 * recoil;
    const by = 286 + 3 * Math.sin(tick * Math.PI / 12 + .3) ** 2 - 4 * kick;
    const descriptors = [
      {uid, x: ax, y: ay, facing: 1, lean: 10 * approach + 17 * jab - 30 * recoil},
      {uid: rival, x: bx, y: by, facing: -1, lean: -10 * approach + 32 * dodge + 22 * kick},
    ];
    for (const actor of descriptors) {
      const state = (await call('get_project')).objects.find(item => item.uid === actor.uid);
      await call('move_figure', {uid: actor.uid, dx: actor.x - state.x, dy: actor.y - state.y});
      await call('pose_joint', {uid: actor.uid, jointId: joint('Torso'), x: actor.x + actor.lean, y: actor.y - 78});
      const shoulderY = actor.y - Math.sqrt(78 ** 2 - Math.min(60, Math.abs(actor.lean)) ** 2);
      // A guard pose with two staggered hands and a wide, grounded stance.
      const front = {x: actor.x + actor.lean + actor.facing * 43, y: shoulderY - 6};
      const rear = {x: actor.x + actor.lean + actor.facing * 23, y: shoulderY + 14};
      if (actor.uid === uid) {
        front.x += jab * 48; front.y -= jab * 4;
        rear.x -= recoil * 12; rear.y -= recoil * 15;
      } else {
        front.x += kick * 12; front.y -= kick * 10;
      }
      await call('pose_joint', {uid: actor.uid, jointId: joint('Left forearm'), ...front});
      await call('pose_joint', {uid: actor.uid, jointId: joint('Right forearm'), ...rear});
      const frontFoot = {x: actor.x + actor.facing * 35, y: 405};
      if (actor.uid === rival) {
        frontFoot.x = frontFoot.x * (1 - kick) + (ax + 15) * kick;
        frontFoot.y = frontFoot.y * (1 - kick) + (ay - 25) * kick;
      }
      await call('pose_joint', {uid: actor.uid, jointId: joint('Left lower leg'), ...frontFoot});
      await call('pose_joint', {uid: actor.uid, jointId: joint('Right lower leg'), x: actor.x - actor.facing * 40, y: 405});
    }
    if (tick % 24 === 0) console.log(`Authored fight frame ${tick + 1}/${frameCount} through MCP.`);
  }
} else {
for (const name of ['Left lower leg', 'Right lower leg']) await call('configure_ik', {uid, jointId: joint(name), bendDirection: -1});
for (const name of ['Left forearm', 'Right forearm']) await call('configure_ik', {uid, jointId: joint(name), bendDirection: 1});
await call('set_fps', {fps: 24});
const ground = 402;
for (let tick = 0; tick < 48; tick++) {
  await call('insert_keyframe', {uid, tick});
  const phase = (tick % 24) / 24, theta = phase * Math.PI * 2;
  const x = 320 + tick * (80 / 12), y = 282 + 3 * Math.cos(theta * 2);
  const current = (await call('get_project')).objects.find(item => item.uid === uid);
  await call('move_figure', {uid, dx: x - current.x, dy: y - current.y});
  for (const [side, offset] of [['Left', 0], ['Right', .5]]) {
    const p = (phase + offset) % 1, swing = p < .5, u = swing ? p * 2 : (p - .5) * 2;
    const footX = swing ? -40 + 80 * (u * u * (3 - 2 * u)) : 40 - 80 * u;
    await call('pose_joint', {uid, jointId: joint(`${side} lower leg`), x: x + footX, y: ground - (swing ? 30 * Math.sin(Math.PI * u) : 0)});
    // Arms swing opposite their corresponding legs; both elbows bend toward the back.
    const armX = -32 * Math.cos(p * Math.PI * 2);
    await call('pose_joint', {uid, jointId: joint(`${side} forearm`), x: x + armX, y: y + 13 - 5 * Math.abs(Math.sin(theta))});
  }
  if (tick % 12 === 0) console.log(`Authored walk frame ${tick + 1}/48 through MCP.`);
}
}
// Hide editing guides through the app's presentation commands for clean previews.
await page.evaluate(() => {
  if (window.styk.state.onion) window.styk.actions['toggle-onion']();
  if (window.styk.state.handles) window.styk.actions['toggle-handles']();
});
// Capture every actual authored frame via MCP for a shareable animated preview.
const images = path.join(scratch, 'frames'); await mkdir(images, {recursive: true});
const scene = await page.locator('#scene').boundingBox();
const viewport = page.viewportSize() || await page.evaluate(() => ({width: innerWidth, height: innerHeight}));
await writeFile(path.join(scratch, 'capture.json'), JSON.stringify({scene, viewport, fps: 24, frames: frameCount}));
for (let tick = 0; tick < frameCount; tick++) {
  await call('seek', {tick});
  const image = await call('get_frame_preview');
  await writeFile(path.join(images, `${String(tick).padStart(3, '0')}.png`), Buffer.from(image.data, 'base64'));
}
await call('seek', {tick: 0});
const file = path.join(output, `${basename}.styk`);
await app.evaluate(({dialog}, target) => {
  globalThis.__demoOriginalSaveDialog = dialog.showSaveDialog;
  dialog.showSaveDialog = async () => ({canceled: false, filePath: target});
}, file);
let saved;
try { saved = await page.evaluate(() => window.styk.actions.save()); }
finally {
  await app.evaluate(({dialog}) => {
    dialog.showSaveDialog = globalThis.__demoOriginalSaveDialog;
    delete globalThis.__demoOriginalSaveDialog;
  });
}
if (!saved) throw new Error('Project save did not complete.');
await writeFile(path.join(scratch, 'mcp-calls.json'), JSON.stringify(log, null, 2));
console.log(`Saved editable animation: ${file}`);
await page.evaluate(() => window.styk.actions.play());
await app.evaluate(({BrowserWindow}) => {const window = BrowserWindow.getAllWindows()[0]; window.show(); window.focus();});
await client.close();
console.log(`${basename.toUpperCase()}_READY: animation is playing; leave this process running to keep the editor open.`);
await new Promise(resolve => app.on('close', resolve));

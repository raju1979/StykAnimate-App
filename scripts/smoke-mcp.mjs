import {_electron as electron} from 'playwright';
import {Client} from '@modelcontextprotocol/sdk/client/index.js';
import {StdioClientTransport} from '@modelcontextprotocol/sdk/client/stdio.js';
import {mkdtemp, mkdir, writeFile, rm} from 'node:fs/promises';
import {randomBytes} from 'node:crypto';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createServer} from 'node:net';
const root = path.resolve(import.meta.dirname, '..');
const profile = await mkdtemp(path.join(os.tmpdir(), 'styk-mcp-'));
await writeFile(path.join(profile, 'telemetry-state.json'), JSON.stringify({decided: true, consent: {analytics: false, diagnostics: false, policyVersion: '2026-09-10', updatedAt: Date.now()}}));
const token = randomBytes(32).toString('hex');
const reservation = createServer();
await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
const port = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const env = {...process.env, STYK_USER_DATA_DIR: profile, STYK_STARTUP_VIEW: 'editor', STYK_MCP: '1', STYK_MCP_TOKEN: token, STYK_MCP_PORT: String(port)};
let app, client;
try {
  app = await electron.launch({args: [root], cwd: root, env, timeout: 60000});
  const page = await app.firstWindow();
  await page.waitForFunction(() => window.styk?.ready, undefined, {timeout: 60000});
  for (const selector of ['.styk-privacy [data-decline]', '.release-splash [aria-label="Close welcome screen"]']) {
    await page.locator(selector).click({timeout: 1500}).catch(() => {});
  }
  client = new Client({name: 'stykanimate-smoke', version: '1.0.0'});
  await client.connect(new StdioClientTransport({command: process.execPath, args: [path.join(root, 'scripts/mcp-server.mjs')], env: {...process.env, STYK_MCP_TOKEN: token, STYK_MCP_PORT: String(port)}}));
  const listed = await client.listTools(); assert.equal(listed.tools.length, 7);
  const call = async (name, args = {}) => {
    const result = await client.callTool({name, arguments: args});
    assert.ok(!result.isError, JSON.stringify(result)); return result;
  };
  const project = JSON.parse((await call('get_project')).content[0].text), object = project.objects[0];
  await call('move_figure', {uid: object.uid, dx: 20, dy: 5});
  const moved = JSON.parse((await call('get_project')).content[0].text).objects[0];
  assert.equal(moved.x, object.x + 20);
  await call('undo');
  assert.equal(JSON.parse((await call('get_project')).content[0].text).objects[0].x, object.x);
  await call('redo'); await call('undo');
  const preview = (await call('get_frame_preview')).content[0];
  assert.equal(preview.type, 'image'); assert.equal(Buffer.from(preview.data, 'base64').subarray(1, 4).toString(), 'PNG');
  await mkdir(path.join(root, 'artifacts'), {recursive: true});
  await writeFile(path.join(root, 'artifacts/mcp-preview.png'), Buffer.from(preview.data, 'base64'));
  await call('set_fps', {fps: project.fps === 30 ? 24 : 30}); await call('undo');
  console.log('PASS: MCP discovery, live project, move, undo/redo, FPS and PNG preview.');
} finally {
  await client?.close();
  if (app) {await app.evaluate(({app}) => app.exit(0)).catch(() => {}); await app.close().catch(() => {});}
  await rm(profile, {recursive: true, force: true});
}

import {Client} from '@modelcontextprotocol/sdk/client/index.js';
import {StdioClientTransport} from '@modelcontextprotocol/sdk/client/stdio.js';
import {spawn} from 'node:child_process';
import {randomBytes} from 'node:crypto';
import {createServer} from 'node:net';
import {mkdir, readFile, writeFile} from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {setTimeout as delay} from 'node:timers/promises';

const root = path.resolve(import.meta.dirname, '..');
const executable = path.resolve(process.argv[2] ?? path.join(root, 'native/rust-app/target/debug/stykanimate-rust-preview.exe'));
const directory = path.join(root, 'artifacts/rust-app-smoke');
await mkdir(directory, {recursive: true});
const reservation = createServer();
await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
const port = String(reservation.address().port);
await new Promise(resolve => reservation.close(resolve));
const env = {...process.env, STYK_MCP: '1', STYK_MCP_TOKEN: randomBytes(32).toString('hex'), STYK_MCP_PORT: port};
const gui = spawn(executable, [], {env, stdio: ['ignore', 'ignore', 'pipe']});
let stderr = '';
gui.stderr.on('data', data => {stderr += data;});
gui.on('error', error => {stderr += error.message;});
const client = new Client({name: 'native-rust-smoke', version: '1.0.0'});
try {
  await client.connect(new StdioClientTransport({command: executable, args: ['--mcp'], env}));
  assert.equal((await client.listTools()).tools.length, 15);
  const call = async (name, args = {}) => {
    const result = await client.callTool({name, arguments: args});
    assert.ok(!result.isError, JSON.stringify(result));
    return result;
  };
  const value = async (name, args) => JSON.parse((await call(name, args)).content[0].text);
  let project;
  for (let attempt = 0; attempt < 40; attempt++) {
    try {project = await value('get_project'); break;} catch {await delay(250);}
  }
  assert.ok(project, `Native GUI did not start: ${stderr}`);
  const first = project.objects[0];
  await call('move_figure', {uid: first.uid, dx: 23, dy: 7});
  assert.equal((await value('get_project')).objects[0].x, first.x + 23);
  await call('undo');
  assert.equal((await value('get_project')).objects[0].x, first.x);
  await call('redo'); await call('undo');
  const duplicate = await value('duplicate_figure', {uid: first.uid});
  await call('set_figure_color', {uid: duplicate.uid, color: '#e05030'});
  await call('insert_keyframe', {tick: 12});
  await call('move_figure', {uid: duplicate.uid, dx: 75, dy: 0});
  const figure = await value('get_figure', {uid: duplicate.uid});
  const hand = figure.joints.find(j => j.name === 'Left forearm');
  assert.ok(hand);
  await call('pose_joint', {uid: duplicate.uid, jointId: hand.id, x: hand.x + 15, y: hand.y - 15});
  const snapshot = await value('get_project');
  const rejected = await client.callTool({name: 'pose_joint', arguments: {uid: duplicate.uid, jointId: 99999, x: 0, y: 0}});
  assert.equal(rejected.isError, true);
  assert.deepEqual(await value('get_project'), snapshot);
  const file = path.join(directory, 'two-figures.rustanim.json');
  await call('save_project', {path: file});
  const saved = JSON.parse(await readFile(file, 'utf8'));
  assert.equal(saved.keys['12'].figures.length, 2);
  const preview = (await call('get_frame_preview')).content[0];
  assert.equal(preview.type, 'image');
  const png = Buffer.from(preview.data, 'base64');
  assert.equal(png.subarray(1, 4).toString(), 'PNG');
  await writeFile(path.join(directory, 'preview.png'), png);
  const wrong = new Client({name: 'unauthorized-smoke', version: '1.0.0'});
  try {
    await wrong.connect(new StdioClientTransport({command: executable, args: ['--mcp'], env: {...env, STYK_MCP_TOKEN: 'x'.repeat(64)}}));
    assert.equal((await wrong.callTool({name: 'get_project', arguments: {}})).isError, true);
  } finally {await wrong.close();}
  console.log('PASS: native GUI, 15 MCP tools, move, undo/redo, duplicate, color, key, IK pose, rejected-edit rollback, atomic save, PNG and authentication.');
} finally {
  await client.close();
  gui.kill();
}

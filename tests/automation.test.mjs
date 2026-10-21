import test from 'node:test';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {Editor} from '../src/editor.mjs';
import {createAutomation} from '../src/automation.mjs';
const {startAutomation} = createRequire(import.meta.url)('../desktop/automation.cjs');

test('automation moves the requested figure with real undo/redo and validates input', async () => {
  const editor = new Editor({demo: false, propertyChannels: true, layerTimeline: true}).newDocument();
  const run = createAutomation({editor, ready: () => true, busy: () => false, preview: () => ({png: 'test'})});
  const initial = await run('get_project'), object = initial.objects[0];
  assert.ok(object);
  await run('move_figure', {uid: object.uid, dx: 15, dy: -10});
  let moved = (await run('get_project')).objects.find(item => item.uid === object.uid);
  assert.equal(moved.x, object.x + 15);
  assert.equal(moved.y, object.y - 10);
  await run('undo');
  assert.equal((await run('get_project')).objects[0].x, object.x);
  await run('redo');
  assert.equal((await run('get_project')).objects[0].x, object.x + 15);
  await run('set_fps', {fps: 30});
  assert.equal(editor.report.fps, 30);
  await run('undo'); assert.equal(editor.report.fps, initial.fps);
  await assert.rejects(run('move_figure', {uid: object.uid, dx: Infinity, dy: 0}));
  await assert.rejects(run('move_figure', {uid: -999, dx: 1, dy: 0}));
  await assert.rejects(run('set_fps', {fps: 0}));
  await assert.rejects(run('execute_code', {}));
});

test('automation refuses edits while not ready or interacting', async () => {
  for (const [ready, busy] of [[false, false], [true, true]]) {
    const run = createAutomation({ready: () => ready, busy: () => busy});
    await assert.rejects(run('set_fps', {fps: 30}));
  }
});

test('bridge requires its secret, rejects browser origins and unknown commands', async () => {
  const token = 'a'.repeat(64), calls = [];
  const bridge = await startAutomation({token, dispatch: async (method, args) => {calls.push(method); return args;}});
  const call = (headers, method = 'get_project') => fetch(`http://127.0.0.1:${bridge.port}/command`, {
    method: 'POST', headers, body: JSON.stringify({method, args: {ok: true}})});
  try {
    assert.equal((await call({})).status, 403);
    assert.equal((await call({Authorization: 'Bearer wrong'})).status, 403);
    assert.equal((await call({Authorization: `Bearer ${token}`, Origin: 'https://example.com'})).status, 403);
    assert.equal((await call({Authorization: `Bearer ${token}`}, 'execute_code')).status, 400);
    const result = await call({Authorization: `Bearer ${token}`});
    assert.deepEqual(await result.json(), {result: {ok: true}});
    assert.deepEqual(calls, ['get_project']);
  } finally {bridge.close();}
});

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

test('duplicating a fighter preserves its rig, recolors only that instance and undoes cleanly', async () => {
  const editor = new Editor({demo: false, propertyChannels: true, layerTimeline: true}).newDocument();
  const run = createAutomation({editor, ready: () => true, busy: () => false});
  const copy = await run('duplicate_figure', {uid: 1});
  assert.notEqual(copy.uid, 1);
  assert.equal((await run('get_project')).objectCount, 2);
  assert.ok(editor.object._stykIK[8].enabled);
  await run('set_figure_color', {uid: copy.uid, color: '#ff5544'});
  assert.equal(editor.object.color_override_raw, 0xff4455ff);
  editor.select(1); assert.equal(editor.object.color_override_raw, 0);
  await run('undo'); await run('undo');
  assert.equal((await run('get_project')).objectCount, 1);
});

test('pose keys retain different IK poses and undo restores the previous key pose', async () => {
  const editor = new Editor({demo: false, propertyChannels: true, layerTimeline: true}).newDocument();
  const run = createAutomation({editor, ready: () => true, busy: () => false});
  const figure = await run('get_figure', {uid: 1});
  const foot = figure.joints.find(joint => joint.name === 'Left lower leg');
  await run('configure_ik', {uid: 1, jointId: foot.id, bendDirection: -1});
  await run('insert_keyframe', {uid: 1, tick: 15});
  await run('pose_joint', {uid: 1, jointId: foot.id, x: foot.x - 15, y: foot.y - 25});
  const posed = (await run('get_figure', {uid: 1})).joints.find(joint => joint.id === foot.id);
  assert.ok(Math.abs(posed.x - (foot.x - 15)) < .01);
  assert.ok(Math.abs(posed.y - (foot.y - 25)) < .01);
  await run('undo');
  const restored = (await run('get_figure', {uid: 1})).joints.find(joint => joint.id === foot.id);
  assert.ok(Math.abs(restored.y - foot.y) < .01);
  await run('redo');
  await run('seek', {tick: 0});
  assert.ok(Math.abs((await run('get_figure', {uid: 1})).joints.find(joint => joint.id === foot.id).y - foot.y) < .01);
  await assert.rejects(run('pose_joint', {uid: 1, jointId: -9, x: 1, y: 1}));
  await assert.rejects(run('insert_keyframe', {uid: -9, tick: 16}));
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

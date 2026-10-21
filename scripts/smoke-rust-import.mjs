// Actual desktop import parity. Supply a valid .piv fixture and build Rust first.
import {_electron as electron} from 'playwright';
import {mkdtemp, mkdir, writeFile, rm} from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
const root = path.resolve(import.meta.dirname, '..');
const fixture = path.resolve(process.argv[2] || 'artifacts/rust-smoke.piv');
const rows = [];
for (const mode of ['rust', 'python']) {
  const profile = await mkdtemp(path.join(os.tmpdir(), 'styk-rust-import-'));
  await writeFile(path.join(profile, 'telemetry-state.json'), JSON.stringify({decided: true, consent: {analytics: false, diagnostics: false, policyVersion: '2026-09-10', updatedAt: Date.now()}}));
  let app;
  try {
    app = await electron.launch({args: [root], cwd: root, timeout: 60000,
      env: {...process.env, STYK_USER_DATA_DIR: profile, STYK_STARTUP_VIEW: 'editor', STYK_DISABLE_NATIVE_SCAN: mode === 'python' ? '1' : '0'}});
    const page = await app.firstWindow();
    await page.waitForFunction(() => window.styk?.ready, undefined, {timeout: 60000});
    for (const selector of ['.styk-privacy [data-decline]', '.release-splash [aria-label="Close welcome screen"]']) await page.locator(selector).click({timeout: 1500}).catch(() => {});
    const health = await page.evaluate(async () => (await fetch('/api/health')).json());
    assert.equal(health.nativeFrameScanner, mode);
    await app.evaluate(({dialog}, file) => {dialog.showOpenDialog = async () => ({canceled: false, filePaths: [file]});}, fixture);
    await page.evaluate(() => window.styk.actions.open());
    await page.waitForFunction(() => window.styk.state.lastImport, undefined, {timeout: 60000});
    const snapshot = await page.evaluate(() => {
      const editor = window.styk.editor;
      return {fps: editor.report.fps, ticks: editor.totalTicks,
        objects: editor.evaluatedFrame.objects.map(object => ({uid: object.unique_id, x: object.root_x, y: object.root_y})),
        geometry: [...editor.geometry.objects].map(([slot, geometry]) => [slot, [...geometry.joints]])};
    });
    rows.push({mode, snapshot});
    console.log(`PASS: ${mode} desktop import: ${snapshot.ticks} ticks, ${snapshot.objects.length} figures.`);
  } finally {
    if (app) {await app.evaluate(({app}) => app.exit(0)).catch(() => {}); await app.close().catch(() => {});}
    await rm(profile, {recursive: true, force: true});
  }
}
assert.deepEqual(rows[0].snapshot, rows[1].snapshot, 'Imported document timing, poses and geometry must match.');
await mkdir(path.join(root, 'artifacts'), {recursive: true});
await writeFile(path.join(root, 'artifacts/rust-desktop-import.json'), JSON.stringify(rows, null, 2));
console.log('PASS: Rust and Python desktop imports have identical timing, poses and evaluated joints.');

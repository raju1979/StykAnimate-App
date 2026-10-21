'use strict';
const http = require('node:http');
const {randomBytes, timingSafeEqual} = require('node:crypto');
const METHODS = new Set(['get_project', 'get_frame_preview', 'seek', 'move_figure', 'set_fps', 'undo', 'redo']);

// Opt-in and loopback-only. The caller supplies a secret through its environment.
async function startAutomation({token, port = 0, dispatch}) {
  if (typeof token !== 'string' || token.length < 32) throw new Error('STYK_MCP_TOKEN must contain at least 32 characters.');
  let running = false;
  const server = http.createServer(async (request, response) => {
    const reply = (status, value) => {
      response.writeHead(status, {'Content-Type': 'application/json', 'Cache-Control': 'no-store'});
      response.end(JSON.stringify(value));
    };
    const supplied = Buffer.from(request.headers.authorization || '');
    const expected = Buffer.from(`Bearer ${token}`);
    if (request.headers.origin || supplied.length !== expected.length || !timingSafeEqual(supplied, expected)) {
      request.resume(); return reply(403, {error: 'Unauthorized.'});
    }
    if (request.method !== 'POST' || request.url !== '/command') { request.resume(); return reply(404, {error: 'Not found.'}); }
    let bytes = 0, chunks = [];
    try {
      for await (const chunk of request) {
        bytes += chunk.length;
        if (bytes > 16384) { reply(413, {error: 'Request too large.'}); request.destroy(); return; }
        chunks.push(chunk);
      }
      const {method, args = {}} = JSON.parse(Buffer.concat(chunks).toString('utf8'));
      if (!METHODS.has(method)) return reply(400, {error: 'Unknown automation method.'});
      if (running) return reply(409, {error: 'An automation command is already running.'});
      running = true;
      try { reply(200, {result: await dispatch(method, args)}); }
      finally { running = false; }
    } catch (error) { reply(400, {error: error.message}); }
  });
  server.requestTimeout = 10000;
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(port, '127.0.0.1', resolve); });
  return {port: server.address().port, close: () => { server.closeAllConnections(); server.close(); }};
}

function rendererDispatch({ipcMain, getWindow}) {
  const pending = new Map();
  ipcMain.on('styk:automation-result', (event, message) => {
    const item = pending.get(message?.id);
    if (!item || event.sender !== item.sender || event.senderFrame !== item.sender.mainFrame) return;
    pending.delete(message.id); clearTimeout(item.timer);
    if (message.error) item.reject(new Error(message.error)); else item.resolve(message.result);
  });
  return (method, args) => new Promise((resolve, reject) => {
    const window = getWindow();
    if (!window || window.isDestroyed() || window.webContents.isLoading()) return reject(new Error('No ready editor window.'));
    const id = randomBytes(16).toString('hex'), sender = window.webContents;
    const timer = setTimeout(() => { pending.delete(id); reject(new Error('Editor response timed out; inspect the project before retrying an edit.')); }, 15000);
    pending.set(id, {resolve, reject, timer, sender});
    sender.send('styk:automation-command', {id, method, args});
  });
}
module.exports = {startAutomation, rendererDispatch};

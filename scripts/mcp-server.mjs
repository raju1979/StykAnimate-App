import {McpServer} from '@modelcontextprotocol/sdk/server/mcp.js';
import {StdioServerTransport} from '@modelcontextprotocol/sdk/server/stdio.js';
import {z} from 'zod';

const token = process.env.STYK_MCP_TOKEN;
const port = Number(process.env.STYK_MCP_PORT || 47832);
if (!token || token.length < 32 || !Number.isInteger(port) || port < 1 || port > 65535) {
  throw new Error('Set STYK_MCP_TOKEN (at least 32 characters) and optionally STYK_MCP_PORT.');
}
const server = new McpServer({name: 'stykanimate', version: '0.1.0'});
const tools = [
  ['get_project', 'Inspect the active project and up to 1000 figure instances at the current tick. Coordinates are canvas units; ticks start at zero.', {}],
  ['get_frame_preview', 'Capture the visible editor viewport as a PNG, including editor overlays. Use after editing to inspect the result.', {}],
  ['seek', 'Seek to a zero-based animation tick. Returns the actual tick after clamping.', {tick: z.number().int().min(0).max(10000000)}],
  ['move_figure', 'Translate a figure at the current tick by canvas units in one undoable edit. Joined figures may move together. Use get_project to obtain its uid.', {uid: z.number().int(), dx: z.number().min(-100000).max(100000), dy: z.number().min(-100000).max(100000)}],
  ['set_fps', 'Change project FPS in one undoable edit.', {fps: z.number().int().min(1).max(1000)}],
  ['undo', 'Undo the most recent editor command, including manual edits.', {}],
  ['redo', 'Redo the most recently undone editor command.', {}],
];
for (const [name, description, inputSchema] of tools) {
  server.registerTool(name, {description, inputSchema, annotations: {readOnlyHint: ['get_project', 'get_frame_preview'].includes(name), destructiveHint: false}}, async args => {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/command`, {method: 'POST',
        headers: {'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json'},
        body: JSON.stringify({method: name, args}), signal: AbortSignal.timeout(20000)});
      const body = await response.json();
      if (!response.ok) throw new Error(body.error || `Bridge returned ${response.status}`);
      return {content: name === 'get_frame_preview'
        ? [{type: 'image', mimeType: 'image/png', data: body.result.png}]
        : [{type: 'text', text: JSON.stringify(body.result)}]};
    } catch (error) { return {isError: true, content: [{type: 'text', text: error.message}]}; }
  });
}
await server.connect(new StdioServerTransport());

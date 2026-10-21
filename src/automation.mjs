// A deliberately small command API. All document mutations use Editor history.
export function createAutomation({editor, ready, busy, preview}) {
  return async function execute(method, args = {}) {
    if (!ready()) throw new Error('The editor is not ready.');
    if (busy()) throw new Error('Finish the current interaction before using automation.');
    if (!args || typeof args !== 'object' || Array.isArray(args)) throw new Error('Arguments must be an object.');
    switch (method) {
      case 'get_project': {
        const frame = editor.evaluatedFrame;
        return {fps: editor.report.fps, width: editor.report.canvas_width, height: editor.report.canvas_height,
          tick: editor.tick, frameIndex: editor.frameIndex, dirty: editor.dirty,
          keyframeCount: editor.report.keyframes.length,
          objectCount: frame.objects.length, truncated: frame.objects.length > 1000,
          objects: frame.objects.slice(0, 1000).map(object => ({uid: object.unique_id, figureTypeId: object.figure_type_id,
            x: object.root_x, y: object.root_y}))};
      }
      case 'get_frame_preview': return preview();
      case 'seek': {
        if (!Number.isInteger(args.tick) || args.tick < 0 || args.tick > 10000000) throw new Error('tick must be a nonnegative integer up to 10000000.');
        editor.seek(args.tick); return {tick: editor.tick};
      }
      case 'move_figure': {
        if (!Number.isInteger(args.uid) || !editor.evaluatedFrame.objects.some(object => object.unique_id === args.uid)) throw new Error('Unknown figure uid at the current tick.');
        if (![args.dx, args.dy].every(value => Number.isFinite(value) && Math.abs(value) <= 100000)) throw new Error('dx and dy must be finite numbers within +/-100000.');
        editor.select(args.uid);
        if (editor.selectedUid !== args.uid || editor.isFigureFocusLocked(args.uid)) throw new Error('The requested figure cannot be selected for editing.');
        return {changed: Boolean(editor.moveSelection(args.dx, args.dy))};
      }
      case 'set_fps': {
        if (!Number.isInteger(args.fps) || args.fps < 1 || args.fps > 1000) throw new Error('fps must be an integer from 1 to 1000.');
        return {changed: Boolean(editor.setFPS(args.fps))};
      }
      case 'undo': return {changed: Boolean(await editor.undo())};
      case 'redo': return {changed: Boolean(await editor.redo())};
      default: throw new Error('Unknown automation method.');
    }
  };
}

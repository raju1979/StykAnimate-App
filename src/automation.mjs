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
      case 'duplicate_figure':
      case 'set_figure_color': {
        if (!Number.isInteger(args.uid) || !editor.evaluatedFrame.objects.some(object => object.unique_id === args.uid) || editor.isFigureFocusLocked(args.uid)) throw new Error('Unknown or locked figure uid.');
        if (method === 'set_figure_color' && !/^#[0-9a-f]{6}$/i.test(args.color)) throw new Error('color must be #RRGGBB.');
        editor.select(args.uid);
        if (method === 'duplicate_figure') return {uid: editor.duplicateFigure()?.unique_id};
        const rgb = args.color.slice(1).match(/../g).map(value => parseInt(value, 16));
        return {changed: Boolean(editor.updateSelection({color_override_raw: (0xff000000 | rgb[0] | rgb[1] << 8 | rgb[2] << 16) >>> 0}))};
      }
      case 'get_figure': {
        if (!Number.isInteger(args.uid)) throw new Error('uid must be an integer.');
        editor.select(args.uid);
        if (editor.selectedUid !== args.uid) throw new Error('Unknown or locked figure uid.');
        const geometry = editor.geometry.objects.get(editor.selectedSlot);
        return {uid: args.uid, joints: editor.figure.segments.slice(0, 1000).map(segment => ({
          id: segment.segment_id, name: segment.name, parentId: segment.parent_id,
          length: segment.length, ...geometry.joints.get(segment.segment_id)}))};
      }
      case 'insert_keyframe': {
        if (!Number.isInteger(args.tick) || args.tick < 0 || args.tick > 100000) throw new Error('tick must be an integer from 0 to 100000.');
        if (!Number.isInteger(args.uid) || !editor.evaluatedFrame.objects.some(object => object.unique_id === args.uid) || !editor.report._styk?.layerSheet) throw new Error('A current layer-sheet figure uid is required.');
        const id = editor.layerForUid(args.uid);
        if (editor.isLayerFocusLocked(id)) throw new Error('The figure layer is locked.');
        const changed = editor.sheetInsertKey([id], args.tick);
        editor.seek(args.tick);
        return {changed: Boolean(changed), tick: editor.tick};
      }
      case 'configure_ik':
      case 'pose_joint': {
        const configuring = method === 'configure_ik';
        if (configuring && (!Number.isInteger(args.bendDirection) || ![-1, 0, 1].includes(args.bendDirection))) throw new Error('bendDirection must be -1, 0 or 1.');
        if (!configuring && ![args.x, args.y].every(value => Number.isFinite(value) && Math.abs(value) <= 100000)) throw new Error('x and y must be finite canvas coordinates within +/-100000.');
        if (!Number.isInteger(args.uid) || !Number.isInteger(args.jointId)) throw new Error('uid and jointId must be integers.');
        editor.select(args.uid);
        if (editor.selectedUid !== args.uid || editor.isFigureFocusLocked(args.uid)) throw new Error('Unknown or locked figure uid.');
        const index = editor.figure.segments.findIndex(segment => segment.segment_id === args.jointId);
        if (index < 0) throw new Error('Unknown joint id.');
        editor.select(args.uid, index);
        if (configuring) {
          if (!editor.getIKSettings().enabled) throw new Error('The joint needs an enabled IK controller.');
          return {changed: Boolean(editor.setNodeIK({bendDirection: args.bendDirection}))};
        }
        return {changed: Boolean(editor.poseJoint(args.x, args.y, {snap: false}))};
      }
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

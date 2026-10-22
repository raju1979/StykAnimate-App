use crate::{
    model::{Editor, Figure, Point, Result},
    render,
    tracks::{Layer, Track},
};
use base64::Engine;
use serde_json::{json, Value};
use std::path::Path;

fn integer(args: &Value, key: &str) -> Result<u64> {
    args[key]
        .as_u64()
        .ok_or(format!("{key} must be a nonnegative integer"))
}
fn number(args: &Value, key: &str) -> Result<f32> {
    let value = args[key]
        .as_f64()
        .ok_or(format!("{key} must be a number"))? as f32;
    if !value.is_finite() || value.abs() > 100000. {
        Err(format!("{key} is outside the supported range"))
    } else {
        Ok(value)
    }
}
pub fn execute(editor: &mut Editor, method: &str, args: &Value) -> Result<Value> {
    if !args.is_object() {
        return Err("Arguments must be an object".into());
    }
    match method {
        "get_project" => Ok(
            json!({"name":editor.doc.name,"width":editor.doc.width,"height":editor.doc.height,"fps":editor.doc.fps,"tick":editor.tick,"duration":editor.doc.duration,"keyframeCount":editor.doc.layers.iter().map(|l|l.keys.len()).sum::<usize>(),"dirty":editor.dirty,
            "camera":editor.doc.camera(editor.tick as f32), "scene":editor.doc.scene(editor.tick as f32),
            "cameraLocked":editor.doc.camera_locked,"sceneLocked":editor.doc.scene_locked,
            "layers":editor.doc.layers.iter().rev().map(|l|json!({"id":l.id,"name":l.name,"visible":l.visible,"locked":l.locked,"keys":l.keys.keys().collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "objects":editor.doc.sample(editor.tick as f32).figures.iter().map(|f|json!({"uid":f.id,"name":f.name,"x":f.root.x,"y":f.root.y,"color":f.color})).collect::<Vec<_>>()}),
        ),
        "get_figure" => {
            let uid = integer(args, "uid")?;
            let frame = editor.doc.sample(editor.tick as f32);
            let figure = frame
                .figures
                .iter()
                .find(|f| f.id == uid)
                .ok_or("Unknown figure")?;
            editor.selected = uid;
            editor.track = Track::Layer(editor.doc.owner(uid).ok_or("Unknown figure")?);
            let points = figure.points();
            Ok(
                json!({"uid":uid,"joints":figure.bones.iter().enumerate().map(|(i,b)|json!({"id":i+1,"name":b.name,"parentId":b.parent.map(|p|p+1).unwrap_or(0),"x":points[i].x,"y":points[i].y,"length":b.length,"angle":b.angle,"ik":b.ik})).collect::<Vec<_>>()}),
            )
        }
        "get_frame_preview" => Ok(
            json!({"png":base64::engine::general_purpose::STANDARD.encode(render::png(&editor.doc,editor.tick as f32)?)}),
        ),
        "seek" => {
            editor.tick =
                (integer(args, "tick")?.min(editor.doc.duration.saturating_sub(1) as u64)) as u32;
            Ok(json!({"tick":editor.tick}))
        }
        "undo" => Ok(json!({"changed":editor.undo()})),
        "redo" => Ok(json!({"changed":editor.redo()})),
        "set_fps" => {
            let fps = integer(args, "fps")?;
            if !(1..=240).contains(&fps) {
                return Err("FPS must be 1–240".into());
            }
            editor.change(|d| {
                d.fps = fps as u32;
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "save_project" => {
            let path = args["path"]
                .as_str()
                .ok_or("A .rustanim.json path is required")?;
            editor.doc.save(Path::new(path))?;
            editor.dirty = false;
            Ok(json!({"saved":path}))
        }
        "insert_keyframe" => {
            let tick = integer(args, "tick")?;
            if tick >= 10000 {
                return Err("tick must be less than 10000".into());
            }
            let track = editor.track;
            editor.change(|d| d.insert_track_key(track, tick as u32, false))?;
            editor.tick = tick as u32;
            Ok(json!({"tick":tick,"changed":true}))
        }
        "add_figure" => {
            let id = editor.doc.next_id();
            let tick = editor.tick;
            let Track::Layer(layer) = editor.track else {
                return Err("Select a figure layer first".into());
            };
            editor.change(|d| {
                let figure = Figure::blue(id, d.width as f32 / 2., d.height as f32 / 2.);
                d.layer_key(layer, tick)?.figures.push(figure);
                Ok(())
            })?;
            editor.selected = id;
            Ok(json!({"uid":id}))
        }
        "duplicate_figure" => {
            let uid = integer(args, "uid")?;
            let id = editor.doc.next_id();
            let tick = editor.tick;
            editor.change(|d| {
                let mut f = d.figure_mut(tick, uid)?.clone();
                f.id = id;
                f.name = format!("{} copy", f.name);
                f.root.x += 24.;
                f.root.y += 24.;
                let layer = d.owner(uid).ok_or("Unknown figure")?;
                d.layer_key(layer, tick)?.figures.push(f);
                Ok(())
            })?;
            editor.selected = id;
            Ok(json!({"uid":id}))
        }
        "move_figure" => {
            let uid = integer(args, "uid")?;
            let dx = number(args, "dx")?;
            let dy = number(args, "dy")?;
            let tick = editor.tick;
            editor.change(|d| {
                let f = d.figure_mut(tick, uid)?;
                f.root.x += dx;
                f.root.y += dy;
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "set_figure_color" => {
            let uid = integer(args, "uid")?;
            let value = args["color"].as_str().ok_or("color must be #RRGGBB")?;
            if value.len() != 7
                || !value.starts_with('#')
                || !value[1..].bytes().all(|c| c.is_ascii_hexdigit())
            {
                return Err("color must be #RRGGBB".into());
            }
            let color = u32::from_str_radix(&value[1..], 16).map_err(|e| e.to_string())?;
            let tick = editor.tick;
            editor.change(|d| {
                d.figure_mut(tick, uid)?.color =
                    [(color >> 16) as u8, (color >> 8) as u8, color as u8];
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "pose_joint" | "configure_ik" => {
            let uid = integer(args, "uid")?;
            let joint = integer(args, "jointId")?;
            if joint == 0 {
                return Err("Use move_figure for the origin".into());
            }
            let tick = editor.tick;
            if method == "pose_joint" {
                let point = Point::new(number(args, "x")?, number(args, "y")?);
                editor.change(|d| d.figure_mut(tick, uid)?.pose((joint - 1) as usize, point))?;
            } else {
                let bend = args["bendDirection"]
                    .as_i64()
                    .filter(|v| [-1, 0, 1].contains(v))
                    .ok_or("bendDirection must be -1, 0 or 1")?;
                editor.change(|d| {
                    let bone = d
                        .figure_mut(tick, uid)?
                        .bones
                        .get_mut((joint - 1) as usize)
                        .ok_or("Unknown joint")?;
                    if bone.parent.is_none() {
                        return Err("IK needs two bones".into());
                    }
                    bone.bend = bend as i8;
                    bone.ik = true;
                    Ok(())
                })?;
            }
            editor.selected = uid;
            Ok(json!({"changed":true}))
        }
        "add_layer" => {
            let id = editor.doc.layers.iter().map(|l| l.id).max().unwrap_or(0) + 1;
            let name = args["name"].as_str().unwrap_or("New layer").to_owned();
            editor.change(|d| {
                d.layers.push(Layer {
                    id,
                    name,
                    visible: true,
                    locked: false,
                    keys: std::collections::BTreeMap::from([(
                        0,
                        crate::model::Frame { figures: vec![] },
                    )]),
                });
                Ok(())
            })?;
            editor.track = Track::Layer(id);
            editor.selected = 0;
            Ok(json!({"layerId":id}))
        }
        "select_track" => {
            editor.track = match args["track"].as_str() {
                Some("camera") => Track::Camera,
                Some("scene") => Track::Scene,
                Some("layer") => {
                    let id = integer(args, "layerId")?;
                    editor.doc.layer(id)?;
                    Track::Layer(id)
                }
                _ => return Err("track must be camera, scene or layer".into()),
            };
            Ok(json!({"selected":true}))
        }
        "set_layer" => {
            let id = integer(args, "layerId")?;
            editor.change(|d| {
                let layer = d.layer_mut(id)?;
                if let Some(name) = args.get("name") {
                    layer.name = name.as_str().ok_or("name must be a string")?.to_owned();
                }
                if let Some(visible) = args.get("visible") {
                    layer.visible = visible.as_bool().ok_or("visible must be boolean")?;
                }
                if let Some(locked) = args.get("locked") {
                    layer.locked = locked.as_bool().ok_or("locked must be boolean")?;
                }
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "delete_layer" => {
            let id = integer(args, "layerId")?;
            editor.change(|d| {
                if d.layers.len() == 1 {
                    return Err("Keep at least one figure layer".into());
                }
                if d.layer(id)?.locked {
                    return Err("Layer is locked".into());
                }
                d.layers.retain(|l| l.id != id);
                Ok(())
            })?;
            editor.track = Track::Layer(editor.doc.layers[0].id);
            Ok(json!({"changed":true}))
        }
        "move_to_layer" => {
            let uid = integer(args, "uid")?;
            let target = integer(args, "layerId")?;
            editor.change(|d| d.move_to_layer(uid, target))?;
            editor.track = Track::Layer(target);
            Ok(json!({"changed":true}))
        }
        "reorder_layer" => {
            let id = integer(args, "layerId")?;
            let offset = args["offset"]
                .as_i64()
                .filter(|x| [-1, 1].contains(x))
                .ok_or("offset must be -1 or 1")?;
            editor.change(|d| {
                let i = d
                    .layers
                    .iter()
                    .position(|l| l.id == id)
                    .ok_or("Unknown layer")?;
                let j = (i as i64 + offset).clamp(0, d.layers.len() as i64 - 1) as usize;
                d.layers.swap(i, j);
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "delete_figure" => {
            let uid = integer(args, "uid")?;
            let tick = editor.tick;
            editor.change(|d| {
                let layer = d.owner(uid).ok_or("Unknown figure")?;
                d.layer_key(layer, tick)?.figures.retain(|f| f.id != uid);
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "delete_keyframe" | "blank_keyframe" => {
            let tick = integer(args, "tick")?;
            if tick >= 10000 {
                return Err("tick must be less than 10000".into());
            }
            let track = editor.track;
            if method == "blank_keyframe" && !matches!(track, Track::Layer(_)) {
                return Err("Blank keys apply to figure layers".into());
            }
            editor.change(|d| {
                if method == "delete_keyframe" {
                    d.delete_track_key(track, tick as u32)
                } else {
                    d.insert_track_key(track, tick as u32, true)
                }
            })?;
            Ok(json!({"changed":true}))
        }
        "set_camera" => {
            if editor.doc.camera_locked {
                return Err("Camera track is locked".into());
            }
            let tick = editor.tick;
            let mut camera = editor.doc.camera(tick as f32);
            if args.get("x").is_some() {
                camera.center.x = number(args, "x")?;
            }
            if args.get("y").is_some() {
                camera.center.y = number(args, "y")?;
            }
            if args.get("zoom").is_some() {
                camera.zoom = number(args, "zoom")?;
            }
            if args.get("angle").is_some() {
                camera.angle = number(args, "angle")?.to_radians();
            }
            editor.change(|d| {
                d.camera_keys.insert(tick, camera);
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "set_scene" => {
            if editor.doc.scene_locked {
                return Err("Scene track is locked".into());
            }
            let color = args["background"]
                .as_array()
                .ok_or("background must be [r,g,b]")?;
            if color.len() != 3 {
                return Err("background must have three channels".into());
            }
            let mut background = [0u8; 3];
            for (i, c) in color.iter().enumerate() {
                background[i] = c
                    .as_u64()
                    .filter(|n| *n <= 255)
                    .ok_or("Color channels must be 0–255")? as u8;
            }
            let tick = editor.tick;
            editor.change(|d| {
                d.scene_keys
                    .insert(tick, crate::tracks::Scene { background });
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        "set_track_lock" => {
            let locked = args["locked"].as_bool().ok_or("locked must be boolean")?;
            let camera = match args["track"].as_str() {
                Some("camera") => true,
                Some("scene") => false,
                _ => return Err("track must be camera or scene".into()),
            };
            editor.change(|d| {
                if camera {
                    d.camera_locked = locked;
                } else {
                    d.scene_locked = locked;
                }
                Ok(())
            })?;
            Ok(json!({"changed":true}))
        }
        _ => Err("Unknown native preview tool".into()),
    }
}
pub fn tools() -> Vec<Value> {
    let integer = json!({"type":"integer","minimum":0});
    let number = json!({"type":"number"});
    let text = json!({"type":"string"});
    vec![
        ("get_project","Inspect the native Rust preview project",json!({}),vec![]),
        ("get_figure","Inspect named joints and evaluated world coordinates",json!({"uid":integer}),vec!["uid"]),
        ("get_frame_preview","Render the native scene to a PNG image",json!({}),vec![]),
        ("add_figure","Add an editable Blue figure at the current key",json!({}),vec![]),
        ("duplicate_figure","Duplicate an existing figure and its rig",json!({"uid":integer}),vec!["uid"]),
        ("set_figure_color","Set a figure color using #RRGGBB",json!({"uid":integer,"color":text}),vec!["uid","color"]),
        ("move_figure","Move a figure by canvas units",json!({"uid":integer,"dx":number,"dy":number}),vec!["uid","dx","dy"]),
        ("pose_joint","Pose a joint without changing bone lengths; use native joint IDs",json!({"uid":integer,"jointId":integer,"x":number,"y":number}),vec!["uid","jointId","x","y"]),
        ("configure_ik","Choose two-bone IK bend direction: -1, 0 or 1",json!({"uid":integer,"jointId":integer,"bendDirection":{"type":"integer"}}),vec!["uid","jointId","bendDirection"]),
        ("insert_keyframe","Insert a key on the selected independent track at zero-based tick",json!({"tick":integer}),vec!["tick"]),
        ("seek","Seek to a zero-based tick",json!({"tick":integer}),vec!["tick"]),
        ("set_fps","Change project FPS from 1 to 240",json!({"fps":integer}),vec!["fps"]),
        ("save_project","Save an editable preview project; path must end with .rustanim.json",json!({"path":text}),vec!["path"]),
        ("undo","Undo the latest edit, including manual edits",json!({}),vec![]),
        ("redo","Redo the latest undone edit",json!({}),vec![]),
        ("add_layer","Add an independent figure layer",json!({"name":text}),vec![]),
        ("select_track","Select camera, scene or layer for key operations",json!({"track":text,"layerId":integer}),vec!["track"]),
        ("set_layer","Rename a layer or change visibility/lock",json!({"layerId":integer,"name":text,"visible":{"type":"boolean"},"locked":{"type":"boolean"}}),vec!["layerId"]),
        ("delete_layer","Delete a figure layer and its animation; keeps at least one layer",json!({"layerId":integer}),vec!["layerId"]),
        ("move_to_layer","Move a figure's full pose animation to another layer",json!({"uid":integer,"layerId":integer}),vec!["uid","layerId"]),
        ("reorder_layer","Move layer forward (+1) or back (-1)",json!({"layerId":integer,"offset":{"type":"integer","enum":[-1,1]}}),vec!["layerId","offset"]),
        ("delete_figure","Remove figure at current layer pose key",json!({"uid":integer}),vec!["uid"]),
        ("delete_keyframe","Delete a selected-track key, keeping opening key",json!({"tick":integer}),vec!["tick"]),
        ("blank_keyframe","Insert blank exposure on selected figure layer",json!({"tick":integer}),vec!["tick"]),
        ("set_camera","Key camera center, zoom multiplier and rotation in degrees",json!({"x":number,"y":number,"zoom":number,"angle":number}),vec![]),
        ("set_scene","Key scene background RGB color",json!({"background":{"type":"array","items":{"type":"integer","minimum":0,"maximum":255},"minItems":3,"maxItems":3}}),vec!["background"]),
        ("set_track_lock","Lock or unlock permanent camera or scene track",json!({"track":text,"locked":{"type":"boolean"}}),vec!["track","locked"]),
    ].into_iter().map(|(name,description,properties,required)|json!({"name":name,"description":description,"inputSchema":{"type":"object","properties":properties,"required":required,"additionalProperties":false}})).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_commands_do_not_add_keys_or_history() {
        let mut editor = Editor::default();
        editor.tick = 12;
        let before = editor.doc.clone();
        assert!(execute(
            &mut editor,
            "pose_joint",
            &json!({"uid": 1, "jointId": 9000, "x": 1, "y": 2})
        )
        .is_err());
        assert_eq!(editor.doc, before);
        assert!(!editor.dirty);
        assert!(!editor.undo());
        assert!(execute(&mut editor, "set_fps", &json!({"fps": 0})).is_err());
        assert!(execute(
            &mut editor,
            "set_figure_color",
            &json!({"uid": 1, "color": "#zz0000"})
        )
        .is_err());
        assert_eq!(editor.doc, before);
    }

    #[test]
    fn keys_capture_evaluated_pose_and_commands_share_history() {
        let mut editor = Editor::default();
        let start = editor.doc.sample(0.).figures[0].root;
        execute(&mut editor, "insert_keyframe", &json!({"tick": 12})).unwrap();
        execute(
            &mut editor,
            "move_figure",
            &json!({"uid": 1, "dx": 120, "dy": 0}),
        )
        .unwrap();
        assert_eq!(editor.doc.sample(6.).figures[0].root.x, start.x + 60.);
        assert_eq!(editor.doc.sample(0.).figures[0].root, start);
        assert!(editor.undo());
        assert_eq!(editor.doc.sample(12.).figures[0].root, start);
        assert!(editor.redo());
        assert_eq!(editor.doc.sample(12.).figures[0].root.x, start.x + 120.);
    }
    #[test]
    fn permanent_track_locks_reject_edits_and_unlock_is_undoable() {
        let mut e = Editor::default();
        for track in ["camera", "scene"] {
            execute(
                &mut e,
                "set_track_lock",
                &json!({"track":track,"locked":true}),
            )
            .unwrap();
            execute(&mut e, "select_track", &json!({"track":track})).unwrap();
            let before = e.doc.clone();
            assert!(execute(&mut e, "insert_keyframe", &json!({"tick":5})).is_err());
            let (method, args) = if track == "camera" {
                ("set_camera", json!({"zoom":2.}))
            } else {
                ("set_scene", json!({"background":[0,0,0]}))
            };
            assert!(execute(&mut e, method, &args).is_err());
            assert_eq!(e.doc, before);
            assert!(e.undo());
            assert!(!e.doc.track_locked(e.track));
        }
    }
}

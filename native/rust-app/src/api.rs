use crate::{
    model::{Editor, Figure, Point, Result},
    render,
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
            json!({"name":editor.doc.name,"width":editor.doc.width,"height":editor.doc.height,"fps":editor.doc.fps,"tick":editor.tick,"duration":editor.doc.duration,"keyframeCount":editor.doc.keys.len(),"dirty":editor.dirty,
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
            editor.change(|d| {
                d.key(tick as u32);
                Ok(())
            })?;
            editor.tick = tick as u32;
            Ok(json!({"tick":tick,"changed":true}))
        }
        "add_figure" => {
            let id = editor.doc.next_id();
            let tick = editor.tick;
            editor.change(|d| {
                let figure = Figure::blue(id, d.width as f32 / 2., d.height as f32 / 2.);
                d.key(tick).figures.push(figure);
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
                d.key(tick).figures.push(f);
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
        ("insert_keyframe","Insert a whole-scene pose key at zero-based tick",json!({"tick":integer}),vec!["tick"]),
        ("seek","Seek to a zero-based tick",json!({"tick":integer}),vec!["tick"]),
        ("set_fps","Change project FPS from 1 to 240",json!({"fps":integer}),vec!["fps"]),
        ("save_project","Save an editable preview project; path must end with .rustanim.json",json!({"path":text}),vec!["path"]),
        ("undo","Undo the latest edit, including manual edits",json!({}),vec![]),
        ("redo","Redo the latest undone edit",json!({}),vec![]),
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
}

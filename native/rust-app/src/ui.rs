use crate::tracks::Track;
use crate::{
    api, mcp,
    model::{Bone, Document, Editor, Figure, Point},
    render,
};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
mod workspace;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

pub struct App {
    editor: Editor,
    playing: bool,
    last: Instant,
    playhead: f32,
    handles: bool,
    onion: bool,
    build: bool,
    circle: bool,
    selected_joint: Option<usize>,
    drag: Option<(Document, Option<usize>, Point)>,
    path: Option<PathBuf>,
    status: String,
    pending: Option<Document>,
    quit_prompt: bool,
    force_close: bool,
    bridge: Option<mcp::Bridge>,
    export: Option<(Arc<AtomicBool>, mpsc::Receiver<String>)>,
    capture: Option<PathBuf>,
    capture_requested: bool,
    frames: usize,
    zoom: f32,
    pan: Vec2,
    pan_tool: bool,
    grid: bool,
    timeline_scroll: f32,
    save_error: Option<String>,
}
#[derive(Clone, Copy)]
enum UnsavedChoice {
    Save,
    Discard,
    Cancel,
}
impl App {
    pub fn preview_unsaved(mut self, enabled: bool) -> Self {
        if enabled && self.capture.is_some() {
            self.editor.dirty = true;
            self.pending = Some(Document::default());
        }
        self
    }
    pub fn at_tick(mut self, tick: u32) -> Self {
        self.editor.tick = tick.min(self.editor.doc.duration - 1);
        self
    }
    pub fn with_track(mut self, name: Option<String>) -> Self {
        self.editor.track = match name.as_deref() {
            Some("camera") => Track::Camera,
            Some("scene") => Track::Scene,
            _ => Track::Layer(
                self.editor
                    .doc
                    .owner(self.editor.selected)
                    .unwrap_or(self.editor.doc.layers[0].id),
            ),
        };
        self
    }
    pub fn new(
        context: &eframe::CreationContext<'_>,
        doc: Document,
        capture: Option<PathBuf>,
    ) -> Self {
        Self::create(&context.egui_ctx, doc, capture)
    }
    fn create(context: &egui::Context, doc: Document, capture: Option<PathBuf>) -> Self {
        workspace::theme(context);
        let mut app = Self {
            editor: Editor::default(),
            playing: false,
            last: Instant::now(),
            playhead: 0.,
            handles: true,
            onion: false,
            build: false,
            circle: false,
            selected_joint: None,
            drag: None,
            path: None,
            status: "Ready".into(),
            pending: None,
            quit_prompt: false,
            force_close: false,
            bridge: None,
            export: None,
            capture,
            capture_requested: false,
            frames: 0,
            zoom: 1.,
            pan: Vec2::ZERO,
            pan_tool: false,
            grid: false,
            timeline_scroll: 0.,
            save_error: None,
        };
        app.editor.doc = doc;
        if std::env::var("STYK_MCP").as_deref() == Ok("1") {
            match mcp::start(context.clone()) {
                Ok(bridge) => {
                    app.bridge = Some(bridge);
                    app.status = "Local MCP enabled".into();
                }
                Err(error) => app.status = error,
            }
        }
        app
    }
    fn load(&mut self, doc: Document) {
        self.editor = Editor::default();
        self.editor.doc = doc;
        self.playing = false;
        self.selected_joint = None;
        self.playhead = 0.;
        self.path = None;
        self.zoom = 1.;
        self.pan = Vec2::ZERO;
    }
    fn replace(&mut self, doc: Document) {
        self.playing = false;
        self.save_error = None;
        if self.editor.dirty {
            self.pending = Some(doc);
        } else {
            self.load(doc);
        }
    }
    fn save(&mut self, save_as: bool) -> bool {
        self.save_error = None;
        let file = if save_as || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("Rust preview project", &["json"])
                .set_file_name("animation.rustanim.json")
                .save_file()
        } else {
            self.path.clone()
        };
        let Some(file) = file else {
            return false;
        };
        match self.editor.doc.save(&file) {
            Ok(()) => {
                self.status = format!("Saved {}", file.display());
                self.path = Some(file);
                self.editor.dirty = false;
                true
            }
            Err(error) => {
                self.save_error = Some(error.clone());
                self.status = error;
                false
            }
        }
    }
    fn resolve_unsaved(&mut self, ctx: &egui::Context, choice: UnsavedChoice) {
        if matches!(choice, UnsavedChoice::Cancel) {
            self.pending = None;
            self.quit_prompt = false;
            self.save_error = None;
            return;
        }
        if matches!(choice, UnsavedChoice::Save) && !self.save(false) {
            return;
        }
        if self.quit_prompt {
            self.force_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if let Some(doc) = self.pending.take() {
            self.load(doc);
        }
        self.save_error = None;
    }
    fn edit(&mut self, method: &str, args: serde_json::Value) {
        self.playing = false;
        let selected = self.editor.selected;
        match api::execute(&mut self.editor, method, &args) {
            Ok(_) => self.status = "Edit complete".into(),
            Err(error) => self.status = error,
        }
        if self.editor.selected != selected {
            self.selected_joint = None;
        }
    }
    fn export(&mut self, kind: &str) {
        if self.export.is_some() {
            self.status = "Wait for the current export or cancel it first".into();
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .add_filter(kind, &[kind])
            .set_file_name(format!("animation.{kind}"))
            .save_file()
        else {
            return;
        };
        if kind == "gif" {
            let doc = self.editor.doc.clone();
            let cancel = Arc::new(AtomicBool::new(false));
            let cancellation = cancel.clone();
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let message = match render::gif(&doc, &path, &cancellation) {
                    Ok(()) => format!("Exported {}", path.display()),
                    Err(e) => e,
                };
                let _ = tx.send(message);
            });
            self.export = Some((cancel, rx));
            self.status = "Exporting GIF…".into();
        } else {
            let result = if kind == "png" {
                render::png(&self.editor.doc, self.editor.tick as f32)
                    .and_then(|bytes| render::atomic_bytes(&path, &bytes))
            } else {
                render::atomic_bytes(
                    &path,
                    render::svg(&self.editor.doc, self.editor.tick as f32).as_bytes(),
                )
            };
            self.status = match result {
                Ok(()) => format!("Exported {}", path.display()),
                Err(error) => error,
            };
        }
    }
    fn paint_figure(
        painter: &egui::Painter,
        figure: &Figure,
        origin: Pos2,
        scale: f32,
        alpha: u8,
        handles: bool,
    ) {
        let color = Color32::from_rgba_unmultiplied(
            figure.color[0],
            figure.color[1],
            figure.color[2],
            alpha,
        );
        let points = figure.points();
        let screen = |p: Point| origin + Vec2::new(p.x, p.y) * scale;
        for (i, bone) in figure.bones.iter().enumerate() {
            let start = bone.parent.map(|p| points[p]).unwrap_or(figure.root);
            let end = points[i];
            if bone.circle {
                painter.circle_filled(
                    screen(Point::new((start.x + end.x) / 2., (start.y + end.y) / 2.)),
                    bone.length * scale / 2.,
                    color,
                );
            } else if bone.width > 0. {
                // One capsule avoids antialias seams between a thick line and
                // separately drawn endpoint circles at presentation zooms.
                let a = screen(start);
                let b = screen(end);
                let direction = (b.y - a.y).atan2(b.x - a.x);
                let radius = bone.width * scale / 2.;
                let mut outline = Vec::with_capacity(18);
                for (center, offset) in [
                    (a, std::f32::consts::FRAC_PI_2),
                    (b, -std::f32::consts::FRAC_PI_2),
                ] {
                    for i in 0..=8 {
                        let angle = direction + offset + i as f32 * std::f32::consts::PI / 8.;
                        outline.push(center + Vec2::new(angle.cos(), angle.sin()) * radius);
                    }
                }
                painter.add(egui::Shape::convex_polygon(outline, color, Stroke::NONE));
            }
        }
        if handles {
            for point in points {
                painter.circle_filled(screen(point), 4., Color32::from_rgb(245, 110, 92));
            }
            painter.circle_filled(screen(figure.root), 6., Color32::from_rgb(255, 190, 65));
        }
    }
    fn canvas(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let doc = &self.editor.doc;
        let scale = ((rect.width() - 160.) / doc.width as f32)
            .min((rect.height() - 56.) / doc.height as f32)
            .max(0.01)
            * self.zoom;
        let canvas = Rect::from_center_size(
            rect.center() + self.pan,
            Vec2::new(doc.width as f32, doc.height as f32) * scale,
        );
        ui.painter()
            .rect_filled(rect, 0., Color32::from_rgb(237, 239, 245));
        ui.painter().rect_filled(
            canvas.translate(Vec2::new(0., 4.)).expand(3.),
            2.,
            Color32::from_rgb(223, 226, 234),
        );
        let time = if self.playing {
            self.playhead
        } else {
            self.editor.tick as f32
        };
        let bg = doc.scene(time).background;
        ui.painter()
            .rect_filled(canvas, 0., Color32::from_rgb(bg[0], bg[1], bg[2]));
        let painter = ui.painter().with_clip_rect(canvas);
        if self.grid {
            let step = 40. * scale;
            for x in 0..=(doc.width as f32 / 40.) as usize {
                let x = canvas.left() + x as f32 * step;
                painter.line_segment(
                    [Pos2::new(x, canvas.top()), Pos2::new(x, canvas.bottom())],
                    Stroke::new(0.5, Color32::from_gray(230)),
                );
            }
            for y in 0..=(doc.height as f32 / 40.) as usize {
                let y = canvas.top() + y as f32 * step;
                painter.line_segment(
                    [Pos2::new(canvas.left(), y), Pos2::new(canvas.right(), y)],
                    Stroke::new(0.5, Color32::from_gray(230)),
                );
            }
        }
        let tick = self.editor.tick;
        if self.onion && !self.playing {
            for t in [tick.saturating_sub(1), (tick + 1).min(doc.duration - 1)] {
                if t != tick {
                    for f in doc.rendered(t as f32).figures {
                        Self::paint_figure(&painter, &f, canvas.min, scale, 35, false);
                    }
                }
            }
        }
        let frame = doc.rendered(if self.playing {
            self.playhead
        } else {
            tick as f32
        });
        for figure in &frame.figures {
            Self::paint_figure(
                &painter,
                figure,
                canvas.min,
                scale,
                255,
                self.handles && !self.playing && figure.id == self.editor.selected,
            );
        }
        if self.playing {
            return;
        }
        if self.pan_tool {
            if response.dragged() {
                self.pan += response.drag_delta();
            }
            response.on_hover_cursor(egui::CursorIcon::Grab);
            return;
        }
        let camera = doc.camera(time);
        let (width, height) = (doc.width, doc.height);
        let world = |p: Pos2| {
            camera.unproject(
                Point::new((p.x - canvas.min.x) / scale, (p.y - canvas.min.y) / scale),
                width,
                height,
            )
        };
        if self.build && response.clicked() {
            if let Some(pointer) = response
                .interact_pointer_pos()
                .filter(|p| canvas.contains(*p))
            {
                let uid = self.editor.selected;
                let parent = self.selected_joint;
                let target = world(pointer);
                let circle = self.circle;
                let result = self.editor.change(|d| {
                    let f = d.figure_mut(tick, uid)?;
                    let parent = parent.filter(|p| *p < f.bones.len());
                    let origin = parent.map(|p| f.points()[p]).unwrap_or(f.root);
                    let length = origin.distance(target);
                    if length < 1. {
                        return Ok(());
                    }
                    f.bones.push(Bone {
                        name: if circle {
                            "Circle".into()
                        } else {
                            "Line".into()
                        },
                        parent,
                        length,
                        angle: (target.y - origin.y).atan2(target.x - origin.x),
                        width: 12.,
                        circle,
                        ik: false,
                        bend: 1,
                    });
                    Ok(())
                });
                self.status = result.err().unwrap_or_else(|| "Added part".into());
            }
        } else if response.drag_started() || response.clicked() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let target = world(pointer);
                let mut nearest = None;
                let mut distance = 14. / (scale * camera.zoom);
                for f in doc.sample(time).figures.iter().rev() {
                    if doc
                        .owner(f.id)
                        .and_then(|id| doc.layer(id).ok())
                        .is_none_or(|l| l.locked)
                    {
                        continue;
                    }
                    let d = f.root.distance(target);
                    if d < distance {
                        distance = d;
                        nearest = Some((
                            f.id,
                            None,
                            Point::new(target.x - f.root.x, target.y - f.root.y),
                        ));
                    }
                    for (index, point) in f.points().iter().enumerate() {
                        let d = point.distance(target);
                        if d < distance {
                            distance = d;
                            nearest = Some((f.id, Some(index), Point::default()));
                        }
                    }
                }
                if let Some((uid, joint, offset)) = nearest {
                    self.editor.selected = uid;
                    self.editor.track = Track::Layer(self.editor.doc.owner(uid).unwrap());
                    self.selected_joint = joint;
                    if response.drag_started() {
                        self.drag = Some((self.editor.doc.clone(), joint, offset));
                    }
                }
            }
        }
        if response.dragged() {
            if let (Some((_, joint, offset)), Some(pointer)) =
                (&self.drag, response.interact_pointer_pos())
            {
                let target = world(pointer);
                let uid = self.editor.selected;
                if let Ok(figure) = self.editor.doc.figure_mut(tick, uid) {
                    if let Some(index) = joint {
                        let _ = figure.pose(*index, target);
                    } else {
                        figure.root = Point::new(target.x - offset.x, target.y - offset.y);
                    }
                }
            }
        }
        if response.drag_stopped() {
            if let Some((before, _, _)) = self.drag.take() {
                if self.editor.doc.validate().is_ok() {
                    self.editor.record(before);
                } else {
                    self.editor.doc = before;
                }
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some((before, _, _)) = self.drag.take() {
                self.editor.doc = before;
            }
        }
    }
}
impl Drop for App {
    fn drop(&mut self) {
        if let Some((cancel, _)) = &self.export {
            cancel.store(true, Ordering::Relaxed);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frames += 1;
        if let Some(bridge) = &self.bridge {
            while let Ok(request) = bridge.receiver.try_recv() {
                let selected = self.editor.selected;
                let result = if Instant::now() > request.deadline {
                    Err("Command expired before execution".into())
                } else if self.drag.is_some()
                    || self.pending.is_some()
                    || self.quit_prompt
                    || self.playing
                        && !["get_project", "get_frame_preview", "get_figure"]
                            .contains(&request.method.as_str())
                {
                    Err("Pause playback and finish the current interaction first".into())
                } else {
                    api::execute(&mut self.editor, &request.method, &request.args)
                };
                if self.editor.selected != selected {
                    self.selected_joint = None;
                }
                let _ = request.response.send(result);
            }
        }
        if let Some((_, receiver)) = &self.export {
            if let Ok(message) = receiver.try_recv() {
                self.status = message;
                self.export = None;
            }
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        if self.playing {
            let now = Instant::now();
            self.playhead = (self.playhead
                + (now - self.last).as_secs_f32() * self.editor.doc.fps as f32)
                % self.editor.doc.duration as f32;
            self.editor.tick = self.playhead.floor() as u32;
            self.last = now;
            ctx.request_repaint();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) && !ctx.wants_keyboard_input() {
            self.playing = !self.playing;
            self.last = Instant::now();
            self.playhead = self.editor.tick as f32;
        }
        if ctx.input(|i| i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z))
            && self.drag.is_none()
            && !ctx.wants_keyboard_input()
        {
            self.playing = false;
            self.editor.undo();
        }
        if ctx.input(|i| {
            i.modifiers.command
                && (i.key_pressed(egui::Key::Y) || i.modifiers.shift && i.key_pressed(egui::Key::Z))
        }) && self.drag.is_none()
            && !ctx.wants_keyboard_input()
        {
            self.playing = false;
            self.editor.redo();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save(false);
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.editor.dirty && !self.force_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.quit_prompt = true;
            self.playing = false;
        }
        if !matches!(self.editor.track, Track::Layer(id) if self.editor.doc.layer(id).is_ok())
            && matches!(self.editor.track, Track::Layer(_))
        {
            self.editor.track = Track::Layer(self.editor.doc.layers[0].id);
        }
        self.toolbar(ctx);
        self.timeline(ctx);
        self.inspector(ctx);
        self.stage(ctx);
        self.unsaved_dialog(ctx);
        if self.capture.is_some() && self.frames >= 5 && !self.capture_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            self.capture_requested = true;
        }
        let screenshots = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|event| {
                    if let egui::Event::Screenshot { image, .. } = event {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        });
        if let (Some(path), Some(image)) = (self.capture.as_ref(), screenshots.first()) {
            let bytes = image
                .pixels
                .iter()
                .flat_map(|p| p.to_array())
                .collect::<Vec<_>>();
            if let Some(buffer) =
                image::RgbaImage::from_raw(image.size[0] as u32, image.size[1] as u32, bytes)
            {
                if let Err(error) = buffer.save(path) {
                    eprintln!("Screenshot failed: {error}");
                }
            }
            self.force_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.capture.is_some() {
            ctx.request_repaint_after(Duration::from_millis(30));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn draw(app: &mut App, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 800.))),
                events,
                ..Default::default()
            },
            |ctx| {
                app.toolbar(ctx);
                app.timeline(ctx);
                app.inspector(ctx);
                app.stage(ctx);
                app.unsaved_dialog(ctx);
            },
        )
    }
    fn label(output: &egui::FullOutput, text: &str) -> Pos2 {
        output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == text && t.pos.y > 500. => {
                    Some(t.pos + t.galley.size() / 2.)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("Missing visible label: {text}"))
    }
    fn click(app: &mut App, ctx: &egui::Context, p: Pos2) {
        for pressed in [true, false] {
            draw(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(p),
                    egui::Event::PointerButton {
                        pos: p,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    #[test]
    fn timeline_rows_select_real_tracks_and_add_independent_layers() {
        let ctx = egui::Context::default();
        let mut app = App::create(&ctx, Document::default(), None);
        draw(&mut app, &ctx, vec![]);
        let output = draw(&mut app, &ctx, vec![]);
        let camera = label(&output, "Camera");
        let scene = label(&output, "Scene");
        let layer = label(&output, "Layer 1");
        assert!(
            camera.y < scene.y && scene.y < layer.y,
            "Track labels must be separate rows"
        );
        click(&mut app, &ctx, camera);
        assert_eq!(app.editor.track, Track::Camera);
        click(&mut app, &ctx, scene);
        assert_eq!(app.editor.track, Track::Scene);
        let output = draw(&mut app, &ctx, vec![]);
        let add = label(&output, "+");
        click(&mut app, &ctx, add);
        assert_eq!(app.editor.doc.layers.len(), 2);
        assert_eq!(app.editor.track, Track::Layer(2));
        assert_eq!(app.editor.doc.layers[0].keys.len(), 1);
        assert!(app.editor.doc.layers[1].keys[&0].figures.is_empty());
        assert_eq!(app.editor.doc.camera_keys.len(), 1);
        assert_eq!(app.editor.doc.scene_keys.len(), 1);
    }
    fn text_position(output: &egui::FullOutput, text: &str) -> Pos2 {
        output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(t) if t.galley.job.text == text => {
                    Some(t.pos + t.galley.size() / 2.)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("Missing modal text: {text}"))
    }
    #[test]
    fn unsaved_dialog_actions_share_one_row_and_cancel_keeps_work() {
        let ctx = egui::Context::default();
        let mut app = App::create(&ctx, Document::default(), None);
        app.edit("move_figure", serde_json::json!({"uid":1,"dx":20,"dy":0}));
        let before = app.editor.doc.clone();
        app.replace(Document::walk());
        draw(&mut app, &ctx, vec![]);
        draw(&mut app, &ctx, vec![]);
        let output = draw(&mut app, &ctx, vec![]);
        let cancel = text_position(&output, "Cancel");
        let discard = text_position(&output, "Discard");
        let save = text_position(&output, "Save changes");
        assert!((cancel.y - discard.y).abs() < 1. && (cancel.y - save.y).abs() < 1.);
        assert!(cancel.x < discard.x && discard.x < save.x);
        click(&mut app, &ctx, cancel);
        assert!(app.pending.is_none());
        assert!(app.editor.dirty);
        assert_eq!(app.editor.doc, before);
        app.replace(Document::walk());
        draw(&mut app, &ctx, vec![]);
        draw(
            &mut app,
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(app.pending.is_none());
        assert_eq!(app.editor.doc, before);
        assert!(app.editor.dirty);
    }
    #[test]
    fn save_before_replacement_preserves_work_and_failed_save_stays_open() {
        let ctx = egui::Context::default();
        let mut app = App::create(&ctx, Document::default(), None);
        let directory = tempfile::tempdir().unwrap();
        let protected = directory.path().join("original.styk");
        std::fs::write(&protected, b"original bytes").unwrap();
        app.edit("move_figure", serde_json::json!({"uid":1,"dx":20,"dy":0}));
        let before = app.editor.doc.clone();
        let next = Document::walk();
        app.replace(next.clone());
        app.path = Some(protected.clone());
        app.resolve_unsaved(&ctx, UnsavedChoice::Save);
        assert_eq!(std::fs::read(protected).unwrap(), b"original bytes");
        assert!(app.pending.is_some());
        assert!(app.save_error.is_some());
        assert!(app.editor.dirty);
        assert_eq!(app.editor.doc, before);
        let saved = directory.path().join("saved.rustanim.json");
        app.path = Some(saved.clone());
        app.resolve_unsaved(&ctx, UnsavedChoice::Save);
        assert_eq!(Document::open(&saved).unwrap(), before);
        assert_eq!(app.editor.doc, next);
        assert!(app.pending.is_none());
        assert!(app.save_error.is_none());
        assert!(!app.editor.dirty);
    }
    #[test]
    fn discard_replaces_without_saving_and_close_cancel_keeps_document() {
        let ctx = egui::Context::default();
        let mut app = App::create(&ctx, Document::default(), None);
        app.edit("move_figure", serde_json::json!({"uid":1,"dx":20,"dy":0}));
        let before = app.editor.doc.clone();
        app.quit_prompt = true;
        app.resolve_unsaved(&ctx, UnsavedChoice::Cancel);
        assert!(!app.quit_prompt);
        assert!(!app.force_close);
        assert_eq!(app.editor.doc, before);
        assert!(app.editor.dirty);
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("untouched.rustanim.json");
        app.path = Some(file.clone());
        let next = Document::walk();
        app.replace(next.clone());
        app.resolve_unsaved(&ctx, UnsavedChoice::Discard);
        assert_eq!(app.editor.doc, next);
        assert!(!app.editor.dirty);
        assert!(!file.exists());
    }
}

use crate::{
    api, mcp,
    model::{Bone, Document, Editor, Figure, Point},
    render,
};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
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
}
impl App {
    pub fn new(
        context: &eframe::CreationContext<'_>,
        doc: Document,
        capture: Option<PathBuf>,
    ) -> Self {
        context.egui_ctx.set_visuals(egui::Visuals::dark());
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
            status: "Native Rust preview — full feature parity is still in development".into(),
            pending: None,
            quit_prompt: false,
            force_close: false,
            bridge: None,
            export: None,
            capture,
            capture_requested: false,
            frames: 0,
        };
        app.editor.doc = doc;
        if std::env::var("STYK_MCP").as_deref() == Ok("1") {
            match mcp::start(context.egui_ctx.clone()) {
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
    }
    fn replace(&mut self, doc: Document) {
        if self.editor.dirty {
            self.pending = Some(doc);
        } else {
            self.load(doc);
        }
    }
    fn save(&mut self, save_as: bool) -> bool {
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
                self.status = error;
                false
            }
        }
    }
    fn edit(&mut self, method: &str, args: serde_json::Value) {
        self.playing = false;
        match api::execute(&mut self.editor, method, &args) {
            Ok(_) => self.status = "Edit complete".into(),
            Err(error) => self.status = error,
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
                painter.line_segment(
                    [screen(start), screen(end)],
                    Stroke::new(bone.width * scale, color),
                );
                painter.circle_filled(screen(start), bone.width * scale / 2., color);
                painter.circle_filled(screen(end), bone.width * scale / 2., color);
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
        let scale = ((rect.width() - 32.) / doc.width as f32)
            .min((rect.height() - 32.) / doc.height as f32)
            .max(0.01);
        let canvas = Rect::from_center_size(
            rect.center(),
            Vec2::new(doc.width as f32, doc.height as f32) * scale,
        );
        ui.painter()
            .rect_filled(rect, 8., Color32::from_rgb(24, 29, 39));
        ui.painter().rect_filled(
            canvas,
            0.,
            Color32::from_rgb(doc.background[0], doc.background[1], doc.background[2]),
        );
        let painter = ui.painter().with_clip_rect(canvas);
        let tick = self.editor.tick;
        if self.onion && !self.playing {
            for t in [tick.saturating_sub(1), (tick + 1).min(doc.duration - 1)] {
                if t != tick {
                    for f in doc.sample(t as f32).figures {
                        Self::paint_figure(&painter, &f, canvas.min, scale, 35, false);
                    }
                }
            }
        }
        let frame = doc.sample(if self.playing {
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
        let world =
            |p: Pos2| Point::new((p.x - canvas.min.x) / scale, (p.y - canvas.min.y) / scale);
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
                let mut distance = 14. / scale;
                for f in frame.figures.iter().rev() {
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
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) && self.drag.is_none()
        {
            self.playing = false;
            self.editor.undo();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save(false);
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.editor.dirty && !self.force_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.quit_prompt = true;
        }
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("StykAnimate · Rust preview");
                ui.separator();
                if ui.button("New").clicked() {
                    self.replace(Document::default());
                }
                if ui.button("Open…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Rust preview", &["json"])
                        .pick_file()
                    {
                        match Document::open(&path) {
                            Ok(d) => self.replace(d),
                            Err(e) => self.status = e,
                        }
                    }
                }
                if ui.button("Save").clicked() {
                    self.save(false);
                }
                if ui.button("Save as…").clicked() {
                    self.save(true);
                }
                ui.menu_button("Export", |ui| {
                    for format in ["png", "svg", "gif"] {
                        if ui.button(format.to_uppercase()).clicked() {
                            self.export(format);
                            ui.close_menu();
                        }
                    }
                });
                if ui.button("Undo").clicked() {
                    self.playing = false;
                    self.editor.undo();
                }
                if ui.button("Redo").clicked() {
                    self.playing = false;
                    self.editor.redo();
                }
            });
            ui.horizontal(|ui| {
                if ui
                    .button(if self.playing { "Pause" } else { "Play" })
                    .clicked()
                {
                    self.playing = !self.playing;
                    self.last = Instant::now();
                    self.playhead = self.editor.tick as f32;
                }
                if ui.button("Add figure").clicked() {
                    self.edit("add_figure", serde_json::json!({}));
                }
                if ui.button("Walk demo").clicked() {
                    self.replace(Document::walk());
                }
                ui.checkbox(&mut self.handles, "Joints");
                ui.checkbox(&mut self.onion, "Onion skins");
                ui.checkbox(&mut self.build, "Build");
                if self.build {
                    ui.checkbox(&mut self.circle, "Circle");
                    ui.label("Select a joint, then click to add a part");
                }
                ui.separator();
                ui.label(format!(
                    "{} × {}   ·   {} FPS",
                    self.editor.doc.width, self.editor.doc.height, self.editor.doc.fps
                ));
            });
        });
        egui::SidePanel::right("inspector").default_width(235.).show(ctx,|ui|{
            ui.heading("Figures");let frame=self.editor.doc.sample(self.editor.tick as f32);
            for f in &frame.figures{if ui.selectable_label(f.id==self.editor.selected,&f.name).clicked(){self.editor.selected=f.id;self.selected_joint=None;}}
            ui.separator();
            if let Some(f)=frame.figures.iter().find(|f|f.id==self.editor.selected){
                let uid=f.id;ui.label(format!("Origin  {:.1}, {:.1}",f.root.x,f.root.y));
                let mut color=f.color;if ui.color_edit_button_srgb(&mut color).changed(){self.edit("set_figure_color",serde_json::json!({"uid":uid,"color":format!("#{:02x}{:02x}{:02x}",color[0],color[1],color[2])}));}
                if ui.button("Duplicate").clicked(){self.edit("duplicate_figure",serde_json::json!({"uid":uid}));}
                if ui.button("Delete figure").clicked(){let tick=self.editor.tick;let _=self.editor.change(|d|{d.key(tick).figures.retain(|f|f.id!=uid);Ok(())});}
                ui.separator();ui.label("Structure");
                egui::ScrollArea::vertical().max_height(290.).show(ui,|ui|{for (i,b) in f.bones.iter().enumerate(){if ui.selectable_label(self.selected_joint==Some(i),&b.name).clicked(){self.selected_joint=Some(i);}}});
                if let Some(index)=self.selected_joint.filter(|i|*i<f.bones.len()){
                    ui.separator();let bone=&f.bones[index];ui.strong(&bone.name);ui.label(format!("Length {:.1} · Angle {:.1}°",bone.length,bone.angle.to_degrees()));
                    if bone.parent.is_some(){let mut ik=bone.ik;if ui.checkbox(&mut ik,"Two-bone IK").changed(){let tick=self.editor.tick;let _=self.editor.change(|d|{d.figure_mut(tick,uid)?.bones[index].ik=ik;Ok(())});}
                        for (label,bend) in [("Bend left",-1),("Bend right",1)]{if ui.button(label).clicked(){self.edit("configure_ik",serde_json::json!({"uid":uid,"jointId":index+1,"bendDirection":bend}));}}
                    }
                }
            }
            ui.separator();ui.label("Project timing");let mut fps=self.editor.doc.fps;if ui.add(egui::DragValue::new(&mut fps).range(1..=240).prefix("FPS ")).changed(){self.edit("set_fps",serde_json::json!({"fps":fps}));}
            let mut duration=self.editor.doc.duration;
            if ui.add(egui::DragValue::new(&mut duration).range(1..=10000).prefix("Frames ")).changed(){let _=self.editor.change(|d|{d.duration=duration;d.keys.retain(|tick,_|*tick<duration);Ok(())});self.editor.tick=self.editor.tick.min(duration-1);}
        });
        egui::TopBottomPanel::bottom("timeline").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Timeline");
                ui.label(format!(
                    "Frame {} / {}",
                    self.editor.tick + 1,
                    self.editor.doc.duration
                ));
                if ui.button("Insert key").clicked() {
                    self.edit(
                        "insert_keyframe",
                        serde_json::json!({"tick":self.editor.tick}),
                    );
                }
                if self.editor.tick > 0 && ui.button("Delete key").clicked() {
                    let tick = self.editor.tick;
                    let _ = self.editor.change(|d| {
                        d.keys.remove(&tick);
                        Ok(())
                    });
                }
                if self.bridge.is_some() {
                    ui.label("MCP enabled");
                }
            });
            if ui
                .add(
                    egui::Slider::new(&mut self.editor.tick, 0..=self.editor.doc.duration - 1)
                        .show_value(false),
                )
                .changed()
            {
                self.playing = false;
            }
            egui::ScrollArea::horizontal()
                .id_salt("keyframes")
                .show_viewport(ui, |ui, viewport| {
                    let (rect, _) = ui.allocate_exact_size(
                        Vec2::new(self.editor.doc.duration as f32 * 30., 34.),
                        Sense::hover(),
                    );
                    let start = (viewport.min.x / 30.).floor().max(0.) as u32;
                    let end =
                        ((viewport.max.x / 30.).ceil() as u32 + 1).min(self.editor.doc.duration);
                    for tick in start..end {
                        let label = if self.editor.doc.keys.contains_key(&tick) {
                            format!("{}\n•", tick + 1)
                        } else {
                            format!("{}", tick + 1)
                        };
                        if ui
                            .put(
                                Rect::from_min_size(
                                    rect.min + Vec2::new(tick as f32 * 30., 0.),
                                    Vec2::new(28., 32.),
                                ),
                                egui::Button::new(label).selected(tick == self.editor.tick),
                            )
                            .clicked()
                        {
                            self.playing = false;
                            self.editor.tick = tick;
                        }
                    }
                });
            ui.horizontal(|ui| {
                ui.label(&self.status);
                if let Some((cancel, _)) = &self.export {
                    if ui.button("Cancel export").clicked() {
                        cancel.store(true, Ordering::Relaxed);
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| self.canvas(ui));
        if self.pending.is_some() || self.quit_prompt {
            egui::Window::new("Unsaved changes")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Save the current preview project before continuing?");
                    let mut proceed = false;
                    if ui.button("Save and continue").clicked() {
                        proceed = self.save(false);
                    }
                    if ui.button("Discard changes").clicked() {
                        proceed = true;
                    }
                    if proceed {
                        if self.quit_prompt {
                            self.force_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        } else if let Some(doc) = self.pending.take() {
                            self.load(doc);
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.pending = None;
                        self.quit_prompt = false;
                    }
                });
        }
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

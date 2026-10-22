use super::*;
use egui::{Align, FontId, Layout, RichText};
use serde_json::json;

const INK: Color32 = Color32::from_rgb(31, 38, 53);
const MUTED: Color32 = Color32::from_rgb(112, 121, 140);
const BLUE: Color32 = Color32::from_rgb(0, 151, 245);
const LINE: Color32 = Color32::from_rgb(229, 232, 240);
const SOFT: Color32 = Color32::from_rgb(244, 245, 250);

fn icon(ui: &egui::Ui, rect: Rect, kind: usize, color: Color32) {
    let p = ui.painter();
    let c = rect.center();
    let stroke = Stroke::new(1.6, color);
    let at = |x, y| c + Vec2::new(x, y);
    match kind {
        0 => {
            p.line_segment([at(-6., 0.), at(6., 0.)], stroke);
            p.line_segment([at(0., -6.), at(0., 6.)], stroke);
        }
        1 => {
            p.add(egui::Shape::closed_line(
                vec![at(-6., -7.), at(7., -2.), at(1., 1.), at(-2., 7.)],
                stroke,
            ));
        }
        2 => {
            p.line_segment([at(-6., 5.), at(0., -5.)], stroke);
            p.line_segment([at(0., -5.), at(6., 4.)], stroke);
            for pt in [at(-6., 5.), at(0., -5.), at(6., 4.)] {
                p.circle_filled(pt, 2.2, color);
            }
        }
        3 => {
            p.line_segment([at(-7., 0.), at(7., 0.)], stroke);
            p.line_segment([at(0., -7.), at(0., 7.)], stroke);
            for (x, y, dx, dy) in [
                (-7., 0., 3., 3.),
                (7., 0., -3., 3.),
                (0., -7., 3., 3.),
                (0., 7., 3., -3.),
            ] {
                let points = if x != 0. {
                    vec![at(x + dx, y - dy), at(x, y), at(x + dx, y + dy)]
                } else {
                    vec![at(x - dx, y + dy), at(x, y), at(x + dx, y + dy)]
                };
                p.add(egui::Shape::line(points, stroke));
            }
        }
        4 => {
            p.add(egui::Shape::ellipse_stroke(c, Vec2::new(7., 4.), stroke));
            p.circle_filled(c, 2., color);
        }
        5 => {
            p.add(egui::Shape::closed_line(
                vec![at(-5., 0.), at(5., 0.), at(5., 7.), at(-5., 7.)],
                stroke,
            ));
            p.add(egui::Shape::line(
                vec![at(-3., 0.), at(-3., -5.), at(3., -5.), at(3., 0.)],
                stroke,
            ));
        }
        6 | 7 => {
            let flip = if kind == 6 { 1. } else { -1. };
            let at = |x: f32, y| c + Vec2::new(x * flip, y);
            p.add(egui::Shape::line(
                vec![
                    at(-6., -2.),
                    at(1., -2.),
                    at(5., 1.),
                    at(5., 5.),
                    at(1., 7.),
                ],
                stroke,
            ));
            p.add(egui::Shape::line(
                vec![at(-2., -6.), at(-6., -2.), at(-2., 2.)],
                stroke,
            ));
        }
        8 => {
            p.add(egui::Shape::closed_line(
                vec![
                    at(-6., -7.),
                    at(4., -7.),
                    at(7., -4.),
                    at(7., 7.),
                    at(-6., 7.),
                ],
                stroke,
            ));
            p.add(egui::Shape::line(
                vec![at(-2., -7.), at(-2., -1.), at(3., -1.), at(3., -7.)],
                stroke,
            ));
            p.add(egui::Shape::line(
                vec![at(-2., 7.), at(-2., 3.), at(3., 3.), at(3., 7.)],
                stroke,
            ));
        }
        9 => {
            p.add(egui::Shape::closed_line(
                vec![at(-3., -3.), at(6., -3.), at(6., 6.), at(-3., 6.)],
                stroke,
            ));
            p.add(egui::Shape::line(
                vec![at(-6., 3.), at(-6., -6.), at(3., -6.)],
                stroke,
            ));
        }
        10 => {
            p.line_segment([at(-7., -4.), at(7., -4.)], stroke);
            p.add(egui::Shape::line(
                vec![at(-5., -4.), at(-4., 7.), at(4., 7.), at(5., -4.)],
                stroke,
            ));
            p.line_segment([at(-2., -7.), at(2., -7.)], stroke);
            for x in [-2., 2.] {
                p.line_segment([at(x, -1.), at(x, 4.)], stroke);
            }
        }
        11 => {
            p.add(egui::Shape::convex_polygon(
                vec![at(-4., -7.), at(7., 0.), at(-4., 7.)],
                color,
                Stroke::NONE,
            ));
        }
        12 => {
            for x in [-3., 3.] {
                p.line_segment([at(x, -6.), at(x, 6.)], Stroke::new(3., color));
            }
        }
        13..=15 => {
            let flip = if kind == 15 { -1. } else { 1. };
            p.add(egui::Shape::line(
                vec![at(3. * flip, -5.), at(-3. * flip, 0.), at(3. * flip, 5.)],
                stroke,
            ));
            if kind == 13 {
                p.line_segment([at(-6., -6.), at(-6., 6.)], stroke);
            }
        }
        16 => {
            p.add(egui::Shape::closed_line(
                vec![at(-7., -4.), at(2., -4.), at(2., 5.), at(-7., 5.)],
                stroke,
            ));
            p.add(egui::Shape::closed_line(
                vec![at(2., -2.), at(7., -5.), at(7., 6.), at(2., 3.)],
                stroke,
            ));
        }
        17 => {
            p.add(egui::Shape::closed_line(
                vec![at(-7., -5.), at(7., -5.), at(7., 6.), at(-7., 6.)],
                stroke,
            ));
            p.line_segment([at(-7., -1.), at(7., -1.)], stroke);
        }
        18 => {
            for y in [-4., 0., 4.] {
                p.add(egui::Shape::line(
                    vec![at(-7., y), at(0., y + 3.), at(7., y)],
                    stroke,
                ));
            }
        }
        _ => {}
    }
}

fn semibold(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text).font(FontId::new(size, egui::FontFamily::Name("semibold".into())))
}

fn icon_button(ui: &mut egui::Ui, kind: usize, tip: &str, primary: bool) -> egui::Response {
    let response = ui.add_sized(
        Vec2::new(32., 32.),
        egui::Button::new("")
            .fill(if primary { BLUE } else { Color32::TRANSPARENT })
            .corner_radius(9),
    );
    icon(
        ui,
        response.rect,
        kind,
        if primary { Color32::WHITE } else { MUTED },
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    response.on_hover_text(tip)
}

pub(super) fn theme(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    // Use installed Windows UI fonts without redistributing them; the bundled
    // egui fonts remain the fallback on other systems or missing font files.
    let mut fonts = egui::FontDefinitions::default();
    let mut headings = fonts.families[&egui::FontFamily::Proportional].clone();
    if cfg!(target_os = "windows") {
        if let Some(windows) = std::env::var_os("WINDIR") {
            let folder = PathBuf::from(windows).join("Fonts");
            for (name, file) in [
                ("system-ui", "segoeui.ttf"),
                ("system-semibold", "seguisb.ttf"),
            ] {
                if let Ok(bytes) = std::fs::read(folder.join(file)) {
                    fonts
                        .font_data
                        .insert(name.into(), egui::FontData::from_owned(bytes).into());
                    if name == "system-ui" {
                        fonts
                            .families
                            .get_mut(&egui::FontFamily::Proportional)
                            .unwrap()
                            .insert(0, name.into());
                    } else {
                        headings.insert(0, name.into());
                    }
                }
            }
        }
    }
    fonts
        .families
        .insert(egui::FontFamily::Name("semibold".into()), headings);
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(13.));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(13.));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(11.));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(17.));
    style.spacing.item_spacing = Vec2::new(8., 7.);
    style.spacing.button_padding = Vec2::new(12., 7.);
    style.spacing.interact_size.y = 30.;
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(INK);
    style.visuals.panel_fill = Color32::WHITE;
    style.visuals.window_fill = Color32::WHITE;
    style.visuals.extreme_bg_color = SOFT;
    style.visuals.faint_bg_color = SOFT;
    style.visuals.selection.bg_fill = Color32::from_rgb(224, 242, 255);
    style.visuals.selection.stroke = Stroke::new(1., BLUE);
    style.visuals.window_corner_radius = 12.into();
    for w in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        w.corner_radius = 8.into();
        w.bg_stroke = Stroke::NONE;
    }
    style.visuals.widgets.inactive.bg_fill = SOFT;
    style.visuals.widgets.inactive.weak_bg_fill = SOFT;
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(232, 240, 250);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(232, 240, 250);
    ctx.set_style(style);
}

fn section(ui: &mut egui::Ui, title: &str, color: Color32) {
    ui.add_space(6.);
    ui.separator();
    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.), Sense::hover());
    ui.painter()
        .circle_filled(r.left_center() + Vec2::new(4., 0.), 3., color);
    ui.painter().text(
        r.left_center() + Vec2::new(18., 0.),
        egui::Align2::LEFT_CENTER,
        title,
        FontId::new(13., egui::FontFamily::Name("semibold".into())),
        INK,
    );
}

impl App {
    pub(super) fn unsaved_dialog(&mut self, ctx: &egui::Context) {
        if self.pending.is_none() && !self.quit_prompt {
            return;
        }
        let mut choice = None;
        let frame = egui::Frame::popup(&ctx.style())
            .fill(Color32::WHITE)
            .corner_radius(14)
            .inner_margin(24)
            .stroke(Stroke::new(1., LINE));
        let response = egui::Modal::new(egui::Id::new("unsaved_changes"))
            .frame(frame)
            .backdrop_color(Color32::from_black_alpha(64))
            .show(ctx, |ui| {
                ui.set_width(440.0_f32.min(ctx.screen_rect().width() - 80.));
                ui.spacing_mut().item_spacing = Vec2::new(8., 8.);
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(40.), Sense::hover());
                    ui.painter()
                        .rect_filled(r, 12., Color32::from_rgb(232, 245, 255));
                    icon(ui, r, 8, BLUE);
                    ui.add_space(6.);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 3.;
                        ui.label(semibold("Save your changes?", 19.));
                        let mut name = self.editor.doc.name.chars().take(64).collect::<String>();
                        if self.editor.doc.name.chars().count() > 64 {
                            name.push('…');
                        }
                        ui.label(RichText::new(name).size(12.).color(MUTED));
                    });
                });
                ui.add_space(10.);
                ui.label(
                    RichText::new(if self.quit_prompt {
                        "Save your work before closing StykAnimate."
                    } else {
                        "Save your work before opening another animation."
                    })
                    .size(13.)
                    .color(INK),
                );
                ui.label(
                    RichText::new("Unsaved edits will be lost if you discard them.")
                        .size(12.)
                        .color(MUTED),
                );
                if let Some(error) = &self.save_error {
                    ui.add_space(4.);
                    ui.colored_label(
                        Color32::from_rgb(190, 57, 57),
                        format!("Couldn’t save: {error}"),
                    );
                }
                ui.add_space(18.);
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 336.).max(0.));
                    if ui
                        .add_sized(
                            Vec2::new(88., 36.),
                            egui::Button::new("Cancel")
                                .fill(Color32::WHITE)
                                .stroke(Stroke::new(1., LINE)),
                        )
                        .clicked()
                    {
                        choice = Some(UnsavedChoice::Cancel);
                    }
                    if ui
                        .add_sized(
                            Vec2::new(108., 36.),
                            egui::Button::new("Discard").fill(SOFT),
                        )
                        .clicked()
                    {
                        choice = Some(UnsavedChoice::Discard);
                    }
                    if ui
                        .add_sized(
                            Vec2::new(124., 36.),
                            egui::Button::new(semibold("Save changes", 12.).color(Color32::WHITE))
                                .fill(BLUE),
                        )
                        .clicked()
                    {
                        choice = Some(UnsavedChoice::Save);
                    }
                });
            });
        if response.is_top_modal
            && !response.any_popup_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            choice = Some(UnsavedChoice::Cancel);
        }
        if let Some(choice) = choice {
            self.resolve_unsaved(ctx, choice);
        }
    }

    fn toggle_play(&mut self) {
        self.playing = !self.playing;
        self.last = Instant::now();
        self.playhead = self.editor.tick as f32;
    }
    fn open_dialog(&mut self) {
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
    pub(super) fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("app_header")
            .frame(
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .inner_margin(egui::Margin::symmetric(20, 12)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (mark, _) = ui.allocate_exact_size(Vec2::splat(28.), Sense::hover());
                    ui.painter().rect_filled(mark, 8., BLUE);
                    let c = mark.center();
                    let stroke = Stroke::new(1.7, Color32::WHITE);
                    ui.painter()
                        .circle_filled(c + Vec2::new(1., -7.), 2.1, Color32::WHITE);
                    for (a, b) in [
                        ((1., -3.), (-1., 3.)),
                        ((-1., 3.), (-6., 8.)),
                        ((-1., 3.), (5., 8.)),
                        ((0., 0.), (-6., -1.)),
                        ((0., 0.), (6., -3.)),
                    ] {
                        ui.painter().line_segment(
                            [c + Vec2::new(a.0, a.1), c + Vec2::new(b.0, b.1)],
                            stroke,
                        );
                    }
                    ui.label(semibold("StykAnimate", 18.));
                    if ui.available_width() > 1050. {
                        ui.label(RichText::new("RUST PREVIEW").size(9.).color(MUTED));
                    }
                    ui.add_space(10.);
                    ui.scope(|ui| {
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                        ui.menu_button("File", |ui| {
                            if ui.button("New animation").clicked() {
                                self.replace(Document::default());
                                ui.close_menu();
                            }
                            if ui.button("Open project…").clicked() {
                                self.open_dialog();
                                ui.close_menu();
                            }
                            if ui.button("Save as…").clicked() {
                                self.save(true);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Motion study demo").clicked() {
                                self.replace(crate::demos::showcase());
                                ui.close_menu();
                            }
                            if ui.button("Walking demo").clicked() {
                                self.replace(Document::walk());
                                ui.close_menu();
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if ui.button("Undo    Ctrl+Z").clicked() {
                                self.playing = false;
                                self.editor.undo();
                                ui.close_menu();
                            }
                            if ui.button("Redo    Ctrl+Y").clicked() {
                                self.playing = false;
                                self.editor.redo();
                                ui.close_menu();
                            }
                        });
                    });
                    if ui.available_width() > 620. {
                        ui.separator();
                        ui.add(egui::Label::new(semibold(&self.editor.doc.name, 12.)).truncate());
                        let text = if self.editor.dirty {
                            "Unsaved"
                        } else if self.path.is_some() {
                            "Saved"
                        } else {
                            "Ready"
                        };
                        ui.label(RichText::new(format!("• {text}")).size(11.).color(
                            if self.editor.dirty {
                                MUTED
                            } else {
                                Color32::from_rgb(37, 167, 130)
                            },
                        ));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.scope(|ui| {
                            ui.visuals_mut().widgets.inactive.weak_bg_fill = INK;
                            ui.menu_button(
                                semibold("Export  ↗", 12.).color(Color32::WHITE),
                                |ui| {
                                    ui.visuals_mut().widgets.inactive.weak_bg_fill = SOFT;
                                    for (format, label) in [
                                        ("png", "PNG image"),
                                        ("svg", "SVG artwork"),
                                        ("gif", "Animated GIF"),
                                    ] {
                                        if ui.button(label).clicked() {
                                            self.export(format);
                                            ui.close_menu();
                                        }
                                    }
                                },
                            );
                        });
                        if icon_button(ui, 8, "Save project · Ctrl+S", false).clicked() {
                            self.save(false);
                        }
                        ui.separator();
                        if icon_button(ui, 7, "Redo · Ctrl+Y", false).clicked() {
                            self.playing = false;
                            self.editor.redo();
                        }
                        if icon_button(ui, 6, "Undo · Ctrl+Z", false).clicked() {
                            self.playing = false;
                            self.editor.undo();
                        }
                        ui.add_space(14.);
                        egui::Frame::new()
                            .fill(SOFT)
                            .corner_radius(20)
                            .inner_margin(3)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.;
                                    for (name, build) in [("Animate", false), ("Build", true)] {
                                        if ui
                                            .add(
                                                egui::Button::new(semibold(name, 12.))
                                                    .fill(if self.build == build {
                                                        Color32::WHITE
                                                    } else {
                                                        Color32::TRANSPARENT
                                                    })
                                                    .corner_radius(16),
                                            )
                                            .clicked()
                                        {
                                            self.build = build;
                                            self.pan_tool = false;
                                        }
                                    }
                                });
                            });
                    });
                });
            });
        egui::TopBottomPanel::top("stage_header")
            .frame(
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .inner_margin(egui::Margin::symmetric(20, 7)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Scene").frame(false)).clicked() {
                        self.editor.track = Track::Scene;
                    }
                    ui.label(
                        RichText::new(format!(
                            "{} × {}",
                            self.editor.doc.width, self.editor.doc.height
                        ))
                        .size(11.)
                        .color(MUTED),
                    );
                    if self.build {
                        ui.separator();
                        ui.selectable_value(&mut self.circle, false, "Line");
                        ui.selectable_value(&mut self.circle, true, "Circle");
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new("Fit").frame(false))
                            .on_hover_text("Fit canvas and reset viewport pan")
                            .clicked()
                        {
                            self.zoom = 1.;
                            self.pan = Vec2::ZERO;
                        }
                        let mut percent = self.zoom * 100.;
                        if ui
                            .add(
                                egui::DragValue::new(&mut percent)
                                    .range(25..=300)
                                    .suffix("%")
                                    .speed(1.),
                            )
                            .on_hover_text("Viewport zoom")
                            .changed()
                        {
                            self.zoom = percent / 100.;
                        }
                        ui.separator();
                        ui.toggle_value(&mut self.grid, "Grid");
                        ui.toggle_value(&mut self.onion, "Onion skins");
                        ui.toggle_value(&mut self.handles, "Joints");
                    });
                });
            });
    }

    pub(super) fn inspector(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(300.)
            .width_range(260.0..=420.0)
            .frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(16))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(semibold("Properties", 14.));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let kind = match self.editor.track {
                            Track::Camera => "CAMERA",
                            Track::Scene => "SCENE",
                            Track::Layer(_) => "FIGURE / LAYER",
                        };
                        ui.label(RichText::new(kind).size(9.).color(MUTED));
                    });
                });
                ui.add_space(8.);
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("properties_scroll")
                    .show(ui, |ui| match self.editor.track {
                        Track::Camera => self.camera_properties(ui),
                        Track::Scene => self.scene_properties(ui),
                        Track::Layer(id) => self.figure_properties(ui, id),
                    });
            });
    }
    fn camera_properties(&mut self, ui: &mut egui::Ui) {
        section(ui, "Camera", Color32::from_rgb(215, 139, 43));
        if self.editor.doc.camera_locked {
            ui.label("Camera locked — unlock it in the timeline.");
            return;
        }
        ui.label(RichText::new("Frame the animation").color(MUTED));
        let mut camera = self.editor.doc.camera(self.editor.tick as f32);
        let mut angle = camera.angle.to_degrees();
        let mut changed = false;
        section(ui, "Transform", BLUE);
        ui.horizontal(|ui| {
            changed |= ui
                .add(
                    egui::DragValue::new(&mut camera.center.x)
                        .prefix("X  ")
                        .suffix(" px")
                        .speed(1.),
                )
                .changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut camera.center.y)
                        .prefix("Y  ")
                        .suffix(" px")
                        .speed(1.),
                )
                .changed();
        });
        let mut zoom = camera.zoom * 100.;
        changed |= ui
            .add(
                egui::Slider::new(&mut zoom, 5.0..=2000.0)
                    .text("Zoom %")
                    .logarithmic(true),
            )
            .changed();
        changed |= ui
            .add(
                egui::DragValue::new(&mut angle)
                    .prefix("Rotation  ")
                    .suffix("°")
                    .speed(0.5),
            )
            .changed();
        if changed {
            self.edit(
                "set_camera",
                json!({"x":camera.center.x,"y":camera.center.y,"zoom":zoom/100.,"angle":angle}),
            );
        }
        ui.add_space(8.);
        if ui.button("Reset camera").clicked() {
            self.edit("set_camera",json!({"x":self.editor.doc.width as f32/2.,"y":self.editor.doc.height as f32/2.,"zoom":1.,"angle":0.}));
        }
        section(ui, "Animation", MUTED);
        ui.label("Camera keys animate pan, zoom and rotation independently from figure poses.");
        if ui.button("Insert camera key").clicked() {
            self.edit("insert_keyframe", json!({"tick":self.editor.tick}));
        }
    }
    fn scene_properties(&mut self, ui: &mut egui::Ui) {
        section(ui, "Scene", Color32::from_rgb(129, 95, 229));
        if self.editor.doc.scene_locked {
            ui.label("Scene locked — unlock it in the timeline.");
            return;
        }
        section(ui, "Background", BLUE);
        let mut color = self.editor.doc.scene(self.editor.tick as f32).background;
        ui.horizontal(|ui| {
            if ui.color_edit_button_srgb(&mut color).changed() {
                self.edit("set_scene", json!({"background":color}));
            }
            ui.label(format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]));
        });
        ui.label(
            RichText::new("Background colors hold until the next Scene key.")
                .small()
                .color(MUTED),
        );
        section(ui, "Project timing", MUTED);
        let mut fps = self.editor.doc.fps;
        if ui
            .add(
                egui::DragValue::new(&mut fps)
                    .range(1..=240)
                    .prefix("Frame rate  ")
                    .suffix(" fps"),
            )
            .changed()
        {
            self.edit("set_fps", json!({"fps":fps}));
        }
        let mut duration = self.editor.doc.duration;
        if ui
            .add(
                egui::DragValue::new(&mut duration)
                    .range(1..=10000)
                    .prefix("Duration  ")
                    .suffix(" frames"),
            )
            .changed()
        {
            let result = self.editor.change(|d| d.resize_duration(duration));
            self.status = result.err().unwrap_or_else(|| "Duration updated".into());
            self.editor.tick = self.editor.tick.min(self.editor.doc.duration - 1);
        }
        let mut interpolation = self.editor.doc.interpolation;
        ui.horizontal(|ui| {
            ui.label("Between keys");
            ui.selectable_value(
                &mut interpolation,
                crate::model::Interpolation::Linear,
                "Linear",
            );
            ui.selectable_value(
                &mut interpolation,
                crate::model::Interpolation::Hold,
                "Hold",
            );
        });
        if interpolation != self.editor.doc.interpolation {
            let _ = self.editor.change(|d| {
                d.interpolation = interpolation;
                Ok(())
            });
        }
    }
    fn figure_properties(&mut self, ui: &mut egui::Ui, id: u64) {
        let Ok(layer) = self.editor.doc.layer(id).cloned() else {
            return;
        };
        let frame = self.editor.doc.sample_layer(id, self.editor.tick as f32);
        if !frame.figures.iter().any(|f| f.id == self.editor.selected) {
            self.layer_properties(ui, id);
            return;
        }
        if layer.locked {
            ui.label(
                RichText::new("Layer locked — unlock it in the timeline to edit.").color(MUTED),
            );
            return;
        }
        let Some(f) = frame.figures.iter().find(|f| f.id == self.editor.selected) else {
            return;
        };
        let uid = f.id;
        ui.add_space(12.);
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(30.), Sense::hover());
            ui.painter()
                .rect_filled(r, 9., Color32::from_rgb(f.color[0], f.color[1], f.color[2]));
            icon(ui, r, 2, Color32::WHITE);
            ui.label(semibold(&f.name, 17.));
        });
        ui.add_space(10.);
        let half = (ui.available_width() - 8.) / 2.;
        ui.horizontal(|ui| {
            if ui
                .add_sized(Vec2::new(half, 32.), egui::Button::new("Duplicate"))
                .clicked()
            {
                self.edit("duplicate_figure", json!({"uid":uid}));
            }
            if ui
                .add_sized(Vec2::new(half, 32.), egui::Button::new("Delete"))
                .clicked()
            {
                self.edit("delete_figure", json!({"uid":uid}));
            }
        });
        section(ui, "Transform", BLUE);
        let mut root = f.root;
        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add_sized(
                    Vec2::new(half, 34.),
                    egui::DragValue::new(&mut root.x)
                        .prefix("X  ")
                        .suffix(" px"),
                )
                .changed();
            changed |= ui
                .add_sized(
                    Vec2::new(half, 34.),
                    egui::DragValue::new(&mut root.y)
                        .prefix("Y  ")
                        .suffix(" px"),
                )
                .changed();
        });
        if changed {
            self.edit(
                "move_figure",
                json!({"uid":uid,"dx":root.x-f.root.x,"dy":root.y-f.root.y}),
            );
        }
        section(ui, "Appearance", Color32::from_rgb(241, 124, 74));
        let mut color = f.color;
        egui::Frame::new().fill(SOFT).corner_radius(9).inner_margin(8).show(ui,|ui| {ui.set_min_width(ui.available_width());ui.horizontal(|ui| {
            if ui.color_edit_button_srgb(&mut color).changed() {self.edit("set_figure_color",json!({"uid":uid,"color":format!("#{:02x}{:02x}{:02x}",color[0],color[1],color[2])}));}
            ui.label(RichText::new(format!("#{:02X}{:02X}{:02X}",color[0],color[1],color[2])).monospace().size(12.));
        });});
        section(ui, "Arrange", Color32::from_rgb(46, 177, 141));
        egui::ComboBox::from_id_salt("figure_layer")
            .width(ui.available_width())
            .selected_text(&layer.name)
            .show_ui(ui, |ui| {
                let layers = self
                    .editor
                    .doc
                    .layers
                    .iter()
                    .map(|l| (l.id, l.name.clone()))
                    .collect::<Vec<_>>();
                for (target, name) in layers {
                    if ui.selectable_label(target == id, name).clicked() {
                        self.edit("move_to_layer", json!({"uid":uid,"layerId":target}));
                    }
                }
            });
        ui.horizontal(|ui| {
            if ui.button("Layer forward").clicked() {
                self.edit("reorder_layer", json!({"layerId":id,"offset":1}));
            }
            if ui.button("Layer back").clicked() {
                self.edit("reorder_layer", json!({"layerId":id,"offset":-1}));
            }
        });
        ui.add_space(10.);
        egui::CollapsingHeader::new("Rig & joints")
            .default_open(false)
            .show(ui, |ui| {
                for (i, b) in f.bones.iter().enumerate() {
                    if ui
                        .selectable_label(self.selected_joint == Some(i), &b.name)
                        .clicked()
                    {
                        self.selected_joint = Some(i);
                    }
                }
                if let Some((i, b)) = self
                    .selected_joint
                    .and_then(|i| f.bones.get(i).map(|b| (i, b)))
                {
                    ui.separator();
                    ui.strong(&b.name);
                    ui.small(format!(
                        "Length {:.1} px   ·   Angle {:.1}°",
                        b.length,
                        b.angle.to_degrees()
                    ));
                    if b.parent.is_some() {
                        let mut ik = b.ik;
                        if ui.checkbox(&mut ik, "Two-bone IK").changed() {
                            let tick = self.editor.tick;
                            let _ = self.editor.change(|d| {
                                d.figure_mut(tick, uid)?.bones[i].ik = ik;
                                Ok(())
                            });
                        }
                        ui.horizontal(|ui| {
                            for (label, bend) in [("Bend left", -1), ("Bend right", 1)] {
                                if ui.button(label).clicked() {
                                    self.edit(
                                        "configure_ik",
                                        json!({"uid":uid,"jointId":i+1,"bendDirection":bend}),
                                    );
                                }
                            }
                        });
                    }
                }
            });
    }

    fn layer_properties(&mut self, ui: &mut egui::Ui, id: u64) {
        let Ok(layer) = self.editor.doc.layer(id).cloned() else {
            return;
        };
        let frame = self.editor.doc.sample_layer(id, self.editor.tick as f32);
        section(ui, &layer.name, BLUE);
        let mut name = layer.name.clone();
        let r = ui.text_edit_singleline(&mut name);
        if r.changed() && name != layer.name {
            self.edit("set_layer", json!({"layerId":id,"name":name}));
        }
        ui.horizontal(|ui| {
            if ui.button("+ Figure").clicked() {
                self.edit("add_figure", json!({}));
            }
            ui.label(
                RichText::new(format!(
                    "{} figure{}",
                    frame.figures.len(),
                    if frame.figures.len() == 1 { "" } else { "s" }
                ))
                .small()
                .color(MUTED),
            );
        });
        ui.horizontal(|ui| {
            let mut visible = layer.visible;
            let mut locked = layer.locked;
            if ui.checkbox(&mut visible, "Visible").changed() {
                self.edit("set_layer", json!({"layerId":id,"visible":visible}));
            }
            if ui.checkbox(&mut locked, "Locked").changed() {
                self.edit("set_layer", json!({"layerId":id,"locked":locked}));
            }
        });
        for f in &frame.figures {
            if ui
                .selectable_label(f.id == self.editor.selected, &f.name)
                .clicked()
            {
                self.editor.selected = f.id;
                self.selected_joint = None;
            }
        }
        ui.add_space(12.);
        if ui
            .add_enabled(
                self.editor.doc.layers.len() > 1 && !layer.locked,
                egui::Button::new("Delete layer"),
            )
            .clicked()
        {
            self.edit("delete_layer", json!({"layerId":id}));
        }
    }

    pub(super) fn timeline(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("timeline").resizable(true).default_height(274.).height_range(210.0..=480.0)
            .frame(egui::Frame::new().fill(Color32::WHITE).inner_margin(0)).show(ctx,|ui| {
                egui::Frame::new().inner_margin(egui::Margin::symmetric(16,8)).show(ui,|ui| {
                    ui.horizontal(|ui| {
                        ui.label(semibold("Timeline",13.));
                        ui.add_space(8.);
                        if ui.button("+ Keyframe").clicked() {self.edit("insert_keyframe",json!({"tick":self.editor.tick}));}
                        ui.menu_button("Key actions",|ui| {
                            if ui.add_enabled(matches!(self.editor.track,Track::Layer(_)),egui::Button::new("Insert blank exposure")).clicked() {self.edit("blank_keyframe",json!({"tick":self.editor.tick}));ui.close_menu();}
                            if ui.add_enabled(self.editor.tick>0,egui::Button::new("Delete keyframe")).clicked() {self.edit("delete_keyframe",json!({"tick":self.editor.tick}));ui.close_menu();}
                        });
                        ui.add_space(16.);ui.separator();ui.add_space(8.);
                        if icon_button(ui,13,"First frame",false).clicked() {self.editor.tick=0;self.playing=false;}
                        if icon_button(ui,14,"Previous frame",false).clicked() {self.editor.tick=self.editor.tick.saturating_sub(1);self.playing=false;}
                        if icon_button(ui,if self.playing {12}else{11},"Play / pause · Space",true).clicked() {self.toggle_play();}
                        if icon_button(ui,15,"Next frame",false).clicked() {self.editor.tick=(self.editor.tick+1).min(self.editor.doc.duration-1);self.playing=false;}
                        ui.add_space(10.);
                        ui.label(RichText::new(format!("{:02}:{:02}:{:02}",self.editor.tick/self.editor.doc.fps/60,self.editor.tick/self.editor.doc.fps%60,self.editor.tick%self.editor.doc.fps)).monospace().size(12.));
                        ui.label(RichText::new(format!("{} fps",self.editor.doc.fps)).size(11.).color(MUTED));
                        if ui.available_width()>160. {
                            ui.with_layout(Layout::right_to_left(Align::Center),|ui| {ui.label(RichText::new(format!("Frame {} / {}",self.editor.tick+1,self.editor.doc.duration)).size(11.).color(MUTED));});
                        }
                    });
                });
                ui.separator();
                let gutter=224.;let cell=14.;let row_height=32.;let ruler=30.;
                let mut rows=vec![
                    (Track::Camera,"Camera".to_owned(),Color32::from_rgb(190,135,53),self.editor.doc.camera_keys.keys().copied().collect::<Vec<_>>()),
                    (Track::Scene,"Scene".to_owned(),Color32::from_rgb(138,109,216),self.editor.doc.scene_keys.keys().copied().collect()),
                ];
                for l in self.editor.doc.layers.iter().rev() {
                    let color=l.keys.values().flat_map(|f|f.figures.iter()).next().map(|f|Color32::from_rgb(f.color[0],f.color[1],f.color[2])).unwrap_or(BLUE);
                    rows.push((Track::Layer(l.id),l.name.clone(),color,l.keys.keys().copied().collect()));
                }
                let available=(ui.available_height()-27.).max(80.);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing=Vec2::ZERO;
                    ui.allocate_ui_with_layout(Vec2::new(gutter,available),Layout::top_down(Align::Min),|ui| {
                        ui.spacing_mut().item_spacing.y=0.;
                        let (header,_)=ui.allocate_exact_size(Vec2::new(gutter,ruler),Sense::hover());
                        ui.painter().text(header.left_center()+Vec2::new(16.,0.),egui::Align2::LEFT_CENTER,"LAYERS",FontId::proportional(10.),MUTED);
                        if ui.put(Rect::from_center_size(Pos2::new(header.right()-22.,header.center().y),Vec2::new(24.,24.)),egui::Button::new("+").frame(false)).on_hover_text("Add layer").clicked() {self.edit("add_layer",json!({"name":format!("Layer {}",self.editor.doc.layers.len()+1)}));}
                        let names=egui::ScrollArea::vertical().id_salt("layer_names").vertical_scroll_offset(self.timeline_scroll).max_height(available-ruler).show(ui,|ui| {
                            for (track,name,color,_) in &rows {
                                let (r,_)=ui.allocate_exact_size(Vec2::new(gutter,row_height),Sense::hover());
                                if self.editor.track==*track {ui.painter().rect_filled(r,0.,Color32::from_rgb(234,246,255));}
                                ui.painter().line_segment([r.left_bottom(),r.right_bottom()],Stroke::new(0.6,LINE));
                                let symbol=match track {Track::Camera=>16,Track::Scene=>17,_=>18};
                                icon(ui,Rect::from_center_size(Pos2::new(r.left()+22.,r.center().y),Vec2::splat(20.)),symbol,*color);
                                let label=Rect::from_min_max(r.min+Vec2::new(40.,0.),r.max-Vec2::new(62.,0.));
                                let response=ui.interact(label,ui.id().with(format!("track-{track:?}")),Sense::click());
                                ui.painter().with_clip_rect(label).text(label.left_center(),egui::Align2::LEFT_CENTER,name,FontId::new(12.,egui::FontFamily::Name("semibold".into())),INK);
                                if response.clicked() {self.editor.track = *track;self.editor.selected=0;self.selected_joint=None;}
                                let lock=Rect::from_center_size(Pos2::new(r.right()-20.,r.center().y),Vec2::new(26.,26.));
                                if let Track::Layer(id)=track {
                                    if let Ok(layer)=self.editor.doc.layer(*id).cloned() {
                                        let eye=lock.translate(Vec2::new(-28.,0.));
                                        if ui.put(eye,egui::Button::new("").frame(false)).on_hover_text("Show / hide layer").clicked() {self.edit("set_layer",json!({"layerId":id,"visible":!layer.visible}));}
                                        icon(ui,eye,4,if layer.visible {MUTED}else{LINE});
                                        if ui.put(lock,egui::Button::new("").frame(false)).on_hover_text("Lock / unlock layer").clicked() {self.edit("set_layer",json!({"layerId":id,"locked":!layer.locked}));}
                                        icon(ui,lock,5,if layer.locked {BLUE}else{Color32::from_rgb(173,182,197)});
                                    }
                                } else {
                                    let locked=self.editor.doc.track_locked(*track);
                                    if ui.put(lock,egui::Button::new("").frame(false)).on_hover_text("Lock / unlock track").clicked() {self.edit("set_track_lock",json!({"track":if *track==Track::Camera {"camera"}else{"scene"},"locked":!locked}));}
                                    icon(ui,lock,5,if locked {BLUE}else{Color32::from_rgb(173,182,197)});
                                }
                            }
                        });
                        if ui.rect_contains_pointer(names.inner_rect) {self.timeline_scroll=names.state.offset.y;}
                    });
                    let grid=egui::ScrollArea::both().id_salt("track_grid").vertical_scroll_offset(self.timeline_scroll).max_height(available).show_viewport(ui,|ui,view| {
                        let width=(self.editor.doc.duration.max(100) as f32*cell).max(ui.available_width());
                        let (r,response)=ui.allocate_exact_size(Vec2::new(width,ruler+rows.len() as f32*row_height),Sense::click_and_drag());
                        let painter=ui.painter();let end_x=r.left()+self.editor.doc.duration as f32*cell;
                        painter.rect_filled(r,0.,SOFT);
                        painter.rect_filled(Rect::from_min_max(r.min,Pos2::new(end_x,r.bottom())),0.,Color32::WHITE);
                        let a=(view.left()/cell).floor().max(0.) as u32;
                        let b=((view.right()/cell).ceil() as u32+1).min(10000);
                        for (i,(track,_,color,keys)) in rows.iter().enumerate() {
                            let y=r.top()+ruler+i as f32*row_height;
                            let row=Rect::from_min_size(Pos2::new(r.left(),y),Vec2::new(width,row_height));
                            if self.editor.track==*track {painter.rect_filled(row,0.,Color32::from_rgb(239,248,255));}
                            painter.line_segment([row.left_bottom(),row.right_bottom()],Stroke::new(0.6,LINE));
                            let tint=Color32::from_rgb((color.r() as f32*0.16+255.*0.84) as u8,(color.g() as f32*0.16+255.*0.84) as u8,(color.b() as f32*0.16+255.*0.84) as u8);
                            for (j,tick) in keys.iter().enumerate() {
                                let end=keys.get(j+1).copied().unwrap_or(self.editor.doc.duration);
                                if end<a || *tick>b {continue;}
                                let x=r.left()+*tick as f32*cell;let right=r.left()+end as f32*cell;
                                let blank=if let Track::Layer(id)=track {self.editor.doc.layer(*id).ok().and_then(|l|l.keys.get(tick)).is_some_and(|f|f.figures.is_empty())}else{false};
                                if !blank {painter.rect_filled(Rect::from_min_max(Pos2::new(x+1.,y+6.),Pos2::new(right-1.,y+row_height-6.)),4.,tint);}
                                let c=Pos2::new(x+cell/2.,y+row_height/2.);
                                let diamond=vec![c+Vec2::new(0.,-3.5),c+Vec2::new(3.5,0.),c+Vec2::new(0.,3.5),c+Vec2::new(-3.5,0.)];
                                painter.add(egui::Shape::convex_polygon(diamond,if blank {Color32::WHITE}else{*color},Stroke::new(1.,*color)));
                            }
                        }
                        for tick in a..b {
                            let x=r.left()+tick as f32*cell;
                            let major=tick%5==0;
                            painter.line_segment([Pos2::new(x,r.top()+ruler-4.),Pos2::new(x,r.bottom())],Stroke::new(0.4,if major {LINE}else{Color32::from_rgba_unmultiplied(229,232,240,100)}));
                            if tick==0 || (tick+1)%5==0 {painter.text(Pos2::new(x+4.,r.top()+13.),egui::Align2::LEFT_CENTER,(tick+1).to_string(),FontId::proportional(10.),MUTED);}
                        }
                        painter.line_segment([Pos2::new(end_x,r.top()),Pos2::new(end_x,r.bottom())],Stroke::new(1.,LINE));
                        let x=r.left()+(self.editor.tick as f32+0.5)*cell;
                        painter.line_segment([Pos2::new(x,r.top()+ruler-4.),Pos2::new(x,r.bottom())],Stroke::new(1.5,BLUE));
                        let head=Rect::from_center_size(Pos2::new(x,r.top()+13.),Vec2::new(26.,22.));
                        painter.rect_filled(head,5.,BLUE);painter.text(head.center(),egui::Align2::CENTER_CENTER,(self.editor.tick+1).to_string(),FontId::proportional(11.),Color32::WHITE);
                        if response.clicked() || response.dragged() {
                            if let Some(p)=response.interact_pointer_pos() {
                                self.playing=false;self.editor.tick=(((p.x-r.left())/cell).floor().max(0.) as u32).min(self.editor.doc.duration-1);
                                if p.y>=r.top()+ruler {let i=((p.y-r.top()-ruler)/row_height) as usize;if let Some((track,_,_,_))=rows.get(i) {self.editor.track = *track;self.editor.selected=0;}}
                            }
                        }
                    });
                    if ui.rect_contains_pointer(grid.inner_rect) {self.timeline_scroll=grid.state.offset.y;}
                });
                ui.horizontal(|ui| {
                    ui.add_space(16.);ui.label(RichText::new(&self.status).size(10.).color(MUTED));
                    if let Some((cancel,_))=&self.export {if ui.small_button("Cancel export").clicked() {cancel.store(true,Ordering::Relaxed);}}
                    ui.with_layout(Layout::right_to_left(Align::Center),|ui| {
                        ui.label(RichText::new(if self.bridge.is_some() {"● MCP enabled"}else{"MCP off"}).size(10.).color(if self.bridge.is_some() {Color32::from_rgb(37,167,130)}else{MUTED})).on_hover_text("Local AI control through MCP. Enable the bridge to connect a client.");
                        ui.add_space(12.);ui.label(RichText::new(format!("{} layers  ·  Native preview",self.editor.doc.layers.len())).size(10.).color(MUTED));
                    });
                });
            });
    }

    pub(super) fn stage(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(237, 239, 245)))
            .show(ctx, |ui| {
                self.canvas(ui);
                let left = ui.max_rect().left() + 16.;
                let center = ui.max_rect().center().y;
                let r = Rect::from_min_size(Pos2::new(left, center - 82.), Vec2::new(46., 164.));
                ui.painter().rect_filled(
                    r.translate(Vec2::new(0., 2.)),
                    14.,
                    Color32::from_rgb(218, 222, 232),
                );
                ui.painter().rect_filled(r, 14., Color32::WHITE);
                for (i, (_text, tip)) in [
                    ("+", "Add figure"),
                    ("S", "Select / pose"),
                    ("B", "Build figure"),
                    ("H", "Pan canvas"),
                ]
                .iter()
                .enumerate()
                {
                    let selected = match i {
                        1 => !self.pan_tool && !self.build,
                        2 => self.build && !self.pan_tool,
                        3 => self.pan_tool,
                        _ => false,
                    };
                    let button =
                        egui::Button::new("").fill(if selected { BLUE } else { Color32::WHITE });
                    let button_rect = Rect::from_min_size(
                        r.min + Vec2::new(5., 6. + i as f32 * 38.),
                        Vec2::new(36., 34.),
                    );
                    if ui
                        .put(
                            Rect::from_min_size(
                                r.min + Vec2::new(5., 6. + i as f32 * 38.),
                                Vec2::new(36., 34.),
                            ),
                            button,
                        )
                        .on_hover_text(*tip)
                        .clicked()
                    {
                        match i {
                            0 => self.edit("add_figure", json!({})),
                            1 => {
                                self.pan_tool = false;
                                self.build = false;
                            }
                            2 => {
                                self.pan_tool = false;
                                self.build = true;
                            }
                            _ => self.pan_tool = true,
                        }
                    }
                    icon(
                        ui,
                        button_rect,
                        i,
                        if selected { Color32::WHITE } else { MUTED },
                    );
                }
            });
    }
}

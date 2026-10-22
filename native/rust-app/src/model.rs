use crate::tracks::{Camera, Layer, Scene, Track};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, f32::consts::PI, fs, io::Write, path::Path};

pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn distance(self, other: Self) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }
    pub fn offset(self, angle: f32, length: f32) -> Self {
        Self::new(self.x + angle.cos() * length, self.y + angle.sin() * length)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub length: f32,
    pub angle: f32,
    pub width: f32,
    pub circle: bool,
    pub ik: bool,
    pub bend: i8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Figure {
    pub id: u64,
    pub name: String,
    pub root: Point,
    pub color: [u8; 3],
    pub bones: Vec<Bone>,
}
impl Figure {
    pub fn blue(id: u64, x: f32, y: f32) -> Self {
        let names = [
            "Torso",
            "Head",
            "Left upper arm",
            "Left forearm",
            "Right upper arm",
            "Right forearm",
            "Left thigh",
            "Left lower leg",
            "Right thigh",
            "Right lower leg",
        ];
        let parents = [
            None,
            Some(0),
            Some(0),
            Some(2),
            Some(0),
            Some(4),
            None,
            Some(6),
            None,
            Some(8),
        ];
        let lengths = [78., 44., 53., 48., 53., 48., 66., 63., 66., 63.];
        let angles = [
            -PI / 2.,
            -PI / 2.,
            1.05,
            1.32,
            PI - 1.05,
            PI - 1.32,
            1.12,
            1.52,
            PI - 1.12,
            PI - 1.52,
        ];
        Self {
            id,
            name: if id == 1 {
                "Blue".into()
            } else {
                format!("Figure {id}")
            },
            root: Point::new(x, y),
            color: [13, 153, 255],
            bones: (0..10)
                .map(|i| Bone {
                    name: names[i].into(),
                    parent: parents[i],
                    length: lengths[i],
                    angle: angles[i],
                    width: if i == 0 {
                        16.
                    } else if i >= 6 {
                        14.
                    } else {
                        12.
                    },
                    circle: i == 1,
                    ik: [3, 5, 7, 9].contains(&i),
                    bend: if i >= 6 { -1 } else { 1 },
                })
                .collect(),
        }
    }
    pub fn points(&self) -> Vec<Point> {
        let mut points: Vec<Point> = Vec::with_capacity(self.bones.len());
        for bone in &self.bones {
            let start = bone.parent.map(|p| points[p]).unwrap_or(self.root);
            points.push(start.offset(bone.angle, bone.length));
        }
        points
    }
    pub fn pose(&mut self, index: usize, target: Point) -> Result<()> {
        if index >= self.bones.len() || !target.x.is_finite() || !target.y.is_finite() {
            return Err("Invalid joint target".into());
        }
        let points = self.points();
        if self.bones[index].ik {
            let upper = self.bones[index]
                .parent
                .ok_or("IK needs two connected bones")?;
            let origin = self.bones[upper]
                .parent
                .map(|i| points[i])
                .unwrap_or(self.root);
            let a = self.bones[upper].length;
            let b = self.bones[index].length;
            let dx = target.x - origin.x;
            let dy = target.y - origin.y;
            let distance = dx.hypot(dy).clamp((a - b).abs() + 0.0001, a + b - 0.0001);
            let direction = dy.atan2(dx);
            let bend = if self.bones[index].bend < 0 { -1. } else { 1. };
            let angle = direction
                + bend
                    * ((a * a + distance * distance - b * b) / (2. * a * distance))
                        .clamp(-1., 1.)
                        .acos();
            self.bones[upper].angle = angle;
            let elbow = origin.offset(angle, a);
            let end = origin.offset(direction, distance);
            self.bones[index].angle = (end.y - elbow.y).atan2(end.x - elbow.x);
        } else {
            let origin = self.bones[index]
                .parent
                .map(|i| points[i])
                .unwrap_or(self.root);
            let delta = shortest(
                (target.y - origin.y).atan2(target.x - origin.x) - self.bones[index].angle,
            );
            for i in index..self.bones.len() {
                let mut ancestor = Some(i);
                while let Some(node) = ancestor {
                    if node == index {
                        self.bones[i].angle += delta;
                        break;
                    }
                    ancestor = self.bones[node].parent;
                }
            }
        }
        Ok(())
    }
}
pub fn shortest(angle: f32) -> f32 {
    (angle + PI).rem_euclid(2. * PI) - PI
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub figures: Vec<Figure>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Interpolation {
    Hold,
    Linear,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub format: String,
    pub version: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration: u32,
    pub interpolation: Interpolation,
    pub layers: Vec<Layer>,
    pub camera_keys: BTreeMap<u32, Camera>,
    pub scene_keys: BTreeMap<u32, Scene>,
    #[serde(default)]
    pub camera_locked: bool,
    #[serde(default)]
    pub scene_locked: bool,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            format: "StykAnimateRustPreview".into(),
            version: 2,
            name: "Untitled animation".into(),
            width: 960,
            height: 540,
            fps: 24,
            duration: 10,
            interpolation: Interpolation::Linear,
            layers: vec![Layer {
                id: 1,
                name: "Layer 1".into(),
                visible: true,
                locked: false,
                keys: BTreeMap::from([(
                    0,
                    Frame {
                        figures: vec![Figure::blue(1, 480., 280.)],
                    },
                )]),
            }],
            camera_keys: BTreeMap::from([(
                0,
                Camera {
                    center: Point::new(480., 270.),
                    zoom: 1.,
                    angle: 0.,
                },
            )]),
            scene_keys: BTreeMap::from([(
                0,
                Scene {
                    background: [255, 255, 255],
                },
            )]),
            camera_locked: false,
            scene_locked: false,
        }
    }
}
impl Document {
    pub fn track_locked(&self, track: Track) -> bool {
        match track {
            Track::Camera => self.camera_locked,
            Track::Scene => self.scene_locked,
            Track::Layer(id) => self.layer(id).map(|l| l.locked).unwrap_or(true),
        }
    }
    pub fn owner(&self, uid: u64) -> Option<u64> {
        self.layers
            .iter()
            .find(|l| {
                l.keys
                    .values()
                    .any(|f| f.figures.iter().any(|f| f.id == uid))
            })
            .map(|l| l.id)
    }
    pub fn layer(&self, id: u64) -> Result<&Layer> {
        self.layers
            .iter()
            .find(|l| l.id == id)
            .ok_or("Unknown layer".into())
    }
    pub fn layer_mut(&mut self, id: u64) -> Result<&mut Layer> {
        self.layers
            .iter_mut()
            .find(|l| l.id == id)
            .ok_or("Unknown layer".into())
    }
    pub fn camera(&self, tick: f32) -> Camera {
        let tick = tick.clamp(0., self.duration.saturating_sub(1) as f32);
        let (&start, value) = self
            .camera_keys
            .range(..=tick.floor() as u32)
            .next_back()
            .unwrap();
        let mut camera = *value;
        if self.interpolation == Interpolation::Linear {
            if let Some((&end, next)) = self.camera_keys.range(start + 1..).next() {
                let t = (tick - start as f32) / (end - start) as f32;
                camera.center.x += (next.center.x - camera.center.x) * t;
                camera.center.y += (next.center.y - camera.center.y) * t;
                camera.zoom += (next.zoom - camera.zoom) * t;
                camera.angle += shortest(next.angle - camera.angle) * t;
            }
        }
        camera
    }
    pub fn scene(&self, tick: f32) -> Scene {
        *self
            .scene_keys
            .range(..=tick.max(0.).floor() as u32)
            .next_back()
            .unwrap()
            .1
    }
    pub fn rendered(&self, tick: f32) -> Frame {
        let camera = self.camera(tick);
        let mut frame = self.sample(tick);
        for f in &mut frame.figures {
            f.root = camera.project(f.root, self.width, self.height);
            for b in &mut f.bones {
                b.angle -= camera.angle;
                b.length *= camera.zoom;
                b.width *= camera.zoom;
            }
        }
        frame
    }
    pub fn insert_track_key(&mut self, track: Track, tick: u32, blank: bool) -> Result<()> {
        if self.track_locked(track) {
            return Err("Track is locked".into());
        }
        if tick >= 10000 {
            return Err("tick must be less than 10000".into());
        }
        match track {
            Track::Layer(id) => {
                let frame = self.layer_key(id, tick)?;
                if blank {
                    frame.figures.clear();
                }
            }
            Track::Camera => {
                let camera = self.camera(tick as f32);
                self.camera_keys.insert(tick, camera);
            }
            Track::Scene => {
                let scene = self.scene(tick as f32);
                self.scene_keys.insert(tick, scene);
            }
        }
        self.duration = self.duration.max(tick + 1);
        Ok(())
    }
    pub fn delete_track_key(&mut self, track: Track, tick: u32) -> Result<()> {
        if self.track_locked(track) {
            return Err("Track is locked".into());
        }
        if tick == 0 {
            return Err("The opening key must be retained".into());
        }
        match track {
            Track::Layer(id) => {
                let layer = self.layer_mut(id)?;
                if layer.locked {
                    return Err("Layer is locked".into());
                }
                layer.keys.remove(&tick);
            }
            Track::Camera => {
                self.camera_keys.remove(&tick);
            }
            Track::Scene => {
                self.scene_keys.remove(&tick);
            }
        }
        Ok(())
    }
    pub fn resize_duration(&mut self, duration: u32) -> Result<()> {
        if !(1..=10000).contains(&duration) {
            return Err("Invalid duration".into());
        }
        self.duration = duration;
        self.camera_keys.retain(|t, _| *t < duration);
        self.scene_keys.retain(|t, _| *t < duration);
        for layer in &mut self.layers {
            layer.keys.retain(|t, _| *t < duration);
        }
        Ok(())
    }
    pub fn move_to_layer(&mut self, uid: u64, target: u64) -> Result<()> {
        let source = self.owner(uid).ok_or("Unknown figure")?;
        if source == target {
            return Ok(());
        }
        if self.layer(source)?.locked || self.layer(target)?.locked {
            return Err("Layer is locked".into());
        }
        let ticks: std::collections::BTreeSet<u32> = self
            .layer(source)?
            .keys
            .keys()
            .chain(self.layer(target)?.keys.keys())
            .copied()
            .collect();
        let mut replacement = BTreeMap::new();
        for tick in ticks {
            let mut frame = self.sample_layer(target, tick as f32);
            if let Some(figure) = self
                .sample_layer(source, tick as f32)
                .figures
                .into_iter()
                .find(|f| f.id == uid)
            {
                frame.figures.push(figure);
            }
            replacement.insert(tick, frame);
        }
        self.layer_mut(target)?.keys = replacement;
        for frame in self.layer_mut(source)?.keys.values_mut() {
            frame.figures.retain(|f| f.id != uid);
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        if self.format != "StykAnimateRustPreview" || self.version != 2 {
            return Err("This build opens only Rust preview projects; original .styk/.piv/.nodes files are not yet supported".into());
        }
        if !(64..=4096).contains(&self.width)
            || !(64..=4096).contains(&self.height)
            || !(1..=240).contains(&self.fps)
            || !(1..=10000).contains(&self.duration)
            || self.layers.is_empty()
            || self.layers.len() > 128
            || !self.camera_keys.contains_key(&0)
            || !self.scene_keys.contains_key(&0)
            || self
                .camera_keys
                .keys()
                .chain(self.scene_keys.keys())
                .any(|t| *t >= self.duration)
            || self.name.len() > 1024
        {
            return Err("Invalid project dimensions, timing or keys".into());
        }
        let mut count = 0;
        for camera in self.camera_keys.values() {
            if !camera.center.x.is_finite()
                || !camera.center.y.is_finite()
                || camera.center.x.abs() > 1e6
                || camera.center.y.abs() > 1e6
                || !camera.zoom.is_finite()
                || !(0.05..=20.).contains(&camera.zoom)
                || !camera.angle.is_finite()
                || camera.angle.abs() > 10000.
            {
                return Err("Invalid camera transform".into());
            }
        }
        let mut layer_ids = std::collections::HashSet::new();
        let mut owners = std::collections::HashMap::new();
        for layer in &self.layers {
            if layer.id == 0
                || layer.id >= u64::MAX - 1
                || !layer_ids.insert(layer.id)
                || layer.name.len() > 1024
                || !layer.keys.contains_key(&0)
            {
                return Err("Invalid layer identity or opening key".into());
            }
            for (tick, frame) in &layer.keys {
                if *tick >= self.duration || frame.figures.len() > 1000 {
                    return Err("Invalid keyframe range or figure count".into());
                }
                let mut ids = std::collections::HashSet::new();
                for figure in &frame.figures {
                    if owners
                        .insert(figure.id, layer.id)
                        .is_some_and(|id| id != layer.id)
                    {
                        return Err("A figure cannot belong to multiple layers".into());
                    }
                    if figure.id == 0
                        || figure.id >= u64::MAX - 1
                        || !ids.insert(figure.id)
                        || figure.name.len() > 1024
                        || !figure.root.x.is_finite()
                        || !figure.root.y.is_finite()
                        || figure.root.x.abs() > 1e6
                        || figure.root.y.abs() > 1e6
                        || figure.bones.is_empty()
                        || figure.bones.len() > 1000
                    {
                        return Err("Invalid figure or identity".into());
                    }
                    for (i, bone) in figure.bones.iter().enumerate() {
                        count += 1;
                        if count > 1_000_000
                            || bone.parent.is_some_and(|p| p >= i)
                            || !bone.length.is_finite()
                            || !(0.001..=10000.).contains(&bone.length)
                            || !bone.angle.is_finite()
                            || bone.angle.abs() > 10000.
                            || !bone.width.is_finite()
                            || !(0. ..=1000.).contains(&bone.width)
                            || bone.name.len() > 1024
                            || ![-1, 0, 1].contains(&bone.bend)
                            || (bone.ik && bone.parent.is_none())
                        {
                            return Err("Invalid bone topology, size or pose".into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn sample(&self, tick: f32) -> Frame {
        Frame {
            figures: self
                .layers
                .iter()
                .filter(|l| l.visible)
                .flat_map(|l| self.sample_layer(l.id, tick).figures)
                .collect(),
        }
    }
    pub fn sample_layer(&self, id: u64, tick: f32) -> Frame {
        let Some(layer) = self.layers.iter().find(|l| l.id == id) else {
            return Frame { figures: vec![] };
        };
        let keys = &layer.keys;
        let tick = tick.clamp(0., self.duration.saturating_sub(1) as f32);
        let (&start, from) = keys.range(..=tick.floor() as u32).next_back().unwrap();
        let next = keys.range(start + 1..).next();
        if self.interpolation == Interpolation::Hold || next.is_none() {
            return from.clone();
        }
        let (&end, to) = next.unwrap();
        let t = (tick - start as f32) / (end - start) as f32;
        let mut frame = from.clone();
        for figure in &mut frame.figures {
            let Some(other) = to.figures.iter().find(|other| other.id == figure.id) else {
                continue;
            };
            if figure.bones.len() != other.bones.len()
                || figure
                    .bones
                    .iter()
                    .zip(&other.bones)
                    .any(|(a, b)| a.parent != b.parent || a.circle != b.circle)
            {
                continue;
            }
            figure.root.x += (other.root.x - figure.root.x) * t;
            figure.root.y += (other.root.y - figure.root.y) * t;
            for (bone, other) in figure.bones.iter_mut().zip(&other.bones) {
                bone.angle += shortest(other.angle - bone.angle) * t;
                bone.length += (other.length - bone.length) * t;
            }
        }
        frame
    }
    pub fn layer_key(&mut self, id: u64, tick: u32) -> Result<&mut Frame> {
        let index = self
            .layers
            .iter()
            .position(|l| l.id == id)
            .ok_or("Unknown layer")?;
        if self.layers[index].locked {
            return Err("Layer is locked".into());
        }
        // Sampling borrows the complete document, so it cannot run inside an
        // entry closure that already holds a mutable borrow of the key map.
        #[allow(clippy::map_entry)]
        if !self.layers[index].keys.contains_key(&tick) {
            let sample = self.sample_layer(id, tick as f32);
            self.layers[index].keys.insert(tick, sample);
        }
        self.duration = self.duration.max(tick + 1);
        Ok(self.layers[index].keys.get_mut(&tick).unwrap())
    }
    pub fn figure_mut(&mut self, tick: u32, id: u64) -> Result<&mut Figure> {
        let layer = self.owner(id).ok_or("Unknown figure")?;
        self.layer_key(layer, tick)?
            .figures
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or("Unknown figure".into())
    }
    pub fn next_id(&self) -> u64 {
        self.layers
            .iter()
            .flat_map(|l| l.keys.values())
            .flat_map(|f| f.figures.iter().map(|f| f.id))
            .max()
            .unwrap_or(0)
            + 1
    }
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.camera_keys.len() * (std::mem::size_of::<Camera>() + 64)
            + self.scene_keys.len() * (std::mem::size_of::<Scene>() + 64)
            + self.layers.capacity() * std::mem::size_of::<Layer>()
            + self.layers.iter().map(|l| l.name.capacity()).sum::<usize>()
            + self.name.capacity()
            + self.format.capacity()
            + self
                .layers
                .iter()
                .flat_map(|l| l.keys.values())
                .map(|frame| {
                    80 + frame.figures.capacity() * std::mem::size_of::<Figure>()
                        + frame
                            .figures
                            .iter()
                            .map(|f| {
                                f.name.capacity()
                                    + f.bones.capacity() * std::mem::size_of::<Bone>()
                                    + f.bones.iter().map(|b| b.name.capacity()).sum::<usize>()
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if !path.to_string_lossy().ends_with(".rustanim.json") {
            return Err("Use the .rustanim.json extension; this preview cannot overwrite original StykAnimate files".into());
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(temp.as_file_mut(), self).map_err(|e| e.to_string())?;
        temp.flush().map_err(|e| e.to_string())?;
        if temp.as_file().metadata().map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
            return Err("Project exceeds the preview's 16 MiB limit".into());
        }
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        temp.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn open(path: &Path) -> Result<Self> {
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
            return Err("Project exceeds the preview's 16 MiB limit".into());
        }
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        let mut value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Unsupported or invalid preview project: {e}"))?;
        if value["format"] == "StykAnimateRustPreview" && value["version"] == 1 {
            let object = value.as_object_mut().ok_or("Invalid document")?;
            let keys = object.remove("keys").ok_or("Missing legacy keys")?;
            let width = object
                .get("width")
                .and_then(|v| v.as_u64())
                .ok_or("Invalid width")? as f32;
            let height = object
                .get("height")
                .and_then(|v| v.as_u64())
                .ok_or("Invalid height")? as f32;
            let bg = object
                .remove("background")
                .ok_or("Missing legacy background")?;
            object.insert("version".into(), serde_json::json!(2));
            object.insert("layers".into(), serde_json::json!([{ "id":1, "name":"Layer 1", "visible":true, "locked":false, "keys":keys }]));
            object.insert("camera_keys".into(), serde_json::json!({"0":{"center":{"x":width/2.,"y":height/2.},"zoom":1.,"angle":0.}}));
            object.insert(
                "scene_keys".into(),
                serde_json::json!({"0":{"background":bg}}),
            );
        }
        let document: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
        document.validate()?;
        Ok(document)
    }
    pub fn walk() -> Self {
        let mut doc = Self {
            name: "Blue — walking".into(),
            duration: 48,
            ..Self::default()
        };
        doc.layers[0].keys.clear();
        for tick in 0..48 {
            let phase = (tick % 24) as f32 / 24.;
            let angle = phase * PI * 2.;
            let x = 320. + tick as f32 * 80. / 12.;
            let y = 282. + 3. * (angle * 2.).cos();
            let mut figure = Figure::blue(1, x, y);
            for (foot, hand, offset) in [(7, 3, 0.), (9, 5, 0.5)] {
                let p = (phase + offset) % 1.;
                let swing = p < 0.5;
                let u = if swing { p * 2. } else { (p - 0.5) * 2. };
                let dx = if swing {
                    -40. + 80. * u * u * (3. - 2. * u)
                } else {
                    40. - 80. * u
                };
                figure
                    .pose(
                        foot,
                        Point::new(x + dx, 402. - if swing { 30. * (PI * u).sin() } else { 0. }),
                    )
                    .unwrap();
                figure
                    .pose(
                        hand,
                        Point::new(
                            x - 32. * (p * PI * 2.).cos(),
                            y + 13. - 5. * angle.sin().abs(),
                        ),
                    )
                    .unwrap();
            }
            doc.layers[0].keys.insert(
                tick,
                Frame {
                    figures: vec![figure],
                },
            );
        }
        doc
    }
}
#[derive(Clone)]
struct Change {
    before: Document,
    after: Document,
    bytes: usize,
}
pub struct Editor {
    pub doc: Document,
    pub tick: u32,
    pub selected: u64,
    pub track: Track,
    pub dirty: bool,
    undo: Vec<Change>,
    redo: Vec<Change>,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            doc: Document::default(),
            tick: 0,
            selected: 1,
            track: Track::Layer(1),
            dirty: false,
            undo: vec![],
            redo: vec![],
        }
    }
}
impl Editor {
    pub fn change(&mut self, edit: impl FnOnce(&mut Document) -> Result<()>) -> Result<()> {
        let before = self.doc.clone();
        let result = edit(&mut self.doc).and_then(|_| self.doc.validate());
        if let Err(error) = result {
            self.doc = before;
            return Err(error);
        }
        self.record(before);
        Ok(())
    }
    pub fn record(&mut self, before: Document) {
        if before == self.doc {
            return;
        }
        let bytes = before.memory_bytes() + self.doc.memory_bytes();
        self.redo.clear();
        self.undo.push(Change {
            before,
            after: self.doc.clone(),
            bytes,
        });
        while self.undo.len() > 100
            || self.undo.iter().map(|c| c.bytes).sum::<usize>() > 64 * 1024 * 1024
        {
            self.undo.remove(0);
        }
        self.dirty = true;
    }
    pub fn undo(&mut self) -> bool {
        if let Some(change) = self.undo.pop() {
            self.doc = change.before.clone();
            self.redo.push(change);
            self.tick = self.tick.min(self.doc.duration - 1);
            self.dirty = true;
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        if let Some(change) = self.redo.pop() {
            self.doc = change.after.clone();
            self.undo.push(change);
            self.dirty = true;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ik_preserves_lengths_and_reaches_target() {
        let mut f = Figure::blue(1, 480., 280.);
        let target = Point::new(490., 370.);
        f.pose(7, target).unwrap();
        let points = f.points();
        assert!(points[7].distance(target) < 0.001);
        assert!((points[6].distance(f.root) - 66.).abs() < 0.001);
        assert!((points[7].distance(points[6]) - 63.).abs() < 0.001);
    }
    #[test]
    fn failed_edits_are_atomic_and_history_restores_pose() {
        let mut e = Editor::default();
        let original = e.doc.clone();
        assert!(e
            .change(|d| {
                d.fps = 0;
                Ok(())
            })
            .is_err());
        assert_eq!(e.doc, original);
        e.change(|d| {
            d.figure_mut(0, 1)?.root.x += 20.;
            Ok(())
        })
        .unwrap();
        assert!(e.undo());
        assert_eq!(e.doc, original);
        assert!(e.redo());
        assert_eq!(e.doc.sample(0.).figures[0].root.x, 500.);
    }
    #[test]
    fn interpolation_uses_short_rotation_and_never_introduces_future_figures() {
        let mut d = Document {
            duration: 11,
            ..Document::default()
        };
        d.layers[0].keys.get_mut(&0).unwrap().figures[0].bones[0].angle = PI - 0.1;
        let mut next = d.layers[0].keys[&0].clone();
        next.figures[0].bones[0].angle = -PI + 0.1;
        next.figures.push(Figure::blue(2, 20., 20.));
        d.layers[0].keys.insert(10, next);
        assert_eq!(d.sample(5.).figures.len(), 1);
        assert!((d.sample(5.).figures[0].bones[0].angle - PI).abs() < 0.001);
    }
    #[test]
    fn save_roundtrip_and_invalid_topology_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("walk.rustanim.json");
        let doc = Document::walk();
        doc.save(&file).unwrap();
        assert_eq!(Document::open(&file).unwrap(), doc);
        assert!(doc.save(&dir.path().join("original.styk")).is_err());
        let mut invalid = doc.clone();
        invalid.layers[0].keys.get_mut(&0).unwrap().figures[0].bones[0].parent = Some(1);
        assert!(invalid.validate().is_err());
    }
    #[test]
    fn layer_keys_camera_and_scene_are_independent() {
        let mut d = Document::default();
        d.layers.push(Layer {
            id: 2,
            name: "Opponent".into(),
            visible: true,
            locked: false,
            keys: BTreeMap::from([(
                0,
                Frame {
                    figures: vec![Figure::blue(2, 700., 280.)],
                },
            )]),
        });
        d.insert_track_key(Track::Layer(2), 8, false).unwrap();
        d.figure_mut(8, 2).unwrap().root.x = 780.;
        assert_eq!(d.sample_layer(1, 4.).figures[0].root.x, 480.);
        assert_eq!(d.sample_layer(2, 4.).figures[0].root.x, 740.);
        assert_eq!(d.layers[0].keys.len(), 1);
        let world = d.sample(4.);
        let mut camera = d.camera(0.);
        camera.center.x += 80.;
        camera.zoom = 2.;
        d.camera_keys.insert(8, camera);
        d.scene_keys.insert(
            5,
            Scene {
                background: [30, 40, 50],
            },
        );
        assert_eq!(d.sample(4.), world);
        assert_eq!(d.scene(4.).background, [255; 3]);
        assert_eq!(d.scene(5.).background, [30, 40, 50]);
        assert!((d.camera(4.).zoom - 1.5).abs() < 0.001);
        let p = Point::new(300., 100.);
        let c = d.camera(4.);
        assert!(
            p.distance(c.unproject(c.project(p, d.width, d.height), d.width, d.height)) < 0.001
        );
        d.insert_track_key(Track::Layer(2), 6, true).unwrap();
        assert_eq!(d.sample(7.).figures.len(), 1);
        assert_eq!(d.sample(8.).figures.len(), 2);
        d.layers[1].visible = false;
        assert_eq!(d.sample(8.).figures.len(), 1);
        assert!(d.validate().is_ok());
    }
    #[test]
    fn layer_lock_move_and_roundtrip_preserve_animation() {
        let mut d = Document::walk();
        d.layers.push(Layer {
            id: 2,
            name: "Destination".into(),
            visible: true,
            locked: false,
            keys: BTreeMap::from([(0, Frame { figures: vec![] })]),
        });
        let before = d.sample(13.5);
        d.move_to_layer(1, 2).unwrap();
        assert_eq!(d.sample(13.5), before);
        assert!(d.layers[0].keys.values().all(|f| f.figures.is_empty()));
        d.layers[1].locked = true;
        assert!(d.figure_mut(5, 1).is_err());
        assert!(d.insert_track_key(Track::Layer(2), 5, true).is_err());
        assert!(d.delete_track_key(Track::Layer(2), 5).is_err());
        assert!(d.delete_track_key(Track::Camera, 0).is_err());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("layers.rustanim.json");
        d.save(&path).unwrap();
        assert_eq!(Document::open(&path).unwrap(), d);
    }
    #[test]
    fn old_preview_files_migrate_without_changing_poses() {
        let doc = Document::walk();
        let mut legacy = serde_json::to_value(&doc).unwrap();
        let object = legacy.as_object_mut().unwrap();
        object.insert("version".into(), serde_json::json!(1));
        object.insert("background".into(), serde_json::json!([255, 255, 255]));
        object.insert(
            "keys".into(),
            serde_json::to_value(&doc.layers[0].keys).unwrap(),
        );
        for field in [
            "layers",
            "camera_keys",
            "scene_keys",
            "camera_locked",
            "scene_locked",
        ] {
            object.remove(field);
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old.rustanim.json");
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        let migrated = Document::open(&path).unwrap();
        assert_eq!(migrated, doc);
        assert_eq!(migrated.rendered(12.5), doc.rendered(12.5));
        legacy.as_object_mut().unwrap().remove("width");
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert!(Document::open(&path).is_err());
    }
}

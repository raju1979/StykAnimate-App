use crate::model::{Frame, Point};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub id: u64,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub keys: BTreeMap<u32, Frame>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    pub center: Point,
    pub zoom: f32,
    pub angle: f32,
}
impl Camera {
    pub fn project(self, p: Point, width: u32, height: u32) -> Point {
        let dx = p.x - self.center.x;
        let dy = p.y - self.center.y;
        let (s, c) = self.angle.sin_cos();
        Point::new(
            width as f32 / 2. + (dx * c + dy * s) * self.zoom,
            height as f32 / 2. + (-dx * s + dy * c) * self.zoom,
        )
    }
    pub fn unproject(self, p: Point, width: u32, height: u32) -> Point {
        let dx = (p.x - width as f32 / 2.) / self.zoom;
        let dy = (p.y - height as f32 / 2.) / self.zoom;
        let (s, c) = self.angle.sin_cos();
        Point::new(
            self.center.x + dx * c - dy * s,
            self.center.y + dx * s + dy * c,
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub background: [u8; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    Camera,
    Scene,
    Layer(u64),
}

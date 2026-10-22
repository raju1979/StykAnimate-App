use crate::{
    model::{Bone, Document, Figure, Frame, Point},
    tracks::{Camera, Layer, Scene},
};
use std::collections::BTreeMap;

/// An ordinary editable project, built with the same records as user scenes.
pub fn showcase() -> Document {
    let walk = Document::walk();
    let mut azure = BTreeMap::new();
    let mut coral = BTreeMap::new();
    for tick in (0..72).step_by(4).chain(std::iter::once(71)) {
        let mut a = walk.layers[0].keys[&(tick % 24)].figures[0].clone();
        a.name = "Azure".into();
        let sway = 22. * (tick as f32 / 72. * std::f32::consts::TAU).sin();
        a.root.x = 325. + sway;
        a.color = [25, 142, 246];
        let mut b = walk.layers[0].keys[&((tick + 6) % 24)].figures[0].clone();
        b.id = 2;
        b.name = "Coral".into();
        b.root.x = 635. - sway;
        for bone in &mut b.bones {
            bone.angle = std::f32::consts::PI - bone.angle;
            bone.bend *= -1;
        }
        b.color = [242, 115, 93];
        azure.insert(tick, Frame { figures: vec![a] });
        coral.insert(tick, Frame { figures: vec![b] });
    }
    let ground = Figure {
        id: 3,
        name: "Ground".into(),
        root: Point::new(115., 402.),
        color: [219, 226, 236],
        bones: vec![Bone {
            name: "Baseline".into(),
            parent: None,
            length: 730.,
            angle: 0.,
            width: 2.,
            circle: false,
            ik: false,
            bend: 1,
        }],
    };
    Document {
        name: "Motion study".into(),
        duration: 72,
        layers: vec![
            Layer {
                id: 3,
                name: "Stage".into(),
                visible: true,
                locked: true,
                keys: BTreeMap::from([(
                    0,
                    Frame {
                        figures: vec![ground],
                    },
                )]),
            },
            Layer {
                id: 1,
                name: "Azure".into(),
                visible: true,
                locked: false,
                keys: azure,
            },
            Layer {
                id: 2,
                name: "Coral".into(),
                visible: true,
                locked: false,
                keys: coral,
            },
        ],
        camera_keys: BTreeMap::from([
            (
                0,
                Camera {
                    center: Point::new(480., 270.),
                    zoom: 1.,
                    angle: 0.,
                },
            ),
            (
                36,
                Camera {
                    center: Point::new(486., 272.),
                    zoom: 1.035,
                    angle: 0.,
                },
            ),
            (
                71,
                Camera {
                    center: Point::new(480., 270.),
                    zoom: 1.,
                    angle: 0.,
                },
            ),
        ]),
        scene_keys: BTreeMap::from([(
            0,
            Scene {
                background: [250, 252, 255],
            },
        )]),
        ..Document::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn showcase_is_an_editable_project_and_survives_save() {
        let mut doc = showcase();
        doc.validate().unwrap();
        assert_eq!(doc.sample(24.).figures.len(), 3);
        assert!(doc.figure_mut(24, 3).is_err());
        doc.figure_mut(24, 1).unwrap().root.x += 10.;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("showcase.rustanim.json");
        doc.save(&path).unwrap();
        assert_eq!(Document::open(&path).unwrap(), doc);
        assert_ne!(doc.sample(0.), doc.sample(12.));
    }
}

use crate::model::{Document, Figure, Point, Result};
use image::{Rgba, RgbaImage};
use std::{
    io::{Cursor, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

fn disk(image: &mut RgbaImage, center: Point, radius: f32, color: [u8; 3]) {
    let x0 = (center.x - radius).floor().max(0.) as u32;
    let x1 = (center.x + radius).ceil().max(0.).min(image.width() as f32) as u32;
    let y0 = (center.y - radius).floor().max(0.) as u32;
    let y1 = (center.y + radius)
        .ceil()
        .max(0.)
        .min(image.height() as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            if (x as f32 + 0.5 - center.x).hypot(y as f32 + 0.5 - center.y) <= radius {
                image.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
            }
        }
    }
}
fn line(image: &mut RgbaImage, a: Point, b: Point, width: f32, color: [u8; 3]) {
    let radius = width / 2.;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let squared = dx * dx + dy * dy;
    if squared < 0.0001 {
        disk(image, a, radius, color);
        return;
    }
    let x0 = (a.x.min(b.x) - radius).floor().max(0.) as u32;
    let x1 = (a.x.max(b.x) + radius)
        .ceil()
        .max(0.)
        .min(image.width() as f32) as u32;
    let y0 = (a.y.min(b.y) - radius).floor().max(0.) as u32;
    let y1 = (a.y.max(b.y) + radius)
        .ceil()
        .max(0.)
        .min(image.height() as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let t = (((px - a.x) * dx + (py - a.y) * dy) / squared).clamp(0., 1.);
            if (px - a.x - t * dx).hypot(py - a.y - t * dy) <= radius {
                image.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
            }
        }
    }
}
pub fn raster(document: &Document, tick: f32) -> RgbaImage {
    let c = document.background;
    let mut image = RgbaImage::from_pixel(
        document.width,
        document.height,
        Rgba([c[0], c[1], c[2], 255]),
    );
    for figure in document.sample(tick).figures {
        let points = figure.points();
        for (i, bone) in figure.bones.iter().enumerate() {
            let start = bone.parent.map(|i| points[i]).unwrap_or(figure.root);
            let end = points[i];
            if bone.circle {
                disk(
                    &mut image,
                    Point::new((start.x + end.x) / 2., (start.y + end.y) / 2.),
                    bone.length / 2.,
                    figure.color,
                );
            } else if bone.width > 0. {
                line(&mut image, start, end, bone.width, figure.color);
            }
        }
    }
    image
}
pub fn png(document: &Document, tick: f32) -> Result<Vec<u8>> {
    let mut output = Cursor::new(Vec::new());
    raster(document, tick)
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(output.into_inner())
}
pub fn atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn svg(document: &Document, tick: f32) -> String {
    let bg = document.background;
    let mut output=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\"><rect width=\"100%\" height=\"100%\" fill=\"rgb({},{},{})\"/>",document.width,document.height,document.width,document.height,bg[0],bg[1],bg[2]);
    for Figure {
        root, color, bones, ..
    } in document.sample(tick).figures
    {
        let figure = Figure {
            id: 1,
            name: String::new(),
            root,
            color,
            bones,
        };
        let points = figure.points();
        for (i, bone) in figure.bones.iter().enumerate() {
            let a = bone.parent.map(|p| points[p]).unwrap_or(root);
            let b = points[i];
            if bone.circle {
                output.push_str(&format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"rgb({},{},{})\"/>",
                    (a.x + b.x) / 2.,
                    (a.y + b.y) / 2.,
                    bone.length / 2.,
                    color[0],
                    color[1],
                    color[2]
                ));
            } else {
                output.push_str(&format!("<path d=\"M {} {} L {} {}\" stroke=\"rgb({},{},{})\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",a.x,a.y,b.x,b.y,color[0],color[1],color[2],bone.width));
            }
        }
    }
    output.push_str("</svg>");
    output
}
pub fn gif(document: &Document, path: &Path, cancel: &AtomicBool) -> Result<()> {
    document.validate()?;
    if document.width as u64 * document.height as u64 * document.duration as u64 > 512_000_000 {
        return Err(
            "This preview limits GIF exports to 512 million pixels across all frames".into(),
        );
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut output = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    {
        let mut encoder = gif::Encoder::new(
            output.as_file_mut(),
            document.width as u16,
            document.height as u16,
            &[],
        )
        .map_err(|e| e.to_string())?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| e.to_string())?;
        for tick in 0..document.duration {
            if cancel.load(Ordering::Relaxed) {
                return Err("Export cancelled".into());
            }
            let mut pixels = raster(document, tick as f32).into_raw();
            let mut frame = gif::Frame::from_rgba_speed(
                document.width as u16,
                document.height as u16,
                &mut pixels,
                10,
            );
            let end = ((tick + 1) as f64 * 100. / document.fps as f64).round() as u32;
            let start = (tick as f64 * 100. / document.fps as f64).round() as u32;
            frame.delay = (end - start).max(1) as u16;
            encoder.write_frame(&frame).map_err(|e| e.to_string())?;
        }
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Export cancelled".into());
    }
    output.as_file().sync_all().map_err(|e| e.to_string())?;
    output.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exports_have_real_headers_and_cancel_preserves_existing_file() {
        let d = Document::default();
        assert_eq!(&png(&d, 0.).unwrap()[..8], b"\x89PNG\r\n\x1a\n");
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.gif");
        std::fs::write(&path, b"existing").unwrap();
        assert!(gif(&d, &path, &AtomicBool::new(true)).is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"existing");
        assert!(svg(&d, 0.).contains("<circle"));
    }
}

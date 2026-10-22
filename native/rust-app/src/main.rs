mod api;
mod demos;
mod mcp;
mod model;
mod render;
mod tracks;
mod ui;

fn main() -> eframe::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|a| a == "--mcp") {
        if let Err(error) = mcp::stdio() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let capture = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from);
    let mut doc = if args.iter().any(|a| a == "--showcase") {
        demos::showcase()
    } else if args.iter().any(|a| a == "--walk") {
        model::Document::walk()
    } else {
        model::Document::default()
    };
    if let Some(file) = args
        .iter()
        .position(|a| a == "--open")
        .and_then(|i| args.get(i + 1))
    {
        match model::Document::open(std::path::Path::new(file)) {
            Ok(document) => doc = document,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    }
    // Share the same document and export path as the GUI for reproducible demos.
    for (flag, gif) in [("--save-project", false), ("--export-gif", true)] {
        if let Some(path) = args
            .iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
        {
            let result = if gif {
                render::gif(
                    &doc,
                    std::path::Path::new(path),
                    &std::sync::atomic::AtomicBool::new(false),
                )
            } else {
                doc.save(std::path::Path::new(path))
            };
            if let Err(error) = result {
                eprintln!("{error}");
                std::process::exit(2);
            }
            println!("Saved {path}");
            return Ok(());
        }
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1440., 900.])
            .with_min_inner_size([900., 600.]),
        ..Default::default()
    };
    let track = args
        .iter()
        .position(|a| a == "--track")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let tick = args
        .iter()
        .position(|a| a == "--tick")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let preview_unsaved = args.iter().any(|a| a == "--preview-unsaved");
    eframe::run_native(
        "StykAnimate Rust Preview",
        options,
        Box::new(move |context| {
            Ok(Box::new(
                ui::App::new(context, doc, capture)
                    .with_track(track)
                    .at_tick(tick)
                    .preview_unsaved(preview_unsaved),
            ))
        }),
    )
}

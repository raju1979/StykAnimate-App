mod api;
mod mcp;
mod model;
mod render;
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
    let mut doc = if args.iter().any(|a| a == "--walk") {
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
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280., 800.])
            .with_min_inner_size([900., 600.]),
        ..Default::default()
    };
    eframe::run_native(
        "StykAnimate Rust Preview",
        options,
        Box::new(move |context| Ok(Box::new(ui::App::new(context, doc, capture)))),
    )
}

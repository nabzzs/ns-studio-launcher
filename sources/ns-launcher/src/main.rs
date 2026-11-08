#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() -> eframe::Result {
    let pixels = image::load_from_memory(include_bytes!("../../../branding/ns-studio-logo.png"))
        .ok()
        .map(|i| i.to_rgba8());
    let mut viewport = egui::ViewportBuilder::default()
        .with_transparent(true)
        .with_title("NS Studio")
        .with_app_id("studio.ns.launcher")
        .with_inner_size([1180., 860.])
        .with_min_inner_size([840., 650.]);
    if let Some(p) = pixels {
        viewport = viewport.with_icon(egui::IconData {
            width: p.width(),
            height: p.height(),
            rgba: p.into_raw(),
        });
    }
    eframe::run_native(
        "NS Studio",
        eframe::NativeOptions {
            viewport,
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(ns_launcher::Launcher::new(&cc.egui_ctx)))),
    )
}

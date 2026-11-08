use egui_kittest::kittest::Queryable;
fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("Output path required")?;
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1180., 860.))
        .with_pixels_per_point(1.)
        .wgpu()
        .build_eframe(|cc| ns_launcher::Launcher::new(&cc.egui_ctx));
    h.run_steps(4);
    if std::env::args().any(|a| a == "--appearance") {
        h.get_by_label("Themes").click();
        h.run_steps(20);
        let _ = h.get_by_label("G");
        let _ = h.get_by_label("W");
    }
    h.render()?.save(path).map_err(|e| e.to_string())?;
    Ok(())
}

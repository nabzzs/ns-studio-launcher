#![forbid(unsafe_code)]
use egui::{Color32, RichText, Stroke, vec2};
use std::path::PathBuf;

#[derive(Clone, Copy)]
struct Product {
    id: &'static str,
    name: &'static str,
    role: &'static str,
    group: &'static str,
    icon: &'static [u8],
    guide: &'static str,
}
const PRODUCTS: &[Product] = &[
    Product {
        id: "photocraft",
        name: "Pixavyn NS",
        role: "Create and refine images",
        group: "Photo",
        icon: include_bytes!("../../../branding/photo.png"),
        guide: "1. Create a document or open an image/PSD.\n2. Build your composition with layers.\n3. Use selections and masks to isolate edits.\n4. Adjust color and add text.\n5. Save the editable project; export PNG or JPEG for sharing.\n\nStart with a copy of an important image. PSD fidelity is still under development.",
    },
    Product {
        id: "vectorcraft",
        name: "Vectoryn NS",
        role: "Draw scalable illustrations",
        group: "Design",
        icon: include_bytes!("../../../branding/vector.png"),
        guide: "1. Create an artboard.\n2. Draw shapes or paths with the pen tool.\n3. Refine anchors, fills and strokes.\n4. Organize groups and layers.\n5. Save the editable project and export SVG/PDF.\n\nTest imported AI/EPS artwork before depending on its appearance.",
    },
    Product {
        id: "effectcraft",
        name: "Kinetryn NS",
        role: "Animate titles and visual effects",
        group: "Video",
        icon: include_bytes!("../../../branding/motion.png"),
        guide: "1. Create a composition with size, frame rate and duration.\n2. Add footage, text or shape layers.\n3. Animate position, scale and opacity with keyframes.\n4. Add effects and preview the result.\n5. Save the project and render your composition.\n\nAdobe AEP projects and Adobe plug-ins are not supported.",
    },
    Product {
        id: "filmcraft",
        name: "Cinevyn NS",
        role: "Edit video and sound",
        group: "Video",
        icon: include_bytes!("../../../branding/film.png"),
        guide: "1. Import media into the project.\n2. Create a sequence and arrange clips on its timeline.\n3. Trim cuts and adjust audio levels.\n4. Add titles, transitions and color adjustments.\n5. Save the project and export a test segment before the final movie.\n\nCheck the codecs and workflow required for each project.",
    },
    Product {
        id: "lightcraft",
        name: "Lumivyn NS",
        role: "Organize photos and develop RAW",
        group: "Photo",
        icon: include_bytes!("../../../branding/light.png"),
        guide: "1. Add photos to a library.\n2. Organize and rate the images.\n3. Adjust exposure, white balance and tone in Develop.\n4. Apply supported local adjustments.\n5. Export copies while retaining the original files.\n\nRAW support and color calibration differ by camera. Test your camera's files.",
    },
    Product {
        id: "designcraft",
        name: "Foliovyn NS",
        role: "Compose pages and publications",
        group: "Design",
        icon: include_bytes!("../../../branding/layout.png"),
        guide: "1. Create a document with page size and margins.\n2. Add text and image frames.\n3. Apply paragraph styles and align the layout.\n4. Review text overflow and linked assets.\n5. Save the editable document and export PDF.\n\nSelect two text frames and use Link text frames to connect their stories. Unlink following detaches the downstream frames. Professional print fidelity remains under development.",
    },
    Product {
        id: "pdfcraft",
        name: "Docuvyn NS",
        role: "Read, annotate and edit PDFs",
        group: "Documents",
        icon: include_bytes!("../../../branding/pdf.png"),
        guide: "1. Open a PDF and navigate its pages.\n2. Use supported text/image editing or annotations.\n3. Organize pages and inspect the result.\n4. Save a new copy of an important document.\n5. Reopen the exported file to verify its appearance.\n\nComplex PDF editing and Office conversion still have fidelity limits.",
    },
];
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Preferences {
    theme: String,
    sync_apps: bool,
    accent: [u8; 3],
    secondary: [u8; 3],
    light: bool,
    gradient: bool,
    glass: bool,
    opacity: f32,
    wallpaper: Option<PathBuf>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: glass(),
            sync_apps: false,
            accent: [164, 177, 200],
            secondary: [178, 147, 203],
            light: false,
            gradient: false,
            glass: true,
            opacity: 0.24,
            wallpaper: None,
        }
    }
}
fn glass() -> String {
    "glass".into()
}
const THEMES: &[(&str, &str)] = &[
    ("glass", "NS Glass"),
    ("dark", "NS Dark"),
    ("classic", "Classic Pro"),
    ("white", "NS White"),
    ("blue", "NS Blue"),
    ("red", "NS Red"),
    ("gradient", "NS Gradient"),
];
pub struct Launcher {
    root: PathBuf,
    prefs: Preferences,
    query: String,
    group: String,
    page: usize,
    guide: Option<usize>,
    textures: Vec<Option<egui::TextureHandle>>,
    logo: Option<egui::TextureHandle>,
    error: Option<String>,
    backdrop_key: String,
    backdrop_ready: bool,
}
fn settings() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("NS Studio Suite/launcher.json"))
}
pub fn suite_root() -> PathBuf {
    if let Some(root) = std::env::var_os("NS_SUITE_ROOT") {
        return PathBuf::from(root);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("suite.json").is_file() {
                return dir.to_path_buf();
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<egui::TextureHandle> {
    let pixels = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .ok()?
        .to_rgba8();
    let ci = egui::ColorImage::from_rgba_unmultiplied(
        [pixels.width() as usize, pixels.height() as usize],
        pixels.as_raw(),
    );
    Some(ctx.load_texture(name, ci, egui::TextureOptions::LINEAR))
}
impl Launcher {
    pub fn new(ctx: &egui::Context) -> Self {
        let prefs = settings()
            .and_then(|p| std::fs::read(p).ok())
            .filter(|b| b.len() < 65536)
            .and_then(|b| serde_json::from_slice::<Preferences>(&b).ok())
            .unwrap_or_else(|| Preferences::default());
        let mut prefs = prefs;
        prefs.opacity = if prefs.opacity.is_finite() {
            prefs.opacity.clamp(0.12, 0.85)
        } else {
            0.24
        };
        // Legacy wallpaper preferences never replace the live compositor backdrop.
        prefs.wallpaper = None;
        let textures = PRODUCTS
            .iter()
            .map(|p| texture(ctx, p.id, p.icon))
            .collect();
        Self {
            root: suite_root(),
            prefs,
            query: String::new(),
            group: "All".into(),
            page: 0,
            guide: None,
            textures,
            logo: texture(
                ctx,
                "NS Studio",
                include_bytes!("../../../branding/ns-studio-logo.png"),
            ),
            error: None,
            backdrop_key: String::new(),
            backdrop_ready: false,
        }
    }
    fn save(&mut self) {
        let result = (|| -> Result<(), String> {
            let path = settings().ok_or("No settings directory")?;
            let parent = path.parent().ok_or("Invalid settings path")?;
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            let bytes = serde_json::to_vec_pretty(&self.prefs).map_err(|e| e.to_string())?;
            let tmp = path.with_extension("tmp");
            std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
            std::fs::rename(tmp, path).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if let Err(e) = result {
            self.error = Some(e);
        }
    }
    fn executable(&self, p: Product) -> PathBuf {
        self.root
            .join("apps")
            .join(p.id)
            .join(format!("{}.exe", p.name.replace(' ', "-")))
    }
    fn launch(&mut self, p: Product) {
        self.launch_mode(p, false);
    }
    fn launch_mode(&mut self, p: Product, toolkit: bool) {
        let exe = self.executable(p);
        if !exe.is_file() {
            self.error = Some(format!("{} is not included in this package yet.", p.name));
            return;
        }
        let mut command = std::process::Command::new(&exe);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        if let Some(dir) = exe.parent() {
            command.current_dir(dir);
        }
        if toolkit && p.id == "effectcraft" {
            command.env(
                "NS_NATIVE_PANEL",
                self.root.join("extensions/NS-Studio-Native.jsx"),
            );
        }
        if self.prefs.sync_apps {
            command.env("NS_STUDIO_THEME", "custom");
            command.env(
                "NS_STUDIO_COLORS",
                format!(
                    "{},{},{},{},{},{},{},{}",
                    self.prefs.accent[0],
                    self.prefs.accent[1],
                    self.prefs.accent[2],
                    self.prefs.secondary[0],
                    self.prefs.secondary[1],
                    self.prefs.secondary[2],
                    u8::from(self.is_light()),
                    u8::from(self.prefs.gradient)
                ),
            );
        }
        if let Some(dir) = settings().and_then(|file| {
            file.parent()
                .map(|d| d.join("profiles-live-glass-final").join(p.id))
        }) {
            command.env(format!("{}_CONFIG_DIR", p.id.to_ascii_uppercase()), dir);
        }
        if let Err(e) = command.spawn() {
            self.error = Some(format!("Could not open {}: {e}", p.name));
        }
    }
    fn is_light(&self) -> bool {
        self.prefs.theme == "white" || self.prefs.light
    }
    fn colors(&self) -> (Color32, Color32, Color32, Color32) {
        let light = self.is_light();
        let tint = Color32::from_rgb(
            self.prefs.accent[0],
            self.prefs.accent[1],
            self.prefs.accent[2],
        );
        let bg = if light {
            Color32::from_rgb(232, 235, 240)
        } else {
            Color32::from_rgb(16, 18, 22)
        };
        let panel = if light {
            Color32::from_rgb(249, 250, 253)
        } else {
            Color32::from_rgb(30, 33, 39)
        };
        let text = if light {
            Color32::from_rgb(25, 28, 34)
        } else {
            Color32::from_rgb(241, 243, 247)
        };
        let accent =
            ns_glass::readable_accent(tint, Color32::from_gray(if light { 211 } else { 102 }));
        (bg, panel.lerp_to_gamma(tint, 0.06), text, accent)
    }
    fn appearance(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(300.0);
        ui.style_mut().spacing.slider_width = 120.0;
        let mut changed = false;
        ui.strong("Appearance");
        ui.label(RichText::new("Live windows behind this launcher · system blur").small());
        ui.horizontal(|ui| {
            changed |= ui
                .selectable_value(&mut self.prefs.light, false, "Dark")
                .changed();
            changed |= ui
                .selectable_value(&mut self.prefs.light, true, "Light")
                .changed();
        });
        changed |= ui.checkbox(&mut self.prefs.glass, "Clear glass").changed();
        if self.prefs.glass {
            changed |= ui
                .add(
                    egui::Slider::new(&mut self.prefs.opacity, 0.12..=0.85)
                        .text("Glass tint")
                        .trailing_fill(true)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                        .custom_parser(|s| {
                            s.trim_end_matches('%')
                                .parse::<f64>()
                                .ok()
                                .filter(|v| v.is_finite())
                                .map(|v| v / 100.0)
                        }),
                )
                .changed();
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Accent color");
            changed |= ui.color_edit_button_srgb(&mut self.prefs.accent).changed();
        });
        changed |= ui
            .checkbox(&mut self.prefs.gradient, "Gradient tint")
            .changed();
        if self.prefs.gradient {
            ui.horizontal(|ui| {
                ui.label("Second color");
                changed |= ui
                    .color_edit_button_srgb(&mut self.prefs.secondary)
                    .changed();
            });
        }
        egui::ComboBox::from_label("Presets")
            .selected_text(
                THEMES
                    .iter()
                    .find(|(id, _)| *id == self.prefs.theme)
                    .map_or("Custom", |(_, label)| *label),
            )
            .show_ui(ui, |ui| {
                for (id, label) in THEMES {
                    if ui
                        .selectable_label(self.prefs.theme == *id, *label)
                        .clicked()
                    {
                        self.prefs.theme = (*id).into();
                        self.prefs.light = *id == "white";
                        self.prefs.glass = *id == "glass";
                        self.prefs.gradient = *id == "gradient";
                        self.prefs.accent = match *id {
                            "red" => [230, 80, 105],
                            "blue" => [80, 135, 240],
                            "gradient" => [136, 118, 230],
                            _ => [164, 177, 200],
                        };
                        self.save();
                    }
                }
            });
        ui.separator();
        if !self.backdrop_ready {
            ui.label(
                RichText::new("Live backdrop unavailable; using readable solid surfaces.").small(),
            );
        }
        ui.separator();
        changed |= ui
            .checkbox(
                &mut self.prefs.sync_apps,
                "Use these colors when opening apps",
            )
            .changed();
        if changed {
            self.prefs.theme = "custom".into();
            self.save();
        }
    }
    fn background(&self, ui: &mut egui::Ui) {
        // Leave the swapchain transparent: DWM supplies the live blurred windows.
        // Never paint a wallpaper texture over that composited background.
        if !self.prefs.glass || !self.backdrop_ready {
            ui.painter()
                .rect_filled(ui.max_rect().expand(24.0), 0, self.colors().0);
        }
    }
    fn panel_shapes(&self, rect: egui::Rect) -> Vec<egui::Shape> {
        let (_, panel, text, _) = self.colors();
        let alpha = if self.backdrop_ready {
            if self.is_light() {
                (220.0 + self.prefs.opacity * 25.0) as u8
            } else {
                (184.0 + self.prefs.opacity * 40.0) as u8
            }
        } else {
            255
        };
        let fill = Color32::from_rgba_unmultiplied(panel.r(), panel.g(), panel.b(), alpha);
        let body = if self.prefs.gradient {
            let second = panel.lerp_to_gamma(
                Color32::from_rgb(
                    self.prefs.secondary[0],
                    self.prefs.secondary[1],
                    self.prefs.secondary[2],
                ),
                0.06,
            );
            let second = Color32::from_rgba_unmultiplied(second.r(), second.g(), second.b(), alpha);
            let mut mesh = egui::Mesh::default();
            let radius = 16.0_f32.min(rect.width() * 0.5).min(rect.height() * 0.5);
            let color_at = |x: f32| {
                fill.lerp_to_gamma(second, ((x - rect.left()) / rect.width()).clamp(0.0, 1.0))
            };
            mesh.colored_vertex(rect.center(), color_at(rect.center().x));
            for (center, start) in [
                (rect.left_top() + vec2(radius, radius), std::f32::consts::PI),
                (
                    rect.right_top() + vec2(-radius, radius),
                    std::f32::consts::PI * 1.5,
                ),
                (rect.right_bottom() + vec2(-radius, -radius), 0.0),
                (
                    rect.left_bottom() + vec2(radius, -radius),
                    std::f32::consts::FRAC_PI_2,
                ),
            ] {
                for step in 0..=8 {
                    let angle = start + std::f32::consts::FRAC_PI_2 * step as f32 / 8.0;
                    let pos = center + vec2(angle.cos(), angle.sin()) * radius;
                    mesh.colored_vertex(pos, color_at(pos.x));
                }
            }
            let count = mesh.vertices.len() as u32 - 1;
            for i in 1..=count {
                mesh.add_triangle(0, i, if i == count { 1 } else { i + 1 });
            }
            egui::Shape::mesh(mesh)
        } else {
            egui::Shape::rect_filled(rect, 16.0, fill)
        };
        vec![
            body,
            egui::Shape::rect_stroke(
                rect.shrink(0.5),
                16.0,
                Stroke::new(1.0, text.gamma_multiply(0.20)),
                egui::StrokeKind::Inside,
            ),
            egui::Shape::line_segment(
                [
                    rect.left_top() + vec2(16.0, 1.0),
                    rect.right_top() + vec2(-16.0, 1.0),
                ],
                Stroke::new(1.0, Color32::from_white_alpha(45)),
            ),
        ]
    }
    fn pressure_button(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        size: egui::Vec2,
        enabled: bool,
    ) -> egui::Response {
        let (_, panel, text, accent) = self.colors();
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let response = response.on_hover_cursor(if enabled {
            egui::CursorIcon::PointingHand
        } else {
            egui::CursorIcon::NotAllowed
        });
        let down = enabled && response.is_pointer_button_down_on();
        let hover = enabled && response.hovered();
        let pressure = ui.ctx().animate_value_with_time(
            response.id.with("pressure"),
            if down { 1.0 } else { 0.0 },
            0.10,
        );
        let glow = ui.ctx().animate_value_with_time(
            response.id.with("glow"),
            if down {
                1.0
            } else if hover {
                0.45
            } else {
                0.0
            },
            0.16,
        );
        let visual = rect.shrink2(vec2(1.8 * pressure, 0.8 * pressure));
        let fill = panel.lerp_to_gamma(accent, 0.08 + glow * 0.20);
        let fill = ns_glass::background_for_text(fill, text);
        ui.painter()
            .rect_filled(visual, 10.0 + pressure * 2.0, fill);
        ui.painter().rect_stroke(
            visual,
            10.0 + pressure * 2.0,
            Stroke::new(1.0 + glow, accent.gamma_multiply(0.30 + glow * 0.60)),
            egui::StrokeKind::Inside,
        );
        if label=="Themes" {
            let c=visual.center();
            let points:Vec<_>=(0..=16).map(|n|{let a=std::f32::consts::FRAC_PI_2+n as f32*std::f32::consts::PI/16.0;c+vec2(a.cos(),a.sin())*8.0})
                .chain((0..=16).map(|n|{let a=std::f32::consts::PI*1.5-n as f32*std::f32::consts::PI/16.0;c+vec2(4.0,0.0)+vec2(a.cos(),a.sin())*8.0})).collect();
            ui.painter().add(egui::Shape::closed_line(points,Stroke::new(1.4,text)));
        }else{
        ui.painter().text(
            visual.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(13.0),
            if enabled {
                text
            } else {
                text.gamma_multiply(0.55)
            },
        );
        }
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
        response
    }
    fn appearance_menu(&mut self, ui: &mut egui::Ui) {
        let response = self.pressure_button(ui, "Themes", vec2(32.0, 32.0), true);
        ns_glass::theme_popup(ui, &response, |ui| {
            let mut chosen = false;
            ui.horizontal(|ui| {
                for (id, label) in [
                    ("glass", "Glass"),
                    ("dark", "Black"),
                    ("classic", "Classic Pro"),
                    ("white", "White"),
                    ("blue", "NS Blue"),
                    ("gradient", "NS Gradient"),
                ] {
                    ui.vertical(|ui| {
                        let short = match id {
                            "glass" => "G",
                            "dark" => "B",
                            "classic" => "CP",
                            "white" => "W",
                            "blue" => "NB",
                            _ => "NG",
                        };
                        if ui
                            .add(
                                egui::Button::new(short)
                                    .min_size(vec2(36.0, 36.0))
                                    .corner_radius(18),
                            )
                            .on_hover_text(label)
                            .clicked()
                        {
                            self.prefs.theme = id.into();
                            self.prefs.glass = id == "glass";
                            self.prefs.light = id == "white";
                            self.prefs.gradient = id == "gradient";
                            self.save();
                            chosen = true;
                        }
                        ui.label(RichText::new(label).size(10.0));
                    });
                }
                ui.menu_button("Colors", |ui| self.appearance(ui));
            });
            chosen
        });
    }
    fn style(&self, ctx: &egui::Context) {
        let (bg, panel, text, accent) = self.colors();
        ctx.all_styles_mut(|s| {
            let mut v = if self.is_light() {
                egui::Visuals::light()
            } else {
                egui::Visuals::dark()
            };
            v.panel_fill = bg;
            v.window_fill = panel;
            v.extreme_bg_color = bg;
            v.override_text_color = None;
            v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text);
            v.hyperlink_color = accent;
            v.selection.bg_fill = accent.gamma_multiply(0.25);
            v.selection.stroke = Stroke::new(1., accent);
            v.window_corner_radius = egui::CornerRadius::same(9);
            v.widgets.inactive.bg_fill = panel;
            v.widgets.inactive.weak_bg_fill = panel;
            v.widgets.inactive.bg_stroke = Stroke::new(1., accent.gamma_multiply(0.25));
            v.widgets.hovered.bg_fill = panel.lerp_to_gamma(accent, 0.12);
            v.widgets.active.bg_fill =
                ns_glass::background_for_text(panel.lerp_to_gamma(accent, 0.30), text);
            v.widgets.active.fg_stroke = Stroke::new(1.4, text);
            for w in [
                &mut v.widgets.inactive,
                &mut v.widgets.hovered,
                &mut v.widgets.active,
            ] {
                w.corner_radius = egui::CornerRadius::same(9);
                w.fg_stroke = Stroke::new(1., text);
            }
            v.widgets.active.fg_stroke = Stroke::new(1.4, text);
            v.widgets.active.bg_stroke = Stroke::new(1.8, accent);
            v.widgets.hovered.bg_stroke = Stroke::new(1.3, accent);
            s.visuals = v;
            s.animation_time = 0.12;
            s.spacing.item_spacing = vec2(10., 8.);
            s.spacing.button_padding = vec2(14., 8.);
        });
    }
    fn image(ui: &mut egui::Ui, texture: Option<&egui::TextureHandle>, size: f32) {
        if let Some(t) = texture {
            ui.add(egui::Image::new(t).fit_to_exact_size(vec2(size, size)));
        }
    }
    fn card(&mut self, ui: &mut egui::Ui, index: usize) {
        let Some(p) = PRODUCTS.get(index).copied() else {
            return;
        };
        let (_, panel, text, accent) = self.colors();
        let ready = self.executable(p).is_file();
        egui::Frame::new()
            .fill(if self.prefs.glass {
                Color32::TRANSPARENT
            } else {
                panel
            })
            .corner_radius(8)
            .stroke(if self.prefs.glass {
                Stroke::NONE
            } else {
                Stroke::new(1., text.gamma_multiply(0.15))
            })
            .inner_margin(14)
            .show(ui, |ui| {
                let slot = if self.prefs.glass {
                    Some(ns_glass::reserve_surface(ui.painter()))
                } else {
                    None
                };
                ui.set_min_height(144.);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{:02}", index + 1))
                            .monospace()
                            .color(accent),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.hyperlink_to(
                            "NS Studio",
                            "https://nabzz-studio.nexto-ai-8590.chatgpt.site/",
                        );
                    });
                });
                ui.add_space(12.);
                ui.horizontal(|ui| {
                    Self::image(ui, self.textures.get(index).and_then(Option::as_ref), 48.);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(p.name).size(21.).strong());
                        ui.label(RichText::new(p.role).size(12.).color(text));
                    });
                });
                ui.add_space(12.);
                ui.label(
                    RichText::new(if ready {
                        "INSTALLED  ·  NS development build"
                    } else {
                        "NOT INCLUDED"
                    })
                    .size(11.)
                    .monospace()
                    .color(text),
                );
                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 42.0).max(70.0);
                    if self
                        .pressure_button(ui, "Open", vec2(width, 32.0), ready)
                        .clicked()
                        && ready
                    {
                        self.launch(p);
                    }
                    ui.menu_button("...", |ui| {
                        if p.id == "effectcraft" && ui.button("Open NS native tools").clicked() {
                            self.launch_mode(p, true);
                            ui.close();
                        }
                        if ui.button("Quick guide").clicked() {
                            self.guide = Some(index);
                            ui.close();
                        }
                    });
                });
                if let Some(slot) = slot {
                    ui.painter().set(
                        slot,
                        egui::Shape::Vec(self.panel_shapes(ui.min_rect().expand(14.0))),
                    );
                }
            });
    }
    pub fn frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.style(&ctx);
        let (_, _, text, _) = self.colors();
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(24),
            )
            .show(ui, |ui| {
                self.background(ui);
                let (_, panel, _, _) = self.colors();
                let header = egui::Rect::from_min_size(
                    ui.max_rect().min - egui::vec2(8.0, 8.0),
                    egui::vec2(ui.max_rect().width() + 16.0, 192.0),
                );
                if self.prefs.glass {
                    ui.painter()
                        .add(egui::Shape::Vec(self.panel_shapes(header)));
                } else {
                    ui.painter().rect_filled(header, 8.0, panel);
                }
                ui.horizontal(|ui| {
                    if let Some(logo) = self.logo.as_ref() {
                        if ui
                            .add(
                                egui::Image::new(logo)
                                    .fit_to_exact_size(vec2(44.0, 44.0))
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text("NS Studio Website")
                            .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(
                                "https://nabzz-studio.nexto-ai-8590.chatgpt.site/",
                            ));
                        }
                    }
                    ui.label(RichText::new("NS STUDIO").size(23.).strong());
                    ui.label(RichText::new("LAUNCHER").monospace().size(11.));
                    ui.label(RichText::new("CREATIVE APP SUITE").size(10.).color(text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.appearance_menu(ui);
                        if ui.selectable_label(self.page == 1, "TABLE").clicked() {
                            self.page = 1;
                        }
                        if ui.selectable_label(self.page == 0, "GRID").clicked() {
                            self.page = 0;
                        }
                    });
                });
                ui.add_space(16.);
                ui.separator();
                ui.add_space(14.);
                ui.horizontal(|ui| {
                    for label in ["All", "Installed", "Not installed"] {
                        if ui
                            .selectable_label(self.group == label, label.to_uppercase())
                            .clicked()
                        {
                            self.group = label.into();
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.query)
                                .hint_text("Search apps")
                                .desired_width(220.),
                        );
                    });
                });
                ui.add_space(30.);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("01 / CREATIVE APPS")
                            .size(12.)
                            .monospace()
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("7 APPS");
                    });
                });
                ui.add_space(12.);
                if let Some(error) = self.error.clone() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(error);
                        if ui.button("Dismiss").clicked() {
                            self.error = None;
                        }
                    });
                }
                let q = self.query.to_ascii_lowercase();
                let ids: Vec<usize> = PRODUCTS
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| {
                        let ready = self.executable(**p).is_file();
                        (self.group == "All"
                            || self.group == "Installed" && ready
                            || self.group == "Not installed" && !ready)
                            && (q.is_empty()
                                || format!("{} {} {}", p.name, p.role, p.group)
                                    .to_ascii_lowercase()
                                    .contains(&q))
                    })
                    .map(|(i, _)| i)
                    .collect();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.page == 1 {
                            for i in ids {
                                let Some(p) = PRODUCTS.get(i).copied() else {
                                    continue;
                                };
                                ui.horizontal(|ui| {
                                    Self::image(
                                        ui,
                                        self.textures.get(i).and_then(Option::as_ref),
                                        32.,
                                    );
                                    ui.label(RichText::new(p.name).strong());
                                    ui.label(p.role);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("Guide").clicked() {
                                                self.guide = Some(i);
                                            }
                                            if ui
                                                .add_enabled(
                                                    self.executable(p).is_file(),
                                                    egui::Button::new("Open"),
                                                )
                                                .clicked()
                                            {
                                                self.launch(p);
                                            }
                                        },
                                    );
                                });
                                ui.separator();
                            }
                        } else {
                            let n = if ui.available_width() > 950. {
                                3
                            } else if ui.available_width() > 620. {
                                2
                            } else {
                                1
                            };
                            for row in ids.chunks(n) {
                                ui.columns(n, |cols| {
                                    for (i, col) in row.iter().zip(cols.iter_mut()) {
                                        self.card(col, *i);
                                    }
                                });
                                ui.add_space(2.);
                            }
                        }
                    });
            });
        if let Some(index) = self.guide {
            let Some(p) = PRODUCTS.get(index).copied() else {
                return;
            };
            let mut open = true;
            egui::Window::new(format!("{} · Quick guide", p.name))
                .open(&mut open)
                .default_width(580.)
                .show(&ctx, |ui| {
                    ui.label(p.guide);
                    if ui.button("Open application").clicked() {
                        self.launch(p);
                    }
                });
            if !open {
                self.guide = None;
            }
        }
    }
}

impl eframe::App for Launcher {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        #[cfg(windows)]
        if let Some(window) = frame.winit_window() {
            let key = format!(
                "{}:{}:{}:{}",
                self.prefs.glass,
                self.is_light(),
                self.prefs.opacity,
                false
            );
            if key != self.backdrop_key {
                self.backdrop_key = key;
                if self.prefs.glass {
                    let base = if self.is_light() {
                        (235, 237, 240)
                    } else {
                        (18, 20, 24)
                    };
                    match window_vibrancy::apply_acrylic(
                        window.as_ref(),
                        Some((base.0, base.1, base.2, 72)),
                    ) {
                        Ok(()) => self.backdrop_ready = true,
                        Err(e) => {
                            self.backdrop_ready = false;
                            self.error = Some(format!(
                                "Windows glass unavailable: {e}. Using a solid readable background."
                            ));
                        }
                    }
                } else {
                    if let Err(e) = window_vibrancy::clear_acrylic(window.as_ref()) {
                        self.error = Some(format!("Could not clear desktop backdrop: {e}"));
                    }
                    self.backdrop_ready = false;
                }
            }
        }
        #[cfg(not(windows))]
        let _ = frame;
        self.frame(ui);
        // Optional native smoke check used by the build workflow; normal launches do not set it.
        #[cfg(windows)]
        if let Some(path) = std::env::var_os("NS_LAUNCHER_GLASS_REPORT") {
            if frame.winit_window().is_some() {
                let report = format!(
                    "Windows live Desktop Acrylic setup succeeded: {}\nError: {}\n",
                    self.backdrop_ready,
                    self.error.as_deref().unwrap_or("none")
                );
                if let Err(e) = std::fs::write(path, report) {
                    self.error = Some(format!("Could not write glass check: {e}"));
                }
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_names_end_in_ns_and_executables_stay_under_the_suite() {
        let ctx = egui::Context::default();
        let app = Launcher::new(&ctx);
        for p in PRODUCTS {
            assert!(p.name.ends_with(" NS"));
            assert!(app.executable(*p).starts_with(&app.root));
        }
    }
    #[test]
    fn every_product_has_a_distinct_decodable_icon() {
        let mut fingerprints = std::collections::BTreeSet::new();
        for p in PRODUCTS {
            let image = image::load_from_memory(p.icon).expect("valid original PNG");
            assert_eq!(image.width(), 512);
            assert!(fingerprints.insert(p.icon));
        }
    }
}

#[cfg(test)]
mod appearance_tests {
    use super::*;
    #[test]
    fn legacy_preferences_gain_glass_and_color_controls() {
        let p: Preferences = serde_json::from_str(r#"{"theme":"dark","sync_apps":true}"#).unwrap();
        assert!(p.glass);
        assert!(p.sync_apps);
        assert_eq!(p.accent, [164, 177, 200]);
    }
    #[test]
    fn custom_preferences_survive_serialization() {
        let p = Preferences {
            accent: [235, 20, 80],
            secondary: [15, 155, 230],
            light: true,
            gradient: true,
            opacity: 0.2,
            ..Default::default()
        };
        let encoded = serde_json::to_string(&p).unwrap();
        let q: Preferences = serde_json::from_str(&encoded).unwrap();
        assert_eq!(q.accent, p.accent);
        assert_eq!(q.secondary, p.secondary);
        assert!(q.light && q.gradient);
        assert_eq!(q.opacity, 0.2);
    }
    #[test]
    fn custom_accent_is_readable_on_the_material_body() {
        let ctx = egui::Context::default();
        let mut app = Launcher::new(&ctx);
        for light in [false, true] {
            app.prefs.light = light;
            app.prefs.theme = "custom".into();
            for c in [[255, 255, 255], [0, 0, 0], [255, 220, 0], [255, 0, 160]] {
                app.prefs.accent = c;
                let accent = app.colors().3;
                assert!(
                    ns_glass::contrast(accent, Color32::from_gray(if light { 211 } else { 102 }))
                        >= 4.5
                );
                assert!(ns_glass::contrast(ns_glass::on_color(accent), accent) >= 4.5);
            }
        }
    }
}

#[cfg(test)]
mod native_regressions {
    use super::*;
    #[test]
    fn strong_headings_and_active_buttons_have_independent_readable_colors() {
        let ctx = egui::Context::default();
        let mut app = Launcher::new(&ctx);
        for light in [false, true] {
            for accent in [[255, 255, 255], [0, 0, 0], [255, 220, 0]] {
                app.prefs.light = light;
                app.prefs.theme = "custom".into();
                app.prefs.accent = accent;
                app.style(&ctx);
                let style = ctx.style_of(ctx.theme());
                let v = &style.visuals;
                assert!(
                    ns_glass::contrast(
                        v.strong_text_color(),
                        Color32::from_gray(if light { 211 } else { 102 })
                    ) >= 4.5
                );
                assert!(
                    ns_glass::contrast(v.widgets.active.fg_stroke.color, v.widgets.active.bg_fill)
                        >= 4.5
                );
            }
        }
    }
}

#[cfg(test)]
mod live_backdrop_regressions {
    use super::*;
    #[test]
    fn glass_body_preserves_text_contrast_over_bright_and_dark_windows() {
        let ctx = egui::Context::default();
        let mut app = Launcher::new(&ctx);
        app.backdrop_ready = true;
        app.prefs.gradient = false;
        for light in [false, true] {
            for tint in [
                [0, 0, 0],
                [255, 255, 255],
                [255, 0, 0],
                [0, 255, 0],
                [0, 0, 255],
            ] {
                app.prefs.light = light;
                app.prefs.theme = "custom".into();
                app.prefs.accent = tint;
                app.prefs.opacity = 0.12;
                let shapes = app.panel_shapes(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    vec2(300.0, 200.0),
                ));
                let egui::Shape::Rect(body) = &shapes[0] else {
                    panic!("body must be a translucent native surface")
                };
                let rgba = body.fill.to_srgba_unmultiplied();
                assert!(rgba[3] < 255);
                for backdrop in [0.0, 255.0] {
                    let alpha = rgba[3] as f32 / 255.0;
                    let rgb = [0, 1, 2]
                        .map(|n| (rgba[n] as f32 * alpha + backdrop * (1.0 - alpha)) as u8);
                    assert!(
                        ns_glass::contrast(
                            app.colors().2,
                            Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                        ) >= 4.5
                    );
                }
            }
        }
    }
    #[test]
    fn live_glass_does_not_allocate_a_wallpaper_texture() {
        let ctx = egui::Context::default();
        let mut app = Launcher::new(&ctx);
        app.backdrop_ready = true;
        assert!(app.prefs.wallpaper.is_none());
        app.backdrop_ready = false;
        let mut warmup = ctx.run_ui(Default::default(), |ui| app.frame(ui));
        warmup.textures_delta.clear();
        let count = ctx.tex_manager().read().num_allocated();
        app.backdrop_ready = true;
        let mut output = ctx.run_ui(Default::default(), |ui| app.frame(ui));
        output.textures_delta.clear();
        assert_eq!(
            ctx.tex_manager().read().num_allocated(),
            count,
            "glass must not load a backdrop image"
        );
    }
}

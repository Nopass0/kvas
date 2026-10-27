//! Standalone game player. Runs a packed game (`game.pak` next to the executable) or, for
//! development, a project folder: `kvas_player --project path/to/project`.

use eframe::egui;
use kvas_engine::assets::Assets;
use kvas_engine::project::{Project, ProjectConfig};
use kvas_engine::registry::Registry;
use kvas_engine::runtime::Runtime;
use kvas_engine::scene::Scene;
use kvas_engine::script::ScriptHost;
use kvas_render::debug::DebugDraw;
use kvas_render::{game_view, shell, Frame, Renderer};
use std::path::PathBuf;
use std::time::Instant;

struct Player {
    runtime: Option<Runtime>,
    renderer: Renderer,
    debug: DebugDraw,
    status: String,
    started: Instant,
    audio: Option<kvas_audio::AudioEngine>,
    assets: Assets,
    /// Frames shown so far: the first ones show a loading screen before assets are decoded.
    frames: u32,
}

struct Loaded {
    assets: Assets,
    config: ProjectConfig,
    scripts: Option<PathBuf>,
    title: String,
}

fn locate() -> Result<Loaded, String> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--project") {
        let dir = args.get(i + 1).ok_or("--project needs a folder")?;
        let project = Project::open(&PathBuf::from(dir))?;
        let scripts = project.root.join(".kvas").join(project.script_lib_name());
        return Ok(Loaded {
            assets: Assets::Dir(project.assets_dir()),
            title: project.config.name.clone(),
            scripts: scripts.exists().then_some(scripts),
            config: project.config,
        });
    }
    let exe_dir = std::env::current_exe().map_err(|e| e.to_string())?.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let assets = Assets::open_pak(&exe_dir.join("game.pak")).map_err(|e| format!("game.pak: {e}"))?;
    let config = ProjectConfig::from_json(&assets.read_string("project.json").map_err(|e| format!("project.json: {e}"))?)?;
    let lib = Project { root: exe_dir.clone(), config: config.clone() }.script_lib_name();
    let scripts = exe_dir.join(lib);
    Ok(Loaded { title: config.name.clone(), scripts: scripts.exists().then_some(scripts), assets, config })
}

fn start(loaded: &Loaded) -> Result<Runtime, String> {
    let text = loaded.assets.read_string(&loaded.config.start_scene).map_err(|e| format!("{}: {e}", loaded.config.start_scene))?;
    let (scene, warnings) = Scene::from_json(&Registry::default(), &text)?;
    let host = match &loaded.scripts {
        Some(path) => Some(ScriptHost::load(path)?),
        None => None,
    };
    let errors = loaded.config.validate();
    if !errors.is_empty() { return Err(errors.join("\n")); }
    let mut rt = Runtime::with_config(scene, host, loaded.config.timing)?;
    rt.set_time_scale(loaded.config.time_scale)?;
    rt.scene.audio.master_volume = loaded.config.master_volume;
    rt.set_bindings(&loaded.config.binding_defs());
    rt.log.extend(warnings);
    Ok(rt)
}

impl Player {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("wgpu backend required");
        let (runtime, assets, status) = match locate() {
            Ok(loaded) => match start(&loaded) {
                Ok(rt) => (Some(rt), loaded.assets, String::new()),
                Err(e) => (None, loaded.assets, e),
            },
            Err(e) => (None, Assets::Dir(PathBuf::new()), e),
        };
        kvas_engine::meshmod::set_assets(assets.clone());
        Self { runtime, renderer: Renderer::new(rs, assets.clone()), debug: DebugDraw::default(), status, started: Instant::now(), audio: kvas_audio::AudioEngine::new(), assets, frames: 0 }
    }
}

impl eframe::App for Player {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();
        if self.frames < 3 {
            // Decoding models and textures blocks the first real frame; show that something is happening first.
            self.frames += 1;
            egui::CentralPanel::default().frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(0x1E, 0x1F, 0x22))).show(ui, |ui| {
                ui.centered_and_justified(|ui| ui.label(egui::RichText::new("Loading...").size(22.0).color(egui::Color32::from_gray(200))));
            });
            return;
        }
        let (dt, ppp) = ctx.input(|i| (i.unstable_dt, i.pixels_per_point));
        if let Some(rt) = &mut self.runtime {
            let center = ctx.content_rect().center();
            ctx.input(|i| shell::feed_input(&mut rt.input, i, Some(center)));
            shell::apply_cursor(&ctx, &rt.input, center);
            rt.tick(dt);
            if let Some(audio) = &mut self.audio {
                audio.update(&mut rt.scene, &self.assets);
                for w in audio.warnings.drain(..) {
                    eprintln!("[audio] {w}");
                }
            }
            for line in rt.log.drain(..) {
                eprintln!("[game] {line}");
            }
        }
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            let size = ui.available_size();
            let (Some(rt), true) = (&self.runtime, size.x > 1.0 && size.y > 1.0) else {
                ui.centered_and_justified(|ui| ui.label(&self.status));
                return;
            };
            match game_view(&rt.scene, size.x / size.y) {
                Some(view) => {
                    let frame = Frame {
                        view,
                        size: [(size.x * ppp) as u32, (size.y * ppp) as u32],
                        time: self.started.elapsed().as_secs_f32(),
                        dt,
                        debug: &self.debug,
                        shadows: true,
                        debug_view: 0,
                        cinematic: true,
                    };
                    let tex = self.renderer.render(&rt.scene, &frame);
                    let response = ui.image((tex, size));
                    shell::draw_hud(ui.painter(), response.rect, &rt.input.hud());
                }
                None => {
                    ui.centered_and_justified(|ui| ui.label("The scene has no Camera"));
                }
            }
        });
        for e in self.renderer.take_errors() {
            eprintln!("[render] {e}");
        }
    }
}

/// The game's taskbar icon: `icon.png` next to the executable, else the Kvas logo.
fn window_icon(builder: egui::ViewportBuilder) -> egui::ViewportBuilder {
    let beside = std::env::current_exe().ok().and_then(|p| std::fs::read(p.with_file_name("icon.png")).ok());
    let bytes = beside.unwrap_or_else(|| include_bytes!("../../kvas_editor/assets/logo-256.png").to_vec());
    match eframe::icon_data::from_png_bytes(&bytes) {
        Ok(icon) => builder.with_icon(icon),
        Err(_) => builder,
    }
}

fn main() -> eframe::Result {
    let loaded = locate();
    let (title, window) = match &loaded {
        Ok(l) => {
            let b = &l.config.build;
            (if b.title.trim().is_empty() { l.title.clone() } else { b.title.clone() }, b.clone())
        }
        Err(_) => ("Kvas Player".to_string(), Default::default()),
    };
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([window.width.max(320) as f32, window.height.max(240) as f32])
        .with_title(title)
        .with_resizable(window.resizable)
        .with_maximized(window.maximized)
        .with_fullscreen(window.fullscreen);
    let mut options = eframe::NativeOptions { viewport: window_icon(viewport), ..Default::default() };
    options.wgpu_options.surface.present_mode = if window.vsync { eframe::wgpu::PresentMode::AutoVsync } else { eframe::wgpu::PresentMode::AutoNoVsync };
    eframe::run_native("Kvas Player", options, Box::new(|cc| Ok(Box::new(Player::new(cc)))))
}

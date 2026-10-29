//! Every screen of LAST SERVICE, on the "UI" object: the title menu, options (owned and saved
//! here, published as `ls.settings`), pause, the journal, notes, the power panel, the HUD
//! (objective, subtitles from `ls.say`, messages from `ls.msg`), the intro card, the death
//! screen, the dawn text and the credits. Story requests go to the director as `ls.menu` and
//! `ls.panel` signals.

use crate::s_shared::*;
use kvas_engine::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Main,
    Options,
    Credits,
    ConfirmNew,
}

#[derive(Clone)]
struct Line {
    who: String,
    en: String,
    ru: String,
    time: f32,
}

pub struct GameUI {
    page: Page,
    set: Settings,
    subs: VecDeque<Line>,
    sub_t: f32,
    message: (String, String, f32),
    fade: f32,
    journal_note: Option<String>,
    has_save: bool,
    last_mode: Option<Mode>,
}

impl Default for GameUI {
    fn default() -> Self {
        Self {
            page: Page::Main,
            set: Settings::default(),
            subs: VecDeque::new(),
            sub_t: 0.0,
            message: (String::new(), String::new(), 0.0),
            fade: 1.0,
            journal_note: None,
            has_save: false,
            last_mode: None,
        }
    }
}

impl GameUI {
    fn t(&self, en: &str, ru: &str) -> String {
        if self.set.russian { ru.to_string() } else { en.to_string() }
    }

    fn style(&self, size: f32) -> ButtonStyle {
        ButtonStyle { font: FONT.into(), size, text: PAPER, background: [0.0, 0.0, 0.0, 0.0], hover: [0.55, 0.12, 0.08, 0.55], outline: [0.0; 4], radius: 2.0, align: Align::Left }
    }

    fn menu(&self, ctx: &mut Ctx, cmd: &str) {
        let _ = ctx.emit("ls.menu", serde_json::json!({ "cmd": cmd }));
    }

    fn resume(&self, ctx: &mut Ctx) {
        set_mode(ctx, Mode::Playing);
    }

    // ------------------------------------------------------------ settings
    fn load_settings(&mut self, ctx: &Ctx) {
        let s = &mut self.set;
        s.master = ctx.setting_f32("master", s.master);
        s.music = ctx.setting_f32("music", s.music);
        s.effects = ctx.setting_f32("effects", s.effects);
        s.sensitivity = ctx.setting_f32("sensitivity", s.sensitivity);
        s.vhs = ctx.setting_bool("vhs", s.vhs);
        s.ps1 = ctx.setting_bool("ps1", s.ps1);
        s.reduced_motion = ctx.setting_bool("reduced_motion", s.reduced_motion);
        s.no_chase = ctx.setting_bool("no_chase", s.no_chase);
        s.russian = ctx.setting_text("language", "ru") == "ru";
        s.fullscreen = ctx.setting_bool("fullscreen", false);
    }

    fn save_settings(&self, ctx: &Ctx) {
        let s = &self.set;
        ctx.set_setting("master", s.master.into());
        ctx.set_setting("music", s.music.into());
        ctx.set_setting("effects", s.effects.into());
        ctx.set_setting("sensitivity", s.sensitivity.into());
        ctx.set_setting("vhs", s.vhs.into());
        ctx.set_setting("ps1", s.ps1.into());
        ctx.set_setting("reduced_motion", s.reduced_motion.into());
        ctx.set_setting("no_chase", s.no_chase.into());
        ctx.set_setting("language", (if s.russian { "ru" } else { "en" }).into());
        ctx.set_setting("fullscreen", s.fullscreen.into());
    }

    fn apply_settings(&self, ctx: &mut Ctx) {
        ctx.set_volumes(self.set.master, self.set.music, self.set.effects);
        ctx.set_fullscreen(self.set.fullscreen);
        ctx.set_global(SETTINGS, serde_json::to_value(&self.set).unwrap_or_default());
    }

    // ------------------------------------------------------------ title menu
    fn menu_ui(&mut self, ctx: &mut Ctx) {
        ctx.ui().set_style(self.style(26.0));
        let ui = ctx.ui();
        ui.panel([0.0, 0.0, 520.0, 720.0], [0.0, 0.0, 0.0, 0.55], 0.0);
        ui.image("ui/branding/last_service_logo.png", [48.0, 60.0, 360.0, 240.0], [1.0; 4]);
        match self.page {
            Page::Main => {
                let mut y = 330.0;
                let labels = [self.t("CONTINUE", "ПРОДОЛЖИТЬ"), self.t("NEW SHIFT", "НОВАЯ СМЕНА"), self.t("OPTIONS", "НАСТРОЙКИ"), self.t("CREDITS", "АВТОРЫ"), self.t("QUIT", "ВЫХОД")];
                if self.has_save {
                    if ctx.ui().button("continue", [60.0, y, 360.0, 44.0], &labels[0]) {
                        sfx(ctx, "sfx_ui", 0.7);
                        self.menu(ctx, "continue");
                        return;
                    }
                    y += 52.0;
                }
                if ctx.ui().button("new", [60.0, y, 360.0, 44.0], &labels[1]) {
                    sfx(ctx, "sfx_ui", 0.7);
                    if self.has_save {
                        self.page = Page::ConfirmNew;
                    } else {
                        self.menu(ctx, "new");
                    }
                    return;
                }
                y += 52.0;
                if ctx.ui().button("options", [60.0, y, 360.0, 44.0], &labels[2]) {
                    self.page = Page::Options;
                }
                y += 52.0;
                if ctx.ui().button("credits", [60.0, y, 360.0, 44.0], &labels[3]) {
                    self.page = Page::Credits;
                }
                y += 52.0;
                if ctx.ui().button("quit", [60.0, y, 360.0, 44.0], &labels[4]) {
                    ctx.quit_game();
                }
                if ctx.setting_bool("finished", false) {
                    let done = self.t("The shift is over.", "Смена закрыта.");
                    ctx.ui().text(&done, [60.0, 680.0], 15.0, DIM, "", Align::Left);
                }
            }
            Page::ConfirmNew => {
                let q = self.t("Start a new shift? The saved one will be lost.", "Начать новую смену? Сохранение будет потеряно.");
                ctx.ui().paragraph(&q, [60.0, 340.0], 380.0, 18.0, PAPER, "");
                let (yes, no) = (self.t("YES", "ДА"), self.t("BACK", "НАЗАД"));
                if ctx.ui().button("yes", [60.0, 420.0, 180.0, 44.0], &yes) {
                    let _ = ctx.storage().delete_save("autosave");
                    let _ = ctx.storage().delete_save("manual");
                    self.page = Page::Main;
                    self.menu(ctx, "new");
                    return;
                }
                if ctx.ui().button("no", [250.0, 420.0, 180.0, 44.0], &no) {
                    self.page = Page::Main;
                }
            }
            Page::Options => {
                if self.options_ui(ctx, 60.0, 320.0) {
                    self.page = Page::Main;
                }
            }
            Page::Credits => {
                self.credits_text(ctx, 60.0, 320.0);
                let back = self.t("BACK", "НАЗАД");
                if ctx.ui().button("back", [60.0, 640.0, 200.0, 40.0], &back) || ctx.key_pressed(Key::Escape) {
                    self.page = Page::Main;
                }
            }
        }
    }

    /// The options list; returns true when "back" was chosen.
    fn options_ui(&mut self, ctx: &mut Ctx, x: f32, y0: f32) -> bool {
        ctx.ui().set_style(ButtonStyle { size: 18.0, ..self.style(18.0) });
        let labels = [self.t("MASTER VOLUME", "ОБЩАЯ ГРОМКОСТЬ"), self.t("MUSIC", "МУЗЫКА"), self.t("EFFECTS", "ЭФФЕКТЫ"), self.t("MOUSE SENSITIVITY", "ЧУВСТВИТЕЛЬНОСТЬ МЫШИ")];
        let mut y = y0;
        let values = [self.set.master, self.set.music, self.set.effects, (self.set.sensitivity - 0.03) / 0.3];
        let mut out = values;
        for (i, label) in labels.iter().enumerate() {
            let ui = ctx.ui();
            ui.text(label, [x, y], 15.0, DIM, FONT, Align::Left);
            out[i] = ui.slider(&format!("opt{i}"), [x + 200.0, y + 2.0, 200.0, 16.0], values[i]);
            y += 32.0;
        }
        let mut changed = out != values;
        if changed {
            self.set.master = out[0];
            self.set.music = out[1];
            self.set.effects = out[2];
            self.set.sensitivity = 0.03 + out[3] * 0.3;
        }
        let toggles = [
            (self.t("TAPE DISTORTION", "ПЛЁНКА VHS"), self.set.vhs),
            (self.t("PS1 DITHER", "ДИЗЕРИНГ PS1"), self.set.ps1),
            (self.t("REDUCED MOTION", "МЕНЬШЕ ДВИЖЕНИЯ"), self.set.reduced_motion),
            (self.t("NO CHASE (story only)", "БЕЗ ПРЕСЛЕДОВАНИЯ"), self.set.no_chase),
            (self.t("FULLSCREEN", "ПОЛНЫЙ ЭКРАН"), self.set.fullscreen),
            (String::from("LANGUAGE: RU / EN"), self.set.russian),
        ];
        let mut res = [false; 6];
        for (i, (label, on)) in toggles.iter().enumerate() {
            res[i] = ctx.ui().toggle(&format!("tog{i}"), [x, y, 400.0, 30.0], label, *on);
            y += 32.0;
        }
        if res != [self.set.vhs, self.set.ps1, self.set.reduced_motion, self.set.no_chase, self.set.fullscreen, self.set.russian] {
            self.set.vhs = res[0];
            self.set.ps1 = res[1];
            self.set.reduced_motion = res[2];
            self.set.no_chase = res[3];
            self.set.fullscreen = res[4];
            self.set.russian = res[5];
            changed = true;
        }
        if changed {
            self.apply_settings(ctx);
            self.save_settings(ctx);
        }
        let back = self.t("BACK", "НАЗАД");
        ctx.ui().set_style(self.style(22.0));
        ctx.ui().button("opt_back", [x, y + 12.0, 200.0, 40.0], &back) || ctx.key_pressed(Key::Escape)
    }

    fn credits_text(&self, ctx: &mut Ctx, x: f32, y: f32) {
        let ru = self.set.russian;
        let lines = [
            ("LAST SERVICE", 24.0),
            (if ru { "Психологический хоррор на движке Kvas" } else { "A psychological horror game made with Kvas" }, 15.0),
            ("", 10.0),
            (if ru { "Сценарий и мир: производственная библия «Последняя подача»" } else { "Story and world: the LAST SERVICE production bible" }, 14.0),
            (if ru { "Модели: PSX Horror Props, PSX Kitchen Props, Tree Pack" } else { "Models: PSX Horror Props, PSX Kitchen Props, Tree Pack" }, 14.0),
            (if ru { "Музыка: библиотека horror_day, звук: синтез" } else { "Music: horror_day library; sound: synthesized" }, 14.0),
            (if ru { "Все события и персонажи вымышлены." } else { "All events and characters are fictional." }, 14.0),
        ];
        let mut yy = y;
        for (l, size) in lines {
            ctx.ui().text(l, [x, yy], size, PAPER, if size > 20.0 { FONT } else { "" }, Align::Left);
            yy += size + 12.0;
        }
    }

    // ------------------------------------------------------------ in play
    fn hud(&mut self, ctx: &mut Ctx, dt: f32) {
        let w = ctx.ui_width();
        let m = mode(ctx);
        ctx.hud_crosshair(m == Mode::Playing);
        if let Some(line) = self.subs.front().cloned() {
            self.sub_t += dt;
            let who = match (line.who.as_str(), self.set.russian) {
                ("Anna", true) => "Анна",
                ("Lida", true) => "Лида",
                ("Pavel", true) => "Павел",
                (w, _) => w,
            };
            let text = format!("{who}: {}", if self.set.russian { &line.ru } else { &line.en });
            let ui = ctx.ui();
            ui.panel([w * 0.5 - 380.0, 560.0, 760.0, 54.0], [0.0, 0.0, 0.0, 0.55], 6.0);
            ui.text(&text, [w * 0.5, 575.0], 19.0, if line.who == "Anna" { [0.95, 0.75, 0.6, 1.0] } else { PAPER }, "", Align::Center);
            if self.sub_t > line.time {
                self.sub_t = 0.0;
                self.subs.pop_front();
            }
        }
        if m != Mode::Playing {
            return;
        }
        let objective = objective(&story(ctx), self.set.russian);
        let ui = ctx.ui();
        ui.text(&objective, [24.0, 20.0], 15.0, [PAPER[0], PAPER[1], PAPER[2], 0.85], "", Align::Left);
        if self.message.2 > 0.0 {
            self.message.2 -= dt;
            let a = self.message.2.min(1.0);
            let text = if self.set.russian { &self.message.1 } else { &self.message.0 };
            ui.text(text, [w * 0.5, 110.0], 18.0, [PAPER[0], PAPER[1], PAPER[2], a], "", Align::Center);
        }
    }

    fn pause_ui(&mut self, ctx: &mut Ctx) {
        let w = ctx.ui_width();
        ctx.ui().panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, 0.6], 0.0);
        ctx.ui().set_style(self.style(26.0));
        if self.page == Page::Options {
            if self.options_ui(ctx, w * 0.5 - 200.0, 200.0) {
                self.page = Page::Main;
            }
            return;
        }
        let title = self.t("PAUSE", "ПАУЗА");
        let items = [self.t("RESUME", "ПРОДОЛЖИТЬ"), self.t("JOURNAL", "ЖУРНАЛ"), self.t("SAVE", "СОХРАНИТЬ"), self.t("OPTIONS", "НАСТРОЙКИ"), self.t("QUIT TO MENU", "ВЫЙТИ В МЕНЮ")];
        ctx.ui().text(&title, [w * 0.5, 190.0], 34.0, PAPER, FONT, Align::Center);
        let mut picked = None;
        for (i, label) in items.iter().enumerate() {
            if ctx.ui().button(&format!("p{i}"), [w * 0.5 - 180.0, 260.0 + i as f32 * 52.0, 360.0, 44.0], label) {
                picked = Some(i);
            }
        }
        match picked {
            Some(0) => self.resume(ctx),
            Some(1) => set_mode(ctx, Mode::Journal),
            Some(2) => {
                self.menu(ctx, "save");
                self.resume(ctx);
            }
            Some(3) => self.page = Page::Options,
            Some(4) => self.menu(ctx, "quit_to_menu"),
            _ => {
                if ctx.key_pressed(Key::Escape) && mode_time(ctx) > 0.2 {
                    self.resume(ctx);
                }
            }
        }
    }

    fn reading_ui(&mut self, ctx: &mut Ctx) {
        let w = ctx.ui_width();
        let id = ctx.global(READING).and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
        let (title, body) = note_text(&id, self.set.russian);
        let img = format!("ui/notes/{id}.png");
        let close = self.t("[E] / [ESC]  close", "[E] / [ESC]  закрыть");
        let ui = ctx.ui();
        ui.panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, 0.75], 0.0);
        ui.image(&img, [w * 0.5 - 330.0, 90.0, 300.0, 400.0], [1.0; 4]);
        ui.frame([w * 0.5, 90.0, 360.0, 400.0], [0.9, 0.86, 0.74, 0.97], [0.3, 0.25, 0.2, 1.0], 3.0);
        ui.text(title, [w * 0.5 + 20.0, 110.0], 20.0, [0.15, 0.1, 0.08, 1.0], "", Align::Left);
        ui.paragraph(body, [w * 0.5 + 20.0, 150.0], 320.0, 18.0, [0.12, 0.1, 0.08, 1.0], "");
        ui.text(&close, [w * 0.5, 520.0], 15.0, DIM, "", Align::Center);
        if mode_time(ctx) > 0.15 && (ctx.action_pressed("Interact") || ctx.key_pressed(Key::Escape) || ctx.mouse_pressed(MouseButton::Left)) {
            self.resume(ctx);
        }
    }

    fn journal_ui(&mut self, ctx: &mut Ctx) {
        let w = ctx.ui_width();
        let s = story(ctx);
        ctx.ui().panel([0.0, 0.0, w, 720.0], [0.02, 0.02, 0.02, 0.88], 0.0);
        let (title, obj, names_t) = (self.t("JOURNAL", "ЖУРНАЛ"), objective(&s, self.set.russian), self.t("ROLE - NAME - EVIDENCE", "РОЛЬ - ИМЯ - СВИДЕТЕЛЬСТВО"));
        ctx.ui().text(&title, [80.0, 50.0], 30.0, PAPER, FONT, Align::Left);
        ctx.ui().paragraph(&obj, [80.0, 100.0], 520.0, 18.0, PAPER, "");
        ctx.ui().text(&names_t, [80.0, 170.0], 16.0, DIM, FONT, Align::Left);
        let rows = [
            (self.t("Driver", "Водитель"), if s.key { "Pavel Zorin / Павел Зорин" } else { "?" }, if s.key { self.t("key, route sheet", "ключ, путевой лист") } else { String::new() }),
            (self.t("Teacher", "Учительница"), if s.vera_book { "Vera Sedova / Вера Седова" } else { "?" }, if s.vera_book { self.t("exercise book", "тетрадь") } else { String::new() }),
            (self.t("Girl", "Девочка"), if s.nina_label { "Nina Sedova / Нина Седова" } else { "?" }, if s.doll { self.t("doll", "кукла") } else { String::new() }),
        ];
        for (i, (role, name, ev)) in rows.iter().enumerate() {
            let y = 200.0 + i as f32 * 30.0;
            ctx.ui().text(role, [80.0, y], 17.0, PAPER, "", Align::Left);
            ctx.ui().text(name, [230.0, y], 17.0, PAPER, "", Align::Left);
            ctx.ui().text(ev, [470.0, y], 17.0, DIM, "", Align::Left);
        }
        let notes_t = self.t("NOTES", "ЗАПИСКИ");
        ctx.ui().text(&notes_t, [80.0, 310.0], 16.0, DIM, FONT, Align::Left);
        ctx.ui().set_style(ButtonStyle { size: 16.0, align: Align::Left, ..self.style(16.0) });
        for (i, id) in s.notes.iter().enumerate() {
            let (title, _) = note_text(id, self.set.russian);
            if ctx.ui().button(&format!("note{i}"), [80.0, 340.0 + i as f32 * 34.0, 420.0, 30.0], &format!("{id}  {title}")) {
                self.journal_note = Some(id.clone());
            }
        }
        if let Some(id) = self.journal_note.clone() {
            let (title, body) = note_text(&id, self.set.russian);
            ctx.ui().frame([w - 520.0, 170.0, 440.0, 420.0], [0.9, 0.86, 0.74, 0.97], [0.3, 0.25, 0.2, 1.0], 3.0);
            ctx.ui().text(title, [w - 500.0, 190.0], 20.0, [0.15, 0.1, 0.08, 1.0], "", Align::Left);
            ctx.ui().paragraph(body, [w - 500.0, 230.0], 400.0, 17.0, [0.12, 0.1, 0.08, 1.0], "");
        }
        let close = self.t("[J] / [ESC]  close", "[J] / [ESC]  закрыть");
        ctx.ui().text(&close, [80.0, 680.0], 15.0, DIM, "", Align::Left);
        if mode_time(ctx) > 0.15 && (ctx.action_pressed("Journal") || ctx.key_pressed(Key::Escape)) {
            self.resume(ctx);
        }
    }

    /// The power panel: three switches (cold 3, hall 2, archive 2), at most 4 in total.
    fn panel_ui(&mut self, ctx: &mut Ctx) {
        let w = ctx.ui_width();
        let s = story(ctx);
        let names = [self.t("COLD  3", "ХОЛОД  3"), self.t("HALL  2", "ЗАЛ  2"), self.t("ARCHIVE  2", "АРХИВ  2")];
        let cost = [3, 2, 2];
        let title = self.t("SWITCH PANEL  -  MAX LOAD 4", "ЩИТОК  -  НАГРУЗКА НЕ БОЛЕЕ 4");
        let hint = self.t("For the archive: COLD off, HALL on, ARCHIVE on.", "Для доступа к архиву: ХОЛОД — выкл; ЗАЛ — вкл; АРХИВ — вкл.");
        let close = self.t("CLOSE", "ЗАКРЫТЬ");
        let ui = ctx.ui();
        ui.panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, 0.7], 0.0);
        ui.frame([w * 0.5 - 330.0, 150.0, 660.0, 420.0], [0.08, 0.1, 0.09, 0.97], [0.5, 0.48, 0.36, 1.0], 6.0);
        ui.text(&title, [w * 0.5, 175.0], 24.0, PAPER, FONT, Align::Center);
        let load: i32 = (0..3).filter(|i| s.power[*i]).map(|i| cost[i]).sum();
        let mut clicked = None;
        for i in 0..3 {
            let x = w * 0.5 - 270.0 + i as f32 * 190.0;
            ui.text(&names[i], [x + 75.0, 240.0], 20.0, PAPER, FONT, Align::Center);
            let on = s.power[i];
            ui.panel([x + 45.0, 280.0, 60.0, 130.0], [0.03, 0.03, 0.03, 1.0], 4.0);
            ui.panel([x + 50.0, if on { 285.0 } else { 345.0 }, 50.0, 60.0], if on { [0.75, 0.6, 0.2, 1.0] } else { [0.35, 0.33, 0.3, 1.0] }, 3.0);
            ui.text(if on { "ON" } else { "OFF" }, [x + 75.0, 420.0], 18.0, if on { [0.9, 0.8, 0.3, 1.0] } else { DIM }, FONT, Align::Center);
            if ui.button(&format!("sw{i}"), [x + 40.0, 275.0, 70.0, 140.0], "") {
                clicked = Some(i);
            }
        }
        ui.text(&format!("LOAD {load} / 4"), [w * 0.5, 470.0], 22.0, if load > 4 { RED } else { PAPER }, FONT, Align::Center);
        ui.paragraph(&hint, [w * 0.5 - 300.0, 505.0], 600.0, 15.0, DIM, "");
        let close_clicked = ui.button("panel_close", [w * 0.5 - 70.0, 530.0, 140.0, 34.0], &close);
        if let Some(i) = clicked {
            let _ = ctx.emit("ls.panel", serde_json::json!({ "switch": i }));
        }
        if mode_time(ctx) > 0.15 && (close_clicked || ctx.key_pressed(Key::Escape) || ctx.key_pressed(Key::E) && !ctx.mouse_down(MouseButton::Left)) {
            self.resume(ctx);
        }
    }

    fn intro_ui(&mut self, ctx: &mut Ctx) {
        let w = ctx.ui_width();
        let t = mode_time(ctx);
        let a = (t / 1.5).min(1.0) * (1.0 - ((t - 4.5) / 1.5).clamp(0.0, 1.0));
        let title = self.t("14 NOVEMBER 1998", "14 НОЯБРЯ 1998");
        let sub = self.t("Pereval roadside diner. Closed.", "Придорожная столовая «Перевал». Закрыта.");
        let ui = ctx.ui();
        ui.panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, 1.0 - (t - 5.0).clamp(0.0, 1.0)], 0.0);
        ui.text(&title, [w * 0.5, 320.0], 34.0, [PAPER[0], PAPER[1], PAPER[2], a], FONT, Align::Center);
        ui.text(&sub, [w * 0.5, 370.0], 17.0, [DIM[0], DIM[1], DIM[2], a], "", Align::Center);
        self.fade = 0.0;
    }

    fn dead_ui(&mut self, ctx: &mut Ctx) {
        let t = mode_time(ctx);
        if t > 1.2 {
            let w = ctx.ui_width();
            let a = ((t - 1.2) / 0.8).min(1.0);
            let text = self.t("VISITORS: 0", "ПОСЕТИТЕЛЕЙ: 0");
            let ui = ctx.ui();
            ui.panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, a], 0.0);
            ui.text(&text, [w * 0.5, 330.0], 36.0, [RED[0], RED[1], RED[2], a], FONT, Align::Center);
        }
    }

    fn ending_ui(&mut self, ctx: &mut Ctx) {
        let t = mode_time(ctx);
        if t > 9.0 {
            let w = ctx.ui_width();
            let a = ((t - 9.0) / 2.0).min(1.0);
            let names = if self.set.russian { "Павел Зорин. Вера Седова. Нина Седова." } else { "Pavel Zorin. Vera Sedova. Nina Sedova." };
            let line = self.t("Their names were entered in the case file on 14 November 1998.", "Их имена были внесены в дело 14 ноября 1998 года.");
            let ui = ctx.ui();
            ui.panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, a], 0.0);
            ui.text(names, [w * 0.5, 300.0], 26.0, [PAPER[0], PAPER[1], PAPER[2], a], "", Align::Center);
            ui.text(&line, [w * 0.5, 350.0], 18.0, [DIM[0], DIM[1], DIM[2], a], "", Align::Center);
        }
    }

    fn credits_ui(&mut self, ctx: &mut Ctx) {
        let t = mode_time(ctx);
        let w = ctx.ui_width();
        ctx.ui().panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, 1.0], 0.0);
        let y = 720.0 - t * 40.0;
        ctx.ui().image("ui/branding/last_service_logo.png", [w * 0.5 - 180.0, y, 360.0, 240.0], [1.0; 4]);
        self.credits_text(ctx, w * 0.5 - 260.0, y + 280.0);
        let skip = self.t("[SPACE] skip", "[ПРОБЕЛ] пропустить");
        ctx.ui().text(&skip, [w - 30.0, 680.0], 14.0, DIM, "", Align::Right);
        if (t > 5.0 && (ctx.key_pressed(Key::Space) || ctx.key_pressed(Key::Escape))) || t > 45.0 {
            ctx.play_sound("audio/sfx/sfx_plate.wav", 0.4);
            self.menu(ctx, "credits_done");
            set_mode(ctx, Mode::Menu);
        }
    }
}

impl Script for GameUI {
    fn start(&mut self, ctx: &mut Ctx) {
        self.load_settings(ctx);
        self.apply_settings(ctx);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt().min(0.1);
        for sig in ctx.signals("ls.say") {
            let p = &sig.payload;
            let text = |k: &str| p[k].as_str().unwrap_or("").to_string();
            self.subs.push_back(Line { who: text("who"), en: text("en"), ru: text("ru"), time: p["time"].as_f64().unwrap_or(3.0) as f32 });
        }
        for sig in ctx.signals("ls.msg") {
            let p = &sig.payload;
            self.message = (p["en"].as_str().unwrap_or("").to_string(), p["ru"].as_str().unwrap_or("").to_string(), 3.5);
        }
        for sig in ctx.signals("ls.fade") {
            self.fade = sig.payload["value"].as_f64().unwrap_or(1.0) as f32;
        }
        // The autotest may change settings (it turns the chase off); keep the published ones.
        let published = settings(ctx);
        if published != self.set {
            self.set = published;
        }
        let m = mode(ctx);
        if self.last_mode != Some(m) {
            if m == Mode::Menu {
                self.page = Page::Main;
                self.subs.clear();
                self.has_save = ctx.has_save("autosave") || ctx.has_save("manual");
            }
            if m == Mode::Paused {
                self.page = Page::Main;
            }
            if m == Mode::Journal {
                self.journal_note = None;
            }
            self.last_mode = Some(m);
        }
        if m == Mode::Playing {
            if ctx.action_pressed("Pause") {
                set_mode(ctx, Mode::Paused);
            } else if ctx.action_pressed("Journal") {
                set_mode(ctx, Mode::Journal);
            }
        }
        match mode(ctx) {
            Mode::Menu => self.menu_ui(ctx),
            Mode::Intro => self.intro_ui(ctx),
            Mode::Paused => self.pause_ui(ctx),
            Mode::Reading => self.reading_ui(ctx),
            Mode::Journal => self.journal_ui(ctx),
            Mode::Panel => self.panel_ui(ctx),
            Mode::Dead => self.dead_ui(ctx),
            Mode::Ending => self.ending_ui(ctx),
            Mode::Credits => self.credits_ui(ctx),
            Mode::Playing => {}
        }
        let m = mode(ctx);
        if m != Mode::Menu && m != Mode::Credits {
            self.hud(ctx, dt);
        }
        self.fade += (0.0 - self.fade) * (dt * 2.5).min(1.0);
        if self.fade > 0.002 {
            let w = ctx.ui_width();
            ctx.ui().panel([0.0, 0.0, w, 720.0], [0.0, 0.0, 0.0, self.fade.clamp(0.0, 1.0)], 0.0);
        }
    }
}

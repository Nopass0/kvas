//! The story of LAST SERVICE. Lives on the "Game" object and owns the story state: what each
//! thing offers and does when used (`ls.look` → `ls.offer`, `ls.act`), where things are and which
//! doors are locked, the power lines, checkpoints and saves, music, the intro, death, the ending.
//!
//! Other scripts do the rest on their own objects: `PlayerController`, `MouseLook`, `Interactor`,
//! `Inventory`, `Flashlight` (player), `Door`, `FlickerLight`, `Spinner` (world), `MonsterAi`
//! (the Counter), `GameUI` (menus and HUD).

use crate::s_shared::*;
use kvas_engine::prelude::*;
use std::collections::HashMap;

/// Doors, the story flag that unlocks each, and its name for prompts.
const DOORS: [&str; 6] = ["Door kitchen", "Door office", "Door storage", "Door ring", "Door emergency", "Door side yard"];

pub struct GameDirector {
    s: Story,
    published: Option<Story>,
    names: HashMap<String, Entity>,
    triggers: HashMap<String, Collider>,
    music: Option<SoundId>,
    music_clip: String,
    one_shots: Vec<String>,
    last_mode: Option<Mode>,
    panel_try: f32,
    /// Automated walk-through: step index, phase, wait.
    auto: Option<(usize, u8, f32)>,
}

impl Default for GameDirector {
    fn default() -> Self {
        Self {
            s: Story::default(),
            published: None,
            names: HashMap::new(),
            triggers: HashMap::new(),
            music: None,
            music_clip: String::new(),
            one_shots: Vec::new(),
            last_mode: None,
            panel_try: 0.0,
            auto: None,
        }
    }
}

impl GameDirector {
    fn e(&self, name: &str) -> Option<Entity> {
        self.names.get(name).copied()
    }

    fn t(&self, ctx: &Ctx, en: &str, ru: &str) -> String {
        tr(ctx, en, ru)
    }

    fn once(&mut self, id: &str) -> bool {
        if self.one_shots.iter().any(|o| o == id) {
            return false;
        }
        self.one_shots.push(id.to_string());
        true
    }

    fn set_music(&mut self, ctx: &mut Ctx, clip: &str, volume: f32) {
        if self.music_clip == clip {
            return;
        }
        if let Some(id) = self.music.take() {
            ctx.stop_sound(id);
        }
        self.music_clip = clip.to_string();
        if !clip.is_empty() {
            self.music = Some(ctx.play_music(clip, volume));
        }
    }

    fn monster(&self, ctx: &mut Ctx, cmd: &str) {
        let _ = ctx.emit("ls.monster", serde_json::json!({ "cmd": cmd }));
    }

    fn publish(&mut self, ctx: &mut Ctx) {
        if self.published.as_ref() != Some(&self.s) {
            ctx.set_global(STORY, serde_json::to_value(&self.s).unwrap_or_default());
            self.published = Some(self.s.clone());
        }
    }

    // ------------------------------------------------------------ saves
    fn checkpoint(&mut self, ctx: &mut Ctx, n: u8) {
        self.s.checkpoint = self.s.checkpoint.max(n);
        self.write_save(ctx, "autosave");
        message(ctx, "Progress saved", "Прогресс сохранён");
    }

    fn write_save(&mut self, ctx: &mut Ctx, slot: &str) {
        if let Some(p) = self.e("Lida").and_then(|e| ctx.position_of(e)) {
            self.s.pos = [p.x, p.y, p.z];
        }
        self.s.yaw = self.e("Eyes").and_then(|e| ctx.get_of::<Transform>(e)).map_or(0.0, |t| t.rotation.y);
        let title = objective(&self.s, settings(ctx).russian);
        if let Ok(v) = serde_json::to_value(&self.s) {
            let _ = ctx.save_game(slot, &title, &v);
        }
    }

    fn newest_save(&self, ctx: &Ctx) -> Option<Story> {
        let slot = ctx.save_slots().into_iter().find(|i| i.slot == "autosave" || i.slot == "manual")?;
        serde_json::from_value(ctx.load_game(&slot.slot)?).ok()
    }

    // ------------------------------------------------------------ the world from the story
    /// Locks or unlocks a door; a door that gets unlocked swings open (the story opened it).
    fn door(&mut self, ctx: &mut Ctx, name: &str, unlocked: bool, fresh: bool) {
        let Some(e) = self.e(name) else { return };
        let was_locked = ctx.var_bool(e, "Door", "locked").unwrap_or(true);
        if !unlocked {
            ctx.set_var(e, "Door", "locked", true.into());
            ctx.set_var(e, "Door", "open", false.into());
        } else if was_locked || fresh {
            ctx.set_var(e, "Door", "locked", false.into());
            ctx.set_var(e, "Door", "open", true.into());
        }
    }

    /// Shows or hides a thing; a hidden thing cannot be looked at either.
    fn show(&mut self, ctx: &mut Ctx, name: &str, on: bool) {
        if let Some(e) = self.e(name) {
            ctx.set_visible(e, on);
        }
        self.set_use(ctx, name, on);
    }

    /// Whether the player's look can find the thing (things put on the table cannot).
    fn set_use(&mut self, ctx: &mut Ctx, name: &str, on: bool) {
        let key = format!("{name}#use");
        let Some(t) = self.e(&key) else { return };
        if let Some(c) = ctx.get_of::<Collider>(t).cloned() {
            self.triggers.insert(key.clone(), c);
        }
        match (on, ctx.get_of::<Collider>(t).is_some()) {
            (false, true) => {
                ctx.remove::<Collider>(t);
            }
            (true, false) => {
                if let Some(c) = self.triggers.get(&key).cloned() {
                    ctx.insert(t, c);
                }
            }
            _ => {}
        }
    }

    fn place(&self, ctx: &mut Ctx, name: &str, at: Vec3) {
        if let Some(t) = self.e(name).and_then(|e| ctx.get_mut_of::<Transform>(e)) {
            t.position = at;
        }
    }

    fn seat(&self, ctx: &Ctx, i: usize) -> Vec3 {
        let name = ["I_seat_driver", "I_seat_teacher", "I_seat_girl", "I_seat_fourth"][i];
        self.e(name).and_then(|e| ctx.position_of(e)).unwrap_or(b(8.3, 4.5, 0.78))
    }

    fn lamp_power(&self, ctx: &mut Ctx, tag: &str, power: f32) {
        for e in ctx.tagged(tag) {
            ctx.set_var(e, "FlickerLight", "power", power.into());
        }
    }

    /// Puts every object where the story says it is (new game, load, after each change).
    fn apply_world(&mut self, ctx: &mut Ctx, fresh_doors: bool) {
        let s = self.s.clone();
        // Kitchen, office-block back door (card), storage → office, corridor → storage, emergency
        // exit, hall → yard.
        let open = [s.talked_anna, s.office_open, s.office_open, true, s.exit_open, s.talked_anna];
        for (name, unlocked) in DOORS.iter().zip(open) {
            self.door(ctx, name, unlocked, fresh_doors);
        }
        self.show(ctx, "I_flashlight", !s.flashlight);
        for i in 0..3 {
            let name = format!("I_plate_{i}");
            self.show(ctx, &name, s.plates[i] == 0 || s.plates[i] == 3);
            if s.plates[i] == 3 {
                let at = self.seat(ctx, i) + Vec3::Y * 0.01;
                self.place(ctx, &name, at);
            }
            if s.plates[i] >= 2 {
                if let Some(m) = self.e(&name).and_then(|e| ctx.get_mut_of::<MeshRenderer>(e)) {
                    m.mesh = "models/kitchen/clean_plate.glb".into();
                }
            }
            let meal = ["I_meal_water", "I_meal_bread", "I_meal_sweet"][i];
            self.show(ctx, meal, s.meals[i] != 1);
            if s.meals[i] == 2 {
                let toward = (b(8.3, 4.5, 0.0) - self.seat(ctx, i)).with_y(0.0).normalize_or_zero();
                let at = self.seat(ctx, i) + toward * 0.05 + Vec3::Y * 0.03 + Vec3::new(0.08, 0.0, 0.0);
                self.place(ctx, meal, at);
            }
        }
        self.show(ctx, "I_fuse_box", true);
        self.show(ctx, "I_keycard", s.keycard == 0);
        self.show(ctx, "I_crowbar", !s.crowbar);
        self.show(ctx, "Locker board", !s.locker_open);
        self.show(ctx, "I_key", s.locker_open && !s.key);
        self.show(ctx, "I_vera_book", !s.vera_book || s.placed[1]);
        self.show(ctx, "I_doll", !s.doll || s.placed[2]);
        self.show(ctx, "I_padlock", !s.padlock_off);
        self.show(ctx, "I_plate_fourth", s.corrected && s.fourth != 1);
        if s.fourth == 2 {
            let at = self.seat(ctx, 3);
            self.place(ctx, "I_plate_fourth", at);
        }
        // The names' things on the table once placed.
        if s.placed[0] {
            self.show(ctx, "I_key", true);
            let at = self.seat(ctx, 0) + Vec3::new(-0.12, 0.02, 0.1);
            self.place(ctx, "I_key", at);
        }
        if s.placed[1] {
            let at = self.seat(ctx, 1) + Vec3::new(0.1, 0.02, 0.1);
            self.place(ctx, "I_vera_book", at);
        }
        if s.placed[2] {
            let at = self.seat(ctx, 2) + Vec3::new(0.12, 0.0, 0.0);
            self.place(ctx, "I_doll", at);
        }
        // Things on the table stay there.
        for i in 0..3 {
            if s.plates[i] == 3 {
                self.set_use(ctx, &format!("I_plate_{i}"), false);
            }
            if s.meals[i] == 2 {
                self.set_use(ctx, ["I_meal_water", "I_meal_bread", "I_meal_sweet"][i], false);
            }
        }
        if s.placed[0] || s.key {
            self.set_use(ctx, "I_key", false);
        }
        if s.vera_book {
            self.set_use(ctx, "I_vera_book", false);
        }
        if s.doll {
            self.set_use(ctx, "I_doll", false);
        }
        if s.fourth == 2 {
            self.set_use(ctx, "I_plate_fourth", false);
        }
        self.set_use(ctx, "I_yard_locker", !s.locker_open);
        self.set_use(ctx, "I_fuse_box", s.fuse == 0);
        self.set_use(ctx, "I_padlock", !s.exit_open);
        // Anna stands behind the counter until the names are found; she sits at the fourth place at the end.
        let anna_here = !s.all_names() || s.corrected;
        self.show(ctx, "Anna", anna_here && !s.finished);
        self.set_use(ctx, "I_anna", anna_here && !s.finished);
        if let Some(anna) = self.e("Anna") {
            if s.corrected {
                // At the fourth place, facing the table, sitting.
                self.place(ctx, "Anna", b(8.3, 3.55, -0.4));
                if let Some(t) = ctx.get_mut_of::<Transform>(anna) {
                    t.rotation.y = 180.0;
                }
                ctx.set_var(anna, "Npc", "work_clip", "Sit".into());
            } else {
                ctx.set_var(anna, "Npc", "work_clip", "Wipe".into());
            }
        }
        self.show(ctx, "Pavel", s.ledger_read && !s.key);
        // Power: the hall line dims the hall, the archive line feeds the office lamp.
        self.lamp_power(ctx, "hall_power", if s.power_ok && !s.power[1] { 0.3 } else { 1.0 });
        self.lamp_power(ctx, "archive_power", if s.power[2] && s.fuse == 2 { 1.0 } else { 0.0 });
        self.publish(ctx);
    }

    // ------------------------------------------------------------ starting and ending play
    fn begin(&mut self, ctx: &mut Ctx, story: Story, fresh: bool) {
        self.s = story;
        self.one_shots.clear();
        let start = if fresh || self.s.pos == [0.0; 3] { b(6.0, -8.0, 1.0) } else { Vec3::from(self.s.pos) };
        if let Some(p) = self.e("Lida") {
            ctx.teleport_of(p, start, true);
        }
        if let Some(t) = self.e("Eyes").and_then(|e| ctx.get_mut_of::<Transform>(e)) {
            t.rotation = Vec3::new(0.0, if fresh { 0.0 } else { self.s.yaw }, 0.0);
        }
        self.apply_world(ctx, true);
        self.monster(ctx, "reset");
        ctx.set_active_camera_named("Eyes");
        let _ = ctx.emit("ls.fade", serde_json::json!({ "value": 1.0 }));
        set_mode(ctx, if fresh { Mode::Intro } else { Mode::Playing });
        self.set_music(ctx, "", 0.0);
    }

    fn to_menu(&mut self, ctx: &mut Ctx) {
        set_mode(ctx, Mode::Menu);
        ctx.set_active_camera_named("Menu camera");
        self.set_music(ctx, "audio/sfx/music_menu_drone.wav", 0.55);
        self.monster(ctx, "hide");
        ctx.set_time_of_day(23.0);
        let _ = ctx.emit("ls.fade", serde_json::json!({ "value": 1.0 }));
    }

    // ------------------------------------------------------------ what things offer and do
    /// What looking at `id` offers right now: the prompt, or `None`.
    fn offer(&self, ctx: &Ctx, id: &str) -> Option<String> {
        let s = &self.s;
        let hand_dirty = s.plates.iter().any(|p| *p == 1);
        let hand_clean = s.plates.iter().any(|p| *p == 2);
        let hand_meal = s.meals.iter().position(|m| *m == 1);
        let all3 = s.all_names();
        let t = |en: &str, ru: &str| Some(self.t(ctx, en, ru));
        match id {
            "I_flashlight" if !s.flashlight => t("Take the flashlight", "Взять фонарь"),
            "I_note_N01" => t("Read the note", "Прочитать записку"),
            "I_anna" if !s.talked_anna => t("Mum?", "Мама?"),
            "I_anna" if s.corrected && s.fourth == 0 => t("Anna", "Анна"),
            "I_note_N02" if s.talked_anna => t("Read the order", "Прочитать заказ"),
            p if p.starts_with("I_plate_") && p != "I_plate_fourth" => {
                let i: usize = p[8..].parse().ok()?;
                (s.plates[i] == 0).then(|| self.t(ctx, "Take a dirty plate", "Взять грязную тарелку"))
            }
            "I_sink" if hand_dirty => t("Hold E to wash the plates", "Удерживать E — вымыть тарелки"),
            "I_meal_water" | "I_meal_bread" | "I_meal_sweet" if s.service_done => None,
            "I_meal_water" if s.meals[0] == 0 && s.plates.iter().all(|p| *p == 3) && hand_meal.is_none() => t("Take the glass of water", "Взять стакан воды"),
            "I_meal_bread" if s.meals[1] == 0 && s.plates.iter().all(|p| *p == 3) && hand_meal.is_none() => t("Take the bread", "Взять хлеб"),
            "I_meal_sweet" if s.meals[2] == 0 && s.plates.iter().all(|p| *p == 3) && hand_meal.is_none() => t("Take the chocolate", "Взять шоколад"),
            "I_seat_driver" | "I_seat_teacher" | "I_seat_girl" => {
                let i = ["I_seat_driver", "I_seat_teacher", "I_seat_girl"].iter().position(|n| *n == id)?;
                let role = [self.t(ctx, "DRIVER (road)", "ВОДИТЕЛЬ (дорога)"), self.t(ctx, "TEACHER (line)", "УЧИТЕЛЬНИЦА (строка)"), self.t(ctx, "GIRL (ribbon)", "ДЕВОЧКА (лента)")][i].clone();
                if hand_clean && s.plates[i] != 3 {
                    Some(format!("{} — {role}", self.t(ctx, "Put a clean plate", "Поставить чистую тарелку")))
                } else if hand_meal.is_some() && s.plates[i] == 3 {
                    Some(format!("{} — {role}", self.t(ctx, "Serve", "Подать")))
                } else if s.exit_open && all3 && !s.placed[i] {
                    let what = [self.t(ctx, "the key", "ключ"), self.t(ctx, "the exercise book", "тетрадь"), self.t(ctx, "the doll", "куклу")][i].clone();
                    Some(format!("{} {what} — {role}", self.t(ctx, "Put", "Положить")))
                } else {
                    Some(role)
                }
            }
            "I_seat_fourth" if s.fourth == 1 => t("Set the fourth place", "Поставить четвёртый прибор"),
            "I_seat_fourth" if s.placed.iter().all(|p| *p) && !s.corrected => t("Hold E: write \"Visitors: 3. Exit open.\"", "Удерживать E: вписать «Посетителей: 3. Выход открыт»"),
            "I_fuse_box" if s.service_done && s.fuse == 0 => t("Take the fuse (yellow box)", "Взять предохранитель (жёлтая коробка)"),
            "I_panel" if s.fuse == 1 => t("Install the fuse", "Вставить предохранитель"),
            "I_panel" if s.fuse == 2 => t("Open the switch panel", "Открыть щиток"),
            "I_panel" => t("Switch panel. It needs a fuse.", "Щиток. Нужен предохранитель."),
            "I_keycard" if s.power_ok && s.keycard == 0 => t("Take the keycard", "Взять карту"),
            "I_keycard" if s.keycard == 0 => t("The keycard is behind the grille. No power.", "Карта за решёткой. Нет питания."),
            "I_reader" if s.keycard == 1 && !s.office_open => t("Use the keycard", "Приложить карту"),
            "I_reader" if !s.office_open => t("Card reader. Locked.", "Считыватель. Закрыто."),
            "I_ledger" if s.office_open && !s.ledger_read => t("Read the ledger", "Прочитать журнал"),
            "I_ledger" if all3 && !s.confession => t("Turn the page", "Перевернуть страницу"),
            "I_ledger" if s.ledger_read => t("The ledger", "Журнал"),
            "I_vera_book" if s.ledger_read && !s.vera_book => t("Take the exercise book", "Взять тетрадь"),
            "I_crowbar" if !s.crowbar => t("Take the crowbar", "Взять лом"),
            "I_yard_locker" if !s.locker_open && s.crowbar && s.ledger_read => t("Pry off the board", "Снять доску ломом"),
            "I_yard_locker" if !s.locker_open => t("The driver's locker. A board is nailed across it.", "Шкафчик водителя. Заколочен доской."),
            "I_key" if s.locker_open && !s.key => t("Take the key and the route sheet", "Взять ключ и путевой лист"),
            "I_doll" if !s.doll && s.key => t("Open the cupboard with the key", "Открыть шкаф ключом"),
            "I_doll" if !s.doll && !s.nina_label => t("Read the label through the glass", "Прочитать бирку через стекло"),
            "I_doll" if !s.doll => t("Locked. Lost property.", "Заперто. Забытые вещи."),
            "I_padlock" if s.confession && !s.padlock_off && s.key => t("Unlock the padlock", "Снять замок"),
            "I_padlock" if s.padlock_off && !s.exit_open => t("Hold E to pull the latch", "Удерживать E — отодвинуть засов"),
            "I_padlock" if !s.exit_open => t("Read the sign", "Прочитать табличку"),
            "I_plate_fourth" if s.corrected && s.fourth == 0 => t("Take the clean plate", "Взять чистую тарелку"),
            "I_bus_stop" if s.anna_ack => t("Wait for the first bus", "Дождаться первого автобуса"),
            _ => None,
        }
    }

    /// Things that need E held for a moment instead of a press.
    fn hold_time(&self, id: &str) -> f32 {
        match id {
            "I_sink" => 1.2,
            "I_padlock" if self.s.padlock_off => 1.5,
            "I_seat_fourth" if self.s.placed.iter().all(|p| *p) && !self.s.corrected => 2.0,
            _ => 0.0,
        }
    }

    fn read(&mut self, ctx: &mut Ctx, id: &str) {
        if !self.s.notes.iter().any(|n| n == id) {
            self.s.notes.push(id.to_string());
        }
        ctx.set_global(READING, id.into());
        set_mode(ctx, Mode::Reading);
        sfx(ctx, "sfx_ui", 0.6);
    }

    fn act(&mut self, ctx: &mut Ctx, id: &str) {
        match id {
            "I_flashlight" => {
                self.s.flashlight = true;
                message(ctx, "F - flashlight", "F — фонарь");
                sfx(ctx, "sfx_latch", 0.5);
            }
            "I_note_N01" => self.read(ctx, "N01"),
            "I_note_N02" => self.read(ctx, "N02"),
            "I_anna" if !self.s.talked_anna => {
                self.s.talked_anna = true;
                say(ctx, "Anna", "You're early. I haven't closed yet.", "Ты рано. Я ещё не закрылась.", 3.5);
                say(ctx, "Lida", "Mum?..", "Мам?..", 2.0);
                say(ctx, "Anna", "Dishes first. We'll talk after the shift.", "Посуду сначала. Поговорим после смены.", 4.0);
                sfx(ctx, "sfx_knock", 0.4);
                self.checkpoint(ctx, 0);
            }
            "I_anna" => say(ctx, "Anna", "There is no room for me in the ledger.", "В журнале нет места для меня.", 4.0),
            p if p.starts_with("I_plate_") && p != "I_plate_fourth" => {
                let i: usize = p[8..].parse().unwrap_or(0);
                self.s.plates[i] = 1;
                sfx(ctx, "sfx_plate", 0.6);
            }
            "I_sink" => {
                for p in self.s.plates.iter_mut() {
                    if *p == 1 {
                        *p = 2;
                    }
                }
                sfx(ctx, "sfx_plate", 0.8);
                message(ctx, "The plates are clean", "Тарелки чистые");
            }
            "I_meal_water" | "I_meal_bread" | "I_meal_sweet" => {
                let i = ["I_meal_water", "I_meal_bread", "I_meal_sweet"].iter().position(|n| *n == id).unwrap_or(0);
                self.s.meals[i] = 1;
            }
            "I_seat_driver" | "I_seat_teacher" | "I_seat_girl" => {
                let i = ["I_seat_driver", "I_seat_teacher", "I_seat_girl"].iter().position(|n| *n == id).unwrap_or(0);
                if let Some(p) = self.s.plates.iter().position(|p| *p == 2).filter(|_| self.s.plates[i] != 3) {
                    // Plates are interchangeable; plate `i` always sits at seat `i`.
                    self.s.plates[p] = self.s.plates[i];
                    self.s.plates[i] = 3;
                    sfx(ctx, "sfx_plate", 0.7);
                } else if let Some(m) = self.s.meals.iter().position(|m| *m == 1) {
                    if m == i {
                        self.s.meals[m] = 2;
                        sfx(ctx, "sfx_plate", 0.5);
                        let served = self.s.meals.iter().filter(|m| **m == 2).count();
                        match served {
                            1 => {
                                message(ctx, "Somewhere at the empty table, someone breathes out.", "У пустого стола кто-то выдохнул.");
                                sfx(ctx, "amb_room_hum", 0.2);
                            }
                            2 => say(ctx, "Lida", "There's someone at the window...", "В окне кто-то стоит...", 3.0),
                            _ => self.service_complete(ctx),
                        }
                    } else {
                        let (en, ru) = [("The driver asked for water.", "Водителю просили воду."), ("The teacher asked for bread.", "Учительнице просили хлеб."), ("The girl asked for something sweet.", "Девочке просили сладкое.")][m];
                        say(ctx, "Lida", en, ru, 3.0);
                    }
                } else if self.s.exit_open && self.s.all_names() && !self.s.placed[i] {
                    self.s.placed[i] = true;
                    sfx(ctx, "sfx_plate", 0.5);
                    let (en, ru) = [("Pavel Zorin.", "Павел Зорин."), ("Vera Sedova.", "Вера Седова."), ("Nina Sedova.", "Нина Седова.")][i];
                    say(ctx, "Lida", en, ru, 2.5);
                }
            }
            "I_seat_fourth" if self.s.fourth == 1 => {
                self.s.fourth = 2;
                self.s.anna_ack = true;
                sfx(ctx, "sfx_plate", 0.6);
                say(ctx, "Lida", "Then stay. Not on duty. Be my mum.", "Тогда останься не дежурной. Побудь моей мамой.", 4.0);
                say(ctx, "Anna", "I'm sorry you found out like this.", "Прости, что ты узнала так.", 3.5);
                say(ctx, "Lida", "I can't forgive you for them. But no one will wait behind that door again.", "Я не могу простить за них. Но больше никто не будет ждать за дверью.", 5.0);
                self.checkpoint(ctx, 5);
                self.apply_world(ctx, false);
                self.monster(ctx, "reset");
                self.set_music(ctx, "audio/music/chase_2.m4a", 0.55);
            }
            "I_seat_fourth" => {
                self.s.corrected = true;
                sfx(ctx, "sfx_latch", 0.6);
                say(ctx, "Lida", "Visitors: 3. Exit open.", "Посетителей: 3. Выход открыт.", 3.0);
                say(ctx, "Lida", "Are you still on your shift?", "А ты всё ещё на смене?", 3.0);
                say(ctx, "Anna", "There is no room for me in the ledger.", "В журнале нет места для меня.", 4.0);
                message(ctx, "A clean fourth plate lies on the counter.", "На стойке лежит четвёртая чистая тарелка.");
                self.checkpoint(ctx, 5);
                self.apply_world(ctx, false);
                self.monster(ctx, "reset");
                self.set_music(ctx, "audio/music/nostalgic.m4a", 0.45);
            }
            "I_fuse_box" => {
                self.s.fuse = 1;
                sfx(ctx, "sfx_fuse", 0.6);
                message(ctx, "Fuse taken", "Предохранитель взят");
            }
            "I_panel" if self.s.fuse >= 1 => {
                if self.s.fuse == 1 {
                    self.s.fuse = 2;
                    sfx(ctx, "sfx_fuse", 0.8);
                }
                set_mode(ctx, Mode::Panel);
            }
            "I_keycard" if self.s.power_ok => {
                self.s.keycard = 1;
                sfx(ctx, "sfx_ui", 0.6);
            }
            "I_reader" if self.s.keycard == 1 => {
                self.s.office_open = true;
                sfx(ctx, "sfx_latch", 0.8);
            }
            "I_ledger" if !self.s.ledger_read && self.s.office_open => {
                self.s.ledger_read = true;
                self.read(ctx, "N03");
                say(ctx, "Lida", "Three dinners. Zero visitors.", "Три ужина. Ноль посетителей.", 3.5);
                self.checkpoint(ctx, 2);
            }
            "I_ledger" if self.s.all_names() && !self.s.confession => {
                self.s.confession = true;
                self.read(ctx, "N07");
                self.checkpoint(ctx, 3);
                self.apply_world(ctx, false);
                self.monster(ctx, "reset");
                self.monster(ctx, "warn");
                self.set_music(ctx, "audio/music/doom_approaches.m4a", 0.4);
            }
            "I_ledger" => self.read(ctx, "N03"),
            "I_vera_book" => {
                self.s.vera_book = true;
                self.read(ctx, "N05");
                self.evidence(ctx);
            }
            "I_crowbar" => {
                self.s.crowbar = true;
                sfx(ctx, "sfx_latch", 0.5);
            }
            "I_yard_locker" if self.s.crowbar && self.s.ledger_read => {
                self.s.locker_open = true;
                sfx(ctx, "sfx_knock", 0.8);
            }
            "I_key" if !self.s.key => {
                self.s.key = true;
                self.read(ctx, "N04");
                say(ctx, "Pavel", "I promised to get them there by morning.", "Я обещал привезти их до утра.", 4.0);
                self.evidence(ctx);
            }
            "I_doll" if self.s.key && !self.s.doll => {
                self.s.doll = true;
                self.s.nina_label = true;
                self.read(ctx, "N06");
                say(ctx, "Lida", "I had one just like it.", "Такая же была у меня.", 3.0);
                self.evidence(ctx);
            }
            "I_doll" if !self.s.nina_label => {
                self.s.nina_label = true;
                self.read(ctx, "N06");
            }
            "I_padlock" if self.s.confession && !self.s.padlock_off && self.s.key => {
                self.s.padlock_off = true;
                sfx(ctx, "sfx_latch", 0.8);
            }
            "I_padlock" if self.s.padlock_off && !self.s.exit_open => {
                self.s.exit_open = true;
                sfx(ctx, "sfx_latch", 1.0);
                say(ctx, "Anna", "I wrote zero. I thought if no one knew, no one would wait.", "Я написала ноль. Думала, если никто не узнает, никто не будет ждать.", 5.0);
                say(ctx, "Lida", "They were waiting for you.", "Они ждали тебя.", 2.5);
                say(ctx, "Anna", "I know.", "Знаю.", 2.0);
                self.checkpoint(ctx, 4);
            }
            "I_padlock" => self.read(ctx, "N08"),
            "I_plate_fourth" => {
                self.s.fourth = 1;
                sfx(ctx, "sfx_plate", 0.6);
            }
            "I_bus_stop" if self.s.anna_ack => {
                self.s.finished = true;
                set_mode(ctx, Mode::Ending);
                // The names are written; the Counter has nothing left to count.
                self.monster(ctx, "collapse");
                self.set_music(ctx, "audio/music/roll_credits.m4a", 0.6);
                sfx(ctx, "amb_dawn_engine", 0.8);
                ctx.set_setting("finished", true.into());
                let _ = ctx.storage().delete_save("autosave");
                let _ = ctx.storage().delete_save("manual");
            }
            _ => {}
        }
        self.apply_world(ctx, false);
    }

    fn service_complete(&mut self, ctx: &mut Ctx) {
        self.s.service_done = true;
        say(ctx, "Anna", "Now clear it away. There are no visitors.", "Теперь убери. Посетителей нет.", 3.5);
        say(ctx, "Lida", "But you asked me to set it.", "Но ты просила накрыть.", 3.0);
        say(ctx, "Anna", "Not in the ledger.", "В журнале нет.", 3.0);
        sfx(ctx, "sfx_counter_warning", 0.5);
        self.monster(ctx, "glimpse");
        self.checkpoint(ctx, 1);
    }

    fn evidence(&mut self, ctx: &mut Ctx) {
        sfx(ctx, "sfx_knock", 0.5);
        if self.s.all_names() {
            say(ctx, "Lida", "Mum? ...She's gone.", "Мам? ...Её нет.", 3.0);
            message(ctx, "Something changed in the office.", "В конторке что-то изменилось.");
            self.set_music(ctx, "audio/music/mystery.m4a", 0.4);
        }
        self.write_save(ctx, "autosave");
    }

    /// A switch of the power panel (three lines: cold 3, hall 2, archive 2; at most 4 in total).
    fn toggle_power(&mut self, ctx: &mut Ctx, i: usize) {
        let cost = [3, 2, 2];
        let mut next = self.s.power;
        next[i] = !next[i];
        let load: i32 = (0..3).filter(|k| next[*k]).map(|k| cost[k]).sum();
        if load > 4 {
            sfx(ctx, "sfx_fuse", 0.9);
            message(ctx, "Overload. The switch clicks back.", "Перегрузка. Рычаг щёлкнул обратно.");
            return;
        }
        self.s.power = next;
        sfx(ctx, "sfx_latch", 0.5);
        if !self.s.power[0] && self.once("fridge_knocks") {
            sfx(ctx, "sfx_knock", 0.9);
            say(ctx, "Lida", "Three knocks... from the fridge.", "Три удара... из холодильника.", 3.0);
        }
        if self.s.power == [false, true, true] && !self.s.power_ok {
            self.s.power_ok = true;
            message(ctx, "The reader at the office is lit.", "Считыватель у конторки загорелся.");
            self.checkpoint(ctx, 2);
        }
        self.apply_world(ctx, false);
    }

    // ------------------------------------------------------------ per mode
    fn play(&mut self, ctx: &mut Ctx, dt: f32) {
        self.s.play_time += dt;
        let p = self.e("Lida").and_then(|e| ctx.position_of(e)).unwrap_or(Vec3::ZERO);
        // Before the story lets her go, the road brings her back to the diner.
        if !self.s.anna_ack && self.s.talked_anna && (p.z > 13.0 || p.x > 20.0) {
            if let Some(l) = self.e("Lida") {
                ctx.teleport_of(l, b(6.0, -8.0, 1.0), true);
            }
            let _ = ctx.emit("ls.fade", serde_json::json!({ "value": 1.0 }));
            if self.once("road_loop") {
                say(ctx, "Lida", "I was just at the road.", "Я только что была у дороги.", 3.0);
            }
        }
        if !self.s.talked_anna {
            if let Some(a) = self.e("Anna").and_then(|e| ctx.position_of(e)) {
                if a.distance(p) < 3.2 && self.once("anna_near") {
                    say(ctx, "Anna", "You're early.", "Ты рано.", 2.5);
                }
            }
        }
        // The ring door opens from inside the storage room.
        if !self.s.ring_open && p.x > 7.0 && p.z < -11.2 && p.z > -15.0 && self.s.ledger_read {
            self.s.ring_open = true;
            self.apply_world(ctx, false);
        }
        if self.s.confession && ctx.global_f32(FEAR, 0.0) > 0.0 && self.once("hunt_line") {
            say(ctx, "Lida", "Something is walking in the kitchen.", "На кухне кто-то ходит.", 3.0);
        }
        // Music by situation.
        if self.s.confession && !self.s.corrected {
            let clip = if ctx.global_f32(FEAR, 0.0) > 0.6 { "audio/music/chase_2.m4a" } else { "audio/music/doom_approaches.m4a" };
            self.set_music(ctx, clip, 0.45);
        } else if !self.s.confession && self.s.talked_anna && !self.s.service_done {
            self.set_music(ctx, "audio/music/expecting.m4a", 0.3);
        } else if self.s.service_done && !self.s.confession && !self.s.all_names() {
            self.set_music(ctx, "audio/music/deep.m4a", 0.3);
        }
        // Answer what the player looks at.
        let look = ctx.global(LOOK).and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
        let offer = if look.is_empty() { None } else { self.offer(ctx, &look) };
        let value = match offer {
            Some(text) => serde_json::json!({ "id": look, "text": text, "hold": self.hold_time(&look) }),
            None => serde_json::Value::Null,
        };
        if ctx.global(OFFER) != Some(&value) {
            ctx.set_global(OFFER, value);
        }
        for sig in ctx.signals("ls.act") {
            let id = sig.payload["id"].as_str().unwrap_or("").to_string();
            // Only what is offered right now (the story may have moved on since the press).
            if self.offer(ctx, &id).is_some() {
                self.act(ctx, &id);
            }
        }
        if !ctx.signals("ls.caught").is_empty() {
            self.s.deaths += 1;
            ctx.play_sound("audio/music/jumpscare.m4a", 1.0);
            set_mode(ctx, Mode::Dead);
        }
    }

    fn dead(&mut self, ctx: &mut Ctx) {
        if mode_time(ctx) > 4.0 {
            let deaths = self.s.deaths;
            let story = ctx.load_game("autosave").and_then(|v| serde_json::from_value::<Story>(v).ok()).unwrap_or_default();
            self.begin(ctx, Story { deaths, ..story }, false);
            if deaths >= 2 && self.once("offer_easier") {
                message(ctx, "Too hard? OPTIONS > NO CHASE keeps the story without the hunt.", "Сложно? НАСТРОЙКИ > БЕЗ ПРЕСЛЕДОВАНИЯ — история без охоты.");
            }
        }
    }

    fn ending(&mut self, ctx: &mut Ctx) {
        let t = mode_time(ctx);
        // Dawn comes up over the road.
        let hour = 23.0 + (t / 8.0).min(1.0) * 7.5;
        ctx.set_time_of_day(if hour >= 24.0 { hour - 24.0 } else { hour });
        if let Some(l) = self.e("Moon").and_then(|m| ctx.get_mut_of::<Light>(m)) {
            l.intensity = 0.22 + (t / 8.0).min(1.0) * 0.9;
            l.color = [1.0, 0.82, 0.62, 1.0];
        }
        if t > 4.0 && self.once("window_off") {
            if let Some(e) = self.e("Hall lamp") {
                ctx.set_var(e, "FlickerLight", "power", 0.0.into());
            }
        }
        if t > 17.0 {
            set_mode(ctx, Mode::Credits);
        }
    }

    /// Plays the story by itself (see `autotest.rs`) and logs `AUTOTEST PASS` or the failing step.
    fn autotest(&mut self, ctx: &mut Ctx, dt: f32) {
        use crate::s_autotest::STEPS;
        if self.auto.is_none() {
            if mode(ctx) != Mode::Menu {
                return;
            }
            let quiet = Settings { no_chase: true, ..settings(ctx) };
            ctx.set_global(SETTINGS, serde_json::to_value(quiet).unwrap_or_default());
            self.begin(ctx, Story::default(), true);
            set_mode(ctx, Mode::Playing);
            self.auto = Some((0, 0, 0.0));
            ctx.log("AUTOTEST start");
            return;
        }
        let (i, phase, wait) = self.auto.unwrap_or_default();
        if i >= STEPS.len() {
            if mode(ctx) == Mode::Credits && wait >= 0.0 {
                ctx.log("AUTOTEST PASS");
                self.auto = Some((i, 0, -1.0));
            }
            if mode(ctx) == Mode::Ending {
                ctx.set_global(MODE_T, (mode_time(ctx) + dt * 4.0).into());
            }
            return;
        }
        if matches!(mode(ctx), Mode::Reading | Mode::Panel) {
            set_mode(ctx, Mode::Playing);
        }
        let (id, stand) = STEPS[i];
        // Look at the area as a player would: at eye height where the area is tall (a person
        // behind the counter), at its middle otherwise.
        let target = self.e(&format!("{id}#use")).and_then(|e| {
            let c = ctx.position_of(e)?;
            let half = ctx.get_of::<Collider>(e).map_or(0.0, |col| col.size.y * 0.5);
            Some(c.with_y(1.55f32.clamp(c.y - half * 0.8, c.y + half * 0.8)))
        });
        let (Some(lida), Some(eyes)) = (self.e("Lida"), self.e("Eyes")) else { return };
        match phase {
            0 => {
                ctx.teleport_of(lida, Vec3::new(stand[0], 1.0, stand[1]), true);
                if let Some(t) = target {
                    let d = t - Vec3::new(stand[0], 1.62, stand[1]);
                    if let Some(tr) = ctx.get_mut_of::<Transform>(eyes) {
                        tr.rotation.y = (-d.x).atan2(-d.z).to_degrees();
                        tr.rotation.x = d.y.atan2(d.with_y(0.0).length()).to_degrees();
                    }
                }
                self.auto = Some((i, 1, 0.0));
            }
            _ if wait < 0.25 => self.auto = Some((i, phase, wait + dt)),
            _ => {
                if id == "POWER" {
                    self.s.power = [false, true, true];
                    self.s.power_ok = true;
                    self.apply_world(ctx, false);
                } else {
                    let m = ctx.world_matrix_of(eyes);
                    let (eye, dir) = (m.w_axis.truncate(), -m.z_axis.truncate().normalize_or_zero());
                    let hit = look_at(ctx, eye, dir, 2.6, Some(lida));
                    let name = hit.map(|h| hierarchy::name_of(&ctx.scene.world, h).to_string()).unwrap_or_default();
                    if name != format!("{id}#use") {
                        let solid = ctx.raycast_filtered(eye, dir, 2.6, kvas_engine::physics::QueryOptions { exclude: Some(lida), ..Default::default() }).map(|h| (h.point, h.distance));
                        ctx.log(format!("AUTOTEST FAIL step {i} {id}: the ray hits '{name}' from {stand:?} (eye {eye}, dir {dir}, solid hit {solid:?}, area at {target:?})"));
                        self.auto = Some((STEPS.len() + 1, 0, -1.0));
                        return;
                    }
                    match self.offer(ctx, id) {
                        Some(text) => ctx.log(format!("AUTOTEST {i} {id}: {text}")),
                        None => {
                            ctx.log(format!("AUTOTEST FAIL step {i} {id}: nothing to do (story {:?})", self.s));
                            self.auto = Some((STEPS.len() + 1, 0, -1.0));
                            return;
                        }
                    }
                    self.act(ctx, id);
                    if matches!(mode(ctx), Mode::Reading | Mode::Panel) {
                        set_mode(ctx, Mode::Playing);
                    }
                }
                self.auto = Some((i + 1, 0, 0.0));
            }
        }
    }
}

impl Script for GameDirector {
    fn start(&mut self, ctx: &mut Ctx) {
        for e in ctx.scene.world.entities() {
            let name = hierarchy::name_of(&ctx.scene.world, e).to_string();
            if !name.is_empty() {
                self.names.entry(name).or_insert(e);
            }
        }
        self.s = Story::default();
        self.to_menu(ctx);
        self.apply_world(ctx, true);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt().min(0.1);
        ctx.set_global(MODE_T, (mode_time(ctx) + dt).into());
        if ctx.global_bool("ls_autotest", false) {
            self.autotest(ctx, dt);
        }
        // Requests from the menus.
        for sig in ctx.signals("ls.menu") {
            match sig.payload["cmd"].as_str().unwrap_or("") {
                "new" => self.begin(ctx, Story::default(), true),
                "continue" => {
                    if let Some(story) = self.newest_save(ctx) {
                        self.begin(ctx, story, false);
                    }
                }
                "save" => {
                    self.write_save(ctx, "manual");
                    message(ctx, "Saved", "Сохранено");
                }
                "quit_to_menu" => {
                    self.write_save(ctx, "manual");
                    self.to_menu(ctx);
                }
                "credits_done" => {
                    self.s = Story::default();
                    self.apply_world(ctx, true);
                    self.to_menu(ctx);
                }
                _ => {}
            }
        }
        for sig in ctx.signals("ls.panel") {
            if let Some(i) = sig.payload["switch"].as_u64() {
                self.toggle_power(ctx, (i as usize).min(2));
            }
        }
        let m = mode(ctx);
        match m {
            Mode::Intro => {
                if mode_time(ctx) > 1.0 && self.once("engine_pass") {
                    ctx.play_sound_panned("audio/sfx/amb_dawn_engine.wav", 0.6, 0.8);
                }
                if mode_time(ctx) > 6.0 || (mode_time(ctx) > 1.0 && ctx.key_pressed(Key::Space)) {
                    set_mode(ctx, Mode::Playing);
                }
            }
            Mode::Playing => self.play(ctx, dt),
            Mode::Panel => {
                self.panel_try += dt;
                if !self.s.power_ok && self.panel_try > 90.0 && self.once("panel_hint") {
                    say(ctx, "Lida", "Cold off. Hall and archive on. That's four.", "Холод выключить. Зал и архив включить. Это четыре.", 4.0);
                }
            }
            Mode::Dead => self.dead(ctx),
            Mode::Ending => self.ending(ctx),
            _ => {}
        }
        // The mouse belongs to the game while playing and to the menus otherwise.
        let m = mode(ctx);
        if self.last_mode != Some(m) {
            self.last_mode = Some(m);
            ctx.lock_cursor(matches!(m, Mode::Playing | Mode::Intro));
        }
        self.publish(ctx);
    }
}

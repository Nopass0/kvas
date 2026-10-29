//! What the scripts of LAST SERVICE share. This file has no script in it, so it is always
//! compiled (see `build.rs`).
//!
//! The scripts live on their own objects and talk in two ways:
//! * **state** in `ctx.global` (one owner writes, everyone reads): the game mode, the story,
//!   the settings, what the player looks at, the fear level;
//! * **signals** for events: `ls.act` (use a thing), `ls.say` / `ls.msg` (subtitles and
//!   messages), `use` / `open` / `close` to a door, `ls.monster` commands, `ls.caught`.

use kvas_engine::prelude::*;
use serde::{Deserialize, Serialize};

pub const FONT: &str = "ui/font/LastServiceTerminal.ttf";
pub const PAPER: [f32; 4] = [0.86, 0.82, 0.68, 1.0];
pub const DIM: [f32; 4] = [0.62, 0.6, 0.52, 1.0];
pub const RED: [f32; 4] = [0.85, 0.16, 0.12, 1.0];

/// Global keys.
pub const MODE: &str = "ls.mode";
pub const MODE_T: &str = "ls.mode_t";
pub const STORY: &str = "ls.story";
pub const SETTINGS: &str = "ls.settings";
pub const LOOK: &str = "ls.look";
pub const OFFER: &str = "ls.offer";
pub const FEAR: &str = "ls.fear";
pub const FLASH_ON: &str = "ls.flash_on";
pub const READING: &str = "ls.reading";
pub const PLAYER: &str = "ls.player";

/// Story position in plan metres (Blender: +Y from the entrance to the kitchen) to the engine.
pub fn b(x: f32, y: f32, h: f32) -> Vec3 {
    Vec3::new(x, h, -y)
}

/// Forward vector of a yaw/pitch in degrees (the engine's camera convention).
pub fn look_dir(yaw: f32, pitch: f32) -> Vec3 {
    let (y, p) = (yaw.to_radians(), pitch.to_radians());
    Vec3::new(-y.sin() * p.cos(), p.sin(), -y.cos() * p.cos())
}

/// What the whole game is doing; scripts act only in their modes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Mode {
    #[default]
    Menu,
    Intro,
    Playing,
    Paused,
    Reading,
    Journal,
    Panel,
    Dead,
    Ending,
    Credits,
}

pub fn mode(ctx: &Ctx) -> Mode {
    ctx.global(MODE).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
}

/// Switches the mode and restarts the mode clock (`mode_time`).
pub fn set_mode(ctx: &mut Ctx, m: Mode) {
    if mode(ctx) != m {
        ctx.set_global(MODE, serde_json::to_value(m).unwrap_or_default());
        ctx.set_global(MODE_T, 0.0.into());
    }
}

/// Seconds since the current mode started (advanced by the director).
pub fn mode_time(ctx: &Ctx) -> f32 {
    ctx.global_f32(MODE_T, 0.0)
}

/// Everything the story needs to be rebuilt from (saved at checkpoints).
#[derive(Clone, Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
pub struct Story {
    pub checkpoint: u8,
    pub flashlight: bool,
    pub talked_anna: bool,
    pub notes: Vec<String>,
    /// Plates: 0 dirty at the sink, 1 dirty in hand, 2 clean in hand, 3 on the table.
    pub plates: [u8; 3],
    /// Meals (water, bread, sweet): 0 on the pass, 1 in hand, 2 on the table.
    pub meals: [u8; 3],
    pub service_done: bool,
    pub fuse: u8,
    pub power: [bool; 3],
    pub power_ok: bool,
    pub keycard: u8,
    pub office_open: bool,
    pub ledger_read: bool,
    pub crowbar: bool,
    pub locker_open: bool,
    pub key: bool,
    pub vera_book: bool,
    pub nina_label: bool,
    pub doll: bool,
    pub confession: bool,
    pub ring_open: bool,
    pub padlock_off: bool,
    pub exit_open: bool,
    pub placed: [bool; 3],
    pub corrected: bool,
    pub fourth: u8,
    pub anna_ack: bool,
    pub finished: bool,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub deaths: u32,
    pub play_time: f32,
}

impl Story {
    pub fn all_names(&self) -> bool {
        self.key && self.vera_book && self.doll
    }
}

/// The story as the director last published it.
pub fn story(ctx: &Ctx) -> Story {
    ctx.global(STORY).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
}

/// Player options, owned by the UI (saved with `ctx.set_setting`).
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub master: f32,
    pub music: f32,
    pub effects: f32,
    pub sensitivity: f32,
    pub vhs: bool,
    pub ps1: bool,
    pub reduced_motion: bool,
    pub no_chase: bool,
    pub russian: bool,
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { master: 0.9, music: 0.7, effects: 1.0, sensitivity: 0.11, vhs: true, ps1: false, reduced_motion: false, no_chase: false, russian: true, fullscreen: false }
    }
}

pub fn settings(ctx: &Ctx) -> Settings {
    ctx.global(SETTINGS).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
}

/// English or Russian by the current setting.
pub fn tr(ctx: &Ctx, en: &str, ru: &str) -> String {
    if settings(ctx).russian { ru.to_string() } else { en.to_string() }
}

/// What the player's body does this frame (written by `PlayerController`).
#[derive(Clone, Copy, Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct PlayerState {
    pub crouch: f32,
    pub sprint: bool,
    pub moving: bool,
}

pub fn player_state(ctx: &Ctx) -> PlayerState {
    ctx.global(PLAYER).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default()
}

/// A subtitle line for the UI.
pub fn say(ctx: &mut Ctx, who: &str, en: &str, ru: &str, time: f32) {
    let _ = ctx.emit("ls.say", serde_json::json!({ "who": who, "en": en, "ru": ru, "time": time }));
}

/// A short message in the middle of the screen.
pub fn message(ctx: &mut Ctx, en: &str, ru: &str) {
    let _ = ctx.emit("ls.msg", serde_json::json!({ "en": en, "ru": ru }));
}

/// A sound effect with a little pitch variation.
pub fn sfx(ctx: &mut Ctx, clip: &str, volume: f32) {
    let pitch = 0.92 + ctx.random() * 0.16;
    ctx.play_sound_pitched(&format!("audio/sfx/{clip}.wav"), volume, pitch);
}

/// Entity by name (cached by callers that need it every frame).
pub fn named(ctx: &Ctx, name: &str) -> Option<Entity> {
    ctx.find(name)
}

/// Whether an entity carries the script `script`.
pub fn has_script(ctx: &Ctx, e: Entity, script: &str) -> bool {
    ctx.get_of::<Scripts>(e).is_some_and(|s| s.0.iter().any(|slot| slot.name == script))
}

/// The entity or its nearest ancestor that has the script `script`.
pub fn with_script(ctx: &Ctx, mut e: Entity, script: &str) -> Option<Entity> {
    for _ in 0..8 {
        if has_script(ctx, e, script) {
            return Some(e);
        }
        e = hierarchy::parent_of(&ctx.scene.world, e)?;
    }
    None
}

/// What the eyes find along `dir`: a "use" area (even behind a thin rim such as a counter edge,
/// up to `slack` metres) or else the first solid thing. `exclude` is the player's own body.
pub fn look_at(ctx: &Ctx, eye: Vec3, dir: Vec3, reach: f32, exclude: Option<Entity>) -> Option<Entity> {
    use kvas_engine::physics::QueryOptions;
    let slack = 0.35;
    let solid = ctx.raycast_filtered(eye, dir, reach, QueryOptions { exclude, ..Default::default() });
    let area = ctx.raycast_filtered(eye, dir, reach, QueryOptions { exclude, only_triggers: true, ..Default::default() });
    match (area, solid) {
        (Some(a), Some(s)) if s.distance + slack < a.distance => Some(s.entity),
        (Some(a), _) => Some(a.entity),
        (None, s) => s.map(|s| s.entity),
    }
}

/// Note titles and texts (the journal and the reading screen).
pub fn note_text(id: &str, ru: bool) -> (&'static str, &'static str) {
    match (id, ru) {
        ("N01", false) => ("Note on the counter", "Lida, the papers are in the office. Clear away the three places before you leave. Mum.\n\nThe date is written twice: 14.XI.98 / 14.XI.70."),
        ("N01", true) => ("Записка на стойке", "Лида, документы в конторке. Перед уходом убери три прибора. Мама.\n\nДата написана дважды: 14.XI.98 / 14.XI.70."),
        ("N02", false) => ("Order at the pass", "Route 17. Driver - water. Teacher - bread. Girl - something sweet. Do not heat the food. Three plates. Do not clear until the guests leave."),
        ("N02", true) => ("Заказ на выдаче", "Рейс 17. Водителю — вода. Учительнице — хлеб. Девочке — сладкое. Еду не греть. Три тарелки. Не убирать до ухода гостей."),
        ("N03", false) => ("Ledger, first page", "14.XI.70. Route 17. Stop at 02:17. Dinners served: 3.\nVisitors at closing: 0.\nOn duty: A. Lesina."),
        ("N03", true) => ("Журнал, первая страница", "14.XI.70. Рейс 17. Остановка 02:17. Выдано ужинов: 3.\nПосетителей на момент закрытия: 0.\nДежурная А. Лесина."),
        ("N04", false) => ("Route sheet", "Driver Pavel Zorin. Route 17. Pereval - Ozyornoye. Passengers: 2. If the road is bad, wait at the diner. Leave when ready."),
        ("N04", true) => ("Путевой лист", "Водитель Павел Зорин. Рейс 17. Перевал — Озёрное. Пассажиров: 2. При неисправности дороги ждать у столовой. Выезд по готовности."),
        ("N05", false) => ("Exercise book", "\"Nina, don't be afraid. Ask for warm water. We'll be home soon. Mum.\"\n\nOn the cover: Vera Sedova. Class 4B. Check the books by Monday."),
        ("N05", true) => ("Тетрадь", "«Нина, не бойся. Попроси тёплой воды. Мы скоро дома. Мама».\n\nНа обложке: Вера Седова. 4Б. Проверить тетради до понедельника."),
        ("N06", false) => ("Doll label", "To Nina Sedova from Mum. Don't leave her at the bus stop."),
        ("N06", true) => ("Бирка куклы", "Нине Седовой от мамы. Не оставляй её на остановке."),
        ("N07", false) => ("Back of the ledger", "Pavel Zorin. Vera Sedova. Nina Sedova.\nI heard them. I did not open.\nEvery year I write it here, because on the first page I never had the courage."),
        ("N07", true) => ("Оборот журнала", "Павел Зорин. Вера Седова. Нина Седова.\nЯ слышала. Я не открыла.\nКаждый год пишу здесь, потому что на первой странице не хватает смелости."),
        ("N08", false) => ("Emergency door", "EXIT TO THE FOREST. LATCH ON THE KITCHEN SIDE. KEEP CLEAR."),
        ("N08", true) => ("Аварийная дверь", "ВЫХОД В ЛЕС. ЗАСОВ СО СТОРОНЫ КУХНИ. НЕ ЗАГОРАЖИВАТЬ."),
        (_, false) => ("Note", ""),
        (_, true) => ("Записка", ""),
    }
}

/// The current goal for the HUD, the journal and save titles.
pub fn objective(s: &Story, ru: bool) -> String {
    let (en, r) = if !s.talked_anna {
        ("Go inside. Find out why the light is on.", "Войти внутрь. Узнать, почему горит свет.")
    } else if !s.service_done {
        ("Wash three plates and set the table for Route 17.", "Вымыть три тарелки и накрыть стол для рейса 17.")
    } else if !s.power_ok {
        ("Restore power to the office: fuse and switch panel in the kitchen.", "Вернуть питание конторке: предохранитель и щиток на кухне.")
    } else if !s.office_open {
        ("Take the keycard from the pass and open the office.", "Взять карту у раздачи и открыть конторку.")
    } else if !s.ledger_read {
        ("Read the ledger in the office.", "Прочитать журнал в конторке.")
    } else if !s.all_names() {
        ("Find who the three places were for (yard, office, storage).", "Узнать, для кого были три места (двор, конторка, кладовая).")
    } else if !s.confession {
        ("Something changed in the office.", "В конторке что-то изменилось.")
    } else if !s.exit_open {
        ("Open the emergency exit in the kitchen. Stay out of its sight.", "Открыть аварийный выход на кухне. Не попадаться ему на глаза.")
    } else if !s.corrected {
        ("Return the names to the table and correct the ledger.", "Вернуть имена за стол и исправить запись.")
    } else if !s.anna_ack {
        ("Set the fourth plate.", "Поставить четвёртый прибор.")
    } else {
        ("Get to the bus stop.", "Дойти до остановки.")
    };
    if ru { r.to_string() } else { en.to_string() }
}

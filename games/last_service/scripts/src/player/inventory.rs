//! What Lida carries, shown as a row of item icons at the bottom of the screen. The story is the
//! source of truth (a plate in hand, the fuse, the key...); this script turns it into the hotbar.

use crate::s_shared::*;
use kvas_engine::prelude::*;

#[derive(Default)]
pub struct Inventory {
    /// Icons shown last frame, to animate new ones in.
    shown: Vec<(String, f32)>,
}

/// Item icon and its name in both languages.
fn items(s: &Story) -> Vec<(&'static str, &'static str, &'static str)> {
    let mut out = Vec::new();
    for p in s.plates {
        if p == 1 {
            out.push(("ui/icons/items/plate_dirty.png", "Dirty plate", "Грязная тарелка"));
        } else if p == 2 {
            out.push(("ui/icons/items/plate_clean.png", "Clean plate", "Чистая тарелка"));
        }
    }
    let meals = [("ui/icons/items/water.png", "Glass of water", "Стакан воды"), ("ui/icons/items/bread.png", "Bread", "Хлеб"), ("ui/icons/items/chocolate.png", "Chocolate", "Шоколад")];
    for (i, m) in s.meals.iter().enumerate() {
        if *m == 1 {
            out.push(meals[i]);
        }
    }
    if s.fuse == 1 {
        out.push(("ui/icons/items/fuse.png", "Fuse", "Предохранитель"));
    }
    if s.keycard == 1 && !s.office_open {
        out.push(("ui/icons/items/keycard.png", "Keycard", "Карта-ключ"));
    }
    if s.crowbar && !s.locker_open {
        out.push(("ui/icons/items/crowbar.png", "Crowbar", "Лом"));
    }
    if s.key && !s.placed[0] {
        out.push(("ui/icons/items/key.png", "Key and route sheet", "Ключ и путевой лист"));
    }
    if s.vera_book && !s.placed[1] {
        out.push(("ui/icons/items/book.png", "Exercise book", "Тетрадь"));
    }
    if s.doll && !s.placed[2] {
        out.push(("ui/icons/items/doll.png", "Doll", "Кукла"));
    }
    if s.fourth == 1 {
        out.push(("ui/icons/items/plate_clean.png", "Fourth plate", "Четвёртая тарелка"));
    }
    out
}

impl Script for Inventory {
    fn update(&mut self, ctx: &mut Ctx) {
        if mode(ctx) != Mode::Playing {
            return;
        }
        let dt = ctx.dt();
        let s = story(ctx);
        let ru = settings(ctx).russian;
        let list = items(&s);
        // New items slide in.
        let mut shown = Vec::new();
        for (icon, _, _) in &list {
            let age = self.shown.iter().find(|(i, _)| i == icon).map_or(0.0, |(_, a)| *a);
            shown.push((icon.to_string(), age + dt));
        }
        self.shown = shown;
        let w = ctx.ui_width();
        let ui = ctx.ui();
        for (i, (icon, en, r)) in list.iter().enumerate() {
            let age = self.shown.get(i).map_or(1.0, |(_, a)| *a);
            let rise = (1.0 - (age / 0.3).min(1.0)) * 30.0;
            let x = 24.0 + i as f32 * 70.0;
            ui.frame([x - 4.0, 638.0 + rise, 64.0, 64.0], [0.02, 0.02, 0.02, 0.6], [0.5, 0.46, 0.36, 0.8], 6.0);
            ui.image(icon, [x, 642.0 + rise, 56.0, 56.0], [1.0, 1.0, 1.0, 0.95]);
            if age < 2.5 {
                let a = (2.5 - age).min(1.0);
                ui.text(if ru { r } else { en }, [x, 620.0 + rise], 13.0, [PAPER[0], PAPER[1], PAPER[2], a], "", Align::Left);
            }
        }
        if s.flashlight {
            let on = ctx.global_bool(FLASH_ON, false);
            let ui = ctx.ui();
            ui.image("ui/icons/items/flashlight.png", [w - 84.0, 642.0, 56.0, 56.0], [1.0, 1.0, 1.0, if on { 0.95 } else { 0.35 }]);
            ui.text("F", [w - 56.0, 700.0], 12.0, DIM, "", Align::Center);
        }
    }
}

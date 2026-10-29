//! Using things by looking at them. A ray from the eyes finds either a door (an object with the
//! `Door` script) or a story thing (a trigger named `<id>#use`).
//! * doors: the prompt comes from the door itself; E sends it `use`;
//! * story things: this script publishes `ls.look` (the id), the director answers with
//!   `ls.offer` (prompt text and how long E must be held), and E sends `ls.act` to the director.

use crate::s_shared::*;
use kvas_engine::prelude::*;

pub struct Interactor {
    pub reach: f32,
    hold: f32,
    hold_id: String,
    eyes: Option<Entity>,
}

impl Default for Interactor {
    fn default() -> Self {
        Self { reach: 2.6, hold: 0.0, hold_id: String::new(), eyes: None }
    }
}

impl Script for Interactor {
    fn start(&mut self, ctx: &mut Ctx) {
        self.eyes = ctx.find("Eyes");
    }

    fn update(&mut self, ctx: &mut Ctx) {
        if mode(ctx) != Mode::Playing {
            ctx.set_global(LOOK, "".into());
            self.hold = 0.0;
            return;
        }
        let dt = ctx.dt().min(0.1);
        let Some(eyes) = self.eyes else { return };
        let m = ctx.world_matrix_of(eyes);
        let (eye, dir) = (m.w_axis.truncate(), -m.z_axis.truncate().normalize_or_zero());
        let me = ctx.entity;
        let entity = look_at(ctx, eye, dir, self.reach, Some(me));
        // A door in front of the eyes.
        if let Some(door) = entity.and_then(|e| with_script(ctx, e, "Door")) {
            ctx.set_global(LOOK, "".into());
            let locked = ctx.var_bool(door, "Door", "locked").unwrap_or(false);
            let open = ctx.var_bool(door, "Door", "open").unwrap_or(false);
            let ru = settings(ctx).russian;
            let name = ctx.var(door, "Door", if ru { "name_ru" } else { "name_en" }).and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
            let text = match (locked, open, ru) {
                (true, _, false) => format!("The {name} is locked"),
                (true, _, true) => format!("{} заперта", capital(&name)),
                (false, true, false) => format!("Close the {name}"),
                (false, true, true) => format!("Закрыть {name}"),
                (false, false, false) => format!("Open the {name}"),
                (false, false, true) => format!("Открыть {name}"),
            };
            ctx.hud_prompt(&format!("[E]  {text}"));
            if ctx.action_pressed("Interact") {
                let _ = ctx.send_to(door, "use", serde_json::Value::Null);
            }
            return;
        }
        let id = entity.map(|e| hierarchy::name_of(&ctx.scene.world, e).to_string()).and_then(|n| n.strip_suffix("#use").map(str::to_string)).unwrap_or_default();
        ctx.set_global(LOOK, id.clone().into());
        let offer = ctx.global(OFFER).cloned().unwrap_or_default();
        if id.is_empty() || offer["id"].as_str() != Some(id.as_str()) {
            self.hold = 0.0;
            return;
        }
        let text = offer["text"].as_str().unwrap_or("").to_string();
        let need = offer["hold"].as_f64().unwrap_or(0.0) as f32;
        if need > 0.0 {
            if ctx.action_down("Interact") {
                if self.hold_id != id {
                    self.hold_id = id.clone();
                    self.hold = 0.0;
                }
                self.hold += dt;
                if self.hold >= need {
                    self.hold = 0.0;
                    self.hold_id.clear();
                    let _ = ctx.emit("ls.act", serde_json::json!({ "id": id }));
                }
            } else {
                self.hold = 0.0;
            }
            let bar = "#".repeat(((self.hold / need).clamp(0.0, 1.0) * 12.0) as usize);
            ctx.hud_prompt(&format!("[E]  {text}  {bar}"));
        } else {
            ctx.hud_prompt(&format!("[E]  {text}"));
            if ctx.action_pressed("Interact") {
                let _ = ctx.emit("ls.act", serde_json::json!({ "id": id }));
            }
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("reach", &mut self.reach)]
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

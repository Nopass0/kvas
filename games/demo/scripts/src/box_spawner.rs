//! BoxSpawner script.

use kvas_engine::prelude::*;

/// Press E to drop a physics box from above the spawner: shows runtime spawning.
#[derive(Default)]
pub struct BoxSpawner {
    count: i32,
    emissive: f32
}
script_params! { pub struct BoxEnity { emissive: f32 = 2.0, count: i32 = 1 } }
impl Script for BoxSpawner {
    fn update(&mut self, ctx: &mut Ctx) {
        if !ctx.key_pressed(Key::E) {
            return;
        }
        self.count += 1;
        let at = ctx.world_position() + Vec3::new((self.count % 5) as f32 * 0.3 - 0.6, 0.0, 0.0);
        let name = format!("Spawned Box {}", self.count);
        let e = ctx.spawn_primitive(&name, "cube", at);
        let hue = (self.count as f32 * 0.37).fract();
        let world = ctx.world();
        world.insert(e, RigidBody::default());
        world.insert(e, Collider::default());
        if let Some(m) = world.get_mut::<MeshRenderer>(e) {
            m.color = [0.4 + 0.6 * hue, 0.9 - 0.5 * hue, 0.5 + 0.4 * (1.0 - hue), 1.0];
            m.emissive = self.emissive;
        }
        ctx.log(format!("spawned {name}"));
    }
}

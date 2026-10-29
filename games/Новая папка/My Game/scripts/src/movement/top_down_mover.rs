use kvas_engine::prelude::*;

script_params! {
    /// Moves a body on the ground plane with WASD (top-down and isometric games).
    pub struct TopDownMover {
        speed: f32 = 5.0,
        turn_to_move: bool = true,
    }
}

impl Script for TopDownMover {
    fn update(&mut self, ctx: &mut Ctx) {
        let input = ctx.action_vector("MoveLeft", "MoveRight", "MoveBack", "MoveForward");
        let wish = Vec3::new(input.x, 0.0, -input.y) * self.speed;
        let v = ctx.velocity();
        ctx.set_velocity(Vec3::new(wish.x, v.y, wish.z));
        if self.turn_to_move && wish.length() > 0.1 {
            let yaw = (-wish.x).atan2(-wish.z).to_degrees();
            ctx.transform_mut().rotation.y = yaw;
        }
    }
    fn fields(&mut self) -> Vec<Field<'_>> {
        self.params()
    }
}

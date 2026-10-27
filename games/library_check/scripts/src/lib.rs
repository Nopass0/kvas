//! Compiles every library script of the editor, so they stay valid.
use kvas_engine::prelude::*;

#[path = "../../../../crates/kvas_editor/library/scripts/audio/ambient_loop.rs"]
mod ambient_loop;
pub use ambient_loop::AmbientLoop;

#[path = "../../../../crates/kvas_editor/library/scripts/camera/camera_shake.rs"]
mod camera_shake;
pub use camera_shake::CameraShake;

#[path = "../../../../crates/kvas_editor/library/scripts/camera/zoom_on_key.rs"]
mod zoom_on_key;
pub use zoom_on_key::ZoomOnKey;

#[path = "../../../../crates/kvas_editor/library/scripts/gameplay/collectible.rs"]
mod collectible;
pub use collectible::Collectible;

#[path = "../../../../crates/kvas_editor/library/scripts/gameplay/look_at.rs"]
mod look_at;
pub use look_at::LookAt;

#[path = "../../../../crates/kvas_editor/library/scripts/gameplay/respawn.rs"]
mod respawn;
pub use respawn::Respawn;

#[path = "../../../../crates/kvas_editor/library/scripts/gameplay/score_counter.rs"]
mod score_counter;
pub use score_counter::ScoreCounter;

#[path = "../../../../crates/kvas_editor/library/scripts/gameplay/spawner.rs"]
mod spawner;
pub use spawner::Spawner;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/bobber.rs"]
mod bobber;
pub use bobber::Bobber;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/fly_camera.rs"]
mod fly_camera;
pub use fly_camera::FlyCamera;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/follow_camera.rs"]
mod follow_camera;
pub use follow_camera::FollowCamera;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/fps_player.rs"]
mod fps_player;
pub use fps_player::FpsPlayer;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/jumper.rs"]
mod jumper;
pub use jumper::Jumper;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/orbit.rs"]
mod orbit;
pub use orbit::Orbit;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/patrol.rs"]
mod patrol;
pub use patrol::Patrol;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/spinner.rs"]
mod spinner;
pub use spinner::Spinner;

#[path = "../../../../crates/kvas_editor/library/scripts/movement/top_down_mover.rs"]
mod top_down_mover;
pub use top_down_mover::TopDownMover;

#[path = "../../../../crates/kvas_editor/library/scripts/ui/welcome_message.rs"]
mod welcome_message;
pub use welcome_message::WelcomeMessage;

#[path = "../../../../crates/kvas_editor/library/scripts/world/day_night_cycle.rs"]
mod day_night_cycle;
pub use day_night_cycle::DayNightCycle;

#[path = "../../../../crates/kvas_editor/library/scripts/world/night_light.rs"]
mod night_light;
pub use night_light::NightLight;

register_scripts!(AmbientLoop, CameraShake, ZoomOnKey, Collectible, LookAt, Respawn, ScoreCounter, Spawner, Bobber, FlyCamera, FollowCamera, FpsPlayer, Jumper, Orbit, Patrol, Spinner, TopDownMover, WelcomeMessage, DayNightCycle, NightLight);

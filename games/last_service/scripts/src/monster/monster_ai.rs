//! The Counter. Lives on the monster model: patrols a graph of passages through the diner, sees
//! with a cone and a ray (further when the flashlight is on, less when Lida crouches), hears
//! running, searches the last seen place, opens unlocked doors on its way and catches.
//!
//! The director commands it with `ls.monster` signals: `{"cmd": "reset"}` (place it as the story
//! wants), `"glimpse"`, `"hide"`, `"warn"`. It reports `ls.caught` and publishes `ls.fear`.

use crate::s_shared::*;
use kvas_engine::physics::QueryOptions;
use kvas_engine::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Hidden,
    Glimpse,
    Patrol,
    Chase,
    Search,
    Table,
    Follow,
    /// Collapsed (a ragdoll) when the story lets it go.
    Fallen,
}

/// Navigation points (plan coordinates) and the passages between them, as the building really
/// is (measured with `examples/wall_scan.rs`); a passage can need a door.
const NODES: [(f32, f32); 23] = [
    (6.0, 4.0), (2.5, 3.0), (10.5, 2.5), (10.5, 6.5), (6.5, 7.3), // hall 0-4
    (6.5, 8.8), (3.5, 10.5), (2.0, 9.2), (0.8, 10.0), (4.5, 12.0), // kitchen 5-9
    (-0.8, 10.0), (-2.5, 12.0), (-0.8, 6.5), (0.8, 6.5), (-0.8, 14.0), // yard 10-14 (13 is inside the hall)
    (0.8, 14.0), (4.0, 14.0), (6.3, 14.05), // north corridor 15-17
    (7.7, 14.05), (9.5, 13.0), (9.1, 11.7), // storage 18-20
    (9.1, 10.3), (9.5, 9.5), // office 21-22
];
/// (a, b, door) — door: 0 none, 1 back door of the office block, 2 storage → office, 3 corridor →
/// storage, 4 emergency exit, 5 hall → yard, 6 kitchen.
const EDGES: [(usize, usize, u8); 27] = [
    (0, 1, 0), (0, 2, 0), (0, 4, 0), (2, 3, 0), (3, 4, 0), (1, 4, 0), (1, 13, 0), (4, 13, 0), (4, 5, 6), (5, 6, 0), (6, 7, 0), (6, 8, 0), (6, 9, 0), (7, 8, 0),
    (8, 10, 4), (10, 11, 0), (11, 12, 0), (11, 14, 0), (12, 13, 5), (14, 15, 1), (15, 16, 0), (16, 17, 0), (17, 18, 3),
    (18, 19, 0), (19, 20, 0), (20, 21, 2), (21, 22, 0),
];
const PATROL: [usize; 18] = [19, 18, 17, 16, 15, 14, 11, 10, 8, 6, 9, 6, 5, 4, 0, 2, 3, 4];
const AWAY: Vec3 = Vec3::new(0.0, -50.0, 0.0);

pub struct MonsterAi {
    pub walk_speed: f32,
    pub chase_speed: f32,
    pub sight: f32,
    state: State,
    pos: Vec3,
    yaw: f32,
    path: Vec<Vec3>,
    patrol_i: usize,
    last_seen: Vec3,
    lost: f32,
    timer: f32,
    step: f32,
    warn: f32,
    fear: f32,
    player: Option<Entity>,
    eyes: Option<Entity>,
    glow: Option<Entity>,
    doors: Vec<Entity>,
}

impl Default for MonsterAi {
    fn default() -> Self {
        Self {
            walk_speed: 1.3,
            chase_speed: 3.1,
            sight: 8.0,
            state: State::Hidden,
            pos: AWAY,
            yaw: 0.0,
            path: Vec::new(),
            patrol_i: 0,
            last_seen: Vec3::ZERO,
            lost: 0.0,
            timer: 0.0,
            step: 0.0,
            warn: 0.0,
            fear: 0.0,
            player: None,
            eyes: None,
            glow: None,
            doors: Vec::new(),
        }
    }
}

fn node(i: usize) -> Vec3 {
    b(NODES[i].0, NODES[i].1, 0.0)
}

fn nearest_node(p: Vec3) -> usize {
    (0..NODES.len()).min_by(|a, c| node(*a).distance_squared(p.with_y(0.0)).total_cmp(&node(*c).distance_squared(p.with_y(0.0)))).unwrap_or(0)
}

fn door_open(s: &Story, door: u8) -> bool {
    match door {
        1 | 2 => s.office_open,
        4 => s.exit_open,
        5 | 6 => s.talked_anna,
        _ => true,
    }
}

/// Shortest way through the passages the story has opened (Dijkstra on 22 points).
fn route(s: &Story, from: Vec3, to: Vec3) -> Vec<Vec3> {
    let (a, goal) = (nearest_node(from), nearest_node(to));
    let n = NODES.len();
    let mut dist = vec![f32::MAX; n];
    let mut prev = vec![usize::MAX; n];
    let mut done = vec![false; n];
    dist[a] = 0.0;
    for _ in 0..n {
        let Some(u) = (0..n).filter(|i| !done[*i] && dist[*i] < f32::MAX).min_by(|x, y| dist[*x].total_cmp(&dist[*y])) else { break };
        done[u] = true;
        for (x, y, door) in EDGES {
            if !door_open(s, door) {
                continue;
            }
            let v = if x == u { y } else if y == u { x } else { continue };
            let d = dist[u] + node(u).distance(node(v));
            if d < dist[v] {
                dist[v] = d;
                prev[v] = u;
            }
        }
    }
    let mut path = vec![to.with_y(0.0)];
    let mut c = goal;
    while c != usize::MAX && c != a {
        path.push(node(c));
        c = prev[c];
    }
    path.push(node(a));
    path.reverse();
    path
}

impl MonsterAi {
    fn eye(&self, ctx: &Ctx) -> Vec3 {
        self.eyes.map_or(Vec3::ZERO, |e| ctx.world_matrix_of(e).w_axis.truncate())
    }

    fn sees(&self, ctx: &Ctx, eye: Vec3) -> bool {
        let head = self.pos + Vec3::Y * 1.6;
        let to = eye - head;
        let dist = to.length();
        let range = if ctx.global_bool(FLASH_ON, false) { self.sight * 1.9 } else if player_state(ctx).crouch > 0.5 { self.sight * 0.5 } else { self.sight };
        if dist > range {
            return false;
        }
        let cone = look_dir(self.yaw, 0.0).dot(to.with_y(0.0).normalize_or_zero());
        if dist > 2.2 && cone < 0.25 {
            return false;
        }
        match ctx.raycast_filtered(head, to / dist.max(1e-3), dist, QueryOptions::default()) {
            Some(hit) => Some(hit.entity) == self.player || hit.distance > dist - 0.4,
            None => true,
        }
    }

    /// Its body follows what it does: the clips of the model (`MonsterPSX.fbx`). Walking and
    /// running play at the speed it really moves, so the feet do not slide.
    fn animate(&mut self, ctx: &mut Ctx, moving: bool, speed: f32) {
        let (clip, rate) = match self.state {
            State::Glimpse => ("Scream", 1.0),
            State::Chase if moving => ("Run_Frantic", (speed / 3.1).clamp(0.6, 1.6)),
            State::Patrol | State::Search | State::Follow if moving => ("Walk_Nervous", (speed / 1.3).clamp(0.6, 2.0)),
            _ => ("Idle_Watchful", 1.0),
        };
        let me = ctx.entity;
        ctx.play_animation(me, clip);
        if let Some(a) = ctx.get_mut::<Animator>() {
            a.looping = true;
            a.speed = rate;
        }
    }

    /// Places it as the story says (new game, load, story turns).
    fn reset(&mut self, ctx: &Ctx) {
        let s = story(ctx);
        self.state = if s.finished {
            State::Hidden
        } else if s.anna_ack {
            State::Follow
        } else if s.corrected {
            State::Table
        } else if s.confession && !settings(ctx).no_chase {
            State::Patrol
        } else {
            State::Hidden
        };
        self.pos = match self.state {
            State::Table | State::Follow => b(5.3, 8.3, 0.0),
            State::Patrol => b(10.0, 13.0, 0.0),
            _ => AWAY,
        };
        self.path.clear();
        self.lost = 0.0;
        self.patrol_i = 0;
    }

    fn show(&mut self, ctx: &mut Ctx) {
        let visible = self.state != State::Hidden;
        let (pos, yaw) = (self.pos, self.yaw);
        {
            let t = ctx.transform_mut();
            t.position.x = pos.x;
            t.position.z = pos.z;
            if !visible {
                t.position.y = AWAY.y;
            } else if t.position.y < -10.0 {
                t.position.y = 0.0;
            }
            t.rotation.y = yaw;
        }
        let me = ctx.entity;
        ctx.set_visible(me, visible);
        if let Some(g) = self.glow {
            if let Some(t) = ctx.get_mut_of::<Transform>(g) {
                t.position = pos + Vec3::Y * 1.9;
            }
            if let Some(l) = ctx.get_mut_of::<Light>(g) {
                l.intensity = if visible { 1.2 } else { 0.0 };
            }
        }
    }
}

impl Script for MonsterAi {
    fn start(&mut self, ctx: &mut Ctx) {
        self.player = ctx.find("Lida");
        self.eyes = ctx.find("Eyes");
        self.glow = ctx.find("Counter glow");
        self.doors = ctx.scene.world.entities().into_iter().filter(|e| has_script(ctx, *e, "Door")).collect();
        self.show(ctx);
    }

    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.dt().min(0.1);
        for sig in ctx.signals("ls.monster") {
            match sig.payload["cmd"].as_str().unwrap_or("") {
                "reset" => {
                    let me = ctx.entity;
                    ctx.stop_ragdoll(me);
                    self.reset(ctx);
                }
                "hide" => {
                    let me = ctx.entity;
                    ctx.stop_ragdoll(me);
                    self.state = State::Hidden;
                    self.pos = AWAY;
                }
                "collapse" if self.state != State::Hidden => {
                    // It goes limp where it stands, falling the way it was walking.
                    let me = ctx.entity;
                    let push = look_dir(self.yaw, 0.0) * 30.0 + Vec3::Y * 5.0;
                    ctx.ragdoll(me, push);
                    ctx.look_at_point(me, None);
                    self.state = State::Fallen;
                }
                "glimpse" => {
                    // First sight of it, behind the office door.
                    self.state = State::Glimpse;
                    self.pos = b(8.6, 9.6, 0.0);
                    self.timer = 4.0;
                }
                "warn" => self.warn = 3.0,
                _ => {}
            }
        }
        // It watches Lida with its head while it stands guard, follows or hunts.
        let me = ctx.entity;
        let watching = matches!(self.state, State::Table | State::Follow | State::Chase | State::Search);
        let eye_point = self.eyes.map(|e| ctx.world_matrix_of(e).w_axis.truncate());
        ctx.look_at_point(me, if watching { eye_point } else { None });
        let playing = mode(ctx) == Mode::Playing;
        if self.state == State::Fallen {
            self.fear = (self.fear - dt * 0.5).max(0.0);
            ctx.set_global(FEAR, self.fear.into());
            return;
        }
        if !playing {
            if self.state == State::Hidden || mode(ctx) == Mode::Menu {
                self.fear = 0.0;
            }
            ctx.set_global(FEAR, self.fear.into());
            self.show(ctx);
            return;
        }
        let player = self.player.and_then(|p| ctx.position_of(p)).unwrap_or(Vec3::ZERO);
        let eye = self.eye(ctx);
        if self.state == State::Hidden {
            self.fear = (self.fear - dt * 0.5).max(0.0);
            ctx.set_global(FEAR, self.fear.into());
            self.show(ctx);
            return;
        }
        if self.state == State::Glimpse {
            self.animate(ctx, false, 0.0);
            self.timer -= dt;
            self.yaw += dt * 20.0;
            if self.timer <= 0.0 {
                self.state = State::Hidden;
                self.pos = AWAY;
            }
            self.show(ctx);
            return;
        }
        let s = story(ctx);
        let sees = self.sees(ctx, eye);
        let body = player_state(ctx);
        let noisy = body.sprint && body.moving && player.distance(self.pos) < 9.0;
        let mut speed = self.walk_speed;
        let mut goal = None;
        match self.state {
            State::Patrol | State::Search => {
                if sees || noisy {
                    self.state = State::Chase;
                    self.last_seen = player;
                    self.warn = 1.5;
                } else if self.state == State::Search {
                    self.lost += dt;
                    if self.lost > 8.0 {
                        self.state = State::Patrol;
                        self.path.clear();
                    }
                }
            }
            State::Chase => {
                speed = self.chase_speed;
                if sees {
                    self.last_seen = player;
                    self.lost = 0.0;
                    goal = Some(player);
                } else {
                    self.lost += dt;
                    if self.lost > 5.0 {
                        self.state = State::Search;
                        self.lost = 0.0;
                        self.path.clear();
                    }
                }
            }
            State::Table => {
                // Stands at the pass and watches.
                let d = player - self.pos;
                self.yaw = (-d.x).atan2(-d.z).to_degrees();
            }
            State::Follow => {
                speed = 2.6;
                goal = Some(player);
            }
            _ => {}
        }
        if let Some(g) = goal.filter(|_| sees || self.state == State::Follow) {
            // Straight at the player while in sight; through the passages otherwise.
            if sees {
                self.path = vec![g.with_y(0.0)];
            } else if self.path.is_empty() || self.timer <= 0.0 {
                self.timer = 1.0;
            }
        }
        self.timer -= dt;
        let needs_route = self.path.is_empty() || (self.state == State::Follow && !sees && self.timer <= 0.0);
        if needs_route && matches!(self.state, State::Patrol | State::Search | State::Chase | State::Follow) {
            let target = match self.state {
                State::Patrol => node(PATROL[self.patrol_i % PATROL.len()]),
                State::Search | State::Chase => self.last_seen,
                _ => player,
            };
            self.path = route(&s, self.pos, target);
            self.timer = 1.2;
            if self.state == State::Patrol {
                self.patrol_i += 1;
            }
        }
        if let Some(next) = self.path.first().copied() {
            let to = (next - self.pos).with_y(0.0);
            if to.length() < 0.25 {
                self.path.remove(0);
            } else {
                let step = to.normalize() * speed * dt;
                self.pos += if step.length() > to.length() { to } else { step };
                let want = (-to.x).atan2(-to.z).to_degrees();
                let diff = (want - self.yaw + 540.0).rem_euclid(360.0) - 180.0;
                self.yaw += diff.clamp(-300.0 * dt, 300.0 * dt);
                self.step -= dt * speed;
                if self.step <= 0.0 {
                    self.step = 0.9;
                    let at = self.pos;
                    ctx.play_sound_at("audio/sfx/sfx_step_tile.wav", 0.8, at);
                }
            }
        }
        // It opens doors that are not locked when it walks into them.
        for d in self.doors.clone() {
            let near = ctx.position_of(d).is_some_and(|p| p.with_y(0.0).distance(self.pos) < 1.3);
            if near && !ctx.var_bool(d, "Door", "open").unwrap_or(true) && !ctx.var_bool(d, "Door", "locked").unwrap_or(true) {
                let _ = ctx.send_to(d, "open", serde_json::Value::Null);
            }
        }
        let dist = self.pos.distance(player.with_y(0.0));
        let near = (1.0 - (dist - 1.5) / 10.0).clamp(0.0, 1.0);
        self.fear += (near - self.fear) * (dt * 2.0).min(1.0);
        ctx.set_global(FEAR, self.fear.into());
        if self.warn > 0.0 {
            self.warn -= dt;
            if self.warn <= 0.0 {
                let at = self.pos;
                ctx.play_sound_at("audio/sfx/sfx_counter_warning.wav", 1.0, at);
            }
        }
        self.show(ctx);
        let moving = !self.path.is_empty();
        self.animate(ctx, moving, speed);
        let caught = dist < 1.05 && matches!(self.state, State::Chase | State::Follow) && !settings(ctx).no_chase;
        if caught {
            // Right in front of the eyes.
            let yaw = self.eyes.and_then(|e| ctx.get_of::<Transform>(e)).map_or(0.0, |t| t.rotation.y);
            self.pos = (eye + look_dir(yaw, 0.0) * 0.75).with_y(0.0);
            self.yaw = yaw + 180.0;
            self.show(ctx);
            let me = ctx.entity;
            ctx.play_animation(me, "Attack_Lunge");
            if let Some(a) = ctx.get_mut::<Animator>() {
                a.looping = false;
                a.speed = 1.0;
            }
            let _ = ctx.emit("ls.caught", serde_json::Value::Null);
        }
    }

    fn fields(&mut self) -> Vec<Field<'_>> {
        vec![Field::f32("walk_speed", &mut self.walk_speed), Field::f32("chase_speed", &mut self.chase_speed), Field::f32("sight", &mut self.sight)]
    }
}

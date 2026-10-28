//! Small CPU supervised networks with portable JSON weights and a planar steering component.
//! This is imitation/regression training, not reinforcement learning, navigation or NVIDIA inference.
use crate::{components::{Component, Transform}, hierarchy, reflect::Field};
use glam::Vec3;
use kvas_ecs::World;
use serde::{Deserialize, Serialize};

/// One supervised training pair. Normalize features and targets before training.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Example {
    /// Input features.
    pub input: Vec<f32>,
    /// Desired outputs.
    pub target: Vec<f32>,
}

/// One tanh hidden layer and a linear output layer, stored as row-major JSON weights.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Network {
    /// Format version; currently 1.
    pub version: u32,
    /// Number of input features.
    pub inputs: usize,
    /// Number of tanh neurons.
    pub hidden: usize,
    /// Number of predicted outputs.
    pub outputs: usize,
    /// Hidden-layer weights, hidden × inputs.
    pub w1: Vec<f32>,
    /// Hidden-layer biases.
    pub b1: Vec<f32>,
    /// Output-layer weights, outputs × hidden.
    pub w2: Vec<f32>,
    /// Output-layer biases.
    pub b2: Vec<f32>,
}

impl Network {
    /// Deterministic Xavier-like initialization. Each dimension must be 1..=128.
    pub fn new(inputs: usize, hidden: usize, outputs: usize, seed: u64) -> Result<Self, String> {
        if [inputs, hidden, outputs].iter().any(|&n| n == 0 || n > 128) { return Err("network dimensions must be 1..128".into()); }
        let mut state = seed;
        let mut weight = |scale: f32| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (((state >> 40) as f32 / 16777215.0) * 2.0 - 1.0) * scale
        };
        Ok(Self { version: 1, inputs, hidden, outputs, w1: (0..hidden * inputs).map(|_| weight((6.0 / (inputs + hidden) as f32).sqrt())).collect(), b1: vec![0.0; hidden], w2: (0..outputs * hidden).map(|_| weight((6.0 / (hidden + outputs) as f32).sqrt())).collect(), b2: vec![0.0; outputs] })
    }

    /// Rejects unsupported versions, oversized dimensions, corrupt shapes and non-finite weights.
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || [self.inputs, self.hidden, self.outputs].iter().any(|&n| n == 0 || n > 128) { return Err("unsupported network version or dimensions".into()); }
        if self.w1.len() != self.inputs * self.hidden || self.b1.len() != self.hidden || self.w2.len() != self.hidden * self.outputs || self.b2.len() != self.outputs { return Err("network weight shape mismatch".into()); }
        if self.w1.iter().chain(&self.b1).chain(&self.w2).chain(&self.b2).any(|v| !v.is_finite()) { return Err("non-finite network weights".into()); }
        Ok(())
    }

    /// Parses and validates portable weights.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let network: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        network.validate()?; Ok(network)
    }

    /// Atomically saves validated weights. Parent directory must exist.
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        crate::file_io::atomic_write(path, &bytes).map_err(|e| e.to_string())
    }

    fn forward(&self, input: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let hidden: Vec<f32> = (0..self.hidden).map(|h| (self.b1[h] + (0..self.inputs).map(|i| input[i] * self.w1[h * self.inputs + i]).sum::<f32>()).tanh()).collect();
        let output = (0..self.outputs).map(|o| self.b2[o] + (0..self.hidden).map(|h| hidden[h] * self.w2[o * self.hidden + h]).sum::<f32>()).collect();
        (hidden, output)
    }

    /// Predicts finite outputs or reports invalid features/weights.
    pub fn predict(&self, input: &[f32]) -> Result<Vec<f32>, String> {
        self.validate()?;
        if input.len() != self.inputs || input.iter().any(|v| !v.is_finite()) { return Err("invalid network input".into()); }
        let output = self.forward(input).1;
        if output.iter().any(|v| !v.is_finite()) { return Err("network output overflow".into()); }
        Ok(output)
    }

    /// One deterministic SGD epoch, returning pre-update mean squared error.
    /// Gradient clipping bounds each parameter update. Call repeatedly on a worker, with cancellation.
    pub fn train_epoch(&mut self, examples: &[Example], rate: f32) -> Result<f32, String> {
        self.validate()?;
        if !rate.is_finite() || rate <= 0.0 || rate > 1.0 || examples.is_empty() || examples.len() > 100_000 { return Err("invalid learning rate or dataset size".into()); }
        for e in examples {
            if e.input.len() != self.inputs || e.target.len() != self.outputs || e.input.iter().chain(&e.target).any(|v| !v.is_finite() || v.abs() > 1000.0) { return Err("training data shape mismatch or invalid feature; normalize data".into()); }
        }
        let mut mse = 0.0;
        for e in examples {
            let (h, y) = self.forward(&e.input);
            let errors: Vec<_> = y.iter().zip(&e.target).map(|(a, b)| a - b).collect();
            mse += errors.iter().map(|v| v * v).sum::<f32>() / self.outputs as f32;
            let grad_out: Vec<_> = errors.iter().map(|v| (2.0 * v / self.outputs as f32).clamp(-10.0, 10.0)).collect();
            let grad_hidden: Vec<_> = (0..self.hidden).map(|j| ((0..self.outputs).map(|o| grad_out[o] * self.w2[o * self.hidden + j]).sum::<f32>() * (1.0 - h[j] * h[j])).clamp(-10.0, 10.0)).collect();
            for o in 0..self.outputs {
                self.b2[o] -= rate * grad_out[o];
                for j in 0..self.hidden { self.w2[o * self.hidden + j] -= rate * (grad_out[o] * h[j]).clamp(-10.0, 10.0); }
            }
            for j in 0..self.hidden {
                self.b1[j] -= rate * grad_hidden[j];
                for i in 0..self.inputs { self.w1[j * self.inputs + i] -= rate * (grad_hidden[j] * e.input[i]).clamp(-10.0, 10.0); }
            }
        }
        self.validate()?;
        let result = mse / examples.len() as f32;
        if !result.is_finite() { return Err("training diverged".into()); }
        Ok(result)
    }
}

/// Small supervised steering controller. Inputs are normalized world X/Z offsets to a named target.
/// Outputs are planar movement X/Z. Does not perform pathfinding or control dynamic rigid bodies.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NeuralAgent {
    /// Enable CPU inference during Play.
    pub enabled: bool,
    /// Exact scene name of the target.
    pub target: String,
    /// Maximum world movement speed in metres per second.
    pub speed: f32,
    /// Distance represented by a feature magnitude of one.
    pub input_range: f32,
    /// Stop within this world-space distance.
    pub stop_distance: f32,
    /// Embedded trained weights, saved with the scene.
    pub network: Network,
}
impl Default for NeuralAgent {
    fn default() -> Self { Self { enabled: false, target: String::new(), speed: 2.0, input_range: 10.0, stop_distance: 0.3, network: Network::new(2, 8, 2, 42).unwrap() } }
}
impl Component for NeuralAgent {
    fn fields(&mut self) -> Vec<Field<'_>> { vec![Field::bool("enabled", &mut self.enabled), Field::entity("target", &mut self.target), Field::f32("speed", &mut self.speed), Field::f32("input_range", &mut self.input_range), Field::f32("stop_distance", &mut self.stop_distance)] }
}

/// Advances enabled, valid steering agents. Invalid models/settings leave entities untouched.
pub fn update(world: &mut World, dt: f32) {
    if !dt.is_finite() || dt <= 0.0 { return; }
    for e in world.entities_with::<NeuralAgent>() {
        let Some(agent) = world.get::<NeuralAgent>(e) else { continue };
        if !agent.enabled || !agent.speed.is_finite() || agent.speed < 0.0 || !agent.input_range.is_finite() || agent.input_range <= 0.0 || !agent.stop_distance.is_finite() || agent.stop_distance < 0.0 || agent.network.inputs != 2 || agent.network.outputs != 2 { continue; }
        if world.has::<crate::components::RigidBody>(e) { continue; }
        let Some(target) = hierarchy::find_by_name(world, &agent.target) else { continue };
        let position = hierarchy::global_matrix(world, e).w_axis.truncate();
        let delta = hierarchy::global_matrix(world, target).w_axis.truncate() - position;
        let planar = glam::Vec2::new(delta.x, delta.z);
        let distance = planar.length();
        if distance <= agent.stop_distance { continue; }
        let input = (planar / agent.input_range).clamp(glam::Vec2::NEG_ONE, glam::Vec2::ONE);
        let Ok(output) = agent.network.predict(&input.to_array()) else { continue };
        let direction = glam::Vec2::new(output[0], output[1]).clamp_length_max(1.0);
        let step = (agent.speed * dt).min((distance - agent.stop_distance).max(0.0));
        let desired = position + Vec3::new(direction.x, 0.0, direction.y) * step;
        let parent = hierarchy::parent_of(world, e).map_or(glam::Mat4::IDENTITY, |p| hierarchy::global_matrix(world, p));
        let local = parent.inverse().transform_point3(desired);
        if local.is_finite() { if let Some(pose) = world.get_mut::<Transform>(e) { pose.position = local; } }
    }
}

/// Reproducible steering examples: normalized X/Z offsets mapped to desired X/Z movement.
pub fn steering_examples() -> Vec<Example> {
    [-1.0f32, -0.5, 0.0, 0.5, 1.0].into_iter().flat_map(|x| [-1.0f32, -0.5, 0.0, 0.5, 1.0].into_iter().map(move |z| Example { input: vec![x, z], target: vec![x, z] })).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn training_reduces_error_and_weights_round_trip() {
        let mut model = Network::new(2, 8, 2, 42).unwrap(); let data = steering_examples();
        let initial = model.train_epoch(&data, 0.02).unwrap();
        let mut loss = initial;
        for _ in 0..500 { loss = model.train_epoch(&data, 0.02).unwrap(); }
        assert!(loss < initial * 0.05, "{initial} -> {loss}");
        let output = model.predict(&[0.75, -0.75]).unwrap(); assert!(output[0] > 0.6 && output[1] < -0.6);
        let loaded = Network::from_json(&serde_json::to_string(&model).unwrap()).unwrap();
        assert_eq!(output, loaded.predict(&[0.75, -0.75]).unwrap());
        model.w1.clear(); assert!(model.predict(&[0.0, 0.0]).is_err());
    }
}

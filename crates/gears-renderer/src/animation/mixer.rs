use super::{AnimationTarget, AnimationValue};
use crate::animation::clip::AnimationClip;
use std::collections::HashMap;
use std::time::Duration;

/// Blend modes for combining animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    /// Replace lower layer animations completely.
    Override,
    /// Add to lower layer animations.
    Additive,
    /// Multiply with lower layer animations.
    Multiply,
    /// Subtract from lower layer animations.
    Subtract,
    /// Use the maximum value.
    Maximum,
    /// Use the minimum value.
    Minimum,
    /// Screen blend mode.
    Screen,
    /// Overlay blend mode.
    Overlay,
}

/// A blend layer containing multiple animations.
#[derive(Debug, Clone)]
pub struct BlendLayer {
    /// Layer index (higher numbers are processed later).
    pub index: i32,
    /// Blend mode for this layer.
    pub blend_mode: BlendMode,
    /// Overall weight of this layer.
    pub weight: f32,
    /// Whether this layer is enabled.
    pub enabled: bool,
    /// Mask defining which targets this layer affects.
    pub mask: Option<AnimationMask>,
    /// Animations in this layer with their weights.
    pub animations: HashMap<String, LayerAnimation>,
}

/// Animation data within a blend layer.
#[derive(Debug, Clone)]
pub struct LayerAnimation {
    /// Reference to the animation clip.
    pub clip_name: String,
    /// Weight of this animation within the layer.
    pub weight: f32,
    /// Current playback time.
    pub time: f32,
    /// Playback speed multiplier.
    pub speed: f32,
    /// Whether this animation is playing.
    pub playing: bool,
    /// Time offset for this animation.
    pub time_offset: f32,
}

/// Mask defining which animation targets are affected.
#[derive(Debug, Clone)]
pub struct AnimationMask {
    /// Set of targets that are included (if empty, all targets are included).
    pub included_targets: Vec<AnimationTarget>,
    /// Set of targets that are excluded.
    pub excluded_targets: Vec<AnimationTarget>,
    /// Weight multiplier per target.
    pub target_weights: HashMap<AnimationTarget, f32>,
}

impl AnimationMask {
    /// Creates a new empty mask that affects all targets.
    ///
    /// # Returns
    ///
    /// A new [`AnimationMask`] instance.
    pub fn new() -> Self {
        Self {
            included_targets: Vec::new(),
            excluded_targets: Vec::new(),
            target_weights: HashMap::new(),
        }
    }

    /// Creates a mask that only affects specific targets.
    ///
    /// # Arguments
    ///
    /// * `targets` - The list of targets to include.
    ///
    /// # Returns
    ///
    /// A new [`AnimationMask`] instance.
    pub fn include_only(targets: Vec<AnimationTarget>) -> Self {
        Self {
            included_targets: targets,
            excluded_targets: Vec::new(),
            target_weights: HashMap::new(),
        }
    }

    /// Creates a mask that excludes specific targets.
    ///
    /// # Arguments
    ///
    /// * `targets` - The list of targets to exclude.
    ///
    /// # Returns
    ///
    /// A new [`AnimationMask`] instance.
    pub fn exclude(targets: Vec<AnimationTarget>) -> Self {
        Self {
            included_targets: Vec::new(),
            excluded_targets: targets,
            target_weights: HashMap::new(),
        }
    }

    /// Checks if a target is affected by this mask.
    ///
    /// # Arguments
    ///
    /// * `target` - The animation target to check.
    ///
    /// # Returns
    ///
    /// `true` if the target is affected by this mask.
    pub fn affects_target(&self, target: &AnimationTarget) -> bool {
        // If excluded, return false
        if self.excluded_targets.contains(target) {
            return false;
        }

        // If included list is empty, all targets are included (except excluded ones)
        if self.included_targets.is_empty() {
            return true;
        }

        // Check if target is in included list
        self.included_targets.contains(target)
    }

    /// Gets the weight multiplier for a target.
    ///
    /// # Arguments
    ///
    /// * `target` - The animation target to query.
    ///
    /// # Returns
    ///
    /// The weight multiplier for the target.
    pub fn get_target_weight(&self, target: &AnimationTarget) -> f32 {
        if !self.affects_target(target) {
            return 0.0;
        }

        self.target_weights.get(target).copied().unwrap_or(1.0)
    }

    /// Sets the weight for a specific target.
    ///
    /// # Arguments
    ///
    /// * `target` - The animation target to modify.
    /// * `weight` - The weight value (clamped to 0.0-1.0).
    pub fn set_target_weight(&mut self, target: AnimationTarget, weight: f32) {
        self.target_weights.insert(target, weight.clamp(0.0, 1.0));
    }
}

impl Default for AnimationMask {
    /// Creates a default animation mask that affects all targets.
    ///
    /// # Returns
    ///
    /// A new [`AnimationMask`] instance.
    fn default() -> Self {
        Self::new()
    }
}

impl BlendLayer {
    /// Creates a new blend layer.
    ///
    /// # Arguments
    ///
    /// * `index` - The layer index (higher numbers are processed later).
    ///
    /// # Returns
    ///
    /// A new [`BlendLayer`] instance.
    pub fn new(index: i32) -> Self {
        Self {
            index,
            blend_mode: BlendMode::Override,
            weight: 1.0,
            enabled: true,
            mask: None,
            animations: HashMap::new(),
        }
    }

    /// Creates a new blend layer with a specific blend mode.
    ///
    /// # Arguments
    ///
    /// * `index` - The layer index.
    /// * `blend_mode` - The blend mode to use for this layer.
    ///
    /// # Returns
    ///
    /// A new [`BlendLayer`] instance.
    pub fn with_blend_mode(index: i32, blend_mode: BlendMode) -> Self {
        Self {
            index,
            blend_mode,
            weight: 1.0,
            enabled: true,
            mask: None,
            animations: HashMap::new(),
        }
    }

    /// Adds an animation to this layer.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation clip.
    /// * `weight` - The weight of the animation (clamped to 0.0-1.0).
    pub fn add_animation(&mut self, name: String, weight: f32) {
        let animation = LayerAnimation {
            clip_name: name.clone(),
            weight: weight.clamp(0.0, 1.0),
            time: 0.0,
            speed: 1.0,
            playing: false,
            time_offset: 0.0,
        };
        self.animations.insert(name, animation);
    }

    /// Removes an animation from this layer.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation to remove.
    ///
    /// # Returns
    ///
    /// The removed layer animation if it existed.
    pub fn remove_animation(&mut self, name: &str) -> Option<LayerAnimation> {
        self.animations.remove(name)
    }

    /// Plays an animation in this layer.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation to play.
    ///
    /// # Returns
    ///
    /// An error message if the animation was not found.
    pub fn play_animation(&mut self, name: &str) -> Result<(), String> {
        if let Some(animation) = self.animations.get_mut(name) {
            animation.playing = true;
            animation.time = animation.time_offset;
            Ok(())
        } else {
            Err(format!(
                "Animation '{}' not found in layer {}",
                name, self.index
            ))
        }
    }

    /// Stops an animation in this layer.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation to stop.
    pub fn stop_animation(&mut self, name: &str) {
        if let Some(animation) = self.animations.get_mut(name) {
            animation.playing = false;
            animation.time = animation.time_offset;
        }
    }

    /// Sets the weight of an animation in this layer.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation.
    /// * `weight` - The new weight value (clamped to 0.0-1.0).
    ///
    /// # Returns
    ///
    /// An error message if the animation was not found.
    pub fn set_animation_weight(&mut self, name: &str, weight: f32) -> Result<(), String> {
        if let Some(animation) = self.animations.get_mut(name) {
            animation.weight = weight.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(format!(
                "Animation '{}' not found in layer {}",
                name, self.index
            ))
        }
    }

    /// Updates all animations in this layer.
    ///
    /// # Arguments
    ///
    /// * `dt` - The time delta since the last update.
    pub fn update(&mut self, dt: Duration) {
        for animation in self.animations.values_mut() {
            if animation.playing {
                animation.time += dt.as_secs_f32() * animation.speed;
            }
        }
    }

    /// Samples all animations in this layer and blends them.
    ///
    /// # Arguments
    ///
    /// * `clips` - The available animation clips to sample from.
    ///
    /// # Returns
    ///
    /// The blended animation values for all targets.
    pub fn sample(
        &self,
        clips: &HashMap<String, AnimationClip>,
    ) -> HashMap<AnimationTarget, AnimationValue> {
        let mut result: HashMap<AnimationTarget, AnimationValue> = HashMap::new();
        let mut total_weights: HashMap<AnimationTarget, f32> = HashMap::new();

        for animation in self.animations.values() {
            if !animation.playing || animation.weight <= 0.0 {
                continue;
            }

            if let Some(clip) = clips.get(&animation.clip_name) {
                let (loop_time, _) = clip.calculate_loop_time(animation.time);
                let clip_sample = clip.sample(loop_time);

                for (target, value) in clip_sample {
                    // Apply mask if present
                    let effective_weight = if let Some(mask) = &self.mask {
                        if !mask.affects_target(&target) {
                            continue;
                        }
                        animation.weight * mask.get_target_weight(&target)
                    } else {
                        animation.weight
                    };

                    if effective_weight <= 0.0 {
                        continue;
                    }

                    // Blend with existing values in this layer
                    let blended_value = if let Some(existing) = result.get(&target) {
                        let existing_weight = total_weights.get(&target).copied().unwrap_or(0.0);
                        let total_weight = existing_weight + effective_weight;

                        if total_weight > 0.0 {
                            let blend_factor = effective_weight / total_weight;
                            existing.lerp(&value, blend_factor).unwrap_or(value)
                        } else {
                            value
                        }
                    } else {
                        value
                    };

                    result.insert(target.clone(), blended_value);
                    *total_weights.entry(target).or_insert(0.0) += effective_weight;
                }
            }
        }

        result
    }

    /// Sets the mask for this layer.
    ///
    /// # Arguments
    ///
    /// * `mask` - Optional mask to apply to this layer.
    pub fn set_mask(&mut self, mask: Option<AnimationMask>) {
        self.mask = mask;
    }

    /// Checks if any layer has playing animations.
    ///
    /// # Returns
    ///
    /// `true` if any layer has playing animations.
    pub fn has_playing_animations(&self) -> bool {
        self.animations.values().any(|anim| anim.playing)
    }
}

/// The animation mixer manages multiple blend layers.
#[derive(Debug)]
pub struct AnimationMixer {
    /// Blend layers sorted by index.
    layers: Vec<BlendLayer>,
    /// Available animation clips.
    clips: HashMap<String, AnimationClip>,
    /// Global mixer weight
    global_weight: f32,
    /// Whether the mixer is enabled.
    enabled: bool,
}

impl AnimationMixer {
    /// Creates a new animation mixer.
    ///
    /// # Returns
    ///
    /// A new [`AnimationMixer`] instance.
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            clips: HashMap::new(),
            global_weight: 1.0,
            enabled: true,
        }
    }

    /// Adds an animation clip to the mixer.
    ///
    /// # Arguments
    ///
    /// * `clip` - The animation clip to add.
    pub fn add_clip(&mut self, clip: AnimationClip) {
        self.clips.insert(clip.name.clone(), clip);
    }

    /// Remove an animation clip from the mixer.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation clip to remove.
    ///
    /// # Returns
    ///
    /// The removed animation clip if it existed.
    pub fn remove_clip(&mut self, name: &str) -> Option<AnimationClip> {
        // Remove from all layers
        for layer in &mut self.layers {
            layer.remove_animation(name);
        }
        self.clips.remove(name)
    }

    /// Add a blend layer.
    ///
    /// # Arguments
    ///
    /// * `layer` - The blend layer to add.
    pub fn add_layer(&mut self, layer: BlendLayer) {
        // Insert in sorted order by index
        let insert_pos = self
            .layers
            .iter()
            .position(|l| l.index > layer.index)
            .unwrap_or(self.layers.len());
        self.layers.insert(insert_pos, layer);
    }

    /// Remove a blend layer.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the layer to remove.
    ///
    /// # Returns
    ///
    /// The removed blend layer if it existed.
    pub fn remove_layer(&mut self, index: i32) -> Option<BlendLayer> {
        if let Some(pos) = self.layers.iter().position(|l| l.index == index) {
            Some(self.layers.remove(pos))
        } else {
            None
        }
    }

    /// Gets a mutable reference to a layer.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the layer.
    ///
    /// # Returns
    ///
    /// A mutable reference to the layer if it exists.
    pub fn get_layer_mut(&mut self, index: i32) -> Option<&mut BlendLayer> {
        self.layers.iter_mut().find(|l| l.index == index)
    }

    /// Gets a reference to a layer.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the layer.
    ///
    /// # Returns
    ///
    /// A reference to the layer if it exists.
    pub fn get_layer(&self, index: i32) -> Option<&BlendLayer> {
        self.layers.iter().find(|l| l.index == index)
    }

    /// Plays an animation on a specific layer.
    ///
    /// # Arguments
    ///
    /// * `layer_index` - The index of the layer.
    /// * `animation_name` - The name of the animation to play.
    /// * `weight` - The weight of the animation (clamped to 0.0-1.0).
    ///
    /// # Returns
    ///
    /// An error message if the animation clip was not found.
    pub fn play_on_layer(
        &mut self,
        layer_index: i32,
        animation_name: &str,
        weight: f32,
    ) -> Result<(), String> {
        if !self.clips.contains_key(animation_name) {
            return Err(format!("Animation clip '{}' not found", animation_name));
        }

        // Find or create the layer
        if self.get_layer(layer_index).is_none() {
            self.add_layer(BlendLayer::new(layer_index));
        }

        let layer = self.get_layer_mut(layer_index).unwrap();

        // Add animation if not present
        if !layer.animations.contains_key(animation_name) {
            layer.add_animation(animation_name.to_string(), weight);
        } else {
            layer.set_animation_weight(animation_name, weight)?;
        }

        layer.play_animation(animation_name)
    }

    /// Stops an animation on a specific layer.
    ///
    /// # Arguments
    ///
    /// * `layer_index` - The index of the layer.
    /// * `animation_name` - The name of the animation to stop.
    ///
    /// # Returns
    ///
    /// An error message if the layer or animation was not found.
    pub fn stop_on_layer(&mut self, layer_index: i32, animation_name: &str) {
        if let Some(layer) = self.get_layer_mut(layer_index) {
            layer.stop_animation(animation_name);
        }
    }

    /// Update all layers.
    ///
    /// # Arguments
    ///
    /// * `dt` - The time delta since the last update.
    pub fn update(&mut self, dt: Duration) {
        if !self.enabled {
            return;
        }

        for layer in &mut self.layers {
            if layer.enabled {
                layer.update(dt);
            }
        }
    }

    /// Sample and blend all layers.
    ///
    /// # Returns
    ///
    /// The final blended animation values for all targets.
    pub fn sample(&self) -> HashMap<AnimationTarget, AnimationValue> {
        if !self.enabled {
            return HashMap::new();
        }

        let mut result = HashMap::new();

        for layer in &self.layers {
            if !layer.enabled || layer.weight <= 0.0 {
                continue;
            }

            let layer_sample = layer.sample(&self.clips);

            for (target, value) in layer_sample {
                let blended_value = if let Some(existing) = result.get(&target) {
                    self.blend_values(existing, &value, layer.blend_mode, layer.weight)
                } else {
                    // First layer for this target
                    match layer.blend_mode {
                        BlendMode::Override => value,
                        _ => value, // For other modes, treat as base value
                    }
                };

                result.insert(target, blended_value);
            }
        }

        // Apply global weight
        if self.global_weight != 1.0 {
            for value in result.values_mut() {
                *value = self.apply_global_weight(value, self.global_weight);
            }
        }

        result
    }

    /// Blend two animation values using the specified blend mode.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `blend_mode` - The blend mode to use.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn blend_values(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        blend_mode: BlendMode,
        weight: f32,
    ) -> AnimationValue {
        match blend_mode {
            BlendMode::Override => base
                .lerp(overlay, weight)
                .unwrap_or_else(|| overlay.clone()),
            BlendMode::Additive => self.additive_blend(base, overlay, weight),
            BlendMode::Multiply => self.multiply_blend(base, overlay, weight),
            BlendMode::Subtract => self.subtract_blend(base, overlay, weight),
            BlendMode::Maximum => self.maximum_blend(base, overlay, weight),
            BlendMode::Minimum => self.minimum_blend(base, overlay, weight),
            BlendMode::Screen => self.screen_blend(base, overlay, weight),
            BlendMode::Overlay => self.overlay_blend(base, overlay, weight),
        }
    }

    /// Additive blend mode implementation.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn additive_blend(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        weight: f32,
    ) -> AnimationValue {
        match (base, overlay) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                AnimationValue::Float(a + b * weight)
            }
            (AnimationValue::Vector3(a), AnimationValue::Vector3(b)) => {
                AnimationValue::Vector3(a + b * weight)
            }
            (AnimationValue::Quaternion(a), AnimationValue::Quaternion(b)) => {
                // Quaternion addition is complex; use slerp with weight
                AnimationValue::Quaternion(a.slerp(*b, weight))
            }
            _ => base.clone(),
        }
    }

    /// Multiply blend mode implementation.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn multiply_blend(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        weight: f32,
    ) -> AnimationValue {
        match (base, overlay) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                let multiplied = a * b;
                AnimationValue::Float(a + (multiplied - a) * weight)
            }
            (AnimationValue::Vector3(a), AnimationValue::Vector3(b)) => {
                let multiplied = cgmath::Vector3::new(a.x * b.x, a.y * b.y, a.z * b.z);
                AnimationValue::Vector3(a + (multiplied - a) * weight)
            }
            _ => base.clone(),
        }
    }

    /// Subtract blend mode implementation.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn subtract_blend(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        weight: f32,
    ) -> AnimationValue {
        match (base, overlay) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                AnimationValue::Float(a - b * weight)
            }
            (AnimationValue::Vector3(a), AnimationValue::Vector3(b)) => {
                AnimationValue::Vector3(a - b * weight)
            }
            _ => base.clone(),
        }
    }

    /// Subtract blend mode implementation.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn maximum_blend(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        weight: f32,
    ) -> AnimationValue {
        match (base, overlay) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                let max_val = a.max(*b);
                AnimationValue::Float(a + (max_val - a) * weight)
            }
            _ => base.lerp(overlay, weight).unwrap_or_else(|| base.clone()),
        }
    }

    /// Subtract blend mode implementation.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn minimum_blend(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        weight: f32,
    ) -> AnimationValue {
        match (base, overlay) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                let min_val = a.min(*b);
                AnimationValue::Float(a + (min_val - a) * weight)
            }
            _ => base.lerp(overlay, weight).unwrap_or_else(|| base.clone()),
        }
    }

    /// Subtract blend mode implementation.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn screen_blend(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        weight: f32,
    ) -> AnimationValue {
        match (base, overlay) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                let screen = 1.0 - (1.0 - a) * (1.0 - b);
                AnimationValue::Float(a + (screen - a) * weight)
            }
            _ => base.lerp(overlay, weight).unwrap_or_else(|| base.clone()),
        }
    }

    /// Subtract blend mode implementation.
    ///
    /// # Arguments
    ///
    /// * `base` - The base animation value.
    /// * `overlay` - The overlay animation value.
    /// * `weight` - The weight of the overlay value.
    ///
    /// # Returns
    ///
    /// The blended animation value.
    fn overlay_blend(
        &self,
        base: &AnimationValue,
        overlay: &AnimationValue,
        weight: f32,
    ) -> AnimationValue {
        match (base, overlay) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                let overlay_val = if *a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                };
                AnimationValue::Float(a + (overlay_val - a) * weight)
            }
            _ => base.lerp(overlay, weight).unwrap_or_else(|| base.clone()),
        }
    }

    /// Apply global weight to an animation value.
    ///
    /// # Arguments
    ///
    /// * `value` - The animation value to modify.
    /// * `weight` - The global weight to apply.
    ///
    /// # Returns
    ///
    /// The modified animation value.
    fn apply_global_weight(&self, value: &AnimationValue, weight: f32) -> AnimationValue {
        match value {
            AnimationValue::Float(f) => AnimationValue::Float(f * weight),
            AnimationValue::Vector3(v) => AnimationValue::Vector3(v * weight),
            AnimationValue::Quaternion(q) => {
                // For quaternions, interpolate with identity
                let identity = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
                AnimationValue::Quaternion(identity.slerp(*q, weight))
            }
            AnimationValue::FloatArray(arr) => {
                AnimationValue::FloatArray(arr.iter().map(|f| f * weight).collect())
            }
        }
    }

    /// Set global mixer weight.
    ///
    /// # Arguments
    ///
    /// * `weight` - The global weight value (clamped to 0.0-1.0).
    pub fn set_global_weight(&mut self, weight: f32) {
        self.global_weight = weight.clamp(0.0, 1.0);
    }

    /// Enables or disables the mixer.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the mixer should be enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Gets the number of layers.
    ///
    /// # Returns
    ///
    /// The number of blend layers in the mixer.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Gets all layer indices.
    ///
    /// # Returns
    ///
    /// A vector of all layer indices.
    pub fn get_layer_indices(&self) -> Vec<i32> {
        self.layers.iter().map(|l| l.index).collect()
    }

    /// Check if mixer has any playing animations.
    ///
    /// # Returns
    ///
    /// `true` if any layer has playing animations.
    pub fn has_playing_animations(&self) -> bool {
        self.layers
            .iter()
            .any(|layer| layer.enabled && layer.has_playing_animations())
    }
}

impl Default for AnimationMixer {
    /// Creates a default animation mixer.
    ///
    /// # Returns
    ///
    /// A new [`AnimationMixer`] instance.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{AnimationClip, AnimationTarget};

    #[test]
    fn test_mixer_creation() {
        let mixer = AnimationMixer::new();
        assert_eq!(mixer.layer_count(), 0);
        assert!(mixer.enabled);
    }

    #[test]
    fn test_layer_management() {
        let mut mixer = AnimationMixer::new();

        let layer = BlendLayer::new(0);
        mixer.add_layer(layer);

        assert_eq!(mixer.layer_count(), 1);
        assert!(mixer.get_layer(0).is_some());

        mixer.remove_layer(0);
        assert_eq!(mixer.layer_count(), 0);
    }

    #[test]
    fn test_animation_mask() {
        let mut mask = AnimationMask::new();

        let target = AnimationTarget::Translation;
        assert!(mask.affects_target(&target));

        mask.excluded_targets.push(target.clone());
        assert!(!mask.affects_target(&target));

        let mask2 = AnimationMask::include_only(vec![AnimationTarget::Rotation]);
        assert!(!mask2.affects_target(&target));
        assert!(mask2.affects_target(&AnimationTarget::Rotation));
    }
}

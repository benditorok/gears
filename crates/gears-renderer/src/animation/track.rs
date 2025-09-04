//! Animation track implementation for storing and interpolating keyframes.

use super::{AnimationValue, InterpolationMode, Keyframe};
use cgmath::{InnerSpace, Quaternion, Vector3};

/// An animation track contains keyframes for a single animated property
#[derive(Debug, Clone)]
pub struct AnimationTrack {
    /// The keyframes that define this track
    pub keyframes: Vec<Keyframe>,
    /// Default interpolation mode for this track
    pub default_interpolation: InterpolationMode,
    /// Whether this track should be normalized (for rotations)
    pub normalize: bool,
}

impl AnimationTrack {
    /// Create a new empty animation track
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            default_interpolation: InterpolationMode::Linear,
            normalize: false,
        }
    }

    /// Create a new animation track with specified interpolation mode
    pub fn with_interpolation(interpolation: InterpolationMode) -> Self {
        Self {
            keyframes: Vec::new(),
            default_interpolation: interpolation,
            normalize: false,
        }
    }

    /// Create a track for rotation data (enables quaternion normalization)
    pub fn new_rotation_track() -> Self {
        Self {
            keyframes: Vec::new(),
            default_interpolation: InterpolationMode::Linear,
            normalize: true,
        }
    }

    /// Add a keyframe to this track
    pub fn add_keyframe(&mut self, keyframe: Keyframe) -> &mut Self {
        // Insert keyframe in sorted order by time
        let insert_pos = self
            .keyframes
            .iter()
            .position(|k| k.time > keyframe.time)
            .unwrap_or(self.keyframes.len());

        self.keyframes.insert(insert_pos, keyframe);
        self
    }

    /// Add a keyframe with time and value
    pub fn add_keyframe_simple(&mut self, time: f32, value: AnimationValue) -> &mut Self {
        let keyframe = Keyframe::new(time, value);
        self.add_keyframe(keyframe)
    }

    /// Get the duration of this track (time of last keyframe)
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }

    /// Sample the track at a given time
    pub fn sample(&self, time: f32) -> Option<AnimationValue> {
        if self.keyframes.is_empty() {
            return None;
        }

        // Handle time before first keyframe
        if time <= self.keyframes[0].time {
            return Some(self.keyframes[0].value.clone());
        }

        // Handle time after last keyframe
        if time >= self.keyframes.last().unwrap().time {
            return Some(self.keyframes.last().unwrap().value.clone());
        }

        // Find the two keyframes to interpolate between
        for i in 0..self.keyframes.len() - 1 {
            let current = &self.keyframes[i];
            let next = &self.keyframes[i + 1];

            if time >= current.time && time <= next.time {
                return self.interpolate_between(current, next, time);
            }
        }

        None
    }

    /// Interpolate between two keyframes at the given time
    fn interpolate_between(
        &self,
        from: &Keyframe,
        to: &Keyframe,
        time: f32,
    ) -> Option<AnimationValue> {
        let duration = to.time - from.time;
        if duration <= 0.0 {
            return Some(from.value.clone());
        }

        let t = (time - from.time) / duration;
        let interpolation = if from.interpolation == InterpolationMode::Custom {
            self.default_interpolation
        } else {
            from.interpolation
        };

        match interpolation {
            InterpolationMode::Step => Some(from.value.clone()),
            InterpolationMode::Linear => {
                let result = from.value.lerp(&to.value, t)?;
                if self.normalize {
                    self.normalize_value(result)
                } else {
                    Some(result)
                }
            }
            InterpolationMode::CubicSpline => {
                // For now, fall back to linear interpolation
                // TODO: Implement proper cubic spline interpolation
                let result = from.value.lerp(&to.value, t)?;
                if self.normalize {
                    self.normalize_value(result)
                } else {
                    Some(result)
                }
            }
            InterpolationMode::Custom => {
                // Custom interpolation should be handled by the caller
                from.value.lerp(&to.value, t)
            }
        }
    }

    /// Normalize animation values (mainly for quaternions)
    fn normalize_value(&self, value: AnimationValue) -> Option<AnimationValue> {
        match value {
            AnimationValue::Quaternion(mut q) => {
                q = q.normalize();
                Some(AnimationValue::Quaternion(q))
            }
            AnimationValue::Vector3(mut v) => {
                if v.magnitude2() > 0.0 {
                    v = v.normalize();
                }
                Some(AnimationValue::Vector3(v))
            }
            other => Some(other),
        }
    }

    /// Get keyframe at a specific index
    pub fn get_keyframe(&self, index: usize) -> Option<&Keyframe> {
        self.keyframes.get(index)
    }

    /// Get mutable keyframe at a specific index
    pub fn get_keyframe_mut(&mut self, index: usize) -> Option<&mut Keyframe> {
        self.keyframes.get_mut(index)
    }

    /// Remove a keyframe at the specified index
    pub fn remove_keyframe(&mut self, index: usize) -> Option<Keyframe> {
        if index < self.keyframes.len() {
            Some(self.keyframes.remove(index))
        } else {
            None
        }
    }

    /// Clear all keyframes
    pub fn clear(&mut self) {
        self.keyframes.clear();
    }

    /// Get the number of keyframes
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Check if this track is empty
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Find the keyframe indices that surround the given time
    pub fn find_keyframe_indices(&self, time: f32) -> Option<(usize, usize)> {
        if self.keyframes.len() < 2 {
            return None;
        }

        for i in 0..self.keyframes.len() - 1 {
            if time >= self.keyframes[i].time && time <= self.keyframes[i + 1].time {
                return Some((i, i + 1));
            }
        }

        None
    }

    /// Scale all keyframe times by a factor
    pub fn scale_time(&mut self, scale_factor: f32) {
        for keyframe in &mut self.keyframes {
            keyframe.time *= scale_factor;
        }
    }

    /// Offset all keyframe times by a constant
    pub fn offset_time(&mut self, offset: f32) {
        for keyframe in &mut self.keyframes {
            keyframe.time += offset;
        }
    }

    /// Create a sub-track from a time range
    pub fn create_sub_track(&self, start_time: f32, end_time: f32) -> Option<AnimationTrack> {
        if start_time >= end_time {
            return None;
        }

        let mut sub_track = AnimationTrack {
            keyframes: Vec::new(),
            default_interpolation: self.default_interpolation,
            normalize: self.normalize,
        };

        // Add keyframes that fall within the range
        for keyframe in &self.keyframes {
            if keyframe.time >= start_time && keyframe.time <= end_time {
                let mut new_keyframe = keyframe.clone();
                new_keyframe.time -= start_time; // Adjust time to start from 0
                sub_track.keyframes.push(new_keyframe);
            }
        }

        // If no keyframes in range, sample at start and end
        if sub_track.keyframes.is_empty() {
            if let Some(start_value) = self.sample(start_time) {
                sub_track.add_keyframe_simple(0.0, start_value);
            }
            if let Some(end_value) = self.sample(end_time) {
                sub_track.add_keyframe_simple(end_time - start_time, end_value);
            }
        } else {
            // Ensure we have keyframes at the exact start and end times
            if sub_track.keyframes[0].time > 0.0 {
                if let Some(start_value) = self.sample(start_time) {
                    sub_track
                        .keyframes
                        .insert(0, Keyframe::new(0.0, start_value));
                }
            }

            let duration = end_time - start_time;
            if sub_track.keyframes.last().unwrap().time < duration {
                if let Some(end_value) = self.sample(end_time) {
                    sub_track.add_keyframe_simple(duration, end_value);
                }
            }
        }

        if sub_track.keyframes.is_empty() {
            None
        } else {
            Some(sub_track)
        }
    }

    /// Reverse the track (play backwards)
    pub fn reverse(&mut self) {
        let duration = self.duration();

        for keyframe in &mut self.keyframes {
            keyframe.time = duration - keyframe.time;
        }

        self.keyframes.reverse();
    }

    /// Combine this track with another track using weighted blending
    pub fn blend_with(
        &self,
        other: &AnimationTrack,
        weight: f32,
        time: f32,
    ) -> Option<AnimationValue> {
        let value_a = self.sample(time)?;
        let value_b = other.sample(time)?;

        // Blend the two values
        let blended = value_a.lerp(&value_b, weight)?;

        if self.normalize {
            self.normalize_value(blended)
        } else {
            Some(blended)
        }
    }
}

impl Default for AnimationTrack {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating animation tracks fluently
pub struct AnimationTrackBuilder {
    track: AnimationTrack,
}

impl AnimationTrackBuilder {
    pub fn new() -> Self {
        Self {
            track: AnimationTrack::new(),
        }
    }

    pub fn with_interpolation(mut self, interpolation: InterpolationMode) -> Self {
        self.track.default_interpolation = interpolation;
        self
    }

    pub fn with_normalization(mut self, normalize: bool) -> Self {
        self.track.normalize = normalize;
        self
    }

    pub fn add_keyframe(mut self, keyframe: Keyframe) -> Self {
        self.track.add_keyframe(keyframe);
        self
    }

    pub fn add_keyframe_simple(mut self, time: f32, value: AnimationValue) -> Self {
        self.track.add_keyframe_simple(time, value);
        self
    }

    pub fn add_translation_keyframe(mut self, time: f32, translation: Vector3<f32>) -> Self {
        self.track
            .add_keyframe_simple(time, AnimationValue::Vector3(translation));
        self
    }

    pub fn add_rotation_keyframe(mut self, time: f32, rotation: Quaternion<f32>) -> Self {
        self.track
            .add_keyframe_simple(time, AnimationValue::Quaternion(rotation));
        self
    }

    pub fn add_scale_keyframe(mut self, time: f32, scale: Vector3<f32>) -> Self {
        self.track
            .add_keyframe_simple(time, AnimationValue::Vector3(scale));
        self
    }

    pub fn build(self) -> AnimationTrack {
        self.track
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Vector3;

    #[test]
    fn test_track_creation() {
        let track = AnimationTrack::new();
        assert!(track.is_empty());
        assert_eq!(track.duration(), 0.0);
    }

    #[test]
    fn test_keyframe_addition() {
        let mut track = AnimationTrack::new();
        let keyframe1 = Keyframe::new(1.0, AnimationValue::Float(1.0));
        let keyframe2 = Keyframe::new(0.5, AnimationValue::Float(0.5));

        track.add_keyframe(keyframe1);
        track.add_keyframe(keyframe2);

        assert_eq!(track.keyframe_count(), 2);
        assert_eq!(track.get_keyframe(0).unwrap().time, 0.5); // Should be sorted
        assert_eq!(track.get_keyframe(1).unwrap().time, 1.0);
    }

    #[test]
    fn test_sampling() {
        let mut track = AnimationTrack::new();
        track.add_keyframe_simple(0.0, AnimationValue::Float(0.0));
        track.add_keyframe_simple(1.0, AnimationValue::Float(1.0));

        let sampled = track.sample(0.5).unwrap();
        if let AnimationValue::Float(value) = sampled {
            assert!((value - 0.5).abs() < 1e-6);
        } else {
            panic!("Expected Float value");
        }
    }

    #[test]
    fn test_track_builder() {
        let track = AnimationTrackBuilder::new()
            .with_interpolation(InterpolationMode::Linear)
            .add_translation_keyframe(0.0, Vector3::new(0.0, 0.0, 0.0))
            .add_translation_keyframe(1.0, Vector3::new(1.0, 1.0, 1.0))
            .build();

        assert_eq!(track.keyframe_count(), 2);
        assert_eq!(track.duration(), 1.0);
    }
}

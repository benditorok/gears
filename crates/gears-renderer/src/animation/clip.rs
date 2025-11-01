use super::{AnimationEvent, AnimationTarget, LoopMode};
use crate::animation::track::AnimationTrack;
use std::collections::HashMap;

/// An animation clip contains multiple tracks and events that define a complete animation.
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// Unique name for this animation clip.
    pub name: String,
    /// Duration of the animation in seconds.
    pub duration: f32,
    /// Animation tracks organized by target.
    pub tracks: HashMap<AnimationTarget, AnimationTrack>,
    /// Events that trigger at specific times during the animation.
    pub events: Vec<AnimationEvent>,
    /// How the animation should loop.
    pub loop_mode: LoopMode,
    /// Priority for blending (higher values take precedence).
    pub priority: i32,
    /// Whether this clip can be blended with others.
    pub blendable: bool,
}

impl AnimationClip {
    /// Create a new animation clip with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation clip.
    ///
    /// # Returns
    ///
    /// A new [`AnimationClip`] instance.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            duration: 0.0,
            tracks: HashMap::new(),
            events: Vec::new(),
            loop_mode: LoopMode::Once,
            priority: 0,
            blendable: true,
        }
    }

    /// Create a new animation clip with specified duration.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation clip.
    /// * `duration` - The duration of the animation clip in seconds.
    ///
    /// # Returns
    ///
    /// A new [`AnimationClip`] instance.
    pub fn with_duration(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            tracks: HashMap::new(),
            events: Vec::new(),
            loop_mode: LoopMode::Once,
            priority: 0,
            blendable: true,
        }
    }

    /// Add an animation track to this clip.
    ///
    /// # Arguments
    ///
    /// * `target` - The target that this track affects.
    /// * `track` - The animation track to add.
    ///
    /// # Returns
    ///
    /// A mutable reference to the [`AnimationClip`] instance.
    pub fn add_track(&mut self, target: AnimationTarget, track: AnimationTrack) -> &mut Self {
        // Update duration based on track duration
        if track.duration() > self.duration {
            self.duration = track.duration();
        }
        self.tracks.insert(target.clone(), track);
        self
    }

    /// Add an event to this clip.
    ///
    /// # Arguments
    ///
    /// * `event` - The animation event to add.
    ///
    /// # Returns
    ///
    /// A mutable reference to the [`AnimationClip`] instance.
    pub fn add_event(&mut self, event: AnimationEvent) -> &mut Self {
        // Keep events sorted by time for efficient processing
        let insert_pos = self
            .events
            .iter()
            .position(|e| e.time > event.time)
            .unwrap_or(self.events.len());
        self.events.insert(insert_pos, event);
        self
    }

    /// Set the loop mode for this clip.
    ///
    /// # Arguments
    ///
    /// * `loop_mode` - The loop mode to set.
    pub fn with_loop_mode(mut self, loop_mode: LoopMode) -> Self {
        self.loop_mode = loop_mode;
        self
    }

    /// Set the priority for this clip.
    ///
    /// # Arguments
    ///
    /// * `priority` - The priority to set.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set whether this clip can be blended.
    ///
    /// # Arguments
    ///
    /// * `blendable` - Whether the clip is blendable.
    pub fn with_blendable(mut self, blendable: bool) -> Self {
        self.blendable = blendable;
        self
    }

    /// Get a track for a specific target.
    ///
    /// # Arguments
    ///
    /// * `target` - The target to get the track for.
    ///
    /// # Returns
    ///
    /// An optional reference to the [`AnimationTrack`].
    pub fn get_track(&self, target: &AnimationTarget) -> Option<&AnimationTrack> {
        self.tracks.get(target)
    }

    /// Get a mutable track for a specific target.
    ///
    /// # Arguments
    ///
    /// * `target` - The target to get the track for.
    ///
    /// # Returns
    ///
    /// An optional mutable reference to the [`AnimationTrack`].
    pub fn get_track_mut(&mut self, target: &AnimationTarget) -> Option<&mut AnimationTrack> {
        self.tracks.get_mut(target)
    }

    /// Get all events that should trigger between two time points.
    ///
    /// # Arguments
    ///
    /// * `start_time` - The start time of the range.
    /// * `end_time` - The end time of the range.
    ///
    /// # Returns
    ///
    /// A vector of references to the [`AnimationEvent`]s in the specified range.
    pub fn get_events_in_range(&self, start_time: f32, end_time: f32) -> Vec<&AnimationEvent> {
        self.events
            .iter()
            .filter(|event| event.time >= start_time && event.time <= end_time)
            .collect()
    }

    /// Sample all tracks at a given time and return the results.
    ///
    /// # Arguments
    ///
    /// * `time` - The time at which to sample the animation.
    ///
    /// # Returns
    ///
    /// A HashMap of AnimationTarget to sampled AnimationValue.
    pub fn sample(&self, time: f32) -> HashMap<AnimationTarget, super::AnimationValue> {
        let mut results = HashMap::new();

        for (target, track) in &self.tracks {
            if let Some(value) = track.sample(time) {
                results.insert(target.clone(), value);
            }
        }

        results
    }

    /// Calculate the actual time considering loop mode.
    ///
    /// # Arguments
    ///
    /// * `time` - The input time to calculate.
    ///
    /// # Returns
    ///
    /// A tuple containing the calculated time and a boolean indicating if the animation has finished.
    pub fn calculate_loop_time(&self, time: f32) -> (f32, bool) {
        if self.duration <= 0.0 {
            return (0.0, false);
        }

        match self.loop_mode {
            LoopMode::Once => {
                if time >= self.duration {
                    (self.duration, true) // Finished
                } else {
                    (time, false)
                }
            }
            LoopMode::Repeat => {
                (time % self.duration, false) // Never finished
            }
            LoopMode::RepeatCount(count) => {
                let cycle_time = time / self.duration;
                if cycle_time >= count as f32 {
                    (self.duration, true) // Finished after specified cycles
                } else {
                    (time % self.duration, false)
                }
            }
            LoopMode::PingPong => {
                let cycle_time = time / self.duration;
                let cycle_index = cycle_time as i32;
                let local_time = time % self.duration;

                if cycle_index % 2 == 0 {
                    // Forward direction
                    (local_time, false)
                } else {
                    // Backward direction
                    (self.duration - local_time, false)
                }
            }
        }
    }

    /// Check if this clip has any tracks.
    ///
    /// # Returns
    ///
    /// `true` if the clip has no tracks.
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// Get the number of tracks in this clip.
    ///
    /// # Returns
    ///
    /// The number of tracks.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Get all animation targets that this clip affects.
    ///
    /// # Returns
    ///
    /// A vector of references to the [`AnimationTarget`]s.
    pub fn get_targets(&self) -> Vec<&AnimationTarget> {
        self.tracks.keys().collect()
    }

    /// Clone this clip with a new name.
    ///
    /// # Arguments
    ///
    /// * `new_name` - The new name for the cloned clip.
    ///
    /// # Returns
    ///
    /// A new [`AnimationClip`] instance with the specified name.
    pub fn clone_with_name(&self, new_name: impl Into<String>) -> Self {
        let mut clone = self.clone();
        clone.name = new_name.into();
        clone
    }

    /// Scale the duration and timing of this clip.
    ///
    /// # Arguments
    ///
    /// * `scale_factor` - The factor by which to scale the time.
    pub fn scale_time(&mut self, scale_factor: f32) {
        if scale_factor <= 0.0 {
            return;
        }

        self.duration *= scale_factor;

        // Scale all track timings
        for track in self.tracks.values_mut() {
            track.scale_time(scale_factor);
        }

        // Scale all event timings
        for event in &mut self.events {
            event.time *= scale_factor;
        }
    }

    /// Create a sub-clip from a time range.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the sub-clip.
    /// * `start_time` - The start time of the sub-clip.
    /// * `end_time` - The end time of the sub-clip.
    ///
    /// # Returns
    ///
    /// A new [`AnimationClip`] instance representing the sub-clip.
    pub fn create_sub_clip(&self, name: impl Into<String>, start_time: f32, end_time: f32) -> Self {
        if start_time >= end_time || start_time < 0.0 || end_time > self.duration {
            return Self::new(name);
        }

        let mut clip = Self::new(name);
        clip.duration = end_time - start_time;
        clip.loop_mode = self.loop_mode;
        clip.priority = self.priority;
        clip.blendable = self.blendable;

        // Create sub-tracks
        for (target, track) in &self.tracks {
            if let Some(sub_track) = track.create_sub_track(start_time, end_time) {
                clip.tracks.insert(target.clone(), sub_track);
            }
        }

        // Filter events
        clip.events = self
            .events
            .iter()
            .filter(|event| event.time >= start_time && event.time <= end_time)
            .map(|event| AnimationEvent {
                time: event.time - start_time,
                name: event.name.clone(),
                data: event.data.clone(),
            })
            .collect();

        clip
    }
}

impl Default for AnimationClip {
    /// Creates a default animation clip with the name "Default".
    ///
    /// # Returns
    ///
    /// The default [`AnimationClip`] instance.
    fn default() -> Self {
        Self::new("Default")
    }
}

/// Builder for creating animation clips.
pub struct AnimationClipBuilder {
    /// The animation clip being built0
    clip: AnimationClip,
}

impl AnimationClipBuilder {
    /// Creates a new builder instance.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation clip.
    ///
    /// # Returns
    ///
    /// A new [`AnimationClipBuilder`] instance.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            clip: AnimationClip::new(name),
        }
    }

    /// Sets the duration of the animation clip.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration in seconds.
    ///
    /// # Returns
    ///
    /// The updated [`AnimationClipBuilder`] instance.
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.clip.duration = duration;
        self
    }

    /// Sets the loop mode of the animation clip.
    ///
    /// # Arguments
    ///
    /// * `loop_mode` - The loop mode to set.
    ///
    /// # Returns
    ///
    /// The updated [`AnimationClipBuilder`] instance.
    pub fn with_loop_mode(mut self, loop_mode: LoopMode) -> Self {
        self.clip.loop_mode = loop_mode;
        self
    }

    /// Sets the priority of the animation clip.
    ///
    /// # Arguments
    ///
    /// * `priority` - The priority to set.
    ///
    /// # Returns
    ///
    /// The updated [`AnimationClipBuilder`] instance.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.clip.priority = priority;
        self
    }

    /// Sets whether the animation clip is blendable.
    ///
    /// # Arguments
    ///
    /// * `blendable` - Whether the clip is blendable.
    ///
    /// # Returns
    ///
    /// The updated [`AnimationClipBuilder`] instance.
    pub fn with_blendable(mut self, blendable: bool) -> Self {
        self.clip.blendable = blendable;
        self
    }

    /// Adds an animation track to the clip.
    ///
    /// # Arguments
    ///
    /// * `target` - The target that this track affects.
    /// * `track` - The animation track to add.
    ///
    /// # Returns
    ///
    /// The updated [`AnimationClipBuilder`] instance.
    pub fn add_track(mut self, target: AnimationTarget, track: AnimationTrack) -> Self {
        self.clip.add_track(target, track);
        self
    }

    /// Adds an animation event to the clip.
    ///
    /// # Arguments
    ///
    /// * `event` - The animation event to add.
    ///
    /// # Returns
    ///
    /// The updated [`AnimationClipBuilder`] instance.
    pub fn add_event(mut self, event: AnimationEvent) -> Self {
        self.clip.add_event(event);
        self
    }

    /// Builds and returns the animation clip.
    ///
    /// # Returns
    ///
    /// The built [`AnimationClip`] instance.
    pub fn build(self) -> AnimationClip {
        self.clip
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{AnimationValue, Keyframe};

    #[test]
    fn test_clip_creation() {
        let clip = AnimationClip::new("test_clip");
        assert_eq!(clip.name, "test_clip");
        assert_eq!(clip.duration, 0.0);
        assert!(clip.tracks.is_empty());
        assert!(clip.events.is_empty());
    }

    #[test]
    fn test_clip_loop_time_calculation() {
        let mut clip = AnimationClip::with_duration("test", 2.0);
        clip.loop_mode = LoopMode::Repeat;

        let (time, finished) = clip.calculate_loop_time(5.0);
        assert_eq!(time, 1.0); // 5.0 % 2.0 = 1.0
        assert!(!finished);

        clip.loop_mode = LoopMode::Once;
        let (time, finished) = clip.calculate_loop_time(3.0);
        assert_eq!(time, 2.0);
        assert!(finished);
    }

    #[test]
    fn test_clip_builder() {
        let clip = AnimationClipBuilder::new("builder_test")
            .with_duration(5.0)
            .with_loop_mode(LoopMode::Repeat)
            .with_priority(10)
            .build();

        assert_eq!(clip.name, "builder_test");
        assert_eq!(clip.duration, 5.0);
        assert_eq!(clip.loop_mode, LoopMode::Repeat);
        assert_eq!(clip.priority, 10);
    }
}

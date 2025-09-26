//! Animation timeline for keyframe editing and precise timing control.

use super::{
    AnimationClip, AnimationEvent, AnimationTarget, AnimationTrack, AnimationValue, Keyframe,
};
use std::collections::HashMap;
use std::time::Duration;

/// Represents a marker on the timeline for navigation and editing
#[derive(Debug, Clone)]
pub struct TimelineMarker {
    /// Time position of the marker
    pub time: f32,
    /// Optional label for the marker
    pub label: Option<String>,
    /// Color for visual representation (RGBA)
    pub color: [f32; 4],
}

impl TimelineMarker {
    pub fn new(time: f32) -> Self {
        Self {
            time,
            label: None,
            color: [1.0, 1.0, 1.0, 1.0], // White by default
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

/// Timeline editing operations for keyframes
#[derive(Debug, Clone)]
pub enum TimelineOperation {
    /// Add a new keyframe
    AddKeyframe {
        target: AnimationTarget,
        keyframe: Keyframe,
    },
    /// Remove a keyframe at specific time
    RemoveKeyframe {
        target: AnimationTarget,
        time: f32,
        tolerance: f32,
    },
    /// Move a keyframe to a new time
    MoveKeyframe {
        target: AnimationTarget,
        old_time: f32,
        new_time: f32,
        tolerance: f32,
    },
    /// Modify keyframe value
    ModifyKeyframe {
        target: AnimationTarget,
        time: f32,
        new_value: AnimationValue,
        tolerance: f32,
    },
    /// Scale time range
    ScaleTime {
        start_time: f32,
        end_time: f32,
        scale_factor: f32,
    },
    /// Offset time range
    OffsetTime {
        start_time: f32,
        end_time: f32,
        offset: f32,
    },
    /// Add event
    AddEvent { event: AnimationEvent },
    /// Remove event
    RemoveEvent { time: f32, tolerance: f32 },
}

/// Timeline viewport settings for visualization
#[derive(Debug, Clone)]
pub struct TimelineViewport {
    /// Start time visible in the viewport
    pub start_time: f32,
    /// End time visible in the viewport
    pub end_time: f32,
    /// Zoom level (1.0 = normal, >1.0 = zoomed in, <1.0 = zoomed out)
    pub zoom: f32,
    /// Whether to show keyframes
    pub show_keyframes: bool,
    /// Whether to show events
    pub show_events: bool,
    /// Whether to show markers
    pub show_markers: bool,
    /// Whether to snap to grid/keyframes
    pub snap_enabled: bool,
    /// Grid snap interval
    pub snap_interval: f32,
}

impl Default for TimelineViewport {
    fn default() -> Self {
        Self {
            start_time: 0.0,
            end_time: 10.0,
            zoom: 1.0,
            show_keyframes: true,
            show_events: true,
            show_markers: true,
            snap_enabled: true,
            snap_interval: 0.1, // 100ms grid
        }
    }
}

impl TimelineViewport {
    /// Check if a time is visible in the current viewport
    pub fn is_time_visible(&self, time: f32) -> bool {
        time >= self.start_time && time <= self.end_time
    }

    /// Get the duration of the current viewport
    pub fn duration(&self) -> f32 {
        self.end_time - self.start_time
    }

    /// Zoom to fit a specific time range
    pub fn zoom_to_fit(&mut self, start_time: f32, end_time: f32) {
        self.start_time = start_time;
        self.end_time = end_time;
        self.zoom = 1.0;
    }

    /// Zoom in/out by a factor at a specific time point
    pub fn zoom_at_time(&mut self, time: f32, zoom_factor: f32) {
        let new_zoom = (self.zoom * zoom_factor).clamp(0.1, 100.0);
        let zoom_ratio = new_zoom / self.zoom;

        let duration = self.duration();
        let new_duration = duration / zoom_ratio;

        // Keep the zoom center at the specified time
        let time_ratio = (time - self.start_time) / duration;
        self.start_time = time - new_duration * time_ratio;
        self.end_time = self.start_time + new_duration;
        self.zoom = new_zoom;
    }

    /// Pan the viewport by a time offset
    pub fn pan(&mut self, offset: f32) {
        self.start_time += offset;
        self.end_time += offset;
    }

    /// Snap a time value to the grid if snapping is enabled
    pub fn snap_time(&self, time: f32) -> f32 {
        if self.snap_enabled && self.snap_interval > 0.0 {
            (time / self.snap_interval).round() * self.snap_interval
        } else {
            time
        }
    }
}

/// Undo/Redo history for timeline operations
#[derive(Debug)]
pub struct TimelineHistory {
    /// History of operations (for undo)
    undo_stack: Vec<TimelineOperation>,
    /// Redo stack
    redo_stack: Vec<TimelineOperation>,
    /// Maximum history size
    max_history_size: usize,
}

impl TimelineHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_size: 100,
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_history_size = max_size;
        self
    }

    /// Record an operation for undo/redo
    pub fn record_operation(&mut self, operation: TimelineOperation) {
        self.undo_stack.push(operation);
        self.redo_stack.clear(); // Clear redo stack when new operation is recorded

        // Limit history size
        if self.undo_stack.len() > self.max_history_size {
            self.undo_stack.remove(0);
        }
    }

    /// Get the last operation for undo
    pub fn undo(&mut self) -> Option<TimelineOperation> {
        if let Some(operation) = self.undo_stack.pop() {
            self.redo_stack.push(operation.clone());
            Some(operation)
        } else {
            None
        }
    }

    /// Get an operation for redo
    pub fn redo(&mut self) -> Option<TimelineOperation> {
        if let Some(operation) = self.redo_stack.pop() {
            self.undo_stack.push(operation.clone());
            Some(operation)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for TimelineHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Main timeline structure for animation editing
#[derive(Debug)]
pub struct AnimationTimeline {
    /// The animation clip being edited
    clip: AnimationClip,
    /// Current playback time (playhead position)
    current_time: f32,
    /// Whether the timeline is playing
    playing: bool,
    /// Playback speed multiplier
    playback_speed: f32,
    /// Whether to loop playback
    loop_playback: bool,
    /// Timeline markers
    markers: Vec<TimelineMarker>,
    /// Viewport settings
    viewport: TimelineViewport,
    /// Operation history for undo/redo
    history: TimelineHistory,
    /// Selected keyframes (target, time)
    selected_keyframes: Vec<(AnimationTarget, f32)>,
    /// Selected events (time)
    selected_events: Vec<f32>,
    /// Whether the timeline is in edit mode
    edit_mode: bool,
}

impl AnimationTimeline {
    /// Create a new animation timeline
    pub fn new(clip: AnimationClip) -> Self {
        Self {
            clip,
            current_time: 0.0,
            playing: false,
            playback_speed: 1.0,
            loop_playback: false,
            markers: Vec::new(),
            viewport: TimelineViewport::default(),
            history: TimelineHistory::new(),
            selected_keyframes: Vec::new(),
            selected_events: Vec::new(),
            edit_mode: false,
        }
    }

    /// Get a reference to the animation clip
    pub fn clip(&self) -> &AnimationClip {
        &self.clip
    }

    /// Get a mutable reference to the animation clip
    pub fn clip_mut(&mut self) -> &mut AnimationClip {
        &mut self.clip
    }

    /// Set the current playback time
    pub fn set_current_time(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.clip.duration);
    }

    /// Get the current playback time
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Start playback
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Toggle playback
    pub fn toggle_playback(&mut self) {
        self.playing = !self.playing;
    }

    /// Set playback speed
    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.0);
    }

    /// Enable or disable loop playback
    pub fn set_loop_playback(&mut self, enable: bool) {
        self.loop_playback = enable;
    }

    /// Update the timeline (advance playback time)
    pub fn update(&mut self, dt: Duration) {
        if self.playing {
            self.current_time += dt.as_secs_f32() * self.playback_speed;

            if self.current_time >= self.clip.duration {
                if self.loop_playback {
                    self.current_time = 0.0;
                } else {
                    self.current_time = self.clip.duration;
                    self.playing = false;
                }
            }
        }
    }

    /// Apply a timeline operation
    pub fn apply_operation(&mut self, operation: TimelineOperation) -> Result<(), String> {
        // Record operation for undo/redo (except for some operations)
        let should_record = matches!(
            operation,
            TimelineOperation::AddKeyframe { .. }
                | TimelineOperation::RemoveKeyframe { .. }
                | TimelineOperation::MoveKeyframe { .. }
                | TimelineOperation::ModifyKeyframe { .. }
                | TimelineOperation::AddEvent { .. }
                | TimelineOperation::RemoveEvent { .. }
        );

        if should_record {
            self.history.record_operation(operation.clone());
        }

        match operation {
            TimelineOperation::AddKeyframe { target, keyframe } => {
                if let Some(track) = self.clip.tracks.get_mut(&target) {
                    track.add_keyframe(keyframe);
                } else {
                    let mut new_track = AnimationTrack::new();
                    new_track.add_keyframe(keyframe);
                    self.clip.tracks.insert(target, new_track);
                }
            }
            TimelineOperation::RemoveKeyframe {
                target,
                time,
                tolerance,
            } => {
                if let Some(track) = self.clip.tracks.get_mut(&target) {
                    // Find keyframe within tolerance
                    if let Some(index) = track
                        .keyframes
                        .iter()
                        .position(|k| (k.time - time).abs() <= tolerance)
                    {
                        track.remove_keyframe(index);
                    }
                }
            }
            TimelineOperation::MoveKeyframe {
                target,
                old_time,
                new_time,
                tolerance,
            } => {
                if let Some(track) = self.clip.tracks.get_mut(&target) {
                    if let Some(keyframe) = track
                        .keyframes
                        .iter_mut()
                        .find(|k| (k.time - old_time).abs() <= tolerance)
                    {
                        keyframe.time = new_time;
                        // Re-sort keyframes
                        track
                            .keyframes
                            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
                    }
                }
            }
            TimelineOperation::ModifyKeyframe {
                target,
                time,
                new_value,
                tolerance,
            } => {
                if let Some(track) = self.clip.tracks.get_mut(&target) {
                    if let Some(keyframe) = track
                        .keyframes
                        .iter_mut()
                        .find(|k| (k.time - time).abs() <= tolerance)
                    {
                        keyframe.value = new_value;
                    }
                }
            }
            TimelineOperation::ScaleTime {
                start_time,
                end_time,
                scale_factor,
            } => {
                for track in self.clip.tracks.values_mut() {
                    for keyframe in &mut track.keyframes {
                        if keyframe.time >= start_time && keyframe.time <= end_time {
                            let relative_time = keyframe.time - start_time;
                            keyframe.time = start_time + relative_time * scale_factor;
                        }
                    }
                }

                // Scale events in the same range
                for event in &mut self.clip.events {
                    if event.time >= start_time && event.time <= end_time {
                        let relative_time = event.time - start_time;
                        event.time = start_time + relative_time * scale_factor;
                    }
                }
            }
            TimelineOperation::OffsetTime {
                start_time,
                end_time,
                offset,
            } => {
                for track in self.clip.tracks.values_mut() {
                    for keyframe in &mut track.keyframes {
                        if keyframe.time >= start_time && keyframe.time <= end_time {
                            keyframe.time += offset;
                        }
                    }
                }

                // Offset events in the same range
                for event in &mut self.clip.events {
                    if event.time >= start_time && event.time <= end_time {
                        event.time += offset;
                    }
                }
            }
            TimelineOperation::AddEvent { event } => {
                self.clip.add_event(event);
            }
            TimelineOperation::RemoveEvent { time, tolerance } => {
                self.clip
                    .events
                    .retain(|event| (event.time - time).abs() > tolerance);
            }
        }

        Ok(())
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> Result<(), String> {
        if let Some(_operation) = self.history.undo() {
            unimplemented!("Undo not implemented")
        } else {
            Err("Nothing to undo".to_string())
        }
    }

    /// Redo the last undone operation
    pub fn redo(&mut self) -> Result<(), String> {
        if let Some(operation) = self.history.redo() {
            self.apply_operation(operation)
        } else {
            Err("Nothing to redo".to_string())
        }
    }

    /// Add a timeline marker
    pub fn add_marker(&mut self, marker: TimelineMarker) {
        // Insert in sorted order by time
        let insert_pos = self
            .markers
            .iter()
            .position(|m| m.time > marker.time)
            .unwrap_or(self.markers.len());
        self.markers.insert(insert_pos, marker);
    }

    /// Remove a timeline marker
    pub fn remove_marker(&mut self, time: f32, tolerance: f32) -> bool {
        if let Some(index) = self
            .markers
            .iter()
            .position(|m| (m.time - time).abs() <= tolerance)
        {
            self.markers.remove(index);
            true
        } else {
            false
        }
    }

    /// Get all markers in a time range
    pub fn get_markers_in_range(&self, start_time: f32, end_time: f32) -> Vec<&TimelineMarker> {
        self.markers
            .iter()
            .filter(|marker| marker.time >= start_time && marker.time <= end_time)
            .collect()
    }

    /// Select keyframes in a time range
    pub fn select_keyframes_in_range(
        &mut self,
        target: &AnimationTarget,
        start_time: f32,
        end_time: f32,
    ) {
        self.selected_keyframes.clear();

        if let Some(track) = self.clip.tracks.get(target) {
            for keyframe in &track.keyframes {
                if keyframe.time >= start_time && keyframe.time <= end_time {
                    self.selected_keyframes
                        .push((target.clone(), keyframe.time));
                }
            }
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_keyframes.clear();
        self.selected_events.clear();
    }

    /// Get the viewport settings
    pub fn viewport(&self) -> &TimelineViewport {
        &self.viewport
    }

    /// Get mutable viewport settings
    pub fn viewport_mut(&mut self) -> &mut TimelineViewport {
        &mut self.viewport
    }

    /// Set edit mode
    pub fn set_edit_mode(&mut self, edit_mode: bool) {
        self.edit_mode = edit_mode;
    }

    /// Check if in edit mode
    pub fn is_edit_mode(&self) -> bool {
        self.edit_mode
    }

    /// Sample the animation at the current time
    pub fn sample_current(&self) -> HashMap<AnimationTarget, AnimationValue> {
        self.clip.sample(self.current_time)
    }

    /// Sample the animation at a specific time
    pub fn sample_at_time(&self, time: f32) -> HashMap<AnimationTarget, AnimationValue> {
        self.clip.sample(time)
    }

    /// Get all keyframe times for a specific target
    pub fn get_keyframe_times(&self, target: &AnimationTarget) -> Vec<f32> {
        if let Some(track) = self.clip.tracks.get(target) {
            track.keyframes.iter().map(|k| k.time).collect()
        } else {
            Vec::new()
        }
    }

    /// Get all event times
    pub fn get_event_times(&self) -> Vec<f32> {
        self.clip.events.iter().map(|e| e.time).collect()
    }

    /// Find the nearest keyframe time to a given time
    pub fn find_nearest_keyframe_time(&self, target: &AnimationTarget, time: f32) -> Option<f32> {
        if let Some(track) = self.clip.tracks.get(target) {
            track
                .keyframes
                .iter()
                .min_by(|a, b| {
                    (a.time - time)
                        .abs()
                        .partial_cmp(&(b.time - time).abs())
                        .unwrap()
                })
                .map(|k| k.time)
        } else {
            None
        }
    }

    /// Jump to the next keyframe
    pub fn jump_to_next_keyframe(&mut self, target: &AnimationTarget) {
        if let Some(track) = self.clip.tracks.get(target) {
            if let Some(next_keyframe) = track.keyframes.iter().find(|k| k.time > self.current_time)
            {
                self.set_current_time(next_keyframe.time);
            }
        }
    }

    /// Jump to the previous keyframe
    pub fn jump_to_previous_keyframe(&mut self, target: &AnimationTarget) {
        if let Some(track) = self.clip.tracks.get(target) {
            if let Some(prev_keyframe) = track
                .keyframes
                .iter()
                .rev()
                .find(|k| k.time < self.current_time)
            {
                self.set_current_time(prev_keyframe.time);
            }
        }
    }

    /// Get the total number of keyframes across all tracks
    pub fn total_keyframe_count(&self) -> usize {
        self.clip
            .tracks
            .values()
            .map(|track| track.keyframe_count())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{AnimationTarget, AnimationValue};

    #[test]
    fn test_timeline_creation() {
        let clip = AnimationClip::new("test_clip");
        let timeline = AnimationTimeline::new(clip);

        assert_eq!(timeline.current_time(), 0.0);
        assert!(!timeline.playing);
        assert_eq!(timeline.playback_speed, 1.0);
    }

    #[test]
    fn test_timeline_playback() {
        let clip = AnimationClip::with_duration("test_clip", 2.0);
        let mut timeline = AnimationTimeline::new(clip);

        timeline.play();
        assert!(timeline.playing);

        timeline.update(Duration::from_millis(500));
        assert_eq!(timeline.current_time(), 0.5);

        timeline.pause();
        assert!(!timeline.playing);
    }

    #[test]
    fn test_keyframe_operations() {
        let clip = AnimationClip::new("test_clip");
        let mut timeline = AnimationTimeline::new(clip);

        let operation = TimelineOperation::AddKeyframe {
            target: AnimationTarget::Translation,
            keyframe: Keyframe::new(1.0, AnimationValue::Float(1.0)),
        };

        timeline.apply_operation(operation).unwrap();

        assert!(
            timeline
                .clip()
                .tracks
                .contains_key(&AnimationTarget::Translation)
        );
    }

    #[test]
    fn test_timeline_markers() {
        let clip = AnimationClip::new("test_clip");
        let mut timeline = AnimationTimeline::new(clip);

        let marker = TimelineMarker::new(1.5).with_label("Test Marker");
        timeline.add_marker(marker);

        let markers = timeline.get_markers_in_range(1.0, 2.0);
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].time, 1.5);
    }

    #[test]
    fn test_viewport_operations() {
        let mut viewport = TimelineViewport::default();

        viewport.zoom_to_fit(2.0, 8.0);
        assert_eq!(viewport.start_time, 2.0);
        assert_eq!(viewport.end_time, 8.0);

        viewport.pan(1.0);
        assert_eq!(viewport.start_time, 3.0);
        assert_eq!(viewport.end_time, 9.0);
    }
}

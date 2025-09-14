//! # Pathfinding Components and A* Algorithm Implementation
//!
//! This module provides A* pathfinding capabilities that integrate with the ECS collision system.
//! It includes components for pathfinding behavior and a grid-based A* implementation that
//! considers collision boxes from entities in the world.
//!
//! ## Features
//!
//! - **Grid-based A* Algorithm**: Efficient pathfinding with configurable grid resolution
//! - **Collision Awareness**: Considers collision boxes from entities when building the grid
//! - **ECS Integration**: Components that work seamlessly with the existing ECS architecture
//! - **Target Following**: Automatic path recalculation when targets move
//! - **Movement Integration**: Smooth movement along calculated paths
//!
//! ## Example Usage
//!
//! ```rust
//! use gears_ecs::components::pathfinding::*;
//!
//! // Create a pathfinding component
//! let pathfinder = PathfindingComponent::new(
//!     cgmath::Vector3::new(10.0, 0.0, 10.0), // target position
//!     2.0,  // movement speed
//!     0.5   // grid cell size
//! );
//!
//! // Add to an entity
//! new_entity!(
//!     app,
//!     Name("Enemy"),
//!     Pos3::default(),
//!     pathfinder,
//!     EnemyMarker,
//! );
//! ```

use crate::Component;
use crate::components::physics::AABBCollisionBox;
use cgmath::InnerSpace;
use gears_macro::Component;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Grid position used for A* pathfinding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Convert world position to grid position
    pub fn from_world_pos(world_pos: cgmath::Vector3<f32>, cell_size: f32) -> Self {
        Self {
            x: (world_pos.x / cell_size).round() as i32,
            y: (world_pos.y / cell_size).round() as i32,
            z: (world_pos.z / cell_size).round() as i32,
        }
    }

    /// Convert grid position to world position
    pub fn to_world_pos(self, cell_size: f32) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.x as f32 * cell_size,
            self.y as f32 * cell_size,
            self.z as f32 * cell_size,
        )
    }

    /// Calculate Manhattan distance between two grid positions
    pub fn manhattan_distance(self, other: GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    /// Get all 26 neighbors in 3D space (including diagonals)
    pub fn get_neighbors(self) -> Vec<GridPos> {
        let mut neighbors = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue; // Skip the center position
                    }
                    neighbors.push(GridPos::new(self.x + dx, self.y + dy, self.z + dz));
                }
            }
        }
        neighbors
    }

    /// Get 6 neighbors (face-connected only, no diagonals)
    pub fn get_face_neighbors(self) -> Vec<GridPos> {
        vec![
            GridPos::new(self.x + 1, self.y, self.z),
            GridPos::new(self.x - 1, self.y, self.z),
            GridPos::new(self.x, self.y + 1, self.z),
            GridPos::new(self.x, self.y - 1, self.z),
            GridPos::new(self.x, self.y, self.z + 1),
            GridPos::new(self.x, self.y, self.z - 1),
        ]
    }
}

/// Node used in the A* algorithm priority queue
#[derive(Debug, Clone)]
struct AStarNode {
    position: GridPos,
    g_cost: f32, // Distance from start
    h_cost: f32, // Heuristic distance to goal
    #[allow(dead_code)] // Used conceptually in algorithm but not directly accessed
    parent: Option<GridPos>,
}

impl AStarNode {
    fn new(position: GridPos, g_cost: f32, h_cost: f32, parent: Option<GridPos>) -> Self {
        Self {
            position,
            g_cost,
            h_cost,
            parent,
        }
    }

    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other
            .f_cost()
            .partial_cmp(&self.f_cost())
            .unwrap_or(Ordering::Equal)
    }
}

/// Pathfinding grid that tracks obstacles based on entity collision boxes
#[derive(Debug, Clone)]
pub struct PathfindingGrid {
    obstacles: HashSet<GridPos>,
    cell_size: f32,
    /// Bounds of the grid (min and max positions)
    bounds: Option<(GridPos, GridPos)>,
}

impl PathfindingGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            obstacles: HashSet::new(),
            cell_size,
            bounds: None,
        }
    }

    /// Add an obstacle at the given world position with the specified collision box
    pub fn add_obstacle(
        &mut self,
        world_pos: cgmath::Vector3<f32>,
        collision_box: &AABBCollisionBox,
    ) {
        // Calculate the grid cells that this collision box occupies
        let min_world = world_pos + collision_box.min;
        let max_world = world_pos + collision_box.max;

        let min_grid = GridPos::from_world_pos(min_world, self.cell_size);
        let max_grid = GridPos::from_world_pos(max_world, self.cell_size);

        // Mark all grid cells within the bounding box as obstacles
        for x in min_grid.x..=max_grid.x {
            for y in min_grid.y..=max_grid.y {
                for z in min_grid.z..=max_grid.z {
                    self.obstacles.insert(GridPos::new(x, y, z));
                }
            }
        }

        // Update bounds
        if let Some((mut bounds_min, mut bounds_max)) = self.bounds {
            bounds_min.x = bounds_min.x.min(min_grid.x);
            bounds_min.y = bounds_min.y.min(min_grid.y);
            bounds_min.z = bounds_min.z.min(min_grid.z);
            bounds_max.x = bounds_max.x.max(max_grid.x);
            bounds_max.y = bounds_max.y.max(max_grid.y);
            bounds_max.z = bounds_max.z.max(max_grid.z);
            self.bounds = Some((bounds_min, bounds_max));
        } else {
            self.bounds = Some((min_grid, max_grid));
        }
    }

    /// Check if a grid position is an obstacle
    pub fn is_obstacle(&self, pos: GridPos) -> bool {
        self.obstacles.contains(&pos)
    }

    /// Clear all obstacles
    pub fn clear_obstacles(&mut self) {
        self.obstacles.clear();
        self.bounds = None;
    }

    /// Get the cell size
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }
}

/// A* pathfinding algorithm implementation
pub struct AStar {
    grid: PathfindingGrid,
}

impl AStar {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: PathfindingGrid::new(cell_size),
        }
    }

    /// Build the pathfinding grid from entities in the world
    pub fn build_grid_from_entities<'a, I>(&mut self, entities: I)
    where
        I: Iterator<Item = (&'a cgmath::Vector3<f32>, &'a AABBCollisionBox)>,
    {
        self.grid.clear_obstacles();

        for (world_pos, collision_box) in entities {
            self.grid.add_obstacle(*world_pos, collision_box);
        }
    }

    /// Find a path from start to goal using A* algorithm
    pub fn find_path(
        &self,
        start: cgmath::Vector3<f32>,
        goal: cgmath::Vector3<f32>,
    ) -> Option<Vec<cgmath::Vector3<f32>>> {
        let start_grid = GridPos::from_world_pos(start, self.grid.cell_size);
        let goal_grid = GridPos::from_world_pos(goal, self.grid.cell_size);

        // Early return if start or goal is an obstacle
        if self.grid.is_obstacle(start_grid) || self.grid.is_obstacle(goal_grid) {
            return None;
        }

        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut g_scores = HashMap::new();

        // Initialize start node
        let h_cost = self.heuristic(start_grid, goal_grid);
        open_set.push(AStarNode::new(start_grid, 0.0, h_cost, None));
        g_scores.insert(start_grid, 0.0);

        let mut came_from = HashMap::new();

        while let Some(current) = open_set.pop() {
            let current_pos = current.position;

            // Check if we've reached the goal
            if current_pos == goal_grid {
                return Some(self.reconstruct_path(&came_from, current_pos));
            }

            closed_set.insert(current_pos);

            // Check all neighbors (using face neighbors for simpler pathfinding)
            for neighbor_pos in current_pos.get_face_neighbors() {
                if closed_set.contains(&neighbor_pos) || self.grid.is_obstacle(neighbor_pos) {
                    continue;
                }

                let movement_cost = self.get_movement_cost(current_pos, neighbor_pos);
                let tentative_g_score = current.g_cost + movement_cost;

                let neighbor_g_score = g_scores
                    .get(&neighbor_pos)
                    .copied()
                    .unwrap_or(f32::INFINITY);

                if tentative_g_score < neighbor_g_score {
                    came_from.insert(neighbor_pos, current_pos);
                    g_scores.insert(neighbor_pos, tentative_g_score);

                    let h_cost = self.heuristic(neighbor_pos, goal_grid);
                    open_set.push(AStarNode::new(
                        neighbor_pos,
                        tentative_g_score,
                        h_cost,
                        Some(current_pos),
                    ));
                }
            }
        }

        None // No path found
    }

    /// Heuristic function for A* (Manhattan distance)
    fn heuristic(&self, from: GridPos, to: GridPos) -> f32 {
        from.manhattan_distance(to) as f32
    }

    /// Calculate movement cost between two adjacent grid positions
    fn get_movement_cost(&self, _from: GridPos, _to: GridPos) -> f32 {
        // Basic movement cost - can be extended for different terrain types
        1.0
    }

    /// Reconstruct the path from came_from map
    fn reconstruct_path(
        &self,
        came_from: &HashMap<GridPos, GridPos>,
        mut current: GridPos,
    ) -> Vec<cgmath::Vector3<f32>> {
        let mut path = Vec::new();

        loop {
            path.push(current.to_world_pos(self.grid.cell_size));

            if let Some(&parent) = came_from.get(&current) {
                current = parent;
            } else {
                break;
            }
        }

        path.reverse();
        path
    }
}

/// Component for entities that use pathfinding
#[derive(Component, Debug, Clone)]
pub struct PathfindingComponent {
    /// Current target position to path towards
    pub target: cgmath::Vector3<f32>,
    /// Current calculated path (list of waypoints)
    pub path: Vec<cgmath::Vector3<f32>>,
    /// Current waypoint index
    pub current_waypoint: usize,
    /// Movement speed
    pub speed: f32,
    /// Grid cell size for pathfinding
    pub cell_size: f32,
    /// Distance threshold to consider a waypoint "reached"
    pub waypoint_threshold: f32,
    /// Time since last path calculation
    pub time_since_last_path: f32,
    /// How often to recalculate path (in seconds)
    pub path_recalc_interval: f32,
    /// Whether pathfinding is currently active
    pub active: bool,
}

impl PathfindingComponent {
    pub fn new(target: cgmath::Vector3<f32>, speed: f32, cell_size: f32) -> Self {
        Self {
            target,
            path: Vec::new(),
            current_waypoint: 0,
            speed,
            cell_size,
            waypoint_threshold: cell_size * 0.5, // Half a grid cell
            time_since_last_path: 0.0,
            path_recalc_interval: 3.0, // Recalculate every 3 seconds (much less frequent)
            active: true,
        }
    }

    /// Set a new target and clear the current path
    pub fn set_target(&mut self, new_target: cgmath::Vector3<f32>) {
        // Only force recalculation if target moved significantly
        let distance_moved = (self.target - new_target).magnitude();
        if distance_moved > self.cell_size * 2.0 {
            // Only if target moved more than 2 grid cells
            self.target = new_target;
            self.path.clear();
            self.current_waypoint = 0;
            self.time_since_last_path = self.path_recalc_interval; // Force recalculation
        } else {
            self.target = new_target; // Update target but don't force recalculation
        }
    }

    /// Update the pathfinding component
    pub fn update(&mut self, dt: f32) {
        self.time_since_last_path += dt;
    }

    /// Check if it's time to recalculate the path
    pub fn should_recalculate_path(&self) -> bool {
        self.active
            && (self.path.is_empty() || self.time_since_last_path >= self.path_recalc_interval)
    }

    /// Check if pathfinding is needed (target is far enough away)
    pub fn needs_pathfinding(&self, current_pos: cgmath::Vector3<f32>) -> bool {
        let distance_to_target = (self.target - current_pos).magnitude();
        distance_to_target > self.cell_size * 3.0 // Only pathfind if target is more than 3 cells away
    }

    /// Set a new path and reset pathfinding state
    pub fn set_path(&mut self, new_path: Vec<cgmath::Vector3<f32>>) {
        self.path = new_path;
        self.current_waypoint = 0;
        self.time_since_last_path = 0.0;
    }

    /// Get the current waypoint, if any
    pub fn current_waypoint(&self) -> Option<cgmath::Vector3<f32>> {
        self.path.get(self.current_waypoint).copied()
    }

    /// Move to the next waypoint
    pub fn advance_waypoint(&mut self) -> bool {
        if self.current_waypoint + 1 < self.path.len() {
            self.current_waypoint += 1;
            true
        } else {
            false // Reached end of path
        }
    }

    /// Check if the entity has reached its target
    pub fn has_reached_target(&self, current_pos: cgmath::Vector3<f32>) -> bool {
        let distance = (self.target - current_pos).magnitude();
        distance <= self.waypoint_threshold
    }

    /// Check if the entity has reached the current waypoint
    pub fn has_reached_current_waypoint(&self, current_pos: cgmath::Vector3<f32>) -> bool {
        if let Some(waypoint) = self.current_waypoint() {
            let distance = (waypoint - current_pos).magnitude();
            distance <= self.waypoint_threshold
        } else {
            false
        }
    }
}

impl Default for PathfindingComponent {
    fn default() -> Self {
        Self::new(cgmath::Vector3::new(0.0, 0.0, 0.0), 5.0, 1.0)
    }
}

/// Marker component for entities that should be tracked by pathfinding entities
#[derive(Component, Debug, Clone, Copy)]
pub struct PathfindingTarget;

/// Marker component for entities that use pathfinding to follow targets
#[derive(Component, Debug, Clone, Copy)]
pub struct PathfindingFollower;

use crate::Component;
use crate::components::physics::AABBCollisionBox;
use cgmath::InnerSpace;
use gears_macro::Component;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Distance heuristic used for pathfinding
///
/// Determines the heuristic used to estimate the distance between two grid positions.
/// Default is Manhattan distance.
#[derive(Debug, Copy, Clone, Default)]
pub enum DistanceHeuristic {
    #[default]
    Manhattan,
    Euclidean,
    Chebyshev,
}

/// Grid position used for A* pathfinding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    /// The x-coordinate of the grid position.
    pub x: i32,
    /// The y-coordinate of the grid position.
    pub y: i32,
    /// The z-coordinate of the grid position.
    pub z: i32,
}

impl GridPos {
    /// Create a new grid position.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the grid position.
    /// * `y` - The y-coordinate of the grid position.
    /// * `z` - The z-coordinate of the grid position.
    ///
    /// # Returns
    ///
    /// The new [`GridPos`] instance.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Convert world position to grid position.
    ///
    /// # Arguments
    ///
    /// * `world_pos` - The world position to convert.
    /// * `cell_size` - The size of each cell in the grid.
    ///
    /// # Returns
    ///
    /// The grid position corresponding to the given world position.
    pub fn from_world_pos(world_pos: cgmath::Vector3<f32>, cell_size: f32) -> Self {
        Self {
            x: (world_pos.x / cell_size).round() as i32,
            y: (world_pos.y / cell_size).round() as i32,
            z: (world_pos.z / cell_size).round() as i32,
        }
    }

    /// Calculate the distance between two grid positions using the specified heuristic.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting grid position.
    /// * `to` - The target grid position.
    /// * `heuristic` - The distance heuristic to use.
    ///
    /// # Returns
    ///
    /// The distance between the two grid positions as an `f32`
    pub fn distance(from: GridPos, to: GridPos, heuristic: DistanceHeuristic) -> f32 {
        match heuristic {
            DistanceHeuristic::Manhattan => {
                (to.x - from.x).abs() as f32 + (to.y - from.y).abs() as f32
            }
            DistanceHeuristic::Euclidean => {
                (to.x - from.x).pow(2) as f32 + (to.y - from.y).pow(2) as f32
            }
            DistanceHeuristic::Chebyshev => {
                ((to.x - from.x).abs() as f32).max((to.y - from.y).abs() as f32)
            }
        }
    }

    /// Convert grid position to world position.
    ///
    /// # Arguments
    ///
    /// * `cell_size` - The size of each cell in the grid.
    ///
    /// # Returns
    ///
    /// A [`cgmath::Vector3<f32>`] representing the world position.
    pub fn to_world_pos(self, cell_size: f32) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.x as f32 * cell_size,
            self.y as f32 * cell_size,
            self.z as f32 * cell_size,
        )
    }

    /// Get all 26 neighbors in 3D space (including diagonals).
    ///
    /// # Returns
    ///
    /// A vector of [`GridPos`] representing the 26 neighbors.
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

    /// Get 6 neighbors (face-connected only, no diagonals).
    ///
    /// # Returns
    ///
    /// A vector of [`GridPos`] representing the 6 neighbors.
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
    /// The position of the node.
    position: GridPos,
    /// Distance from start
    g_cost: f32,
    /// Heuristic distance to goal
    h_cost: f32,
    #[allow(dead_code)] // Used conceptually in algorithm but not directly accessed
    parent: Option<GridPos>,
}

impl AStarNode {
    /// Create a new A* node.
    ///
    /// # Arguments
    ///
    /// * `position` - The position of the node.
    /// * `g_cost` - The distance from the start node.
    /// * `h_cost` - The heuristic distance to the goal node.
    /// * `parent` - The parent node.
    ///
    /// # Returns
    ///
    /// A new [`AStarNode`] instance.
    fn new(position: GridPos, g_cost: f32, h_cost: f32, parent: Option<GridPos>) -> Self {
        Self {
            position,
            g_cost,
            h_cost,
            parent,
        }
    }

    /// Calculate the f-cost of the node.
    ///
    /// # Returns
    ///
    /// The f-cost of the node as an `f32`.
    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for AStarNode {
    // Check if two A* nodes are equal based on their positions.
    //
    // # Arguments
    //
    // * `other` - The second A* node to compare.
    //
    // # Returns
    //
    // `true` if the positions are equal.
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    /// Compare two A* nodes based on their f-costs.
    ///
    /// # Arguments
    ///
    /// * `other` - The second A* node to compare.
    ///
    /// # Returns
    ///
    /// An `Ordering` representing the ordering of the two A* nodes.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    /// Compare two A* nodes based on their f-costs.
    ///
    /// # Arguments
    ///
    /// * `other` - The second A* node to compare.
    ///
    /// # Returns
    ///
    /// An `Ordering` representing the ordering of the two A* nodes.
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other
            .f_cost()
            .partial_cmp(&self.f_cost())
            .unwrap_or(Ordering::Equal)
    }
}

/// Pathfinding grid that tracks obstacles based on entity collision boxes.
#[derive(Debug, Clone, Default)]
pub struct PathfindingGrid {
    // Obstacles in grid coordinates.
    obstacles: HashSet<GridPos>,
    // Cell size in world units.
    cell_size: f32,
    /// Bounds of the grid (min and max positions).
    bounds: Option<(GridPos, GridPos)>,
}

impl PathfindingGrid {
    /// Create a new pathfinding grid with the given cell size.
    ///
    /// # Arguments
    ///
    /// * `cell_size` - The size of each grid cell in world units.
    ///
    /// # Returns
    ///
    /// A new [`PathfindingGrid`] instance.
    pub fn new(cell_size: f32) -> Self {
        Self {
            obstacles: HashSet::new(),
            cell_size,
            bounds: None,
        }
    }

    /// Add an obstacle at the given world position with the specified collision box.
    ///
    /// # Arguments
    ///
    /// * `world_pos` - The world position of the obstacle.
    /// * `collision_box` - The collision box of the obstacle.
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

    /// Check if a grid position is an obstacle.
    ///
    /// # Arguments
    ///
    /// * `pos` - The grid position to check.
    ///
    /// # Returns
    ///
    /// `true` if the position is an obstacle.
    pub fn is_obstacle(&self, pos: GridPos) -> bool {
        self.obstacles.contains(&pos)
    }

    /// Clear all obstacles.
    pub fn clear_obstacles(&mut self) {
        self.obstacles.clear();
        self.bounds = None;
    }

    /// Get the cell size.
    ///
    /// # Returns
    ///
    /// The cell size as a `f32`.
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }
}

/// A* pathfinding algorithm implementation.
#[derive(Debug, Default)]
pub struct AStar {
    /// The grid used for pathfinding.
    grid: PathfindingGrid,
    /// The distance heuristic to use.
    distance_heuristic: DistanceHeuristic,
}

impl AStar {
    /// Create a new A* pathfinding algorithm instance.
    ///
    /// # Arguments
    /// * `cell_size` - The size of each cell in the grid.
    /// * `distance_heuristic` - The distance heuristic to use.
    ///
    /// # Returns
    ///
    /// A new [`AStar`] instance.
    pub fn new(cell_size: f32, distance_heuristic: DistanceHeuristic) -> Self {
        Self {
            grid: PathfindingGrid::new(cell_size),
            distance_heuristic,
        }
    }

    /// Build the pathfinding grid from entities in the world.
    ///
    /// # Arguments
    /// * `entities` - An iterator over entities with their world position and collision box.
    ///
    /// # Returns
    ///
    /// The path as a `Vec<cgmath::Vector3<f32>>` if one was found.
    pub fn build_grid_from_entities<'a, I>(&mut self, entities: I)
    where
        I: Iterator<Item = (&'a cgmath::Vector3<f32>, &'a AABBCollisionBox)>,
    {
        self.grid.clear_obstacles();

        for (world_pos, collision_box) in entities {
            self.grid.add_obstacle(*world_pos, collision_box);
        }
    }

    /// Find a path from start to goal using A* algorithm.
    ///
    /// # Arguments
    /// * `start` - The starting position.
    /// * `goal` - The ending position.
    ///
    /// # Returns
    ///
    /// The path as a `Vec<cgmath::Vector3<f32>>` if one was found.
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

    /// Heuristic function for A*.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting grid position.
    /// * `to` - The ending grid position.
    ///
    /// # Returns
    ///
    /// The heuristic cost as a `f32`.
    fn heuristic(&self, from: GridPos, to: GridPos) -> f32 {
        GridPos::distance(from, to, self.distance_heuristic)
    }

    /// Calculate movement cost between two adjacent grid positions.
    ///
    /// # Arguments
    /// * `_from` - The starting grid position.
    /// * `_to` - The ending grid position.
    ///
    /// # Returns
    ///
    /// The movement cost as a `f32`.
    fn get_movement_cost(&self, _from: GridPos, _to: GridPos) -> f32 {
        // Basic movement cost - could be extended for different terrain types
        1.0
    }

    /// Reconstruct the path from `came_from` map.
    ///
    /// # Arguments
    /// * `came_from` - A map of grid positions to their parent positions.
    /// * `current` - The starting position for path reconstruction.
    ///
    /// # Returns
    /// A vector of world positions as `Vec<cgmath::Vector3<f32>>` representing the path.
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

/// Component for entities that use pathfinding.
#[derive(Component, Debug, Clone)]
pub struct PathfindingComponent {
    /// Current target position to path towards.
    pub target: cgmath::Vector3<f32>,
    /// Current calculated path (list of waypoints).
    pub path: Vec<cgmath::Vector3<f32>>,
    /// Current waypoint index.
    pub current_waypoint: usize,
    /// Movement speed.
    pub speed: f32,
    /// Grid cell size for pathfinding.
    pub cell_size: f32,
    /// Distance threshold to consider a waypoint "reached".
    pub waypoint_threshold: f32,
    /// Time since last path calculation.
    pub time_since_last_path: f32,
    /// How often to recalculate path (in seconds).
    pub path_recalc_interval: f32,
    /// Whether pathfinding is currently active.
    pub active: bool,
}

impl PathfindingComponent {
    /// Create a new pathfinding component with the given target, speed, and cell size.
    ///
    /// # Arguments
    ///
    /// * `target` - The initial target position.
    /// * `speed` - The movement speed.
    /// * `cell_size` - The grid cell size for pathfinding.
    ///
    /// # Returns
    ///
    /// A new [`PathfindingComponent`] instance.
    pub fn new(target: cgmath::Vector3<f32>, speed: f32, cell_size: f32) -> Self {
        Self {
            target,
            path: Vec::new(),
            current_waypoint: 0,
            speed,
            cell_size,
            waypoint_threshold: cell_size * 0.5, // Half a grid cell
            time_since_last_path: 0.0,
            path_recalc_interval: 1.5, // Recalculate every 1.5 seconds
            active: true,
        }
    }

    /// Set a new target and clear the current path.
    ///
    /// # Arguments
    ///
    /// * `new_target` - The new target position.
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

    /// Update the pathfinding component.
    ///
    /// # Arguments
    ///
    /// * `dt` - The time elapsed since the last update.
    pub fn update(&mut self, dt: f32) {
        self.time_since_last_path += dt;
    }

    /// Check if it's time to recalculate the path.
    ///
    /// # Returns
    ///
    /// `true` if it's time to recalculate the path.
    pub fn should_recalculate_path(&self) -> bool {
        self.active
            && (self.path.is_empty() || self.time_since_last_path >= self.path_recalc_interval)
    }

    /// Check if pathfinding is needed (target is far enough away).
    ///
    /// # Arguments
    ///
    /// * `current_pos` - The current position of the entity.
    ///
    /// # Returns
    ///
    /// `true` if pathfinding is needed.
    pub fn needs_pathfinding(&self, current_pos: cgmath::Vector3<f32>) -> bool {
        let distance_to_target = (self.target - current_pos).magnitude();
        distance_to_target > self.cell_size * 3.0 // Only pathfind if target is more than 3 cells away
    }

    /// Set a new path and reset pathfinding state.
    ///
    /// # Arguments
    ///
    /// * `new_path` - The new path to follow.
    pub fn set_path(&mut self, new_path: Vec<cgmath::Vector3<f32>>) {
        self.path = new_path;
        self.current_waypoint = 0;
        self.time_since_last_path = 0.0;
    }

    /// Get the current waypoint, if any.
    ///
    /// # Returns
    ///
    /// The current waypoint as [`cgmath::Vector3<f32>`] if it exists.
    pub fn current_waypoint(&self) -> Option<cgmath::Vector3<f32>> {
        self.path.get(self.current_waypoint).copied()
    }

    /// Move to the next waypoint.
    ///
    /// # Returns
    ///
    /// `true` if the entity has moved to the next waypoint.
    pub fn advance_waypoint(&mut self) -> bool {
        if self.current_waypoint + 1 < self.path.len() {
            self.current_waypoint += 1;
            true
        } else {
            false // Reached end of path
        }
    }

    /// Check if the entity has reached its target.
    ///
    /// # Arguments
    ///
    /// * `current_pos` - The current position of the entity.
    ///
    /// # Returns
    ///
    /// `true` if the entity has reached its target.
    pub fn has_reached_target(&self, current_pos: cgmath::Vector3<f32>) -> bool {
        let distance = (self.target - current_pos).magnitude();
        distance <= self.waypoint_threshold
    }

    /// Check if the entity has reached the current waypoint.
    ///
    /// # Arguments
    ///
    /// * `current_pos` - The current position of the entity.
    ///
    /// # Returns
    ///
    /// `true` if the entity has reached the current waypoint.
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
    /// Create a new pathfinding component with default values.
    ///
    /// # Returns
    ///
    /// A new [`PathfindingComponent`] instance.
    fn default() -> Self {
        Self::new(cgmath::Vector3::new(0.0, 0.0, 0.0), 5.0, 1.0)
    }
}

/// Marker component for entities that should be tracked by pathfinding entities.
#[derive(Component, Debug, Clone, Copy)]
pub struct PathfindingTarget;

/// Marker component for entities that use pathfinding to follow targets.
#[derive(Component, Debug, Clone, Copy)]
pub struct PathfindingFollower;

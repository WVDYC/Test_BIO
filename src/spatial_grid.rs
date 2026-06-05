use bevy::prelude::*;
use crate::components::Position;

/// Structure stored inside the spatial grid cells to allow lock-free position queries.
#[derive(Clone, Copy, Debug)]
pub struct GridEntry {
    pub entity: Entity,
    pub position: Vec2,
}

/// A highly optimized Spatial Hash Grid mapped to a flat 2D physical space.
/// Cleared and rebuilt every frame to provide O(1) cell inserts and O(1) local queries.
#[derive(Resource, Debug)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub width: f32,
    pub height: f32,
    pub min_x: f32,
    pub min_y: f32,
    pub cols: usize,
    pub rows: usize,
    /// Flat 1D representation of a 2D grid: cells[y * cols + x]
    /// Using `Vec<Vec<GridEntry>>` where we reuse inner capacity via `.clear()`
    /// prevents heap allocation thrashing on every frame rebuild.
    pub cells: Vec<Vec<GridEntry>>,
}

impl SpatialGrid {
    /// Creates a new spatial grid covering [min_x, min_x + width] x [min_y, min_y + height].
    pub fn new(min_x: f32, min_y: f32, width: f32, height: f32, cell_size: f32) -> Self {
        let cols = (width / cell_size).ceil() as usize;
        let rows = (height / cell_size).ceil() as usize;
        let total_cells = cols * rows;

        let mut cells = Vec::with_capacity(total_cells);
        for _ in 0..total_cells {
            cells.push(Vec::with_capacity(16));
        }

        Self {
            cell_size,
            width,
            height,
            min_x,
            min_y,
            cols,
            rows,
            cells,
        }
    }

    /// Clears the grid but preserves the internal capacity of all cells.
    /// This is extremely fast because it does not trigger heap deallocations.
    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            cell.clear();
        }
    }

    /// Maps a 2D world position to cell grid coordinates (x, y).
    #[inline(always)]
    pub fn pos_to_cell(&self, pos: Vec2) -> Option<(usize, usize)> {
        let relative_x = pos.x - self.min_x;
        let relative_y = pos.y - self.min_y;

        if relative_x < 0.0 || relative_x >= self.width || relative_y < 0.0 || relative_y >= self.height {
            return None;
        }

        let cell_x = (relative_x / self.cell_size) as usize;
        let cell_y = (relative_y / self.cell_size) as usize;

        if cell_x < self.cols && cell_y < self.rows {
            Some((cell_x, cell_y))
        } else {
            None
        }
    }

    /// Maps a 2D world position to cell grid coordinates, clamping to grid boundaries.
    #[inline(always)]
    pub fn pos_to_cell_clamped(&self, pos: Vec2) -> (usize, usize) {
        let relative_x = (pos.x - self.min_x).clamp(0.0, self.width - 0.001);
        let relative_y = (pos.y - self.min_y).clamp(0.0, self.height - 0.001);

        let cell_x = (relative_x / self.cell_size) as usize;
        let cell_y = (relative_y / self.cell_size) as usize;

        (cell_x.min(self.cols - 1), cell_y.min(self.rows - 1))
    }

    /// Maps cell grid coordinates to a flat 1D index.
    #[inline(always)]
    pub fn cell_to_index(&self, x: usize, y: usize) -> usize {
        y * self.cols + x
    }

    /// Inserts an entity into the grid based on its world position.
    pub fn insert(&mut self, entity: Entity, pos: Vec2) {
        if let Some((cx, cy)) = self.pos_to_cell(pos) {
            let idx = self.cell_to_index(cx, cy);
            self.cells[idx].push(GridEntry { entity, position: pos });
        }
    }

    /// Queries all entities within the cells overlapping a spatial search region.
    /// Uses a callback closure (`impl FnMut`) to achieve ZERO heap allocations during lookups.
    #[inline(always)]
    pub fn query_nearby(&self, pos: Vec2, radius: f32, mut callback: impl FnMut(GridEntry)) {
        let min_pos = pos - Vec2::splat(radius);
        let max_pos = pos + Vec2::splat(radius);

        let (min_cx, min_cy) = self.pos_to_cell_clamped(min_pos);
        let (max_cx, max_cy) = self.pos_to_cell_clamped(max_pos);

        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                let idx = self.cell_to_index(cx, cy);
                for &entry in &self.cells[idx] {
                    callback(entry);
                }
            }
        }
    }
}

/// Bevy system to clear and rebuild the Spatial Hash Grid with bacteria.
pub fn rebuild_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Position), With<crate::components::Dna>>,
) {
    grid.clear();
    for (entity, pos) in query.iter() {
        grid.insert(entity, pos.0);
    }
}

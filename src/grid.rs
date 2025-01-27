use crate::cell::Cell;
use crate::types::Point;
use rand::Rng;
use rayon::prelude::*;

pub struct Grid {
    width: usize,
    height: usize,
    pub initial_cells: Vec<Cell>,
    pub cells: Vec<Cell>,
    pub cells_probabilities: Vec<usize>,
    pub iteration: usize,
    pub launch_count: usize,
    pub max_iterations: usize,
    pub max_launch_count: usize,
    pub dead_probability: f64,
    pub alive_probability: f64,
}

impl Grid {
    // Width and height of the Grid
    pub fn new(
        width: usize,
        height: usize,
        max_iterations: usize,
        max_launch_count: usize,
        dead_probability: f64,
        alive_probability: f64,
    ) -> Self {
        Self {
            width,
            height,
            initial_cells: vec![Cell::new(false); width * height],
            cells: vec![Cell::new(false); width * height],
            cells_probabilities: vec![0; width * height],
            iteration: 0,
            launch_count: 0,
            max_iterations,
            max_launch_count,
            dead_probability,
            alive_probability,
        }
    }
    // These functions are using in the main.rs
    pub fn set_state(&mut self, cells_coords: &[Point]) {
        self.cells = vec![Cell::new(false); self.width * self.height];
        for &pos in cells_coords.iter() {
            let idx = self.coords_to_index(pos);
            self.cells[idx].set_state(true);
        }
    }
    pub fn set_initial_state(&mut self, cells_coords: &[Point]) {
        self.initial_cells = vec![Cell::new(false); self.width * self.height];
        for &pos in cells_coords.iter() {
            let idx = self.coords_to_index(pos);
            self.cells[idx].set_state(true);
        }
        self.initial_cells = self.cells.clone();
    }

    pub fn reset_state(&mut self) {
        for i in 0..self.initial_cells.len() {
            let cell = self.initial_cells[i].clone();
            self.cells[i] = cell;
        }
        self.iteration = 0;
    }

    fn cell_next_state(&self, cell_idx: usize) -> bool {
        let cell = self.cells[cell_idx].clone();
        let cell_pos = self.index_to_coords(cell_idx);
        // Check boundaries and add neighgours
        let mut num_neighbour_alive = 0;
        for &x_off in [-1, 0, 1].iter() {
            for &y_off in [-1, 0, 1].iter() {
                if x_off == 0 && y_off == 0 {
                    continue;
                }
                let neighbour_pos;
                let neighbour_coords = (cell_pos.x as isize + x_off, cell_pos.y as isize + y_off);

                // Make torus
                if neighbour_coords.0 < 0 {
                    // top-left cell
                    if neighbour_coords.1 < 0 {
                        neighbour_pos = Point {
                            x: self.width - 1,
                            y: self.height - 1,
                        }
                    } else if neighbour_coords.1 > self.height as isize - 1 {
                        // bottom-left cell
                        neighbour_pos = Point {
                            x: self.width - 1,
                            y: 0,
                        }
                    } else {
                        // left cell
                        neighbour_pos = Point {
                            x: self.width - 1,
                            y: neighbour_coords.1 as usize,
                        }
                    }
                } else if neighbour_coords.0 > self.width as isize - 1 {
                    if neighbour_coords.1 < 0 {
                        // top-right cell
                        neighbour_pos = Point {
                            x: 0,
                            y: self.height - 1,
                        }
                    } else if neighbour_coords.1 > self.height as isize - 1 {
                        // bottom-right cell
                        neighbour_pos = Point { x: 0, y: 0 }
                    } else {
                        // right cell
                        neighbour_pos = Point {
                            x: 0,
                            y: neighbour_coords.1 as usize,
                        }
                    }
                } else if neighbour_coords.1 < 0 {
                    // top cell
                    neighbour_pos = Point {
                        x: neighbour_coords.0 as usize,
                        y: self.height - 1,
                    }
                } else if neighbour_coords.1 > self.height as isize - 1 {
                    // bottom cell
                    neighbour_pos = Point {
                        x: neighbour_coords.0 as usize,
                        y: 0,
                    }
                } else {
                    // Others cells
                    neighbour_pos = Point {
                        x: neighbour_coords.0 as usize,
                        y: neighbour_coords.1 as usize,
                    };
                }

                let idx = self.coords_to_index(neighbour_pos);
                if self.cells[idx].is_alive() {
                    num_neighbour_alive += 1;
                }
            }
        }
        let mut rnd = rand::thread_rng();

        // Rules https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
        if cell.is_alive() && (num_neighbour_alive == 2 || num_neighbour_alive == 3)
            || (!cell.is_alive() && num_neighbour_alive == 3)
        {
            let probability = rnd.gen_range(0.0..1.0);

            if probability <= self.alive_probability {
                return true; // alive
            }

            return false;
        }

        let probability = rnd.gen_range(0.0..1.0);

        if probability <= self.dead_probability {
            return false;
        }

        return true;
    }

    pub fn set_probability(&mut self, idx: usize) {
        let cell = self.cells[idx].clone();
        if cell.is_alive() {
            self.cells_probabilities[idx] += 1;
        }
    }

    pub fn calculate_entropy(&mut self) {
        let size = self.cells_probabilities.len();
        let mut entropy = 0.0;
        let mut entropy_vec: Vec<f64> = vec![0.0; size];

        for idx in 0..size {
            entropy_vec[idx] = self.cells_probabilities[idx] as f64 / self.max_launch_count as f64;
        }

        for idx in 0..size {
            if entropy_vec[idx] > f64::powf(10.0, -3.0) {
                entropy += entropy_vec[idx] * entropy_vec[idx].log2();
            }
        }

        println!("entropy: {}", f64::abs(entropy));
    }

    pub fn update(&mut self) {
        // Vector of next states. It will match by index
        // Get next states
        let next_states = (0..self.cells.len())
            .into_par_iter()
            .map(|idx| {
                // next state
                self.cell_next_state(idx)
            })
            .collect::<Vec<bool>>();

        self.cells = (0..self.cells.len())
            .into_par_iter()
            .map(|idx| Cell::new(next_states[idx]))
            .collect::<Vec<Cell>>();

        if self.iteration == self.max_iterations {
            for idx in 0..self.cells.len() {
                self.set_probability(idx);
            }
            println!("launches: {}", self.launch_count);

            if self.max_launch_count == self.launch_count {
                self.calculate_entropy();
            }
        }

        if self.iteration == self.max_iterations + 1 {
            self.reset_state();
            self.launch_count += 1;
        } else {
            self.iteration += 1;
        }
    }

    /// Converts a pair of cell coords to index in the cells vector
    pub fn coords_to_index(&self, pos: Point) -> usize {
        pos.y * self.width + pos.x
    }

    /// Converts a index in the cells vecotr into pair of cell coords
    pub fn index_to_coords(&self, index: usize) -> Point {
        Point {
            x: index % self.height,
            y: index / self.width,
        }
    }
}

use std::{collections::HashSet, fs::File, io::Write};

const CHECKERBOARD_WHITE: [usize; 18] = gen_checkerboard(true);
const CHECKERBOARD_BLACK: [usize; 18] = gen_checkerboard(false);
const CHECKERBOARD_LOOKUP: u64 = gen_checkerboard_lookup();

fn main() {
    /*
    {
        // Sanity check, we know this puzzle is valid
        // 0 0 1 0 0 0
        // 0 0 0 0 1 0
        // 0 0 0 0 0 0
        // 0 1 0 0 0 0
        // 0 0 0 0 0 0
        // 0 0 1 0 0 0
        let mut valid_grid = 0u64;
        valid_grid |= 1u64 << 2;
        valid_grid |= 1u64 << 10;
        valid_grid |= 1u64 << 19;
        valid_grid |= 1u64 << 32;
        let valid_grid = valid_grid;
        // Print the valid grid
        println!("Valid grid: {}", grid_string(valid_grid));
        // Test the SloopEdges struct
        let sloop_edges = SloopEdges::new(valid_grid);
        let count = sloop_edges.solution_count(2);
        eprintln!("Valid grid has {} solutions", count);
    }
    */

    // Compute the number of possible 6x6 grids that might be a valid sloop puzzle
    // Each sloop puzzle has some "clued" cells and the rest are "unclued"
    // So technically there are 2^36 possible grids, but we can reduce this number
    // First, we know that the initial clue will always be in the top left 3x3 of the grid
    // Second, each clued cell has a counterpart in the opposite checkerboard color, meaning there are an even number of clued cells
    // We also don't care about puzzles with more than 10 clued cells because they are too easy
    let mut num_possible_grids: usize = 0;
    let mut num_unique_grids: usize = 0;

    let mut grid_stack: Vec<(u64, usize)> = Vec::new();

    // Loop through all possible initial clue locations
    // The grid is 6x6 and the initial clue will always be in the top left 3x3 of the grid
    const INITIAL_CLUE_LOCATIONS: [usize; 9] = [0, 1, 2, 6, 7, 8, 12, 13, 14];
    for &initial_clue_location in INITIAL_CLUE_LOCATIONS.iter() {
        place_clue(&mut grid_stack, 0, initial_clue_location);
    }

    // Open the output file as text
    let mut file = File::create("sloop_puzzles.txt").unwrap();

    let mut seen_puzzles: HashSet<u64> = HashSet::new();

    while let Some((grid, next_clue_index)) = grid_stack.pop() {
        let grid = minlex_grid(grid);
        if !seen_puzzles.insert(grid) {
            continue;
        }

        num_possible_grids += 1;
        if (num_possible_grids % 10000000) == 0 {
            println!(
                "[Progress] Number of unique 6x6 grids: {} / {}",
                num_unique_grids, num_possible_grids
            );
        }

        let sloop_edges = SloopEdges::new(grid);
        let solution_count = sloop_edges.solution_count(2);
        if solution_count == 1 {
            // Write this grid to the output file
            let mut grid_str = grid_string(grid);
            grid_str += "\n";
            file.write_all(grid_str.as_bytes()).unwrap();
            file.flush().unwrap();
            num_unique_grids += 1;
        }

        // If the grid is full or there are at least 10 clues, we don't need to place any more clues
        if next_clue_index == 36 || grid.count_ones() >= 10 {
            continue;
        }

        // Place the next clue
        for next_clue_index in next_clue_index..36 {
            place_clue(&mut grid_stack, grid, next_clue_index);
        }
    }

    println!(
        "Number of unique 6x6 grids: {} / {}",
        num_unique_grids, num_possible_grids
    );
}

fn minlex_grid(grid: u64) -> u64 {
    let mut minlex_grid = grid;
    for flip in 0..2 {
        let mut rotated_grid = grid;
        if flip == 1 {
            rotated_grid = flip_grid(rotated_grid);
            if rotated_grid < minlex_grid {
                minlex_grid = rotated_grid;
            }
        }
        for _ in 0..4 {
            rotated_grid = rotate_grid(rotated_grid);
            if rotated_grid < minlex_grid {
                minlex_grid = rotated_grid;
            }
        }
    }
    minlex_grid
}

fn rotate_grid(grid: u64) -> u64 {
    let mut rotated_grid = 0u64;
    for cell_index in 0..36 {
        if (grid & (1u64 << cell_index)) != 0 {
            let row = cell_index / 6;
            let col = cell_index % 6;
            let new_row = col;
            let new_col = 5 - row;
            let new_cell_index = new_row * 6 + new_col;
            rotated_grid |= 1u64 << new_cell_index;
        }
    }
    rotated_grid
}

fn flip_grid(grid: u64) -> u64 {
    let mut flipped_grid = 0u64;
    for cell_index in 0..36 {
        if (grid & (1u64 << cell_index)) != 0 {
            let row = cell_index / 6;
            let col = cell_index % 6;
            let new_row = 5 - row;
            let new_col = col;
            let new_cell_index = new_row * 6 + new_col;
            flipped_grid |= 1u64 << new_cell_index;
        }
    }
    flipped_grid
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct SloopEdge {
    cell0: usize,
    cell1: usize,
}

#[derive(Clone, Debug)]
struct SloopEdges {
    original_grid: u64,
    loop_edges: Vec<SloopEdge>,
    free_edges: Vec<SloopEdge>,
    cell_loop_edge_counts: [usize; 36],
    cell_free_edge_counts: [usize; 36],
    open_cells_count: usize,
}

impl SloopEdges {
    fn new(grid: u64) -> SloopEdges {
        let loop_edges: Vec<SloopEdge> = Vec::new();
        let mut free_edges: Vec<SloopEdge> = Vec::new();
        let mut cell_loop_edge_counts: [usize; 36] = [0; 36];
        let mut cell_free_edge_counts: [usize; 36] = [0; 36];

        // Loop through all cells
        for cell_index in 0..36 {
            // If the cell is clued, we don't need to do anything
            if (grid & (1u64 << cell_index)) != 0 {
                continue;
            }

            let row = cell_index / 6;
            let col = cell_index % 6;

            // Check the cell to the right
            if col < 5 {
                let right_cell_index = cell_index + 1;
                if (grid & (1u64 << right_cell_index)) == 0 {
                    free_edges.push(SloopEdge {
                        cell0: cell_index,
                        cell1: right_cell_index,
                    });
                    cell_free_edge_counts[cell_index] += 1;
                    cell_free_edge_counts[right_cell_index] += 1;
                }
            }

            // Check the cell below
            if row < 5 {
                let below_cell_index = cell_index + 6;
                if (grid & (1u64 << below_cell_index)) == 0 {
                    free_edges.push(SloopEdge {
                        cell0: cell_index,
                        cell1: below_cell_index,
                    });
                    cell_free_edge_counts[cell_index] += 1;
                    cell_free_edge_counts[below_cell_index] += 1;
                }
            }
        }

        // Setting clued cell edges to be as if they have a loop edge count of 2
        // simplifies the logic for checking if the puzzle is solved
        for (cell_index, count) in cell_loop_edge_counts.iter_mut().enumerate() {
            if (grid & (1u64 << cell_index)) != 0 {
                *count = 2;
            }
        }

        let open_cells_count = cell_loop_edge_counts
            .iter()
            .filter(|&&count| count < 2)
            .count();

        SloopEdges {
            original_grid: grid,
            loop_edges,
            free_edges,
            cell_loop_edge_counts,
            cell_free_edge_counts,
            open_cells_count,
        }
    }

    fn cleanup_invalid_free_edges(&mut self) {
        // Remove any edges that would cause a cell to have more than 2 edges
        let mut new_free_edges = Vec::with_capacity(self.free_edges.len());
        for edge in self.free_edges.iter() {
            if self.cell_loop_edge_counts[edge.cell0] >= 2
                || self.cell_loop_edge_counts[edge.cell1] >= 2
                || self.would_edge_early_loop(*edge)
            {
                self.cell_free_edge_counts[edge.cell0] -= 1;
                self.cell_free_edge_counts[edge.cell1] -= 1;
            } else {
                new_free_edges.push(*edge);
            }
        }
        self.free_edges = new_free_edges;
    }

    fn would_edge_early_loop(&self, edge: SloopEdge) -> bool {
        // If both cells already have 1 edge, adding this edge would cause a loop
        if self.cell_loop_edge_counts[edge.cell0] == 1
            && self.cell_loop_edge_counts[edge.cell1] == 1
        {
            // The loop is ok if this would make every cell have 2 edges
            return self.open_cells_count > 2;
        }
        false
    }

    fn add_free_edge_to_loop(&mut self, free_edge_index: usize) {
        let edge = self.free_edges[free_edge_index];
        assert!(edge.cell0 != edge.cell1);
        self.loop_edges.push(edge);
        self.cell_loop_edge_counts[edge.cell0] += 1;
        self.cell_loop_edge_counts[edge.cell1] += 1;
        assert!(self.cell_loop_edge_counts[edge.cell0] <= 2);
        assert!(self.cell_loop_edge_counts[edge.cell1] <= 2);
        if self.cell_loop_edge_counts[edge.cell0] == 2 {
            self.open_cells_count -= 1;
        }
        if self.cell_loop_edge_counts[edge.cell1] == 2 {
            self.open_cells_count -= 1;
        }

        self.free_edges.remove(free_edge_index);
        self.cell_free_edge_counts[edge.cell0] -= 1;
        self.cell_free_edge_counts[edge.cell1] -= 1;
    }

    fn clear_free_edge(&mut self, free_edge_index: usize) {
        let edge = self.free_edges[free_edge_index];
        self.cell_free_edge_counts[edge.cell0] -= 1;
        self.cell_free_edge_counts[edge.cell1] -= 1;
        self.free_edges.remove(free_edge_index);
    }

    fn is_solved(&self) -> bool {
        self.cell_loop_edge_counts.iter().all(|&count| count == 2)
    }

    fn path_to_string(&self) -> String {
        let mut path_chars = ['B'; 36];
        // Fill in the path with appropriate characters
        // Start with all cells as empty (0)
        // Then the first line seen for a cell sets it to a temp character to remind which direction it goes
        // And then the second line seen sets it to a good looking character, either a bend or a straight line
        // Those characters are: ─ │ ┌ ┐ └ ┘
        for edge in &self.loop_edges {
            let cell0 = edge.cell0;
            let cell1 = edge.cell1;
            let row0 = cell0 / 6;
            let row1 = cell1 / 6;
            if row0 == row1 {
                // Horizontal

                // Cell0 is left of Cell1, so for Cell0, the path goes right
                if path_chars[cell0] == 'B' {
                    path_chars[cell0] = 'R';
                } else if path_chars[cell0] == 'D' {
                    path_chars[cell0] = '┌';
                } else if path_chars[cell0] == 'U' {
                    path_chars[cell0] = '└';
                } else if path_chars[cell0] == 'L' {
                    path_chars[cell0] = '─';
                }

                // Cell1 is right of Cell0, so for Cell1, the path goes left
                if path_chars[cell1] == 'B' {
                    path_chars[cell1] = 'L';
                } else if path_chars[cell1] == 'D' {
                    path_chars[cell1] = '┐';
                } else if path_chars[cell1] == 'U' {
                    path_chars[cell1] = '┘';
                } else if path_chars[cell1] == 'R' {
                    path_chars[cell1] = '─';
                }
            } else {
                // Vertical

                // Cell0 is above Cell1, so for Cell0, the path goes down
                if path_chars[cell0] == 'B' {
                    path_chars[cell0] = 'D';
                } else if path_chars[cell0] == 'R' {
                    path_chars[cell0] = '┌';
                } else if path_chars[cell0] == 'L' {
                    path_chars[cell0] = '┐';
                } else if path_chars[cell0] == 'U' {
                    path_chars[cell0] = '│';
                }

                // Cell1 is below Cell0, so for Cell1, the path goes up
                if path_chars[cell1] == 'B' {
                    path_chars[cell1] = 'U';
                } else if path_chars[cell1] == 'R' {
                    path_chars[cell1] = '└';
                } else if path_chars[cell1] == 'L' {
                    path_chars[cell1] = '┘';
                } else if path_chars[cell1] == 'D' {
                    path_chars[cell1] = '│';
                }
            }
        }

        // Construct a string, putting a newline afer each row
        let mut path_str = String::new();
        for row in 0..6 {
            for col in 0..6 {
                let cell_index = row * 6 + col;
                path_str.push(path_chars[cell_index]);
            }
            path_str.push('\n');
        }
        path_str
    }

    fn is_impossible(&self) -> bool {
        // If there is a cell that cannot get to 2 edges, the puzzle is impossible
        for counts in self
            .cell_free_edge_counts
            .iter()
            .zip(self.cell_loop_edge_counts.iter())
        {
            if counts.0 + counts.1 < 2 {
                return true;
            }
        }

        false
    }

    fn continue_loop(&self) -> Option<SloopEdges> {
        // No free edges left
        if self.free_edges.is_empty() {
            return None;
        }

        // Special case: No loop edges yet, so just add the first free edge
        if self.loop_edges.is_empty() {
            let mut new_sloop_edges_clone = self.clone();
            new_sloop_edges_clone.add_free_edge_to_loop(0);
            return Some(new_sloop_edges_clone);
        }

        // Find which cells are allowed to have a free edge added to the loop
        let mut allowed_cells: Vec<usize> = Vec::new();
        for edge in &self.loop_edges {
            if self.cell_loop_edge_counts[edge.cell0] == 1 {
                allowed_cells.push(edge.cell0);
            }
            if self.cell_loop_edge_counts[edge.cell1] == 1 {
                allowed_cells.push(edge.cell1);
            }
        }

        // If any of the allowed cells has only one free edge, force that move
        for &cell_index in &allowed_cells {
            if self.cell_free_edge_counts[cell_index] == 1 {
                for (free_edge_index, edge) in self.free_edges.iter().enumerate() {
                    if edge.cell0 == cell_index || edge.cell1 == cell_index {
                        let mut new_sloop_edges_clone = self.clone();
                        new_sloop_edges_clone.add_free_edge_to_loop(free_edge_index);
                        return Some(new_sloop_edges_clone);
                    }
                }
            }
        }

        // There are no forced moves, so add any free edge that would increase a cell's edge count to exactly 2, which continues the loop
        for free_edge_index in 0..self.free_edges.len() {
            let edge = self.free_edges[free_edge_index];
            if self.cell_loop_edge_counts[edge.cell0] == 1
                || self.cell_loop_edge_counts[edge.cell1] == 1
            {
                let mut new_sloop_edges_clone = self.clone();
                new_sloop_edges_clone.add_free_edge_to_loop(free_edge_index);
                return Some(new_sloop_edges_clone);
            }
        }

        None
    }

    fn solution_count(&self, count_cap: usize) -> usize {
        // Create a stack of sloop edges
        let mut sloop_edges_stack: Vec<SloopEdges> = Vec::new();

        // Start with a clone
        let mut self_clone = self.clone();
        self_clone.cleanup_invalid_free_edges();
        sloop_edges_stack.push(self_clone);

        let mut count = 0;
        while let Some(sloop_edges) = sloop_edges_stack.pop() {
            // Find a way to continue the loop
            let new_sloop_edges_option = sloop_edges.continue_loop();
            if let Some(new_sloop_edges) = new_sloop_edges_option {
                assert!(new_sloop_edges.loop_edges.len() > sloop_edges.loop_edges.len());

                // Queue up a copy of this board with the new edge removed
                let added_edge = new_sloop_edges.loop_edges.last().unwrap();
                let mut backtrace_sloop_edges = sloop_edges.clone();
                let added_edge_index = backtrace_sloop_edges
                    .free_edges
                    .iter()
                    .position(|edge| edge == added_edge)
                    .unwrap();
                backtrace_sloop_edges.clear_free_edge(added_edge_index);
                backtrace_sloop_edges.cleanup_invalid_free_edges();
                if !backtrace_sloop_edges.free_edges.is_empty()
                    && !backtrace_sloop_edges.is_impossible()
                {
                    sloop_edges_stack.push(backtrace_sloop_edges);
                }

                if new_sloop_edges.is_solved() {
                    count += 1;
                    if count >= count_cap {
                        return count;
                    }
                } else {
                    let mut new_sloop_edges = new_sloop_edges.clone();
                    new_sloop_edges.cleanup_invalid_free_edges();
                    if !new_sloop_edges.free_edges.is_empty() && !new_sloop_edges.is_impossible() {
                        sloop_edges_stack.push(new_sloop_edges);
                    }
                }
            }
        }
        count
    }
}

fn place_clue(grid_stack: &mut Vec<(u64, usize)>, base_grid: u64, clue_index: usize) {
    // Clue is already placed
    if (base_grid & (1u64 << clue_index)) != 0 {
        return;
    }

    let mut next_clue_index = clue_index + 1;
    while (base_grid & (1u64 << next_clue_index)) != 0 {
        next_clue_index += 1;
    }

    // Place the requested clue
    let new_grid = base_grid | (1u64 << clue_index);

    // Place a counterpart clue in the opposite checkerboard color
    let opposite_checkerboard = if CHECKERBOARD_LOOKUP & (1 << clue_index) != 0 {
        &CHECKERBOARD_WHITE
    } else {
        &CHECKERBOARD_BLACK
    };
    for &opposite_clue_index in opposite_checkerboard.iter() {
        // Do not place the clue if it is "before" the requested
        if opposite_clue_index < clue_index {
            continue;
        }

        // Do not place the clue if it is already placed
        if (new_grid & (1u64 << opposite_clue_index)) != 0 {
            continue;
        }

        let final_grid = new_grid | (1u64 << opposite_clue_index);
        grid_stack.push((final_grid, next_clue_index));
    }
}

fn grid_string(grid: u64) -> String {
    let mut grid_str = String::new();
    for cell_index in 0..36 {
        if (grid & (1u64 << cell_index)) != 0 {
            grid_str.push('1');
        } else {
            grid_str.push('0');
        }
    }
    grid_str
}

const fn gen_checkerboard(is_white: bool) -> [usize; 18] {
    let mut checkerboard_white = [0; 18];
    let mut array_index = 0;
    let mut cell_index = 0;
    while cell_index < 36 {
        let row = cell_index / 6;
        let col = cell_index % 6;
        if (row + col) % 2 == (if is_white { 0 } else { 1 }) {
            checkerboard_white[array_index] = cell_index;
            array_index += 1;
        }
        cell_index += 1;
    }
    checkerboard_white
}

const fn gen_checkerboard_lookup() -> u64 {
    let mut checkerboard_lookup = 0;
    let mut cell_index = 0;
    while cell_index < 36 {
        let row = cell_index / 6;
        let col = cell_index % 6;
        if (row + col) % 2 == 0 {
            checkerboard_lookup |= 1 << cell_index;
        }
        cell_index += 1;
    }
    checkerboard_lookup
}

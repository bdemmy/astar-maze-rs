#[derive(Debug, Copy, Clone, Hash)]
struct Node {
    left: u8,
    right: u8,
    top: u8,
    bottom: u8,
}

impl Node {
    fn get_neighbor_positions(&self, x: u16, y: u16) -> Vec<(u16, u16)> {
        let mut neighbors = Vec::new();

        // Left wall
        if self.left > 0 {
            neighbors.push((x - self.left as u16, y))
        }

        if self.right > 0 {
            neighbors.push((x + self.right as u16, y))
        }

        if self.top > 0 {
            neighbors.push((x, y - self.top as u16))
        }

        if self.bottom > 0 {
            neighbors.push((x, y + self.bottom as u16))
        }

        neighbors
    }
}

#[derive(Eq, PartialEq)]
enum MazeBlock {
    Wall,
    Open,
}

use image::io::Reader as ImageReader;
use image::{GrayImage, Rgb, RgbImage};
use std::collections::{HashSet, HashMap};
use priority_queue::PriorityQueue;
use std::time::Instant;
use crate::MazeBlock::{Open, Wall};
use itertools::Itertools;
use std::cmp::{max, min};
use std::io::{Write};
use std::process::exit;

// Get the state of a cell given a reference image and coordinates
fn get_cell_from_image(img: &GrayImage, x: usize, y: usize) -> MazeBlock {
    // Out of bounds
    if x >= img.width() as usize {
        return MazeBlock::Wall;
    }

    // Out of bounds
    if y >= img.height() as usize {
        return MazeBlock::Wall;
    }

    // Dark pixel - wall
    if img.get_pixel(x as u32, y as u32).0[0] < 150 {
        return MazeBlock::Wall;
    }

    // If nothing else, the cell is white and pathable
    return MazeBlock::Open;
}

// Get the neighbor offsets from an image
// Searches in each direction from given x,y until a position for a node is found
// Returns the distance to that node in each direction
fn get_neighbors_from_image(img: &GrayImage, x: usize, y: usize) -> (u8, u8, u8, u8) {
    // Get top neighbors
    let cell = get_cell_from_image(img, x, y);
    if cell == Wall {
        return (0, 0, 0, 0);
    }

    // Check to the left
    let mut left = 0u8;
    for offset in 1..=x {
        if get_cell_from_image(img, x - offset, y) == Wall {
            left = (offset - 1) as u8;
            break;
        }

        if get_cell_from_image(img, x - offset, y + 1) == Open || get_cell_from_image(img, x - offset, y - 1) == Open {
            left = offset as u8;
            break;
        }
    }

    // Check to the right
    let mut right = 0u8;
    for offset in 1..=img.width() as usize - x {
        if get_cell_from_image(img, x + offset, y) == Wall {
            right = (offset - 1) as u8;
            break;
        }

        if get_cell_from_image(img, x + offset, y + 1) == Open || get_cell_from_image(img, x + offset, y - 1) == Open {
            right = offset as u8;
            break;
        }
    }

    // Check to the top
    let mut top = 0u8;
    for offset in 1..=y {
        if get_cell_from_image(img, x, y - offset) == Wall {
            top = (offset - 1) as u8;
            break;
        }

        // Check right and left
        if get_cell_from_image(img, x + 1, y - offset) == Open || get_cell_from_image(img, x - 1, y - offset) == Open {
            top = offset as u8;
            break;
        }
    }

    // Check to the bottom
    let mut bottom = 0u8;
    for offset in 1..=img.height() as usize - y {
        if get_cell_from_image(img, x, y + offset) == Wall {
            bottom = (offset - 1) as u8;
            break;
        }

        // Check right and left
        if get_cell_from_image(img, x + 1, y + offset) == Open || get_cell_from_image(img, x - 1, y + offset) == Open {
            bottom = offset as u8;
            break;
        }
    }

    // Return the offsets that we found pointing to the neighbors
    (left, right, top, bottom)
}

// Generate a hashmap of positions and maze nodes given an image
fn generate_nodes_from_image(img: &GrayImage) -> HashMap<(u16, u16), Node> {
    let width = img.width() as usize;
    let height = img.height() as usize;

    // Map to place nodes into
    let mut map = HashMap::new();

    // Set to mark what we have visited already
    let mut visited = HashSet::new();
    // Stack that we will use to visit neighboring nodes
    let mut stack = Vec::new();

    'outer: for y in 0..height {
        for x in 0..width {
            // Find the first open cell
            if get_cell_from_image(img, x, y) == Open {
                stack.push((x as u16, y as u16));
                break 'outer;
            }
        }
    }

    while let Some((x, y)) = stack.pop() {
        // Did we already visit this node?
        if visited.contains(&(x, y)) {
            continue;
        }

        // Get the neighbors using the image
        let (left, right, top, bottom) = get_neighbors_from_image(img, x as usize, y as usize);

        // Create the node using the neighbors
        let node = Node {
            left,
            right,
            top,
            bottom,
        };

        // Insert current node into the map
        map.insert((x, y), node);

        // Visit neighbor
        if left > 0 {
            stack.push((x - left as u16, y));
        }

        // Visit neighbor
        if right > 0 {
            stack.push((x + right as u16, y));
        }

        // Visit neighbor
        if top > 0 {
            stack.push((x, y - top as u16));
        }

        // Visit neighbor
        if bottom > 0 {
            stack.push((x, y + bottom as u16));
        }

        // Mark current node as visited
        visited.insert((x, y));
    }

    // Our finished map
    map
}

// Calculate the manhattan distance between two positions
fn manhattan(pos1: (u16, u16), pos2: (u16, u16)) -> u32 {
    ((pos1.0 as i32 - pos2.0 as i32).abs() + (pos1.1 as i32 - pos2.1 as i32).abs()) as u32
}

fn get_file_path(prompt: &str) -> String {
    // Get the input file name
    let mut input_path = String::new();
    print!("{}", prompt);
    std::io::stdout().flush().expect("Error flushing stdout.");

    let _ = std::io::stdin().read_line(&mut input_path).unwrap();

    // Strip newline that wont go away with .trim()
    if input_path.ends_with("\n") {
        input_path.truncate(input_path.len() - 1);
    }
    input_path = input_path.trim().to_string();

    input_path
}

// Attempt to find entrance/exit of the maze
// Assumes entrance is highest node in image, and exit is lowest node in image
// If not found, will return the top left and bottom right nodes
fn find_exits(maze: &HashMap<(u16, u16), Node>, width: usize, height: usize)
              -> (Option<(u16, u16)>, Option<(u16, u16)>) {

    let mut start = None;
    let mut end = None;

    // Increment y position only after we checked every x position for the row
    'outer: for y in 0..height as u16 {
        for x in 0..width as u16 {
            if let Some(_) = maze.get(&(x, y)) {
                // If we haven't yet found a start node
                if start.is_none() {
                    start = Some((x,y));
                }
            }

            // Flipped coordinates to check for end pos
            let (bx, by) = (width as u16 - x, height as u16 - y);
            // Check if we found an end node
            if let Some(_) = maze.get(&(bx, by)) {
                // If we haven't yet found an end node
                if end.is_none() {
                    end = Some((bx, by));
                }
            }

            // If we found both values we can stop looping
            if start.is_some() && end.is_some() {
                break 'outer;
            }
        }
    }

    (start, end)
}

// Hacked up helper to draw a line on an image between two nodes
// Assumes nodes are aligned on either the x axis or y axis
fn draw_line_between_nodes(img: &mut RgbImage, node: &(u16, u16), next: &(u16, u16), r: u8, g: u8, b: u8) {
    // Draw horizontal
    if node.1 == next.1 {
        let (start, end) = (min(node.0, next.0), max(node.0, next.0));
        for i in start..=end {
            img.put_pixel(i as u32, node.1 as u32, Rgb([r,g,b]));
        }
    }

    // Draw vertical
    if node.0 == next.0 {
        let (start, end) = (min(node.1, next.1), max(node.1, next.1));
        for i in start..=end {
            img.put_pixel(node.0 as u32, i as u32, Rgb([r,g,b]));
        }
    }
}

fn main() {
    println!("Current working dir: {}", std::env::current_dir().unwrap().display());

    // Get the input path
    let input_path = get_file_path("Enter image input name: ");

    // Get the output path
    let output_path = get_file_path("Enter image output name: ");

    println!("Input name: {}\nOutput name: {}", input_path, output_path);
    println!("Input name length: {}", input_path.len());

    // Load the image from disk
    let source_img = ImageReader::open(&input_path).unwrap().decode().unwrap().into_luma8();

    // Get the cell count
    let num_cells_h = source_img.width() as usize;
    let num_cells_v = source_img.height() as usize;
    println!("pixel count: {}", num_cells_h * num_cells_v);

    // Build the cells from the image
    let maze_nodes = generate_nodes_from_image(&source_img);
    println!("node count: {}", maze_nodes.len());
    println!("ratio: {}", maze_nodes.len() as f32 / (num_cells_h * num_cells_v) as f32);

    // Get the start and end pos, and unwrap (for now, TODO: this is unsafe)
    let (s, e) = find_exits(&maze_nodes, num_cells_h, num_cells_v);
    let (start_pos, end_pos) = (s.unwrap(), e.unwrap());
    println!("start: {:?}, end: {:?}", start_pos, end_pos);

    // Get the start time
    let start = Instant::now();

    // Closed list: Hashset containing positions that we have visited
    let mut closed_list: HashSet<(u16, u16)> = HashSet::with_capacity(num_cells_h * num_cells_v / 2);
    // Open list: Priority Queue for our search, search higher priority nodes first
    let mut open_list: PriorityQueue<(u16, u16), std::cmp::Reverse<usize>> = PriorityQueue::new();
    // Cost map: Hashmap of nodes and their costs. This could be contained within the node struct
    //           but that is not good for memory, as unvisited nodes do not have a cost
    let mut cost_map: HashMap<(u16, u16), usize> = HashMap::new();
    // Parent map: Hashmap of nodes and their parents.  Same situation as cost map, could be inside
    //           node but that is expensive memory-wise.
    let mut parent_map: HashMap<(u16, u16), (u16, u16)> = HashMap::new();

    // Start with our start position
    open_list.push(
        start_pos,
        std::cmp::Reverse(0),
    );

    // Start A* search
    while open_list.len() > 0 {
        // Get the current position from the top of the open list
        // TODO: Unsafe
        let cur_pos = open_list.pop().unwrap().0;

        // Get the current node and cost
        // TODO: Unsafe
        let cur = *maze_nodes.get(&cur_pos).unwrap();
        let cur_cost = *cost_map.get(&cur_pos).unwrap_or(&0);

        // Insert current node into visited list
        closed_list.insert(cur_pos);

        // Check if we found the path
        if cur_pos == end_pos {
            println!("Found path!");
            break;
        }

        // Update neighbors
        for neighbor_pos in cur.get_neighbor_positions(cur_pos.0, cur_pos.1) {
            if !closed_list.contains(&neighbor_pos) {
                let manhattan = manhattan(neighbor_pos, end_pos);
                let cost = std::cmp::min((cur_cost + 1) as usize, manhattan as usize);

                open_list.push(neighbor_pos, std::cmp::Reverse(cost));

                cost_map.insert(neighbor_pos, cur_cost + 1);
                parent_map.insert(neighbor_pos, cur_pos);
            }
        }
    }

    // Current stop and path vec
    let mut cur = parent_map.get(&end_pos);
    let mut path: Vec<(u16, u16)> = Vec::new();
    path.push(end_pos);

    // While we can find a parent
    while let Some(cur_node) = cur {
        path.push(*cur_node);

        cur = parent_map.get(cur_node);
    }

    // Reverse the path to be from start to finish
    path.reverse();

    let duration = start.elapsed();
    println!("Time elapsed in A* is: {:?}", duration);
    println!("Path Length: {}", &path.len());
    println!("Saving image...");

    // Create image for writing
    let mut output_image = ImageReader::open(&input_path).unwrap().decode().unwrap().into_rgb8();

    // Draw all of the generated maze nodes
    for (pos, _) in maze_nodes {
        output_image.put_pixel(pos.0 as u32, pos.1 as u32, Rgb([200,200,255]));
    }

    // Draw all of the visited points
    for node in &closed_list {
        if let Some(parent) = parent_map.get(&node) {
            draw_line_between_nodes(&mut output_image, &node, parent, 255, 0, 0);
        }
    }

    // Draw the path segments
    for (node, next) in path.iter().tuple_windows() {
        draw_line_between_nodes(&mut output_image, node, next, 0, 255, 0);
    }

    output_image.save(output_path).unwrap();
    println!("Done!");
}

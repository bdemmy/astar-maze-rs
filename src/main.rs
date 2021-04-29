#[derive(Debug, Copy, Clone, Hash)]
struct Node {
    left: u8,
    right: u8,
    top: u8,
    bottom: u8,
}

impl Node {
    fn get_neighbor_positions(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();

        // Left wall
        if self.left > 0 {
            neighbors.push((x - self.left as usize, y))
        }

        if self.right > 0 {
            neighbors.push((x + self.right as usize, y))
        }

        if self.top > 0 {
            neighbors.push((x, y - self.top as usize))
        }

        if self.bottom > 0 {
            neighbors.push((x, y + self.bottom as usize))
        }

        neighbors
    }
}

use image::io::Reader as ImageReader;
use image::{GrayImage, Rgb};
use std::collections::{HashSet, HashMap};
use priority_queue::PriorityQueue;
use std::time::Instant;
use crate::MazeBlock::{Open, Wall, Empty};

#[derive(Eq, PartialEq)]
enum MazeBlock {
    Empty,
    Wall,
    Open,
}

fn get_cell_from_image(img: &GrayImage, x: usize, y: usize) -> MazeBlock {
    if x >= img.width() as usize {
        return MazeBlock::Empty;
    }

    if y >= img.height() as usize {
        return MazeBlock::Empty;
    }

    if img.get_pixel(x as u32, y as u32).0[0] < 150 {
        return MazeBlock::Wall;
    }

    return MazeBlock::Open;
}

fn get_neighbors_from_image(img: &GrayImage, x: usize, y: usize) -> (u8, u8, u8, u8) {
    // Get top neighbors
    let cell = get_cell_from_image(img, x, y);
    if cell == Wall || cell == Empty {
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

    // Check to the left
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

    (left, right, top, bottom)
}

fn generate_nodes_from_image(img: &GrayImage) -> HashMap<(usize, usize), Node> {
    let num_cells_h = img.width() as usize;
    let num_cells_v = img.height() as usize;

    let mut map = HashMap::new();

    let mut visited = HashSet::new();
    let mut stack = Vec::new();

    'outer: for y in 0..num_cells_v {
        for x in 0..num_cells_h {
            // Find the first open cell
            if get_cell_from_image(img, x, y) == Open {
                stack.push((x, y));
                break 'outer;
            }
        }
    }

    while let Some((x, y)) = stack.pop() {
        if visited.contains(&(x, y)) {
            continue;
        }

        let (left, right, top, bottom) = get_neighbors_from_image(img, x, y);

        let node = Node {
            left,
            right,
            top,
            bottom,
        };
        map.insert((x, y), node);

        if left > 0 {
            stack.push((x - left as usize, y));
        }

        if right > 0 {
            stack.push((x + right as usize, y));
        }

        if top > 0 {
            stack.push((x, y - top as usize));
        }

        if bottom > 0 {
            stack.push((x, y + bottom as usize));
        }

        visited.insert((x, y));
    }

    map
}

fn manhattan(pos1: (usize, usize), pos2: (usize, usize)) -> u32 {
    ((pos1.0 as i32 - pos2.0 as i32).abs() + (pos1.1 as i32 - pos2.1 as i32).abs()) as u32
}

fn get_input_path() -> String {
    // Get the input file name
    let mut input_path = String::new();
    println!("Enter input file name: ");
    let _ = std::io::stdin().read_line(&mut input_path).unwrap();
    // Strip newline that wont go away with .trim()
    if input_path.ends_with("\n") {
        input_path.truncate(input_path.len() - 1);
    }

    input_path
}

fn main() {
    // Get the input path
    let input_path = get_input_path();

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

    // Start A*
    let start = Instant::now();

    let start_pos = (1usize, 1usize);
    let end_pos = (num_cells_h - 2, num_cells_v - 2);

    let mut closed_list: HashSet<(usize, usize)> = HashSet::with_capacity(num_cells_h * num_cells_v / 2);
    let mut open_list: PriorityQueue<(usize, usize), std::cmp::Reverse<usize>> = PriorityQueue::new();
    let mut cost_map: HashMap<(usize, usize), usize> = HashMap::new();
    let mut parent_map: HashMap<(usize, usize), (usize, usize)> = HashMap::new();

    open_list.push(
        start_pos,
        std::cmp::Reverse(0),
    );

    while open_list.len() > 0 {
        let cur_pos = open_list.pop().unwrap().0;

        let cur = *maze_nodes.get(&cur_pos).unwrap();
        let cur_cost = *cost_map.get(&cur_pos).unwrap_or(&0);

        if cur_pos == end_pos {
            println!("Found path!");
            break;
        }

        closed_list.insert(cur_pos);

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
    let mut path: Vec<(usize, usize)> = Vec::new();

    // While we can find a parent
    while let Some(cur_node) = cur {
        path.push(*cur_node);

        cur = parent_map.get(cur_node);
    }

    path.reverse();

    // Load the image from disk
    /*for (idx, node) in path.iter().enumerate() {
        if let Some(next) = path.get(idx + 1) {
            // Draw horizontal
            if node.1 == next.1 {
                for i in node.0..=next.0 {
                    output_image.put_pixel(i as u32, node.1 as u32, Rgb([0, 255, 0]));
                }
            }

            // Draw vertical
            if node.0 == next.0 {
                for i in node.1..=next.1 {
                    output_image.put_pixel(node.0 as u32, i as u32, Rgb([0, 255, 0]));
                }
            }
        }
    }*/

    let mut output_image = ImageReader::open(&input_path).unwrap().decode().unwrap().into_rgb8();

    for node in &closed_list {
        output_image.put_pixel(node.0 as u32, node.1 as u32, Rgb([255, 0, 0]))
    }

    for node in &path {
        output_image.put_pixel(node.0 as u32, node.1 as u32, Rgb([0, 255, 0]));
    }

    output_image.save("out_reverse_manhattan.png").unwrap();

    let duration = start.elapsed();
    println!("Time elapsed in A* is: {:?}", duration);
    println!("Path Length: {}", &path.len());
}

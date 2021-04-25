
#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
struct Cell {
    top_wall: bool,
    bottom_wall: bool,
    left_wall: bool,
    right_wall: bool,
    position: (usize, usize)
}
impl Cell {
    fn get_neighbor_positions(&self) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();

        // Left wall
        if !self.left_wall {
            neighbors.push((self.position.0 - 1, self.position.1))
        }

        if !self.right_wall {
            neighbors.push((self.position.0 + 1, self.position.1))
        }

        if !self.top_wall {
            neighbors.push((self.position.0, self.position.1 - 1))
        }

        if !self.bottom_wall {
            neighbors.push((self.position.0, self.position.1 + 1))
        }

        neighbors
    }
}

#[derive(Debug, Copy, Clone, Hash)]
struct Node {
    parent: Option<(usize, usize)>,
    cost: usize,
    cell: Cell
}

use image::io::Reader as ImageReader;
use array2d::Array2D;
use image::{GrayImage, Rgb};
use std::cmp::Ordering;
use std::collections::{hash_set, HashSet, HashMap};
use priority_queue::PriorityQueue;
use std::time::Instant;

fn get_cell(img: &GrayImage, x: usize, y: usize) -> Option<Cell> {
    let imgx = 2 + x as u32;
    let imgy = 2 + y as u32;

    if img.get_pixel(imgx, imgy).0[0] < 150 {
        return None;
    }

    let top = img.get_pixel(imgx, imgy - 1).0[0] < 150;
    let bottom = img.get_pixel(imgx, imgy + 1).0[0] < 150;
    let left = img.get_pixel(imgx - 1, imgy).0[0] < 150;
    let right = img.get_pixel(imgx + 1, imgy).0[0] < 150;

    Some(Cell {
        top_wall: top,
        bottom_wall: bottom,
        left_wall: left,
        right_wall: right,
        position: (x as usize, y as usize)
    })
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
    let num_cells_h = (source_img.width() - 4) as usize;
    let num_cells_v = (source_img.height() - 4) as usize;
    println!("cell count x: {}, y: {}", num_cells_h, num_cells_v);

    // Build the cells from the image
    let mut maze_nodes = HashMap::new();
    for y in 0..num_cells_v {
        for x in 0..num_cells_h {
            if let Some(cell) = get_cell(&source_img, x, y) {
                maze_nodes.insert((x, y), Node {
                    parent: None,
                    cost: 0,
                    cell
                });
            }
        }
    }

    // Start A*
    let start = Instant::now();

    let start_pos = (0usize, 0usize);
    let end_pos = (num_cells_h - 1, num_cells_v - 1);

    let mut closed_list: HashSet<(usize, usize)> = HashSet::with_capacity(num_cells_h * num_cells_v / 2);
    let mut open_list: PriorityQueue<(usize, usize), std::cmp::Reverse<usize>> = PriorityQueue::new();

    open_list.push(
        start_pos,
        std::cmp::Reverse(0)
    );

    let mut count = 0;
    let maxcount = 1000000;

    let mut output_image = ImageReader::open(&input_path).unwrap().decode().unwrap().into_rgb8();
    while open_list.len() > 0 {
        let cur_pos = open_list.pop().unwrap().0;
        let cur = *maze_nodes.get(&cur_pos).unwrap();

        if cur_pos == end_pos {
            println!("Found path!");
            break;
        }

        closed_list.insert(cur_pos);

        for neighbor_pos in cur.cell.get_neighbor_positions() {
            if !closed_list.contains(&neighbor_pos) {
                let manhattan = manhattan(neighbor_pos, end_pos);
                let cost = std::cmp::min((cur.cost + 1) as usize, manhattan as usize);

                open_list.push(neighbor_pos, std::cmp::Reverse(cost));

                let node = maze_nodes.get_mut(&neighbor_pos).unwrap();
                node.cost = cur.cost + 1;
                node.parent = Some(cur_pos);
            }
        }

        count = (count + 1) % maxcount;

        if count == maxcount - 1 {
            for node in &closed_list {
                let imgx = 2 + node.0;
                let imgy = 2 + node.1;

                output_image.put_pixel(imgx as u32, imgy as u32, Rgb([255, 0, 0]))
            }

            output_image.save("out_reverse_manhattan.png").unwrap();
        }
    }

    let mut cur =  maze_nodes.get(&end_pos);
    let mut path: Vec<(usize, usize)> = Vec::new();
    while let Some(cur_node) = cur {
        if let Some(parent_pos) = cur_node.parent {
            path.push(parent_pos);
            cur = maze_nodes.get(&parent_pos)
        } else { break };
    }

    path.reverse();

    // Load the image from disk
    for node in path {
        let imgx = 2 + node.0;
        let imgy = 2 + node.1;

        output_image.put_pixel(imgx as u32, imgy as u32, Rgb([0, 255, 0]))
    }
    output_image.save("out_reverse_manhattan.png").unwrap();

    let duration = start.elapsed();
    println!("Time elapsed in A* is: {:?}", duration);
}

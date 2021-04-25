
#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
struct Cell {
    top_wall: bool,
    bottom_wall: bool,
    left_wall: bool,
    right_wall: bool
}

#[derive(Clone, Debug, Hash, Copy)]
struct Node {
    cell: Cell,
    parent: Option<(usize, usize)>,
    distance: u32,
    position: (usize, usize)
}
impl Node {
    fn get_neighbor_positions(&self) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();

        // Left wall
        if !self.cell.left_wall {
            neighbors.push((self.position.0 - 1, self.position.1))
        }

        if !self.cell.right_wall {
            neighbors.push((self.position.0 + 1, self.position.1))
        }

        if !self.cell.top_wall {
            neighbors.push((self.position.0, self.position.1 - 1))
        }

        if !self.cell.bottom_wall {
            neighbors.push((self.position.0, self.position.1 + 1))
        }

        neighbors
    }
}
// Implement ordering/equality
impl Eq for Node {}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.cmp(&other.distance)
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use image::io::Reader as ImageReader;
use array2d::Array2D;
use image::{GrayImage, Rgb};
use std::cmp::Ordering;
use std::collections::{hash_set, HashSet};
use priority_queue::PriorityQueue;
use std::time::Instant;

fn get_cell(img: &GrayImage, x: u32, y: u32) -> Option<Cell> {
    let imgx = 2 + x;
    let imgy = 2 + y;

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
        right_wall: right
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
    let mut maze_nodes = Array2D::filled_with(Option::<Node>::None, num_cells_h, num_cells_v);
    for y in 0..num_cells_v as u32 {
        for x in 0..num_cells_h as u32 {
            if let Some(cell) = get_cell(&source_img, x, y) {
                maze_nodes[(x as usize, y as usize)] = Some(Node{
                    distance: 0,
                    parent: None,
                    cell,
                    position: (x as usize, y as usize)
                })
            }
        }
    }

    // Start A*
    let start = Instant::now();

    let start_pos = (0usize, 0usize);
    let end_pos = (num_cells_h - 1, num_cells_v - 1);

    let mut visited: HashSet<(usize, usize)> = HashSet::with_capacity(num_cells_h * num_cells_v / 2);
    let mut pq: PriorityQueue<(usize, usize), std::cmp::Reverse<u32>> = PriorityQueue::new();

    pq.push(
        start_pos,
        std::cmp::Reverse(0)
    );

    let mut count = 0;
    let maxcount = 1000000;

    let mut output_image = ImageReader::open(&input_path).unwrap().decode().unwrap().into_rgb8();
    while pq.len() > 0 {
        let cur_pos = pq.pop().unwrap().0;
        let cur:&Node = &maze_nodes[cur_pos].unwrap();

        if cur_pos == end_pos {
            println!("Found path!");
            break;
        }

        visited.insert(cur_pos);

        for neighbor_pos in cur.get_neighbor_positions() {
            let manhattan = manhattan(neighbor_pos, end_pos);

            if !visited.contains(&neighbor_pos) {
                pq.push(neighbor_pos, std::cmp::Reverse(std::cmp::min(cur.distance + 1, manhattan)));

                let mut updated_node = maze_nodes[neighbor_pos].unwrap();
                updated_node.distance = cur.distance + 1;
                updated_node.parent = Some(cur_pos);
                maze_nodes[neighbor_pos] = Some(updated_node);
            }
        }

        count = (count + 1) % maxcount;

        if count == maxcount - 1 {
            for node in &visited {
                let imgx = 2 + node.0;
                let imgy = 2 + node.1;

                output_image.put_pixel(imgx as u32, imgy as u32, Rgb([255, 0, 0]))
            }

            output_image.save("out_reverse_manhattan.png").unwrap();
        }
    }

    let mut cur =  maze_nodes[end_pos];
    let mut path: Vec<(usize, usize)> = Vec::new();
    while let Some(cur_node) = cur {
        if cur_node.parent.is_none() {
            break;
        }

        path.push(cur_node.parent.unwrap());
        cur = maze_nodes[cur_node.parent.unwrap()];
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

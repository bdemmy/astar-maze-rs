
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
    distance: u32
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
use priority_queue::PriorityQueue;

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

fn main() {
    // Load the image from disk
    let img = ImageReader::open("500.png").unwrap().decode().unwrap().into_luma8();

    // Get the image size
    println!("w: {}, h: {}", img.width(), img.height());

    // Get the cell count
    let num_cells_h = (img.width() - 4) as usize;
    let num_cells_v = (img.height() - 4) as usize;
    println!("cell count x: {}, y: {}", num_cells_h, num_cells_v);

    // Build the cells from the image
    let mut maze_nodes = Array2D::filled_with(Option::<Node>::None, num_cells_h, num_cells_v);
    for y in 0..num_cells_v as u32 {
        for x in 0..num_cells_h as u32 {
            if let Some(cell) = get_cell(&img, x, y) {
                maze_nodes[(x as usize, y as usize)] = Some(Node{
                    distance: 0,
                    parent: None,
                    cell
                })
            }
        }
    }

    // Debug print the cells
    /*for row in maze_nodes.as_columns() {
        println!("{:?}", row)
    }*/

    // Start A*
    let start_pos = (
        0 as usize,
        0 as usize
    );
    let end_pos = (
        (num_cells_h - 1) as usize,
        (num_cells_v - 1) as usize
    );

    println!("Start position: {:?}", start_pos);

    let mut visited: Vec<(usize, usize)> = Vec::new();
    let mut pq: PriorityQueue<(usize, usize), std::cmp::Reverse<u32>> = PriorityQueue::new();

    pq.push(
        start_pos,
        std::cmp::Reverse(0)
    );

    while pq.len() > 0 {
        let cur_pos = pq.pop().unwrap().0;
        let cur = &maze_nodes[cur_pos].unwrap();

        if cur_pos == end_pos {
            println!("Found path!");
            break;
        }

        visited.push(cur_pos);

        if !cur.cell.left_wall && cur_pos.0 > 0 {
            let neighbor_pos = (cur_pos.0 - 1, cur_pos.1);
            let manhattan = manhattan(neighbor_pos, end_pos);

            if !visited.contains(&neighbor_pos) {
                pq.push(neighbor_pos, std::cmp::Reverse(manhattan));

                let mut test_node = maze_nodes[neighbor_pos].unwrap();
                test_node.distance = cur.distance + 1;
                test_node.parent = Some(cur_pos);

                maze_nodes[neighbor_pos] = Some(test_node);
            }
        }

        if !cur.cell.right_wall && cur_pos.0 < (num_cells_h - 1) {
            let neighbor_pos = (cur_pos.0 + 1, cur_pos.1);
            let manhattan = manhattan(neighbor_pos, end_pos);

            if !visited.contains(&neighbor_pos) {
                pq.push(neighbor_pos, std::cmp::Reverse(manhattan));

                let mut test_node = maze_nodes[neighbor_pos].unwrap();
                test_node.distance = cur.distance + 1;
                test_node.parent = Some(cur_pos);

                maze_nodes[neighbor_pos] = Some(test_node);
            }
        }

        if !cur.cell.bottom_wall && cur_pos.1 < (num_cells_v - 1) {
            let neighbor_pos = (cur_pos.0, cur_pos.1 + 1);
            let manhattan = manhattan(neighbor_pos, end_pos);

            if !visited.contains(&neighbor_pos) {
                pq.push(neighbor_pos, std::cmp::Reverse(manhattan));

                let mut test_node = maze_nodes[neighbor_pos].unwrap();
                test_node.distance = cur.distance + 1;
                test_node.parent = Some(cur_pos);

                maze_nodes[neighbor_pos] = Some(test_node);
            }
        }

        if !cur.cell.top_wall && cur_pos.1 > 0 {
            let neighbor_pos = (cur_pos.0, cur_pos.1 - 1);
            let manhattan = manhattan(neighbor_pos, end_pos);

            if !visited.contains(&neighbor_pos) {
                pq.push(neighbor_pos, std::cmp::Reverse(manhattan));

                let mut test_node = maze_nodes[neighbor_pos].unwrap();
                test_node.distance = cur.distance + 1;
                test_node.parent = Some(cur_pos);

                maze_nodes[neighbor_pos] = Some(test_node);
            }
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
    let mut testImg = ImageReader::open("500.png").unwrap().decode().unwrap().into_rgb();
    for node in visited {
        let imgx = 2 + node.0;
        let imgy = 2 + node.1;

        testImg.put_pixel(imgx as u32, imgy as u32, Rgb([255, 0, 0]))
    }
    for node in path {
        let imgx = 2 + node.0;
        let imgy = 2 + node.1;

        testImg.put_pixel(imgx as u32, imgy as u32, Rgb([0, 255, 0]))
    }
    testImg.save("out.png").unwrap();
}

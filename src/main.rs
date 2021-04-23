
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
    pos: (usize, usize),
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

fn get_cell(img: &GrayImage, x: u32, y: u32) -> Cell {
    let imgx = 1 + x * 16;
    let imgy = 1 + y * 16;

    let top = img.get_pixel(imgx + 1, imgy).0[0] < 150;
    let bottom = img.get_pixel(imgx + 1, imgy + 15).0[0] < 150;
    let left = img.get_pixel(imgx, imgy + 1).0[0] < 150;
    let right = img.get_pixel(imgx + 15, imgy + 1).0[0] < 150;

    Cell {
        top_wall: top,
        bottom_wall: bottom,
        left_wall: left,
        right_wall: right
    }
}

fn manhattan(pos1: (usize, usize), pos2: (usize, usize)) -> u32 {
    ((pos1.0 as i32 - pos2.0 as i32).abs() + (pos1.1 as i32 - pos2.1 as i32).abs()) as u32
}

fn main() {
    // Load the image from disk
    let img = ImageReader::open("input2.png").unwrap().decode().unwrap().into_luma8();

    // Get the image size
    println!("w: {}, h: {}", img.width(), img.height());

    // Get the cell count
    let num_cells_h = ((img.width() - 2) / 16) as usize;
    let num_cells_v = ((img.height() - 2) / 16) as usize;
    println!("cell count x: {}, y: {}", num_cells_h, num_cells_v);

    // Build the cells from the image
    let mut maze_nodes = Array2D::filled_with(Node {
        cell: Cell {
            top_wall: false,
            bottom_wall: false,
            left_wall: false,
            right_wall: false
        },
        pos: (0, 0),
        parent: None,
        distance: 0
    }, num_cells_h, num_cells_v);
    for y in 0..num_cells_v as u32 {
        for x in 0..num_cells_h as u32 {
            maze_nodes[(x as usize, y as usize)].cell = get_cell(&img, x, y);
            maze_nodes[(x as usize, y as usize)].pos = (x as usize, y as usize)
        }
    }

    // Debug print the cells
    /*for row in maze_nodes.as_columns() {
        println!("{:?}", row)
    }*/

    // Start A*
    let start_pos = (
        ((img.width() - 2) / 2 / 16) as usize,
        0 as usize
    );
    let end_pos = (
        ((img.width() - 2) / 2 / 16) as usize,
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
        let cur = maze_nodes[cur_pos];

        if cur_pos == end_pos {
            println!("Found path!");
            break;
        }

        visited.push(cur_pos);

        if !cur.cell.left_wall && cur_pos.0 > 0 {
            let left_pos = (cur_pos.0 - 1, cur_pos.1);
            let manhattan = manhattan(left_pos, end_pos);

            if !visited.contains(&left_pos) {
                pq.push(left_pos, std::cmp::Reverse(manhattan));

                maze_nodes[left_pos].distance = cur.distance + 1;
                maze_nodes[left_pos].parent = Some(cur_pos);
            }
        }

        if !cur.cell.right_wall && cur_pos.0 < (num_cells_h - 1) {
            let right_pos = (cur_pos.0 + 1, cur_pos.1);
            let manhattan = manhattan(right_pos, end_pos);

            if !visited.contains(&right_pos) {
                pq.push(right_pos, std::cmp::Reverse(manhattan));

                maze_nodes[right_pos].distance = cur.distance + 1;
                maze_nodes[right_pos].parent = Some(cur_pos);
            }
        }

        if !cur.cell.bottom_wall && cur_pos.1 < (num_cells_v - 1) {
            let bottom_pos = (cur_pos.0, cur_pos.1 + 1);
            let manhattan = manhattan(bottom_pos, end_pos);

            if !visited.contains(&bottom_pos) {
                pq.push(bottom_pos, std::cmp::Reverse(manhattan));

                maze_nodes[bottom_pos].distance = cur.distance + 1;
                maze_nodes[bottom_pos].parent = Some(cur_pos);
            }
        }

        if !cur.cell.top_wall && cur_pos.1 > 0 {
            let top_pos = (cur_pos.0, cur_pos.1 - 1);
            let manhattan = manhattan(top_pos, end_pos);

            if !visited.contains(&top_pos) {
                pq.push(top_pos, std::cmp::Reverse(manhattan));

                maze_nodes[top_pos].distance = cur.distance + 1;
                maze_nodes[top_pos].parent = Some(cur_pos);
            }
        }
    }

    let mut cur = Some(end_pos);
    let mut path = Vec::new();
    while cur.is_some() {
        path.push(cur.unwrap());
        let curnode = maze_nodes[cur.unwrap()];
        cur = curnode.parent;
    }

    path.push(start_pos);
    path.reverse();
    println!("{:?}", path);

    // Load the image from disk
    let mut testImg = ImageReader::open("input2.png").unwrap().decode().unwrap().into_rgb();
    for node in visited {
        let imgx = 4 + (node.0 * 16);
        let imgy = 4 + (node.1 * 16);

        for x in imgx..imgx+10 {
            for y in imgy..imgy+10 {
                testImg.put_pixel(x as u32, y as u32, Rgb([255, 0, 0]))
            }
        }
    }
    for node in path {
        let imgx = 4 + (node.0 * 16);
        let imgy = 4 + (node.1 * 16);

        for x in imgx..imgx+10 {
            for y in imgy..imgy+10 {
                testImg.put_pixel(x as u32, y as u32, Rgb([0, 255, 0]))
            }
        }
    }
    testImg.save("out2.png").unwrap();
}

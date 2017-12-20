const WIDTH: usize = 8;
const HEIGHT: usize = 8;

struct Position {
    x: usize,
    y: usize,
}

struct Entity {
    pos: Position,
}

fn print_map(entities: &Vec<Entity>) {
    let mut tiles = [["   "; WIDTH]; HEIGHT];
    for e in entities {
        tiles[e.pos.x][e.pos.y] = "XXX";
    }

    println!("{}", ".".repeat(WIDTH*3+2));
    for y in 0..HEIGHT {
        print!(".");
        for x in 0..WIDTH {
            print!("{}", tiles[x][y]);
        }
        println!(".");
    }
    println!("{}", ".".repeat(WIDTH*3+2));
}

fn main() {
    println!("Hello, world!");
    let mut entities: Vec<Entity> = Vec::new();
    entities.push(
        Entity {
            pos: Position { x: 5, y: 1}
        });
    print_map(&entities)
}

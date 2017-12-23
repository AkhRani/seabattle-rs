const WIDTH: usize = 8;
const HEIGHT: usize = 8;

struct Position {
    x: usize,
    y: usize,
}

enum EType {
    Player,
    Island,
    Ship,
    Mine,
    HQ,
    Monster
}

struct Entity {
    pos: Position,
    etype: EType,
}

fn print_map(entities: &Vec<Entity>) {
    let mut tiles = [["   "; WIDTH]; HEIGHT];
    for e in entities {
        tiles[e.pos.x][e.pos.y] = match e.etype {
            EType::Player => "(X)",
            EType::Island => "***",
            EType::Ship => "\\S/",
            EType::Mine => " $ ",
            EType::HQ => "-H-",
            EType::Monster => "SSS",
        }
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

fn setup() -> Vec<Entity> {
    let mut entities = Vec::new();
    entities.push(
        Entity {
            pos: Position { x: 5, y: 1},
            etype: EType::Ship,
        });
    entities
}

fn main() {
    println!("Hello, world!");
    // let mut entities: Vec<Entity> = Vec::new();
    let entities = setup();
    print_map(&entities);
}

extern crate rand;

use rand::Rng;

const WIDTH: usize = 20;
const HEIGHT: usize = 20;

#[derive(PartialEq, Eq)]
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
    alive: bool,
}

impl Entity {
    // syntactic sugar
    fn new(x: usize, y: usize, etype: EType) -> Entity {
        Entity {
            pos: Position {x, y},
            etype,
            alive: true,
        }
    }
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

fn check_collision(entities: &Vec<Entity>, x:usize, y:usize) -> bool {
    for e in entities {
        // Why doesn't this work? if Position{x, y} == e.pos {
        let pos = Position{x, y};
        if pos == e.pos {
            return true;
        }
    }
    return false;
}

fn place_random(entities: &mut Vec<Entity>, rng: &mut rand::ThreadRng, etype: EType) {
    loop {
        let x = rng.gen_range(0, WIDTH);
        let y = rng.gen_range(0, HEIGHT);
        if !check_collision(entities, x, y) {
            entities.push(Entity::new(x, y, etype));
            return;
        }
    }
}

fn setup() -> Vec<Entity> {
    let mut entities = Vec::new();

    // Island Bitmap
    let island = [
        0, 1, 1, 1, 0, 0,
        0, 1, 1, 1, 1, 0,
        1, 1, 1, 0, 1, 1,
        1, 1, 0, 0, 0, 1,
        1, 1, 0, 0, 1, 1,
        0, 1, 1, 0, 1, 0,
        0, 0, 1, 0, 0, 0,
    ];
    let mut i = 0;
    for y in 7..14 {
        for x in 7..13 {
            if island[i] == 1 {
                entities.push(Entity::new(x, y, EType::Island));
            }
            i += 1;
        }
    }

    // Player
    entities.push(Entity::new(10, 10, EType::Player));

    // Enemy Ships
    let mut rng = rand::thread_rng();
    for _i in 0..rng.gen_range(15, 31) {
        place_random(&mut entities, &mut rng, EType::Ship);
    }

    // HQ
    place_random(&mut entities, &mut rng, EType::HQ);

    // Mines
    for _i in 0..rng.gen_range(8, 15) {
        place_random(&mut entities, &mut rng, EType::Mine);
    }

    // Sea Monsters
    for _i in 0..4 {
        place_random(&mut entities, &mut rng, EType::Monster);
    }


    entities
}

fn main() {
    let entities = setup();
    print_map(&entities);
}

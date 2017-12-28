extern crate rand;

use rand::Rng;

const WIDTH: usize = 20;
const HEIGHT: usize = 20;

#[derive(PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(PartialEq)]
enum EType {
    Player,
    Island,
    Ship,
    Mine,
    HQ,
    Monster
}

enum Component {
    Velocity(i8, i8),
}

struct Entity {
    pos: Position,
    etype: EType,
    alive: bool,
    components: Vec<Component>
}

impl Entity {
    // syntactic sugar
    fn new(x: usize, y: usize, etype: EType) -> Entity {
        Entity {
            pos: Position {x, y},
            etype,
            alive: true,
            components: Vec::new(),
        }
    }
}

fn print_map(entities: &Vec<Entity>) {
    let mut tiles = [["   "; WIDTH]; HEIGHT];
    for e in entities {
        // TODO:  Sonar noise
        tiles[e.pos.x][e.pos.y] = match e.etype {
            EType::Player => "(X)",
            EType::Island => "***",
            EType::Ship => "\\#/",
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

fn place_random(entities: &Vec<Entity>, rng: &mut rand::ThreadRng, etype: EType) -> Entity {
    loop {
        let x = rng.gen_range(0, WIDTH);
        let y = rng.gen_range(0, HEIGHT);
        if !check_collision(entities, x, y) {
            return Entity::new(x, y, etype);
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
        let ship = place_random(&entities, &mut rng, EType::Ship);
        entities.push(ship);
    }

    // HQ
    let hq = place_random(&entities, &mut rng, EType::HQ);
    entities.push(hq);

    // Mines
    for _i in 0..rng.gen_range(8, 15) {
        let mine = place_random(&entities, &mut rng, EType::Mine);
        entities.push(mine);
    }

    // Sea Monsters
    for _i in 0..4 {
        let monster = place_random(&entities, &mut rng, EType::Monster);
        entities.push(monster);
    }

    // Add components to entities
    for e in &mut entities {
        if e.etype == EType::Ship || e.etype == EType::Monster {
            e.components.push(Component::Velocity (
                rng.gen_range(-1, 1),
                rng.gen_range(-1, 1),
            ));
        }
    }

    entities
}

fn move_ships(entities: &mut Vec<Entity>) {
    // If I create an unmoved / blocked / moved enum
    // and put it in the Entity, can I do this all in-place?
    // let blocked = Vec::new();
    let mut moved = Vec::new();

    /* This doesn't work, "cannot move out of borrowed context"
    entities.retain(|&e|  {
        if e.components.is_empty() {
            moved.push(e);      // try e.clone()?
            false
        } else {
            true
        }
    });
    */
    let mut i = 0;
    while i != entities.len() {
        // TODO:  If more components are added, check for Velocity.
        if entities[i].components.is_empty() {
            moved.push(entities.remove(i));
        } else {
            i += 1;
        }
    }

    while entities.len() != 0 {
        // For each (unmoved) entity
        let mut i = 0;
        let starting_len = entities.len();
        while i != entities.len() {
            // Calculate destination
            // let &mut vel : Component::Velocity = entities[i].components[0];
            let Component::Velocity(dx, dy) = entities[i].components[0];
            let x = entities[i].pos.x.wrapping_add(dx as usize);
            let y = entities[i].pos.y.wrapping_add(dy as usize);
            if x >= WIDTH || y >= WIDTH {
                // TODO:  Bounce
                println!("thud");
                moved.push(entities.remove(i));
            } else if check_collision(entities, x, y) {
                // If collision in unmoved, leave unmoved for now
                i += 1;
            } else if check_collision(&moved, x, y) {
                // If collision in moved, can't move.
                // TODO:  ship collision, monster fun, etc.
                println!("bang");
                moved.push(entities.remove(i));
            } else {
                // No collision, move.
                entities[i].pos = Position {x, y};
                moved.push(entities.remove(i));
            }
        }
        if starting_len == entities.len() {
            // Movement blocked, give up
            println!("stalemate!");
            break;
        }
    }
    entities.extend(moved.drain(..));
}

fn main() {
    let mut entities = setup();
    print_map(&entities);
    move_ships(&mut entities);
    print_map(&entities);
}

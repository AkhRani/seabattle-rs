extern crate rand;

use rand::Rng;

const WIDTH: usize = 20;
const HEIGHT: usize = 20;

#[derive(PartialEq, Eq, Clone)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(PartialEq, Clone, Debug)]
enum EType {
    Player,
    Island,
    Ship,
    Mine,
    HQ,
    Monster
}

#[derive(Clone)]
enum Component {
    Velocity(i8, i8),
}

impl Component {
    fn new_vel(rng : &mut rand::ThreadRng) -> Component {
        let mut dx = 0;
        let mut dy = 0;
        while dx == 0 && dy == 0 {
            dx = rng.gen_range(-1, 1);
            dy = rng.gen_range(-1, 1);
        }
        Component::Velocity(dx, dy)
    }
}

#[derive(Clone)]
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

type EntityColl = std::collections::VecDeque<Entity>;

fn print_map(entities: &EntityColl) {
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

fn get_collision(entities: &EntityColl, x:usize, y:usize) -> Option<Entity> {
    for e in entities {
        let pos = Position{x, y};
        if pos == e.pos {
            return Some(e.clone());
        }
    }
    None
}

fn check_collision(entities: &EntityColl, x:usize, y:usize) -> bool {
    for e in entities {
        // Why doesn't this work? if Position{x, y} == e.pos {
        let pos = Position{x, y};
        if pos == e.pos {
            return true;
        }
    }
    return false;
}

fn place_random(entities: &EntityColl, rng: &mut rand::ThreadRng, etype: EType) -> Entity {
    loop {
        let x = rng.gen_range(0, WIDTH);
        let y = rng.gen_range(0, HEIGHT);
        if !check_collision(entities, x, y) {
            return Entity::new(x, y, etype);
        }
    }
}

fn setup() -> EntityColl {
    let mut entities = EntityColl::new();

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
                entities.push_back(Entity::new(x, y, EType::Island));
            }
            i += 1;
        }
    }

    // Player
    entities.push_back(Entity::new(10, 10, EType::Player));

    // Enemy Ships
    let mut rng = rand::thread_rng();
    for _i in 0..rng.gen_range(15, 31) {
        let mut ship = place_random(&entities, &mut rng, EType::Ship);
        ship.components.push(Component::new_vel(&mut rng));
        entities.push_back(ship);
    }

    // HQ
    let hq = place_random(&entities, &mut rng, EType::HQ);
    entities.push_back(hq);

    // Mines
    for _i in 0..rng.gen_range(8, 15) {
        let mine = place_random(&entities, &mut rng, EType::Mine);
        entities.push_back(mine);
    }

    // Sea Monsters
    for _i in 0..4 {
        let mut monster = place_random(&entities, &mut rng, EType::Monster);
        monster.components.push(Component::new_vel(&mut rng));
        entities.push_back(monster);
    }
    entities
}

fn move_enemy(e: Entity, unmoved: &mut EntityColl, moved: &mut EntityColl) {
    // Calculate destination
    let Component::Velocity(dx, dy) = e.components[0];
    let x = e.pos.x.wrapping_add(dx as usize);
    let y = e.pos.y.wrapping_add(dy as usize);
    if x >= WIDTH || y >= WIDTH {
        // TODO:  Bounce
        println!("thud");
        moved.push_back(e);
    } else if check_collision(unmoved, x, y) {
        // Might be able to move later
        unmoved.push_back(e);
    } else {
        match get_collision(&moved, x, y) {
            Some(crashee) => {
                // If collision in moved, can't move.
                // TODO:  ship collision, monster fun, etc.
                println!("bang, hit {:?}", crashee.etype);
                moved.push_back(e);
            }
            None => {
                // No collision, move.
                let mut moved_entity = e;
                moved_entity.pos = Position {x, y};
                moved.push_back(moved_entity);
            }
        }
    }
}

fn move_enemies(entities: &mut EntityColl) {
    let mut moved = EntityColl::with_capacity(entities.len());
    let mut unmoved = EntityColl::with_capacity(entities.len());

    // Non-moving entities get precedence
    // Might be able to ensure this based on initial order and
    // the generic movement function.
    while let Some(e) = entities.pop_front() {
        if e.components.is_empty() {
            moved.push_back(e);
        } else {
            unmoved.push_back(e);
        }
    }

    while unmoved.len() != 0 {
        let unmoved_len = unmoved.len();
        for _i in 0..unmoved_len {
            let e = unmoved.pop_front().unwrap();
            move_enemy(e, &mut unmoved, &mut moved);
        }
        if unmoved_len == unmoved.len() {
            // TODO:  Change direction of remaining unmoved entities
            println!("Stalemate");
        }
    }
    entities.extend(moved.drain(..));
    entities.extend(unmoved.drain(..));
}

fn main() {
    let mut entities = setup();
    print_map(&entities);
    move_enemies(&mut entities);
    print_map(&entities);
}

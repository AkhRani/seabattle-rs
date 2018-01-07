extern crate rand;
use rand::Rng;

extern crate enum_map;
use enum_map::EnumMap;

use std::io::{self, stdin, stdout, Write};

#[macro_use] extern crate enum_map_derive;

const WIDTH: usize = 20;
const HEIGHT: usize = 20;

#[derive(Debug, EnumMap)]
enum SubSystem {
    Engines,
    Sonar,
    Torpedos,
    Missiles,
    Manuevering,
    Computers,
    Resupply,
    Sabotage,
    Converter,
}

struct PlayerInfo {
    name: String,
    damage: EnumMap<SubSystem, f32>,
    crew: u32,
    power: u32,
    fuel: u32,
    torpedos: u32,
    missiles: u32,
}

fn prompt(pstr: &'static str) -> String {
    print!("{}? ", pstr);
    stdout().flush().unwrap();
    let mut result = String::new();
    stdin().read_line(&mut result).expect("Failed to read line");
    let len = result.trim_right().len();
    result.truncate(len);
    result
}

impl PlayerInfo {
    fn new() -> PlayerInfo {
        PlayerInfo {
            name: prompt("What is your name, captain"),
            damage: EnumMap::<SubSystem, f32>::new(),
            crew: 30,
            power: 6000,
            fuel: 2500,
            torpedos: 10,
            missiles: 3,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
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

#[derive(Debug)]
enum EResolution {
    CrasheeDestroyed,
    MoverDestroyed,
    MoverChangeDirection
}

#[derive(Clone, Debug)]
enum Component {
    Velocity(i8, i8),
}

impl Component {
    fn new_vel() -> Component {
        let mut dx = 0;
        let mut dy = 0;
        while dx == 0 && dy == 0 {
            dx = rand::thread_rng().gen_range(-1, 2);
            dy = rand::thread_rng().gen_range(-1, 2);
        }
        Component::Velocity(dx, dy)
    }
}

#[derive(Clone, Debug)]
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

// Note:  If you alias like this, you can't define your own methods.
type EntityColl = std::collections::VecDeque<Entity>;

// Note:  If you newtype like this, you can't use existing methods by default.
// struct EntityColl(std::collections::VecDeque<Entity>);

fn count_all_of(entities: &EntityColl, etype: EType) -> u32 {
    entities.iter().
         map(|e: &Entity| if e.alive && e.etype == etype {1} else {0}).
         fold(0, |acc, ship| acc+ship)
}

fn change_direction(mut e: Entity) -> Entity {
    // TODO:  If we have different types of components, replace the right one
    e.components[0] = Component::new_vel();
    println!("New velocity: {:?}", e.components[0]);
    e
}

fn print_map(entities: &EntityColl) {
    let mut tiles = [["   "; WIDTH]; HEIGHT];
    for e in entities.iter() {
        if !e.alive {
            panic!("Unexpected dead entity in print_map: {:?}", e);
        }
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

fn get_collision(entities: &mut EntityColl, x:usize, y:usize) -> Option<Entity> {
    let pos = Position{x, y};
    for i in 0..entities.len() {
        if pos == entities[i].pos && entities[i].alive {
            return entities.swap_remove_back(i);
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
        ship.components.push(Component::new_vel());
        entities.push_back(ship);
    }
    /*
    let mut ship = Entity::new(6, 9, EType::Ship);
    ship.components.push(Component::Velocity(1, 0));
    entities.push_back(ship);
    */

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
        monster.components.push(Component::new_vel());
        entities.push_back(monster);
    }
    entities
}

fn resolve_collision(e: &Entity, crashee: &Entity) -> EResolution {
    use EType::*;
    match e.etype {
        Ship => match crashee.etype {
            Island | Ship | Player | HQ => {
                println!("{:?} changed direction to avoid {:?}",
                         e.etype, crashee.etype);
                return EResolution::MoverChangeDirection;
            },
            Mine => {
                println!("{:?} destroyed by a mine!", e.etype);
                return EResolution::MoverDestroyed;
            },
            Monster => {
                println!("{:?} eaten by a monster!", e.etype);
                return EResolution::MoverDestroyed;
            }
        }
        Monster => match crashee.etype {
            Island | Player | HQ => {
                println!("{:?} changed direction to avoid {:?}",
                         e.etype, crashee.etype);
                return EResolution::MoverChangeDirection;
            },
            Ship => {
                println!("Ship eaten by a moving monster!");
                return EResolution::CrasheeDestroyed;
            },
            Mine => {
                println!("{:?} destroyed by a mine!", e.etype);
                return EResolution::MoverDestroyed;
            },
            Monster => {
                println!("{:?} eaten by a monster!", e.etype);
                return EResolution::MoverDestroyed;
            }
        }
        _ => panic!("Unexpected mover type {:?}", e.etype)
    }
}

fn move_enemy(e: Entity, unmoved: &mut EntityColl, moved: &mut EntityColl) {
    // Calculate destination
    let Component::Velocity(dx, dy) = e.components[0];
    let x = e.pos.x.wrapping_add(dx as usize);
    let y = e.pos.y.wrapping_add(dy as usize);
    if x >= WIDTH || y >= WIDTH {
        // TODO:  Bounce
        println!("thud");
        moved.push_back(change_direction(e));
    } else if check_collision(unmoved, x, y) {
        // Might be able to move later
        unmoved.push_back(e);
    } else {
        // If collision in moved, we have to resolve
        match get_collision(moved, x, y) {
            Some(crashee) => {
                use EResolution::*;
                match resolve_collision(&e, &crashee) {
                    CrasheeDestroyed => {
                        let mut moved_entity = e;
                        moved_entity.pos = Position {x, y};
                        moved.push_back(moved_entity);
                    },

                    MoverDestroyed => moved.push_back(crashee),

                    MoverChangeDirection => {
                        unmoved.push_back(change_direction(e));
                        moved.push_back(crashee);
                    }
                }
            },
            None => {
                // No collision, move.
                let mut moved_entity = e;
                moved_entity.pos = Position {x, y};
                // println!("Moving entity: {:?}", moved_entity);
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
            if e.alive {
                move_enemy(e, &mut unmoved, &mut moved);
            }
        }
        if unmoved_len == unmoved.len() {
            // Either un-moved entities are trying to move through
            // each other, or an un-moved entity is blocked by moved
            // entities.
            println!("Stalemate");
            // Change direction of remaining unmoved entities
            for e in &mut unmoved {
                e.components[0] = Component::new_vel();
                println!("New velocity: {:?}", e.components[0]);
            }
            // Better luck next time
            break;
        }
    }
    // TODO:  Filter out dead entities
    entities.extend(moved.into_iter());
    entities.extend(unmoved.into_iter());
}

fn status_report(entities: &EntityColl, info: &PlayerInfo) {
    println!("");
    println!("# of enemy ships left...{}", count_all_of(entities, EType::Ship));
    println!("    SYSTEM       HEALTH  (negative is bad)");
    println!("    ------       ------");
    for (key, value) in info.damage {
        println!("    {:12} {:2.4}", format!("{:?}", key), value);
    }
}

fn main() {
    let mut entities = setup();
    let mut player_info = PlayerInfo::new();

    println!("You must destroy {} enemy ships to win, {}.",
             count_all_of(&entities, EType::Ship), player_info.name);
    print_map(&entities);
    move_enemies(&mut entities);
    status_report(&entities, &player_info);
    print_map(&entities);
}

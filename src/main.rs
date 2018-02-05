extern crate rand;
use rand::{Rng, thread_rng};

// Magic to generate random SubSystem.
#[macro_use]
extern crate rand_derive;

extern crate enum_map;
use enum_map::EnumMap;

use std::io::{stdin, stdout, Write};
use std::{thread, time};

#[macro_use] extern crate enum_map_derive;

const WIDTH: usize = 20;
const HEIGHT: usize = 20;

#[derive(Debug)]
enum Command {
    Sonar,
    Status,
    Torpedo,
    Surrender,
}

#[derive(Debug, EnumMap, Rand)]
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
    alive: bool,
    damage: EnumMap<SubSystem, f32>,
    depth: u32,
    crew: u32,
    power: u32,
    fuel: u32,
    torpedos: u32,
    missiles: u32,
}

fn prompt(pstr: &str) -> String {
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
            name: prompt("What is your name"),
            alive: true,
            damage: EnumMap::<SubSystem, f32>::new(),
            depth: 100,
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

impl Position {
    fn distance(&self, other: &Position, range: f32) -> Option<f32> {
        let dx = ((self.x as f32 - other.x as f32)).abs();
        let dy = ((self.y as f32 - other.y as f32)).abs();
        if dx <= range && dy <= range {
            return Some((dx*dx + dy*dy).sqrt());
        }
        None
    }
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

// Collision outcomes / resolutions
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
    components: Vec<Component>
}

impl Entity {
    // syntactic sugar
    fn new(x: usize, y: usize, etype: EType) -> Entity {
        Entity {
            pos: Position {x, y},
            etype,
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
         map(|e: &Entity| if e.etype == etype {1} else {0}).
         fold(0, |acc, ship| acc+ship)
}

// Shorthand function, Original BASIC code uses this a lot.
fn rnd() -> f32 {
    thread_rng().next_f32()
}

fn change_direction(mut e: Entity) -> Entity {
    // TODO:  If we have different types of components, replace the right one
    e.components[0] = Component::new_vel();
    // println!("New velocity: {:?}", e.components[0]);
    e
}

fn get_player_pos(entities: &EntityColl) -> Option<Position> {
    for e in entities.iter() {
        if EType::Player == e.etype {
            return Some(e.pos.clone());
        }
    }
    None
}

fn get_collision(entities: &mut EntityColl, x:usize, y:usize) -> Option<Entity> {
    let pos = Position{x, y};
    for i in 0..entities.len() {
        if pos == entities[i].pos {
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

fn get_command(player_info: &PlayerInfo) -> Command {
    let prompt_str = &format!("What are your orders, {}", player_info.name);
    loop {
        use Command::*;
        let input = prompt(prompt_str);
        match input.parse::<i32>() {
            Ok(v) => match v {
                1 => return Sonar,
                2 => return Torpedo,
                5 => return Status,
                9 => return Surrender,
                _ => {}
            }
            Err(_) => {}
        }
        println!("The Commands are:");
        println!("      0: Navigation");
        println!("      1: Sonar");
        println!("      2: Torpedo");
        println!("      5: Status");
        println!("      9: Surrender");
    }
}

fn get_direction() -> (i8, i8) {
    // Prompt the player for a direction for navigation, sonar, or weapons.
    loop {
        let input = prompt("What direction");
        match input.parse::<i32>() {
            Ok(v) => match v {
                1 => return (-1, 1),
                2 => return (0, 1),
                3 => return (1, 1),
                4 => return (-1, 0),
                6 => return (1, 0),
                7 => return (-1, -1),
                8 => return (0, -1),
                9 => return (1, -1),
                _ => ()
            }
            _ => ()
        }
        println!("The Directions are:");
        println!(" 7 8 9");
        println!("  \\|/");
        println!(" 4-*-6");
        println!("  /|\\");
        println!(" 1 2 3");
    }
}

/**********************************************************************************
 * Command #1, sonar
 *********************************************************************************/
fn sonar(entities: &EntityColl) {
    let mut tiles = [["   "; WIDTH]; HEIGHT];
    for e in entities.iter() {
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

/**********************************************************************************
 * Command #2, torpedo control
 *********************************************************************************/
fn fire_torpedo(entities: &mut EntityColl, pi: &mut PlayerInfo) {
    if pi.damage[SubSystem::Torpedos] < 0. {
        println!("Torpedo tubes are under repair, {}.", pi.name);
    } else if pi.crew < 10 {
        println!("Not enough crew to fire torpedos, {}.", pi.name);
    } else if pi.torpedos == 0 {
        println!("No torpedos left, {}.", pi.name);
    } else if pi.depth >= 2000 && rnd() > 0.5 {
        println!("Pressure implodes sub upon firing... You're crushed!!");
        pi.alive = false;
    } else {
        pi.torpedos -= 1;
        pi.power -= 150;

        let (dx, dy) = get_direction();
        // Note:  Docs say range is 7-13, but equation below does not match.
        let mut range = 7 - (rnd()*5.).round() as i32;
        if pi.depth > 50 {
            range = range + 5;
        }

        let mut success = false;
        let Position{mut x, mut y} = get_player_pos(entities).unwrap();
        for i in 0..range {
            x = x.wrapping_add(dx as usize);
            y = y.wrapping_add(dy as usize);
            if x >= WIDTH || y >= WIDTH {
                println!("Torpedo out of range... Ineffectual {}", pi.name);
                break;
            }

            // Add some suspense
            print!("..{}..\r", i);
            stdout().flush().unwrap();
            thread::sleep(time::Duration::from_millis(500));

            if let Some(e) = get_collision(entities, x, y) {
                resolve_torpedo(e, entities, pi);
                success = true;
                break;
            }
        }
        if !success {
            println!("Dud.");
        }
    }
}

fn resolve_torpedo(e: Entity, entities: &mut EntityColl, pi: &mut PlayerInfo) {
    use EType::*;
    match e.etype {
        Player => {
            panic!("How did you torpedo yourself?!?");
        }
        Island => {
            println!("You took out some island, {}.", pi.name);
        }
        Ship => {
            println!("Ouch!  You got one, {}!", pi.name);
        }
        Mine => {
            println!("BLAM!!  Shot wasted on a mine.");
            entities.push_back(e);
        }
        HQ => {
            println!("You blew up your headquarters, {}!", pi.name);
        }
        Monster => {
            println!("A sea monster had a torpedo for lunch!");
            entities.push_back(e);
        }
    }
}

/**********************************************************************************
 * Command #5, status report
 *********************************************************************************/
fn status_report(entities: &EntityColl, pi: &PlayerInfo) {
    if pi.damage[SubSystem::Computers] < 0. {
        println!("No reports are able to get through, {}.", pi.name);
    } else if pi.crew <= 3 {
        println!("No one left to give the report, {}.", pi.name);
    } else {
        println!("");
        println!("# of enemy ships left...{}", count_all_of(entities, EType::Ship));
        println!("# of power units left...{}", pi.power);
        println!("# of torpedos  left.....{}", pi.torpedos);
        println!("# of missiles left......{}", pi.missiles);
        println!("# of crewmen left.......{}", pi.crew);
        println!("LBS. of fuel left.......{}", pi.fuel);
        println!();
        println!("    SYSTEM       HEALTH  (negative is bad)");
        println!("    ------       ------");
        for (key, value) in pi.damage {
            println!("    {:12} {:2.4}", format!("{:?}", key), value);
        }
    }
    // println!("You are at {?:}", pi.pos);
}

/**********************************************************************************
 * Command #9, surrender
 *********************************************************************************/
fn surrender(player_info: &mut PlayerInfo) {
    println!("Coward!  You're not very patriotic, {}.", player_info.name);
    player_info.alive = false;
}

/**********************************************************************************
 * Enemy movement
 *********************************************************************************/
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
            // Either un-moved entities are trying to move through
            // each other, or an un-moved entity is blocked by moved
            // entities.
            println!("Stalemate");
            // Change direction of remaining unmoved entities
            for e in &mut unmoved {
                e.components[0] = Component::new_vel();
                // println!("New velocity: {:?}", e.components[0]);
            }
            // Better luck next time
            break;
        }
    }
    // TODO:  Filter out dead entities
    entities.extend(moved.into_iter());
    entities.extend(unmoved.into_iter());
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
        println!("{:?} changed direction to stay in the area.", e.etype);
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

/**********************************************************************************
 * Enemy attacks
 *********************************************************************************/
fn retaliation(entities: &EntityColl, pi: &mut PlayerInfo) {
    let mut threat = 0f32;
    let ppos = get_player_pos(entities).unwrap();
    for e in entities {
        if e.etype == EType::Ship {
            if let Some(dist) = ppos.distance(&e.pos, 4f32) {
                println!("Enemy ship at {:?} firing...", e.pos);
                threat += rnd() / dist;
            }
        }
    }
    println!("Threat: {}", threat);

    let mut power_drain = 0;
    let mut system_count = 0;
    let mut damage = 0f32;

    if threat != 0. {
        println!("Depth charges off {} side, {}!",
                 if rnd() > 0.5 { "port" } else { "starboard" },
                 pi.name);
        if threat <= 0.13 && rnd() <= 0.92 {
            println!("No real damage sustained, {}.", pi.name);
        } else if threat <= 0.36 && rnd() <= 0.96 {
            println!("Light, superficial damage sustained, {}!", pi.name);
            power_drain = 50;
            system_count = 1;
            damage = 2.;
        } else if threat <= 0.6 && rnd() <= 0.975 {
            println!("Moderate damange, repairs needed, {}!!", pi.name);
            power_drain = 75 + (rnd()*30.) as u32;
            system_count = 2;
            damage = 8.;
        } else if threat <= 0.9 && rnd() <= 0.983 {
            println!("Heavy damage!! Repairs immediate, {}!!", pi.name);
            power_drain = 200 + (rnd()*76.) as u32;
            system_count = 4;
            damage = 9.;
        } else {
            println!("Damage Critical!!!  We need help!!!");
            println!("Send 'HELP' in code.  Here is the code: ");
            print!("QOIJ");
            stdout().flush().unwrap();
            thread::sleep(time::Duration::from_millis(500));
            print!("XXXX");
            power_drain = 200 + (rnd()*76.) as u32;
            system_count = 4;
            damage = 11.;
        }
    }

    pi.power = pi.power.saturating_sub(power_drain);
    for _ in 0..system_count {
        let damaged_system: SubSystem = thread_rng().gen();
        pi.damage[damaged_system] -= rnd() * damage;
    }
}

fn repair(pi: &mut PlayerInfo) {
    for (key, value) in pi.damage {
        if value < 3. {
            let mut repair = rnd() * (2.+rnd()*2.);
            if pi.depth < 51 || pi.depth > 2000 {
                repair *= 2.;
            }
            pi.damage[key] = value + repair;
        }
    }
}

fn main() {
    let mut entities = setup();
    let mut player_info = PlayerInfo::new();

    loop {
        let ships = count_all_of(&entities, EType::Ship);
        println!("You must destroy {} enemy ships to win, {}.",
                 ships, player_info.name);
        loop {
            use Command::*;
            match get_command(&player_info) {
                Sonar => sonar(&entities),
                Status => status_report(&entities, &player_info),
                Torpedo => {
                    fire_torpedo(&mut entities, &mut player_info);
                    break;
                },
                Surrender => {
                    surrender(&mut player_info);
                    break;
                },
            }
        }
        // Player may have destroyed itself, surrendered, or won.
        if !player_info.alive {
            break;
        } else if ships == 0 {
            println!("You Won!  All hail {} the glorious!!", player_info.name);
            break;
        } else {
            retaliation(&entities, &mut player_info);
            if !player_info.alive {
                break;
            }
            move_enemies(&mut entities);
            repair(&mut player_info);
        }
    }
    let ships = count_all_of(&entities, EType::Ship);
    if ships > 0 {
        println!("There are still {} enemy ships left, {}.",
                 ships, player_info.name);
        println!("You will be demoted to the rank of Deck Scrubber!!");
        // TODO:  Create outer loop, ask player for another game.
    } else {
        println!("Good work {}, you got them all!!", player_info.name);
        println!("Promotion and commendations will be given immediately!");
    }
}

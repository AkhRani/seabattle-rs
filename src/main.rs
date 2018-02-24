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
    Navigate,
    Sonar,
    Torpedo,
    Missile,
    Manuever,
    Status,
    Resupply,
    // Sabotage
    Convert,
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
    depth: i32,
    crew: u32,
    power: u32,
    fuel: u32,
    torpedos: u32,
    missiles: u32,
    resupply_left: u32,
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
            resupply_left: 2,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    // Return the euclidian distance of the other position, but only if
    // each dimension is within the distance specified by range.
    fn distance_within(&self, other: &Position, range: f32) -> Option<f32> {
        let dx = ((self.x as f32 - other.x as f32)).abs();
        let dy = ((self.y as f32 - other.y as f32)).abs();
        if dx <= range && dy <= range {
            return Some((dx*dx + dy*dy).sqrt());
        }
        None
    }

    // Return the euclidian distance of the other position
    fn distance(&self, other: &Position) -> f32 {
        let dx = self.x as f32 - other.x as f32;
        let dy = self.y as f32 - other.y as f32;
        (dx*dx + dy*dy).sqrt()
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

fn get_first_pos(entities: &EntityColl, etype: EType) -> Option<Position> {
    for e in entities.iter() {
        if etype == e.etype {
            return Some(e.pos.clone());
        }
    }
    None
}

fn get_first(entities: &mut EntityColl, etype: EType) -> Option<Entity> {
    for i in 0..entities.len() {
        if etype == entities[i].etype {
            return entities.swap_remove_back(i);
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
                0 => return Navigate,
                1 => return Sonar,
                2 => return Torpedo,
                3 => return Missile,
                4 => return Manuever,
                5 => return Status,
                6 => return Resupply,
                8 => return Convert,
                9 => return Surrender,
                _ => {}
            }
            Err(_) => {}
        }
        println!("The Commands are:");
        println!("      0: Navigate");
        println!("      1: Sonar");
        println!("      2: Fire Torpedo");
        println!("      3: Fire Missile");
        println!("      4: Manuever");
        println!("      5: Status");
        println!("      6: Resupply");
        println!("      8: Convert Power");
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
 * Command #0, navigate
 *********************************************************************************/
fn navigate(entities: &mut EntityColl, pi: &mut PlayerInfo) -> bool {
    if pi.damage[SubSystem::Engines] < 0. {
        println!("Engines are under repair, {}.", pi.name);
        return false;
    }
    if pi.crew <= 8 {
        println!("Not enough crew to man the engines, {}.", pi.name);
        return false;
    }
    let (dx, dy) = get_direction();
    let p = get_power(pi.power);

    let mut speed = 1.;
    if pi.depth <= 50 {
        speed -= 0.23 + rnd()/10.;
    }
    if p > 1000 && rnd() >= 0.43 {
        println!("Atomic pile goes supercritical, {}!!", pi.name);
        println!("Headquarters will warn all subs to stay away");
        println!("From radioactive area!");
        pi.alive = false;
    }
    let range = ((p as f32) / 100. * speed).round() as u32;

    // extract player entity
    let mut player = get_first(entities, EType::Player).unwrap();
    let Position{mut x, mut y} = player.pos;
    for _ in 0..range {
        pi.power = pi.power.saturating_sub(100);
        let next_x = x.wrapping_add(dx as usize);
        let next_y = y.wrapping_add(dy as usize);
        if next_x >= WIDTH || next_y >= HEIGHT {
            println!("You can't leave the area, {}.", pi.name);
            break;
        }
        if let Some(crashee) = get_collision(entities, next_x, next_y) {
            use EType::*;
            match crashee.etype {
                Island => {
                    println!("You almost ran aground, {}!", pi.name);
                    break;
                },
                Ship => {
                    println!("You rammed a ship!! You're both sunk!");
                    pi.alive = false;
                },
                HQ => {
                    println!("You rammed your headquarters!! You're sunk!");
                    pi.alive = false;
                },
                Mine => {
                    println!("You've been blown up by a mine, {}!", pi.name);
                    pi.alive = false;
                },
                Monster => {
                    if rnd() >= 0.21 {
                        println!("You were eaten by a sea monster, {}!", pi.name);
                        pi.alive = false;
                    } else {
                        // Note:  In this case, the monster and the player occupy the
                        // same position.  In the original game, if this was the final
                        // movement of the player, the sea monster would be eliminated.
                        // For now, I'm unconditionally eliminating the sea monster
                        // by not putting the entity back into the pool.
                        println!("You rammed a sea monster!  Lucky you!");
                    }
                },
                Player => {
                    panic!("How did you ram yourself?!?!");
                }
            }
        } else {
            x = next_x;
            y = next_y;

            if nearby_monsters(entities, x, y) {
                println!("You have been eaten by a sea monster, {}!!", pi.name);
                pi.alive = false;
            }
        }
        if !pi.alive {
            break;
        }
    }
    if pi.alive {
        player.pos = Position {x, y};
        entities.push_back(player);
    }
    true
}

// Return true if the player was eaten by a nearby sea monster
fn nearby_monsters(entities: &EntityColl, x: usize, y: usize) -> bool {
    let mut nearby = false;
    // For each sea monster within +/- 2 cells, 25% chance to be eaten.
    let pos = Position {x, y};
    for e in entities {
        if e.etype == EType::Monster {
            if let Some(_) = pos.distance_within(&e.pos, 2f32) {
                nearby = true;
                if rnd() <= 0.25 {
                    return true;
                }
            }
        }
    }
    if nearby {
        println!("You just had a narrow escape with a sea monster.");
    }
    false
}


fn get_power(avail: u32) -> u32 {
    let prompt_str = &format!("Power available={}.  Power to use", avail);
    loop {
        let input = prompt(prompt_str);
        match input.parse::<u32>() {
            Ok(p) => {
                if p < avail {
                    return p;
                }
            },
            Err(_) => {}
        }
    }
}

/**********************************************************************************
 * Command #1, sonar
 *********************************************************************************/
fn sonar(entities: &EntityColl, pi: &mut PlayerInfo) -> bool {
    if pi.damage[SubSystem::Sonar] < 0. {
        println!("Sonar is under repair.");
    } else if pi.crew <= 5 {
        println!("Not enough crew to operate sonar.");
    } else {
        // TODO:  linear vs map.
        let mut tiles = [["  "; WIDTH]; HEIGHT];
        for e in entities.iter() {
            // TODO:  Sonar noise
            /*
            tiles[e.pos.x][e.pos.y] = match e.etype {
                EType::Player => "(X)",
                EType::Island => "***",
                EType::Ship => "\\#/",
                EType::Mine => " $ ",
                EType::HQ => "-H-",
                EType::Monster => "SSS",
            }
            */
            tiles[e.pos.x][e.pos.y] = match e.etype {
                EType::Player => "==",
                EType::Island => "%%",
                EType::Ship => "<>",
                EType::Mine => " *",
                EType::HQ => "HQ",
                EType::Monster => "SS",
            }
        }

        println!("{}", ".".repeat(WIDTH*2+2));
        for y in 0..HEIGHT {
            print!(".");
            for x in 0..WIDTH {
                print!("{}", tiles[x][y]);
            }
            println!(".");
        }
        println!("{}", ".".repeat(WIDTH*2+2));
        // Same power cost for map and linear sonar
        pi.power = pi.power.saturating_sub(50);
    }
    false
}

/**********************************************************************************
 * Command #2, torpedo control
 *********************************************************************************/
fn fire_torpedo(entities: &mut EntityColl, pi: &mut PlayerInfo) -> bool {
    let mut turn_over = false;
    if pi.damage[SubSystem::Torpedos] < 0. {
        println!("Torpedo tubes are under repair, {}.", pi.name);
    } else if pi.crew < 10 {
        println!("Not enough crew to fire torpedos, {}.", pi.name);
    } else if pi.torpedos == 0 {
        println!("No torpedos left, {}.", pi.name);
    } else if pi.depth >= 2000 && rnd() > 0.5 {
        println!("Pressure implodes sub upon firing... You're crushed!!");
        pi.alive = false;
        turn_over = true;
    } else {
        turn_over = true;
        pi.torpedos -= 1;
        pi.power = pi.power.saturating_sub(150);

        let (dx, dy) = get_direction();
        // Note:  Docs say range is 7-13, but equation below does not match.
        let mut range = 7 - (rnd()*4.).round() as i32;
        if pi.depth > 50 {
            range = range + 5;
        }

        let mut success = false;
        let Position{mut x, mut y} = get_first_pos(entities, EType::Player).unwrap();
        for i in 0..range {
            x = x.wrapping_add(dx as usize);
            y = y.wrapping_add(dy as usize);
            if x >= WIDTH || y >= HEIGHT {
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
    turn_over
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
 * Command #3, Polaris Missiles
 *********************************************************************************/
fn fire_missile(entities: &mut EntityColl, pi: &mut PlayerInfo) -> bool {
    if pi.damage[SubSystem::Missiles] < 0. {
        println!("Missile silos are under repair, {}.", pi.name);
        return false;
    }
    if pi.crew <= 23 {
        println!("Not enough crew left to launch a missile, {}.", pi.name);
        return false;
    }
    if pi.missiles == 0 {
        println!("No missiles left, {}.", pi.name);
        return false;
    }
    if pi.depth <= 50 || pi.depth >= 2000 {
        let input = prompt("Recommend you do not fire at this depth... Proceed (Y/n)");
        if input.starts_with("n") || input.starts_with("N") {
            return false;
        }
        if rnd() >= 0.5 {
            println!("Missile explodes upon firing {}!! You're Dead!!", pi.name);
            pi.alive = false;
            return true;
        }
    }

    let (dx, dy) = get_direction();
    let Position{x, y} = get_first_pos(entities, EType::Player).unwrap();
    loop {
        let input = prompt("Fuel (LBS.)");
        if let Ok(fuel) = input.parse::<u32>() {
            if fuel > 0 && fuel <= pi.fuel {
                pi.fuel -= fuel;
                pi.missiles -= 1;
                let range = (fuel as f32 / 75.0).round() as i32;
                let mx = x.wrapping_add((dx as i32 * range) as usize);
                let my = y.wrapping_add((dy as i32 * range) as usize);
                if mx >= WIDTH || my >= HEIGHT {
                    println!("Missile out of sonar tracking {}.  Missile lost.", pi.name);
                } else {
                    resolve_missile(mx, my, entities, pi);
                }
                return true;
            }
        }
        println!("You have {} LBS. left, {}.", pi.fuel, pi.name);
    }
}

fn resolve_missile(x: usize, y: usize, entities: &mut EntityColl, pi: &mut PlayerInfo) {
    let pos = Position {x, y};
    let (mut monsters, mut ships, mut mines, mut island) = (0, 0, 0, 0);
    for _i in 0..entities.len() {
        let e = entities.pop_front().unwrap();
        if let Some(_) = pos.distance_within(&e.pos, 2f32) {
            use EType::*;
            match e.etype {
                Player => {
                    println!("You just destroyed yourself, {}!  Dummy!!", pi.name);
                    pi.alive = false;
                    // Note:  Original code would kill player instantly.
                    // I'm going to allow the possibility of a draw.
                }
                Island => island += 1,
                Ship => ships += 1,
                Mine => mines += 1,
                HQ => {
                    println!("You've destroyed your headquarters, {}!!", pi.name);
                }
                Monster => monsters += 1,
            }
        } else {
            entities.push_back(e);
        }
    }
    if island > 0 {
        println!("You blew out some island, {}.", pi.name);
    }
    if mines > 0 {
        println!("You destroyed {} mines, {}.", mines, pi.name);
    }
    if monsters > 0 {
        println!("You got {} sea monsters, {}!! Good work!", monsters, pi.name);
    }
    if ships > 0 {
        println!("You destroyed {} enemy ships, {}!!", ships, pi.name);
    }
}

/**********************************************************************************
 * Command #4, Manuever
 *********************************************************************************/
fn manuever(pi: &mut PlayerInfo) -> bool {
    if pi.damage[SubSystem::Computers] < 0. {
        println!("Ballast controls are being repaired {}.", pi.name);
        return false;
    }
    if pi.crew <= 12 {
        println!("There are not enough crew to work the controls, {}.", pi.name);
        return false;
    }
    loop {
        let input = prompt ("New depth");
        if let Ok(depth) = input.parse::<i32>() {
            if depth >= 0 && depth < 3000 {
                let power_used = ((depth - pi.depth).abs() as u32 + 1) / 2;
                pi.power = pi.power.saturating_sub(power_used);
                pi.depth = depth;
            } else {
                println!("Hull crushed by pressure, {}!!", pi.name);
                pi.alive = false;
            }
            return true
        }
    }
}

/**********************************************************************************
 * Command #5, status report
 *********************************************************************************/
fn status_report(entities: &EntityColl, pi: &PlayerInfo) -> bool {
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
    println!("You are at {:?}", get_first_pos(entities, EType::Player).unwrap());
    println!("Depth: {}", pi.depth);
    false
}

/**********************************************************************************
 * Command #6, resupply from HQ
 *********************************************************************************/
fn resupply(entities: &EntityColl, pi: &mut PlayerInfo) -> bool {
    let mut turn_over = false;
    if pi.damage[SubSystem::Resupply] < 0. {
        println!("Loading hatch is damaged.  Unable to resupply, {}.", pi.name);
    } else if pi.resupply_left == 0 {
        println!("Headquarters is abandoned.");
    } else {
        let ppos = get_first_pos(entities, EType::Player).unwrap();
        if let Some(hqpos) = get_first_pos(entities, EType::HQ) {
            if ppos.distance(&hqpos) <= 2. && pi.depth < 51 {
                // Original code is unconditional, which could result in having
                // fewer supplies after resupplying.
                if pi.power < 4000 { pi.power = 4000; }
                if pi.torpedos < 8 { pi.torpedos = 8; }
                if pi.missiles < 2 { pi.missiles = 2; }
                if pi.fuel < 1500 { pi.fuel = 1500; }
                if pi.crew < 25 { pi.crew = 25; }
                println!("Divers from headquarters bring out supplies and men.");
                pi.resupply_left -= 1;
                turn_over = true;
            }
        }
        if !turn_over {
            println!("Unable to comply with docking orders {}.", pi.name);
        }
    }
    turn_over
}

/**********************************************************************************
 * Command #8, Convert power to fuel or fuel to power
 *********************************************************************************/
fn convert_power_or_fuel(pi: &mut PlayerInfo) -> bool {
    if pi.damage[SubSystem::Converter] < 0. {
        println!("Power Converter is off line, {}.", pi.name);
        return false;
    }
    if pi.crew <=5 {
        println!("Not enough men to work the converter, {}.", pi.name);
        return false;
    }

    loop {
        let input = prompt("Option?  (1=Fuel to Power, 2=Power to Fuel");
        if input == "1" {
            convert_fuel_to_power(pi);
            break;
        } else if input == "2" {
            convert_power_to_fuel(pi);
            break;
        }
    }
    println!("Conversion complete.  Power={}.  Fuel={}", pi.power, pi.fuel);
    true
}

fn convert_power_to_fuel(pi: &mut PlayerInfo) {
    let prompt_str = &format!("Power available: {}.  Convert", pi.power-1);
    loop {
        let input = prompt(prompt_str);
        if let Ok(power) = input.parse::<u32>() {
            if power < pi.power {
                pi.power -= power;
                pi.fuel += power * 3;
                break;
            }
        }
    }
}

fn convert_fuel_to_power(pi: &mut PlayerInfo) {
    let prompt_str = &format!("Fuel available: {}.  Convert", pi.fuel);
    loop {
        let input = prompt(prompt_str);
        if let Ok(fuel) = input.parse::<u32>() {
            if fuel <= pi.fuel {
                pi.fuel -= fuel;
                pi.power += fuel / 3;
                break;
            }
        }
    }
}

/**********************************************************************************
 * Command #9, surrender
 *********************************************************************************/
fn surrender(pi: &mut PlayerInfo) -> bool {
    println!("Coward!  You're not very patriotic, {}.", pi.name);
    pi.alive = false;
    true
}

/**********************************************************************************
 * Enemy movement
 *********************************************************************************/
fn move_enemies(entities: &mut EntityColl, pi: &mut PlayerInfo) {
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
            if move_enemy(e, &mut unmoved, &mut moved) {
                pi.alive = false;
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
    use EResolution::*;
    match e.etype {
        Ship => match crashee.etype {
            Island | Ship => {
                println!("Enemy ship changed direction to avoid {:?}",
                         crashee.etype);
                MoverChangeDirection
            },
            Player => {
                println!("You've been rammed by a ship!");
                CrasheeDestroyed
            },
            HQ => {
                println!("Your headquarters was rammed!");
                CrasheeDestroyed
            },
            Mine => {
                if rnd() < 0.7 {
                    println!("Enemy ship changed direction to avoid mine");
                    MoverChangeDirection
                }
                else {
                    println!("Enemy ship was destroyed by a mine!");
                    MoverDestroyed
                }
            },
            Monster => {
                println!("Enemy ship was eaten by a monster!");
                MoverDestroyed
            }
        }
        Monster => match crashee.etype {
            Island => {
                println!("Sea monster changed direction to avoid the island");
                MoverChangeDirection
            },
            Player => {
                println!("You've been eaten by a sea monster!");
                CrasheeDestroyed
            },
            HQ => {
                println!("A sea monster ate your headquarters!");
                CrasheeDestroyed
            },
            Ship => {
                println!("Ship eaten by a moving monster!");
                CrasheeDestroyed
            },
            Mine => {
                println!("{:?} destroyed by a mine!", e.etype);
                MoverDestroyed
            },
            Monster => {
                println!("A sea monster fight!!");
                if rnd() < 0.8 {
                    println!("It's a tie!");
                    MoverChangeDirection
                } else {
                    println!("And one dies!!");
                    MoverDestroyed
                }
            }
        }
        _ => panic!("Unexpected mover type {:?}", e.etype)
    }
}

/// Return true if the player was killed as a result of enemy movement
fn move_enemy(e: Entity, unmoved: &mut EntityColl, moved: &mut EntityColl) -> bool {
    let mut player_killed = false;
    // Calculate destination
    let Component::Velocity(dx, dy) = e.components[0];
    let x = e.pos.x.wrapping_add(dx as usize);
    let y = e.pos.y.wrapping_add(dy as usize);
    if x >= WIDTH || y >= HEIGHT {
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
                        // Special-case handling of player destruction
                        if crashee.etype == EType::Player {
                            player_killed = true;
                        }
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
    player_killed
}

/**********************************************************************************
 * Enemy attacks
 *********************************************************************************/
fn retaliation(entities: &EntityColl, pi: &mut PlayerInfo) {
    let mut threat = 0f32;
    let ppos = get_first_pos(entities, EType::Player).unwrap();
    for e in entities {
        if e.etype == EType::Ship {
            if let Some(dist) = ppos.distance_within(&e.pos, 4f32) {
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
            let done = match get_command(&player_info) {
                Navigate => navigate(&mut entities, &mut player_info),
                Sonar => sonar(&entities, &mut player_info),
                Torpedo => fire_torpedo(&mut entities, &mut player_info),
                Missile => fire_missile(&mut entities, &mut player_info),
                Manuever => manuever(&mut player_info),
                Status => status_report(&entities, &player_info),
                Resupply => resupply(&entities, &mut player_info),
                Convert => convert_power_or_fuel(&mut player_info),
                Surrender => surrender(&mut player_info),
            };
            if done {
                break;
            }
        }
        // Various commands use power.  Maybe too much.
        if player_info.alive && player_info.power == 0 {
            println!("Atomic pile has gone dead!! Sub sinks, crew suffocates.");
            player_info.alive = false;
            break;
        }
        // Player may have destroyed itself, surrendered, or won.
        if !player_info.alive {
            break;
        }
        retaliation(&entities, &mut player_info);
        if !player_info.alive {
            break;
        }
        move_enemies(&mut entities, &mut player_info);
        // Enemies might have run into player
        if !player_info.alive {
            break;
        }
        // Enemies might have run into mines
        if count_all_of(&entities, EType::Ship) == 0 {
            println!("You Won!  All hail {} the glorious!!", player_info.name);
            break;
        }
        repair(&mut player_info);
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
        if !player_info.alive {
            println!("... albeit, posthumously.");
        }
    }
}

use rand::{Rng, rngs::StdRng, SeedableRng};

use crate::thunder;
use crate::math::bernoullis_scheme;

pub fn combo(message_parts: Vec<&str>, get_message: &dyn Fn(&str, Vec<&str>) -> Option<String>) -> String {
    let mut message: String = String::new();

    for (i, command) in message_parts.iter().enumerate() {
        if command != &"!combo" {
            match get_message(command, message_parts[i..message_parts.len()].into()) {
                Some(val) => { message = format!("{} {}", message, val); }
                None => {}
            }
        }
    }

    message
}

pub fn nomic() -> String {
    "no mic.".to_owned()
}

pub fn rolltrident() -> String {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let n: i32 = rng.gen_range(0..=250);
    let durability: i32 = rng.gen_range(0..=n);

    format!("Your trident has {} durability.", durability)
}

pub fn age() -> String {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let age: i32 = rng.gen_range(0..=100);
    
    format!("Oskar is {} years old.", age)
}

pub fn rollseed() -> String {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let seed: i64 = rng.gen();
    
    format!("Your seed: {}.", seed)
}

pub fn findseed() -> String {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let v: Vec<i32> = vec!(0; 12);
    let rolls: Vec<i32> = v.iter().map(|n| (*n == rng.gen_range(0..10)) as i32).collect::<Vec<i32>>(); 
    let eyes: i32 = rolls.iter().sum::<i32>();
    
    format!("Your seed is a {} eye.", eyes)
}

pub fn weather() -> String {
    let (thunder_start, thunder_duration) = thunder::get_first_thunder();
    let formatted_start_time: String = thunder::format_start_time(thunder_start);
    let formatted_duration: String = thunder::format_duration(thunder_duration);
    
    format!("First thunder will start at {} and will last {}.", formatted_start_time, formatted_duration)
}

pub fn thunderodds(message_parts: Vec<&str>) -> String {
    if message_parts.len() <= 0 {   
        return "Error: Invalid syntax; !thunderodds {time in minutes}".to_owned();
    }

    let arg = message_parts[1].parse::<f64>();

    match arg {
        Ok(mins) => {
            let odds: f64 = thunder::get_thunder_odds((mins * 1200.0) as u64);
            return format!("Odds of thunder in first {} minutes: ~{:.4}%", mins, odds * 100.0);
        },
        Err(_) => {
            return "Error: Invalid syntax; !thunderodds {time in minutes}".to_owned();
        }
    }
}

pub fn skullrates(message_parts: Vec<&str>) -> String {
    if message_parts.len() <= 2 {
        return "Error: Invalid syntax; !skullodss {drops} {kills} {looting level}".to_owned();
    }
    
    let drops = message_parts[1].parse::<u128>();
    let kills = message_parts[2].parse::<u128>();
    let looting_level = message_parts[3].parse::<f64>();

    match (kills, drops, looting_level) {
        (Ok(kills), Ok(drops), Ok(looting_level)) => {
            let p: f64 = looting_level / 100.0 + 0.025; 
            if p < 0.0 || p > 1.0 || drops > kills || looting_level < 0.0 {
                return "Error: Invalid syntax; !skullodds {drops} {kills} {looting level}".to_owned();
            }

            // println!("{}",p);
            
            let mut probability: f64 = 0.0;
            for n in drops..=kills { 
                probability += bernoullis_scheme(kills, n, p);
            }

            return format!("Odds of getting {} or more skulls from {} wither skeletons with looting {}: ~{:.10}%", drops, kills, looting_level, probability * 100.0);
        },
        _ => {
            return "Error: Invalid syntax; !skullodds {drops} {kills} {looting level}".to_owned();
        }
    };
}
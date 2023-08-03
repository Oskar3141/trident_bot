use rand::{Rng, rngs::StdRng, SeedableRng};
use rspotify::model::{PlayableItem, AdditionalType};
use rspotify::{prelude::*, AuthCodeSpotify};
use sqlite::{Connection, State};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::thunder;
use crate::math::bernoullis_scheme;

pub fn nomic() -> Result<String, String> {
    Ok("No Microphone.".to_owned())
}

pub fn rolltrident(sqlite_connection: &Connection, user_id: &str) -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let n: i32 = rng.gen_range(0..=250);
    let durability: i32 = rng.gen_range(0..=n);

    // add data to the database
    let unix_time: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let trident_query: &str = &format!("INSERT INTO trident_rolls (durability, unix_time, user_id) VALUES ({}, {}, {});", durability, unix_time, user_id);
    let query_result = sqlite_connection.execute(trident_query);

    if let Err(err) = query_result {
        println!("Trident durability database error: {}", err);
    }

    Ok(format!("Your trident has {} durability.", durability))
}

pub fn age() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let age: i32 = rng.gen_range(0..=100);
    
    Ok(format!("Oskar is {} years old.", age))
}

pub fn rollseed() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let seed: i64 = rng.gen();
    
    Ok(format!("Your seed: {}.", seed))
}

pub fn findseed() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let v: Vec<i32> = vec!(0; 12);
    let rolls: Vec<i32> = v.iter().map(|n| (*n == rng.gen_range(0..10)) as i32).collect::<Vec<i32>>(); 
    let eyes: i32 = rolls.iter().sum::<i32>();
    
    Ok(format!("Your seed is a {} eye.", eyes))
}

pub fn weather() -> Result<String, String> {
    let (thunder_start, thunder_duration) = thunder::get_first_thunder();
    let formatted_start_time: String = thunder::format_start_time(thunder_start);
    let formatted_duration: String = thunder::format_duration(thunder_duration);
    
    Ok(format!("First thunder will start at {} and will last {}.", formatted_start_time, formatted_duration))
}

pub fn thunderodds(message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !thunderodds {time in minutes}".to_owned());

    if message_parts.len() <= 0 {   
        return error;
    }

    let arg = message_parts[1].parse::<f64>();

    match arg {
        Ok(mins) => {
            let odds: f64 = thunder::get_thunder_odds((mins * 1200.0) as u64);
            return Ok(format!("Odds of thunder in first {} minutes: ~{:.4}%", mins, odds * 100.0).replace(".", ","));
        },
        Err(_) => {
            return error;
        }
    }
}

pub fn skullodds(message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !skullodds {drops} {kills} {looting level}".to_owned());
    
    if message_parts.len() <= 2 {
        return error;
    }
    
    let drops = message_parts[1].parse::<u128>();
    let kills = message_parts[2].parse::<u128>();
    let looting_level = message_parts[3].parse::<u32>();

    match (kills, drops, looting_level) {
        (Ok(kills), Ok(drops), Ok(looting_level)) => {
            let p: f64 = (looting_level as f64) / 100.0 + 0.025; 
            if p < 0.0 || p > 1.0 || drops > kills || looting_level > 3 {
                return error;
            }

            // println!("{}",p);
            
            let mut exact_or_more_drops_probability: f64 = 0.0;
            for n in drops..=kills { 
                exact_or_more_drops_probability += bernoullis_scheme(kills, n, p);
            }

            let exact_drops_probability: f64 = bernoullis_scheme(kills, drops, p);

            return Ok(format!(
                "Wither skeleton kills: {}; Looting level: {}; Odds of getting exactly {} skull drops: ~{:.8}%; Odds of getting {} or more skull drops: ~{:.8}%",
                kills,
                looting_level,
                drops,
                exact_drops_probability * 100.0,
                drops,
                exact_or_more_drops_probability * 100.0
            ).replace(".", ","));
        },
        _ => {
            return error;
        }
    };
}

pub fn tridentodds(message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !tridentodds {durability}".to_owned());
    
    if message_parts.len() <= 1 {
        return error;
    }

    let durability = message_parts[1].parse::<u32>();

    match durability {
        Ok(durability) => {
            if durability > 250 {
                return error;
            }   

            let mut exact_durability_odds: f64 = 0.0;
            let mut exact_or_more_durability_odds: f64 = 0.0;

            for k in durability..=250 {
                for n in k..=250 {
                    if k == durability {
                        exact_durability_odds += 1.0 / (251.0 * (n + 1) as f64);
                    }

                    exact_or_more_durability_odds += 1.0 / (251.0 * (n + 1) as f64);
                }
            }
            
            let message: String = if durability == 250 {
                format!(
                    "Odds of getting {} durability trident: {:.8}%.", 
                    durability, 
                    exact_durability_odds * 100.0, 
                )
            } else {
                format!(
                    "Odds of getting exactly {} durability trident: ~{:.8}%; Odds of getting {} or more durability trident: ~{:.8}%", 
                    durability, 
                    exact_durability_odds * 100.0, 
                    durability, 
                    exact_or_more_durability_odds * 100.0
                ).replace(".", ",")
            };

                
            return Ok(message);
        },
        Err(_) => {
            return error;
        },
    }
}

pub fn rolldrowned(message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !rolldrowned {drowned} {looting level}".to_owned());

    if message_parts.len() <= 2 {
        return error;
    }

    let mut rng: StdRng = SeedableRng::from_entropy();

    let kills = message_parts[1].parse::<u32>();
    let looting_level = message_parts[2].parse::<u32>();

    match (kills, looting_level) {
        (Ok(kills), Ok(looting_level)) => {
            if looting_level > 3 {
                return error;
            }

            let mut rotten_flesh: u32 = 0;
            let mut tridents: u32 = 0;
            let mut shells: u32 = 0;
            let mut fishing_rods: u32 = 0;
            let mut copper_ingots: u32 = 0;

            for _ in 0..kills {
                rotten_flesh += rng.gen_range(0..(2 + looting_level));
                copper_ingots += if rng.gen_range(1..=100) <= (11 + looting_level) {
                    1
                } else {
                    0
                };

                if rng.gen_range(1..=1000) <= 85 + looting_level * 10  {
                    if rng.gen_range(1..=10) == 10 {
                        if rng.gen_range(0..16) < 10 {
                            tridents += 1;
                        } else {
                            fishing_rods += 1;
                        }
                    }
                }
                    
                if rng.gen_range(1..=100) <= 3 {
                    shells += 1;
                }
            }

            return Ok(format!(
                "You got {} Rotten Flesh, {} Copper Ingots, {} Nautilus Shells, {} Tridents, {} Fishing Rods from killing {} drowned with looting {}.",
                rotten_flesh,
                copper_ingots,
                shells,
                tridents,
                fishing_rods,
                kills,
                looting_level
            ));
        },
        _ => {
            return error;
        }
    }
}

pub fn fishinge() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let n: u32 = rng.gen_range(1..=20);

    let message: String = "You caught ".to_owned() + if n <= 17 {
        let k: u32 = rng.gen_range(1..=100);

        if k <= 60 {
            "a Raw Cod! 🐟"
        } else if k > 60 && k <= 85 {
            "a Raw Salmon! 🐟"
        } else if k > 85 && k <= 87 {
            "a Tropical Fish! 🐠"
        } else {
            "a Pufferfish! 🐡"
        }
    } else if n == 18 {
        match rng.gen_range(1..=6) {
            1 => {
                "an Enchanted Bow! 🏹"
            },
            2 => {
                "an Enchanted Book! 📖"
            },
            3 => {
                "an Enchanted Fishing Rod! 🎣"
            },
            4 => {
                "a Name Tag! 📛"
            },
            5 => {
                "a Nautilus Shell! 🐚"
            },
            6 => {
                "a Saddle! 🐎"
            },
            _ => {
                "you should never get this."
            }
        }
    } else {
        let k: u32 = rng.gen_range(1..=100);

        if k <= 17 {
            "a Lily Pad! 🪷"
        } else if k > 17 && k <= 27 {
            "a Bowl! 🥣"
        } else if k > 27 && k <= 29 {
            "a Fishing Rod! 🎣"
        } else if k > 29 && k <= 39 {
            "Leather! 💼"
        } else if k > 39 && k <= 49 {
            "a Leather Boots! 👢"
        } else if k > 49 && k <= 59 {
            "a Rotten Flesh! 🥩"
        } else if k > 59 && k <= 64 {
            "a Stick! 🏑"
        } else if k > 64 && k <= 69 {
            "a String! 🪀"
        } else if k > 69 && k <= 79 {
            "a Water Bottle! 💦"
        } else if k > 79 && k <= 89 {
            "a Bone! 🦴"
        } else if k == 90  {
            "10 Ink Sac! 🪶"
        } else  {
            "a Tripwire Hook! 🪝"
        }
    };

    Ok(message)
} 

pub async fn song(spotify: AuthCodeSpotify) -> Result<String, String> {
    let song_response = spotify.current_playing(None, Some([&AdditionalType::Track])).await;
    let mut message = String::new();

    match song_response {
        Ok(playing) => {
            match playing {
                Some(playing) => {
                    match playing.item {
                        Some(plyable_item) => {
                            match plyable_item {
                                PlayableItem::Track(track) => {
                                    let artists = track.artists;
                        
                                    for (i, artist) in artists.iter().enumerate() {
                                        if i != artists.len() - 1 {
                                            message += &format!("{}, ", artist.name);
                                        } else {
                                            message += &format!("{} - ", artist.name);
                                        }
                                    }
                        
                                    message += &track.name;
                                },
                                _ => { }
                            }
                        },
                        None => {
                            return Err("Error: No song is currently playing.".to_owned());
                        }
                    }
                },
                None => {
                    return Err("Error: No song is currently playing.".to_owned());
                }
            }
        },
        Err(err) => {
            println!("Error when getting the song: {:?}", err);
            return Err("Error: Couldn't get the current song.".to_owned());
        }
    }

    Ok(message)
}

pub fn wr() -> Result<String, String> {
    Ok("1.16 AASSG: 1:46 by Oxidiot.".to_owned())
}

pub fn pb() -> Result<String, String> {
    Ok("AA RSG: 1.12: 4:38; 1.16: No completed run, 3:58 thunderless; AA SSG: 1.16: No pb.".to_owned())
}

pub fn topcommands(sqlite_connection: &Connection) -> Result<String, String> {
    let query = "SELECT name, SUM(uses) as total_uses FROM commands GROUP BY name ORDER BY total_uses DESC LIMIT 3;";
    let statement = sqlite_connection.prepare(query);
    let mut message: String = "Top 3 most used commands: ".to_owned();

    match statement {
        Ok(mut statement) => while let Ok(State::Row) = statement.next() {
            let command_name = statement.read::<String, _>("name").unwrap();
            let command_uses = statement.read::<i64, _>("total_uses").unwrap();
        
            message += &format!("{}: {} uses; ", command_name.replace("emark_", "!"), command_uses);
        },
        Err(error) => {
            println!("Top commands error: {}", error);
            return Err(format!("Error: {}", error));
        }
    }
    
    Ok(message)
}

pub fn topchatters(sqlite_connection: &Connection) -> Result<String, String> {
    let query = "SELECT display_name, messages FROM users ORDER BY messages DESC LIMIT 3;";
    let statement = sqlite_connection.prepare(query);
    let mut message: String = "Top 3 chatters: ".to_owned();

    match statement {
        Ok(mut statement) => while let Ok(State::Row) = statement.next() {
            let name = statement.read::<String, _>("display_name").unwrap();
            let messages = statement.read::<i64, _>("messages").unwrap();
        
            message += &format!("{}: {} messages; ", name, messages);
        },
        Err(error) => {
            println!("Top chatters error: {}", error);
            return Err(format!("Error: {}", error));
        }
    }
    
    Ok(message)
}

pub fn topspammers(sqlite_connection: &Connection) -> Result<String, String> {
    let query = "SELECT users.display_name as username, SUM(uses) AS total_uses FROM commands INNER JOIN users on commands.user_id = users.user_id GROUP BY commands.user_id ORDER BY total_uses DESC LIMIT 3;";
    let statement = sqlite_connection.prepare(query);
    let mut message: String = "Top 3 command spammers: ".to_owned();

    match statement {
        Ok(mut statement) => while let Ok(State::Row) = statement.next() {
            let user = statement.read::<String, _>("username").unwrap();
            let command_uses = statement.read::<i64, _>("total_uses").unwrap();
        
            message += &format!("{}: {} command uses; ", user, command_uses);
        },
        Err(error) => {
            println!("Top commands error: {}", error);
            return Err(format!("Error: {}", error));
        }
    }
    
    Ok(message)
}

pub fn rollgp(sqlite_connection: &Connection, user_id: &str) -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();
    let mut gunpowder: u32 = 0;

    for _ in 1..=16 {
        if rng.gen_range(1..=50) <= 10 {
            gunpowder += rng.gen_range(1..=8);
        }
    }

    // add data to the database
    let unix_time: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let gunpowder_query: &str = &format!("INSERT INTO gunpowder_rolls (gunpowder, unix_time, user_id) VALUES ({}, {}, {});", gunpowder, unix_time, user_id);
    let query_result = sqlite_connection.execute(gunpowder_query);

    if let Err(err) = query_result {
        println!("Gunpowder ammount database error: {}", err);
    }

    Ok(format!("You got {} gunpowder!", gunpowder))
}

pub fn rollbiome() -> Result<String, String> {
    // This is a shit way to do this but i'm too lazy to do it in a better way.
    let biomes = HashMap::from([
        ("Forest", 38060816951),
        ("Plains", 39985547347),
        ("River", 18041713211),
        ("Ocean", 33794996808),
        ("Mountains", 29898531283),
        ("Deep Ocean", 32573867472),
        ("Swamp", 16124718827),
        ("Desert", 20897575614),
        ("Taiga", 18633313544),
        ("Wooded Hills", 10223682968),
        ("Lukewarm Ocean", 15745952870),
        ("Cold Ocean", 14917861632),
        ("Beach", 14199898280),
        ("Dark Forest", 11692097469),
        ("Birch Forest", 11661838126),
        ("Savanna", 12964991674),
        ("Wooded Mountains", 7014132060),
        ("Deep Cold Ocean", 10577186376),
        ("Deep Lukewarm Ocean", 11617367856),
        ("Desert Hills", 5810895353),
        ("Taiga Hills", 4128834830),
        ("Birch Forest Hills", 3339030439),
        ("Warm Ocean", 5940180755),
        ("Jungle", 4460886481),
        ("Savanna Plateau", 3282841807),
        ("Snowy Tundra", 8841400745),
        ("Giant Tree Taiga", 2996994360),
        ("Sunflower Plains", 2616855572),
        ("Deep Frozen Ocean", 3627404262),
        ("Flower Forest", 2055260714),
        ("Frozen Ocean", 2631260823),
        ("Jungle Hills", 1550080250),
        ("Snowy Mountains", 2923917321),
        ("Stone Shore", 1945888784),
        ("Giant Tree Taiga Hills", 1342424498),
        ("Badlands", 1815318568),
        ("Snowy Taiga", 2682541180),
        ("Gravelly Mountains", 1353924708),
        ("Wooded Badlands Plateau", 1256286198),
        ("Modified Gravelly Mountains", 884083327),
        ("Desert Lakes", 926615257),
        ("Jungle Edge", 407387345),
        ("Taiga Mountains", 809720951),
        ("Badlands Plateau", 547928247),
        ("Dark Forest Hills", 560542003),
        ("Tall Birch Forest", 551946119),
        ("Swamp Hills", 540238804),
        ("Snowy Taiga Hills", 614618565),
        ("Shattered Savanna", 588903346),
        ("Tall Birch Hills", 415490146),
        ("Bamboo Jungle", 407153563),
        ("Shattered Savanna Plateau", 409085452),
        ("Frozen River", 509199135),
        ("Snowy Beach", 795354682),
        ("Bamboo Jungle Hills", 132055350),
        ("Modified Jungle", 210736272),
        ("Ice Spikes", 413334512),
        ("Giant Spruce Taiga Hills", 160486493),
        ("Giant Spruce Taiga", 159615088),
        ("Mushroom Fields", 135122523),
        ("Eroded Badlands", 101993043),
        ("Mushroom Field Shore", 91394091),
        ("Modified Wooded Badlands Plateau", 60069217),
        ("Snowy Taiga Mountains", 118770189),
        ("Modified Badlands Plateau", 26848829),
        ("Modified Jungle Edge", 1760744)
    ]);
    
    let mut rng: StdRng = SeedableRng::from_entropy();
    let n: u64 = rng.gen_range(1..=443_808_771_309);
    let mut index: u64 = 0;

    for (name, value) in biomes {
        if n > index && n <= index + value {
            return Ok(format!("You got {}!", name));
        } else {
            index += value;
        }
    }

    Err("Error: Couldn't find a biome.".to_owned())
}

pub fn commands() -> Result<String, String> {
    Ok("Commands file: https://github.com/Oskar-Dev/trident_bot/blob/master/src/commands.rs - good luck with figuring out how all this works.".to_owned())
}

pub fn rollcats(message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !rollcats {cats number}".to_owned());

    if message_parts.len() <= 1 {
        return error;
    }

    let mut rng: StdRng = SeedableRng::from_entropy();
    let cats = message_parts[1].parse::<u32>();

    let mut jellie: u32 = 0;
    let mut calico: u32 = 0;
    let mut red: u32 = 0;
    let mut tuxedo: u32 = 0;
    let mut white: u32 = 0;
    let mut ragdoll: u32 = 0;
    let mut british: u32 = 0;
    let mut tabby: u32 = 0;
    let mut persian: u32 = 0;
    let mut siamese: u32 = 0;

    match cats {
        Ok(rolls) => {
            for _ in 0..rolls {
                match rng.gen_range(1..=10) {
                    1 => { jellie += 1; },
                    2 => { calico += 1; },
                    3 => { red += 1; },
                    4 => { tuxedo += 1; },
                    5 => { white += 1; },
                    6 => { ragdoll += 1; },
                    7 => { british += 1; },
                    8 => { tabby += 1; },
                    9 => { persian += 1; },
                    10 => { siamese += 1; },
                    _ => {}
                };
            };
        }
        Err(_) => {
            return error;
        }
    }
        
    Ok(format!("You got {} Jellie, {} Calico, {} Red, {} Tuxedo, {} White, {} Ragdoll, {} British, {} Tabby, {} Persian, {} Siamese.", 
    jellie, calico, red, tuxedo, white, ragdoll, british, tabby, persian, siamese))
}

pub fn rollblazerods(message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !rollblazerods {rods} {looting level}".to_owned());

    if message_parts.len() <= 2 {
        return error;
    }

    let mut rng: StdRng = SeedableRng::from_entropy();
    let rods_number = message_parts[1].parse::<u32>();
    let looting_level = message_parts[2].parse::<u32>();

    let mut rods: u32 = 0;
    let mut kills: u32 = 0;

    match (rods_number, looting_level) {
        (Ok(rods_number), Ok(looting_level)) => {
            if looting_level > 3 {
                return error;
            }

            while rods < rods_number {
                rods += rng.gen_range(0..=(1+looting_level));
                kills += 1;
            }

            return Ok(format!("You got {} blaze rods from killing {} blazes with looting {}.", rods_number, kills, looting_level))
        },
        _ => {
            return error;
        }
    }
}

pub fn tridentjuicers(sqlite_connection: &Connection) -> Result<String, String> {
    let query = "SELECT users.display_name as username, durability FROM trident_rolls INNER JOIN users on trident_rolls.user_id = users.user_id ORDER BY durability DESC LIMIT 3;";
    let statement = sqlite_connection.prepare(query);
    let mut message: String = "Top 3 best trident rolls: ".to_owned();

    match statement {
        Ok(mut statement) => while let Ok(State::Row) = statement.next() {
            let user = statement.read::<String, _>("username").unwrap();
            let durability = statement.read::<i64, _>("durability").unwrap();
        
            message += &format!("{} - {}; ", user, durability);
        },
        Err(error) => {
            println!("Trident juicers error: {}", error);
            return Err(format!("Error: {}", error));
        }
    }
    
    Ok(message)
}

pub fn gpjuicers(sqlite_connection: &Connection) -> Result<String, String> {
    let query = "SELECT users.display_name as username, gunpowder FROM gunpowder_rolls INNER JOIN users on gunpowder_rolls.user_id = users.user_id ORDER BY gunpowder DESC LIMIT 3;";
    let statement = sqlite_connection.prepare(query);
    let mut message: String = "Top 3 best desert temple gunpowder rolls: ".to_owned();

    match statement {
        Ok(mut statement) => while let Ok(State::Row) = statement.next() {
            let user = statement.read::<String, _>("username").unwrap();
            let durability = statement.read::<i64, _>("gunpowder").unwrap();
        
            message += &format!("{} - {}; ", user, durability);
        },
        Err(error) => {
            println!("Gunpowder juicers error: {}", error);
            return Err(format!("Error: {}", error));
        }
    }
    
    Ok(message)
}

pub fn dailytridentjuicers(sqlite_connection: &Connection) -> Result<String, String> {
    let one_day_ms: u128 = 86_400_000;
    let unix_time: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let query = &format!("SELECT users.display_name as username, durability FROM trident_rolls INNER JOIN users on trident_rolls.user_id = users.user_id WHERE unix_time > {} ORDER BY durability DESC LIMIT 3;", unix_time - one_day_ms);
    let statement = sqlite_connection.prepare(query);
    let mut message: String = "Top 3 best trident rolls in last 24 hours: ".to_owned();

    match statement {
        Ok(mut statement) => while let Ok(State::Row) = statement.next() {
            let user = statement.read::<String, _>("username").unwrap();
            let durability = statement.read::<i64, _>("durability").unwrap();
        
            message += &format!("{} - {}; ", user, durability);
        },
        Err(error) => {
            println!("Trident juicers error: {}", error);
            return Err(format!("Error: {}", error));
        }
    }
    
    Ok(message)
}

pub fn tridentnoobs(sqlite_connection: &Connection) -> Result<String, String> {
    let query = "SELECT users.display_name as username, COUNT(durability) as zeros FROM trident_rolls INNER JOIN users on trident_rolls.user_id = users.user_id WHERE durability = 0 GROUP BY username ORDER BY zeros DESC LIMIT 3;";

    let statement = sqlite_connection.prepare(query);
    let mut message: String = "Top 3 chatters with most 0 durability trident rolls: ".to_owned();

    match statement {
        Ok(mut statement) => while let Ok(State::Row) = statement.next() {
            let user = statement.read::<String, _>("username").unwrap();
            let zeros = statement.read::<i64, _>("zeros").unwrap();
        
            message += &format!("{} - {}; ", user, zeros);
        },
        Err(error) => {
            println!("Trident noobs error: {}", error);
            return Err(format!("Error: {}", error));
        }
    }
    
    Ok(message)
}
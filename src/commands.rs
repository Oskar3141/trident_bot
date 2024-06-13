use rand::distributions::{Uniform, Distribution};
use rand::{Rng, rngs::StdRng, SeedableRng};
use rspotify::model::{PlayableItem, AdditionalType};
use rspotify::{prelude::*, AuthCodeSpotify};
use sqlite::{Connection, State};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
// use rand_xoshiro::rand_core::;
use rand_xoshiro::Xoroshiro128PlusPlus;

use crate::thunder::{self, format_start_time};
use crate::math::bernoullis_scheme;
use crate::phantoms::get_phantoms_spawn_time;

pub fn nomic() -> Result<String, String> {
    Ok("No Microphone.".to_owned())
}

pub fn rolltrident(sqlite_connection: &Connection, user_id: &str) -> Result<(String, u32), String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let n: u32 = rng.gen_range(0..=250);
    let durability: u32 = rng.gen_range(0..=n);

    // add data to the database
    let unix_time: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let trident_query: &str = &format!("INSERT INTO trident_rolls (durability, unix_time, user_id) VALUES ({}, {}, {});", durability, unix_time, user_id);
    let query_result = sqlite_connection.execute(trident_query);

    if let Err(err) = query_result {
        println!("Trident durability database error: {}", err);
    }

    if durability == 0 || durability == 1 {
        // let duration: u32 = if durability == 0 { 300 } else { 600 }; 

        // twitch_api::ban(user_id, "Your trident roll sucks.", duration);
        return Ok((format!("Your trident has {} durability LULW !", durability), durability))
    } else {
        return Ok((format!("Your trident has {} durability.", durability), durability))
    }
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

    let mut rng = Xoroshiro128PlusPlus::from_entropy();
    // rng::x

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

            let rotten_flesh_range = Uniform::from(0..(2 + looting_level));
            let one_to_hundred_range = Uniform::from(1..=100);
            let drowned_first_roll_range = Uniform::from(1..=10000);
            let drowned_third_roll_range = Uniform::from(0..16);

            for _ in 0..kills {
                rotten_flesh += rotten_flesh_range.sample(&mut rng);
                copper_ingots += if one_to_hundred_range.sample(&mut rng) <= (11 + looting_level) {
                    1
                } else {
                    0
                };

                if drowned_first_roll_range.sample(&mut rng) <= 85 + looting_level * 10  {
                    if drowned_third_roll_range.sample(&mut rng) < 10 {
                        tridents += 1;
                    } else {
                        fishing_rods += 1;
                    }
                }
                    
                if one_to_hundred_range.sample(&mut rng) <= 3 {
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
            "Leather Boots! 👢"
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
    // Ok("AARSG: 1.16: I don't care; | 1.17: 6:52:10 by Fudge; | 1.18: 4:22:12 by Leonn; | 1.19: 5:10:06 by Leonn; | 1.20: 6:41:18 by Leonn; | 1.20.5: 5:37 by Feinberg;$AASSG: 1.0-1.6: 24:06 by Schnidi_; | 1.8-1.11: 1:41:09 by Unease; | 1.12: 4:05:07 by MeisterMaki; | 1.13: N/A; | 1.14: N/A; | 1.15: N/A; | 1.16: 1:30:15 by me; (1:22:06 Thunderless by me); | 1.17: 4:06:49 by me; | 1.18: N/A; | 1.19: N/A; | 1.20: 3:58 by me;".to_owned())
    Ok("https://docs.google.com/spreadsheets/u/0/d/107ijqjELTQQ29KW4phUmtvYFTX9-pfHsjb18TKoWACk/htmlview#".to_owned())
}

pub fn pb() -> Result<String, String> {
    Ok("AARSG: 1.12: 4:38 | 1.16: No pb (3:58 thunderless); | 1.20.5: 8:14 | AASSG: 1.16: 1:30:15 (1:22:06 thunderless); | 1.17: 4:06:49 | 1.20: 3:58;".to_owned())
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

pub fn rollskulls(message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !rollskulls {skulls} {looting level}".to_owned());

    if message_parts.len() <= 2 {
        return error;
    }

    let mut rng: StdRng = SeedableRng::from_entropy();
    let skulls_number = message_parts[1].parse::<u32>();
    let looting_level = message_parts[2].parse::<u32>();

    let mut skulls: u32 = 0;
    let mut kills: u32 = 0;

    match (skulls_number, looting_level) {
        (Ok(skulls_number), Ok(looting_level)) => {
            if looting_level > 3 {
                return error;
            }

            while skulls < skulls_number {
                skulls += if rng.gen_range(1..=1000) <= 25 + looting_level * 10 { 1 } else { 0 };
                kills += 1;
            }

            return Ok(format!("You got {} skulls from killing {} wither skeletons with looting {}.", skulls_number, kills, looting_level))
        },
        _ => {
            return error;
        }
    }
}

pub fn commandstats(sqlite_connection: &Connection, message_parts: Vec<&str>) -> Result<String, String> {
    let error: Result<String, String> = Err("Error: Invalid syntax; !commandstats {command name}".to_owned());

    if message_parts.len() <= 1 {
        return error;
    }

    let command_name = message_parts[1].parse::<String>();

    match command_name {
        Ok(command_name) => {
            let db_command_name: &str = &command_name.replace("!", "emark_");
            let total_uses_query = &format!("SELECT name, SUM(uses) AS total_uses FROM commands WHERE name = '{}';", db_command_name);
            let top_users_query = &format!("SELECT name, SUM(uses) AS uses, users.display_name as username from commands INNER JOIN users on commands.user_id = users.user_id WHERE name = '{}' GROUP BY username ORDER BY uses DESC LIMIT 3;", db_command_name);

            let total_uses_statement = sqlite_connection.prepare(total_uses_query);
            let top_users_statement = sqlite_connection.prepare(top_users_query);
            let mut message: String = format!("Top 3 users with most {} uses: ", command_name);

            match top_users_statement {
                Ok(mut top_users_statement) => while let Ok(State::Row) = top_users_statement.next() {
                    let user = top_users_statement.read::<String, _>("username").unwrap();
                    let uses = top_users_statement.read::<i64, _>("uses").unwrap();
                    
                    message += &format!("{}: {} uses; ", user, uses);
                },
                Err(error) => {
                    println!("Command stats error: {}", error);
                    return Err(format!("Error: {}", error));
                }
            }

            match total_uses_statement {
                Ok(mut total_uses_statement) => if let Ok(State::Row) = total_uses_statement.next() {
                    let total_uses = total_uses_statement.read::<String, _>("total_uses").unwrap();

                    message += &format!("Total uses: {}.", total_uses);
                },
                Err(error) => {
                    println!("Command stats error: {}", error);
                    return Err(format!("Error: {}", error));
                }
            }

            return Ok(message) 
        },
        _ => {
            return error
        }
    }
}

pub fn raid(file_path: &str) -> Result<String, String> {
    let error: String = "Error: Couldn't get the raids.".to_owned();

    match File::open(file_path) {
        Err(err) => {
            println!("{}, {}", error, err);
            return Err(error);
        },
        Ok(mut file) => {
            let mut value: String = String::new();
            match file.read_to_string(&mut value) {
                Err(err) => {
                    println!("{}, {}", error, err);
                    return Err(error);
                },
                Ok(_) => {
                    return Ok(value);
                }
            }
        }
    }
}

pub fn rollphantoms() -> Result<String, String> {
    let (time, spawns) = get_phantoms_spawn_time(); 
    let formatted_time: String = format_start_time(time);


    Ok(format!("You got {} phantoms spawn at {}!", spawns, formatted_time))
}

pub fn rollaassg() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    // Temple
    if rng.gen_range(1..=100) <= 10 {
        match rng.gen_range(1..=100) {
            1..=10 => {
                return Ok("Your any% Temple blew up.".to_owned());
            },
            11..=30 => {
                return Ok("You died to a Creeper in your any% Temple.".to_owned());
            },
            31..=65 => {
                return Ok("You died to a Zombie in your any% Temple.".to_owned());
            },
            66..=95 => {
                return Ok("You died to a Skeleton in your any% Temple.".to_owned());
            },
            96..=100 => {
                return Ok("You died to a Witch in your any% Temple.".to_owned());

            },
            _ => {
                return Ok("If you see this, something broke.".to_owned());
            }
        }
    }

    // random events
    if rng.gen_range(1..=100) <= 3 {
        match rng.gen_range(1..=100) {
            1..=10 => {
                return Ok("You mispalced obby when building your first nether portal.".to_owned());
            },
            11..=55 => {
                return Ok("Your run died to slow pre-Nether any% crafting.".to_owned());
            },
            56..=100 => {
                return Ok("You couldn't find your wood after blowing up trees in any%.".to_owned());
            },
            _ => {
                return Ok("If you see this, something broke.".to_owned());
            }
        }
    }

    // nether
    if rng.gen_range(1..=100) == 1 {
        match rng.gen_range(1..=100) {
            1..=10 => {
                return Ok("You mispalced obby when building your first nether portal.".to_owned());
            },
            11..=100 => {
                return Ok("You accidentally hit one of the Zombiefied Piglins in pre-Bastion any%, so they killed you.".to_owned());
            },
            _ => {
                return Ok("If you see this, something broke.".to_owned());
            }
        }
    }

    // bastion
    if rng.gen_range(1..=100) <= 75 {
        match rng.gen_range(1..=100) {
            1..=5 => {
                return Ok("You died to Blazes during any% Nether, beacuse you didn't get Fire Resistance from barters.".to_owned());
            },
            6..=60 => {
                return Ok("You didn't get enough Ender Pearls from barters in any% Nether.".to_owned());
            },
            61..=100 => {
                return Ok("You didn't get enough materials for Explosives from barters in any% Nether".to_owned());
            },
            _ => {
                return Ok("If you see this, something broke.".to_owned());
            }
        }
    }

    Ok("Yhh uhh i didn't make this yet.".to_owned())
}

pub fn route() -> Result<String, String> {
    Ok("1.20: https://docs.google.com/document/d/1K2axBuCsNOdQ9vA7AYUaxhqgX5zXmWBWN-rBELLjxJM/edit".to_owned())
}

pub fn rollsilence() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();
    let mut rolls: i32 = 0;
    
    loop {
        rolls += 1;
        if rng.gen_range(1..=1000) <= 12 {
            break;
        }
    }

    Ok(format!("You needed to check only {} chests to get the Silence Trim!", rolls).to_owned())
}

pub fn hdwghfix() -> Result<String, String> {
    Ok("Mojang added six new effects in the 24w13a snapshot, but they are currently not required for HDWGH, so i made a Data Pack that adds these new effects to HDWGH. Link: https://github.com/Oskar-Dev/24w13a_hdwgh_fix".to_owned())
}

pub fn caamel() -> Result<String, String> {
    Ok("chilling cAAmel - Cross-Platform (In the future), high performance AA Tracker made by me in C with SDL2! You can check out my bad code here: https://github.com/Oskar-Dev/kAAmel chilling".to_owned())
}

pub fn rollheavycore() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();
    let mut rolls: i32 = 0;
    
    loop {
        rolls += 1;
        if rng.gen_range(1..=1000) <= 75 {
            break;
        }
    }

    let only: &str = if rng.gen_range(1..=2) == 1 { " only" } else { "" };

    Ok(format!("You needed to open{} {} Ominous Vaults to get the Heavy Core!", only, rolls).to_owned())
}
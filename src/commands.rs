use rand::{Rng, rngs::StdRng, SeedableRng};
use rspotify::model::{PlayableItem, AdditionalType};
use rspotify::{prelude::*, AuthCodeSpotify};

use crate::thunder;
use crate::math::bernoullis_scheme;

pub fn nomic() -> Result<String, String> {
    Ok("No Microphone.".to_owned())
}

pub fn rolltrident() -> Result<String, String> {
    let mut rng: StdRng = SeedableRng::from_entropy();

    let n: i32 = rng.gen_range(0..=250);
    let durability: i32 = rng.gen_range(0..=n);

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

pub fn skullrates(message_parts: Vec<&str>) -> Result<String, String> {
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
    let error: Result<String, String> = Err("Error: Invalid syntax; !skullodds {drops} {kills} {looting level}".to_owned());
    
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
    let error: Result<String, String> = Err("Error: Invalid syntax; !skullodds {drops} {kills} {looting level}".to_owned());

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
            "a Raw Cod!"
        } else if k > 60 && k <= 85 {
            "a Raw Salmon!"
        } else if k > 85 && k <= 87 {
            "a Tropical Fish!"
        } else {
            "a Pufferfish!"
        }
    } else if n == 18 {
        match rng.gen_range(1..=6) {
            1 => {
                "an Enchanted Bow!"
            },
            2 => {
                "an Enchanted Book!"
            },
            3 => {
                "an Enchanted Fishing Rod!"
            },
            4 => {
                "a Name Tag!"
            },
            5 => {
                "a Nautilus Shell!"
            },
            6 => {
                "a Saddle!"
            },
            _ => {
                "you should never get this."
            }
        }
    } else {
        let k: u32 = rng.gen_range(1..=100);

        if k <= 17 {
            "a Lily Pad!"
        } else if k > 17 && k <= 27 {
            "a Bowl!"
        } else if k > 27 && k <= 29 {
            "a Fishing Rod!"
        } else if k > 29 && k <= 39 {
            "a Leather!"
        } else if k > 39 && k <= 49 {
            "an Leather Boots!"
        } else if k > 49 && k <= 59 {
            "an Rotten Flesh!"
        } else if k > 59 && k <= 64 {
            "a Stick!"
        } else if k > 64 && k <= 69 {
            "a String!"
        } else if k > 69 && k <= 79 {
            "a Water Bottle!"
        } else if k > 79 && k <= 89 {
            "a Bone"
        } else if k == 90  {
            "10 Ink Sac!"
        } else  {
            "a Tripwire Hook!"
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
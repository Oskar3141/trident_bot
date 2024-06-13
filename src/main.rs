use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::message::{ServerMessage, UserNoticeEvent};
use twitch_irc::{ClientConfig, SecureTCPTransport};
use twitch_data::{LOGIN, OAUTH_TOKEN, CHANNEL};
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth, Config};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use twitch_api::ban;

mod twitch_data;
mod commands;
mod thunder;
mod math;
mod phantoms;
mod twitch_api;

const RAID_FILE_PATH: &str = "./raid.txt";
const RAID_FILE_DEFAULT_VALUE: &str = "No raids.";
const MAX_MESSAGE_LENGTH: usize = 450;

fn split_message(message: String, slices: &mut Vec<String>) {
    let mut offset: usize = 0;

    while !message.is_char_boundary(MAX_MESSAGE_LENGTH - offset) {
        offset += 1;

        if offset >= MAX_MESSAGE_LENGTH {
            return;
        }
    }

    let (first, last) = message.split_at(MAX_MESSAGE_LENGTH - offset);
    slices.push(first.to_owned());
    
    if last.len() > MAX_MESSAGE_LENGTH {
        split_message(last.to_owned(), slices) 
    } else {
        slices.push(last.to_owned());
    }
}

fn check_raid_file() {
    match fs::metadata(RAID_FILE_PATH) {
        Err(_) => {
            match File::create(RAID_FILE_PATH) {
                Ok(mut file) => {
                    if let Err(err) = file.write(RAID_FILE_DEFAULT_VALUE.as_bytes()) {
                        println!("Couldn't write to raid file: {}", err);
                    }
                },
                Err(err) => {
                    println!("Couldn't create raid file: {}", err);
                }
            }
        },
        Ok(_) => {},
    }
}

async fn send_message(message: String, client: &TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>) {
    let message: String = message.trim().to_owned();
    let messages_split: Vec<&str> = message.split('$').collect();

    for message in messages_split {
        let mut messages: Vec<String> = Vec::new();
        if message.len() > MAX_MESSAGE_LENGTH {
            split_message(message.to_owned(), &mut messages);
        } else {
            messages.push(message.to_owned());
        }

        for message in messages {
            let result = client.say(CHANNEL.to_owned(), message).await;
            
            match result {
                Ok(_) => {},
                Err(err) => {
                    println!("Error when sending a response message: {:?}", err);
                }
            }
        }
    }
}

#[tokio::main]
pub async fn main() {
    check_raid_file();

    // default configuration is to join chat as anonymous.
    let config = ClientConfig::new_simple(
        StaticLoginCredentials::new(LOGIN.to_owned(), Some(OAUTH_TOKEN.to_owned()))
    );
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);
    let send_client = client.clone();

    // spotify
    let rspotify_config = Config {
        token_cached: true,
        token_refreshing: true,
        ..Default::default()
    };
    let creds = Credentials::from_env().unwrap();
    let scopes = scopes!("user-read-currently-playing");
    let oauth = OAuth::from_env(scopes).unwrap();
    let spotify = AuthCodeSpotify::with_config(creds, oauth, rspotify_config);
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();

    // sqlite
    let sqlite_connection = sqlite::open("chat_data.sqlite").unwrap();
    // spotify.add_item_to_queue("https://open.spotify.com/track/3ZEno9fORwMA1HPecdLi0R", None);

    let create_commands_table_query: &str = "CREATE TABLE IF NOT EXISTS commands (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, uses INTEGER, user_id INTEGER);";
    let create_users_table_query: &str = "CREATE TABLE IF NOT EXISTS users (user_id INTEGER PRIMARY KEY, display_name TEXT, messages INTEGER)";
    let create_trident_rolls_table_query: &str = "CREATE TABLE IF NOT EXISTS trident_rolls (id INTEGER PRIMARY KEY AUTOINCREMENT, durability INTEGER, unix_time INTEGER, user_id INTEGER)";
    let create_gunpowder_rolls_table_query: &str = "CREATE TABLE IF NOT EXISTS gunpowder_rolls (id INTEGER PRIMARY KEY AUTOINCREMENT, gunpowder INTEGER, unix_time INTEGER, user_id INTEGER)";

    sqlite_connection.execute(create_commands_table_query).unwrap();
    sqlite_connection.execute(create_users_table_query).unwrap();
    sqlite_connection.execute(create_trident_rolls_table_query).unwrap();
    sqlite_connection.execute(create_gunpowder_rolls_table_query).unwrap();
 
    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let join_handle = tokio::spawn(async move {
        while let Some(server_message) = incoming_messages.recv().await {
            // println!("{:#?}", server_message);
            match server_message {
                ServerMessage::UserNotice(notice) => {
                    match notice.event {
                        UserNoticeEvent::Raid { viewer_count: _, profile_image_url: _ } => { 
                            let user: String = notice.sender.name;
                            let error_message: String = "Error: Couldn't automatically update the !raid command.".to_owned();
                            let mut value: String = String::new();

                            match OpenOptions::new().write(true).read(true).open(RAID_FILE_PATH) {
                                Ok(mut file) => {
                                    match file.read_to_string(&mut value) {
                                        Ok(_) => {
                                            let raid_message: String;

                                            if value == RAID_FILE_DEFAULT_VALUE {
                                                raid_message = format!("{}. PagBounce", user);
                                            } else {
                                                let new_value = value.strip_suffix(". PagBounce");

                                                match new_value {
                                                    Some(v) => {
                                                        raid_message = format!("{}, {}. PagBounce", v, user)
                                                    },
                                                    None => {
                                                        raid_message = format!("{}, {}. PagBounce", value, user)
                                                    }
                                                }
                                            }

                                            match file.set_len(0) {
                                                Ok(_) => {
                                                    match file.write(raid_message.as_bytes()) {
                                                        Err(err) => {
                                                            println!("{} {}", error_message, err);
                                                            send_message(error_message, &send_client).await;
                                                        }
                                                        Ok(_) => {
                                                            let message: String = format!("Automatically updated the !raid command to: {}", raid_message);
                                                            send_message(message, &send_client).await;
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    println!("{} {}", error_message, err);
                                                    send_message(error_message, &send_client).await;
                                                }
                                            }
                                        },
                                        Err(err) => {
                                            println!("{} {}", error_message, err);
                                            send_message(error_message, &send_client).await;
                                        }
                                    }
                                },
                                Err(err) => {
                                    println!("{} {}", error_message, err);
                                    send_message(error_message, &send_client).await;
                                }
                            }
                         },
                         _ => {}
                    }
                },
                ServerMessage::Privmsg(msg) => {
                    let user_id = msg.sender.id;
                    let user_display_name = msg.sender.name;
                    let message_parts: Vec<&str> = msg.message_text.split(" ").collect();

                    // let mut banan: bool = false;
                    // let mut ban_duration: u32 = 0;
                    
                    let mut call_all_commands: bool = false;
                    let mut message: String = String::new();
                    for (i, command) in message_parts.iter().enumerate() {
                        let args: Vec<&str> = message_parts[i..message_parts.len()].into();

                        let result: Option<Result<String, String>> = if call_all_commands || i == 0 {
                            let cmd: &str = &command.to_lowercase();
                            match cmd {
                                "!combo" => {
                                    call_all_commands = true;
                                    None
                                },
                                "!nomic" => {
                                    Some(commands::nomic())
                                },
                                "!rolltrident" => {
                                    // let conn = sqlite_connection;
                                    if let Ok((resp, dur)) = commands::rolltrident(&sqlite_connection, &user_id) {
                                        if dur == 0 || dur == 1 {
                                            // ban_duration = if dur == 1 { 300 } else { 600 }; 
                                            // banan = true;
                                        }

                                        Some(Ok(resp))
                                    } else {
                                        Some(Err("Error message".to_owned()))
                                    }
                                },
                                "!age" => {
                                    Some(commands::age())
                                },
                                "!rollseed" => {
                                    Some(commands::rollseed())
                                },
                                "!findseed" => {
                                    Some(commands::findseed())
                                },
                                "!weather" => {
                                    Some(commands::weather())
                                },
                                "!thunderodds" => {
                                    Some(commands::thunderodds(args))
                                },
                                "!skullodds" => {
                                    Some(commands::skullodds(args))
                                },
                                "!tridentodds" => {
                                    Some(commands::tridentodds(args))
                                },
                                "!rolldrowned" => {
                                    Some(commands::rolldrowned(args))
                                },
                                "Fishinge" => {
                                    Some(commands::fishinge())
                                },
                                "!song" => {
                                    Some(commands::song(spotify.clone()).await)
                                },
                                "!wr" => {
                                    Some(commands::wr())
                                },
                                "!pb" => {
                                    Some(commands::pb())
                                },
                                "!topcommands" => {
                                    Some(commands::topcommands(&sqlite_connection))
                                },
                                "!topchatters" => {
                                    Some(commands::topchatters(&sqlite_connection))
                                },
                                "!topspammers" => {
                                    Some(commands::topspammers(&sqlite_connection))
                                },
                                "!rollgp" => {
                                    Some(commands::rollgp(&sqlite_connection, &user_id))
                                },
                                "!rollbiome" => {
                                    Some(commands::rollbiome())
                                },  
                                "!commands" => {
                                    Some(commands::commands())
                                },
                                "!rollcats" => {
                                    Some(commands::rollcats(args))
                                },
                                "!rollblazerods" => {
                                    Some(commands::rollblazerods(args))
                                },
                                "!tridentjuicers" => {
                                    Some(commands::tridentjuicers(&sqlite_connection))
                                },
                                "!gpjuicers" => {
                                    Some(commands::gpjuicers(&sqlite_connection))
                                },
                                "!dailytridentjuicers" => {
                                    Some(commands::dailytridentjuicers(&sqlite_connection))
                                },
                                "!tridentnoobs" => {
                                    Some(commands::tridentnoobs(&sqlite_connection))
                                },
                                "!rollskulls" => { 
                                    Some(commands::rollskulls(args)) 
                                },
                                "!commandstats" => {
                                    Some(commands::commandstats(&sqlite_connection, args))
                                },
                                "!raid" => {
                                    check_raid_file();
                                    Some(commands::raid(RAID_FILE_PATH))
                                },
                                "!rollphantoms" => {
                                    Some(commands::rollphantoms())
                                },
                                "!rollaassg" => {
                                    Some(commands::rollaassg())
                                },
                                "!route" => {
                                    Some(commands::route())
                                },
                                "!rollsilence" => {
                                    Some(commands::rollsilence())
                                },
                                "!hdwghfix" => {
                                    Some(commands::hdwghfix())
                                },
                                "!caamel" => {
                                    Some(commands::caamel())
                                },
                                "!rollheavycore" => {
                                    Some(commands::rollheavycore())
                                },
                                _ => { None }
                            }
                        } else {
                            None
                        };

                        // let result: Option<Result<String, String>> = None;

                        // ban
                        // if banan {
                        //     ban(&user_id, "Your trident roll sucks.", ban_duration).await;
                        // }

                        // update commands
                        if result != None || command == &"!combo" {
                            let fixed_command_name: &str = &command.replace("!", "emark_");
                            let command_update_query: &str = &format!("UPDATE commands SET uses = uses + 1 WHERE name = '{}' AND user_id = {};", fixed_command_name, user_id);
                            let command_set_query: &str = &format!("INSERT INTO commands (name, uses, user_id) VALUES ('{}', 1, {});", fixed_command_name, user_id);
                            
                            let query_result = sqlite_connection.execute(command_update_query);
                            let mut is_error: bool = false;
                            let mut error_message = String::new();

                            match query_result {
                                Err(err) => {
                                    is_error = true;

                                    println!("command update query error: {}", err);

                                    if let Err(msg_send_error) = send_client.say(CHANNEL.to_owned(), "Error: Database error.".to_owned()).await {
                                        println!("Error when sending a response message: {:?}", msg_send_error);
                                    }
                                },
                                Ok(_) => {
                                    if sqlite_connection.change_count() == 0 {
                                        let query_result = sqlite_connection.execute(command_set_query);
                                            
                                        if let Err(query_error) = query_result {
                                            println!("Command set query error: {}", query_error);
                                            
                                            if let Err(msg_send_error) = send_client.say(CHANNEL.to_owned(), "Error: Database error.".to_owned()).await {
                                                println!("Error when sending a response message: {:?}", msg_send_error);
                                            }
                                        }
                                    }
                                }
                            }

                            
                            // fix error handling eventually
                            // if is_error {
                            //     if let Some(error_message) = query_error.message {
                            //         if error_message == format!("no such column: {}", fixed_command_name) {
                            //             let query_result = sqlite_connection.execute(command_set_query);
                                    
                            //             if let Err(query_error) = query_result {
                            //                 println!("Command set query error: {}", query_error);
                                            
                            //                 if let Err(msg_send_error) = send_client.say(CHANNEL.to_owned(), "Error: Database error.".to_owned()).await {
                            //                     println!("Error when sending a response message: {:?}", msg_send_error);
                            //                 }
                            //             }
                            //         } else {
                            //             if let Err(msg_send_error) = send_client.say(CHANNEL.to_owned(), "Error: Database error.".to_owned()).await {
                            //                 println!("Error when sending a response message: {:?}", msg_send_error);
                            //             }
                            //         }
                            //     }
                            // }
                        }

                        match result {
                            Some(value) => {
                                match value {
                                    Ok(msg) | Err(msg) => {
                                        message += &format!("{} ", msg);
                                    }
                                }
                            },  
                            None => {}
                        }

                    }

                    // update users data
                    let user_update_query: &str = &format!("UPDATE users SET messages = messages + 1 WHERE user_id = {};", user_id);
                    let user_set_query: &str = &format!("INSERT INTO users (user_id, display_name, messages) VALUES ({}, '{}', 1);", user_id, user_display_name);

                    let query_result = sqlite_connection.execute(user_update_query);

                    match query_result {
                        Err(err) => {
                            println!("User update query error: {}", err);

                            if let Err(msg_send_error) = send_client.say(CHANNEL.to_owned(), "Error: Database error.".to_owned()).await {
                                println!("Error when sending a response message: {:?}", msg_send_error);
                            }
                        },
                        Ok(_) => {
                            if sqlite_connection.change_count() == 0 {
                                let query_result = sqlite_connection.execute(user_set_query);
                                    
                                if let Err(query_error) = query_result {
                                    println!("Command set query error: {}", query_error);
                                    
                                    if let Err(msg_send_error) = send_client.say(CHANNEL.to_owned(), "Error: Database error.".to_owned()).await {
                                        println!("Error when sending a response message: {:?}", msg_send_error);
                                    }
                                }
                            }
                        }
                    }

                    // send message
                    send_message(message, &send_client).await;
                    // let result = send_client.say(CHANNEL.to_owned(), message).await;

                    // match result {
                    //     Ok(_) => {},
                    //     Err(err) => {
                    //         println!("Error when sending a response message: {:?}", err);
                    //     }
                    // }
                },
                _ => {}
            }
        }   
    });
    
    // join a channel
    // This function only returns an error if the passed channel login name is malformed,
    // so in this simple case where the channel name is hardcoded we can ignore the potential
    // error with `unwrap`.
    client.join(CHANNEL.to_owned()).unwrap();
    println!("Bot is now running!");
    
    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}

// async fn get_message(command: &str, message_parts: Vec<&str>, spotify: AuthCodeSpotify) -> Option<String> {
//     match command { 
//         // "!combo" => {
//         //     Some(commands::combo(message_parts, &get_message, spotify))
//         // },
//         "!nomic" => {
//             Some(commands::nomic())
//         },
//         "!rolltrident" => {
//             Some(commands::rolltrident())
//         },
//         "!age" => {
//             Some(commands::age())
//         },
//         "!rollseed" => {
//             Some(commands::rollseed())
//         },
//         "!findseed" => {
//             Some(commands::findseed())
//         },
//         "!weather" => {
//             Some(commands::weather())
//         },
//         "!thunderodds" => {
//             Some(commands::thunderodds(message_parts))
//         },
//         "!skullodds" => {
//             Some(commands::skullrates(message_parts))
//         },
//         "!tridentodds" => {
//             Some(commands::tridentodds(message_parts))
//         },
//         "!rolldrowned" => {
//             Some(commands::rolldrowned(message_parts))
//         },
//         "Fishinge" => {
//             Some(commands::fishinge())
//         },
//         "!song" => {
//             Some(commands::song(spotify).await)
//         },
//         _ => { None }
//     }
// }
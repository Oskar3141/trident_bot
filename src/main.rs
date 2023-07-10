use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::message::ServerMessage;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use twitch_data::{LOGIN, OAUTH_TOKEN, CHANNEL};
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth, Config};

mod twitch_data;
mod commands;
mod thunder;
mod math;

#[tokio::main]
pub async fn main() {
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

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(msg) => {
                    let message_parts: Vec<&str> = msg.message_text.split(" ").collect();
                    // let command: &str = message_parts[0]; 
                    // let commands = message_parts.clone();

                    // match get_message(command, message_parts, spotify.clone()).await {
                    //     Some(message) => { send_client.say(CHANNEL.to_owned(), message).await.unwrap(); },
                    //     None => {}
                    // }
                    
                    let mut call_all_commands: bool = false;
                    let mut message: String = String::new();
                    for (i, command) in message_parts.iter().enumerate() {
                        let args: Vec<&str> = message_parts[i..message_parts.len()].into();

                        let result: Option<Result<String, String>> = if call_all_commands || i == 0 {
                            match *command {
                                "!combo" => {
                                    call_all_commands = true;
                                    None
                                },
                                "!nomic" => {
                                    Some(commands::nomic())
                                },
                                "!rolltrident" => {
                                    Some(commands::rolltrident())
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
                                    Some(commands::skullrates(args))
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
                                _ => { None }
                            }
                        } else {
                            None
                        };

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

                    let result = send_client.say(CHANNEL.to_owned(), message).await;

                    match result {
                        Ok(_) => {},
                        Err(err) => {
                            println!("Error when sending a response message: {:?}", err);
                        }
                    }
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
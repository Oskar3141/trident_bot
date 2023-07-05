use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::message::ServerMessage;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use twitch_data::{LOGIN, OAUTH_TOKEN, CHANNEL};

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

    // first thing you should do: start consuming incoming messages,
    // otherwise they will back up.
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(msg) => {
                    let message_parts: Vec<&str> = msg.message_text.split(" ").collect();
                    let command: &str = message_parts[0]; 

                    match get_message(command, message_parts) {
                        Some(message) => { send_client.say(CHANNEL.to_owned(), message).await.unwrap(); },
                        None => {}
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

fn get_message(command: &str, message_parts: Vec<&str>) -> Option<String> {
    match command { 
        "!combo" => {
            Some(commands::combo(message_parts, &get_message))
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
            Some(commands::thunderodds(message_parts))
        },
        "!skullodds" => {
            Some(commands::skullrates(message_parts))
        },
        "!tridentodds" => {
            Some(commands::tridentodds(message_parts))
        },
        _ => { None }
    }
}
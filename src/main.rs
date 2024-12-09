mod auth;
mod handler;
mod models;
mod utils;

use dotenv::dotenv;
use grammers_client::Client;
use handler::send_message_to_user_or_bot;
use std::collections::HashSet;
use tokio::runtime;
use utils::{extract_wallet_data, load_target_usernames};

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Get API credentials
    let api_id = utils::get_api_id()?;
    let api_hash: String = utils::get_api_hash()?;
    // Get session file path from environment variable (falling back to default if not set)
    let session_file = dotenv::var("SESSION_FILE").unwrap_or_else(|_| "user.session".to_string());

    println!("Connecting to Telegram...");

    let client = Client::connect(grammers_client::Config {
        session: grammers_client::session::Session::load_file_or_create(&session_file)?,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    println!("Connected to Telegram!");

    // Handle user authentication
    if !client.is_authorized().await? {
        println!("Not authorized, starting authentication...");
        auth::handle_authentication(&client).await?;
        println!("Authentication completed.");
    } else {
        println!("Already authorized.");
    }

    // Load target usernames
    let target_usernames = load_target_usernames()?;

    // Load target chat names from the environment variable (TARGET_CHANNELS)
    let target_chat_names: Vec<String> = dotenv::var("TARGET_CHANNELS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect();

    println!("Target channels to listen for: {:?}", target_chat_names);

    // Resolve all target chat usernames once, store their chat IDs in a HashSet for fast lookup
    let mut target_chat_ids = HashSet::new();
    for target_chat_name in &target_chat_names {
        if let Ok(maybe_chat) = client.resolve_username(target_chat_name).await {
            if let Some(chat) = maybe_chat {
                target_chat_ids.insert(chat.id());
            } else {
                println!("Could not resolve chat: {}", target_chat_name);
            }
        } else {
            println!("Error resolving chat: {}", target_chat_name);
        }
    }

    // Listening for updates continuously
    println!("Listening for new messages...");

    loop {
        // Listen for new messages
        let update = client.next_update().await?;

        // Check if we received a new message
        if let grammers_client::Update::NewMessage(message) = update {
            let chat = message.chat();

            // Check if the message is from any of the target chats
            if target_chat_ids.contains(&chat.id()) {
                // Process only incoming messages (not outgoing)
                if !message.outgoing() {
                    let message_text = message.text();

                    // Extract wallet data from the message
                    let wallet_data = extract_wallet_data(&message_text);

                    // If no wallet data found, skip the message
                    if wallet_data.wallet_address.is_none() && wallet_data.wallet_name.is_none() {
                        continue; // Skip to the next message
                    }

                    // Send extracted wallet data to the target users
                    for target_username in &target_usernames {
                        if let Err(e) =
                            send_message_to_user_or_bot(&client, target_username, &wallet_data)
                                .await
                        {
                            println!("Error sending wallet data to {}: {}", target_username, e);
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Loaded environment variables.");

    runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}

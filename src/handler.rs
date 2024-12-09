use crate::models::WalletData;
use grammers_client::Client;
use std::error::Error;

pub async fn send_message_to_user_or_bot(
    client: &Client,
    target_username: &str,
    wallet_data: &WalletData, // Accept WalletData
) -> Result<(), Box<dyn Error>> {
    // Format the WalletData into a string message
    let message_text = format_wallet_data(wallet_data);

    // Resolve the target username (either user or bot)
    let target_chat = match client.resolve_username(target_username).await {
        Ok(Some(chat)) => chat,
        Ok(None) => {
            println!("Error: Could not resolve username: {}", target_username);
            return Ok(()); // Skip sending the message to this user
        }
        Err(e) => {
            println!("Error resolving username {}: {}", target_username, e);
            return Ok(()); // Skip sending the message to this user
        }
    };

    // Send the message to the resolved chat (user or bot)
    match client.send_message(&target_chat, &*message_text).await {
        // Dereference message_text
        Ok(_) => {
            println!("Message sent to {}", target_username);
            Ok(())
        }
        Err(e) => {
            println!("Error sending message to {}: {}", target_username, e);
            Ok(()) // Do not panic, just log the error
        }
    }
}

fn format_wallet_data(wallet_data: &WalletData) -> String {
    // Create a long-lived binding for the "N/A" value
    let default_value = "N/A".to_string();

    let address = wallet_data
        .wallet_address
        .as_ref()
        .unwrap_or(&default_value); // Use the long-lived binding
    let name = wallet_data.wallet_name.as_ref().unwrap_or(&default_value); // Use the long-lived binding
    let time = wallet_data.wallet_time.as_ref().unwrap_or(&default_value); // Use the long-lived binding

    // Emoji decorations and cool formatting
    let header = "💎💰  **Wallet Information**  💰💎";
    let separator = "--------------------------------";
    let address_label = "🌍 Wallet Address:";
    let name_label = "👤 Wallet Name:";
    let time_label = "⏰ Time:";

    format!(
        "{}\n{}\n{} {} {}\n{} {} {}\n{} {} {}",
        header,
        separator,
        address_label,
        address,
        separator,
        name_label,
        name,
        separator,
        time_label,
        time,
        separator
    )
}

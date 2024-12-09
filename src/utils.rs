use crate::models::WalletData;
use chrono;
use dotenv::var;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::io::BufRead;

lazy_static! {
    static ref WORDS_TO_INCLUDE: String =
        var("WORDS_TO_INCLUDE").unwrap_or_else(|_| "SOL".to_string());
    static ref WORDS_TO_SKIP: HashSet<String> = var("WORDS_TO_SKIP")
        .unwrap_or_else(|_| "update,pumfun".to_string())
        .split(',')
        .map(|s| s.to_string())
        .collect();
    static ref ADDRESS_REGEX: Regex = Regex::new(
        &var("ADDRESS_REGEX").unwrap_or_else(|_| r"[1-9A-HJ-NP-Za-km-z]{32,44}".to_string())
    )
    .unwrap();
    static ref NAME_START_CHAR: char = var("NAME_START_CHAR")
        .unwrap_or_else(|_| "$".to_string())
        .chars()
        .next()
        .unwrap();
}

/// Function to extract wallet information from the message text.
/// This assumes a message may contain a wallet address, wallet name, and wallet time.
/// It may return an empty WalletData if no valid data is found.
pub fn extract_wallet_data(message_text: &str) -> WalletData {
    // Convert the message to lowercase for case-insensitive comparison
    let message_text_lower = message_text.to_lowercase();

    // Check if the message contains any of the words to skip (case-insensitive)
    if WORDS_TO_SKIP
        .iter()
        .any(|word| message_text_lower.contains(&word.to_lowercase()))
    {
        println!("Skipping message: contains words to skip.");
        return WalletData::new(None, None, None); // Skip if contains words to skip
    }

    // Check if the message contains any of the words to include (case-insensitive)
    if !message_text_lower.contains(&WORDS_TO_INCLUDE.to_lowercase()) {
        println!("Skipping message: does not contain required words to include.");
        return WalletData::new(None, None, None); // Skip if doesn't contain words to include
    }

    // Clean up message for regex match by removing unwanted characters (e.g., parentheses, extra spaces)
    let cleaned_message = message_text.replace(
        |c: char| c == '(' || c == ')' || c == '🔥' || c == ' ' || c == '\n' || c == '\r',
        "",
    );

    // First, try to extract wallet address using the regex (case-insensitive)
    let wallet_address = ADDRESS_REGEX
        .find(&cleaned_message) // Use cleaned-up message text
        .map(|m| m.as_str().to_string());

    // If wallet address is found, consider the message as valid, and extract wallet time as current datetime
    if wallet_address.is_some() {
        let wallet_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        return WalletData::new(wallet_address, None, Some(wallet_time));
    }

    // If no wallet address, check for name starting with the special character (case-insensitive)
    let wallet_name = message_text_lower
        .split_whitespace()
        .find(|&word| word.starts_with(*NAME_START_CHAR))
        .map(|name| name.to_string());

    // If no address and no valid name, return empty WalletData
    if wallet_name.is_none() {
        println!("No wallet address or valid name found.");
        return WalletData::new(None, None, None);
    }

    // If we found a name, take the current time as wallet time (address is still null)
    let wallet_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    WalletData::new(None, wallet_name, Some(wallet_time))
}

pub fn get_api_id() -> Result<i32, Box<dyn Error>> {
    let api_id = env::var("API_ID")?.parse::<i32>()?; // Using env::var to access environment variables
    Ok(api_id)
}

pub fn get_api_hash() -> Result<String, Box<dyn Error>> {
    let api_hash = env::var("API_HASH")?; // Using env::var to access environment variables
    Ok(api_hash)
}

pub fn prompt(message: &str) -> Result<String, Box<dyn Error>> {
    use std::io::{self, Write};

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?; // Prompt user input
    stdout.flush()?; // Ensure it's printed immediately

    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line)?; // Get user input
    Ok(line)
}

pub fn load_target_usernames() -> Result<HashSet<String>, Box<dyn Error>> {
    let target_usernames_str = env::var("TARGET_USERNAMES")?; // Get target usernames from environment variable
    let target_usernames = target_usernames_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<HashSet<String>>(); // Return set of target usernames
    Ok(target_usernames)
}

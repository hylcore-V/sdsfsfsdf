use crate::utils::prompt;
use grammers_client::{Client, SignInError};
use std::error::Error;

pub async fn handle_authentication(client: &Client) -> Result<(), Box<dyn Error>> {
    println!("Not authorized. Signing in...");

    let phone = prompt("Enter your phone number (international format): ")?;
    println!("Phone entered: {}", phone.trim());

    let token = client.request_login_code(&phone).await?;
    let code = prompt("Enter the code you received: ")?;
    println!("Code entered: {}", code.trim());

    let signed_in = match client.sign_in(&token, &code).await {
        Ok(_) => true,
        Err(SignInError::PasswordRequired(password_token)) => {
            // Handle password request
            let hint = password_token.hint().unwrap_or("None");
            let prompt_message = format!("Enter the password (hint {}): ", &hint);
            let password = prompt(prompt_message.as_str())?;
            println!("Password entered: {}", password.trim());

            client
                .check_password(password_token, password.trim())
                .await
                .is_ok()
        }
        Err(e) => {
            println!("Error during sign-in: {}", e);
            false
        }
    };

    if !signed_in {
        return Err("Authentication failed.".into());
    }

    if let Err(e) = client.session().save_to_file("dialogs.session") {
        println!("Failed to save session: {}", e);
    }

    Ok(())
}

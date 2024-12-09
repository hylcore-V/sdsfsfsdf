#[derive(Debug, Clone)]
pub struct WalletData {
    pub wallet_address: Option<String>,
    pub wallet_name: Option<String>,
    pub wallet_time: Option<String>,
}

impl WalletData {
    pub fn new(
        wallet_address: Option<String>,
        wallet_name: Option<String>,
        wallet_time: Option<String>,
    ) -> Self {
        WalletData {
            wallet_address,
            wallet_name,
            wallet_time,
        }
    }
}

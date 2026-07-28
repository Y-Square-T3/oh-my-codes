use super::{ConfigAction, ConfigCommand};
use omc_api::client::OmcClient;

pub async fn run(client: &OmcClient, cmd: ConfigCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.action {
        ConfigAction::Show => {
            let resp = client.config().await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        ConfigAction::Path => {
            let resp = client.config_path().await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}

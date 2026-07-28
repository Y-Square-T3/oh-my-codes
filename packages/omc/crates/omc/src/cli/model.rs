use super::ui;
use super::{ModelAction, ModelCommand};
use comfy_table::{Cell, Table};
use console::style;
use omc_api::client::OmcClient;

pub async fn run(client: &OmcClient, cmd: ModelCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.action {
        ModelAction::List { provider, json } => {
            model_list_effect(client, provider.as_deref(), json).await
        }
        ModelAction::Sync { json } => model_sync_effect(client, json).await,
    }
}

async fn model_list_effect(
    client: &OmcClient,
    provider: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.models_list(provider).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if resp.providers.is_empty() {
        ui::print_warning(
            "No models found. Run `omc model sync` to fetch models from your account.",
        );
        return Ok(());
    }

    if let Some(email) = &resp.account_email {
        println!();
        println!("  {}", style(format!("Models for: {email}")).bold());
        if let Some(url) = &resp.account_url {
            println!("  {}", style(url).dim());
        }
        println!();
    }

    for p in &resp.providers {
        let provider_models: Vec<_> = resp
            .models
            .iter()
            .filter(|m| m.provider_id == p.id)
            .collect();

        println!(
            "  {} {}",
            style(format!("{} ({})", p.name, p.id)).bold(),
            style(format!("— {} models", provider_models.len())).dim()
        );

        if provider_models.is_empty() {
            ui::print_dim("(no models)");
        } else {
            let mut table = Table::new();
            table
                .set_header(vec!["Model", "Family", "Reason", "Tool", "Context"])
                .set_width(80);

            for m in &provider_models {
                let context = ui::format_context(m.limit_context);
                let reasoning_cell = ui::yes_no_cell(m.reasoning == Some(true));
                let tool_cell = ui::yes_no_cell(m.tool_call == Some(true));
                let context_cell = ui::dim_cell(&context);

                table.add_row(vec![
                    Cell::new(&m.name),
                    Cell::new(m.family.as_deref().unwrap_or("-")),
                    reasoning_cell,
                    tool_cell,
                    context_cell,
                ]);
            }

            println!("{table}");
        }

        println!();
    }

    Ok(())
}

async fn model_sync_effect(
    client: &OmcClient,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.models_sync().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    println!(
        "  {} Refreshed {} providers, {} models",
        style("✓").green().bold(),
        resp.providers,
        resp.models
    );

    Ok(())
}

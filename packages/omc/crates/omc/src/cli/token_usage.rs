use super::ui;
use super::{TokenUsageAction, TokenUsageCommand};
use comfy_table::{Cell, CellAlignment, Color as TColor, Table};
use console::style;
use omc_api::client::OmcClient;

pub async fn run(
    client: &OmcClient,
    cmd: TokenUsageCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd
        .action
        .unwrap_or(TokenUsageAction::Status { json: false })
    {
        TokenUsageAction::Status { json } => token_usage_status_effect(client, json).await,
        TokenUsageAction::Push { json } => token_usage_push_effect(client, json).await,
        TokenUsageAction::List { limit, json } => {
            token_usage_list_effect(client, limit, json).await
        }
        TokenUsageAction::Summary { days, json } => {
            token_usage_summary_effect(client, days, json).await
        }
    }
}

async fn token_usage_status_effect(
    client: &OmcClient,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.token_usage_status().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    println!();
    println!("  {}", style("Token Usage Status").bold().underlined());
    println!();
    println!(
        "  {} {}",
        style("Unpushed records:").dim(),
        style(resp.unpushed_count).cyan().bold()
    );
    println!(
        "  {} {}",
        style("Active account:").dim(),
        if resp.has_active_account {
            style("yes").green().to_string()
        } else {
            style("no").yellow().to_string()
        }
    );
    println!();

    Ok(())
}

async fn token_usage_push_effect(
    client: &OmcClient,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.token_usage_push().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if resp.pushed_count == 0 && resp.failed_count == 0 {
        ui::print_dim("No unpushed records.");
        return Ok(());
    }

    println!(
        "  {} {} {} {} {}",
        style("✓").green().bold(),
        style("Pushed"),
        style(resp.pushed_count).cyan().bold(),
        style("records in"),
        style(resp.total_batches).cyan().bold()
    );

    if resp.failed_count > 0 {
        println!(
            "  {} {} {}",
            style("✗").red().bold(),
            style(resp.failed_count).red().bold(),
            style("records failed").red()
        );
    }

    Ok(())
}

async fn token_usage_list_effect(
    client: &OmcClient,
    limit: Option<usize>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.token_usage_list(limit, None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if resp.records.is_empty() {
        ui::print_warning("No token usage records found.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("").set_alignment(CellAlignment::Center),
            Cell::new("Client"),
            Cell::new("Model"),
            Cell::new("Input"),
            Cell::new("Output"),
            Cell::new("Reason"),
            Cell::new("Time"),
        ])
        .set_width(90);

    for r in &resp.records {
        let status_cell = if r.pushed {
            ui::success_cell()
        } else {
            ui::inactive_cell()
        };

        let model_display = ui::truncate_model(&r.model_id, 20);

        let time_display = chrono::DateTime::from_timestamp_millis(r.recorded_at)
            .map(|dt| dt.format("%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            status_cell,
            Cell::new(&r.client),
            ui::cyan_cell(&model_display),
            Cell::new(r.input_tokens),
            Cell::new(r.output_tokens),
            Cell::new(r.reasoning_tokens),
            ui::dim_cell(&time_display),
        ]);
    }

    println!();
    println!("{table}");
    println!(
        "  {} {}  {} {}",
        style("●").green(),
        style("pushed").dim(),
        style("○").dim(),
        style("pending").dim()
    );
    println!(
        "  {} {} records",
        style("Total:").dim(),
        style(resp.total).cyan()
    );
    println!();

    Ok(())
}

async fn token_usage_summary_effect(
    client: &OmcClient,
    days: Option<i64>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.token_usage_summary(days).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if resp.items.is_empty() {
        ui::print_warning("No usage data found.");
        return Ok(());
    }

    let period = days
        .map(|d| format!("Last {} days", d))
        .unwrap_or_else(|| "All time".to_string());

    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("Provider"),
            Cell::new("Model"),
            Cell::new("Requests").set_alignment(CellAlignment::Right),
            Cell::new("Input").set_alignment(CellAlignment::Right),
            Cell::new("Output").set_alignment(CellAlignment::Right),
            Cell::new("Reasoning").set_alignment(CellAlignment::Right),
        ])
        .set_width(90);

    let mut grand_requests: i64 = 0;
    let mut grand_input: i64 = 0;
    let mut grand_output: i64 = 0;

    for item in &resp.items {
        let model_display = ui::truncate_model(&item.model_id, 20);

        table.add_row(vec![
            Cell::new(&item.provider_id),
            ui::cyan_cell(&model_display),
            Cell::new(item.request_count).fg(TColor::Green),
            Cell::new(item.total_input),
            Cell::new(item.total_output),
            Cell::new(item.total_reasoning),
        ]);

        grand_requests += item.request_count;
        grand_input += item.total_input;
        grand_output += item.total_output;
    }

    println!();
    println!(
        "  {} {}",
        style("Token Usage Summary").bold().underlined(),
        style(format!("— {period}")).dim()
    );
    println!();
    println!("{table}");
    println!();
    println!(
        "  {} {} {} {} {} {} {}",
        style("Total:").bold(),
        style(grand_requests).cyan().bold(),
        style("requests,").dim(),
        style(grand_input).bold(),
        style("in,").dim(),
        style(grand_output).bold(),
        style("out").dim()
    );
    println!();

    Ok(())
}

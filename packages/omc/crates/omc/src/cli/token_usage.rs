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
        TokenUsageAction::List {
            limit,
            page,
            all,
            detail,
            json,
        } => token_usage_list_effect(client, limit, page, all, detail, json).await,
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
    limit: usize,
    page: usize,
    all: bool,
    detail: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let pushed_filter = if all { None } else { Some(false) };
    let offset = (page.saturating_sub(1)) * limit;
    let resp = client
        .token_usage_list(Some(limit), Some(offset), pushed_filter)
        .await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    if resp.records.is_empty() {
        if all {
            ui::print_warning("No token usage records found.");
        } else {
            ui::print_warning("No pending records found. Use --all to show all records.");
        }
        return Ok(());
    }

    let total_pages = resp.total.div_ceil(limit);

    let mut table = Table::new();
    let mut headers = vec![
        Cell::new("").set_alignment(CellAlignment::Center),
        Cell::new("Client"),
        Cell::new("Agent"),
        Cell::new("Model"),
    ];
    if detail {
        headers.push(Cell::new("Provider"));
    }
    headers.push(Cell::new("Input").set_alignment(CellAlignment::Right));
    headers.push(Cell::new("Output").set_alignment(CellAlignment::Right));
    headers.push(Cell::new("Reason").set_alignment(CellAlignment::Right));
    if detail {
        headers.push(Cell::new("CacheR").set_alignment(CellAlignment::Right));
        headers.push(Cell::new("CacheW").set_alignment(CellAlignment::Right));
    }
    headers.push(Cell::new("Time"));
    table.set_header(headers);

    let table_width = if detail { 140 } else { 100 };
    table.set_width(table_width);

    let mut total_input: i64 = 0;
    let mut total_output: i64 = 0;
    let mut total_reasoning: i64 = 0;
    let mut total_cache_read: i64 = 0;
    let mut total_cache_write: i64 = 0;

    for r in &resp.records {
        let status_cell = if r.pushed {
            ui::success_cell()
        } else {
            ui::inactive_cell()
        };

        let agent_display = r.agent.as_deref().unwrap_or("-");
        let model_display = ui::truncate_model(&r.model_id, 20);

        let time_display = chrono::DateTime::from_timestamp_millis(r.recorded_at)
            .map(|dt| dt.format("%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string());

        let mut row = vec![
            status_cell,
            Cell::new(&r.client),
            Cell::new(agent_display),
            ui::cyan_cell(&model_display),
        ];
        if detail {
            row.push(Cell::new(&r.provider_id));
        }
        row.push(Cell::new(r.input_tokens).set_alignment(CellAlignment::Right));
        row.push(Cell::new(r.output_tokens).set_alignment(CellAlignment::Right));
        row.push(Cell::new(r.reasoning_tokens).set_alignment(CellAlignment::Right));
        if detail {
            row.push(Cell::new(r.cache_read_tokens).set_alignment(CellAlignment::Right));
            row.push(Cell::new(r.cache_write_tokens).set_alignment(CellAlignment::Right));
        }
        row.push(ui::dim_cell(&time_display));

        table.add_row(row);

        total_input += r.input_tokens;
        total_output += r.output_tokens;
        total_reasoning += r.reasoning_tokens;
        total_cache_read += r.cache_read_tokens;
        total_cache_write += r.cache_write_tokens;
    }

    let filter_label = if all { "all" } else { "pending" };
    println!();
    println!(
        "  {} {}",
        style("Token Usage").bold().underlined(),
        style(format!("— showing {filter_label}")).dim()
    );
    println!();
    println!("{table}");
    println!(
        "  {} {}  {} {}",
        style("●").green(),
        style("pushed").dim(),
        style("○").dim(),
        style("pending").dim()
    );

    let mut totals = format!(
        "  {} {} {} {} {} {}",
        style("Total:").bold(),
        style(total_input).cyan().bold(),
        style("in,").dim(),
        style(total_output).cyan().bold(),
        style("out,").dim(),
        style(total_reasoning).cyan().bold(),
    );
    if detail {
        totals = format!(
            "{totals} {} {} {}, {} {} {}",
            style("reason,").dim(),
            style(total_cache_read).cyan().bold(),
            style("cacheR,").dim(),
            style(total_cache_write).cyan().bold(),
            style("cacheW").dim(),
            "",
        );
    } else {
        totals = format!("{totals} {}", style("reason").dim());
    }
    println!("{totals}");

    println!(
        "  {} {} of {} ({} {})",
        style("Page").dim(),
        style(page).cyan(),
        style(total_pages).cyan(),
        style(resp.total).cyan(),
        style(format!("{filter_label} records")).dim()
    );
    if page < total_pages {
        println!(
            "  {} {}",
            style("Hint:").dim(),
            style(format!("use --page {} to see next page", page + 1)).dim()
        );
    }
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

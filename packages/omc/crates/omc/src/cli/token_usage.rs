use super::ui;
use super::{TokenUsageAction, TokenUsageCommand};
use comfy_table::{Cell, CellAlignment, Color as TColor, Table};
use console::style;
use omc_api::client::OmcClient;

pub async fn run(
    client: &OmcClient,
    cmd: TokenUsageCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.action.unwrap_or(TokenUsageAction::Summary {
        days: None,
        json: false,
    }) {
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
    let overview = client.token_usage_overview(days).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&overview)?);
        return Ok(());
    }

    let period = days
        .map(|d| format!("Last {} days", d))
        .unwrap_or_else(|| "All time".to_string());

    println!();
    println!(
        "  {} {}",
        style("Local Agent Coding Overview").bold().underlined(),
        style(format!("— {period}")).dim()
    );
    println!();

    println!("  {}", style("Headline").bold());
    println!(
        "    {} {}",
        style("Requests:").dim(),
        style(ui::format_human(overview.headline.requests))
            .cyan()
            .bold()
    );
    println!(
        "    {} {}",
        style("Input:").dim(),
        style(ui::format_human(overview.headline.input_tokens))
            .cyan()
            .bold()
    );
    println!(
        "    {} {}",
        style("Output:").dim(),
        style(ui::format_human(overview.headline.output_tokens))
            .cyan()
            .bold()
    );
    println!(
        "    {} {}",
        style("Reasoning:").dim(),
        style(ui::format_human(overview.headline.reasoning_tokens))
            .cyan()
            .bold()
    );
    println!(
        "    {} {}",
        style("Cache read:").dim(),
        style(ui::format_human(overview.headline.cache_read_tokens))
            .cyan()
            .bold()
    );
    println!(
        "    {} {}",
        style("Cache write:").dim(),
        style(ui::format_human(overview.headline.cache_write_tokens))
            .cyan()
            .bold()
    );
    println!(
        "    {} {} {} / {} {}",
        style("Unpushed:").dim(),
        style(overview.headline.unpushed_records).cyan().bold(),
        style("records").dim(),
        style(ui::format_human(overview.headline.unpushed_tokens))
            .cyan()
            .bold(),
        style("tokens").dim()
    );
    println!();

    if !overview.top_models.is_empty() {
        println!("  {}", style("Top models").bold());
        let mut table = Table::new();
        table
            .set_header(vec![
                Cell::new(""),
                Cell::new("Model"),
                Cell::new("Requests").set_alignment(CellAlignment::Right),
                Cell::new("Input").set_alignment(CellAlignment::Right),
                Cell::new("Output").set_alignment(CellAlignment::Right),
            ])
            .set_width(70);

        for (i, m) in overview.top_models.iter().enumerate() {
            let model_display = ui::truncate_model(&m.model_id, 20);
            table.add_row(vec![
                Cell::new(i + 1).fg(TColor::DarkGrey),
                ui::cyan_cell(&model_display),
                Cell::new(ui::format_human(m.request_count)).fg(TColor::Green),
                Cell::new(ui::format_human(m.total_input)),
                Cell::new(ui::format_human(m.total_output)),
            ]);
        }
        println!("{table}");
        println!();
    }

    if !overview.top_agents.is_empty() {
        println!("  {}", style("Top agents").bold());
        let mut table = Table::new();
        table
            .set_header(vec![
                Cell::new(""),
                Cell::new("Agent"),
                Cell::new("Requests").set_alignment(CellAlignment::Right),
                Cell::new("Input").set_alignment(CellAlignment::Right),
                Cell::new("Output").set_alignment(CellAlignment::Right),
            ])
            .set_width(70);

        for (i, a) in overview.top_agents.iter().enumerate() {
            table.add_row(vec![
                Cell::new(i + 1).fg(TColor::DarkGrey),
                ui::cyan_cell(&a.label),
                Cell::new(ui::format_human(a.request_count)).fg(TColor::Green),
                Cell::new(ui::format_human(a.total_input)),
                Cell::new(ui::format_human(a.total_output)),
            ]);
        }
        println!("{table}");
        println!();
    }

    if !overview.top_clients.is_empty() {
        println!("  {}", style("Top clients").bold());
        let mut table = Table::new();
        table
            .set_header(vec![
                Cell::new(""),
                Cell::new("Client"),
                Cell::new("Requests").set_alignment(CellAlignment::Right),
                Cell::new("Input").set_alignment(CellAlignment::Right),
                Cell::new("Output").set_alignment(CellAlignment::Right),
            ])
            .set_width(70);

        for (i, c) in overview.top_clients.iter().enumerate() {
            table.add_row(vec![
                Cell::new(i + 1).fg(TColor::DarkGrey),
                ui::cyan_cell(&c.label),
                Cell::new(ui::format_human(c.request_count)).fg(TColor::Green),
                Cell::new(ui::format_human(c.total_input)),
                Cell::new(ui::format_human(c.total_output)),
            ]);
        }
        println!("{table}");
        println!();
    }

    if !overview.trend.is_empty() {
        println!("  {}", style("7-day trend").bold());
        let max_tokens = overview
            .trend
            .iter()
            .map(|d| d.total_tokens)
            .max()
            .unwrap_or(1);
        let bar_width = 20usize;

        let mut table = Table::new();
        table
            .set_header(vec![
                Cell::new("Date"),
                Cell::new("Requests").set_alignment(CellAlignment::Right),
                Cell::new("Tokens").set_alignment(CellAlignment::Right),
                Cell::new("Activity"),
            ])
            .set_width(70);

        for d in &overview.trend {
            let bar_len = if max_tokens > 0 {
                ((d.total_tokens as f64 / max_tokens as f64) * bar_width as f64).round() as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_len);
            table.add_row(vec![
                Cell::new(&d.date),
                Cell::new(ui::format_human(d.requests)).fg(TColor::Green),
                Cell::new(ui::format_human(d.total_tokens)),
                Cell::new(bar).fg(TColor::Cyan),
            ]);
        }
        println!("{table}");
        println!();
    }

    if overview.headline.unpushed_records > 0 {
        println!(
            "  {} {}",
            style("Hint:").dim(),
            style("run `omc tu push` to upload unpushed records").dim()
        );
        println!();
    }

    Ok(())
}

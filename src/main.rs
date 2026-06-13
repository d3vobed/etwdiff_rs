use clap::Parser;

mod analysis;
mod metrics;
mod parser;
mod report;

use crate::report::{JsonAnalysis, JsonMetrics, JsonResult};

#[derive(Parser)]
#[command(author = "OBED-PROJECT", version = "0.1.0", about = "Quantify ETW telemetry degradation and visibility.", long_about = None)]
struct Cli {
    /// Path to the baseline ETW JSON log
    baseline: String,

    /// Path to the mutated ETW JSON log
    mutated: String,

    /// Filter events by a specific ProcessID
    #[arg(long)]
    pid: Option<String>,

    /// Only analyze a specific provider name
    #[arg(long)]
    provider: Option<String>,

    /// TRS Volume Weight
    #[arg(long, default_value_t = 0.45)]
    w1: f64,

    /// TRS Entropy Weight
    #[arg(long, default_value_t = 0.35)]
    w2: f64,

    /// TRS Timing Weight
    #[arg(long, default_value_t = 0.20)]
    w3: f64,

    /// Path to save HTML report
    #[arg(long)]
    output: Option<String>,

    /// Path to save JSON report
    #[arg(long)]
    json: Option<String>,

    /// Path to append CSV summary
    #[arg(long)]
    csv: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let mut base_events = match parser::parse_etw_json(&cli.baseline, cli.pid.as_deref()) {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Error parsing baseline {}: {}", cli.baseline, e);
            std::process::exit(1);
        }
    };

    let mut mut_events = match parser::parse_etw_json(&cli.mutated, cli.pid.as_deref()) {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Error parsing mutated {}: {}", cli.mutated, e);
            std::process::exit(1);
        }
    };

    if let Some(prov) = &cli.provider {
        base_events.retain(|e| &e.provider_name == prov);
        mut_events.retain(|e| &e.provider_name == prov);
    }

    let base_metrics = metrics::compute_metrics(&base_events);
    let mut_metrics = metrics::compute_metrics(&mut_events);

    let analysis_res = analysis::analyze(
        &base_events,
        &mut_events,
        &base_metrics,
        &mut_metrics,
        cli.w1,
        cli.w2,
        cli.w3,
    );

    let provider_name = cli.provider.unwrap_or_else(|| "Combined".to_string());

    report::print_terminal_summary(&base_metrics, &mut_metrics, &analysis_res, &provider_name);

    let json_result = JsonResult {
        baseline: JsonMetrics {
            f: base_metrics.f,
            h: base_metrics.h,
            cv_t: base_metrics.cv_t,
        },
        mutated: JsonMetrics {
            f: mut_metrics.f,
            h: mut_metrics.h,
            cv_t: mut_metrics.cv_t,
        },
        analysis: JsonAnalysis {
            missing_events: analysis_res.missing_events.clone(),
            provider_reduction_pct: analysis_res.provider_reduction_pct,
            evasion_categories: analysis_res.evasion_categories.clone(),
        },
        trs: analysis_res.trs,
    };

    if let Some(path) = cli.json {
        let _ = report::export_json(&path, &json_result);
    }
    
    if let Some(path) = cli.csv {
        let _ = report::export_csv(&path, &json_result);
    }
    
    if let Some(path) = cli.output {
        let _ = report::export_html(&path, &json_result);
    }
}

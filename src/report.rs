use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;

use crate::analysis::AnalysisResult;
use crate::metrics::Metrics;

#[derive(Serialize)]
pub struct JsonMetrics {
    pub f: usize,
    pub h: f64,
    pub cv_t: f64,
}

#[derive(Serialize)]
pub struct JsonAnalysis {
    pub missing_events: Vec<String>,
    pub provider_reduction_pct: f64,
}

#[derive(Serialize)]
pub struct JsonResult {
    pub baseline: JsonMetrics,
    pub mutated: JsonMetrics,
    pub analysis: JsonAnalysis,
    pub trs: f64,
}

pub fn print_terminal_summary(
    base: &Metrics,
    mut_m: &Metrics,
    analysis: &AnalysisResult,
    provider: &str,
) {
    println!("{:=<60}", "");
    println!(" ETW Telemetry Degradation Summary : {}", provider);
    println!("{:=<60}", "");
    
    println!("\n[ BASELINE ]");
    println!("  Total Events (F)   : {}", base.f);
    println!("  Shannon Entropy (H): {:.4} bits", base.h);
    println!("  Timing Var. (CV_t) : {:.4}", base.cv_t);
    
    println!("\n[ MUTATED ]");
    println!("  Total Events (F)   : {}", mut_m.f);
    println!("  Shannon Entropy (H): {:.4} bits", mut_m.h);
    println!("  Timing Var. (CV_t) : {:.4}", mut_m.cv_t);
    
    println!("\n[ ANALYSIS ]");
    println!("  Missing Event Types: {}", analysis.missing_events.len());
    if !analysis.missing_events.is_empty() {
        for ev in analysis.missing_events.iter().take(10) {
            println!("    - {}", ev);
        }
        if analysis.missing_events.len() > 10 {
            println!("    - ... and {} more", analysis.missing_events.len() - 10);
        }
    }
    println!("  Provider Reduction : {:.2}%", analysis.provider_reduction_pct);
    
    println!("\n{:-<60}", "");
    println!(" TELEMETRY RESILIENCE SCORE (TRS): {:.4}", analysis.trs);
    println!("{:-<60}", "");
}

pub fn export_json(filepath: &str, result: &JsonResult) -> std::io::Result<()> {
    let mut file = std::fs::File::create(filepath)?;
    let content = serde_json::to_string_pretty(result)?;
    file.write_all(content.as_bytes())?;
    println!("[+] JSON report saved to {}", filepath);
    Ok(())
}

pub fn export_csv(filepath: &str, result: &JsonResult) -> std::io::Result<()> {
    let exists = std::path::Path::new(filepath).exists();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(filepath)?;
        
    if !exists {
        writeln!(file, "Base_F,Base_H,Base_CV_t,Mut_F,Mut_H,Mut_CV_t,Missing_Events_Count,Provider_Reduction_Pct,TRS")?;
    }
    
    writeln!(file, "{},{:.4},{:.4},{},{:.4},{:.4},{},{:.2},{:.4}",
        result.baseline.f, result.baseline.h, result.baseline.cv_t,
        result.mutated.f, result.mutated.h, result.mutated.cv_t,
        result.analysis.missing_events.len(), result.analysis.provider_reduction_pct,
        result.trs
    )?;
    
    println!("[+] CSV summary appended to {}", filepath);
    Ok(())
}

pub fn export_html(filepath: &str, result: &JsonResult) -> std::io::Result<()> {
    let mut missing_html = String::new();
    if result.analysis.missing_events.is_empty() {
        missing_html.push_str("<li><em>None</em></li>");
    } else {
        for ev in &result.analysis.missing_events {
            missing_html.push_str(&format!("<li><code>{}</code></li>", ev));
        }
    }

    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>etwdiff Report</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background: #f4f4f9; color: #333; margin: 0; padding: 20px; }}
        h1, h2, h3 {{ color: #2c3e50; }}
        .container {{ max-width: 900px; margin: auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 4px 8px rgba(0,0,0,0.1); }}
        .trs-box {{ background: #ecf0f1; padding: 15px; border-left: 5px solid #e74c3c; font-size: 1.2em; margin-bottom: 20px; }}
        .metrics-card {{ display: inline-block; width: 48%; margin-right: 2%; vertical-align: top; }}
        .metrics-card:last-child {{ margin-right: 0; }}
        .metric-val {{ font-weight: bold; color: #2980b9; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>ETW Telemetry Degradation Report</h1>
        
        <div class="trs-box">
            <strong>Telemetry Resilience Score (TRS):</strong> <span style="color: #e74c3c; font-weight: bold;">{:.4}</span>
        </div>

        <div>
            <div class="metrics-card">
                <h2>Baseline Metrics</h2>
                <ul>
                    <li>Volume (F): <span class="metric-val">{}</span></li>
                    <li>Entropy (H): <span class="metric-val">{:.4} bits</span></li>
                    <li>Timing Var (CV_t): <span class="metric-val">{:.4}</span></li>
                </ul>
            </div>
            
            <div class="metrics-card">
                <h2>Mutated Metrics</h2>
                <ul>
                    <li>Volume (F): <span class="metric-val">{}</span></li>
                    <li>Entropy (H): <span class="metric-val">{:.4} bits</span></li>
                    <li>Timing Var (CV_t): <span class="metric-val">{:.4}</span></li>
                </ul>
            </div>
        </div>

        <h2>Analysis</h2>
        <p>Provider Visibility Reduction: <strong>{:.2}%</strong></p>
        
        <h3>Missing Event Types</h3>
        <ul>
            {}
        </ul>
    </div>
</body>
</html>
"#,
        result.trs,
        result.baseline.f, result.baseline.h, result.baseline.cv_t,
        result.mutated.f, result.mutated.h, result.mutated.cv_t,
        result.analysis.provider_reduction_pct,
        missing_html
    );

    let mut file = std::fs::File::create(filepath)?;
    file.write_all(html.as_bytes())?;
    println!("[+] HTML report saved to {}", filepath);
    Ok(())
}

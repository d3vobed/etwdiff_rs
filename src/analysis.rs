use std::collections::HashSet;
use crate::parser::EtwEvent;
use crate::metrics::Metrics;

pub struct AnalysisResult {
    pub missing_events: Vec<String>,
    pub provider_reduction_pct: f64,
    pub trs: f64,
}

pub fn analyze(
    base_events: &[EtwEvent],
    mut_events: &[EtwEvent],
    base_metrics: &Metrics,
    mut_metrics: &Metrics,
    w_vol: f64,
    w_ent: f64,
    w_time: f64,
) -> AnalysisResult {
    
    let mut base_types = HashSet::new();
    let mut base_providers = HashSet::new();
    for e in base_events {
        base_types.insert(e.event_name.clone());
        base_providers.insert(e.provider_name.clone());
    }
    
    let mut mut_types = HashSet::new();
    let mut mut_providers = HashSet::new();
    for e in mut_events {
        mut_types.insert(e.event_name.clone());
        mut_providers.insert(e.provider_name.clone());
    }
    
    let mut missing_events: Vec<String> = base_types.difference(&mut_types).cloned().collect();
    missing_events.sort();
    
    let missing_providers: HashSet<_> = base_providers.difference(&mut_providers).collect();
    
    let provider_reduction_pct = if base_providers.is_empty() {
        0.0
    } else {
        (missing_providers.len() as f64 / base_providers.len() as f64) * 100.0
    };
    
    let f_base = base_metrics.f as f64;
    let f_mut = mut_metrics.f as f64;
    let h_base = base_metrics.h;
    let h_mut = mut_metrics.h;
    let cv_t_mut = mut_metrics.cv_t;
    
    let vol_ratio = if f_base == 0.0 {
        if f_mut > 0.0 { 1.0 } else { 0.0 }
    } else {
        (f_mut / f_base).min(1.0)
    };
    
    let ent_ratio = if h_base == 0.0 {
        if h_mut > 0.0 { 1.0 } else { 0.0 }
    } else {
        (h_mut / h_base).min(1.0)
    };
    
    let timing_cmp = (1.0 - cv_t_mut).max(0.0);
    
    let mut trs = w_vol * vol_ratio + w_ent * ent_ratio + w_time * timing_cmp;
    trs = trs.max(0.0).min(1.0);
    
    AnalysisResult {
        missing_events,
        provider_reduction_pct,
        trs,
    }
}

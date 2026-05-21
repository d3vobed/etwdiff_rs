use std::collections::HashMap;
use crate::parser::EtwEvent;

pub struct Metrics {
    pub f: usize,
    pub h: f64,
    pub cv_t: f64,
}

pub fn compute_metrics(events: &[EtwEvent]) -> Metrics {
    let f = events.len();
    if f == 0 {
        return Metrics { f: 0, h: 0.0, cv_t: 0.0 };
    }
    
    // Frequency and Entropy
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for e in events {
        *counts.entry(&e.event_name).or_insert(0) += 1;
    }
    
    let mut h = 0.0;
    for &count in counts.values() {
        let p = (count as f64) / (f as f64);
        if p > 0.0 {
            h -= p * p.log2();
        }
    }
    if h < 0.0 {
        h = 0.0;
    }
    
    // Timing
    let mut ts_list: Vec<_> = events.iter().filter_map(|e| e.parsed_ts).collect();
    ts_list.sort();
    
    let cv_t = if ts_list.len() > 1 {
        let mut gaps = Vec::with_capacity(ts_list.len() - 1);
        for i in 0..ts_list.len() - 1 {
            let diff = ts_list[i + 1].signed_duration_since(ts_list[i]).num_microseconds().unwrap_or(0);
            gaps.push(diff as f64);
        }
        
        let sum: f64 = gaps.iter().sum();
        let mean_gap = sum / (gaps.len() as f64);
        
        if mean_gap > 0.0 {
            let var_sum: f64 = gaps.iter().map(|&g| (g - mean_gap) * (g - mean_gap)).sum();
            let std_gap = (var_sum / (gaps.len() as f64)).sqrt();
            std_gap / mean_gap
        } else {
            0.0
        }
    } else {
        0.0
    };
    
    Metrics { f, h, cv_t }
}

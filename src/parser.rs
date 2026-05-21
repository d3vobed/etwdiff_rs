use chrono::{DateTime, Utc, TimeZone};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EtwEvent {
    pub provider_name: String,
    pub event_name: String,
    pub pid: Option<String>,
    pub parsed_ts: Option<DateTime<Utc>>,
}

fn parse_ts(val: Option<&Value>) -> Option<DateTime<Utc>> {
    let s = val?.as_str()?.trim();
    let s = if s.ends_with('Z') {
        format!("{}+00:00", &s[..s.len()-1])
    } else {
        s.to_string()
    };
    
    // Try ISO8601 with tz
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Some(dt.with_timezone(&Utc));
    }
    
    // Fallback: Naive
    if s.len() >= 19 {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&s[..19], "%Y-%m-%dT%H:%M:%S") {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }
    
    None
}

fn get_event_name(e: &Value) -> String {
    if let Some(xml) = e.get("XmlEventData") {
        if let Some(name) = xml.get("EventName").and_then(|v| v.as_str()) {
            return name.to_string();
        }
    }
    
    if let Some(name) = e.get("EventName").and_then(|v| v.as_str()) {
        return name.to_string();
    }
    
    if let Some(op) = e.get("Opcode") {
        let op_str = if op.is_number() {
            op.to_string()
        } else if let Some(s) = op.as_str() {
            s.to_string()
        } else {
            "Unknown".to_string()
        };
        
        if op_str != "0" && op_str != "None" && op_str != "" {
            return format!("Opcode_{}", op_str);
        }
    }
    
    "Unknown".to_string()
}

fn get_provider_name(e: &Value) -> String {
    e.get("ProviderName")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string()
}

fn get_pid(e: &Value) -> Option<String> {
    if let Some(xml) = e.get("XmlEventData") {
        if let Some(pid) = xml.get("PID") {
            return Some(if pid.is_number() { pid.to_string() } else { pid.as_str().unwrap_or("").to_string() });
        }
    }
    if let Some(pid) = e.get("ProcessID") {
        return Some(if pid.is_number() { pid.to_string() } else { pid.as_str().unwrap_or("").to_string() });
    }
    None
}

pub fn parse_etw_json<P: AsRef<Path>>(filepath: P, target_pid: Option<&str>) -> Result<Vec<EtwEvent>, Box<dyn std::error::Error>> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    
    let mut raw_events = Vec::new();
    
    if let Ok(val) = serde_json::from_str::<Value>(&content) {
        if let Some(arr) = val.as_array() {
            raw_events = arr.clone();
        } else if val.is_object() {
            raw_events.push(val);
        }
    } else {
        // Fallback to newline delimited
        for line in content.lines() {
            let line = line.trim().trim_end_matches(',');
            if line.starts_with('{') {
                if let Ok(val) = serde_json::from_str::<Value>(line) {
                    raw_events.push(val);
                }
            }
        }
    }
    
    let mut events = Vec::new();
    for e in raw_events {
        let pid = get_pid(&e);
        if let Some(t_pid) = target_pid {
            if pid.as_deref() != Some(t_pid) {
                continue;
            }
        }
        
        let ts = parse_ts(e.get("TimeStamp").or_else(|| e.get("Timestamp")));
        let event_name = get_event_name(&e);
        let provider_name = get_provider_name(&e);
        
        events.push(EtwEvent {
            provider_name,
            event_name,
            pid,
            parsed_ts: ts,
        });
    }
    
    Ok(events)
}

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use colored::Colorize;
use serde::Serialize;

use crate::api_response::PaginatedList;
use crate::types::*;

/// Maximum number of items to print to the terminal before writing to a file.
/// Prevents terminal hangs when a query returns hundreds or thousands of records.
const MAX_TERMINAL_ITEMS: usize = 10;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn print_result<T: Serialize>(value: &T, _format: &OutputFormat) {
    let json = serde_json::to_string_pretty(value).unwrap_or_default();
    println!("{json}");
}

/// Return a path for a temporary output file in the current working directory.
fn temp_output_path(prefix: &str, ext: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    std::path::PathBuf::from(format!("{prefix}_{ts}.{ext}"))
}

pub fn unique_output_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("output");
    let extension = path.extension().and_then(|value| value.to_str());

    for index in 2.. {
        let filename = match extension {
            Some(extension) => format!("{stem}_{index}.{extension}"),
            None => format!("{stem}_{index}"),
        };
        let candidate = parent.join(filename);
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!("unbounded loop always returns")
}

/// Write a value to a file and print the file path. Returns `true` if the
/// output was redirected to a file (because the item count exceeded the limit).
fn write_large_output<T: Serialize>(value: &T, item_count: usize) -> bool {
    if item_count <= MAX_TERMINAL_ITEMS {
        return false;
    }
    let path = temp_output_path("output", "json");
    let json = serde_json::to_string_pretty(value).unwrap_or_default();
    if fs::write(&path, json).is_ok() {
        println!("当前页 {} 条，已写入文件: {}", item_count, path.display());
        return true;
    }
    false
}

fn pagination_metadata_json<T>(list: &PaginatedList<T>) -> String {
    let mut lines = vec![format!("  \"count\": {}", list.count)];
    if let Some(total_pages) = list.total_pages {
        lines.push(format!("  \"total_pages\": {total_pages}"));
    }
    if let Some(current_page) = list.current_page {
        lines.push(format!("  \"current_page\": {current_page}"));
    }
    if let Some(has_next) = list.has_next {
        lines.push(format!("  \"has_next\": {has_next}"));
    }
    if let Some(has_previous) = list.has_previous {
        lines.push(format!("  \"has_previous\": {has_previous}"));
    }
    format!("{{\n{}\n}}", lines.join(",\n"))
}

pub fn print_pagination_metadata<T>(list: &PaginatedList<T>) {
    println!("{}", pagination_metadata_json(list));
}

/// Print a paginated list. If the number of items on this page exceeds
/// `MAX_TERMINAL_ITEMS`, write the JSON to a file and print the path instead.
pub fn print_paginated_items<T: Serialize>(list: &PaginatedList<T>) {
    print_pagination_metadata(list);
    if write_large_output(&list.items, list.items.len()) {
        return;
    }
    let json = serde_json::to_string_pretty(&list.items).unwrap_or_default();
    println!("{json}");
}

pub fn print_order(order: &Order) {
    let is_sequencing = order.order_type == "sequencing";

    println!("{}", format!("订单 ID  : {}", order.id).bold());
    println!(
        "类型     : {}",
        if is_sequencing {
            "测序"
        } else {
            "引物合成"
        }
    );
    println!("状态     : {}", status_colored(&order.status));
    println!("供应商   : {}", order.supplier_name);
    println!(
        "联系人   : {} {}",
        order.customer_name, order.customer_phone
    );
    if let Some(notes) = order.notes.as_deref().filter(|notes| !notes.is_empty()) {
        println!("备注     : {notes}");
    }
    println!(
        "总价     : {}",
        order.total_price.as_deref().unwrap_or("N/A")
    );
    println!("创建时间 : {}", order.created_at);

    if is_sequencing {
        if !order.items.is_empty() {
            println!("\n测序样品 ({} 条):", order.items.len());
            for item in &order.items {
                println!(
                    "  {:16} type={}  vector={}  测通={}",
                    item.primer_name,
                    item.r#type.as_deref().unwrap_or("-"),
                    item.seq_vector.as_deref().unwrap_or("-"),
                    if item.universal.unwrap_or(false) {
                        "是"
                    } else {
                        "否"
                    },
                );
            }
        }
        if !order.primer_items.is_empty() {
            println!("\n引物合成 ({} 条):", order.primer_items.len());
            for p in &order.primer_items {
                println!(
                    "  {:12} {:32} OD={}  {}",
                    p.primer_name,
                    p.sequence,
                    p.scale_od.as_deref().unwrap_or("-"),
                    p.purification_method.as_deref().unwrap_or(""),
                );
            }
        }
    } else if !order.items.is_empty() {
        println!("\n引物 ({} 条):", order.items.len());
        for item in &order.items {
            let mod_str = item
                .five_modification
                .as_ref()
                .filter(|s| !s.is_empty())
                .map(|s| format!("  [{s}]"))
                .unwrap_or_default();
            println!(
                "  {:12} {:32} {}bp {}  {}nmol{}",
                item.primer_name,
                item.sequence,
                item.base_count.unwrap_or(0),
                item.purification_method.as_deref().unwrap_or(""),
                item.nmoles.map(|v| v.to_string()).unwrap_or_default(),
                mod_str,
            );
        }
    }
}

pub fn print_order_brief(order: &Order) {
    println!(
        "{}  {:8}  {:8}  {:>6}  {}  {}",
        order.id,
        status_colored(&order.status),
        order.supplier_name,
        order.total_price.as_deref().unwrap_or("N/A"),
        &order.created_at[..order.created_at.len().min(19)],
        order.customer_name,
    );
}

fn status_colored(status: &str) -> colored::ColoredString {
    match status {
        "pending" => status.yellow(),
        "received" | "completed" | "done" => status.green(),
        "failed" | "cancelled" | "rejected" => status.red(),
        "processing" | "in_progress" => status.blue(),
        _ => status.normal(),
    }
}

pub fn print_stocks(stocks: &[Stock]) {
    if stocks.is_empty() {
        println!("暂无库存");
        return;
    }
    for s in stocks {
        println!(
            "{}  {:20}  剩余:{}  位置:{}",
            s.id,
            s.name.as_deref().or(s.primer_name.as_deref()).unwrap_or(""),
            s.remaining_quantity
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".into()),
            s.location_path
                .as_deref()
                .or(s.storage_location_id.as_deref())
                .unwrap_or(""),
        );
    }
}

pub fn print_templates(templates: &[Template]) {
    if templates.is_empty() {
        println!("暂无信息模板");
        return;
    }
    for t in templates {
        let default = if t.is_default.unwrap_or(false) {
            " [默认]"
        } else {
            ""
        };
        let ot = t.order_type.as_deref().unwrap_or("通用");
        println!("{}  {:16}  {}{}", t.id, t.name, ot, default);
    }
}

pub fn print_lab_members(members: &[LabMember]) {
    for m in members {
        println!("{}  {:12}  {:24}  {}", m.id, m.full_name, m.email, m.role);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    /// Verify that JSON serialization of Chinese text preserves UTF-8 correctly.
    /// This guards against Windows console encoding regressions where UTF-8 bytes
    /// are misinterpreted as the system code page (e.g. GBK / CP936).
    #[test]
    fn json_output_preserves_chinese_utf8() {
        let record = json!({
            "name": "京科968父本",
            "alias_name": "京92",
            "accession_no": "TS24-021",
            "object_name": "inbred"
        });

        let output = serde_json::to_string_pretty(&record).unwrap();
        assert!(
            output.contains("京科968父本"),
            "Chinese characters must survive JSON serialization: {output}"
        );
        assert!(
            output.contains("京92"),
            "alias_name should contain Chinese: {output}"
        );
    }

    /// Verify that a list of records with Chinese characters serializes correctly.
    #[test]
    fn json_list_output_preserves_chinese_utf8() {
        let records = vec![
            json!({ "name": "京科968DH", "id": "SH000157" }),
            json!({ "name": "京科968母本", "id": "SH000022" }),
            json!({ "name": "迪卡父1", "id": "SH000001" }),
        ];

        let output = serde_json::to_string_pretty(&records).unwrap();
        assert!(output.contains("京科968DH"));
        assert!(output.contains("京科968母本"));
        assert!(output.contains("迪卡父1"));
    }

    #[test]
    fn pagination_metadata_includes_backend_page_fields() {
        let list = crate::api_response::PaginatedList {
            items: vec![json!({ "id": "SH000157" })],
            count: 14440,
            total_pages: Some(145),
            current_page: Some(1),
            has_next: Some(true),
            has_previous: Some(false),
        };

        let output = super::pagination_metadata_json(&list);

        assert_eq!(
            output,
            "{\n  \"count\": 14440,\n  \"total_pages\": 145,\n  \"current_page\": 1,\n  \"has_next\": true,\n  \"has_previous\": false\n}"
        );
    }

    #[test]
    fn unique_output_path_returns_missing_path_unchanged() {
        let path = std::path::PathBuf::from("definitely_missing_output_file.txt");
        assert_eq!(super::unique_output_path(&path), path);
    }
}

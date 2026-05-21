use colored::Colorize;
use serde::Serialize;

use crate::types::*;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn print_result<T: Serialize>(value: &T, _format: &OutputFormat) {
    let json = serde_json::to_string_pretty(value).unwrap_or_default();
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
            s.primer_name.as_deref().unwrap_or(""),
            s.remaining_quantity
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".into()),
            s.location_path.as_deref().unwrap_or(""),
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

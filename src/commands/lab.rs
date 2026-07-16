use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::ScientexClient;
use crate::config::Config;
use crate::output::{
    print_lab_members, print_order_brief, print_paginated_items, print_pagination_metadata,
    print_result, print_stocks, OutputFormat,
};

#[derive(Args)]
pub struct LabArgs {
    #[command(subcommand)]
    pub command: LabCommand,
}

#[derive(Subcommand)]
pub enum LabCommand {
    /// Show lab information.
    Info,
    /// Create a lab.
    Create { name: String },
    /// Update lab settings with a JSON object.
    Update { data: String },
    /// List all orders in my lab.
    Orders {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        order_type: Option<String>,
        #[arg(long)]
        supplier_name: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        price_min: Option<String>,
        #[arg(long)]
        price_max: Option<String>,
        #[arg(long)]
        date_from: Option<String>,
        #[arg(long)]
        date_to: Option<String>,
    },
    /// Show lab order statistics.
    OrdersStats {
        #[arg(long)]
        start_date: Option<String>,
        #[arg(long)]
        end_date: Option<String>,
    },
    /// List shared lab inventory.
    Inventory {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        location_id: Option<String>,
        #[arg(long)]
        low_stock: bool,
    },
    /// List lab members.
    Members,
    /// Update member role.
    UpdateRole { user_id: String, role: String },
    /// Remove a member.
    RemoveMember { user_id: String },
    /// Invite a member.
    Invite {
        email: String,
        #[arg(default_value = "member")]
        role: String,
    },
    /// List invitations.
    Invitations,
    /// Accept an invitation.
    AcceptInvite { invitation_id: String },
    /// Decline an invitation.
    DeclineInvite { invitation_id: String },
    /// Apply to join a lab.
    Join {
        lab_id: String,
        #[arg(default_value = "member")]
        role: String,
    },
    /// List join applications.
    Applications,
    /// Approve a join application.
    ApproveApp { application_id: String },
    /// Reject a join application.
    RejectApp { application_id: String },
    /// List approval rules.
    ApprovalRules,
    /// Add an approval rule from a JSON object.
    AddRule { data: String },
    /// Remove an approval rule.
    RemoveRule { rule_id: String },
}

pub async fn run(
    args: &LabArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;

    match &args.command {
        LabCommand::Info => {
            let lab = client.get_lab().await?;
            print_result(&lab, format);
        }
        LabCommand::Create { name } => {
            let lab = client.create_lab(name).await?;
            print_result(&lab, format);
        }
        LabCommand::Update { data } => {
            let data: serde_json::Value = serde_json::from_str(data)?;
            let lab = client.update_lab(&data).await?;
            print_result(&lab, format);
        }
        LabCommand::Orders {
            skip,
            limit,
            order_type,
            supplier_name,
            status,
            price_min,
            price_max,
            date_from,
            date_to,
        } => {
            let orders = client
                .list_lab_orders(
                    *skip,
                    *limit,
                    order_type.as_deref(),
                    supplier_name.as_deref(),
                    status.as_deref(),
                    price_min.as_deref(),
                    price_max.as_deref(),
                    date_from.as_deref(),
                    date_to.as_deref(),
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&orders, format),
                OutputFormat::Text => {
                    print_pagination_metadata(&orders);
                    if orders.items.is_empty() {
                        println!("No lab orders");
                    } else {
                        for order in &orders.items {
                            print_order_brief(order);
                        }
                    }
                }
            }
        }
        LabCommand::OrdersStats {
            start_date,
            end_date,
        } => {
            let stats = client
                .get_lab_order_stats(start_date.as_deref(), end_date.as_deref())
                .await?;
            print_result(&stats, format);
        }
        LabCommand::Inventory {
            skip,
            limit,
            name,
            location_id,
            low_stock,
        } => {
            let stocks = client
                .list_lab_inventory(
                    *skip,
                    *limit,
                    name.as_deref(),
                    location_id.as_deref(),
                    *low_stock,
                )
                .await?;
            match format {
                OutputFormat::Json => print_result(&stocks, format),
                OutputFormat::Text => {
                    print_pagination_metadata(&stocks);
                    print_stocks(&stocks.items);
                }
            }
        }
        LabCommand::Members => {
            let members = client.list_lab_members().await?;
            match format {
                OutputFormat::Json => print_result(&members, format),
                OutputFormat::Text => {
                    print_pagination_metadata(&members);
                    print_lab_members(&members.items);
                }
            }
        }
        LabCommand::UpdateRole { user_id, role } => {
            let result = client.update_member_role(user_id, role).await?;
            print_result(&result, format);
        }
        LabCommand::RemoveMember { user_id } => {
            let result = client.remove_member(user_id).await?;
            print_result(&result, format);
        }
        LabCommand::Invite { email, role } => {
            let result = client.invite_member(email, role).await?;
            print_result(&result, format);
        }
        LabCommand::Invitations => {
            let invitations = client.list_invitations().await?;
            match format {
                OutputFormat::Json => print_result(&invitations, format),
                OutputFormat::Text => print_paginated_items(&invitations),
            }
        }
        LabCommand::AcceptInvite { invitation_id } => {
            let result = client.accept_invitation(invitation_id).await?;
            print_result(&result, format);
        }
        LabCommand::DeclineInvite { invitation_id } => {
            let result = client.decline_invitation(invitation_id).await?;
            print_result(&result, format);
        }
        LabCommand::Join { lab_id, role } => {
            let result = client.apply_to_join_lab(lab_id, role).await?;
            print_result(&result, format);
        }
        LabCommand::Applications => {
            let applications = client.list_applications().await?;
            match format {
                OutputFormat::Json => print_result(&applications, format),
                OutputFormat::Text => print_paginated_items(&applications),
            }
        }
        LabCommand::ApproveApp { application_id } => {
            let result = client.approve_application(application_id).await?;
            print_result(&result, format);
        }
        LabCommand::RejectApp { application_id } => {
            let result = client.reject_application(application_id).await?;
            print_result(&result, format);
        }
        LabCommand::ApprovalRules => {
            let rules = client.list_approval_rules().await?;
            match format {
                OutputFormat::Json => print_result(&rules, format),
                OutputFormat::Text => print_paginated_items(&rules),
            }
        }
        LabCommand::AddRule { data } => {
            let data: serde_json::Value = serde_json::from_str(data)?;
            let rule = client.add_approval_rule(&data).await?;
            print_result(&rule, format);
        }
        LabCommand::RemoveRule { rule_id } => {
            let result = client.remove_approval_rule(rule_id).await?;
            print_result(&result, format);
        }
    }
    Ok(())
}

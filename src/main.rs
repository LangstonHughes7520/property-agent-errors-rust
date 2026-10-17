mod infrai_client;
mod property_agent;

use property_agent::{decide, InspectionReminder, MaintenanceRequest, TenantDocument};

#[tokio::main]
async fn main() -> Result<(), String> {
    let request = MaintenanceRequest { id: "maint-204".into(), unit: "4B".into(), summary: "Kitchen leak".into() };
    let document = TenantDocument { tenant_id: "tenant-18".into(), document_kind: "lease".into() };
    let reminder = InspectionReminder { unit: request.unit.clone(), due_in_days: 3 };
    let decision = decide(&request, Some(&document), &reminder);
    println!("{}: {:?}", request.id, decision);
    infrai_client::capture_failure(&reqwest::Client::new(), &request.id, "example agent exception")
        .await
        .map_err(|error| format!("{error:?}"))?;
    Ok(())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRequest {
    pub id: String,
    pub unit: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDocument {
    pub tenant_id: String,
    pub document_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionReminder {
    pub unit: String,
    pub due_in_days: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentDecision {
    ScheduleInspection,
    RequestDocument,
    QueueMaintenance,
}

pub fn decide(request: &MaintenanceRequest, document: Option<&TenantDocument>, reminder: &InspectionReminder) -> AgentDecision {
    if reminder.due_in_days <= 0 { return AgentDecision::ScheduleInspection; }
    if document.is_none() { return AgentDecision::RequestDocument; }
    let _ = request;
    AgentDecision::QueueMaintenance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overdue_unit_is_scheduled_before_other_work() {
        let request = MaintenanceRequest { id: "m-1".into(), unit: "4B".into(), summary: "leak".into() };
        let reminder = InspectionReminder { unit: "4B".into(), due_in_days: 0 };
        assert_eq!(decide(&request, None, &reminder), AgentDecision::ScheduleInspection);
    }
}


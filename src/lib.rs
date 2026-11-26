use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    AutoApproved,
    PendingReview,
    ManualApproved,
    Overridden,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePart {
    pub part_number: String,
    pub description: String,
    pub manufacturer: String,
    pub price_cents: u64,
    pub confidence_score: f64,
    pub is_oem: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairLine {
    pub line_id: String,
    pub damage_description: String,
    pub required_assembly: String,
    pub safety_critical: bool,
    pub candidate_parts: Vec<CandidatePart>,
    pub status: ReviewStatus,
    pub selected_part: Option<CandidatePart>,
    pub review_rationale: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let line = RepairLine {
            line_id: "LINE-101".to_string>,
            damage_description: "Front left bumper fascia tear".to_string(),
            required_assembly: "Front Bumper Cover".to_string(),
            safety_critical: false,
            candidate_parts: vec![],
            status: ReviewStatus::PendingReview,
            selected_part: None,
            review_rationale: None,
        };
        assert_eq!(line.status, ReviewStatus::PendingReview);
    }
}

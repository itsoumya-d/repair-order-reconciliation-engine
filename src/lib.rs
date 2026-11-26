use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewStatus {
    AutoApproved,
    PendingReview,
    ManualApproved,
    Overridden,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct ReconciliationPolicy {
    pub min_auto_approve_confidence: f64,
    pub require_oem_for_safety_critical: bool,
}

impl Default for ReconciliationPolicy {
    fn default() -> Self {
        Self {
            min_auto_approve_confidence: 0.95,
            require_oem_for_safety_critical: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManualDecision {
    Approve { part_number: String, rationale: String },
    Override { part_number: String, rationale: String },
    Reject { rationale: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestItem {
    pub line_id: String,
    pub part_number: String,
    pub description: String,
    pub manufacturer: String,
    pub price_cents: u64,
    pub confidence_score: f64,
    pub status: ReviewStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurchaseOrderManifest {
    pub order_id: String,
    pub items: Vec<ManifestItem>,
    pub total_cost_cents: u64,
    pub average_confidence: f64,
    pub auto_approved_count: usize,
    pub manual_reviewed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconciliationError {
    UnresolvedLines(Vec<String>),
    CandidateNotFound(String),
}

impl std::fmt::Display for ReconciliationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnresolvedLines(lines) => {
                write!(f, "Cannot generate manifest with unresolved lines: {:?}", lines)
            }
            Self::CandidateNotFound(part) => write!(f, "Candidate part not found: {}", part),
        }
    }
}

impl std::error::Error for ReconciliationError {}

pub struct ReconciliationEngine {
    policy: ReconciliationPolicy,
}

impl ReconciliationEngine {
    pub fn new(policy: ReconciliationPolicy) -> Self {
        Self { policy }
    }

    pub fn evaluate_line(&self, line: &mut RepairLine) {
        if line.candidate_parts.is_empty() {
            line.status = ReviewStatus::PendingReview;
            line.selected_part = None;
            line.review_rationale = Some("No compatible replacement candidate parts identified".to_string());
            return;
        }

        // Sort candidates by confidence descending
        line.candidate_parts.sort_by(|a, b| {
            b.confidence_score
                .partial_cmp(&a.confidence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let top_candidate = &line.candidate_parts[0];

        if line.safety_critical && self.policy.require_oem_for_safety_critical && !top_candidate.is_oem {
            line.status = ReviewStatus::PendingReview;
            line.selected_part = None;
            line.review_rationale = Some(
                "Safety-critical component requires OEM verification by human estimator".to_string(),
            );
            return;
        }

        if top_candidate.confidence_score >= self.policy.min_auto_approve_confidence {
            line.status = ReviewStatus::AutoApproved;
            line.selected_part = Some(top_candidate.clone());
            line.review_rationale = Some(format!(
                "Confidence {:.1}% exceeds auto-approval threshold ({:.1}%)",
                top_candidate.confidence_score * 100.0,
                self.policy.min_auto_approve_confidence * 100.0
            ));
        } else {
            line.status = ReviewStatus::PendingReview;
            line.selected_part = None;
            line.review_rationale = Some(format!(
                "Top match confidence ({:.1}%) is below auto-approve threshold ({:.1}%); human review required",
                top_candidate.confidence_score * 100.0,
                self.policy.min_auto_approve_confidence * 100.0
            ));
        }
    }

    pub fn apply_manual_decision(
        &self,
        line: &mut RepairLine,
        decision: ManualDecision,
    ) -> Result<(), ReconciliationError> {
        match decision {
            ManualDecision::Approve { part_number, rationale } => {
                let part = line
                    .candidate_parts
                    .iter()
                    .find(|c| c.part_number == part_number)
                    .cloned()
                    .ok_or_else(|| ReconciliationError::CandidateNotFound(part_number))?;
                line.status = ReviewStatus::ManualApproved;
                line.selected_part = Some(part);
                line.review_rationale = Some(rationale);
                Ok(())
            }
            ManualDecision::Override { part_number, rationale } => {
                let part = line
                    .candidate_parts
                    .iter()
                    .find(|c| c.part_number == part_number)
                    .cloned()
                    .ok_or_else(|| ReconciliationError::CandidateNotFound(part_number))?;
                line.status = ReviewStatus::Overridden;
                line.selected_part = Some(part);
                line.review_rationale = Some(rationale);
                Ok(())
            }
            ManualDecision::Reject { rationale } => {
                line.status = ReviewStatus::Rejected;
                line.selected_part = None;
                line.review_rationale = Some(rationale);
                Ok(())
            }
        }
    }

    pub fn generate_manifest(
        &self,
        order_id: &str,
        lines: &[RepairLine],
    ) -> Result<PurchaseOrderManifest, ReconciliationError> {
        let unresolved: Vec<String> = lines
            .iter()
            .filter(|l| l.status == ReviewStatus::PendingReview)
            .map(|l| l.line_id.clone())
            .collect();

        if !unresolved.is_empty() {
            return Err(ReconciliationError::UnresolvedLines(unresolved));
        }

        let mut items = Vec::new();
        let mut total_cost: u64 = 0;
        let mut sum_confidence = 0.0;
        let mut auto_count = 0;
        let mut manual_count = 0;

        for line in lines {
            if let Some(part) = &line.selected_part {
                total_cost += part.price_cents;
                sum_confidence += part.confidence_score;
                if line.status == ReviewStatus::AutoApproved {
                    auto_count += 1;
                } else {
                    manual_count += 1;
                }

                items.push(ManifestItem {
                    line_id: line.line_id.clone(),
                    part_number: part.part_number.clone(),
                    description: part.description.clone(),
                    manufacturer: part.manufacturer.clone(),
                    price_cents: part.price_cents,
                    confidence_score: part.confidence_score,
                    status: line.status.clone(),
                });
            }
        }

        let avg_confidence = if !items.is_empty() {
            sum_confidence / (items.len() as f64)
        } else {
            0.0
        };

        Ok(PurchaseOrderManifest {
            order_id: order_id.to_string(),
            items,
            total_cost_cents: total_cost,
            average_confidence: avg_confidence,
            auto_approved_count: auto_count,
            manual_reviewed_count: manual_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_candidates() -> Vec<CandidatePart> {
        vec![
            CandidatePart {
                part_number: "OEM-86511-G2000".to_string(),
                description: "Front Bumper Cover Fascia Primed".to_string(),
                manufacturer: "Hyundai".to_string(),
                price_cents: 38500,
                confidence_score: 0.98,
                is_oem: true,
            },
            CandidatePart {
                part_number: "CAPA-HY1000213".to_string(),
                description: "Front Bumper Cover CAPA Certified".to_string(),
                manufacturer: "Tong Yang".to_string(),
                price_cents: 21500,
                confidence_score: 0.92,
                is_oem: false,
            },
        ]
    }

    #[test]
    fn test_high_confidence_auto_approval() {
        let engine = ReconciliationEngine::new(ReconciliationPolicy::default());
        let mut line = RepairLine {
            line_id: "L-1".to_string(),
            damage_description: "Cracked front bumper cover".to_string(),
            required_assembly: "Front Bumper Fascia".to_string(),
            safety_critical: false,
            candidate_parts: sample_candidates(),
            status: ReviewStatus::PendingReview,
            selected_part: None,
            review_rationale: None,
        };

        engine.evaluate_line(&mut line);
        assert_eq!(line.status, ReviewStatus::AutoApproved);
        assert_eq!(line.selected_part.as_ref().unwrap().part_number, "OEM-86511-G2000");
    }

    #[test]
    fn test_safety_critical_requires_oem() {
        let engine = ReconciliationEngine::new(ReconciliationPolicy::default());
        let mut line = RepairLine {
            line_id: "L-2".to_string(),
            damage_description: "Front steering knuckle assembly deformation".to_string(),
            required_assembly: "Steering Knuckle LH".to_string(),
            safety_critical: true,
            candidate_parts: vec![CandidatePart {
                part_number: "AM-KNUCKLE-01".to_string(),
                description: "Aftermarket steering knuckle".to_string(),
                manufacturer: "GenericSteering".to_string(),
                price_cents: 14500,
                confidence_score: 0.99,
                is_oem: false,
            }],
            status: ReviewStatus::PendingReview,
            selected_part: None,
            review_rationale: None,
        };

        engine.evaluate_line(&mut line);
        assert_eq!(line.status, ReviewStatus::PendingReview);
        assert!(line.review_rationale.unwrap().contains("requires OEM verification"));
    }

    #[test]
    fn test_estimator_manual_override_and_manifest() {
        let engine = ReconciliationEngine::new(ReconciliationPolicy::default());
        let mut line = RepairLine {
            line_id: "L-3".to_string(),
            damage_description: "Deep scratch on left side mirror cover".to_string(),
            required_assembly: "Mirror Housing LH".to_string(),
            safety_critical: false,
            candidate_parts: sample_candidates(),
            status: ReviewStatus::PendingReview,
            selected_part: None,
            review_rationale: None,
        };

        engine.evaluate_line(&mut line);
        assert_eq!(line.status, ReviewStatus::AutoApproved);

        // Estimator elects aftermarket certified alternative to save cost
        let override_res = engine.apply_manual_decision(
            &mut line,
            ManualDecision::Override {
                part_number: "CAPA-HY1000213".to_string(),
                rationale: "Customer authorized CAPA certified cosmetic replacement".to_string(),
            },
        );
        assert!(override_res.is_ok());
        assert_eq!(line.status, ReviewStatus::Overridden);
        assert_eq!(line.selected_part.as_ref().unwrap().price_cents, 21500);

        let manifest = engine.generate_manifest("PO-98214", &[line]).unwrap();
        assert_eq!(manifest.order_id, "PO-98214");
        assert_eq!(manifest.items.len(), 1);
        assert_eq!(manifest.total_cost_cents, 21500);
        assert_eq!(manifest.manual_reviewed_count, 1);
        assert_eq!(manifest.auto_approved_count, 0);
    }

    #[test]
    fn test_unresolved_lines_fail_manifest_generation() {
        let engine = ReconciliationEngine::new(ReconciliationPolicy::default());
        let line = RepairLine {
            line_id: "L-UNRESOLVED".to_string(),
            damage_description: "Unknown structural bend".to_string(),
            required_assembly: "Subframe Assembly".to_string(),
            safety_critical: true,
            candidate_parts: vec![],
            status: ReviewStatus::PendingReview,
            selected_part: None,
            review_rationale: None,
        };

        let err = engine.generate_manifest("PO-ERR", &[line]).unwrap_err();
        match err {
            ReconciliationError::UnresolvedLines(ids) => {
                assert_eq!(ids, vec!["L-UNRESOLVED".to_string()]);
            }
            _ => panic!("Expected UnresolvedLines error"),
        }
    }
}

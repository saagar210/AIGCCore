use crate::redlineos::model::{SegmentedClause, RiskAssessment, ClauseAnchor};

const HIGH_RISK_KEYWORDS: &[&str] = &[
    "indemnify", "indemnification", "perpetual", "irrevocable",
    "sole discretion", "unrestricted", "terminate at will", "unlimited liability",
];

const MEDIUM_RISK_KEYWORDS: &[&str] = &[
    "limit liability", "limitation of liability", "liability", "breach", "default",
    "force majeure", "governing law", "dispute", "arbitration",
];

/// Assess risk level of a clause based on keyword matching
///
/// Returns HIGH, MEDIUM, or LOW based on keywords found
pub fn assess_clause_risk(
    clause: &SegmentedClause,
    anchor: &ClauseAnchor,
) -> RiskAssessment {
    let text_lower = clause.text.to_lowercase();
    let mut matched_high = Vec::new();
    let mut matched_medium = Vec::new();

    // Check for high-risk keywords
    for keyword in HIGH_RISK_KEYWORDS {
        if text_lower.contains(keyword) {
            matched_high.push(keyword.to_string());
        }
    }

    // Check for medium-risk keywords
    for keyword in MEDIUM_RISK_KEYWORDS {
        if text_lower.contains(keyword) {
            matched_medium.push(keyword.to_string());
        }
    }

    // Determine risk level
    let risk_level = if !matched_high.is_empty() {
        "HIGH"
    } else if !matched_medium.is_empty() {
        "MEDIUM"
    } else {
        "LOW"
    };

    // Build advisory message
    let all_matched = [&matched_high[..], &matched_medium[..]].concat();
    let advisory = if all_matched.is_empty() {
        "No significant risk keywords detected. Standard contract language.".to_string()
    } else {
        format!(
            "Risk level: {}. Keywords found: {}. Recommend legal review.",
            risk_level,
            all_matched.join(", "),
        )
    };

    RiskAssessment {
        anchor_id: anchor.anchor_id.clone(),
        risk_level: risk_level.to_string(),
        keywords_matched: all_matched,
        advisory,
        citations: vec![],  // Will be filled during narrative rendering
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_risk_detection() {
        let clause = SegmentedClause {
            clause_id: "c1_C0".to_string(),
            clause_number: None,
            title: None,
            text: "Licensee shall indemnify Licensor perpetually for all claims."
                .to_string(),
            start_page: 0,
            start_char_offset: 0,
            end_char_offset: 60,
            confidence: 0.9,
        };

        let anchor = ClauseAnchor {
            anchor_id: "REDLINE_c1_abc123_0".to_string(),
            clause_id: "c1_C0".to_string(),
            text_hash: "hash".to_string(),
            page_hint: None,
            char_offset_range: (0, 60),
        };

        let assessment = assess_clause_risk(&clause, &anchor);
        assert_eq!(assessment.risk_level, "HIGH");
        assert!(assessment.keywords_matched.contains(&"indemnify".to_string()));
    }

    #[test]
    fn test_medium_risk_detection() {
        let clause = SegmentedClause {
            clause_id: "c1_C1".to_string(),
            clause_number: None,
            title: None,
            text: "This clause limits liability under the law.".to_string(),
            start_page: 0,
            start_char_offset: 60,
            end_char_offset: 105,
            confidence: 0.9,
        };

        let anchor = ClauseAnchor {
            anchor_id: "REDLINE_c1_def456_60".to_string(),
            clause_id: "c1_C1".to_string(),
            text_hash: "hash2".to_string(),
            page_hint: None,
            char_offset_range: (60, 105),
        };

        let assessment = assess_clause_risk(&clause, &anchor);
        assert_eq!(assessment.risk_level, "MEDIUM");
    }

    #[test]
    fn test_low_risk_detection() {
        let clause = SegmentedClause {
            clause_id: "c1_C2".to_string(),
            clause_number: None,
            title: None,
            text: "This is a standard definition clause.".to_string(),
            start_page: 0,
            start_char_offset: 105,
            end_char_offset: 142,
            confidence: 0.9,
        };

        let anchor = ClauseAnchor {
            anchor_id: "REDLINE_c1_ghi789_105".to_string(),
            clause_id: "c1_C2".to_string(),
            text_hash: "hash3".to_string(),
            page_hint: None,
            char_offset_range: (105, 142),
        };

        let assessment = assess_clause_risk(&clause, &anchor);
        assert_eq!(assessment.risk_level, "LOW");
    }
}

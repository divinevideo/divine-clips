use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of fraud flag detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FraudFlagType {
    VelocitySpike,
    YoungAccount,
    SuspiciousPattern,
}

/// A fraud signal raised by one of the detection functions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudFlag {
    pub flag_type: FraudFlagType,
    pub message: String,
}

/// Check for a velocity spike: more than 10x growth within a 6-hour window.
///
/// Returns `Some(FraudFlag)` if the growth rate exceeds the threshold,
/// `None` otherwise.
pub fn check_velocity(
    prev_views: i64,
    current_views: i64,
    prev_time: DateTime<Utc>,
    current_time: DateTime<Utc>,
) -> Option<FraudFlag> {
    let duration = current_time.signed_duration_since(prev_time);
    let hours = duration.num_seconds() as f64 / 3600.0;

    // Only flag if the window is 6 hours or less
    if hours > 6.0 {
        return None;
    }

    // Avoid division by zero; treat zero prev_views with any growth as a spike
    if prev_views == 0 {
        if current_views > 0 {
            return Some(FraudFlag {
                flag_type: FraudFlagType::VelocitySpike,
                message: format!(
                    "Views jumped from 0 to {} within {:.1}h (infinite growth rate)",
                    current_views, hours
                ),
            });
        }
        return None;
    }

    let growth_factor = current_views as f64 / prev_views as f64;
    if growth_factor >= 10.0 {
        Some(FraudFlag {
            flag_type: FraudFlagType::VelocitySpike,
            message: format!(
                "Views grew {:.1}x in {:.1}h (threshold: 10x in 6h)",
                growth_factor, hours
            ),
        })
    } else {
        None
    }
}

/// Check for a suspicious bot-like pattern: all consecutive view-count deltas
/// are identical across at least 3 data points.
///
/// `view_history` is a slice of `(view_count, timestamp)` pairs ordered
/// chronologically.  Returns `Some(FraudFlag)` when the pattern is detected.
pub fn check_suspicious_pattern(
    view_history: &[(i64, DateTime<Utc>)],
) -> Option<FraudFlag> {
    // Need at least 3 data points to compute 2+ deltas
    if view_history.len() < 3 {
        return None;
    }

    let deltas: Vec<i64> = view_history
        .windows(2)
        .map(|w| w[1].0 - w[0].0)
        .collect();

    let first = deltas[0];
    let all_equal = deltas.iter().all(|&d| d == first);

    if all_equal {
        Some(FraudFlag {
            flag_type: FraudFlagType::SuspiciousPattern,
            message: format!(
                "All {} view deltas are identical ({}), indicating a bot pattern",
                deltas.len(),
                first
            ),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(hour: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(hour * 3600, 0).unwrap()
    }

    // --- check_velocity tests ---

    #[test]
    fn normal_growth_not_flagged() {
        // 2x growth in 3h — below the 10x threshold
        let flag = check_velocity(1_000, 2_000, ts(0), ts(3));
        assert!(flag.is_none());
    }

    #[test]
    fn ten_x_growth_in_6h_is_flagged() {
        // Exactly 10x in 5h
        let flag = check_velocity(1_000, 10_000, ts(0), ts(5));
        assert!(flag.is_some());
        assert_eq!(flag.unwrap().flag_type, FraudFlagType::VelocitySpike);
    }

    #[test]
    fn over_10x_growth_in_under_6h_is_flagged() {
        let flag = check_velocity(100, 5_000, ts(0), ts(2));
        assert!(flag.is_some());
    }

    #[test]
    fn over_6h_window_not_flagged_even_with_high_growth() {
        // 100x growth but over 7 hours — should not flag
        let flag = check_velocity(100, 10_000, ts(0), ts(7));
        assert!(flag.is_none());
    }

    #[test]
    fn exactly_6h_window_with_10x_growth_is_flagged() {
        let flag = check_velocity(1_000, 10_000, ts(0), ts(6));
        assert!(flag.is_some());
    }

    // --- check_suspicious_pattern tests ---

    #[test]
    fn exact_increment_pattern_detected() {
        let history = vec![
            (1_000, ts(0)),
            (2_000, ts(1)),
            (3_000, ts(2)),
            (4_000, ts(3)),
        ];
        let flag = check_suspicious_pattern(&history);
        assert!(flag.is_some());
        assert_eq!(flag.unwrap().flag_type, FraudFlagType::SuspiciousPattern);
    }

    #[test]
    fn fewer_than_3_data_points_not_flagged() {
        let history = vec![(1_000, ts(0)), (2_000, ts(1))];
        let flag = check_suspicious_pattern(&history);
        assert!(flag.is_none());
    }

    #[test]
    fn single_data_point_not_flagged() {
        let history = vec![(1_000, ts(0))];
        let flag = check_suspicious_pattern(&history);
        assert!(flag.is_none());
    }

    #[test]
    fn empty_history_not_flagged() {
        let flag = check_suspicious_pattern(&[]);
        assert!(flag.is_none());
    }

    #[test]
    fn mixed_deltas_not_flagged() {
        let history = vec![
            (1_000, ts(0)),
            (1_500, ts(1)), // delta 500
            (3_000, ts(2)), // delta 1500
            (4_200, ts(3)), // delta 1200
        ];
        let flag = check_suspicious_pattern(&history);
        assert!(flag.is_none());
    }
}

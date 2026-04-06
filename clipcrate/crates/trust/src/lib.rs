// clipcrate-trust: Graduated trust system and fraud detection
// Implements the tiered trust model for content creators and campaign sponsors,
// including rate limiting, submission scoring, and fraud signal aggregation.

pub mod fraud;

/// Calculate trust level based on total verified views and account age.
///
/// - Level 1 (new): default
/// - Level 2: requires >= 100,000 views AND >= 30 days
/// - Level 3: requires >= 1,000,000 views AND >= 90 days
pub fn calculate_trust_level(total_verified_views: i64, account_age_days: i64) -> i32 {
    if total_verified_views >= 1_000_000 && account_age_days >= 90 {
        3
    } else if total_verified_views >= 100_000 && account_age_days >= 30 {
        2
    } else {
        1
    }
}

/// Maximum weekly views allowed per trust level.
///
/// - Level 1: 50,000
/// - Level 2: 500,000
/// - Level 3: i64::MAX (unlimited)
pub fn max_weekly_views(trust_level: i32) -> i64 {
    match trust_level {
        1 => 50_000,
        2 => 500_000,
        _ => i64::MAX,
    }
}

/// Maximum number of active submissions per trust level.
///
/// - Level 1: 5
/// - Level 2: 20
/// - Level 3: i32::MAX (unlimited)
pub fn max_active_submissions(trust_level: i32) -> i32 {
    match trust_level {
        1 => 5,
        2 => 20,
        _ => i32::MAX,
    }
}

/// Payout hold in hours per trust level.
///
/// - Level 1: 48 hours
/// - Level 2: 24 hours
/// - Level 3: 0 (instant)
pub fn payout_hold_hours(trust_level: i32) -> i64 {
    match trust_level {
        1 => 48,
        2 => 24,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- calculate_trust_level tests ---

    #[test]
    fn new_user_gets_level_1() {
        assert_eq!(calculate_trust_level(0, 0), 1);
    }

    #[test]
    fn level_2_requires_both_views_and_days() {
        // Has views but not days
        assert_eq!(calculate_trust_level(100_000, 29), 1);
        // Has days but not views
        assert_eq!(calculate_trust_level(99_999, 30), 1);
        // Has both
        assert_eq!(calculate_trust_level(100_000, 30), 2);
    }

    #[test]
    fn level_3_requires_both_views_and_days() {
        // Has views but not days
        assert_eq!(calculate_trust_level(1_000_000, 89), 2);
        // Has days but not views
        assert_eq!(calculate_trust_level(999_999, 90), 2);
        // Has both
        assert_eq!(calculate_trust_level(1_000_000, 90), 3);
    }

    #[test]
    fn boundary_99999_views_stays_level_1() {
        assert_eq!(calculate_trust_level(99_999, 30), 1);
    }

    #[test]
    fn boundary_100000_views_and_30_days_is_level_2() {
        assert_eq!(calculate_trust_level(100_000, 30), 2);
    }

    // --- max_weekly_views tests ---

    #[test]
    fn max_weekly_views_level_1() {
        assert_eq!(max_weekly_views(1), 50_000);
    }

    #[test]
    fn max_weekly_views_level_2() {
        assert_eq!(max_weekly_views(2), 500_000);
    }

    #[test]
    fn max_weekly_views_level_3_is_unlimited() {
        assert_eq!(max_weekly_views(3), i64::MAX);
    }

    // --- max_active_submissions tests ---

    #[test]
    fn max_active_submissions_level_1() {
        assert_eq!(max_active_submissions(1), 5);
    }

    #[test]
    fn max_active_submissions_level_2() {
        assert_eq!(max_active_submissions(2), 20);
    }

    #[test]
    fn max_active_submissions_level_3_is_unlimited() {
        assert_eq!(max_active_submissions(3), i32::MAX);
    }

    // --- payout_hold_hours tests ---

    #[test]
    fn payout_hold_hours_level_1() {
        assert_eq!(payout_hold_hours(1), 48);
    }

    #[test]
    fn payout_hold_hours_level_2() {
        assert_eq!(payout_hold_hours(2), 24);
    }

    #[test]
    fn payout_hold_hours_level_3_is_instant() {
        assert_eq!(payout_hold_hours(3), 0);
    }
}

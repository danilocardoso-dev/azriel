use chrono::{DateTime, Duration, Utc};

pub const SCHEDULER_VERSION: &str = "STUDY_REVIEW_SCHEDULER_V1";
const AGAIN_SECONDS: i64 = 10 * 60;
const HARD_MIN_SECONDS: i64 = 24 * 60 * 60;
const GOOD_MIN_SECONDS: i64 = 3 * 24 * 60 * 60;
const EASY_MIN_SECONDS: i64 = 7 * 24 * 60 * 60;
const MAX_INTERVAL_SECONDS: i64 = 365 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleOutcome {
    pub next_due_at: DateTime<Utc>,
    pub next_interval_seconds: i64,
    pub correct_increment: i64,
    pub incorrect_increment: i64,
}

pub fn schedule(
    previous_interval_seconds: i64,
    result: &str,
    now: DateTime<Utc>,
) -> Result<ScheduleOutcome, String> {
    let previous = previous_interval_seconds.max(0);
    let (seconds, correct, incorrect) = match result {
        "AGAIN" => (AGAIN_SECONDS, 0, 1),
        "HARD" => (scaled(previous, 3, 2, HARD_MIN_SECONDS), 1, 0),
        "GOOD" => (scaled(previous, 5, 2, GOOD_MIN_SECONDS), 1, 0),
        "EASY" => (scaled(previous, 4, 1, EASY_MIN_SECONDS), 1, 0),
        _ => return Err("Resultado de revisão inválido".into()),
    };
    Ok(ScheduleOutcome {
        next_due_at: now + Duration::seconds(seconds),
        next_interval_seconds: seconds,
        correct_increment: correct,
        incorrect_increment: incorrect,
    })
}

fn scaled(previous: i64, numerator: i64, denominator: i64, minimum: i64) -> i64 {
    let grown = previous.saturating_mul(numerator) / denominator;
    grown.max(minimum).min(MAX_INTERVAL_SECONDS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn clock() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 25, 12, 0, 0).unwrap()
    }

    #[test]
    fn new_card_results_are_deterministic_and_versioned() {
        let now = clock();
        let again = schedule(0, "AGAIN", now).unwrap();
        let hard = schedule(0, "HARD", now).unwrap();
        let good = schedule(0, "GOOD", now).unwrap();
        let easy = schedule(0, "EASY", now).unwrap();
        assert_eq!(SCHEDULER_VERSION, "STUDY_REVIEW_SCHEDULER_V1");
        assert_eq!(again.next_interval_seconds, 600);
        assert_eq!(hard.next_interval_seconds, 86_400);
        assert_eq!(good.next_interval_seconds, 259_200);
        assert_eq!(easy.next_interval_seconds, 604_800);
        assert_eq!(again.next_due_at, now + Duration::minutes(10));
    }

    #[test]
    fn intervals_progress_without_exceeding_the_cap() {
        let now = clock();
        assert_eq!(
            schedule(86_400, "HARD", now).unwrap().next_interval_seconds,
            129_600
        );
        assert_eq!(
            schedule(259_200, "GOOD", now)
                .unwrap()
                .next_interval_seconds,
            648_000
        );
        assert_eq!(
            schedule(604_800, "EASY", now)
                .unwrap()
                .next_interval_seconds,
            2_419_200
        );
        assert_eq!(
            schedule(i64::MAX, "EASY", now)
                .unwrap()
                .next_interval_seconds,
            31_536_000
        );
    }

    #[test]
    fn only_again_counts_as_incorrect() {
        let now = clock();
        assert_eq!(
            (
                schedule(0, "AGAIN", now).unwrap().correct_increment,
                schedule(0, "AGAIN", now).unwrap().incorrect_increment
            ),
            (0, 1)
        );
        for result in ["HARD", "GOOD", "EASY"] {
            let outcome = schedule(0, result, now).unwrap();
            assert_eq!(
                (outcome.correct_increment, outcome.incorrect_increment),
                (1, 0)
            );
        }
        assert!(schedule(0, "UNKNOWN", now).is_err());
    }
}

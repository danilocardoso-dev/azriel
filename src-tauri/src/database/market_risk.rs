use super::market_models::MarketRiskProfile;
use serde::{Deserialize, Serialize};

pub const RISK_POLICY_VERSION: &str = "RISK_POLICY_V2";
const EXPOSURE_EPSILON: f64 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExposureChangeClass {
    RiskIncreasing,
    RiskReducing,
    RiskNeutral,
}

impl ExposureChangeClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RiskIncreasing => "RISK_INCREASING",
            Self::RiskReducing => "RISK_REDUCING",
            Self::RiskNeutral => "RISK_NEUTRAL",
        }
    }
}

pub fn classify_exposure_change(current: f64, target: f64) -> ExposureChangeClass {
    let delta = target - current;
    if delta > EXPOSURE_EPSILON {
        ExposureChangeClass::RiskIncreasing
    } else if delta < -EXPOSURE_EPSILON {
        ExposureChangeClass::RiskReducing
    } else {
        ExposureChangeClass::RiskNeutral
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskRuleStatus {
    Passed,
    Rejected,
    NotApplicable,
}

impl RiskRuleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "PASSED",
            Self::Rejected => "REJECTED",
            Self::NotApplicable => "NOT_APPLICABLE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskRuleEvaluation {
    pub rule: String,
    pub status: RiskRuleStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskEvaluation {
    pub result: String,
    pub reason: String,
    pub approved_position_pct: Option<f64>,
    pub classification: ExposureChangeClass,
    pub current_exposure_pct: f64,
    pub target_exposure_pct: f64,
    pub exposure_delta_pct: f64,
    pub policy_version: String,
    pub rules: Vec<RiskRuleEvaluation>,
}

pub struct RiskEvaluationInput<'a> {
    pub current_exposure_pct: f64,
    pub target_exposure_pct: f64,
    pub operation_count: usize,
    pub daily_loss_pct: f64,
    pub drawdown_pct: f64,
    pub asset_allowed: bool,
    pub position_allowed: bool,
    pub kill_switch_active: bool,
    pub hold_without_order: bool,
    pub profile: &'a MarketRiskProfile,
}

fn rule(rule: &str, status: RiskRuleStatus, reason: Option<&str>) -> RiskRuleEvaluation {
    RiskRuleEvaluation {
        rule: rule.into(),
        status,
        reason: reason.map(str::to_string),
    }
}

fn reduction_reason(classification: ExposureChangeClass) -> &'static str {
    match classification {
        ExposureChangeClass::RiskReducing => "RISK_REDUCING_ACTION",
        ExposureChangeClass::RiskNeutral => "RISK_NEUTRAL_ACTION",
        ExposureChangeClass::RiskIncreasing => "",
    }
}

pub fn evaluate(input: RiskEvaluationInput<'_>) -> RiskEvaluation {
    let current = input.current_exposure_pct.clamp(0.0, 100.0);
    let target = input.target_exposure_pct.clamp(0.0, 100.0);
    let classification = classify_exposure_change(current, target);
    let increasing = classification == ExposureChangeClass::RiskIncreasing;
    let position_limit_already_exceeded =
        increasing && current > input.profile.max_position_pct + EXPOSURE_EPSILON;
    let exposure_limit_already_exceeded =
        increasing && current > input.profile.max_total_exposure_pct + EXPOSURE_EPSILON;
    let mut approved = target;
    let mut rules = vec![rule(
        "POSITION_SEMANTICS",
        if input.position_allowed {
            RiskRuleStatus::Passed
        } else {
            RiskRuleStatus::Rejected
        },
        (!input.position_allowed).then_some("INVALID_POSITION_ACTION"),
    )];
    rules.push(rule(
        "ASSET_ALLOWLIST",
        if input.asset_allowed {
            RiskRuleStatus::Passed
        } else {
            RiskRuleStatus::Rejected
        },
        (!input.asset_allowed).then_some("ASSET_NOT_ALLOWED"),
    ));

    if increasing {
        if position_limit_already_exceeded {
            rules.push(rule(
                "MAX_POSITION",
                RiskRuleStatus::Rejected,
                Some("CURRENT_EXPOSURE_ABOVE_MAX_POSITION"),
            ));
        } else if approved > input.profile.max_position_pct {
            approved = input.profile.max_position_pct;
            rules.push(rule(
                "MAX_POSITION",
                RiskRuleStatus::Passed,
                Some("TARGET_CAPPED_TO_MAX_POSITION"),
            ));
        } else {
            rules.push(rule("MAX_POSITION", RiskRuleStatus::Passed, None));
        }
        if exposure_limit_already_exceeded {
            rules.push(rule(
                "MAX_EXPOSURE",
                RiskRuleStatus::Rejected,
                Some("CURRENT_EXPOSURE_ABOVE_MAX_EXPOSURE"),
            ));
        } else if approved > input.profile.max_total_exposure_pct {
            approved = input.profile.max_total_exposure_pct;
            rules.push(rule(
                "MAX_EXPOSURE",
                RiskRuleStatus::Passed,
                Some("TARGET_CAPPED_TO_MAX_EXPOSURE"),
            ));
        } else {
            rules.push(rule("MAX_EXPOSURE", RiskRuleStatus::Passed, None));
        }
    } else {
        let reason = reduction_reason(classification);
        rules.push(rule(
            "MAX_POSITION",
            RiskRuleStatus::NotApplicable,
            Some(reason),
        ));
        rules.push(rule(
            "MAX_EXPOSURE",
            RiskRuleStatus::NotApplicable,
            Some(reason),
        ));
    }

    let max_operations_reached = input
        .profile
        .max_trades_per_day
        .is_some_and(|max| input.operation_count >= max);
    rules.push(if increasing && max_operations_reached {
        rule(
            "MAX_OPERATIONS",
            RiskRuleStatus::Rejected,
            Some("MAX_OPERATIONS_REACHED"),
        )
    } else if increasing {
        rule("MAX_OPERATIONS", RiskRuleStatus::Passed, None)
    } else {
        rule(
            "MAX_OPERATIONS",
            RiskRuleStatus::NotApplicable,
            Some(reduction_reason(classification)),
        )
    });

    rules.push(if increasing && input.daily_loss_pct >= input.profile.max_daily_loss_pct {
        rule(
            "MAX_DAILY_LOSS",
            RiskRuleStatus::Rejected,
            Some("MAX_DAILY_LOSS_REACHED"),
        )
    } else if increasing {
        rule("MAX_DAILY_LOSS", RiskRuleStatus::Passed, None)
    } else {
        rule(
            "MAX_DAILY_LOSS",
            RiskRuleStatus::NotApplicable,
            Some(reduction_reason(classification)),
        )
    });

    rules.push(if increasing && input.drawdown_pct >= input.profile.max_drawdown_pct {
        rule(
            "MAX_DRAWDOWN",
            RiskRuleStatus::Rejected,
            Some("MAX_DRAWDOWN_REACHED"),
        )
    } else if increasing {
        rule("MAX_DRAWDOWN", RiskRuleStatus::Passed, None)
    } else {
        rule(
            "MAX_DRAWDOWN",
            RiskRuleStatus::NotApplicable,
            Some(reduction_reason(classification)),
        )
    });

    rules.push(rule(
        "KILL_SWITCH",
        if input.kill_switch_active {
            RiskRuleStatus::Rejected
        } else {
            RiskRuleStatus::Passed
        },
        input
            .kill_switch_active
            .then_some("BLOCK_ALL_INCLUDING_RISK_REDUCTION"),
    ));

    let rejection = if input.kill_switch_active {
        Some("Kill switch ativo")
    } else if !input.position_allowed {
        Some("SELL sem posição rejeitado; short está desabilitado")
    } else if !input.asset_allowed {
        Some("Ativo fora da allowlist")
    } else if position_limit_already_exceeded {
        Some("Exposição atual acima da posição máxima; aumento rejeitado")
    } else if exposure_limit_already_exceeded {
        Some("Exposição atual acima do limite total; aumento rejeitado")
    } else if increasing && max_operations_reached {
        Some("Limite de operações atingido")
    } else if increasing && input.daily_loss_pct >= input.profile.max_daily_loss_pct {
        Some("Perda diária máxima atingida")
    } else if increasing && input.drawdown_pct >= input.profile.max_drawdown_pct {
        Some("Drawdown máximo atingido")
    } else {
        None
    };

    let (result, reason, approved_position_pct) = if let Some(reason) = rejection {
        ("REJECTED".into(), reason.into(), None)
    } else if input.hold_without_order {
        ("APPROVED".into(), "HOLD não gera ordem".into(), None)
    } else if (approved - target).abs() > EXPOSURE_EPSILON {
        (
            "MODIFIED".into(),
            format!("Exposição limitada a {:.2}%", approved),
            Some(approved),
        )
    } else {
        (
            "APPROVED".into(),
            "Dentro da política de risco aplicável".into(),
            Some(approved),
        )
    };

    RiskEvaluation {
        result,
        reason,
        approved_position_pct,
        classification,
        current_exposure_pct: current,
        target_exposure_pct: target,
        exposure_delta_pct: target - current,
        policy_version: RISK_POLICY_VERSION.into(),
        rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(max_operations: Option<usize>) -> MarketRiskProfile {
        MarketRiskProfile {
            id: "risk".into(),
            name: "Risk".into(),
            max_position_pct: 50.0,
            max_total_exposure_pct: 50.0,
            max_daily_loss_pct: 5.0,
            max_drawdown_pct: 20.0,
            max_trades_per_day: max_operations,
            allow_leverage: false,
            allow_short: false,
            allowed_assets: vec![],
        }
    }

    fn evaluation(current: f64, target: f64, operations: usize) -> RiskEvaluation {
        let profile = profile(Some(operations));
        evaluate(RiskEvaluationInput {
            current_exposure_pct: current,
            target_exposure_pct: target,
            operation_count: operations,
            daily_loss_pct: 0.0,
            drawdown_pct: 0.0,
            asset_allowed: true,
            position_allowed: true,
            kill_switch_active: false,
            hold_without_order: (current - target).abs() <= EXPOSURE_EPSILON,
            profile: &profile,
        })
    }

    fn status(result: &RiskEvaluation, name: &str) -> RiskRuleStatus {
        result.rules.iter().find(|item| item.rule == name).unwrap().status
    }

    #[test]
    fn exposure_classification_uses_current_target_and_tolerance() {
        assert_eq!(classify_exposure_change(0.0, 25.0), ExposureChangeClass::RiskIncreasing);
        assert_eq!(classify_exposure_change(25.0, 35.0), ExposureChangeClass::RiskIncreasing);
        assert_eq!(classify_exposure_change(40.0, 25.0), ExposureChangeClass::RiskReducing);
        assert_eq!(classify_exposure_change(40.0, 0.0), ExposureChangeClass::RiskReducing);
        assert_eq!(classify_exposure_change(40.0, 40.0), ExposureChangeClass::RiskNeutral);
        assert_eq!(classify_exposure_change(40.0, 40.00005), ExposureChangeClass::RiskNeutral);
    }

    #[test]
    fn max_operations_blocks_only_increasing_risk() {
        let entry = evaluation(0.0, 25.0, 3);
        let increase = evaluation(25.0, 35.0, 3);
        let neutral = evaluation(40.0, 40.0, 3);
        let reduce = evaluation(39.6966, 24.6966, 3);
        let exit = evaluation(45.0, 0.0, 3);
        for rejected in [&entry, &increase] {
            assert_eq!(rejected.result, "REJECTED");
            assert_eq!(status(rejected, "MAX_OPERATIONS"), RiskRuleStatus::Rejected);
        }
        for allowed in [&neutral, &reduce, &exit] {
            assert_ne!(allowed.result, "REJECTED");
            assert_eq!(status(allowed, "MAX_OPERATIONS"), RiskRuleStatus::NotApplicable);
        }
    }

    #[test]
    fn position_exposure_loss_and_drawdown_rules_allow_reduction() {
        let profile = profile(None);
        for (current, target) in [(70.0, 60.0), (40.0, 0.0)] {
            let result = evaluate(RiskEvaluationInput {
                current_exposure_pct: current,
                target_exposure_pct: target,
                operation_count: 0,
                daily_loss_pct: 10.0,
                drawdown_pct: 30.0,
                asset_allowed: true,
                position_allowed: true,
                kill_switch_active: false,
                hold_without_order: false,
                profile: &profile,
            });
            assert_eq!(result.result, "APPROVED");
            for name in ["MAX_POSITION", "MAX_EXPOSURE", "MAX_DAILY_LOSS", "MAX_DRAWDOWN"] {
                assert_eq!(status(&result, name), RiskRuleStatus::NotApplicable);
            }
        }
        let increasing = evaluate(RiskEvaluationInput {
            current_exposure_pct: 20.0,
            target_exposure_pct: 80.0,
            operation_count: 0,
            daily_loss_pct: 10.0,
            drawdown_pct: 30.0,
            asset_allowed: true,
            position_allowed: true,
            kill_switch_active: false,
            hold_without_order: false,
            profile: &profile,
        });
        assert_eq!(increasing.result, "REJECTED");
        assert_eq!(status(&increasing, "MAX_POSITION"), RiskRuleStatus::Passed);
        assert_eq!(status(&increasing, "MAX_DAILY_LOSS"), RiskRuleStatus::Rejected);
        assert_eq!(status(&increasing, "MAX_DRAWDOWN"), RiskRuleStatus::Rejected);
    }

    #[test]
    fn max_position_and_exposure_still_cap_increasing_risk() {
        let profile = profile(None);
        let result = evaluate(RiskEvaluationInput {
            current_exposure_pct: 0.0,
            target_exposure_pct: 90.0,
            operation_count: 0,
            daily_loss_pct: 0.0,
            drawdown_pct: 0.0,
            asset_allowed: true,
            position_allowed: true,
            kill_switch_active: false,
            hold_without_order: false,
            profile: &profile,
        });
        assert_eq!(result.result, "MODIFIED");
        assert_eq!(result.approved_position_pct, Some(50.0));

        let already_over_limit = evaluate(RiskEvaluationInput {
            current_exposure_pct: 70.0,
            target_exposure_pct: 80.0,
            operation_count: 0,
            daily_loss_pct: 0.0,
            drawdown_pct: 0.0,
            asset_allowed: true,
            position_allowed: true,
            kill_switch_active: false,
            hold_without_order: false,
            profile: &profile,
        });
        assert_eq!(already_over_limit.result, "REJECTED");
        assert_eq!(already_over_limit.approved_position_pct, None);
    }

    #[test]
    fn kill_switch_preserves_block_all_policy() {
        let profile = profile(None);
        for (current, target) in [(0.0, 25.0), (40.0, 25.0), (40.0, 0.0)] {
            let result = evaluate(RiskEvaluationInput {
                current_exposure_pct: current,
                target_exposure_pct: target,
                operation_count: 0,
                daily_loss_pct: 0.0,
                drawdown_pct: 0.0,
                asset_allowed: true,
                position_allowed: true,
                kill_switch_active: true,
                hold_without_order: false,
                profile: &profile,
            });
            assert_eq!(result.result, "REJECTED");
            assert_eq!(status(&result, "KILL_SWITCH"), RiskRuleStatus::Rejected);
        }
    }
}

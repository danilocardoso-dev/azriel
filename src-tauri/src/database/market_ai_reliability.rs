use super::market_positions::{self, LifecycleAction, PositionState};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::str::FromStr;

pub const SLOW_CALL_THRESHOLD_MS: u64 = 5_000;
pub const REASON_CODES: &[&str] = &[
    "ALIGNED_BULLISH_SIGNAL",
    "ALIGNED_BEARISH_SIGNAL",
    "TREND_DETERIORATION",
    "MOMENTUM_DETERIORATION",
    "REGIME_REVERSAL",
    "POSITION_ALREADY_OPTIMAL",
    "INSUFFICIENT_SIGNAL",
    "CONFLICTING_SIGNALS",
    "REDUCE_RISK",
    "EXIT_SIGNAL",
    "REENTRY_SIGNAL",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvalidOutputType {
    InvalidJson,
    MissingField,
    InvalidAction,
    InvalidReasonCode,
    InvalidConfidence,
    InvalidTargetExposure,
    InvalidPositionAction,
    InvalidIntent,
    InvalidConfidencePct,
    InvalidPositionIntent,
    PositionContextError,
    SchemaTypeError,
    ExtraText,
    EmptyResponse,
    ProviderError,
    UnknownInvalid,
}

impl InvalidOutputType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidJson => "INVALID_JSON",
            Self::MissingField => "MISSING_FIELD",
            Self::InvalidAction => "INVALID_ACTION",
            Self::InvalidReasonCode => "INVALID_REASON_CODE",
            Self::InvalidConfidence => "INVALID_CONFIDENCE",
            Self::InvalidTargetExposure => "INVALID_TARGET_EXPOSURE",
            Self::InvalidPositionAction => "INVALID_POSITION_ACTION",
            Self::InvalidIntent => "INVALID_INTENT",
            Self::InvalidConfidencePct => "INVALID_CONFIDENCE_PCT",
            Self::InvalidPositionIntent => "INVALID_POSITION_INTENT",
            Self::PositionContextError => "POSITION_CONTEXT_ERROR",
            Self::SchemaTypeError => "SCHEMA_TYPE_ERROR",
            Self::ExtraText => "EXTRA_TEXT",
            Self::EmptyResponse => "EMPTY_RESPONSE",
            Self::ProviderError => "PROVIDER_ERROR",
            Self::UnknownInvalid => "UNKNOWN_INVALID",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DecisionValidationStage {
    pub stage: String,
    pub success: bool,
    pub error_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionAttemptAudit {
    pub attempt_number: usize,
    pub raw_response: Option<String>,
    pub status: String,
    pub error_type: Option<String>,
    pub error_field: Option<String>,
    pub error_value: Option<String>,
    pub error_message: Option<String>,
    pub normalized_from_wrapped_json: bool,
    pub latency_ms: u64,
    pub stages: Vec<DecisionValidationStage>,
}

pub fn audit_error_value(input: &str, field: Option<&str>) -> Option<String> {
    let field = field?;
    let (object, _) = extract_single_object(input).ok()?;
    let value: Value = serde_json::from_str(&object).ok()?;
    value.get(field).map(Value::to_string)
}

#[derive(Debug, Clone)]
pub struct StructuredDecisionV41 {
    pub action: LifecycleAction,
    pub target_exposure_pct: f64,
    pub confidence: f64,
    pub reason_code: String,
    pub reason: String,
    pub normalized_from_wrapped_json: bool,
    pub stages: Vec<DecisionValidationStage>,
}

#[derive(Debug, Clone)]
pub struct StructuredDecisionV42 {
    pub intent: market_positions::PositionIntent,
    pub confidence: f64,
    pub reason_code: String,
    pub reason: String,
    pub normalized_from_wrapped_json: bool,
    pub stages: Vec<DecisionValidationStage>,
}

#[derive(Debug, Clone)]
pub struct ValidationFailure {
    pub error_type: InvalidOutputType,
    pub error_field: Option<String>,
    pub message: String,
    pub normalized_from_wrapped_json: bool,
    pub stages: Vec<DecisionValidationStage>,
}

fn pass(stage: &str) -> DecisionValidationStage {
    DecisionValidationStage {
        stage: stage.into(),
        success: true,
        error_code: None,
        message: None,
    }
}

fn failure(
    mut stages: Vec<DecisionValidationStage>,
    stage: &str,
    error_type: InvalidOutputType,
    field: Option<&str>,
    message: impl Into<String>,
    normalized: bool,
) -> ValidationFailure {
    let message = message.into();
    stages.push(DecisionValidationStage {
        stage: stage.into(),
        success: false,
        error_code: Some(error_type.as_str().into()),
        message: Some(message.clone()),
    });
    ValidationFailure {
        error_type,
        error_field: field.map(str::to_string),
        message,
        normalized_from_wrapped_json: normalized,
        stages,
    }
}

pub fn output_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "action": { "type": "string", "enum": ["ENTER_LONG", "STAY_FLAT", "HOLD_POSITION", "INCREASE_LONG", "REDUCE_LONG", "EXIT_LONG"] },
            "target_exposure_pct": { "type": "number", "minimum": 0, "maximum": 100 },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "reason_code": { "type": "string", "enum": REASON_CODES },
            "reason": { "type": "string", "minLength": 1, "maxLength": 240 }
        },
        "required": ["action", "target_exposure_pct", "confidence", "reason_code", "reason"]
    })
}

pub fn output_schema_v4_2() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "intent": { "type": "string", "enum": ["ENTER", "HOLD", "REDUCE", "EXIT"] },
            "confidence_pct": { "type": "number", "minimum": 0, "maximum": 100 },
            "reason_code": { "type": "string", "enum": REASON_CODES },
            "reason": { "type": "string", "minLength": 1, "maxLength": 240 }
        },
        "required": ["intent", "confidence_pct", "reason_code", "reason"]
    })
}

pub fn retry_instruction(previous_error: InvalidOutputType) -> String {
    format!(
        "Your previous response failed with {}. Return only one valid JSON object matching the required schema.",
        previous_error.as_str()
    )
}

fn extract_single_object(input: &str) -> Result<(String, bool), ValidationFailure> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(failure(
            Vec::new(),
            "NORMALIZER",
            InvalidOutputType::EmptyResponse,
            None,
            "O provider retornou uma resposta vazia",
            false,
        ));
    }
    if serde_json::from_str::<Value>(trimmed).is_ok() {
        return Ok((trimmed.into(), false));
    }

    let mut spans = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in trimmed.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        if character == '"' {
            quoted = true;
        } else if character == '{' {
            if depth == 0 {
                start = Some(index);
            }
            depth += 1;
        } else if character == '}' {
            if depth == 0 {
                return Err(failure(Vec::new(), "NORMALIZER", InvalidOutputType::InvalidJson, None, "Chave de fechamento sem objeto correspondente", false));
            }
            depth -= 1;
            if depth == 0 {
                if let Some(begin) = start.take() {
                    spans.push((begin, index + 1));
                }
            }
        }
    }
    if depth != 0 || quoted {
        return Err(failure(Vec::new(), "NORMALIZER", InvalidOutputType::InvalidJson, None, "Objeto JSON incompleto", false));
    }
    if spans.len() > 1 {
        return Err(failure(Vec::new(), "NORMALIZER", InvalidOutputType::ExtraText, None, "A resposta contém mais de um objeto JSON", false));
    }
    let Some((begin, end)) = spans.first().copied() else {
        return Err(failure(Vec::new(), "NORMALIZER", InvalidOutputType::ExtraText, None, "Nenhum objeto JSON inequívoco foi encontrado", false));
    };
    let candidate = &trimmed[begin..end];
    serde_json::from_str::<Value>(candidate).map_err(|_| failure(Vec::new(), "NORMALIZER", InvalidOutputType::InvalidJson, None, "O objeto encontrado não é JSON válido", true))?;
    Ok((candidate.into(), true))
}

fn required<'a>(object: &'a Map<String, Value>, field: &str, stages: &[DecisionValidationStage], normalized: bool) -> Result<&'a Value, ValidationFailure> {
    object.get(field).ok_or_else(|| failure(stages.to_vec(), "SCHEMA_VALIDATOR", InvalidOutputType::MissingField, Some(field), format!("Campo obrigatório ausente: {field}"), normalized))
}

pub fn validate(
    input: &str,
    position_state: PositionState,
    current_exposure_pct: f64,
) -> Result<StructuredDecisionV41, ValidationFailure> {
    let (normalized_json, normalized) = extract_single_object(input)?;
    let mut stages = vec![pass("NORMALIZER")];
    let value: Value = serde_json::from_str(&normalized_json).map_err(|_| failure(stages.clone(), "JSON_PARSER", InvalidOutputType::InvalidJson, None, "Falha ao interpretar JSON", normalized))?;
    let Some(object) = value.as_object() else {
        return Err(failure(stages, "JSON_PARSER", InvalidOutputType::SchemaTypeError, None, "A raiz da resposta deve ser um objeto JSON", normalized));
    };
    stages.push(pass("JSON_PARSER"));

    let allowed = ["action", "target_exposure_pct", "confidence", "reason_code", "reason"];
    if let Some(extra) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some(extra), format!("Campo não permitido: {extra}"), normalized));
    }
    let action_value = required(object, "action", &stages, normalized)?;
    let target_value = required(object, "target_exposure_pct", &stages, normalized)?;
    let confidence_value = required(object, "confidence", &stages, normalized)?;
    let reason_code_value = required(object, "reason_code", &stages, normalized)?;
    let reason_value = required(object, "reason", &stages, normalized)?;
    let Some(action_name) = action_value.as_str() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("action"), "action deve ser string", normalized));
    };
    let Some(target) = target_value.as_f64() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("target_exposure_pct"), "target_exposure_pct deve ser number", normalized));
    };
    let Some(confidence) = confidence_value.as_f64() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("confidence"), "confidence deve ser number", normalized));
    };
    let Some(reason_code) = reason_code_value.as_str() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("reason_code"), "reason_code deve ser string", normalized));
    };
    let Some(reason) = reason_value.as_str() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("reason"), "reason deve ser string", normalized));
    };
    if reason.trim().is_empty() || reason.chars().count() > 240 {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("reason"), "reason deve conter entre 1 e 240 caracteres", normalized));
    }
    stages.push(pass("SCHEMA_VALIDATOR"));

    let action = LifecycleAction::from_str(action_name).map_err(|_| failure(stages.clone(), "SEMANTIC_VALIDATOR", InvalidOutputType::InvalidAction, Some("action"), format!("Ação não permitida: {action_name}"), normalized))?;
    if !target.is_finite() || !(0.0..=100.0).contains(&target) {
        return Err(failure(stages, "SEMANTIC_VALIDATOR", InvalidOutputType::InvalidTargetExposure, Some("target_exposure_pct"), "target_exposure_pct deve estar entre 0 e 100", normalized));
    }
    if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
        return Err(failure(stages, "SEMANTIC_VALIDATOR", InvalidOutputType::InvalidConfidence, Some("confidence"), "confidence deve estar entre 0 e 1", normalized));
    }
    if !REASON_CODES.contains(&reason_code) {
        return Err(failure(stages, "SEMANTIC_VALIDATOR", InvalidOutputType::InvalidReasonCode, Some("reason_code"), format!("reason_code não permitido: {reason_code}"), normalized));
    }
    stages.push(pass("SEMANTIC_VALIDATOR"));

    let position = market_positions::validate_decision(position_state, current_exposure_pct, action, target);
    if position.validation_code.is_some() {
        let code = match (position_state, action) {
            (PositionState::Flat, LifecycleAction::ExitLong) => "FLAT_EXIT_ATTEMPT",
            (PositionState::Flat, LifecycleAction::ReduceLong) => "FLAT_REDUCE_ATTEMPT",
            (PositionState::LongOpen | PositionState::LongReduced, LifecycleAction::StayFlat) => "LONG_STAY_FLAT_ATTEMPT",
            _ => "TARGET_DIRECTION_MISMATCH",
        };
        let mut result = failure(stages, "POSITION_VALIDATOR", InvalidOutputType::InvalidPositionAction, Some("target_exposure_pct"), code, normalized);
        if let Some(stage) = result.stages.last_mut() {
            stage.error_code = Some(code.into());
        }
        return Err(result);
    }
    stages.push(pass("POSITION_VALIDATOR"));
    Ok(StructuredDecisionV41 {
        action,
        target_exposure_pct: target,
        confidence,
        reason_code: reason_code.into(),
        reason: reason.trim().into(),
        normalized_from_wrapped_json: normalized,
        stages,
    })
}

pub fn validate_v4_2(
    input: &str,
    position_state: PositionState,
) -> Result<StructuredDecisionV42, ValidationFailure> {
    let (normalized_json, normalized) = extract_single_object(input)?;
    let mut stages = vec![pass("NORMALIZER")];
    let value: Value = serde_json::from_str(&normalized_json).map_err(|_| {
        failure(
            stages.clone(),
            "JSON_PARSER",
            InvalidOutputType::InvalidJson,
            None,
            "Falha ao interpretar JSON",
            normalized,
        )
    })?;
    let Some(object) = value.as_object() else {
        return Err(failure(stages, "JSON_PARSER", InvalidOutputType::SchemaTypeError, None, "A raiz da resposta deve ser um objeto JSON", normalized));
    };
    stages.push(pass("JSON_PARSER"));

    let allowed = ["intent", "confidence_pct", "reason_code", "reason"];
    if let Some(extra) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some(extra), format!("Campo não permitido: {extra}"), normalized));
    }
    let intent_value = required(object, "intent", &stages, normalized)?;
    let confidence_value = required(object, "confidence_pct", &stages, normalized)?;
    let reason_code_value = required(object, "reason_code", &stages, normalized)?;
    let reason_value = required(object, "reason", &stages, normalized)?;
    let Some(intent_name) = intent_value.as_str() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("intent"), "intent deve ser string", normalized));
    };
    let Some(confidence_pct) = confidence_value.as_f64() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("confidence_pct"), "confidence_pct deve ser number", normalized));
    };
    let Some(reason_code) = reason_code_value.as_str() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("reason_code"), "reason_code deve ser string", normalized));
    };
    let Some(reason) = reason_value.as_str() else {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("reason"), "reason deve ser string", normalized));
    };
    if reason.trim().is_empty() || reason.chars().count() > 240 {
        return Err(failure(stages, "SCHEMA_VALIDATOR", InvalidOutputType::SchemaTypeError, Some("reason"), "reason deve conter entre 1 e 240 caracteres", normalized));
    }
    stages.push(pass("SCHEMA_VALIDATOR"));

    let intent = market_positions::PositionIntent::from_str(intent_name).map_err(|_| {
        failure(stages.clone(), "SEMANTIC_VALIDATOR", InvalidOutputType::InvalidIntent, Some("intent"), format!("Intent não permitido: {intent_name}"), normalized)
    })?;
    if !confidence_pct.is_finite() || !(0.0..=100.0).contains(&confidence_pct) {
        return Err(failure(stages, "SEMANTIC_VALIDATOR", InvalidOutputType::InvalidConfidencePct, Some("confidence_pct"), "confidence_pct deve estar entre 0 e 100", normalized));
    }
    if !REASON_CODES.contains(&reason_code) {
        return Err(failure(stages, "SEMANTIC_VALIDATOR", InvalidOutputType::InvalidReasonCode, Some("reason_code"), format!("reason_code não permitido: {reason_code}"), normalized));
    }
    stages.push(pass("SEMANTIC_VALIDATOR"));
    if !market_positions::intent_allowed(position_state, intent) {
        return Err(failure(stages, "POSITION_INTENT_VALIDATOR", InvalidOutputType::InvalidPositionIntent, Some("intent"), format!("{} não é permitido para o estado atual", intent.as_str()), normalized));
    }
    stages.push(pass("POSITION_INTENT_VALIDATOR"));
    Ok(StructuredDecisionV42 {
        intent,
        confidence: confidence_pct / 100.0,
        reason_code: reason_code.into(),
        reason: reason.trim().into(),
        normalized_from_wrapped_json: normalized,
        stages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(action: &str, target: &str) -> String {
        format!(r#"{{"action":"{action}","target_exposure_pct":{target},"confidence":0.7,"reason_code":"INSUFFICIENT_SIGNAL","reason":"controlled"}}"#)
    }

    #[test]
    fn exact_and_wrapped_json_are_distinguished() {
        assert!(!validate(&valid("STAY_FLAT", "0"), PositionState::Flat, 0.0).unwrap().normalized_from_wrapped_json);
        assert!(validate(&format!("Decision:\n{}", valid("STAY_FLAT", "0")), PositionState::Flat, 0.0).unwrap().normalized_from_wrapped_json);
    }

    #[test]
    fn malformed_empty_and_multiple_objects_are_classified() {
        assert_eq!(validate("", PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::EmptyResponse);
        assert_eq!(validate("{ action: BUY }", PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidJson);
        assert_eq!(validate("{} {}", PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::ExtraText);
    }

    #[test]
    fn schema_reports_missing_types_and_extra_fields() {
        assert_eq!(validate(r#"{"target_exposure_pct":0,"confidence":0.7,"reason_code":"INSUFFICIENT_SIGNAL","reason":"x"}"#, PositionState::Flat, 0.0).unwrap_err().error_field.as_deref(), Some("action"));
        assert_eq!(validate(r#"{"action":"STAY_FLAT"}"#, PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::MissingField);
        assert_eq!(validate(r#"{"action":"STAY_FLAT","target_exposure_pct":"0","confidence":0.7,"reason_code":"INSUFFICIENT_SIGNAL","reason":"x"}"#, PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::SchemaTypeError);
        assert_eq!(validate(r#"{"action":"STAY_FLAT","target_exposure_pct":0,"confidence":0.7,"reason_code":"INSUFFICIENT_SIGNAL","reason":"x","extra":true}"#, PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::SchemaTypeError);
    }

    #[test]
    fn semantic_ranges_and_enums_are_strict() {
        assert_eq!(validate(&valid("SHORT", "0"), PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidAction);
        assert_eq!(validate(&valid("STAY_FLAT", "-1"), PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidTargetExposure);
        assert_eq!(validate(&valid("STAY_FLAT", "101"), PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidTargetExposure);
        let invalid_confidence = valid("STAY_FLAT", "0").replace("0.7", "1.1");
        assert_eq!(validate(&invalid_confidence, PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidConfidence);
        let negative_confidence = valid("STAY_FLAT", "0").replace("0.7", "-0.1");
        assert_eq!(validate(&negative_confidence, PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidConfidence);
        let invalid_reason = valid("STAY_FLAT", "0").replace("INSUFFICIENT_SIGNAL", "MADE_UP");
        assert_eq!(validate(&invalid_reason, PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidReasonCode);
    }

    #[test]
    fn position_semantics_cover_flat_and_long_states() {
        for action in ["EXIT_LONG", "REDUCE_LONG"] {
            assert_eq!(validate(&valid(action, "0"), PositionState::Flat, 0.0).unwrap_err().error_type, InvalidOutputType::InvalidPositionAction);
        }
        assert!(validate(&valid("ENTER_LONG", "25"), PositionState::Flat, 0.0).is_ok());
        assert!(validate(&valid("STAY_FLAT", "0"), PositionState::Flat, 0.0).is_ok());
        assert!(validate(&valid("HOLD_POSITION", "40"), PositionState::LongOpen, 40.0).is_ok());
        assert!(validate(&valid("INCREASE_LONG", "60"), PositionState::LongOpen, 40.0).is_ok());
        assert!(validate(&valid("REDUCE_LONG", "20"), PositionState::LongOpen, 40.0).is_ok());
        assert!(validate(&valid("EXIT_LONG", "0"), PositionState::LongOpen, 40.0).is_ok());
        assert!(validate(&valid("STAY_FLAT", "0"), PositionState::LongOpen, 40.0).is_err());
        assert!(validate(&valid("REDUCE_LONG", "60"), PositionState::LongOpen, 40.0).is_err());
    }

    #[test]
    fn schema_uses_the_canonical_reason_code_registry() {
        assert_eq!(output_schema()["properties"]["reason_code"]["enum"].as_array().unwrap().len(), REASON_CODES.len());
        assert_eq!(audit_error_value(r#"{"target_exposure_pct":"fifty"}"#, Some("target_exposure_pct")).as_deref(), Some("\"fifty\""));
    }

    fn valid_v42(intent: &str, confidence: &str) -> String {
        format!(r#"{{"intent":"{intent}","confidence_pct":{confidence},"reason_code":"INSUFFICIENT_SIGNAL","reason":"controlled"}}"#)
    }

    #[test]
    fn v42_accepts_percentage_confidence_and_normalizes_it() {
        for confidence in ["0", "83", "100"] {
            let parsed = validate_v4_2(&valid_v42("ENTER", confidence), PositionState::Flat).unwrap();
            assert_eq!(parsed.confidence, confidence.parse::<f64>().unwrap() / 100.0);
        }
        for confidence in ["-1", "101"] {
            assert_eq!(
                validate_v4_2(&valid_v42("ENTER", confidence), PositionState::Flat).unwrap_err().error_type,
                InvalidOutputType::InvalidConfidencePct
            );
        }
    }

    #[test]
    fn v42_validates_intent_against_position_without_accepting_targets() {
        assert!(validate_v4_2(&valid_v42("ENTER", "80"), PositionState::Flat).is_ok());
        assert!(validate_v4_2(&valid_v42("HOLD", "80"), PositionState::Flat).is_ok());
        for intent in ["REDUCE", "EXIT"] {
            assert_eq!(
                validate_v4_2(&valid_v42(intent, "80"), PositionState::Flat).unwrap_err().error_type,
                InvalidOutputType::InvalidPositionIntent
            );
        }
        for intent in ["ENTER", "HOLD", "REDUCE", "EXIT"] {
            assert!(validate_v4_2(&valid_v42(intent, "80"), PositionState::LongOpen).is_ok());
        }
        assert_eq!(
            validate_v4_2(&valid_v42("BUY", "80"), PositionState::Flat).unwrap_err().error_type,
            InvalidOutputType::InvalidIntent
        );
        assert_eq!(
            validate_v4_2(r#"{"intent":"ENTER","confidence_pct":80,"target_exposure_pct":25,"reason_code":"INSUFFICIENT_SIGNAL","reason":"x"}"#, PositionState::Flat).unwrap_err().error_type,
            InvalidOutputType::SchemaTypeError
        );
    }
}

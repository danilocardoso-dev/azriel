use super::engineering_models::{EngineeringCalibration, EngineeringCalibrationInput};
use rusqlite::{params, Connection};

pub fn get_calibration(connection: &Connection) -> Result<EngineeringCalibration, String> {
    connection
        .query_row(
            "SELECT pinch_start_threshold,pinch_release_threshold,smoothing_alpha,rotation_sensitivity,min_scale,max_scale,comfortable_hand_distance,calibrated,updated_at FROM engineering_calibration WHERE id=1",
            [],
            |row| Ok(EngineeringCalibration {
                pinch_start_threshold: row.get(0)?,
                pinch_release_threshold: row.get(1)?,
                smoothing_alpha: row.get(2)?,
                rotation_sensitivity: row.get(3)?,
                min_scale: row.get(4)?,
                max_scale: row.get(5)?,
                comfortable_hand_distance: row.get(6)?,
                calibrated: row.get(7)?,
                updated_at: row.get(8)?,
            }),
        )
        .map_err(|error| error.to_string())
}

pub fn update_calibration(
    connection: &Connection,
    input: &EngineeringCalibrationInput,
) -> Result<EngineeringCalibration, String> {
    validate(input)?;
    connection
        .execute(
            "UPDATE engineering_calibration SET pinch_start_threshold=?1,pinch_release_threshold=?2,smoothing_alpha=?3,rotation_sensitivity=?4,min_scale=?5,max_scale=?6,comfortable_hand_distance=?7,calibrated=?8,updated_at=CURRENT_TIMESTAMP WHERE id=1",
            params![input.pinch_start_threshold, input.pinch_release_threshold, input.smoothing_alpha, input.rotation_sensitivity, input.min_scale, input.max_scale, input.comfortable_hand_distance, input.calibrated],
        )
        .map_err(|error| error.to_string())?;
    get_calibration(connection)
}

pub fn reset_calibration(connection: &Connection) -> Result<EngineeringCalibration, String> {
    connection
        .execute(
            "UPDATE engineering_calibration SET pinch_start_threshold=0.045,pinch_release_threshold=0.065,smoothing_alpha=0.38,rotation_sensitivity=3.2,min_scale=0.45,max_scale=2.4,comfortable_hand_distance=0.42,calibrated=0,updated_at=CURRENT_TIMESTAMP WHERE id=1",
            [],
        )
        .map_err(|error| error.to_string())?;
    get_calibration(connection)
}

fn validate(input: &EngineeringCalibrationInput) -> Result<(), String> {
    let finite = [
        input.pinch_start_threshold,
        input.pinch_release_threshold,
        input.smoothing_alpha,
        input.rotation_sensitivity,
        input.min_scale,
        input.max_scale,
        input.comfortable_hand_distance,
    ]
    .iter()
    .all(|value| value.is_finite());
    if !finite {
        return Err("A calibração contém valor não finito".into());
    }
    if !(0.015..=0.100).contains(&input.pinch_start_threshold) {
        return Err("Pinch start deve estar entre 0.015 e 0.100".into());
    }
    if !(0.025..=0.140).contains(&input.pinch_release_threshold)
        || input.pinch_release_threshold <= input.pinch_start_threshold
    {
        return Err("Pinch release deve ser maior que pinch start e no máximo 0.140".into());
    }
    if !(0.05..=1.0).contains(&input.smoothing_alpha) {
        return Err("Smoothing deve estar entre 0.05 e 1.0".into());
    }
    if !(0.25..=8.0).contains(&input.rotation_sensitivity) {
        return Err("Sensibilidade de rotação deve estar entre 0.25 e 8.0".into());
    }
    if !(0.20..=1.0).contains(&input.min_scale)
        || !(1.0..=5.0).contains(&input.max_scale)
        || input.max_scale <= input.min_scale
    {
        return Err("Limites de escala inválidos".into());
    }
    if !(0.05..=1.0).contains(&input.comfortable_hand_distance) {
        return Err("Distância confortável deve estar entre 0.05 e 1.0".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn input() -> EngineeringCalibrationInput {
        EngineeringCalibrationInput {
            pinch_start_threshold: 0.04,
            pinch_release_threshold: 0.07,
            smoothing_alpha: 0.4,
            rotation_sensitivity: 3.5,
            min_scale: 0.5,
            max_scale: 2.8,
            comfortable_hand_distance: 0.48,
            calibrated: true,
        }
    }

    #[test]
    fn calibration_is_persistent_and_resettable() {
        let path =
            std::env::temp_dir().join(format!("azriel-engineering-{}.db", std::process::id()));
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            let saved = update_calibration(&connection, &input()).unwrap();
            assert!(saved.calibrated);
            assert_eq!(saved.rotation_sensitivity, 3.5);
        }
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            assert_eq!(
                get_calibration(&connection)
                    .unwrap()
                    .comfortable_hand_distance,
                0.48
            );
            assert!(!reset_calibration(&connection).unwrap().calibrated);
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_ranges_are_rejected() {
        let mut invalid = input();
        invalid.pinch_release_threshold = invalid.pinch_start_threshold;
        assert!(validate(&invalid).is_err());
        invalid = input();
        invalid.max_scale = invalid.min_scale;
        assert!(validate(&invalid).is_err());
    }
}

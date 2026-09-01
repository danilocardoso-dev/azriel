use crate::database::{engineering_models::*, engineering_repository, DatabaseState};
use tauri::State;

fn lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn get_engineering_calibration(
    state: State<'_, DatabaseState>,
) -> Result<EngineeringCalibration, String> {
    let connection = lock(&state)?;
    engineering_repository::get_calibration(&connection)
}

#[tauri::command]
pub fn update_engineering_calibration(
    state: State<'_, DatabaseState>,
    input: EngineeringCalibrationInput,
) -> Result<EngineeringCalibration, String> {
    let connection = lock(&state)?;
    engineering_repository::update_calibration(&connection, &input)
}

#[tauri::command]
pub fn reset_engineering_calibration(
    state: State<'_, DatabaseState>,
) -> Result<EngineeringCalibration, String> {
    let connection = lock(&state)?;
    engineering_repository::reset_calibration(&connection)
}

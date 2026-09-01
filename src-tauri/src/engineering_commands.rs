use crate::database::{engineering_models::*, engineering_repository, DatabaseState};
use tauri::State;

const MAX_MODEL_BYTES: u64 = 100 * 1024 * 1024;

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

#[tauri::command]
pub fn read_engineering_model(path: String) -> Result<tauri::ipc::Response, String> {
    read_model_bytes(std::path::Path::new(&path)).map(tauri::ipc::Response::new)
}

fn read_model_bytes(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "O modelo selecionado não possui extensão válida".to_string())?;
    if extension != "glb" && extension != "gltf" {
        return Err("Formato inválido. Selecione um arquivo GLB ou GLTF".into());
    }
    let metadata = std::fs::metadata(&path)
        .map_err(|error| format!("Não foi possível acessar o modelo: {error}"))?;
    if !metadata.is_file() {
        return Err("O caminho selecionado não é um arquivo".into());
    }
    if metadata.len() == 0 {
        return Err("O arquivo selecionado está vazio".into());
    }
    if metadata.len() > MAX_MODEL_BYTES {
        return Err("O modelo excede o limite local de 100 MB desta versão".into());
    }
    std::fs::read(path).map_err(|error| format!("Falha ao ler o modelo local: {error}"))
}

#[cfg(test)]
mod model_file_tests {
    use super::read_model_bytes;

    #[test]
    fn model_reader_validates_extension_empty_file_and_content() {
        let directory =
            std::env::temp_dir().join(format!("azriel-model-reader-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let invalid = directory.join("model.obj");
        std::fs::write(&invalid, [1_u8, 2, 3]).unwrap();
        assert!(read_model_bytes(&invalid)
            .unwrap_err()
            .contains("Formato inválido"));

        let empty = directory.join("empty.glb");
        std::fs::write(&empty, []).unwrap();
        assert!(read_model_bytes(&empty).unwrap_err().contains("vazio"));

        let valid = directory.join("model.glb");
        std::fs::write(&valid, [0x67_u8, 0x6c, 0x54, 0x46]).unwrap();
        assert_eq!(read_model_bytes(&valid).unwrap(), b"glTF");
        std::fs::remove_dir_all(directory).unwrap();
    }
}

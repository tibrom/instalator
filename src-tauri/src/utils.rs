use std::fs;
use std::path::PathBuf;
use tauri::api::path::cache_dir;
use std::io;
use tokio::fs::{read_dir};
use std::time::SystemTime;


use crate::config::{DIRS, CACHDIR, SUBNET_MASKS};
use crate::state::{ConfigStep, StepState};

pub fn create_data_folder() -> io::Result<()> {
    // Получаем директорию кэша
    if let Some(app_path) = cache_dir() {
        for dr in DIRS {
            let cach_dir = app_path.join(CACHDIR);
            let data_folder = cach_dir.join(dr);
            
            // Проверяем, существует ли папка
            if !data_folder.exists() {
                // Создаем папку, если она не существует
                fs::create_dir_all(&data_folder)?;
                println!("Папка для данных успешно создана: {:?}", data_folder);
            } else {
                println!("Папка для данных уже существует: {:?}", data_folder);
            }
        }
        Ok(())
    } else {
        // Если не удалось получить директорию приложения, возвращаем ошибку
        Err(io::Error::new(io::ErrorKind::Other, "Не удалось получить директорию кэша."))
    }
}


pub fn get_path_in_cach(dir: &str) -> PathBuf {
    if let Some(app_path) = cache_dir() {
        let cach_dir = app_path.join(CACHDIR); // Создаем полную директорию кэша
        let data_folder = cach_dir.join(dir);  // Создаем путь к папке

        return data_folder; // Возвращаем PathBuf
    }
    
    // Если не удалось получить путь, возвращаем локальный путь
    println!("Не удалось получить каш директорию у ОС");
    PathBuf::from("./cach") // Возвращаем PathBuf вместо ссылки
}


pub fn get_path_file_cach(dir: Vec<&str>) -> PathBuf {
    if let Some(app_path) = cache_dir() {
        let mut data_folder = app_path.join(CACHDIR); // Создаем полную директорию кэша
        for dr in dir {
            data_folder = data_folder.join(dr);
        }

        // Преобразуем путь в строку
        return data_folder;
    }
    
    // Если не удалось получить путь, возвращаем локальный путь
    println!("Не удалось получить каш директорию у ОС");
    PathBuf::from("./cach")
}

pub async fn find_latest_file(path: PathBuf) -> Option<PathBuf> {
    let mut dir = read_dir(&path).await.ok()?;
    println!("dir {:?}", dir);// Читаем содержимое директории
    let mut latest_file: Option<(SystemTime, PathBuf)> = None;  // Будем хранить время изменения и путь к файлу

    while let Some(entry) = dir.next_entry().await.ok()? {
        let file_path = entry.path();

        if file_path.is_file() {
            let metadata = entry.metadata().await.ok()?;
            if let Ok(modified) = metadata.modified() {
                match &latest_file {
                    Some((last_modified, _)) if modified > *last_modified => {
                        latest_file = Some((modified, file_path));
                    }
                    None => {
                        latest_file = Some((modified, file_path));
                    }
                    _ => {}
                }
            }
        }
    }

    latest_file.map(|(_, file_path)| file_path)  // Возвращаем путь к последнему изменённому файлу
}

pub fn get_mask_by_index(index: u8) -> String {
    
    // Проверка, что индекс находится в допустимом диапазоне
    let index = index as usize;
    if index >= SUBNET_MASKS.len() || index == 0{
        return SUBNET_MASKS[8].to_string(); // Возвращаем маску по умолчанию
    }
    
    SUBNET_MASKS[index].to_string()
}


pub fn process_config_step(step: &ConfigStep) -> (String, String) {
    let state = match step {
        ConfigStep::Create(_) => {"CREATE".to_string()},
        ConfigStep::Scan(_) => {"SCAN".to_string()},
        ConfigStep::ArchiveGeneration(_) => {"ARCHIVEGENERATION".to_string()},
        ConfigStep::ArchiveDownload(_) => {"ARCHIVEDOWNLOAD".to_string()},
        ConfigStep::SendingFiles(_) => {"SENDINGFILES".to_string()},
        ConfigStep::ProgramInstallation(_) => {"INSTALLATE".to_string()},
        ConfigStep::Configurator(_) => {"CONFIGURATOR".to_string()},
        ConfigStep::DbConfigurator(_) => {"DBCONFIGURATOR".to_string()},
        ConfigStep::Coropase(_) => {"COROPASE".to_string()},
        ConfigStep::SendData(_) => {"SEND_DATA".to_string()},

    }.to_string();

    let code = match step {
        ConfigStep::Create(state) |
        ConfigStep::Scan(state) |
        ConfigStep::ArchiveGeneration(state) |
        ConfigStep::ArchiveDownload(state) |
        ConfigStep::SendingFiles(state) |
        ConfigStep::SendData(state) |
        ConfigStep::ProgramInstallation(state) |
        ConfigStep::DbConfigurator(state) |
        ConfigStep::Coropase(state)|
        ConfigStep::Configurator(state) => {
            match state {
                StepState::Success(code) => format!("Success {}", code),
                StepState::Expectation(code) => format!("Expectation {}", code),
                StepState::Failed(code) => format!("Failed {}", code),
            }
        }
    };

    (state, code)
}

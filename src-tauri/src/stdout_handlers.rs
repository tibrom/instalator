use regex::Regex;
use std::path::PathBuf;
use tokio::fs;
use std::io;
use crate::state::{get_master_device, get_slave_device, ConfigStep, StepState, get_command_by_name,
add_clear_stdout, change_device_config_step};
use crate::device::DeviceType;
use crate::utils::get_path_file_cach;
use crate::config::PORTSSHTUNNEL;

pub fn main_filter_srdout(stdout_data: &str) -> Vec<String> {
    let re = Regex::new(r"\[ STATE_[^\]]+ \]").unwrap();
    re.find_iter(stdout_data)
        .map(|mat| {
            let matched_str = mat.as_str();
            // Удаляем первые и последние символы (квадратные скобки и пробелы)
            matched_str[2..matched_str.len() - 2].to_string()
        })
        .collect()
}

fn count_lines(s: &str) -> usize {
    s.lines().count()
}


pub fn hendler_stdout_device(device_type: &DeviceType, stdout_data: &str) -> () {
    let device_sessions= if device_type == &DeviceType::Master {
        get_master_device()
    } else {
        get_slave_device()
    };
    
    let new_clear_stdout: Vec<String> = main_filter_srdout(stdout_data);
    
    let clearstdout: Vec<String> = add_clear_stdout(&device_type, new_clear_stdout);
    
    match device_sessions.config_step {
        ConfigStep::Scan(StepState::Expectation(count_line)) => {
            scan_device_handler(clearstdout, &device_type, count_line, stdout_data)
        },
        
        ConfigStep::ArchiveGeneration(StepState::Expectation(count_line)) => {
            let dv_t = device_type.clone();
            let count_line = count_line.clone();
            let stdout_data = stdout_data.to_string();
            tokio::task::spawn(async move {
                let _ = archive_gen_handler(clearstdout, dv_t, count_line, &stdout_data).await;
            });
        },

        ConfigStep::ProgramInstallation(StepState::Expectation(count_line)) => {
            let _ = install_handler(clearstdout, &device_type, count_line, &stdout_data);
        },
        ConfigStep::Configurator(StepState::Expectation(count_line)) => {
            let _ = configurator_handler(clearstdout, &device_type, count_line, &stdout_data);
        },
        ConfigStep::DbConfigurator(StepState::Expectation(count_line)) => {
            let _ = configurator_db(clearstdout, &device_type, count_line, &stdout_data);
        },
        ConfigStep::Coropase(StepState::Expectation(count_line)) => {
            let device_type = device_type.clone();
            let count_line = count_line.clone();
            let stdout_data = stdout_data.to_string();
            tokio::task::spawn(async move {
                let _ = coropace(clearstdout, &device_type, count_line, &stdout_data).await;
            });
            
        },
        _ =>{
            //send_alert(AlertMessage::new(alert_type, stdout_data.to_string()));
        }
    }
}

fn scan_device_handler(clearstdout: Vec<String>, device_type: &DeviceType, count_line: usize, stdout_data: &str) -> () {
    

    if !clearstdout.contains(&"STATE_SCAN_STOP".to_string()) { //Если скрипт не завершил выполнение, то ничего не проверяем.
        let act = count_lines(stdout_data);
        change_device_config_step(device_type, ConfigStep::Scan(StepState::Expectation(count_line + act)));
        return ()
    }
    println!("SCAN COUNT LINE {:?}", count_line);
    let command_data = get_command_by_name("scan".to_string());
    let stdout_state_ok: Vec<String> = command_data.check_state.iter()
    .map(|s| format!("{}OK", s))
    .collect();

    println!("{:?}", stdout_state_ok);
    let stdout_state_err: Vec<String> = command_data.check_state.iter()
    .map(|s| format!("{}ERR", s))
    .collect();

    let all_state_ok = stdout_state_ok.iter().all(|item| clearstdout.contains(item));
    for state in stdout_state_ok {
        if !clearstdout.contains(&state) {
            println!("state {:?} not in", state)
        }
    };
    if all_state_ok {
        println!("НЕ ошибок");
        change_device_config_step(device_type, ConfigStep::Scan(StepState::Success(0)))
    } else {
        println!("ошибоки есть");
        for err in stdout_state_err {
            
            if clearstdout.contains(&err) {
                println!("{:?}", err);
                change_device_config_step(device_type, ConfigStep::Scan(StepState::Failed(100)))
            }
        }
    }
}


async  fn archive_gen_handler(clearstdout: Vec<String>, device_type: DeviceType, count_line: usize, stdout_data: &str) -> () {
    

    if !clearstdout.contains(&"STATE_ARCHIVGEN_STOP".to_string()) { //Если скрипт не завершил выполнение, то ничего не проверяем.
        let act = count_lines(stdout_data);
        change_device_config_step(&device_type, ConfigStep::ArchiveGeneration(StepState::Expectation(count_line + act)));
        return ()
    }
    
    

    let command_data = get_command_by_name("scan".to_string());
    let stdout_state_ok: Vec<String> = command_data.check_state.iter()
    .map(|s| format!("{}OK", s))
    .collect();

    
    let stdout_state_err: Vec<String> = command_data.check_state.iter()
    .map(|s| format!("{}ERR", s))
    .collect();

    let all_state_ok = stdout_state_ok.iter().all(|item| clearstdout.contains(item));
    for state in stdout_state_ok {
        if !clearstdout.contains(&state) {
            println!("state {:?} not in", state)
        }
    };
    if all_state_ok {
        println!("НЕТ ошибок");
    } else {
        println!("ошибоки есть");
        for err in stdout_state_err {
            
            if clearstdout.contains(&err) {
                println!("{:?}", err);
                change_device_config_step(&device_type, ConfigStep::ArchiveGeneration(StepState::Failed(100)))
            }
        }
    }


    let mut archive_name: Option<String> = None; // Меняем тип на Option<String>
    let mut dowonload_ok: bool = false;
    let mut dowonload_start: bool = false;

    for state in clearstdout {
        if state.contains("STATE_ARCHIVENAME_") {
            println!("file_name: {:?}", state);
            
            if let Some(archive) = state.strip_prefix("STATE_ARCHIVENAME_") {
                let archive = archive.to_string(); // Копируем результат в новую строку
                println!("clear_file_name: {:?}", archive);
                archive_name = Some(archive); // Присваиваем значение переменной archive_name
            }
        }
        
        if state.contains("DOWNLOAD_OK") {
            dowonload_ok = true;
            dowonload_start = false;
        }
        
        if state.contains("DOWNLOAD_START") {
            dowonload_start = true;
        }
    }
    

    let (device, temp_file_name,  directory) = if device_type == DeviceType::Master {
        (get_master_device(), "master.tar.bz2", "master_input")
    } else {
        (get_slave_device(), "slave.tar.bz2",  "slave_input")
    };
    let Some(session) = device.session else {
        return ()
    };
    match archive_name {
        Some(name) => {
            if dowonload_ok {
                let dirs = vec![directory, &name];
                let file_path = get_path_file_cach(dirs);
                let path_to_temp_file = get_path_file_cach(vec!["temp", temp_file_name]);
                println!("file_path {:?}", file_path);
                let status =move_and_rename_file(path_to_temp_file, file_path).await;
                match status {
                    Ok(_) => {
                    change_device_config_step(&device_type, ConfigStep::ArchiveGeneration(StepState::Success(0)))
                    },
                    Err(er) => {println!("Ошибка {:?}", er)}
                }
            } else if !dowonload_start {
                session.send_stdin("echo [ STATE_DOWNLOAD_START ]\n".into()).await.unwrap();
                let download_command = format!("nc localhost {} < ./{} && echo [ STATE_DOWNLOAD_OK ]\n", PORTSSHTUNNEL, name); // скачиваем архив на локальную машину
                println!("download_command {:?}", download_command);
                session.send_stdin(download_command.into()).await.unwrap();
                return;
            }
        },
        _ =>{},
    }
                        
}





async fn move_and_rename_file(src: PathBuf, dest: PathBuf) -> io::Result<()> {
    // Асинхронно перемещаем и переименовываем файл
    fs::rename(&src, &dest).await?;

    println!(
        "Файл перемещён и переименован из '{:?}' в '{:?}'", 
        src, dest
    );
    
    Ok(())
}


fn install_handler(clearstdout: Vec<String>, device_type: &DeviceType, count_line: usize, stdout_data: &str) -> () {
    

    if !clearstdout.contains(&"STATE_MODULE_OK".to_string()) { //Если скрипт не завершил выполнение, то ничего не проверяем.
        let act = count_lines(stdout_data);
        change_device_config_step(&device_type, ConfigStep::ProgramInstallation(StepState::Expectation(count_line + act)));
        return ()
    }
    println!("INSTALENION COUNT LINE {:?}", count_line);
    change_device_config_step(&device_type, ConfigStep::ProgramInstallation(StepState::Success(0)))
    
}

fn configurator_handler(clearstdout: Vec<String>, device_type: &DeviceType, count_line: usize, stdout_data: &str) -> () {
    if !clearstdout.contains(&"STATE_CONFIGURE_STOP".to_string()) { //Если скрипт не завершил выполнение, то ничего не проверяем.
        let act = count_lines(stdout_data);
        change_device_config_step(&device_type, ConfigStep::Configurator(StepState::Expectation(count_line + act)));
        return ()
    }
    println!("CONFIGURATION COUNT LINE {:?}", count_line);
    change_device_config_step(&device_type, ConfigStep::Configurator(StepState::Success(0)))
}


fn configurator_db(clearstdout: Vec<String>, device_type: &DeviceType, count_line: usize, stdout_data: &str) -> () {
    if !clearstdout.contains(&"STATE_MYSQL_OK".to_string()) { //Если скрипт не завершил выполнение, то ничего не проверяем.
        let act = count_lines(stdout_data);
        change_device_config_step(&device_type, ConfigStep::DbConfigurator(StepState::Expectation(count_line + act)));
        return ()
    }
    println!("CONFIGURATION DB COUNT LINE {:?}", count_line);
    change_device_config_step(&device_type, ConfigStep::DbConfigurator(StepState::Success(0)))
}

async fn coropace(clearstdout: Vec<String>, device_type: &DeviceType, count_line: usize, stdout_data: &str) -> () {
    if !clearstdout.contains(&"STATE_COROPASE_OK".to_string()) { //Если скрипт не завершил выполнение, то ничего не проверяем.
        let act = count_lines(stdout_data);
        change_device_config_step(&device_type, ConfigStep::Coropase(StepState::Expectation(count_line + act)));
        return ()
    }
    println!("COROPASE COUNT LINE {:?}", count_line);
    change_device_config_step(&device_type, ConfigStep::Coropase(StepState::Success(0)));
    let device = if *device_type == DeviceType::Master {
        get_master_device()
    } else {
        get_slave_device()
    };
    let Some(session) = device.session else {
        return ()
    };
    session.send_stdin("sudo reboot\n".into()).await.unwrap();


}
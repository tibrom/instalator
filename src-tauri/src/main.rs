// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::net::IpAddr;
use tokio::fs;
use tokio::time::{sleep, Duration};
use std::path::Path;

mod state;
mod client;
mod device;
mod remoute;
mod stdout_handlers;
mod config;
mod utils;
use state::{AlertType, AlertMessage, DeviceSession, ConfigStep, StepState, change_device_config_step,
    get_company_info, get_master_device, get_slave_device, get_alerts_by_type,
    get_command_by_name, set_company_info, send_alert, get_clear_stdout, delete_device_data};
use client::connect_to_server;
use serde::{Serialize, Deserialize};
use device::{ConnectType, Credentials, Device, DeviceType,};
use utils::{create_data_folder, get_path_in_cach, find_latest_file, get_mask_by_index,
    process_config_step, 
};
use crate::config::PORTSSHTUNNEL;


#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateData{
    master_state: String,
    master_code: String,
    slave_state: String, 
    slave_code: String,
}


#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCreationData {
    pub ip_address: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub sudopassword: Option<String>,
    pub file_key: Option<String>,
}


#[tauri::command]
async fn delete_lic_swk(ismaster: bool) {
    let act_device_sessions= if ismaster {
        let state_dev = get_master_device();
        state_dev
    } else {
        let state_dev = get_slave_device();
        state_dev
    };
    let Some(session) = act_device_sessions.session else {

        println!("err 5");
        return ()
    };
    session.send_stdin("sudo rm -r /usr/share/rtu5_installator".into()).await.unwrap();
    

}


#[tauri::command]
async fn delete_device(ismaster: bool) {
    println!("delete_device");
    let act_device_sessions= if ismaster {
        let state_dev = get_master_device();
        state_dev
    } else {
        let state_dev = get_slave_device();
        state_dev
    };
    let Some(device) = act_device_sessions.device else {
        return;
    };
    let _ = delete_device_data(&device.device_type);
    let config_step = ConfigStep::Create(StepState::Failed(100));
    change_device_config_step(&device.device_type, config_step);

}


#[tauri::command]
async fn change_state(ismaster: bool, index: u8) {
    let device_type = if ismaster { DeviceType::Master } else { DeviceType::Slave };

    let allowed_state = [
        ConfigStep::Create(StepState::Success(0)),
        ConfigStep::Scan(StepState::Success(0)),
        ConfigStep::ArchiveGeneration(StepState::Success(0)),
        ConfigStep::ArchiveDownload(StepState::Success(0)),
        ConfigStep::SendingFiles(StepState::Success(0)),
        ConfigStep::ProgramInstallation(StepState::Success(0)),
        ConfigStep::Configurator(StepState::Success(0)),
        ConfigStep::DbConfigurator(StepState::Success(0)),
        ConfigStep::Coropase(StepState::Success(0)),
        ConfigStep::SendData(StepState::Success(0)),
    ];

    // Приведение типов index к usize для правильной работы с массивом
    if (index as usize) >= allowed_state.len() {
        return;
    }

    let config_step = &allowed_state[index as usize];
    change_device_config_step(&device_type, config_step.clone());
}


#[tauri::command]
async fn start_coropace(masterip: String, slaveip: String, swimmingip: String, maskindex: u8, pinggw: String) {
    change_device_config_step(&DeviceType::Master, ConfigStep::Coropase(StepState::Expectation(0)));
    change_device_config_step(&DeviceType::Slave, ConfigStep::Coropase(StepState::Expectation(0)));

    let master_device_sessions = get_master_device();
    let slave_device_sessions = get_slave_device(); 
    let command_install_script = get_command_by_name("coropace".to_string());
    let install_command = command_install_script.command;
    let sub_mask = get_mask_by_index(maskindex);

    let Some(master_session) = master_device_sessions.session else {
        println!("Error: Master session not found");
        return;
    };

    let Some(slave_session) = slave_device_sessions.session else {
        println!("Error: Slave session not found");
        return;
    };
    
    let export_master_ip = format!(
        "sed --in-place 's|export IP_MASTER.*|export IP_MASTER={}|' ./coropace.sh\n",
        masterip
    );
    let export_slave_ip = format!(
        "sed --in-place 's|export IP_SLAVE.*|export IP_SLAVE={}|' ./coropace.sh\n",
        slaveip
    );
    let export_swimm_ip = format!(
        "sed --in-place 's|export IP_SWIMMING=.*|export IP_SWIMMING={}|' ./coropace.sh\n",
        swimmingip
    );
    let export_swimm_ip_mask = format!(
        "sed --in-place 's|export IP_SWIMMING_NETMASK.*|export IP_SWIMMING_NETMASK={}|' ./coropace.sh\n",
        sub_mask
    );
    let export_ping = format!(
        "sed --in-place 's|export PING_GW.*|export PING_GW={}|' ./coropace.sh\n",
        pinggw
    );

    master_session.send_stdin(install_command.clone().into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    master_session.send_stdin(export_master_ip.clone().into()).await.unwrap();
    master_session.send_stdin(export_slave_ip.clone().into()).await.unwrap();
    master_session.send_stdin(export_swimm_ip.clone().into()).await.unwrap();
    master_session.send_stdin(export_swimm_ip_mask.clone().into()).await.unwrap();
    master_session.send_stdin(export_ping.clone().into()).await.unwrap();
    master_session.send_stdin("sudo chmod +x ./coropace.sh\n".into()).await.unwrap();
    slave_session.send_stdin(install_command.clone().into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    slave_session.send_stdin(export_master_ip.clone().into()).await.unwrap();
    slave_session.send_stdin(export_slave_ip.clone().into()).await.unwrap();
    slave_session.send_stdin(export_swimm_ip.clone().into()).await.unwrap();
    slave_session.send_stdin(export_swimm_ip_mask.clone().into()).await.unwrap();
    slave_session.send_stdin(export_ping.clone().into()).await.unwrap();
    slave_session.send_stdin("sudo chmod +x ./coropace.sh\n".into()).await.unwrap();
    slave_session.send_stdin("sudo ./coropace.sh -active_step1\n".into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    master_session.send_stdin("sudo ./coropace.sh -standby_step2\n".into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    slave_session.send_stdin("sudo ./coropace.sh -active_step3\n".into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    slave_session.send_stdin("sudo ./coropace.sh -standby_step4\n".into()).await.unwrap();
    master_session.send_stdin("sudo ./coropace.sh -active_step5\n".into()).await.unwrap();

}


#[tauri::command]
async fn start_config_db(masterip: String, slaveip: String) {
    change_device_config_step(&DeviceType::Master, ConfigStep::DbConfigurator(StepState::Expectation(0)));
    change_device_config_step(&DeviceType::Slave, ConfigStep::DbConfigurator(StepState::Expectation(0)));
    let master_device_sessions = get_master_device();
    let slave_device_sessions = get_slave_device(); // исправлена опечатка
    let command_install_script = get_command_by_name("dbconfig".to_string());
    let install_command = command_install_script.command;

    let Some(master_session) = master_device_sessions.session else {
        println!("Error: Master session not found");
        return;
    };

    let Some(slave_session) = slave_device_sessions.session else {
        println!("Error: Slave session not found");
        return;
    };
    
    let export_master_ip = format!(
        "sed --in-place 's|export IP_MASTER.*|export IP_MASTER={}|' ./dbconfig.sh\n",
        masterip
    );
    let export_slave_ip = format!(
        "sed --in-place 's|export IP_SLAVE.*|export IP_SLAVE={}|' ./dbconfig.sh\n",
        slaveip
    );

    master_session.send_stdin(install_command.clone().into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    master_session.send_stdin(export_master_ip.clone().into()).await.unwrap();
    master_session.send_stdin(export_slave_ip.clone().into()).await.unwrap();
    master_session.send_stdin("sudo chmod +x ./dbconfig.sh\n".into()).await.unwrap();
    slave_session.send_stdin(install_command.clone().into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    slave_session.send_stdin(export_master_ip.clone().into()).await.unwrap();
    slave_session.send_stdin(export_slave_ip.clone().into()).await.unwrap();
    slave_session.send_stdin("sudo chmod +x ./dbconfig.sh\n".into()).await.unwrap();
    slave_session.send_stdin("sudo ./dbconfig.sh -standby_step1\n".into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    master_session.send_stdin("sudo ./dbconfig.sh -active_step2\n".into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    slave_session.send_stdin("sudo ./dbconfig.sh -standby_step3\n".into()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    slave_session.send_stdin("sudo ./dbconfig.sh -standby_step4\n".into()).await.unwrap();
    master_session.send_stdin("sudo ./dbconfig.sh -active_step4\n".into()).await.unwrap();
}


#[tauri::command]
async fn start_config_program(masterip: String, slaveip: String, swimmingip: String, ismaster: bool) {
    let master_device_sessions = get_master_device();
    let slave_device_sessions = get_slave_device(); // исправлена опечатка
    let mut access_db_password_slave = String::new();
    let mut access_log_password_slave = String::new();
    let command_install_script = get_command_by_name("configurator".to_string());
    let install_command = command_install_script.command;

    let Some(master_session) = master_device_sessions.session else {
        println!("Error: Master session not found");
        return;
    };

    let Some(slave_session) = slave_device_sessions.session else {
        println!("Error: Slave session not found");
        return;
    };

    let (main_command, session) = if ismaster {
        change_device_config_step(&DeviceType::Master, ConfigStep::Configurator(StepState::Expectation(0)));
        let db_log_password = "RESULT=$(sudo cat /etc/mvts3g/access-logger-db.conf | grep passwd | cut --delimiter '=' -f2) && echo [ STATE_DBLOGPAS_${RESULT} ]\n"
            .to_string();
        let db_passeord = "RESULT=$(sudo cat /etc/mvts3g/access-db.conf | grep passwd | cut --delimiter '=' -f2) && echo [ STATE_DBPAS_${RESULT} ]\n"
            .to_string();
        slave_session.send_stdin(db_log_password.into()).await.unwrap();
        slave_session.send_stdin(db_passeord.into()).await.unwrap();
        sleep(Duration::from_secs(2)).await;

        master_session.send_stdin(install_command.clone().into()).await.unwrap();
        let device_type = DeviceType::Slave;
        let stdout = get_clear_stdout(&device_type); // async

        for data in stdout {
            println!("{:?}",data);
            if let Some(p) = data.strip_prefix("STATE_DBPAS_") {
                println!("DB Password: {:?}", p);
                access_db_password_slave = p.to_string();
            } else if let Some(p) = data.strip_prefix("STATE_DBLOGPAS_") {
                println!("Log Password: {:?}", p);
                access_log_password_slave = p.to_string();
            }
        }

        if !access_db_password_slave.is_empty() && !access_log_password_slave.is_empty() {
            println!("state 1");
            let command_db_pass = format!(
                "sed --in-place 's|export ACCESS_DB_PASSWORD_SLAVE.*|export ACCESS_DB_PASSWORD_SLAVE={}|' ./configurator.sh\n",
                access_db_password_slave
            );
            let command_log_pass = format!(
                "sed --in-place 's|export ACCESS_LOGGER_DB_PASSWORD_SLAVE.*|export ACCESS_LOGGER_DB_PASSWORD_SLAVE={}|' ./configurator.sh\n",
                access_log_password_slave
            );
            master_session.send_stdin(command_db_pass.into()).await.unwrap();
            println!("state 2");
            master_session.send_stdin(command_log_pass.into()).await.unwrap();
        }

        (
            "sudo chmod +x ./configurator.sh && sudo ./configurator.sh -active\n".to_string(),
            master_session
        )
    } else {
        change_device_config_step(&DeviceType::Slave, ConfigStep::Configurator(StepState::Expectation(0)));
        slave_session.send_stdin(install_command.clone().into()).await.unwrap();
        (
            "sudo chmod +x ./configurator.sh && sudo ./configurator.sh -standby\n".to_string(),
            slave_session
        )
    };

    // Обновление IP-адресов в конфигурационном файле
    
    let export_master_ip = format!(
        "sed --in-place 's|export IP_MASTER.*|export IP_MASTER={}|' ./configurator.sh\n",
        masterip
    );
    let export_slave_ip = format!(
        "sed --in-place 's|export IP_SLAVE.*|export IP_SLAVE={}|' ./configurator.sh\n",
        slaveip
    );
    let export_swimm_ip = format!(
        "sed --in-place 's|export IP_SWIMMING.*|export IP_SWIMMING={}|' ./configurator.sh\n",
        swimmingip
    );
    println!("state 3");
    session.send_stdin(export_master_ip.into()).await.unwrap();
    println!("state 4");
    session.send_stdin(export_slave_ip.into()).await.unwrap();
    println!("state 5");
    session.send_stdin(export_swimm_ip.into()).await.unwrap();
    println!("state 6");
    session.send_stdin(main_command.into()).await.unwrap();
    session.send_stdin("echo [ STATE_CONFIGURE_STOP ]\n".into()).await.unwrap();
}


#[tauri::command]
async fn start_install(masterip: String, slaveip: String, port: u16, ismaster: bool) {
    let (act_device_sessions, main_comand_data) = if ismaster {
        let state_dev = get_master_device();
        (state_dev, "sudo chmod +x installation.sh && sudo ./installation.sh -active\n".to_string())
    } else {
        let state_dev = get_slave_device();
        (state_dev, "sudo chmod +x installation.sh && sudo ./installation.sh -standby\n".to_string())
    };
    let Some(session) = act_device_sessions.session else {

        println!("err 5");
        return ()
    };

    let device_type = act_device_sessions.device.unwrap().device_type;
    let config_step = ConfigStep::ProgramInstallation(StepState::Expectation(0));

    change_device_config_step(&device_type, config_step);

    session.send_stdin("echo [ STATE_INSTALLARTION_START ]\n".into()).await.unwrap();

    let command_instal_sctipt = get_command_by_name("installation".to_string());
    let install_comand =  command_instal_sctipt.command;

    let export_master_ip = format!(
        "sed --in-place 's|export IP_MASTER.*|export IP_MASTER={}|' ./installation.sh\n",
        masterip
    );
    let export_slave_ip = format!(
        "sed --in-place 's|export IP_SLAVE.*|export IP_SLAVE={}|' ./installation.sh\n",
        slaveip
    );
    let export_port_ip = format!(
        "sed --in-place 's|export RSYNC_PORT.*|export RSYNC_PORT={}|' ./installation.sh\n",
        port
    );

    session.send_stdin(install_comand.into()).await.unwrap();
    session.send_stdin(export_master_ip.into()).await.unwrap();
    session.send_stdin(export_slave_ip.into()).await.unwrap();
    session.send_stdin(export_port_ip.into()).await.unwrap();
    session.send_stdin(main_comand_data.into()).await.unwrap();
    session.send_stdin("\n".into()).await.unwrap();
    session.send_stdin("echo [ STATE_INSTALLARTION_STOP ]\n".into()).await.unwrap();
    
}


#[tauri::command]
async fn get_device_ip(ismaster: bool) -> Vec<String>{
    let mut result: Vec<String> =  Vec::new();
    let act_device_sessions = if ismaster {
        get_master_device()
    } else {
        get_slave_device()
    };

    let device_type = act_device_sessions.device.unwrap().device_type;
    let stdout = get_clear_stdout(&device_type);

    for data in stdout {
        if data.contains("STATE_IP_"){
            println!("{:?}", data);
            let ip = data.strip_prefix("STATE_IP_");
            println!("{:?}", ip);
            match ip {
                Some(i) => {
                    if !i.contains("OK") {
                        result.push(i.to_string());
                    }
                    
                }
                _ => {}
            }
            
        }
    }

    println!("{:?}", result);
    result
}



#[tauri::command]
async fn upload_file(licfilepath: String, swkfilepath: String, ismaster: bool)  {
    let (path_to_lic_dir, path_to_swk_dir, act_device_sessions) = if ismaster {
        (
            get_path_in_cach("master_lic"),
            get_path_in_cach("master_swk"),
            get_master_device()
        )
    } else {
        (
            get_path_in_cach("slave_lic"),
            get_path_in_cach("slave_swk"),
            get_slave_device()
        )
    };
    let path_lic = Path::new(&licfilepath);
    let path_swk = Path::new(&swkfilepath);
    
    let Some(lic_name) = path_lic.file_name() else {
        return ;
    };
    let Some(lic_name_str) = lic_name.to_str() else {
        return ();
    };
    let Some(swk_name) = path_swk.file_name() else {
        return ();
    };
    let Some(swk_name_str) = swk_name.to_str() else {
        return ();
    };
    
    let file_path = path_to_lic_dir.join(lic_name);
    
    fs::copy(&path_lic, &file_path).await.unwrap();
    let file_path_swk = path_to_swk_dir.join(swk_name);
    println!("file_path_swk {:?}", file_path_swk);

    fs::copy(&path_swk, &file_path_swk).await.unwrap();
   
    let Some(session) = act_device_sessions.session else {
        println!("err 5");
        return ()
    };
    let config_step = ConfigStep::SendingFiles(StepState::Expectation(0));
    let device_type = act_device_sessions.device.unwrap().device_type;
    change_device_config_step(&device_type, config_step);
    session.send_stdin("echo [ STATE_SNDINGFILE_START ]\n".into()).await.unwrap();

    let command = format!("sudo echo 'GET LIC' | nc localhost {} > ./{}\n", PORTSSHTUNNEL, lic_name_str);
    session.send_stdin(command.into()).await.unwrap();
    let mvcommand = format!("sudo mv ./{:?} /usr/share/rtu5_installator/licenses/\n", lic_name_str);
    session.send_stdin(mvcommand.into()).await.unwrap();

    let commandswk = format!("sudo echo 'GET SWK' | nc localhost {} > ./{:?}\n", PORTSSHTUNNEL, swk_name_str);
    session.send_stdin(commandswk.into()).await.unwrap();
    let mvcommandswk = format!("sudo mv ./{:?} /usr/share/rtu5_installator/licenses/\n", swk_name_str);
    session.send_stdin(mvcommandswk.into()).await.unwrap();

    let config_step2 = ConfigStep::SendingFiles(StepState::Success(0));
    change_device_config_step(&device_type, config_step2);
    session.send_stdin("echo [ STATE_SNDINGFILE_STOP ]\n".into()).await.unwrap();
    session.send_stdin("sudo ls /usr/share/rtu5_installator/licenses/\n".into()).await.unwrap();

}


#[tauri::command]
async fn download_file(filepath: String, ismaster: bool) -> Option<String> {
    let (path_to_dir, device_sessions) = if ismaster {
        (get_path_in_cach("master_input"), get_master_device())
    } else {
        (get_path_in_cach("slave_input"), get_slave_device())
    };
    
    let file_name = find_latest_file(path_to_dir).await;
    println!("file_name {:?}", file_name);
    match file_name {
        Some(file) => {
            let downloads_dir = Path::new(&filepath);
            let Some(name) = file.file_name() else {
                return None;
            };

            let new_path = downloads_dir.join(name);
            let new_file = new_path.clone();
            fs::rename(file, new_path).await.unwrap();
            
            let answer = new_file.to_str();
            match answer {
                Some(a) => {
                    let config_step = ConfigStep::ArchiveDownload(StepState::Success(0));
                    change_device_config_step(&device_sessions.device.unwrap().device_type, config_step);
                    return Some(a.to_string());

                }
                _ => {}
            }
        }
        
        _ => {}
    }
    let config_step = ConfigStep::ArchiveDownload(StepState::Failed(100));
    change_device_config_step(&device_sessions.device.unwrap().device_type, config_step);
    None
}


#[tauri::command]
fn get_general_alert() -> Vec<AlertMessage> {
    let alert_type = AlertType::General;
    let alerts = get_alerts_by_type(alert_type);
    alerts.clone()
}


#[tauri::command]
fn get_terminal_master() -> Vec<AlertMessage> {
    let alert_type = AlertType::Master;
    let alerts = get_alerts_by_type(alert_type);
    alerts.clone()
}


#[tauri::command]
fn get_terminal_slave() -> Vec<AlertMessage> {
    let alert_type = AlertType::Slave;
    let alerts = get_alerts_by_type(alert_type);
    alerts.clone()
}


#[tauri::command]
fn get_device(ismaster: bool) -> Option<Device> {
    let device_sessions = if ismaster {
        get_master_device()
    } else {
        get_slave_device()

    };
    // Получаем доступ к MutexGuard
    device_sessions.device
}

#[tauri::command]
fn get_state() -> StateData {
    let master = get_master_device();
    let slave = get_slave_device();
    let (master_state, master_code) = process_config_step(&master.config_step);
    let (slave_state, slave_code) = process_config_step(&slave.config_step);
    

    let state_info = StateData {
        master_state,
        master_code,
        slave_state,
        slave_code,
    };
    state_info
}

#[tauri::command]
async fn scan_device(ismaster: bool) -> () {
    let device_sessions= if ismaster {
        let device: DeviceSession = get_master_device();
        device
    } else {
        let device: DeviceSession = get_slave_device();
        device
    };

    let config_step = ConfigStep::Scan(StepState::Expectation(0));
    change_device_config_step(&device_sessions.device.unwrap().device_type, config_step);

    let belong = AlertType::General;
    send_alert(AlertMessage::new(belong, String::from("Сканирование сервера")));

    let Some(session) = device_sessions.session else {
        return ()
    };
    let command_data = get_command_by_name("scan".to_string());
    session.send_stdin("echo [ STATE_SCAN_START ]\n".into()).await.unwrap();
    //session.send_stdin("sed -i 's/\r$//' scan.sh\n".into()).await.unwrap();
    session.send_stdin(command_data.command.into()).await.unwrap();
    session.send_stdin("echo [ STATE_SCAN_STOP ]\n".into()).await.unwrap();

}

#[tauri::command]
async fn create_archive(ismaster: bool) -> () {
    let device_sessions = if ismaster {
        let device: DeviceSession = get_master_device();
        device
    } else {
        let device: DeviceSession = get_slave_device();
        device
    };

    let config_step = ConfigStep::ArchiveGeneration(StepState::Expectation(0));
    change_device_config_step(&device_sessions.device.unwrap().device_type, config_step);

    let belong = AlertType::General;
    send_alert(AlertMessage::new(belong, String::from("Создание архива")));

    let Some(session) = device_sessions.session else {
        return ()
    };
    let company_name = get_company_info().companyname;
    let command_export_value = format!("export COMPANY_NAME='{company_name}'\n");
    println!("{:?}",command_export_value);
    let command_data = get_command_by_name("ArchivGen".to_string());
    session.send_stdin("echo [ STATE_ARCHIVGEN_START ]\n".into()).await.unwrap();
    
    session.send_stdin(command_export_value.into()).await.unwrap();
    session.send_stdin(command_data.command.into()).await.unwrap();
    session.send_stdin("echo [ STATE_ARCHIVGEN_STOP ]\n".into()).await.unwrap();

}





#[tauri::command]
async fn send_command(ismaster: bool, command: String) -> () {
    let device_sessions= if ismaster {
        get_master_device()
    } else {
        get_slave_device()
    };
    let Some(session) = device_sessions.session else {
        return ()
    };
    session.send_stdin(command.into()).await.unwrap();

}


#[tauri::command]
async fn send_signal(ismaster: bool, signal: String) -> () {
    let device_sessions= if ismaster {
        get_master_device()
    } else {
        get_slave_device()
    };
    let Some(session) = device_sessions.session else {
        return ()
    };
    _ = session.signal(&signal);
}


#[tauri::command]
fn create_company_name(companyname: String) -> String {
    set_company_info(&companyname);
    companyname
}


#[tauri::command]
fn get_company_name() -> String {
    let company_info = get_company_info();
    company_info.companyname
}


#[tauri::command]
async fn restart_session(ismaster: bool) -> () {
    let mut device_sessions= if ismaster {
        get_master_device()
    } else {
        get_slave_device()
    };
    
    let device: Option<Device> = device_sessions.clone_device();
    
    device_sessions.abort_session();
    device_sessions.delete_all();
    
    let Some(device) = device else {
        return ()
    };
    connect_to_server(device).await;
    let belong = AlertType::General;
    send_alert(AlertMessage::new(belong, String::from("Выполнен вход")));
}


#[tauri::command]
async fn create_device(
    ipaddress: String,
    port: u16,
    username: String,
    password: Option<String>,
    sudopassword: Option<String>,
    file_key: Option<String>,
    ismaster: bool
) ->() {

    let ip_address: IpAddr = ipaddress.parse().expect("ip not found");
    let device_type = if ismaster {
        DeviceType::Master
    } else {
        DeviceType::Slave
    };
    let connect_type: ConnectType = if file_key != Some("".to_string()) {
        if password.is_some() {
            ConnectType::FilekeyPassword
        } else {
            ConnectType::Filekey
        }
    } else {
        ConnectType::Password
    };

    let credentials = Credentials::new(
        username,
        connect_type,
        file_key,
        password,
        sudopassword
    );

    let new_device = Device::new(
        credentials,
        ip_address,
        port,
        device_type
    );

    connect_to_server(new_device).await;
    let belong = AlertType::General;
    send_alert(AlertMessage::new(belong, String::from("Добавлен новый сервер")))
}


fn main() {
    _ = create_data_folder();
   
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        get_device, create_device,  scan_device, get_general_alert, get_terminal_master,
        get_terminal_slave, send_command, send_signal, restart_session, get_state, create_company_name,
        get_company_name, create_archive, download_file, upload_file, get_device_ip, start_install,
        delete_lic_swk, delete_device, start_config_program, change_state, start_config_db, 
        start_config_db, start_coropace

    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
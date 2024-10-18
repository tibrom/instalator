use std::sync::{Mutex, Arc};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use tokio;
use crate::device::{Device, DeviceType};
use makiko::Session;
use serde_yaml;
use std::fs;

//use app::{Credentials, Device, DeviceState, ConnectType};

lazy_static! {
    pub static ref CLEARSTDOUTMASTER: Mutex<Vec<String>> = Mutex::new(Vec::new());
    pub static ref CLEARSTDOUTSLAVE: Mutex<Vec<String>> = Mutex::new(Vec::new());
    pub static ref ALERT: Mutex<Vec<AlertMessage>> = Mutex::new(Vec::new());
    pub static ref MASTERDEVICES: Mutex<DeviceSession> = Mutex::new(DeviceSession::new());
    pub static ref SLAVEDEVICES: Mutex<DeviceSession> = Mutex::new(DeviceSession::new());
    pub static ref COMPANYINFO: Mutex<CompanyInfo> = Mutex::new(CompanyInfo::init());
    pub static ref MASTERINPUTFILE: Mutex<String> = Mutex::new("".to_string());
    pub static ref SVAVEINPUTFILE: Mutex<String> = Mutex::new("".to_string());
}


#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub command: String,
    pub check_state: Vec<String>,
}


#[derive(Debug, Deserialize, Clone)]
struct Config {
    commands: Vec<Command>,
}


lazy_static! {
    static ref COMMANDS: Arc<Mutex<Vec<Command>>> = {
        let file_path = "commands.yaml";
        let yaml_content = fs::read_to_string(file_path).expect("Failed to read commands.yaml");
        let config: Config = serde_yaml::from_str(&yaml_content).expect("Failed to parse YAML");

        Arc::new(Mutex::new(config.commands))
    };
}

pub fn get_command_by_name(name: String) -> Command {
    let cmds = COMMANDS.lock().unwrap();
    for command in cmds.iter() {
        if command.name == name {
            let result = command.clone();
            return result
        }
    };
    panic!("Comand not found")
}



//clear stdout
pub fn get_clear_stdout(device_type: &DeviceType) -> Vec<String> {
    let std_out = match *device_type {
        DeviceType::Master => {
            CLEARSTDOUTMASTER.lock().unwrap()
        },
        _ => {
            CLEARSTDOUTSLAVE.lock().unwrap()
        }
    };
    std_out.clone()
}

pub fn add_clear_stdout(device_type: &DeviceType, std: Vec<String>) -> Vec<String> {
    let mut std_out = match *device_type {
        DeviceType::Master => {
            CLEARSTDOUTMASTER.lock().unwrap()
        },
        _ => {
            CLEARSTDOUTSLAVE.lock().unwrap()
        }
    };
    std_out.extend(std);
    std_out.clone()
}

//alerts
#[derive(Debug, Clone, Serialize,  PartialEq, Eq, Copy)]
pub enum AlertType{
    Master,
    Slave,
    General
}


#[derive(Debug, Clone, Serialize)]
pub struct AlertMessage{
    pub belong: AlertType,
    pub message: String
}

impl AlertMessage {
    pub fn new(belong: AlertType, message: String) -> Self {
        AlertMessage { belong, message }
    }
}


pub fn send_alert(message: AlertMessage) {
    let mut alarm = ALERT.lock().unwrap();

    // Проверяем, больше ли в векторе 1000 элементов
    if alarm.len() > 1000 {
        // Удаляем первые 100 элементов
        println!("ТЕРМИНАЛ ПЕРЕПОЛНЕН");
        alarm.drain(0..100);
    }

    // Добавляем новое сообщение
    alarm.push(message);
}

pub fn get_alerts_by_type(alert_type: AlertType) -> Vec<AlertMessage> {
    let alerts = ALERT.lock().unwrap();
    alerts
        .iter()
        .filter(|alert| alert.belong == alert_type)
        .cloned() 
        .collect()
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StepState {
    Success(usize),
    Expectation(usize),
    Failed(usize)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConfigStep {
    Create(StepState),
    Scan(StepState),
    ArchiveGeneration(StepState),
    ArchiveDownload(StepState),
    SendingFiles(StepState),
    ProgramInstallation(StepState),
    Configurator(StepState),
    DbConfigurator(StepState),
    Coropase(StepState),
    SendData(StepState),
}



pub struct DeviceSession{
    pub device: Option<Device>,
    pub session: Option<Session>,
    pub session_event_task: Option<tokio::task::JoinHandle<()>>, 
    pub client_event_task: Option<tokio::task::JoinHandle<()>>,
    pub config_step: ConfigStep
}

impl DeviceSession {
    pub fn new() -> DeviceSession {
        DeviceSession {
            device: None,
            session: None,
            session_event_task: None,
            client_event_task: None,
            config_step: ConfigStep::Create(StepState::Failed(100))
        }
    }
    pub fn clone(&self) -> Self {
        DeviceSession {
            device: self.device.clone(),
            session: self.session.clone(),
            session_event_task: None,
            client_event_task: None, // Игнорируем JoinHandle при клонировании
            config_step: self.config_step.clone(),
        }
    }
    pub fn replace_device(&mut self, new_device: Device) {
        self.device = Some(new_device);
    }
    pub fn replace_session(
        &mut self,
        new_session: Session,
        session_event_task: tokio::task::JoinHandle<()>,
        client_event_task: tokio::task::JoinHandle<()>,
    ) {
        self.session = Some(new_session);
        self.session_event_task = Some(session_event_task);
        self.client_event_task = Some(client_event_task)
    }
    pub fn abort_session(&mut self) {
        let Some(ref session_event_task) = self.session_event_task else {
            return ()
        };
        session_event_task.abort();
        let Some(ref client_event_task) = self.client_event_task else {
            return ()
        };
        client_event_task.abort();

    }

    pub fn delete_all(&mut self) {
        self.abort_session();
        self.session = None;
        self.device = None;
    }
    pub fn clone_device(&mut self) -> Option<Device>{
        self.device.clone()
    }
    pub fn set_config_state(&mut self, config_step: ConfigStep) -> (){
        self.config_step = config_step
    }
}

pub fn delete_device_data(device_type: &DeviceType) {
    let mut core_device = if *device_type == DeviceType::Master {
        MASTERDEVICES.lock().unwrap()
    } else {
        SLAVEDEVICES.lock().unwrap()
    };
    core_device.delete_all();
}
pub fn replace_device(device: Device) {
    let mut core_device = if device.device_type == DeviceType::Master {
        MASTERDEVICES.lock().unwrap()
    } else {
        SLAVEDEVICES.lock().unwrap()
    };
    core_device.replace_device(device);
}

pub fn replace_session(
    new_session: Session,
    device_type: &DeviceType,
    session_event_task: tokio::task::JoinHandle<()>,
    client_event_task: tokio::task::JoinHandle<()>,

) {
    let mut core_device = if *device_type == DeviceType::Master {
        MASTERDEVICES.lock().unwrap()
    } else {
        SLAVEDEVICES.lock().unwrap()
    };
    core_device.replace_session(new_session, session_event_task, client_event_task)
}

pub fn get_master_device() -> DeviceSession {
    let mastersession = MASTERDEVICES.lock().unwrap();
    mastersession.clone()
}

pub fn get_slave_device() -> DeviceSession {
    let slavesession = SLAVEDEVICES.lock().unwrap();
    slavesession.clone()
}

pub fn change_device_config_step(device_type: &DeviceType, config_step: ConfigStep) -> () {
    println!("config step {:?}", config_step);
    let mut core_device = if *device_type == DeviceType::Master {
        MASTERDEVICES.lock().unwrap()
    } else {
        SLAVEDEVICES.lock().unwrap()
    };
    core_device.set_config_state(config_step);
    println!("{:?}",core_device.config_step);
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompanyInfo {
    pub companyname: String
}

impl CompanyInfo {

    pub fn init() -> CompanyInfo {
        CompanyInfo {
            companyname: "Not Found".to_string(),
        }
    }

    pub fn set(&mut self, companyname: String) {
        self.companyname = companyname;
    }
}

pub fn get_company_info() -> CompanyInfo {
    let company_info = COMPANYINFO.lock().unwrap();
    company_info.clone()
}

pub fn set_company_info(companyname: &str) -> () {
    let mut company_data = COMPANYINFO.lock().unwrap();
    company_data.set(companyname.to_string());
}

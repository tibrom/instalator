use serde::{Serialize, Deserialize};
use std::net::IpAddr;

const DEFAULT_VALUE: &str = "_";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectType{
    Password,
    Filekey,
    FilekeyPassword,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceState{
    Gone(String),
    Creating(String),
    Scanning(String),
    Settings(String),
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceType{
    Master,
    Slave
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub connect_type: ConnectType,
    pub file_key: String,
    pub password: String,
    pub sudopassword: String,
}

impl Credentials {
    pub fn new(
        username: String,
        connect_type: ConnectType,
        file_key: Option<String>,
        password: Option<String>,
        sudopassword: Option<String>
    ) -> Credentials {
        let mut cred = Credentials {
            username,
            connect_type,
            file_key: DEFAULT_VALUE.to_string(),
            password:  DEFAULT_VALUE.to_string(),
            sudopassword:  DEFAULT_VALUE.to_string(),
        };

        match cred.connect_type {
            ConnectType::Password => {
                if let Some(p) = password {
                    cred.password = p;
                } else {
                    panic!("Password not found!")
                }

                if let Some(sp) = sudopassword {
                    cred.sudopassword =sp;
                } else {
                    cred.sudopassword = cred.password.clone();
                }
            }
            ConnectType::Filekey => {
                if let Some(fk) = file_key {
                    cred.file_key = fk;
                } else {
                    panic!("File_key not found!")
                }

                if let Some(sp) = sudopassword {
                    cred.sudopassword = sp;
                }
            }
            ConnectType::FilekeyPassword => {
                if let Some(p) = password {
                    cred.password = p;
                } else {
                    panic!("Password not found!")
                }

                if let Some(sp) = sudopassword {
                    cred.sudopassword = sp;
                } else {
                    cred.sudopassword = cred.password.clone();
                }

                if let Some(fk) = file_key {
                    cred.file_key = fk;
                } else {
                    panic!("File_key not found!")
                }
            }
        }
        cred
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub ip_address: IpAddr,
    pub credentials: Credentials,
    pub state: DeviceState,
    pub port: u16,
    pub device_type: DeviceType,
}

impl Device {
    pub fn new(
        credentials: Credentials,
        ip_address: IpAddr,
        port: u16,
        device_type: DeviceType
    ) -> Device {
        Device{
            ip_address: ip_address,
            credentials: credentials,
            state: DeviceState::Creating(String::from("create")),
            port: port,
            device_type: device_type
        }
    }
}

use makiko::bytes::Bytes;
use std::sync::Arc;

use crate::device::{Device, DeviceType, ConnectType,};
use crate::remoute::command_mgmt::{EventHandlers, TunnelHandlers, ClientEventHandler,
    auth_with_password, session_create, create_client, auth_with_key
    };
use crate::state::{AlertMessage, AlertType, StepState, ConfigStep, replace_device,
    replace_session, send_alert,  change_device_config_step};
use crate::stdout_handlers::hendler_stdout_device;
use crate::utils::{get_path_in_cach, get_path_file_cach};
use crate::config::PORTSSHTUNNEL;






pub async fn connect_to_server(device: Device) -> () {
    let host=device.ip_address.to_string();
    let port=device.port;
    let (share_files,
        temp_file_name,
        alert_type) = match &device.device_type {
        DeviceType::Master => {
            (vec![
                ("GET LIC".to_string(), get_path_in_cach("master_lic")),
                ("GET SWK".to_string(), get_path_in_cach("master_swk")),
            ], "master.tar.bz2", AlertType::Master)
        }
        _ => {
            (vec![
                ("GET LIC".to_string(), get_path_in_cach("slave_lic")),
                ("GET SWK".to_string(), get_path_in_cach("slave_swk")),
            ], "slave.tar.bz2", AlertType::Slave)
        }
    };
    let path_to_temp_file = get_path_file_cach(vec!["temp", temp_file_name]);
    let tunnel_handler: TunnelHandlers = TunnelHandlers::new(share_files, path_to_temp_file);
    let client_handler = ClientEventHandler::new(tunnel_handler);
    let (client, client_even_task) = create_client(&host, port.clone(), client_handler).await;
    _ = client.bind_tunnel(("localhost".to_string(), PORTSSHTUNNEL));

    let username = device.credentials.username.clone();
    let password = device.credentials.password.clone();
    let file_key = device.credentials.file_key.clone();

    match device.credentials.connect_type {
        ConnectType::Filekey => {
            auth_with_key(&client, &username, &file_key).await;
        },
        ConnectType::FilekeyPassword => {
            auth_with_key(&client, &username, &file_key).await;
        },
        ConnectType::Password => {
            auth_with_password(&client, &username, &password).await;
        }
    };
    
    let device_type = device.device_type.clone();

    let handlers: EventHandlers = EventHandlers::new(
        Some(Arc::new(move |data: Bytes| {
            
            let output = std::str::from_utf8(&data).expect("Invalid UTF-8 sequence");
            hendler_stdout_device(&device_type, &output);
            send_alert(AlertMessage::new(alert_type, output.to_string()));
            
            //println!("Process produced stdout: {:?}", output);
        })),
        None,
        None,
        None,
    );
    let (new_session, session_event_task) = session_create(&client, handlers).await;

    let sudopassword = device.credentials.sudopassword.clone();
    let _ = new_session.shell();
    let cmmnd = format!("echo {} | sudo -S su \n", sudopassword);
    new_session.send_stdin(cmmnd.into()).await.unwrap();

    replace_session(
        new_session,
        &device.device_type,
        session_event_task,
        client_even_task
    );
    let config_step = ConfigStep::Create(StepState::Success(0));
    change_device_config_step(&device.device_type, config_step);
    replace_device(device);
    
}
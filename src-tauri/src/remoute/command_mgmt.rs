use makiko::{keys, AuthPasswordResult, AuthPubkeyResult, ChannelConfig, Client, ClientConfig, ClientEvent,
    ExitSignal, Session, SessionEvent, Tunnel, AcceptTunnel};
use makiko::bytes::Bytes;
use makiko::TunnelEvent::Data;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use std::sync::Arc;
use tokio::fs::{self, read_dir};
use std::path::{PathBuf};
use std::time::SystemTime;


pub struct ClientEventHandler {
    pub client_accept: Arc<dyn Fn(AcceptTunnel) + Send + Sync>,
}

impl ClientEventHandler {
    pub fn new(tunnel_event: TunnelHandlers) -> Self {
        let tunnel_event = Arc::new(tunnel_event); // Заворачиваем tunnel_event в Arc для безопасного клонирования
        let client_accept = Arc::new(move |accept: AcceptTunnel| {
            let tunnel_event = Arc::clone(&tunnel_event); // Клонируем tunnel_event
            tokio::task::spawn(async move {
                let config_data = ChannelConfig::default().with(|config| {
                    config.recv_window_max = 2 * 1024 * 1024; // 2 МБ
                    config.recv_packet_len_max = 1 * 1024 * 1024; // 1 МБ
                });
                println!("config_data {}", config_data.recv_window_max);
                let tunnel_data = accept.accept(config_data).await;
                match tunnel_data {
                    Ok(result) => {
                        let (tunnel, mut tunnel_resv) = result;
                        let tunnel_event = Arc::clone(&tunnel_event); // Клонируем tunnel_event
                        tokio::task::spawn(async move {
                            loop {
                                let event = tunnel_resv.recv().await
                                    .expect("Error while receiving tunnel event");

                                let Some(event) = event else {
                                    break;
                                };

                                match event {
                                    Data(bytes) => {
                                        (tunnel_event.data_bytes)(bytes, tunnel.clone());
                                    },
                                    _ => {}
                                }
                            }
                        });
                    }
                    _ => {}
                }
            });
        });

        ClientEventHandler {
            client_accept,
        }
    }
}


pub struct TunnelHandlers {
    pub data_bytes: Arc<dyn Fn(Bytes, Tunnel) + Send + Sync>,
}

impl TunnelHandlers {
    pub fn new(share_files: Vec<(String, PathBuf)>, path_to_temp_file: PathBuf) -> Self {
        // Перемещаем данные в замыкание с помощью `move`
        let default_handler: Arc<dyn Fn(Bytes, Tunnel) + Send + Sync> = Arc::new(move |bytes: Bytes, tunnel: Tunnel| {
            let mut allowed_keys = Vec::new();
            for (k, _) in &share_files {
                allowed_keys.push(k.to_string());
            }
            let mut is_reqwest = false;
            let value = share_files.clone();
            let value_path_to_temp_file = path_to_temp_file.clone();

            tokio::task::spawn(async move {
                let mut key = "".to_string();
                if bytes.len() < 64 {
                    if let Ok(data_str) = String::from_utf8(bytes.to_vec()) {
                        key.push_str(data_str.trim());
                        println!("allowed_keys {:?}", allowed_keys);
                        if allowed_keys.contains(&key) {
                            is_reqwest = true;
                        }
                    }
                }
                if is_reqwest {
                    let mut buffer = [0; 4096];
                    let file_dir = TunnelHandlers::search_dir(value, &key);
                   
                    match file_dir {
                        Some(dir) => {
                            let file_option_data = TunnelHandlers::find_latest_file(dir).await;
                            match file_option_data {
                                Some(file) => {
                                    if let Ok(mut open_file) = tokio::fs::File::open(&file).await {
                                        loop {
                                            let bytes_read = open_file.read(&mut buffer).await.unwrap();
                                            println!("bytes_read, {:?}", bytes_read);
                                            if bytes_read == 0 {
                                                println!();
                                                break;
                                            }
                                            let data = Bytes::copy_from_slice(&buffer[..bytes_read]);
                                            let answer = tunnel.send_data(data).await.unwrap();
                                            println!("answer, {:?}", answer);
                                            //

                                        }
                                        tunnel.send_eof().await.unwrap();
                                    };
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    
                } else {
                    let dir = value_path_to_temp_file.parent().unwrap();
                    println!("dir {:?}", dir);
                    // Создаем директорию, если она не существует
                    fs::create_dir_all(dir).await.unwrap();

                    // Создаем и записываем данные в файл
                    let mut write_file = fs::File::create(&value_path_to_temp_file).await.unwrap();
                    write_file.write_all(&bytes).await.expect("Failed to write data to file");

        

                    //let mut write_file = tokio::fs::File::create(&value_path_to_temp_file.as_path()).await.unwrap();
                    //write_file.write_all(&bytes).await.expect("Failed to write data to file");
                    tunnel.send_eof().await.unwrap();
                }
            });
        });
        
        TunnelHandlers {
            data_bytes: default_handler,
        }
    }

    pub fn get_allowed_keys(share_files: Vec<(String, String)>) -> Vec<String> {
        let mut allowed_keys = Vec::new();
        for (_, v) in share_files {
            allowed_keys.push(v.to_string());
        }
        allowed_keys
    }

    pub async fn search_file(share_files: Vec<(String, String)>, key: &str) -> Option<tokio::fs::File> {
        let s_key = key.to_string();
        for (k, v) in share_files {
            if k == s_key {
                if let Ok(file) = tokio::fs::File::open(&v).await {
                    return Some(file);
                }
            }
        }
        None
    }

    pub fn search_dir(share_files: Vec<(String, PathBuf)>, key: &str) -> Option<PathBuf> {
        let s_key = key.to_string();
        for (k, v) in share_files {
            if k == s_key {
                return  Some(v);
            }
        }
        None
    }

    async fn find_latest_file(path: PathBuf) -> Option<PathBuf> {
        let mut dir = read_dir(&path).await.ok()?;  // Читаем содержимое директории
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

    
}


pub struct EventHandlers {
    pub stdout_handler: Arc<dyn Fn(Bytes) + Send + Sync>,
    pub stderr_handler: Arc<dyn Fn(Bytes) + Send + Sync>,
    pub exitstatus_handler: Arc<dyn Fn(i32) + Send + Sync>,
    pub exitsignal_handler: Arc<dyn Fn(ExitSignal) + Send + Sync>,
}

impl EventHandlers {
    // Функция new с обработчиками и значениями по умолчанию
    pub fn new(
        stdout_handler: Option<Arc<dyn Fn(Bytes) + Send + Sync>>,
        stderr_handler: Option<Arc<dyn Fn(Bytes) + Send + Sync>>,
        exitstatus_handler: Option<Arc<dyn Fn(i32) + Send + Sync>>,
        exitsignal_handler: Option<Arc<dyn Fn(ExitSignal) + Send + Sync>>,
    ) -> Self {
        // Значения по умолчанию для обработчиков
        let default_stdout_handler: Arc<dyn Fn(Bytes) + Send + Sync> = Arc::new(|data: Bytes| {
            println!("Process produced stdout: {:?}", data);
        });

        let default_stderr_handler: Arc<dyn Fn(Bytes) + Send + Sync> = Arc::new(|data: Bytes| {
            println!("Process produced stderr: {:?}", data);
        });

        let default_exitstatus_handler: Arc<dyn Fn(i32) + Send + Sync> = Arc::new(|status: i32| {
            println!("Process exited with status {}", status);
        });

        let default_exitsignal_handler: Arc<dyn Fn(ExitSignal) + Send + Sync> = Arc::new(|signal| {
            println!("Process exited with signal {:?}: {:?}", signal.signal_name, signal.message);
        });

        Self {
            stdout_handler: stdout_handler.unwrap_or(default_stdout_handler),
            stderr_handler: stderr_handler.unwrap_or(default_stderr_handler),
            exitstatus_handler: exitstatus_handler.unwrap_or(default_exitstatus_handler),
            exitsignal_handler: exitsignal_handler.unwrap_or(default_exitsignal_handler),
        }
    }
}

pub async fn create_client(host:&str, port:u16, client_handler: ClientEventHandler) -> (Client, tokio::task::JoinHandle<()>) {
    let socket = tokio::net::TcpStream::connect((host, port)).await
        .expect("Could not open a TCP socket");

    let config = ClientConfig::default();
    let (client, mut client_rx, client_fut) = Client::open(socket, config)
        .expect("Could not open client");

    tokio::task::spawn(async move {
        client_fut.await.expect("Error in client future");
    });
    let client_loop = tokio::task::spawn(async move {
        loop {
            let event = client_rx.recv().await
                .expect("Error while receiving client event");

            let Some(event) = event else {
                break
            };

            match event {
                ClientEvent::ServerPubkey(pubkey, accept) => {
                    println!("Server pubkey type {}, fingerprint {}", pubkey.type_str(), pubkey.fingerprint());
                    accept.accept();
                },
                ClientEvent::Tunnel(accept) => {
                    (client_handler.client_accept)(accept);
                }
                t => {println!("client event {:?}", t)},
            };
        }
    });
    (client, client_loop)
}


pub async fn auth_with_password(client: &Client, username:&str, password:&str) ->() {
    // Try to authenticate using a password.
    let auth_res = client.auth_password(username.into(), password.into()).await
        .expect("Error when trying to authenticate");

    // Deal with all possible outcomes of password authentication.
    match auth_res {
        AuthPasswordResult::Success => {
            println!("We have successfully authenticated using a password");
        },
        AuthPasswordResult::ChangePassword(prompt) => {
            panic!("The server asks us to change password: {:?}", prompt);
        },
        AuthPasswordResult::Failure(failure) => {
            panic!("The server rejected authentication: {:?}", failure);
        }
    };
    
}


pub async fn auth_with_key(client: &Client, user:&str, file_key:&str) -> () {
    //let key_file = r"C:\Users\gorlanov\.ssh\id_rsa";
    
    //let privkey_pem = fs::read(file_key)
    //    .await
    //    .expect("Could not decode a private key from PEM");

    let privkey_pem: Vec<u8> = file_key.as_bytes().to_vec();

    let privkey = keys::decode_pem_privkey_nopass(&privkey_pem)
        .expect("Could not decode a private key from PEM")
        .privkey().cloned()
        .expect("Private key is encrypted");

        let pubkey = privkey.pubkey();

        // Get the public key algorithms supported by the key.
        let available_algos = pubkey.algos();
    
        // Try the algorithms one by one.
        let username: String = user.into();
        for pubkey_algo in available_algos.iter().copied() {
            // Check whether this combination of a public key and algorithm would be acceptable to the
            // server.
            let check_ok = client.check_pubkey(username.clone(), &pubkey, pubkey_algo).await
                .expect("Error when checking a public key");
    
            // Skip this algorithm if the server rejected it.
            if !check_ok {
                println!("Server rejected public key and algorithm {:?}", pubkey_algo.name);
                continue;
            }
    
            // Try to authenticate using this algorithm.
            let auth_res = client.auth_pubkey(username.clone(), privkey.clone(), pubkey_algo).await
                .expect("Error when trying to authenticate");
            match auth_res {
                AuthPubkeyResult::Success => {
                    println!("We have successfully authenticated using algorithm {:?}", pubkey_algo.name);
                    break;
                },
                AuthPubkeyResult::Failure(_) => {
                    println!("Authentication using public key and algorithm {:?} failed", pubkey_algo.name);
                },
            }
        }
    
        // Check that we have been authenticated.
        if !client.is_authenticated().unwrap() {
            panic!("Could not authenticate");
        }
}




pub async fn session_create(client: &Client, handlers: EventHandlers) -> (Session, tokio::task::JoinHandle<()>) {
    let channel_config = ChannelConfig::default().with(|config| {
        config.recv_window_max = 2 * 1024 * 1024; // 2 МБ
        config.recv_packet_len_max = 1 * 1024 * 1024; // 1 МБ
    });
    let (session, mut session_rx) = client.open_session(channel_config).await
        .expect("Could not open a session");

    let session_event_task = tokio::task::spawn(async move {
        loop {
            // Wait for the next event.
            let event = session_rx.recv().await
                .expect("Error while receiving session event");

            // Exit the loop when the session has closed.
            let Some(event) = event else {
                break
            };

            match event {
                // Handle stdout/stderr output from the process.
                SessionEvent::StdoutData(data) => {
                    (handlers.stdout_handler)(data);
                },
                SessionEvent::StderrData(data) => {
                    (handlers.stderr_handler)(data);
                },

                // Handle exit of the process.
                SessionEvent::ExitStatus(status) => {
                    (handlers.exitstatus_handler)(status.try_into().unwrap());
                },
                SessionEvent::ExitSignal(signal) => {
                    (handlers.exitsignal_handler)(signal);
                },

                // Ignore other events
                _ => {},
            }
        }
    });
    
    (session, session_event_task)
}




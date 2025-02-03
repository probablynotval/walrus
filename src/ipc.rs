use crate::{commands::Commands, utils::SOCKET_PATH};

use log::{debug, error};
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::mpsc::Sender,
    thread,
};

impl Commands {
    fn to_bytes(&self) -> Option<u8> {
        match self {
            Commands::Config => None,
            Commands::Next => Some(1),
            Commands::Pause => Some(2),
            Commands::Previous => Some(3),
            Commands::Resume => Some(4),
            Commands::Reload => Some(5),
            Commands::Shutdown => Some(100),
        }
    }

    fn from_byte(byte: u8) -> Option<Commands> {
        match byte {
            1 => Some(Commands::Next),
            2 => Some(Commands::Pause),
            3 => Some(Commands::Previous),
            4 => Some(Commands::Resume),
            5 => Some(Commands::Reload),
            100 => Some(Commands::Shutdown),
            _ => None,
        }
    }
}

pub struct IPCServer {
    socket_path: String,
}

impl IPCServer {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub fn start(&self, tx: Sender<Commands>) -> io::Result<()> {
        if Path::new(&self.socket_path).exists() {
            fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = [0; 1];
                        if stream.read_exact(&mut buffer).is_ok() {
                            if let Some(command) = Commands::from_byte(buffer[0]) {
                                match command {
                                    Commands::Config => (),
                                    Commands::Next => {
                                        debug!("IPC received Next command");
                                        let _ = tx.send(Commands::Next);
                                    }
                                    Commands::Pause => {
                                        debug!("IPC received Pause command");
                                        let _ = tx.send(Commands::Pause);
                                    }
                                    Commands::Previous => {
                                        debug!("IPC received Previous command");
                                        let _ = tx.send(Commands::Previous);
                                    }
                                    Commands::Resume => {
                                        debug!("IPC received Resume command");
                                        let _ = tx.send(Commands::Resume);
                                    }
                                    Commands::Reload => {
                                        debug!("IPC received Reload command");
                                        let _ = tx.send(Commands::Reload);
                                    }
                                    Commands::Shutdown => {
                                        debug!("IPC received Shutdown command");
                                        let _ = tx.send(Commands::Shutdown);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => error!("Error accepting connection: {e:#?}"),
                }
            }
        });

        Ok(())
    }
}

pub struct IPCClient {
    socket_path: String,
}

impl IPCClient {
    fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    fn send_command(&self, command: Commands) -> io::Result<()> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        let Some(cmd) = command.to_bytes() else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to convert command to bytes",
            ));
        };
        stream.write_all(&[cmd])?;
        Ok(())
    }
}

pub fn setup_ipc(tx: Sender<Commands>) -> io::Result<()> {
    debug!("Starting IPC server");
    // TODO: This socket path should actually be in XDG_RUNTIME_DIR, if I am not mistaken.
    // Currently it's in /tmp. I should probably use /tmp/walrus/walrus.sock for fallback. Just
    // remember to cleanup.
    let server = IPCServer::new(String::from(SOCKET_PATH));
    server.start(tx)?;
    Ok(())
}

pub fn send_ipc_command(command: Commands) -> io::Result<()> {
    debug!("IPC sending {:?} command", command);
    // TODO: This socket path should actually be in XDG_RUNTIME_DIR, if I am not mistaken.
    // Currently it's in /tmp. I should probably use /tmp/walrus/walrus.sock for fallback. Just
    // remember to cleanup.
    let client = IPCClient::new(String::from(SOCKET_PATH));
    client.send_command(command)
}

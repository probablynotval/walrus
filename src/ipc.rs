use std::{
    fs,
    io::{self, Read, Write},
    marker::PhantomData,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::mpsc::Sender,
    thread,
};

use log::{debug, error, warn};

use crate::{
    commands::Commands,
    utils::{get_dir, Dirs},
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

pub struct IpcSocket<T> {
    socket_path: PathBuf,
    _phantom: PhantomData<T>,
}

pub struct Server;
pub struct Client;

struct SocketGuard(PathBuf);

impl<T> IpcSocket<T> {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            _phantom: PhantomData,
        }
    }

    pub fn bind_default() -> io::Result<Self> {
        let socket_path = Self::default_path()?;

        Ok(Self::new(socket_path))
    }

    fn default_path() -> io::Result<PathBuf> {
        let path = match get_dir(Dirs::Runtime) {
            Ok(path) => path,
            Err(e) => {
                error!("{}", e);
                warn!("Socket falling back to /tmp/walrus");
                PathBuf::from("/tmp")
            }
        };

        // NOTE:
        // I am assuming that /tmp exists.
        // Probably a bad idea to try to create it anyway, wouldn't even have permission to.
        // Not to mention if neither path exists, it's more likely than not a problem with the
        // user's system.

        debug!("Socket at: {:?}", path.join("walrus"));
        Ok(path.join("walrus"))
    }
}

impl IpcSocket<Server> {
    pub fn start(&self, tx: Sender<Commands>) -> io::Result<()> {
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        let guard = SocketGuard(self.socket_path.clone());

        thread::spawn(move || {
            let _guard = guard;

            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(e) => {
                        error!("Error accepting connection: {e:#?}");
                        continue;
                    }
                };

                let mut buffer = [0; 1];
                if stream.read_exact(&mut buffer).is_err() {
                    continue;
                }

                let Some(command) = Commands::from_byte(buffer[0]) else {
                    continue;
                };

                if let Commands::Config = command {
                    continue;
                }

                debug!("IPC received {:?} command", command);
                let _ = tx.send(command);
            }
            // NOTE: Socket file should be deleted since guard is dropped here
        });

        Ok(())
    }
}

impl IpcSocket<Client> {
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

impl Drop for SocketGuard {
    fn drop(&mut self) {
        if self.0.exists() {
            if let Err(e) = fs::remove_file(&self.0) {
                error!("Failed to clean up socket {:?}: {e}", self.0);
            }
        }
    }
}

pub fn setup_ipc(tx: Sender<Commands>) -> io::Result<()> {
    debug!("Starting IPC server");
    let server = IpcSocket::<Server>::bind_default()?;
    server.start(tx)?;
    Ok(())
}

pub fn send_ipc_command(command: Commands) -> io::Result<()> {
    debug!("IPC sending {:?} command", command);
    let client = IpcSocket::<Client>::bind_default()?;
    client.send_command(command)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn test_ipc_cmd() {
        let (tx, rx) = mpsc::channel();

        setup_ipc(tx.clone()).expect("Failed to setup IPC");

        let cmd = Commands::Next;
        let _ = tx.send(cmd.clone());
        let rx_cmd = rx.recv().unwrap();

        assert_eq!(cmd.to_bytes().unwrap(), rx_cmd.to_bytes().unwrap());
    }
}

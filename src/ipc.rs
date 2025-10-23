use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    marker::PhantomData,
    os::unix::{
        fs::OpenOptionsExt,
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    process,
    sync::mpsc::Sender,
    thread,
};

use log::{debug, error, warn};
use nix::fcntl::{Flock, FlockArg};

use crate::{
    commands::Commands,
    utils::{self, Dirs},
};

impl Commands {
    fn as_byte(&self) -> Option<u8> {
        match self {
            Commands::Config => None,
            cmd => Some((*cmd).into()),
        }
    }

    fn from_byte(byte: u8) -> Option<Commands> {
        match Self::try_from(byte) {
            Ok(Commands::Config) => None,
            Ok(cmd) => Some(cmd),
            Err(_) => unreachable!("Can't be sent a non-existant byte value"),
        }
    }
}

pub struct IpcSocket<T> {
    socket_path: PathBuf,
    guard: IpcGuard,
    _lock: Option<Flock<File>>,
    _phantom: PhantomData<T>,
}

pub struct Server;
struct Client;

impl<T> IpcSocket<T> {
    fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            guard: IpcGuard::new(),
            _lock: None,
            _phantom: PhantomData,
        }
    }

    fn bind_client() -> Self {
        let runtime_dir = utils::get_dir(Dirs::Runtime).expect("Error getting runtime dir");
        let default_path = runtime_dir.join("walrus");

        let bound_path = if default_path.exists() {
            default_path
        } else {
            PathBuf::from("/tmp/walrus")
        };

        Self::new(bound_path)
    }

    fn bind_server() -> Self {
        fn default_path() -> PathBuf {
            let path = match utils::get_dir(Dirs::Runtime) {
                Ok(path) => path,
                Err(e) => {
                    error!("{}", e);
                    warn!("Socket falling back to /tmp/walrus");
                    PathBuf::from("/tmp")
                }
            };

            path.join("walrus")
        }

        let bound_path = default_path();
        let mut server = Self::new(&bound_path);

        let runtime_dir = utils::get_dir(Dirs::Runtime).expect("Error getting runtime dir");

        let lock_path = runtime_dir.join("walrus.lock");
        let lock_file = OpenOptions::new()
            .mode(0o640)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_path)
            .expect("Error creating lock file");
        match Flock::lock(lock_file, FlockArg::LockExclusiveNonblock) {
            Ok(flock) => {
                debug!("Successfully acquired lock file");
                server.guard.add_path(&bound_path);
                server._lock = Some(flock);
            }
            Err((file, nix::Error::EWOULDBLOCK)) => {
                drop(file);
                error!("An instance is already running (lock file is locked)");
                process::exit(1)
            }
            Err((file, e)) => {
                drop(file);
                error!("Error locking lock file: {e}");
                process::exit(1)
            }
        };

        server
    }
}

impl IpcSocket<Server> {
    fn start(&self, tx: Sender<Commands>) {
        if self.socket_path.exists() {
            debug!("Socket file already exists (cleanup may have failed)");

            if let Err(e) = fs::remove_file(&self.socket_path) {
                error!("Failed to remove socket file: {e}");
                process::exit(1);
            }
        }

        let listener = UnixListener::bind(&self.socket_path).expect("Failed to bind socket");

        thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(e) => {
                        error!("Error accepting connection: {e}");
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
        });
    }
}

impl IpcSocket<Client> {
    fn send_command(&self, command: Commands) -> io::Result<()> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        let Some(cmd) = command.as_byte() else {
            return Err(io::Error::other("Failed to convert command to bytes"));
        };
        stream.write_all(&[cmd])?;
        Ok(())
    }
}

struct IpcGuard {
    paths: Vec<PathBuf>,
}

impl IpcGuard {
    fn new() -> Self {
        Self { paths: Vec::new() }
    }

    fn add_path(&mut self, path: impl Into<PathBuf>) {
        self.paths.push(path.into());
    }
}

impl Drop for IpcGuard {
    fn drop(&mut self) {
        for path in &self.paths {
            if !path.exists() {
                continue;
            }
            if let Err(e) = fs::remove_file(path) {
                error!("Failed to clean up file {:?}: {e}", path);
            }
        }
    }
}

pub fn setup_ipc(tx: Sender<Commands>) -> IpcSocket<Server> {
    debug!("Starting IPC server");
    let server = IpcSocket::<Server>::bind_server();
    server.start(tx);

    server
}

pub fn send_ipc_command(command: Commands) -> io::Result<()> {
    debug!("IPC sending {:?} command", command);

    let client = IpcSocket::<Client>::bind_client();
    client.send_command(command)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn test_ipc_cmd() {
        let (tx, rx) = mpsc::channel();

        setup_ipc(tx.clone());

        let cmd = Commands::Next;
        let _ = tx.send(cmd);
        let rx_cmd = rx.recv().unwrap();

        assert_eq!(cmd.as_byte().unwrap(), rx_cmd.as_byte().unwrap());
    }
}

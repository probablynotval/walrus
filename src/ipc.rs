use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::ops::ControlFlow;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

use log::debug;
use log::error;
use nix::fcntl::Flock;
use nix::fcntl::FlockArg;

use crate::commands::Commands;
use crate::utils;
use crate::utils::Dirs;

pub struct IpcServer {
    socket_path: PathBuf,
    // Guard ensures we always cleanup the socket file: $XDG_RUNTIME_DIR/walrus.
    _guard: IpcGuard,
    // To hold the lock for the entire lifetime of the struct.
    _lock: Option<Flock<File>>,
}

impl IpcServer {
    fn new(socket_path: PathBuf, lock_path: PathBuf) -> Self {
        let flock = acquire_lock(&lock_path);
        let mut guard = IpcGuard::new();
        guard.add_path(&socket_path);

        Self {
            socket_path,
            _guard: guard,
            _lock: Some(flock),
        }
    }

    fn start(&self, tx: Sender<Commands>) -> JoinHandle<()> {
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
                let Ok(mut stream) = stream else {
                    error!("Error accepting connection: {}", stream.unwrap_err());
                    continue;
                };

                if parse_stream(&mut stream, &tx).is_break() {
                    break;
                }
            }
        })
    }
}

// This abstraction mostly exists for organisation purposes. I don't want to inline it because it
// makes the protocol logic untestable.
struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    fn send(&self, command: Commands) -> io::Result<()> {
        let mut stream = UnixStream::connect(&self.socket_path)?;
        let cmd = command
            .to_bytes()
            .ok_or(io::Error::other("Failed to convert command to bytes"))?;

        // Bincode uses variable length messages in little-endian order with standard config.
        // For my case, most of the enum variants should be u8. I'm using u16 for the big ones.
        let len = (cmd.len() as u16).to_le_bytes();
        stream.write_all(&len)?;
        stream.write_all(&cmd)?;
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

fn acquire_lock(lock_path: &Path) -> Flock<File> {
    let lock_file = OpenOptions::new()
        .mode(0o640)
        .write(true)
        .create(true)
        .truncate(true)
        .open(lock_path)
        .expect("Error creating lock file");
    match Flock::lock(lock_file, FlockArg::LockExclusiveNonblock) {
        Ok(flock) => {
            debug!("Successfully acquired lock file");
            flock
        }
        Err((_, nix::Error::EWOULDBLOCK)) => {
            error!("An instance is already running (lock file is locked)");
            process::exit(1)
        }
        Err((_, e)) => {
            error!("Error locking lock file: {e}");
            process::exit(1)
        }
    }
}

fn parse_stream<R: Read>(stream: &mut R, tx: &Sender<Commands>) -> ControlFlow<()> {
    let mut len_buffer = [0u8; 2];
    if let Err(e) = stream.read_exact(&mut len_buffer) {
        error!("Error reading length prefix: {e}");
        return ControlFlow::Continue(());
    }
    let len = u16::from_le_bytes(len_buffer);

    let mut cmd_buffer = vec![0u8; len.into()];
    if let Err(e) = stream.read_exact(&mut cmd_buffer) {
        error!("Error reading command bytes: {e}");
        return ControlFlow::Continue(());
    }

    if let Some(command) = Commands::from_bytes(&cmd_buffer) {
        debug!("IPC received {:?} command", command);
        let _ = tx.send(command.clone());

        return match command {
            Commands::Shutdown => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        };
    }

    ControlFlow::Continue(())
}

fn get_paths() -> (PathBuf, PathBuf) {
    let runtime_dir = utils::get_dir(Dirs::Runtime).unwrap_or_else(|e| {
        error!("Error getting runtime directory: {}", e);
        process::exit(1)
    });
    let socket_path = runtime_dir.join("walrus");
    let lock_path = runtime_dir.join("walrus.lock");
    (socket_path, lock_path)
}

pub fn setup_ipc(tx: Sender<Commands>) -> IpcServer {
    debug!("Starting IPC server");
    let (socket_path, lock_path) = get_paths();
    let server = IpcServer::new(socket_path, lock_path);
    server.start(tx);

    server
}

pub fn send_ipc_command(command: Commands) -> io::Result<()> {
    debug!("IPC sending {:?} command", command);

    let (socket_path, _) = get_paths();

    let client = IpcClient::new(socket_path);
    client.send(command)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::mpsc;

    use super::*;

    // These tests will panic if run in parallel, so I'm using a Mutex to ensure they don't.
    // The reason this works is because the call to .lock() is blocking.
    static LOCK: Mutex<()> = Mutex::new(());

    // I'm also using the JoinHandle returned in these tests to ensure that the thread spawned by
    // the IPC server doesn't become detached and keep running. This doesn't matter in the main
    // application because it will be cleaned up by the OS.

    // NOTE: These tests are kinda outside the scope of what unit tests should be. I let it stay
    // for now but I might want to make these unit tests in the future.

    #[test]
    fn test_ipc_cmd() {
        let _lock = LOCK.lock().unwrap();

        let (tx, rx) = mpsc::channel();

        let (socket_path, lock_path) = get_paths();
        let server = IpcServer::new(socket_path.clone(), lock_path);
        let handle = server.start(tx.clone());

        let cmd = Commands::Next;
        let client = IpcClient::new(socket_path);
        client.send(cmd.clone()).unwrap();

        let rx_cmd = rx.recv().unwrap();

        assert_eq!(cmd.to_bytes(), rx_cmd.to_bytes());

        client.send(Commands::Shutdown).unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn test_ipc_cmd_data() {
        let _lock = LOCK.lock().unwrap();

        let (tx, rx) = mpsc::channel();

        let (socket_path, lock_path) = get_paths();
        let server = IpcServer::new(socket_path.clone(), lock_path);
        let handle = server.start(tx.clone());

        let cmd = Commands::Categorise {
            category: "Favourites".into(),
        };
        let client = IpcClient::new(socket_path);
        client.send(cmd.clone()).unwrap();

        let rx_cmd = rx.recv().unwrap();

        assert_eq!(cmd.to_bytes(), rx_cmd.to_bytes());

        client.send(Commands::Shutdown).unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn test_stream_parsing() {
        let (mut client, mut server) = UnixStream::pair().unwrap();
        let (tx, rx) = mpsc::channel();

        let cmds = [
            Commands::Next,
            Commands::Previous,
            Commands::Pause,
            Commands::Resume,
            Commands::Reload,
        ];

        for cmd in cmds {
            let bytes = cmd.to_bytes().unwrap();
            let len = (bytes.len() as u16).to_le_bytes();
            client.write_all(&len).unwrap();
            client.write_all(&bytes).unwrap();

            let control_flow = parse_stream(&mut server, &tx);
            assert!(!control_flow.is_break());

            let received = rx.recv().unwrap();
            assert_eq!(cmd.to_bytes(), received.to_bytes());
        }
    }
}

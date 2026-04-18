use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver};

/// PTY handle for the embedded terminal.
pub struct PtyHandle {
    pair: portable_pty::PtyPair,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
}

impl PtyHandle {
    /// Spawn a new shell in a PTY and stream output through the returned receiver.
    pub fn spawn(
        shell: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<(Self, Receiver<Vec<u8>>)> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(shell.unwrap_or(&default_shell()));
        cmd.cwd(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

        let child = pair.slave.spawn_command(cmd)?;
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(count) => {
                        if tx.send(buffer[..count].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok((
            Self {
                pair,
                child,
                writer,
            },
            rx,
        ))
    }

    /// Write raw bytes to the PTY (user input).
    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Submit a shell command followed by a newline.
    pub fn submit(&mut self, command: &str) -> anyhow::Result<()> {
        self.write(command.as_bytes())?;
        self.write(if cfg!(windows) { b"\r\n" } else { b"\n" })?;
        Ok(())
    }

    /// Send Ctrl+C to the active process.
    pub fn interrupt(&mut self) -> anyhow::Result<()> {
        self.write(&[3])
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.pair.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    /// Terminate the child process.
    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.child.kill()?;
        Ok(())
    }

    /// Check if the child process is still alive.
    pub fn is_alive(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }
}

fn default_shell() -> String {
    if cfg!(windows) {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())
    }
}

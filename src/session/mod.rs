//! This module contains a system independent [Session] representation.
//!
//! But it does set a default [Session<P, S>] processes and stream in order to be able to use Session without generics.
//! It also sets a list of other methods which are available for a platform processes.
//!
//! # Example
//!
//! ```no_run,ignore
//! use std::{process::Command, io::prelude::*};
//! use expectrl::Session;
//!
//! let mut p = Session::spawn(Command::new("cat")).unwrap();
//! writeln!(p, "Hello World").unwrap();
//! let mut line = String::new();
//! p.read_line(&mut line).unwrap();
//! ```

#[cfg(feature = "async")]
pub mod async_session;
#[cfg(not(feature = "async"))]
pub mod sync_session;

use std::io::{Read, Write};

use crate::{
    process::{NonBlocking, Process},
    stream::log::LoggedStream,
    Error,
};

#[cfg(feature = "async")]
use crate::process::IntoAsyncStream;

#[cfg(unix)]
pub(crate) type Proc = crate::process::unix::UnixProcess;
#[cfg(all(unix, not(feature = "async")))]
pub(crate) type Stream = crate::process::unix::PtyStream;
#[cfg(all(unix, feature = "async"))]
pub(crate) type Stream = crate::process::unix::AsyncPtyStream;

#[cfg(windows)]
pub(crate) type Proc = crate::process::windows::WinProcess;
#[cfg(all(windows, not(feature = "async")))]
pub(crate) type Stream = crate::process::windows::ProcessStream;
#[cfg(all(windows, feature = "async"))]
pub(crate) type Stream = crate::process::windows::AsyncProcessStream;

/// Session represents a spawned process and its IO stream.
/// It controlls process and communication with it.
///
/// It represents a expect session.
#[cfg(not(feature = "async"))]
pub type Session<P = Proc, S = Stream> = sync_session::Session<P, S>;

/// Session represents a spawned process and its IO stream.
/// It controlls process and communication with it.
///
/// It represents a expect session.
#[cfg(feature = "async")]
pub type Session<P = Proc, S = Stream> = async_session::Session<P, S>;

impl Session {
    /// Spawns a session on a platform process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::process::Command;
    /// use expectrl::Session;
    ///
    /// let p = Session::spawn(Command::new("cat"));
    /// ```
    pub fn spawn(command: <Proc as Process>::Command) -> Result<Self, Error> {
        let mut process = Proc::spawn_command(command)?;
        let stream = process.open_stream()?;

        #[cfg(feature = "async")]
        let stream = stream.into_async_stream()?;

        let session = Self::new(process, stream)?;

        Ok(session)
    }

    /// Spawns a session on a platform process.
    /// Using a string commandline.
    pub(crate) fn spawn_cmd(cmd: &str) -> Result<Self, Error> {
        let mut process = Proc::spawn(cmd)?;
        let stream = process.open_stream()?;

        #[cfg(feature = "async")]
        let stream = stream.into_async_stream()?;

        let session = Self::new(process, stream)?;

        Ok(session)
    }
}

#[cfg(not(feature = "async"))]
impl<P, S: Read> Session<P, S> {
    /// Set a logger which will write each Read/Write operation into the writter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let p = expectrl::spawn("cat")
    ///     .unwrap()
    ///     .with_log(std::io::stdout())
    ///     .unwrap();
    /// ```
    pub fn with_log<W: Write>(self, logger: W) -> Result<Session<P, LoggedStream<S, W>>, Error> {
        self.swap_stream(|stream| LoggedStream::new(stream, logger))
    }
}

#[cfg(feature = "async")]
impl<P, S> Session<P, S> {
    /// Set a logger which will write each Read/Write operation into the writter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let p = expectrl::spawn("cat")
    ///     .unwrap()
    ///     .with_log(std::io::stdout())
    ///     .unwrap();
    /// ```
    pub fn with_log<W: Write>(self, logger: W) -> Result<Session<P, LoggedStream<S, W>>, Error> {
        self.swap_stream(|stream| LoggedStream::new(stream, logger))
    }
}

#[cfg(all(not(feature = "async"), not(feature = "polling")))]
impl<S> Session<Proc, S>
where
    S: NonBlocking + Write + Read,
{
    /// Interact gives control of the child process to the interactive user (the
    /// human at the keyboard).
    ///
    /// Keystrokes are sent to the child process, and
    /// the `stdout` and `stderr` output of the child process is printed.
    ///
    /// When the user types the `escape_character` this method will return control to a running process.
    /// The escape_character will not be transmitted.
    /// The default for escape_character is entered as `Ctrl-]`, the very same as BSD telnet.
    ///
    /// This simply echos the child `stdout` and `stderr` to the real `stdout` and
    /// it echos the real `stdin` to the child `stdin`.
    ///
    /// BEWARE that interact finishes after a process stops.
    /// So after the return you may not obtain a correct status of a process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut p = expectrl::spawn("cat").unwrap();
    /// p.interact().unwrap();
    /// ```
    ///
    /// You can get a rich set of options for interact session using [crate::interact::InteractOptions].
    pub fn interact(&mut self) -> Result<(), Error> {
        #[cfg(unix)]
        {
            let is_echo = self
                .get_echo()
                .map_err(|e| Error::unknown("failed to get echo", e))?;
            if !is_echo {
                let _ = self.set_echo(true, None);
            }

            crate::interact::InteractOptions::default().interact_in_terminal(self)?;

            if !is_echo {
                let _ = self.set_echo(false, None);
            }
        }

        #[cfg(windows)]
        {
            crate::interact::InteractOptions::default().interact_in_terminal(self)?;
        }

        Ok(())
    }
}

#[cfg(all(unix, feature = "polling", not(feature = "async")))]
impl<S> Session<Proc, S>
where
    S: NonBlocking + Write + Read + polling::Source,
{
    /// Interact gives control of the child process to the interactive user (the
    /// human at the keyboard).
    ///
    /// Keystrokes are sent to the child process, and
    /// the `stdout` and `stderr` output of the child process is printed.
    ///
    /// When the user types the `escape_character` this method will return control to a running process.
    /// The escape_character will not be transmitted.
    /// The default for escape_character is entered as `Ctrl-]`, the very same as BSD telnet.
    ///
    /// This simply echos the child `stdout` and `stderr` to the real `stdout` and
    /// it echos the real `stdin` to the child `stdin`.
    ///
    /// BEWARE that interact finishes after a process stops.
    /// So after the return you may not obtain a correct status of a process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut p = expectrl::spawn("cat").unwrap();
    /// p.interact().unwrap();
    /// ```
    ///
    /// You can get a rich set of options for interact session using [crate::interact::InteractOptions].
    pub fn interact(&mut self) -> Result<(), Error> {
        let is_echo = self
            .get_echo()
            .map_err(|e| Error::unknown("failed to get echo", e))?;
        if !is_echo {
            let _ = self.set_echo(true, None);
        }

        crate::interact::InteractOptions::default().interact_in_terminal(self)?;

        if !is_echo {
            let _ = self.set_echo(false, None);
        }

        Ok(())
    }
}

#[cfg(all(windows, feature = "polling", not(feature = "async")))]
impl Session
{
    /// Interact gives control of the child process to the interactive user (the
    /// human at the keyboard).
    ///
    /// Keystrokes are sent to the child process, and
    /// the `stdout` and `stderr` output of the child process is printed.
    ///
    /// When the user types the `escape_character` this method will return control to a running process.
    /// The escape_character will not be transmitted.
    /// The default for escape_character is entered as `Ctrl-]`, the very same as BSD telnet.
    ///
    /// This simply echos the child `stdout` and `stderr` to the real `stdout` and
    /// it echos the real `stdin` to the child `stdin`.
    ///
    /// BEWARE that interact finishes after a process stops.
    /// So after the return you may not obtain a correct status of a process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut p = expectrl::spawn("cat").unwrap();
    /// p.interact().unwrap();
    /// ```
    ///
    /// You can get a rich set of options for interact session using [crate::interact::InteractOptions].
    pub fn interact(&mut self) -> Result<(), Error> {
        crate::interact::InteractOptions::default().interact_in_terminal(self)
    }
}

#[cfg(feature = "async")]
impl<S> Session<Proc, S>
where
    S: futures_lite::AsyncRead + futures_lite::AsyncWrite + Unpin,
{
    /// Interact gives control of the child process to the interactive user (the
    /// human at the keyboard).
    ///
    /// Keystrokes are sent to the child process, and
    /// the `stdout` and `stderr` output of the child process is printed.
    ///
    /// When the user types the `escape_character` this method will return control to a running process.
    /// The escape_character will not be transmitted.
    /// The default for escape_character is entered as `Ctrl-]`, the very same as BSD telnet.
    ///
    /// This simply echos the child `stdout` and `stderr` to the real `stdout` and
    /// it echos the real `stdin` to the child `stdin`.
    ///
    /// BEWARE that interact finishes after a process stops.
    /// So after the return you may not obtain a correct status of a process.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # futures_lite::future::block_on(async {
    /// let mut p = expectrl::spawn("cat").unwrap();
    /// p.interact().await.unwrap();
    /// # });
    /// ```
    ///
    /// You can get a rich set of options for interact session using [crate::interact::InteractOptions].
    pub async fn interact(&mut self) -> Result<(), Error> {
        #[cfg(unix)]
        {
            let is_echo = self
                .get_echo()
                .map_err(|e| Error::unknown("failed to get echo", e))?;
            if !is_echo {
                let _ = self.set_echo(true, None);
            }
        }

        crate::interact::InteractOptions::default()
            .interact_in_terminal(self)
            .await?;

        #[cfg(unix)]
        {
            if !is_echo {
                let _ = self.set_echo(false, None);
            }
        }

        Ok(())
    }
}

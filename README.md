# expectrl

Spawn, control, and respond to expected patterns of child applications and processes, enabling the automation of interactions and testing.

It also has an async support.

It was heavily inspired by [philippkeller/rexpect](https://github.com/philippkeller/rexpect) and [pexpect](https://pexpect.readthedocs.io/en/stable/overview.html).

## Basic usage

Add the following line to your `Cargo.toml` file:

```toml
[dependencies]
expectrl = "0.1"
```

### An example for interacting via ftp:

```rust
use expectrl::{spawn, Regex, Eof, WaitStatus};

fn main() {
    let mut p = spawn("ftp speedtest.tele2.net").unwrap();
    p.expect(Regex("Name \\(.*\\):")).unwrap();
    p.send_line("anonymous").unwrap();
    p.expect("Password").unwrap();
    p.send_line("test").unwrap();
    p.expect("ftp>").unwrap();
    p.send_line("cd upload").unwrap();
    p.expect("successfully changed.\r\nftp>").unwrap();
    p.send_line("pwd").unwrap();
    p.expect(Regex("[0-9]+ \"/upload\"")).unwrap();
    p.send_line("exit").unwrap();
    p.expect(Eof).unwrap();
    assert_eq!(p.wait().unwrap(), WaitStatus::Exited(p.pid(), 0));
}
```

### Example bash with `async` feature

```rust
use expectrl::{repl::spawn_bash, Regex, Error};
use ptyprocess::ControlCode;
use futures_lite::io::AsyncBufReadExt;

#[tokio::main]
fn main() -> Result<(), Error> {
    let mut p = spawn_bash().await?;

    p.send_line("hostname").await?;
    let mut hostname = String::new();
    p.read_line(&mut hostname).await?;
    p.expect_prompt().await?; // go sure `hostname` is really done
    println!("Current hostname: {:?}", hostname);

    Ok(())
}
```

### Example with bash and job control

One frequent bitfall with sending signals is that you need
to somehow ensure that the program has fully loaded, otherwise they
goes into nowhere. There are 2 handy function `execute` for this purpouse:

- `execute` - does a command and ensures that the prompt is shown again.
- `expect_prompt` - ensures that the prompt is shown.

```rust
use expectrl::{repl::spawn_bash, Error};
use ptyprocess::ControlCode;

fn main() -> Result<(), Error> {
    let mut p = spawn_bash()?;
    p.send_line("ping 8.8.8.8")?;
    p.expect("bytes of data")?;
    p.send_control(ControlCode::Substitute)?; // CTRL_Z
    p.expect_prompt()?;
    // bash writes 'ping 8.8.8.8' to stdout again to state which job was put into background
    p.send_line("bg")?;
    p.expect("ping 8.8.8.8")?;
    p.expect_prompt()?;
    p.send_line("sleep 0.5")?;
    p.expect_prompt()?;
    // bash writes 'ping 8.8.8.8' to stdout again to state which job was put into foreground
    p.send_line("fg")?;
    p.expect("ping 8.8.8.8")?;
    p.send_control(ControlCode::EndOfText)?;
    p.expect("packet loss")?;

    Ok(())
}
```

## Examples

[For more examples, check the examples directory.](https://github.com/zhiburt/expectrl/tree/master/examples)

Licensed under [MIT License](LICENSE)

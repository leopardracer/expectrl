#![cfg(unix)]

use expectrl::{
    repl::{spawn_bash, spawn_python},
    ControlCode, WaitStatus,
};
#[cfg(feature = "async")]
use futures_lite::io::AsyncBufReadExt;
#[cfg(not(feature = "async"))]
use std::io::BufRead;
use std::{thread, time::Duration};

#[cfg(not(feature = "async"))]
#[cfg(target_os = "linux")]
#[test]
fn bash() {
    let mut p = spawn_bash().unwrap();

    p.send_line("echo Hello World").unwrap();
    let mut msg = String::new();
    p.read_line(&mut msg).unwrap();
    assert!(msg.ends_with("Hello World\r\n"));

    p.send_control(ControlCode::EOT).unwrap();

    assert_eq!(p.wait().unwrap(), WaitStatus::Exited(p.pid(), 0));
}

#[cfg(not(feature = "async"))]
#[cfg(target_os = "linux")]
#[test]
fn bash_with_log() {
    let mut p = spawn_bash()
        .unwrap()
        .upgrade_session(|s| s.with_log(std::io::stderr()))
        .unwrap();

    p.send_line("echo Hello World").unwrap();
    let mut msg = String::new();
    p.read_line(&mut msg).unwrap();
    assert!(msg.ends_with("Hello World\r\n"));

    thread::sleep(Duration::from_millis(300));
    p.send_control(ControlCode::EOT).unwrap();

    assert_eq!(p.wait().unwrap(), WaitStatus::Exited(p.pid(), 0));
}

#[cfg(feature = "async")]
#[test]
fn bash() {
    futures_lite::future::block_on(async {
        let mut p = spawn_bash().await.unwrap();

        p.send_line("echo Hello World").await.unwrap();
        let mut msg = String::new();
        p.read_line(&mut msg).await.unwrap();
        assert!(msg.ends_with("Hello World\r\n"));

        thread::sleep(Duration::from_millis(300));
        p.send_control(ControlCode::EOT).await.unwrap();

        assert_eq!(p.wait().unwrap(), WaitStatus::Exited(p.pid(), 0));
    })
}

#[cfg(feature = "async")]
#[test]
fn bash_with_log() {
    futures_lite::future::block_on(async {
        let mut p = spawn_bash()
            .await
            .unwrap()
            .upgrade_session(|s| s.with_log(std::io::stderr()))
            .unwrap();

        p.send_line("echo Hello World").await.unwrap();
        let mut msg = String::new();
        p.read_line(&mut msg).await.unwrap();
        assert!(msg.ends_with("Hello World\r\n"));

        thread::sleep(Duration::from_millis(300));
        p.send_control(ControlCode::EOT).await.unwrap();

        assert_eq!(p.wait().unwrap(), WaitStatus::Exited(p.pid(), 0));
    })
}

#[cfg(feature = "async")]
#[test]
fn python() {
    futures_lite::future::block_on(async {
        let mut p = spawn_python().await.unwrap();

        let prompt = p.execute("print('Hello World')").await.unwrap();
        assert_eq!(prompt, b"Hello World\r\n");

        thread::sleep(Duration::from_millis(300));
        p.send_control(ControlCode::EndOfText).await.unwrap();
        thread::sleep(Duration::from_millis(300));

        let mut msg = String::new();
        p.read_line(&mut msg).await.unwrap();
        assert_eq!(msg, "\r\n");

        let mut msg = String::new();
        p.read_line(&mut msg).await.unwrap();
        assert_eq!(msg, "KeyboardInterrupt\r\n");

        p.expect_prompt().await.unwrap();

        p.send_control(ControlCode::EndOfTransmission)
            .await
            .unwrap();

        assert_eq!(p.wait().unwrap(), WaitStatus::Exited(p.pid(), 0));
    })
}

#[cfg(not(feature = "async"))]
#[test]
fn python() {
    let mut p = spawn_python().unwrap();

    let prompt = p.execute("print('Hello World')").unwrap();
    assert_eq!(prompt, b"Hello World\r\n");

    thread::sleep(Duration::from_millis(300));
    p.send_control(ControlCode::EndOfText).unwrap();
    thread::sleep(Duration::from_millis(300));

    let mut msg = String::new();
    p.read_line(&mut msg).unwrap();
    assert_eq!(msg, "\r\n");

    let mut msg = String::new();
    p.read_line(&mut msg).unwrap();
    assert_eq!(msg, "KeyboardInterrupt\r\n");

    p.expect_prompt().unwrap();

    p.send_control(ControlCode::EndOfTransmission).unwrap();

    assert_eq!(p.wait().unwrap(), WaitStatus::Exited(p.pid(), 0));
}

#[cfg(feature = "async")]
#[test]
fn bash_pwd() {
    futures_lite::future::block_on(async {
        let mut p = spawn_bash().await.unwrap();
        p.execute("cd /tmp/").await.unwrap();
        p.send_line("pwd").await.unwrap();
        let mut pwd = String::new();
        p.read_line(&mut pwd).await.unwrap();
        assert!(pwd.contains("/tmp\r\n"));
    });
}

#[cfg(feature = "async")]
#[test]
fn bash_control_chars() {
    futures_lite::future::block_on(async {
        let mut p = spawn_bash().await.unwrap();
        p.send_line("cat <(echo ready) -").await.unwrap();
        thread::sleep(Duration::from_millis(100));
        p.send_control(ControlCode::EndOfText).await.unwrap(); // abort: SIGINT
        p.expect_prompt().await.unwrap();
        p.send_line("cat <(echo ready) -").await.unwrap();
        thread::sleep(Duration::from_millis(100));
        p.send_control(ControlCode::Substitute).await.unwrap(); // suspend:SIGTSTPcon
        p.expect_prompt().await.unwrap();
    });
}

#[cfg(not(feature = "async"))]
#[test]
fn bash_pwd() {
    let mut p = spawn_bash().unwrap();
    p.execute("cd /tmp/").unwrap();
    p.send_line("pwd").unwrap();
    let mut pwd = String::new();
    p.read_line(&mut pwd).unwrap();
    assert!(pwd.contains("/tmp\r\n"));
}

#[cfg(not(feature = "async"))]
#[test]
fn bash_control_chars() {
    let mut p = spawn_bash().unwrap();
    p.send_line("cat <(echo ready) -").unwrap();
    thread::sleep(Duration::from_millis(300));
    p.send_control(ControlCode::EndOfText).unwrap(); // abort: SIGINT
    p.expect_prompt().unwrap();
    p.send_line("cat <(echo ready) -").unwrap();
    thread::sleep(Duration::from_millis(100));
    p.send_control(ControlCode::Substitute).unwrap(); // suspend:SIGTSTPcon
    p.expect_prompt().unwrap();
}

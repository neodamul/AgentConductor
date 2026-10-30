use std::{net::UdpSocket, process::Command, time::Duration};

#[test]
fn supervised_cli_child_notifies_its_parent_when_the_command_exits() {
    let listener = UdpSocket::bind("127.0.0.1:0").unwrap();
    listener
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let address = listener.local_addr().unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("--invalid-child-event-probe")
        .env("AC_UI_EVENT_ADDRESS", address.to_string())
        .env("AC_UI_EVENT_TOKEN", "test-token")
        .env("AC_UI_EVENT_SURFACE", "workspace")
        .status()
        .unwrap();
    assert!(!status.success());

    let mut packet = [0; 128];
    let (length, _) = listener.recv_from(&mut packet).unwrap();
    assert_eq!(&packet[..length], b"test-token:workspace");
}

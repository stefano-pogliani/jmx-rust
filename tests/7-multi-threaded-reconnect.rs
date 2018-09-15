//!
//! This test is also an example of reconnection of the client in a multi-threaded process.
//!
//! This test:
//!
//!   1. Connects to a JMX server.
//!   2. Fetch two specific JMX attributes.
//!   3. Kill the server.
//!   4. Attempts to fetch an attribute from the server (expect to fail).
//!   5. Re-starts the server.
//!   6. Reconnect to the server.
//!   7. Attempts to fetch an attribute again (expect to fail).
//!
extern crate jmx;

use std::process::Command;
use std::process::Child;
use std::thread;
use std::time::Duration;

use jmx::MBeanAddress;
use jmx::MBeanClientTrait;
use jmx::MBeanThreadedClient;
use jmx::Result;


static JMX_PORT: u16 = 1623;


#[test]
fn multi_threaded_reconnect() {
    // Phase one: start a server, get an attribute, stop server.
    let mut server = start_server();
    let client = run_phase_one();
    let _ = server.kill();

    // Phase two: attempt to get an attribute.
    run_phase_two(&client);

    // Phase three: re-start the server, get an attribute.
    let mut server = start_server();
    run_phase_three(&client);

    // Stop the server once we are done with the test.
    let _ = server.kill();
}

fn run_phase_one() -> MBeanThreadedClient {
    // Create a connection to the remote JMX server.
    let address = MBeanAddress::address(format!("127.0.0.1:{}", JMX_PORT));
    let server = MBeanThreadedClient::connect(address)
        .expect("Failed to connect to the JMX test server");

    // Fetch some attribute from the server.
    let threads: i32 = server.get_attribute("FOO:name=ServerBean", "ThreadCount").unwrap();
    assert_eq!(threads, 16);

    // Return the server to be used in other phases.
    server
}

fn run_phase_two(client: &MBeanThreadedClient) {
    // Attempt to fetch the attribute again.
    let result: Result<i32> = client.get_attribute("FOO:name=ServerBean", "ThreadCount");
    assert!(result.is_err());
}

fn run_phase_three(client: &MBeanThreadedClient) {
    // Re-connect to the remote JMX server.
    let address = MBeanAddress::address(format!("127.0.0.1:{}", JMX_PORT));
    client.reconnect(address)
        .expect("Failed to connect to the JMX test server");

    // Fetch some attribute from the server.
    let threads: i32 = client.get_attribute("FOO:name=ServerBean", "ThreadCount").unwrap();
    assert_eq!(threads, 16);
}

fn start_server() -> Child {
    // Start the server and wait for it to be up.
    let server = Command::new("java")
        .arg("-Dcom.sun.management.jmxremote")
        .arg(format!("-Dcom.sun.management.jmxremote.port={}", JMX_PORT))
        .arg("-Dcom.sun.management.jmxremote.authenticate=false")
        .arg("-Dcom.sun.management.jmxremote.ssl=false")
        .arg("TestServer")
        .current_dir("tests/jmxserver")
        .spawn()
        .expect("Could not start JMX server");
    // Because the test does not create the JVM instance after the server is started
    // we need to wait longer to make sure the server is listening.
    thread::sleep(Duration::from_secs(5));
    server
}

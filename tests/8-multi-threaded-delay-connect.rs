//!
//! This test is also an example of reconnection of the client in a multi-threaded process.
//!
//! This test:
//!
//!   1. Create threaded client without connection.
//!   2. Fetch specific JMX attribute (expect to fail).
//!   3. Connect to server.
//!   4. Fetch an attribute from the server.
//!   5. Disconnect from the server.
//!   6. Fetch specific JMX attribute (expect to fail).
//!
extern crate jmx;

use std::process::Command;
use std::thread;
use std::time::Duration;

use jmx::MBeanAddress;
use jmx::MBeanClientTrait;
use jmx::MBeanThreadedClient;
use jmx::MBeanThreadedClientOptions;
use jmx::Result;


static JMX_PORT: u16 = 1624;


#[test]
fn multi_threaded_delay_connect() {
    // Start the server and wait for it to be up.
    let mut server = Command::new("java")
        .arg("-Dcom.sun.management.jmxremote")
        .arg(format!("-Dcom.sun.management.jmxremote.port={}", JMX_PORT))
        .arg("-Dcom.sun.management.jmxremote.authenticate=false")
        .arg("-Dcom.sun.management.jmxremote.ssl=false")
        .arg("TestServer")
        .current_dir("tests/jmxserver")
        .spawn()
        .expect("Could not start JMX server");
    thread::sleep(Duration::from_secs(1));

    run_test();

    // Stop the server once we are done.
    let _ = server.kill();
}

fn run_test() {
    // Create a connection to the remote JMX server.
    let address = MBeanAddress::address(format!("localhost:{}", JMX_PORT));
    let options = MBeanThreadedClientOptions::default().skip_connect(true);
    let client = MBeanThreadedClient::connect_with_options(address.clone(), options)
        .expect("Failed to create JMX client");

    // Ecpect request to fail.
    let result: Result<i32> = client.get_attribute("FOO:name=ServerBean", "ThreadCount");
    assert!(result.is_err());

    // Connect to the server.
    client.reconnect(address.clone()).expect("Failed to connect to the server");

    // Fetch an attribute.
    let threads: i32 = client.get_attribute("FOO:name=ServerBean", "ThreadCount")
        .expect("Failed to fetch threads count");
    assert_eq!(threads, 16);

    // Disconnect from the server.
    let options = MBeanThreadedClientOptions::default().skip_connect(true);
    client.reconnect_with_options(address, options)
        .expect("Failed to disconnect from the server");

    // Ecpect request to fail.
    let result: Result<i32> = client.get_attribute("FOO:name=ServerBean", "ThreadCount");
    assert!(result.is_err());
}

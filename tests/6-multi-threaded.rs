//!
//! This test is also an example of the basic use the library.
//! The client side is limited to the body of the `run_test` function.
//!
//! This test:
//!
//!   1. Connects to a JMX server using a host/port.
//!   2. Wraps the connection into a reference counted mutex.
//!   3. Shares the connection across threads
//!   4. In parallel:
//!     * Fetch specific JMX attributes from each thread.
//!     * Query MBean information for an MBean.
//!     * Query for MBean names.
//!
extern crate jmx;

use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use jmx::MBeanAddress;
use jmx::MBeanClientTrait;
use jmx::MBeanThreadedClient;


static JMX_PORT: u16 = 1622;


#[test]
fn multi_threaded() {
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
    let client = MBeanThreadedClient::connect(
        MBeanAddress::address(format!("localhost:{}", JMX_PORT))
    ).expect("Failed to connect to the JMX test server");
    let client = Arc::new(client);
    let client1 = Arc::clone(&client);
    let client2 = Arc::clone(&client);
    let client3 = Arc::clone(&client);

    // Fetch some attributes from the server.
    let t1 = thread::spawn(move || {
        let schema: String = client1.get_attribute("FOO:name=ServerBean", "SchemaName").unwrap();
        assert_eq!(schema, "test");
    });
    let t2 = thread::spawn(move || {
        // Fetch an MBean information.
        let mbean = client2.get_mbean_info("FOO:name=ServerBean").unwrap();
        assert_eq!(mbean.attributes.len(), 2);
        assert_eq!(mbean.class_name, "JmxServer");
        assert_eq!(mbean.description, "Information on the management interface of the MBean");

        // Assert attributes are as expected.
        let mut attributes = mbean.attributes;
        attributes.sort_by(|a, b| a.name.cmp(&b.name));
        let schema = &attributes[0];
        let threads = &attributes[1];

        assert_eq!(schema.description, "Attribute exposed for management");
        assert_eq!(schema.is_is, false);
        assert_eq!(schema.is_readable, true);
        assert_eq!(schema.is_writable, false);
        assert_eq!(schema.name, "SchemaName");
        assert_eq!(schema.type_name, "java.lang.String");

        assert_eq!(threads.description, "Attribute exposed for management");
        assert_eq!(threads.is_is, false);
        assert_eq!(threads.is_readable, true);
        assert_eq!(threads.is_writable, true);
        assert_eq!(threads.name, "ThreadCount");
        assert_eq!(threads.type_name, "int");
    });
    let t3 = thread::spawn(move || {
        let threads: i32 = client3.get_attribute("FOO:name=ServerBean", "ThreadCount").unwrap();
        assert_eq!(threads, 16);
    });
    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
}

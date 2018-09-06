//!
//! This test is also an example of the basic use the library.
//! The client side is limited to the body of the `run_test` function.
//!
//! This test:
//!
//!   1. Connects to a JMX server instantiating a default JVM.
//!   2. Fetch an MBean information.
//!
extern crate jmx;

use std::process::Command;
use std::thread;
use std::time::Duration;

use jmx::MBeanAddress;
use jmx::MBeanClientTrait;
use jmx::MBeanServer;


static JMX_PORT: u16 = 1618;


#[test]
fn mbean_info() {
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
    let url = MBeanAddress::service_url(format!(
        "service:jmx:rmi://localhost:{}/jndi/rmi://localhost:{}/jmxrmi",
        JMX_PORT, JMX_PORT
    ));
    let server = MBeanServer::connect(url)
        .expect("Failed to connect to the JMX test server");

    // Fetch an MBean information.
    let mbean = server.get_mbean_info("FOO:name=ServerBean").unwrap();
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
}

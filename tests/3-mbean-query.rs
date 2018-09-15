//!
//! This test is also an example of the basic use the library.
//! The client side is limited to the body of the `run_test` function.
//!
//! This test:
//!
//!   1. Connects to a JMX server instantiating a default JVM.
//!   2. Query the MBean server for MBean names.
//!
extern crate jmx;

use std::process::Command;
use std::thread;
use std::time::Duration;

use jmx::MBeanAddress;
use jmx::MBeanClient;
use jmx::MBeanClientTrait;


static JMX_PORT: u16 = 1619;


#[test]
fn mbean_query() {
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
    let server = MBeanClient::connect(url)
        .expect("Failed to connect to the JMX test server");

    // Query MBean names.
    let mut names = server.query_names("java.lang:type=MemoryManager,*", "").unwrap();
    names.sort();
    assert_eq!(names, vec![
        "java.lang:type=MemoryManager,name=CodeCacheManager".to_owned(),
        "java.lang:type=MemoryManager,name=Metaspace Manager".to_owned(),
    ]);
}

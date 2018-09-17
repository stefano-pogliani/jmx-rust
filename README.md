# JMX for Rust
A [JMX](https://en.wikipedia.org/wiki/Java_Management_Extensions) client library for Rust.

This library allows querying Java JMX attributes from a rust project.


## Building
The `jmx-rust` crate is based off of the `j4rs` crate.

The `j4rs` crate needs `JAVA_HOME` variable to be set to the install path of the JDK for builds
and the `LD_LIBRARY_PATH` to the directory containing `libjvm.so`.

```bash
# For Fedora 28:
export JAVA_HOME="/usr/lib/jvm/java-1.8.0-openjdk-1.8.0.181-7.b13.fc28.x86_64"
export LD_LIBRARY_PATH="${JAVA_HOME}/jre/lib/amd64/server:$LD_LIBRARY_PATH"
```

### Tests
Tests work but starting a test JMX server located under `tests/jmxserver`.
This server is then used by the tests to check the library.

For this to work the test server must be compiled and the correct `java` command
must be available in the `$PATH`:

```bash
cd tests/jmxserver
javac TestServer.java
cd ../..

export PATH="/path/to/java/bin:$PATH"
cargo test
```


## Usage
Creating a client:
```rust
extern crate jmx;

static JMX_PORT: i32 = 1234;

fn main() {
    // Create a connection to the remote JMX server.
    let url = MBeanAddress::service_url(format!(
        "service:jmx:rmi://localhost:{}/jndi/rmi://localhost:{}/jmxrmi",
        JMX_PORT, JMX_PORT
    ));
    let client = MBeanClient::connect(url)
        .expect("Failed to connect to the JMX server");

    // Fetch some attribute from the server.
    let threads: i32 = client.get_attribute("FOO:name=ServerBean", "ThreadCount").unwrap();
}
```

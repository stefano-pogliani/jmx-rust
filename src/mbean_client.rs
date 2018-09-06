use j4rs::Instance;
use j4rs::InvocationArg;
use j4rs::Jvm;
use j4rs::new_jvm;
use serde::de::DeserializeOwned;

use super::MBeanAddress;
use super::MBeanClientTrait;
use super::MBeanInfo;
use super::Result;

use super::constants::JMX_CONNECTOR_FACTORY;
use super::constants::JMX_OBJECT_NAME;
use super::constants::JMX_QUERY_EXP;

use super::util::to_vec;


/// Interface to a remote MBean server.
///
/// Wrapper around the following Java classes:
///
///   * javax.management.remote.JMXServiceURL
///   * javax.management.remote.JMXConnectorFactory
///   * javax.management.MBeanServerConnection
pub struct MBeanClient {
    connection: Instance,
    jvm: Jvm,
    // We access the server from the connection.
    _server: Instance,
}

impl MBeanClient {
    /// Create an `MBeanClient` instance connected to the given address.
    pub fn connect(address: MBeanAddress) -> Result<MBeanClient> {
        MBeanClient::connect_with_options(address, MBeanClientOptions::default())
    }

    /// Create an `MBeanClient` instance connected to the given address and options.
    pub fn connect_with_options(
        address: MBeanAddress, options: MBeanClientOptions
    ) -> Result<MBeanClient> {
        let jvm = MBeanClient::into_jvm(options.jvm)?;
        let service_url = address.for_java(&jvm)?;
        MBeanClient::connect_service_url(jvm, service_url)
    }
}

impl MBeanClient {
    /// Helper to create an MBeanClient given a service url.
    fn connect_service_url(jvm: Jvm, service_url: Instance) -> Result<MBeanClient> {
        let server = MBeanClient::mbean_server(&jvm, service_url)?;
        let connection = MBeanClient::get_connection(&jvm, &server)?;
        Ok(MBeanClient {
            connection,
            jvm,
            _server: server,
        })
    }

    /// Helper to obtain a `javax.management.MBeanServerConnection` instance.
    fn get_connection(jvm: &Jvm, server: &Instance) -> Result<Instance> {
        let connection = jvm.invoke(server, "getMBeanServerConnection", &vec![])?;
        Ok(connection)
    }

    /// Create a default JVM if none was provided.
    fn into_jvm(jvm: Option<Jvm>) -> Result<Jvm> {
        if let Some(jvm) = jvm {
            Ok(jvm)
        } else {
            Ok(new_jvm(vec![], vec![])?)
        }
    }

    /// Helper to obtain a `javax.management.remote.JMXConnectorFactory` instance.
    fn mbean_server(jvm: &Jvm, service_url: Instance) -> Result<Instance> {
        let server = jvm.invoke_static(
            JMX_CONNECTOR_FACTORY, "connect",
            &vec![InvocationArg::from(service_url)]
        )?;
        Ok(server)
    }
}

impl MBeanClientTrait for MBeanClient {
    fn get_attribute<S1, S2, T>(&self, mbean: S1, attribute: S2) -> Result<T>
        where S1: Into<String>,
              S2: Into<String>,
              T: DeserializeOwned,
    {
        let object_name = self.jvm.create_instance(
            JMX_OBJECT_NAME, &vec![InvocationArg::from(mbean.into())]
        )?;
        let value = self.jvm.invoke(
            &self.connection, "getAttribute",
            &vec![InvocationArg::from(object_name), InvocationArg::from(attribute.into())]
        )?;
        Ok(self.jvm.to_rust(value)?)
    }

    fn get_mbean_info<S>(&self, mbean: S) -> Result<MBeanInfo>
        where S: Into<String>,
    {
        let object_name = self.jvm.create_instance(
            JMX_OBJECT_NAME, &vec![InvocationArg::from(mbean.into())]
        )?;
        let info = self.jvm.invoke(
            &self.connection, "getMBeanInfo",
            &vec![InvocationArg::from(object_name)]
        )?;
        MBeanInfo::from_instance(&self.jvm, info)
    }

    fn query_names<S1, S2>(&self, name: S1, query: S2) -> Result<Vec<String>>
        where S1: Into<String>,
              S2: Into<String>,
    {
        let name = self.jvm.create_instance(
            JMX_OBJECT_NAME, &vec![InvocationArg::from(name.into())]
        )?;
        let query = self.jvm.create_instance(
            JMX_OBJECT_NAME, &vec![InvocationArg::from(query.into())]
        )?;
        let query = self.jvm.cast(&query, JMX_QUERY_EXP)?;
        let names = self.jvm.invoke(
            &self.connection, "queryNames",
            &vec![InvocationArg::from(name), InvocationArg::from(query)]
        )?;
        let names = self.jvm.invoke(&names, "toArray", &vec![])?;
        let names = to_vec(&self.jvm, names, JMX_OBJECT_NAME)?;
        let mut result = Vec::new();
        for instance in names {
            let instance = self.jvm.invoke(&instance, "toString", &vec![])?;
            let name: String = self.jvm.to_rust(instance)?;
            result.push(name);
        }
        Ok(result)
    }
}


/// Additional `MBeanClient` connection options.
pub struct MBeanClientOptions {
    jvm: Option<Jvm>,
}

impl MBeanClientOptions {
    /// Use the given JVM instance instead of creating one.
    pub fn jvm(mut self, jvm: Jvm) -> Self {
        self.jvm = Some(jvm);
        self
    }
}

impl Default for MBeanClientOptions {
    fn default() -> Self {
        MBeanClientOptions {
            jvm: None,
        }
    }
}

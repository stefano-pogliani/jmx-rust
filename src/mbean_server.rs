use j4rs::Instance;
use j4rs::InvocationArg;
use j4rs::Jvm;
use j4rs::new_jvm;
use serde::de::DeserializeOwned;

use super::MBeanInfo;
use super::Result;
use super::util::to_vec;


static JMX_CONNECTOR_FACTORY: &'static str = "javax.management.remote.JMXConnectorFactory";
static JMX_OBJECT_NAME: &'static str = "javax.management.ObjectName";
static JMX_QUERY_EXP: &'static str = "javax.management.QueryExp";
static JMX_SERVICE_URL: &'static str = "javax.management.remote.JMXServiceURL";


/// Interface to a remote MBean server.
///
/// Wrapper around the following Java classes:
///
///   * javax.management.remote.JMXServiceURL
///   * javax.management.remote.JMXConnectorFactory
///   * javax.management.MBeanServerConnection
pub struct MBeanServer {
    connection: Instance,
    jvm: Jvm,
    // We access the server from the connection.
    _server: Instance,
}

impl MBeanServer {
    /// Create an `MBeanServer` instance connected to the given service `url`.
    pub fn connect<S>(url: S, jvm: Option<Jvm>) -> Result<MBeanServer>
        where S: Into<String>,
    {
        let jvm = MBeanServer::into_jvm(jvm)?;
        let service_url = MBeanServer::service_url(&jvm, url.into())?;
        let server = MBeanServer::mbean_server(&jvm, service_url)?;
        let connection = MBeanServer::get_connection(&jvm, &server)?;
        Ok(MBeanServer {
            connection,
            jvm,
            _server: server,
        })
    }

    /// Get the value of a specific MBean attribute.
    pub fn get_attribute<S1, S2, T>(&self, mbean: S1, attribute: S2) -> Result<T>
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

    /// Get information about an MBean.
    pub fn get_mbean_info<S>(&self, mbean: S) -> Result<MBeanInfo>
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

    /// Query for the names of MBeans on the JMX server.
    pub fn query_names<S1, S2>(&self, name: S1, query: S2) -> Result<Vec<String>>
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

impl MBeanServer {
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

    /// Convert the connection string to a `javax.management.remote.JMXServiceURL`.
    fn service_url(jvm: &Jvm, connection: String) -> Result<Instance> {
        let url = jvm.create_instance(
            JMX_SERVICE_URL, &vec![InvocationArg::from(connection)]
        )?;
        Ok(url)
    }
}

use failure::ResultExt;
use j4rs::Instance;
use j4rs::InvocationArg;
use j4rs::Jvm;
use j4rs::JvmBuilder;
use serde::de::DeserializeOwned;

use super::ErrorKind;
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
        let jvm = options.jvm.build().with_context(|_| ErrorKind::JvmInit)?;
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
        let connection = jvm.invoke(server, "getMBeanServerConnection", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(server.class_name().to_string(), "getMBeanServerConnection")
        )?;
        Ok(connection)
    }

    /// Helper to obtain a `javax.management.remote.JMXConnectorFactory` instance.
    fn mbean_server(jvm: &Jvm, service_url: Instance) -> Result<Instance> {
        let server = jvm.invoke_static(
            JMX_CONNECTOR_FACTORY, "connect",
            &vec![InvocationArg::from(service_url)]
        ).with_context(|_| ErrorKind::JavaInvokeStatic(JMX_CONNECTOR_FACTORY, "connect"))?;
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
        ).with_context(|_| ErrorKind::JavaCreateInstance(JMX_OBJECT_NAME))?;
        let value = self.jvm.invoke(
            &self.connection, "getAttribute",
            &vec![InvocationArg::from(object_name), InvocationArg::from(attribute.into())]
        ).with_context(
            |_| ErrorKind::JavaInvoke(self.connection.class_name().to_string(), "getAttribute")
        )?;
        let value: T = self.jvm.to_rust(value).with_context(|_| ErrorKind::RustCast("<dynamic>"))?;
        Ok(value)
    }

    fn get_mbean_info<S>(&self, mbean: S) -> Result<MBeanInfo>
        where S: Into<String>,
    {
        let object_name = self.jvm.create_instance(
            JMX_OBJECT_NAME, &vec![InvocationArg::from(mbean.into())]
        ).with_context(|_| ErrorKind::JavaCreateInstance(JMX_OBJECT_NAME))?;
        let info = self.jvm.invoke(
            &self.connection, "getMBeanInfo",
            &vec![InvocationArg::from(object_name)]
        ).with_context(
            |_| ErrorKind::JavaInvoke(self.connection.class_name().to_string(), "getMBeanInfo")
        )?;
        MBeanInfo::from_instance(&self.jvm, info)
    }

    fn query_names<S1, S2>(&self, name: S1, query: S2) -> Result<Vec<String>>
        where S1: Into<String>,
              S2: Into<String>,
    {
        let name = self.jvm.create_instance(
            JMX_OBJECT_NAME, &vec![InvocationArg::from(name.into())]
        ).with_context(|_| ErrorKind::JavaCreateInstance(JMX_OBJECT_NAME))?;
        let query = self.jvm.create_instance(
            JMX_OBJECT_NAME, &vec![InvocationArg::from(query.into())]
        ).with_context(|_| ErrorKind::JavaCreateInstance(JMX_OBJECT_NAME))?;
        let query = self.jvm.cast(&query, JMX_QUERY_EXP)
            .with_context(|_| ErrorKind::JavaCast(JMX_QUERY_EXP.to_string()))?;
        let names = self.jvm.invoke(
            &self.connection, "queryNames",
            &vec![InvocationArg::from(name), InvocationArg::from(query)]
        ).with_context(
            |_| ErrorKind::JavaInvoke(self.connection.class_name().to_string(), "queryNames")
        )?;
        let names = self.jvm.invoke(&names, "toArray", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(names.class_name().to_string(), "toArray")
        )?;
        let names = to_vec(&self.jvm, names, JMX_OBJECT_NAME)?;
        let mut result = Vec::new();
        for instance in names {
            let instance = self.jvm.invoke(&instance, "toString", &vec![]).with_context(
                |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "toString")
            )?;
            let name: String = self.jvm.to_rust(instance)
                .with_context(|_| ErrorKind::RustCast("String"))?;
            result.push(name);
        }
        Ok(result)
    }
}


/// Additional `MBeanClient` connection options.
pub struct MBeanClientOptions<'a> {
    jvm: JvmBuilder<'a>,
}

impl<'a> MBeanClientOptions<'a> {
    /// Use the given JvmBuilder instance instead of the default one.
    pub fn builder(mut self, builder: JvmBuilder<'a>) -> Self {
        self.jvm = builder;
        self
    }
}

impl<'a> Default for MBeanClientOptions<'a> {
    fn default() -> Self {
        MBeanClientOptions {
            jvm: JvmBuilder::new(),
        }
    }
}

use failure::ResultExt;
use j4rs::Instance;
use j4rs::InvocationArg;
use j4rs::Jvm;
use serde::de::DeserializeOwned;

use std::convert::TryFrom;
use super::ErrorKind;
use super::MBeanInfo;
use super::Result;

use super::constants::JMX_SERVICE_URL;


/// Address of a remote JMX server.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum MBeanAddress {
    Address(String),
    ServiceUrl(String),
}

impl MBeanAddress {
    /// Connected to the remote "host:port" JMX server.
    ///
    /// This method connects to a remote server using the RMI protocol
    /// and the default RMI path "/jndi/rmi://{host}:{port}/jmxrmi".
    pub fn address<S>(address: S) -> MBeanAddress
        where S: Into<String>,
    {
        MBeanAddress::Address(address.into())
    }

    /// Create a Java `javax.management.remote.JMXServiceURL` instance.
    pub fn for_java(self, jvm: &Jvm) -> Result<Instance> {
        let instance = match self {
            MBeanAddress::Address(address) => {
                let url = format!(
                    "service:jmx:rmi://{}/jndi/rmi://{}/jmxrmi",
                    address, address
                );
                jvm.create_instance(JMX_SERVICE_URL, &vec![InvocationArg::try_from(url)?])
                    .with_context(|_| ErrorKind::JavaCreateInstance(JMX_SERVICE_URL))?
            },
            MBeanAddress::ServiceUrl(url) => jvm.create_instance(
                JMX_SERVICE_URL, &vec![InvocationArg::try_from(url)?]
            ).with_context(|_| ErrorKind::JavaCreateInstance(JMX_SERVICE_URL))?,
        };
        Ok(instance)
    }

    /// Connected to the remote JMX server by ServiceUrl.
    pub fn service_url<S>(service_url: S) -> MBeanAddress
        where S: Into<String>,
    {
        MBeanAddress::ServiceUrl(service_url.into())
    }
}


/// Trait definition for all MBean clients.
pub trait MBeanClientTrait {
    /// Get the value of a specific MBean attribute.
    fn get_attribute<S1, S2, T>(&self, mbean: S1, attribute: S2) -> Result<T>
        where S1: Into<String>,
              S2: Into<String>,
              T: DeserializeOwned;

    /// Get information about an MBean.
    fn get_mbean_info<S>(&self, mbean: S) -> Result<MBeanInfo>
        where S: Into<String>;

    /// Query for the names of MBeans on the JMX server.
    fn query_names<S1, S2>(&self, name: S1, query: S2) -> Result<Vec<String>>
        where S1: Into<String>,
              S2: Into<String>;
}

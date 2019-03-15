use failure::ResultExt;
use j4rs::Instance;
use j4rs::Jvm;

use super::ErrorKind;
use super::Result;

use super::constants::JMX_MBEAN_ATTRIBUTE_INFO;
use super::constants::JMX_MBEAN_FEATURE_INFO;

use super::util::to_vec;


/// Metadata about an MBean attribute.
///
/// Rust version of `javax.management.MBeanAttributeInfo`
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct MBeanAttribute {
    pub description: String,
    // descriptor
    pub is_is: bool,
    pub is_readable: bool,
    pub is_writable: bool,
    pub name: String,
    pub type_name: String,
}

impl MBeanAttribute {
    /// Create an `MBeanAttribute` instance from a `javax.management.MBeanAttributeInfo`
    /// java instance.
    pub fn from_instance(jvm: &Jvm, instance: Instance) -> Result<MBeanAttribute> {
        let is_is = jvm.invoke(&instance, "isIs", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "isIs")
        )?;
        let is_is: bool = jvm.to_rust(is_is)
            .with_context(|_| ErrorKind::RustCast("bool"))?;
        let is_readable = jvm.invoke(&instance, "isReadable", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "isReadable")
        )?;
        let is_readable: bool = jvm.to_rust(is_readable)
            .with_context(|_| ErrorKind::RustCast("bool"))?;
        let is_writable = jvm.invoke(&instance, "isWritable", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "isWritable")
        )?;
        let is_writable: bool = jvm.to_rust(is_writable)
            .with_context(|_| ErrorKind::RustCast("bool"))?;
        let type_name = jvm.invoke(&instance, "getType", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "getType")
        )?;
        let type_name: String = jvm.to_rust(type_name)
            .with_context(|_| ErrorKind::RustCast("String"))?;

        // Cast to javax.management.MBeanFeatureInfo for the other attributes.
        let instance = jvm.cast(&instance, JMX_MBEAN_FEATURE_INFO)
            .with_context(|_| ErrorKind::JavaCast(JMX_MBEAN_FEATURE_INFO.to_string()))?;
        let description = jvm.invoke(&instance, "getDescription", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "getDescription")
        )?;
        let description: String = jvm.to_rust(description)
            .with_context(|_| ErrorKind::RustCast("String"))?;
        let name = jvm.invoke(&instance, "getName", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "getName")
        )?;
        let name: String = jvm.to_rust(name).with_context(|_| ErrorKind::RustCast("String"))?;
        Ok(MBeanAttribute {
            description,
            is_is,
            is_readable,
            is_writable,
            name,
            type_name,
        })
    }
}


/// Metadata about an MBean.
///
/// Rust version of `javax.management.MBeanInfo`
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct MBeanInfo {
    pub attributes: Vec<MBeanAttribute>,
    pub class_name: String,
    // constructor,
    pub description: String,
    // descriptor,
    // notifications,
    // operations,
}

impl MBeanInfo {
    /// Create an `MBeanInfo` instance from a `javax.management.MBeanInfo` java instance.
    pub fn from_instance(jvm: &Jvm, instance: Instance) -> Result<MBeanInfo> {
        let attributes = MBeanInfo::attributes_from_instance(jvm, &instance)?;
        let class_name = jvm.invoke(&instance, "getClassName", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "getClassName")
        )?;
        let class_name: String = jvm.to_rust(class_name)
            .with_context(|_| ErrorKind::RustCast("String"))?;
        let description = jvm.invoke(&instance, "getDescription", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "getDescription")
        )?;
        let description: String = jvm.to_rust(description)
            .with_context(|_| ErrorKind::RustCast("String"))?;
        Ok(MBeanInfo {
            attributes,
            class_name,
            description,
        })
    }
}

impl MBeanInfo {
    fn attributes_from_instance(jvm: &Jvm, instance: &Instance) -> Result<Vec<MBeanAttribute>> {
        let raw = jvm.invoke(instance, "getAttributes", &vec![]).with_context(
            |_| ErrorKind::JavaInvoke(instance.class_name().to_string(), "getAttributes")
        )?;
        let raw = to_vec(jvm, raw, JMX_MBEAN_ATTRIBUTE_INFO)?;
        let mut attributes = Vec::new();
        for instance in raw {
            attributes.push(MBeanAttribute::from_instance(jvm, instance)?);
        }
        Ok(attributes)
    }
}

use j4rs::Instance;
use j4rs::Jvm;

use super::Result;
use super::util::to_vec;


static JMX_MBEAN_ATTRIBUTE_INFO: &'static str = "javax.management.MBeanAttributeInfo";
static JMX_MBEAN_FEATURE_INFO: &'static str = "javax.management.MBeanFeatureInfo";


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
        let is_is = jvm.invoke(&instance, "isIs", &vec![])?;
        let is_is: bool = jvm.to_rust(is_is)?;
        let is_readable = jvm.invoke(&instance, "isReadable", &vec![])?;
        let is_readable: bool = jvm.to_rust(is_readable)?;
        let is_writable = jvm.invoke(&instance, "isWritable", &vec![])?;
        let is_writable: bool = jvm.to_rust(is_writable)?;
        let type_name = jvm.invoke(&instance, "getType", &vec![])?;
        let type_name: String = jvm.to_rust(type_name)?;

        // Cast to javax.management.MBeanFeatureInfo for the other attributes.
        let instance = jvm.cast(&instance, JMX_MBEAN_FEATURE_INFO)?;
        let description = jvm.invoke(&instance, "getDescription", &vec![])?;
        let description: String = jvm.to_rust(description)?;
        let name = jvm.invoke(&instance, "getName", &vec![])?;
        let name: String = jvm.to_rust(name)?;
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
        let class_name = jvm.invoke(&instance, "getClassName", &vec![])?;
        let class_name: String = jvm.to_rust(class_name)?;
        let description = jvm.invoke(&instance, "getDescription", &vec![])?;
        let description: String = jvm.to_rust(description)?;
        Ok(MBeanInfo {
            attributes,
            class_name,
            description,
        })
    }
}

impl MBeanInfo {
    fn attributes_from_instance(jvm: &Jvm, instance: &Instance) -> Result<Vec<MBeanAttribute>> {
        let raw = jvm.invoke(instance, "getAttributes", &vec![])?;
        let raw = to_vec(jvm, raw, JMX_MBEAN_ATTRIBUTE_INFO)?;
        let mut attributes = Vec::new();
        for instance in raw {
            attributes.push(MBeanAttribute::from_instance(jvm, instance)?);
        }
        Ok(attributes)
    }
}

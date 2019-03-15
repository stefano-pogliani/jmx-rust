use failure::ResultExt;
use j4rs::Instance;
use j4rs::InvocationArg;
use j4rs::Jvm;

use super::ErrorKind;
use super::Result;


use super::constants::JAVA_LANG_INTEGER;
use super::constants::JAVA_LANG_OBJECT;
use super::constants::JAVA_REFLECT_ARRAY;


/// Helper function to convert a Java native array into a rust vector.
///
/// Arrays are converted into vectors of instances using java reflection methods:
///
///   1. The array is cast to `java.lang.Object`.
///   2. The `java.lang.reflect.Array` static methods are used to iterate over the array.
///   3. Each item is cast back into the given `class` and added to the result vector.
pub fn to_vec(jvm: &Jvm, instance: Instance, class: &str) -> Result<Vec<Instance>> {
    let instance = jvm.cast(&instance, JAVA_LANG_OBJECT)
        .with_context(|_| ErrorKind::JavaCast(JAVA_LANG_OBJECT.into()))?;
    let instance_for_len = jvm.clone_instance(&instance).with_context(|_| ErrorKind::JavaClone)?;
    let length = jvm.invoke_static(
        JAVA_REFLECT_ARRAY, "getLength",
        &vec![InvocationArg::from(instance_for_len)]
    ).with_context(|_| ErrorKind::JavaInvokeStatic(JAVA_REFLECT_ARRAY, "getLength"))?;
    let length: i32 = jvm.to_rust(length).with_context(|_| ErrorKind::RustCast("i32"))?;
    let mut vector = Vec::new();
    for idx in 0..length {
        let idx = jvm.create_instance(
            JAVA_LANG_INTEGER, &vec![InvocationArg::from(format!("{}", idx))]
        ).with_context(|_| ErrorKind::JavaCreateInstance(JAVA_LANG_INTEGER))?;
        let idx = jvm.invoke(&idx, "intValue", &vec![])
            .with_context(|_| ErrorKind::JavaInvoke(JAVA_LANG_INTEGER.to_string(), "intValue"))?;
        let instance_for_get = jvm.clone_instance(&instance)
            .with_context(|_| ErrorKind::JavaClone)?;
        let value = jvm.invoke_static(
            JAVA_REFLECT_ARRAY, "get",
            &vec![InvocationArg::from(instance_for_get), InvocationArg::from(idx)]
        ).with_context(|_| ErrorKind::JavaInvokeStatic(JAVA_REFLECT_ARRAY, "get"))?;
        let value = jvm.cast(&value, class)
            .with_context(|_| ErrorKind::JavaCast(class.to_string()))?;
        vector.push(value);
    }
    Ok(vector)
}

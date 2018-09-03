use j4rs::Instance;
use j4rs::InvocationArg;
use j4rs::Jvm;

use super::Result;


static JAVA_LANG_INTEGER: &'static str = "java.lang.Integer";
static JAVA_REFLECT_ARRAY: &'static str = "java.lang.reflect.Array";


/// Helper function to convert a Java native array into a rust vector.
///
/// Arrays are converted into vectors of instances using java reflection methods:
///
///   1. The array is cast to `java.lang.Object`.
///   2. The `java.lang.reflect.Array` static methods are used to iterate over the array.
///   3. Each item is cast back into the given `class` and added to the result vector.
pub fn to_vec(jvm: &Jvm, instance: Instance, class: &str) -> Result<Vec<Instance>> {
    let instance = jvm.cast(&instance, "java.lang.Object")?;
    let length: i32 = jvm.to_rust(jvm.invoke_static(
        JAVA_REFLECT_ARRAY, "getLength",
        &vec![InvocationArg::from(instance.clone())]
    )?)?;
    let mut vector = Vec::new();
    for idx in 0..length {
        let idx = jvm.create_instance(
            JAVA_LANG_INTEGER, &vec![InvocationArg::from(format!("{}", idx))]
        )?;
        let idx = jvm.invoke(&idx, "intValue", &vec![])?;
        let value = jvm.invoke_static(
            JAVA_REFLECT_ARRAY, "get",
            &vec![InvocationArg::from(instance.clone()), InvocationArg::from(idx)]
        )?;
        let value = jvm.cast(&value, class)?;
        vector.push(value);
    }
    Ok(vector)
}

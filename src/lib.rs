extern crate failure;
extern crate j4rs;
extern crate serde;
#[macro_use]
extern crate serde_derive;


mod base;
mod constants;
mod error;
mod mbean_client;
mod mbean_info;
mod util;


pub use self::error::Error;
pub use self::error::ErrorKind;
pub use self::error::Result;

pub use self::base::MBeanAddress;
pub use self::base::MBeanClientTrait;
pub use self::mbean_client::MBeanClient;
pub use self::mbean_client::MBeanClientOptions;
pub use self::mbean_info::MBeanInfo;


// Threaded support feature.
#[cfg(feature = "thread-support")]
extern crate crossbeam_channel;
#[cfg(feature = "thread-support")]
extern crate serde_json;

#[cfg(feature = "thread-support")]
mod mbean_thread;

#[cfg(feature = "thread-support")]
pub use self::mbean_thread::MBeanThreadedClient;
#[cfg(feature = "thread-support")]
pub use self::mbean_thread::MBeanThreadedClientOptions;

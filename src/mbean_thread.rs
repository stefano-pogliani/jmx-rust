use std::thread::Builder;
use std::thread::JoinHandle;

use crossbeam_channel as channel;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

use j4rs::Jvm;
use j4rs::new_jvm;

use serde::de::DeserializeOwned;
use serde_json;
use serde_json::Value;

use super::MBeanAddress;
use super::MBeanClient;
use super::MBeanClientOptions;
use super::MBeanClientTrait;
use super::MBeanInfo;
use super::Error;
use super::Result;
use super::ResultExt;


/// Encode requests sent to the background `MBeanClient`.
enum MBeanRequest {
    /// Ask the worker to perform a `get_attribute` call.
    GetAttribute(String, String, Sender<Result<Value>>),

    /// Ask the worker to perform a `get_mbean_info` call.
    GetMBeanInfo(String, Sender<Result<MBeanInfo>>),

    /// Ask the worker to perform a `query_names` call.
    QueryNames(String, String, Sender<Result<Vec<String>>>),

    /// Request termination of the background thread.
    Quit,

    /// Request the MBean client to re-connect to the given address with the given options.
    Reconnect(MBeanAddress, MBeanThreadedClientOptions, Sender<Result<()>>),
}


/// Encapsulate the logic and state of the async worker thread.
struct MBeanThreadWorker {
    client: Option<MBeanClient>,
    jvm: Jvm,
    receiver: Receiver<MBeanRequest>,
}

impl MBeanThreadWorker {
    fn new(jvm: Jvm, receiver: Receiver<MBeanRequest>) -> MBeanThreadWorker {
        MBeanThreadWorker {
            client: None,
            jvm,
            receiver,
        }
    }

    /// Access the MBeanClient, if one is available.
    fn client(&self) -> Result<&MBeanClient> {
        self.client.as_ref().ok_or_else(|| "The JMX client is not connected".into())
    }

    /// Wait for requests from other threads and process them.
    fn work(&mut self) {
        for request in &self.receiver {
            match request {
                MBeanRequest::GetAttribute(mbean, attribute, sender) => {
                    let response: Result<Value> = self.client()
                        .and_then(|c| c.get_attribute(mbean, attribute));
                    sender.send(response);
                },
                MBeanRequest::GetMBeanInfo(mbean, sender) => {
                    let response = self.client().and_then(|c| c.get_mbean_info(mbean));
                    sender.send(response);
                },
                MBeanRequest::QueryNames(name, query, sender) => {
                    let response = self.client().and_then(|c| c.query_names(name, query));
                    sender.send(response);
                },
                MBeanRequest::Quit => break,
                MBeanRequest::Reconnect(address, options, sender) => {
                    if options.skip_connect {
                        self.client = None;
                        sender.send(Ok(()));
                    } else {
                        let options = options.into_client_options().jvm(self.jvm.clone());
                        match MBeanClient::connect_with_options(address, options) {
                            Err(error) => sender.send(Err(error)),
                            Ok(new_client) => {
                                self.client = Some(new_client);
                                sender.send(Ok(()));
                            }
                        };
                    }
                },
            };
        }
    }
}


/// Implementation of a thread safe `MBeanClient`.
pub struct MBeanThreadedClient {
    // Sender end of the channel to the background thread.
    send_to_worker: Sender<MBeanRequest>,
    // Background worker is `None` only after `Drop::drop` is called.
    worker: Option<JoinHandle<()>>,
}

impl MBeanThreadedClient {
    /// Create an `MBeanThreadedClient` instance connected to the given address.
    ///
    /// See `MBeanClient::connect` for more details.
    pub fn connect(address: MBeanAddress) -> Result<MBeanThreadedClient> {
        MBeanThreadedClient::connect_with_options(address, MBeanThreadedClientOptions::default())
    }

    /// Create an `MBeanThreadedClient` instance connected to the given address and options.
    ///
    /// See `MBeanClient::connect_with_options` for more details.
    pub fn connect_with_options(
        address: MBeanAddress, options: MBeanThreadedClientOptions
    ) -> Result<MBeanThreadedClient> {
        let (send_client_error, receive_client_error): (Sender<Error>, Receiver<Error>) =
            channel::bounded(1);
        let (send_to_worker, worker_receiver) = match options.reqs_buffer {
            None => channel::unbounded(),
            Some(size) => channel::bounded(size),
        };
        let worker = Builder::new().name("MBeanThreadedClient::worker".into()).spawn(|| {
            let jvm = match new_jvm(vec![], vec![]) {
                Ok(jvm) => jvm,
                Err(error) => {
                    send_client_error.send(error.into());
                    return;
                }
            };
            drop(send_client_error);
            let mut worker = MBeanThreadWorker::new(jvm, worker_receiver);
            worker.work();
        })?;
        if let Some(error) = receive_client_error.recv() {
            return Err(error).chain_err(|| "Background thread failed to create JVM");
        }
        let client = MBeanThreadedClient {
            send_to_worker,
            worker: Some(worker),
        };
        if !options.skip_connect {
            client.reconnect_with_options(address, options)?;
        }
        Ok(client)
    }

    /// Request the MBean client to re-connect to the given address.
    pub fn reconnect(&self, address: MBeanAddress) -> Result<()> {
        self.reconnect_with_options(address, MBeanThreadedClientOptions::default())
    }

    /// Request the MBean client to re-connect to the given address with the given options.
    pub fn reconnect_with_options(
        &self, address: MBeanAddress, options: MBeanThreadedClientOptions
    ) -> Result<()> {
        let (sender, receiver) = channel::bounded(1);
        self.send_to_worker.send(MBeanRequest::Reconnect(address, options, sender));
        match receiver.recv() {
            None => Err("Background worker did not send a response".into()),
            Some(result) => result,
        }
    }
}

impl Drop for MBeanThreadedClient {
    fn drop(&mut self) {
        self.send_to_worker.send(MBeanRequest::Quit);
        let _err = self.worker.take().unwrap().join();
    }
}

impl MBeanClientTrait for MBeanThreadedClient {
    fn get_attribute<S1, S2, T>(&self, mbean: S1, attribute: S2) -> Result<T>
        where S1: Into<String>,
              S2: Into<String>,
              T: DeserializeOwned,
    {
        let (sender, receiver) = channel::bounded(1);
        self.send_to_worker.send(MBeanRequest::GetAttribute(
            mbean.into(), attribute.into(), sender
        ));
        let value: Value = match receiver.recv() {
            None => Err("Background worker did not send a response".into()),
            Some(result) => result,
        }?;
        let value: T = serde_json::from_value(value)?;
        Ok(value)
    }

    fn get_mbean_info<S>(&self, mbean: S) -> Result<MBeanInfo>
        where S: Into<String>,
    {
        let (sender, receiver) = channel::bounded(1);
        self.send_to_worker.send(MBeanRequest::GetMBeanInfo(mbean.into(), sender));
        match receiver.recv() {
            None => Err("Background worker did not send a response".into()),
            Some(result) => result,
        }
    }

    fn query_names<S1, S2>(&self, name: S1, query: S2) -> Result<Vec<String>>
        where S1: Into<String>,
              S2: Into<String>,
    {
        let (sender, receiver) = channel::bounded(1);
        self.send_to_worker.send(MBeanRequest::QueryNames(name.into(), query.into(), sender));
        match receiver.recv() {
            None => Err("Background worker did not send a response".into()),
            Some(result) => result,
        }
    }
}


/// Additional `MBeanThreadedClient` connection options.
#[derive(Clone)]
pub struct MBeanThreadedClientOptions {
    reqs_buffer: Option<usize>,
    skip_connect: bool,
}

impl MBeanThreadedClientOptions {
    /// Clear the requests buffer size so unlimited requests are buffered.
    pub fn requests_buffer_unlimited(mut self) -> Self {
        self.reqs_buffer = None;
        self
    }

    /// Set the requests buffer size.
    pub fn requests_buffer_size(mut self, size: usize) -> Self {
        self.reqs_buffer = Some(size);
        self
    }

    pub fn skip_connect(mut self, skip: bool) -> Self {
        self.skip_connect = skip;
        self
    }
}

impl MBeanThreadedClientOptions {
    /// Converts multi-threaded connection options to MBeanClientOptions.
    fn into_client_options(self) -> MBeanClientOptions {
        MBeanClientOptions::default()
    }
}

impl Default for MBeanThreadedClientOptions {
    fn default() -> Self {
        MBeanThreadedClientOptions {
            reqs_buffer: None,
            skip_connect: false,
        }
    }
}

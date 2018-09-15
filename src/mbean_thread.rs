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
        let (send_client_error, receive_client_error) = channel::bounded(1);
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
            let options = options.into_client_options().jvm(jvm.clone());
            let client = match MBeanClient::connect_with_options(address, options) {
                Ok(client) => client,
                Err(error) => {
                    send_client_error.send(error);
                    return;
                },
            };
            drop(send_client_error);
            MBeanThreadedClient::worker_loop(jvm, client, worker_receiver);
        })?;
        if let Some(error) = receive_client_error.recv() {
            return Err(error).chain_err(|| "Background thread failed to create JVM");
        }
        Ok(MBeanThreadedClient {
            send_to_worker,
            worker: Some(worker),
        })
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

impl MBeanThreadedClient {
    /// Wait for requests from other threads and process them.
    fn worker_loop(jvm: Jvm, client: MBeanClient, receiver: Receiver<MBeanRequest>) {
        let mut client = client;
        for request in receiver {
            match request {
                MBeanRequest::GetAttribute(mbean, attribute, sender) => {
                    let response: Result<Value> = client.get_attribute(mbean, attribute);
                    sender.send(response);
                },
                MBeanRequest::GetMBeanInfo(mbean, sender) => {
                    let response = client.get_mbean_info(mbean);
                    sender.send(response);
                },
                MBeanRequest::QueryNames(name, query, sender) => {
                    let response = client.query_names(name, query);
                    sender.send(response);
                },
                MBeanRequest::Quit => break,
                MBeanRequest::Reconnect(address, options, sender) => {
                    let options = options.into_client_options().jvm(jvm.clone());
                    match MBeanClient::connect_with_options(address, options) {
                        Err(error) => sender.send(Err(error)),
                        Ok(new_client) => {
                            client = new_client;
                            sender.send(Ok(()));
                        }
                    };
                },
            };
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
pub struct MBeanThreadedClientOptions {
    reqs_buffer: Option<usize>,
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
        }
    }
}

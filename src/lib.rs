use std::{
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
    marker::PhantomData,
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
};

use orinoco_proc_macro::orinoco_rpc;
use rkyv::rancor;
use uuid::Uuid;

pub use orinoco_proc_macro::orinoco_thread_rpc;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum OrinocoError {
    Unknown(String),
}
impl Display for OrinocoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self:#?}")
    }
}
impl std::error::Error for OrinocoError {}

impl From<anyhow::Error> for OrinocoError {
    fn from(value: anyhow::Error) -> Self {
        OrinocoError::Unknown(format!("{value:#?}"))
    }
}

// #[orinoco_rpc]
pub trait AnyInterface {
    fn open(&self, path: String);
    fn hello(&self, input: String) -> Result<String, OrinocoError>;
    fn goodbye(&self, other: bool, input: String) -> Result<String, OrinocoError>;
}

unsafe impl Sync for OrinocoEndpoint {}
unsafe impl Send for OrinocoEndpoint {}

pub struct OrinocoEndpoint(pub Arc<Box<dyn Fn(TcpStream)>>);

// pub struct OrcEndpointAnyInterface {}
// #[allow(clippy::redundant_closure_call)]
// impl OrcEndpointAnyInterface {
//     pub fn new(inner: Box<dyn Fn() -> Box<dyn AnyInterface>>) -> OrinocoEndpoint {
//         let launch_fn = move |mut socket: TcpStream| {
//             let inner: Box<dyn AnyInterface> = inner();
//             while let Ok(msg) = OrinocoMsg::recv_on(&mut socket) {
//                 let inner_result: anyhow::Result<()> = (|| {
//                     let msg = rkyv::access::<ArchivedOrcReqAnyInterface, rkyv::rancor::Error>(
//                         &msg.bytes,
//                     )?;
//                     let msg = rkyv::deserialize::<OrcReqAnyInterface, rkyv::rancor::Error>(msg)?;
//                     let rsp_bytes = match msg {
//                         OrcReqAnyInterface::open { path } => rkyv::to_bytes::<rkyv::rancor::Error>(
//                             &OrcRspAnyInterface::open(inner.open(path)),
//                         )?
//                         .to_vec(),
//                         OrcReqAnyInterface::hello { input } => {
//                             rkyv::to_bytes::<rkyv::rancor::Error>(&OrcRspAnyInterface::hello(
//                                 inner.hello(input),
//                             ))?
//                             .to_vec()
//                         }
//                         OrcReqAnyInterface::goodbye { other, input } => todo!(),
//                     };
//                     OrinocoMsg {
//                         len: rsp_bytes.len() as u32,
//                         uuid: Uuid::new_v4().as_u128(),
//                         bytes: rsp_bytes,
//                     }
//                     .send_on(&mut socket)?;
//                     Ok(())
//                 })();
//                 let Ok(_) = inner_result else { panic!() };
//             }
//         };
//         OrinocoEndpoint(Box::new(launch_fn))
//     }
// }

// pub fn orc_serve_any_interface()

// #[derive(Clone)]
// pub struct OrcTcpAnyInterfaceClient {
//     client: std::sync::Arc<OrinocoClient>,
// }
// impl OrcTcpAnyInterfaceClient {
//     pub fn connect(
//         host: impl ToString,
//         port: u16,
//         endpoint: impl ToString,
//     ) -> anyhow::Result<Self> {
//         Ok(OrcTcpAnyInterfaceClient {
//             client: std::sync::Arc::new(OrinocoClient::connect(host, port, endpoint)?),
//         })
//     }
//     pub fn request(&self, msg: OrcReqAnyInterface) -> anyhow::Result<OrcRspAnyInterface> {
//         let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&msg)?.to_vec();
//         let rsp_bytes = self.client.request(bytes)?;
//         let rsp = rkyv::access::<ArchivedOrcRspAnyInterface, rkyv::rancor::Error>(&rsp_bytes)?;
//         let rsp = rkyv::deserialize::<OrcRspAnyInterface, rkyv::rancor::Error>(rsp)?;
//         Ok(rsp)
//     }
// }
// #[derive(rkyv :: Archive, rkyv :: Deserialize, rkyv :: Serialize, Clone)]
// enum OrcReqAnyInterface {
//     open { path: String },
//     hello { input: String },
//     goodbye { other: bool, input: String },
// }
// #[derive(rkyv :: Archive, rkyv :: Deserialize, rkyv :: Serialize, Clone)]
// enum OrcRspAnyInterface {
//     open(()),
//     hello(Result<String, OrinocoError>),
//     goodbye(Result<String, OrinocoError>),
// }

// pub trait AnyRpcTemplate {
//     fn hello(&mut self, input: String) -> anyhow::Result<String>;
//     fn goodbye(&mut self, other: bool, input: String) -> anyhow::Result<String>;
// }
// //
// #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
// enum AnyRpcReq {
//     Hello { input: String },
//     Goodbye { other: bool, input: String },
// }
// #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
// enum AnyRpcRsp {
//     Hello(Result<String, OrinocoError>),
//     Goodbye(Result<String, OrinocoError>),
// }

// #[derive(Clone)]
// pub struct RemoteAnyRpc {
//     tx: Sender<(AnyRpcReq, Sender<AnyRpcRsp>)>,
// }
// impl RemoteAnyRpc {
//     pub fn new<T: AnyRpcTemplate>(init: impl FnOnce() -> T) -> Self {
//         let (tx, rx) = channel();
//         RemoteAnyRpc { tx }
//     }
// }
// impl AnyRpcTemplate for RemoteAnyRpc {
//     fn hello(&mut self, input: String) -> anyhow::Result<String> {
//         let (tx, rx) = channel();
//         self.tx.send((AnyRpcReq::Hello { input }, tx))?;
//         let Ok(AnyRpcRsp::Hello(rsp)) = rx.recv() else {
//             return Err(anyhow::anyhow!(
//                 "Wrong Response Received or Transmitter Hung up"
//             ));
//         };
//         Ok(rsp?)
//     }

//     fn goodbye(&mut self, other: bool, input: String) -> anyhow::Result<String> {
//         todo!()
//     }
// }

// #[derive(Clone)]
// pub struct TcpRemoteAnyRpc {
//     client: Arc<OrinocoClient>,
// }
// impl TcpRemoteAnyRpc {
//     pub fn connect(
//         host: impl ToString,
//         port: u16,
//         endpoint: impl ToString,
//     ) -> anyhow::Result<Self> {
//         Ok(TcpRemoteAnyRpc {
//             client: Arc::new(OrinocoClient::connect(host, port, endpoint)?),
//         })
//     }
//     pub fn request(&self, msg: AnyRpcReq) -> anyhow::Result<AnyRpcRsp> {
//         let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&msg)?.to_vec();
//         let rsp_bytes = self.client.request(bytes)?;
//         let rsp = rkyv::access::<ArchivedAnyRpcRsp, rkyv::rancor::Error>(&rsp_bytes)?;
//         let rsp = rkyv::deserialize::<AnyRpcRsp, rkyv::rancor::Error>(rsp)?;
//         Ok(rsp)
//     }
// }

#[derive(Debug)]
pub struct OrinocoMsg {
    pub len: u32,
    pub uuid: u128,
    pub bytes: Vec<u8>,
}
impl OrinocoMsg {
    pub fn recv_on(rx: &mut dyn Read) -> anyhow::Result<Self> {
        let mut len_rx_buf = [0u8; 4];
        rx.read_exact(&mut len_rx_buf)?;
        let mut uuid_rx_buf = [0u8; 16];
        rx.read_exact(&mut uuid_rx_buf)?;

        let len = u32::from_le_bytes(len_rx_buf);
        if len > u32::MAX / 16 {
            return Err(anyhow::anyhow!(
                "MESSAGE LARGER THAT 256M ATTEMPTED RECV -> PANIC"
            ));
        };
        let uuid = u128::from_le_bytes(uuid_rx_buf);
        let mut msg_rx_buf = vec![0; len as usize];
        if rx.read_exact(&mut msg_rx_buf).is_err() {
            return Err(anyhow::anyhow!("rx error: msg"));
        };
        // let msg = rkyv::access::<ArchivedTransportMsg, rkyv::rancor::Error>(&msg_rx_buf)?;
        // let Ok(msg) = rkyv::deserialize::<TransportMsg, rkyv::rancor::Error>(msg) else {
        //     return Err(anyhow::anyhow!("rx error: deserialize"));
        // };
        Ok(OrinocoMsg {
            len,
            uuid,
            bytes: msg_rx_buf,
        })
    }
    pub fn send_on(&self, tx: &mut dyn Write) -> anyhow::Result<()> {
        // let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;
        let mut tx_buffer = Vec::new(); // TODO: I NEED A LEN PREALLOCATED !!!!
        tx_buffer.write_all(&(self.bytes.len() as u32).to_le_bytes())?;
        tx_buffer.write_all(&(self.uuid).to_le_bytes())?;
        tx_buffer.write_all(&self.bytes)?;
        tx.write_all(&tx_buffer)?;
        tx.flush()?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Pending<TX, RX> {
    tx: Sender<(u128, TX)>,
    pend: Arc<Mutex<HashMap<u128, Sender<RX>>>>,
}
impl<TX, RX> Pending<TX, RX> {
    fn new(tx: Sender<(u128, TX)>) -> Self {
        Pending {
            tx,
            pend: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    fn req(&self, msg: TX) -> anyhow::Result<RX> {
        let (tx, rx) = channel();
        let uuid = uuid::Uuid::new_v4().as_u128();
        {
            let Ok(mut pend) = self.pend.lock() else {
                panic!("failed mutex lock");
            };
            pend.insert(uuid, tx);
        }
        if let Err(e) = self.tx.send((uuid, msg)) {
            panic!("{e}");
        };
        let Ok(rx) = rx.recv() else {
            panic!("RX FAIL");
        };
        Ok(rx)
    }
    fn rsp(&self, uuid: u128, msg: RX) -> anyhow::Result<()> {
        // println!("CLI Recv: {uuid}");
        let tx = {
            let Ok(mut pend) = self.pend.lock() else {
                panic!("failed mutex lock");
            };
            // println!("PENDING: {:#?}", pend);
            pend.remove(&uuid)
        };
        let Some(tx) = tx else {
            panic!();
        };
        let Ok(_) = tx.send(msg) else {
            panic!();
        };
        Ok(())
    }
}

pub struct OrinocoServer {
    endpoints: Arc<Mutex<HashMap<String, OrinocoEndpoint>>>,
    listener: JoinHandle<()>,
}
impl OrinocoServer {
    fn _launch_thread(
        socket: TcpListener,
        endpoints: Arc<Mutex<HashMap<String, OrinocoEndpoint>>>,
    ) -> anyhow::Result<JoinHandle<()>> {
        Ok(thread::spawn(move || {
            for socket in socket.incoming() {
                if let Ok(mut socket) = socket
                    && let Ok(msg) = OrinocoMsg::recv_on(&mut socket)
                    && let Ok(endpoint) = String::from_utf8(msg.bytes)
                    && let Ok(endpoints) = endpoints.lock()
                    && let Some(endpoint_handler) = endpoints.get(&endpoint)
                {
                    let handler = OrinocoEndpoint(endpoint_handler.0.clone());
                    thread::spawn(move || {
                        let handler = handler;
                        handler.0(socket);
                    });
                } else {
                    println!("Some shit happened");
                }
            }
        }))
    }
    pub fn bind(host: impl ToString, port: u16) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(format!("{}:{port}", host.to_string()))?;
        let endpoints = Arc::new(Mutex::new(HashMap::new()));
        let handle = Self::_launch_thread(listener, endpoints.clone())?;
        Ok(OrinocoServer {
            endpoints,
            listener: handle,
        })
    }
    pub fn join(self) -> anyhow::Result<()> {
        let Ok(_) = self.listener.join() else {
            panic!();
        };
        Ok(())
    }
    pub fn serve(&self, endpoint: impl ToString, handle: OrinocoEndpoint) -> anyhow::Result<()> {
        let Ok(mut endpoints) = self.endpoints.lock() else {
            panic!()
        };
        endpoints.insert(endpoint.to_string(), handle);
        Ok(())
    }
}

pub struct OrinocoClient {
    pend: Pending<Vec<u8>, Vec<u8>>,
}
impl OrinocoClient {
    fn _launch_thread(
        outgoing: Receiver<(u128, Vec<u8>)>,
        pending: Pending<Vec<u8>, Vec<u8>>,
        socket: TcpStream,
    ) -> anyhow::Result<()> {
        let (mut tcp_rx, mut tcp_tx) = (socket.try_clone()?, socket);
        // let (pend_rx, pend_tx) = (pending.clone(), pending);
        thread::spawn(move || {
            while let Ok(msg) = OrinocoMsg::recv_on(&mut tcp_rx) {
                pending.rsp(msg.uuid, msg.bytes).unwrap();
            }
        });
        thread::spawn(move || {
            for (uuid, bytes) in outgoing {
                // println!("CLI Dispatch: {uuid}");
                OrinocoMsg {
                    len: bytes.len() as _,
                    uuid,
                    bytes,
                }
                .send_on(&mut tcp_tx)
                .unwrap();
            }
        });
        Ok(())
    }
    pub fn connect(
        host: impl ToString,
        port: u16,
        endpoint: impl ToString,
    ) -> anyhow::Result<Self> {
        let host = host.to_string();
        let (tx, rx) = channel();
        let pend = Pending::new(tx);
        let mut socket = TcpStream::connect(format!("{host}:{port}"))?;
        let endpoint = endpoint.to_string();
        let endpoint = endpoint.as_bytes();
        OrinocoMsg {
            len: endpoint.len() as u32,
            uuid: Uuid::new_v4().as_u128(),
            bytes: endpoint.to_vec(),
        }
        .send_on(&mut socket)?;
        Self::_launch_thread(rx, pend.clone(), socket)?;
        Ok(OrinocoClient { pend })
    }
    pub fn request(&self, msg: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        self.pend.req(msg)
    }
}
pub trait OrinocoSerializedIO<Req, Rsp> {
    fn request(&self, msg: Req) -> Rsp;
}
// impl<
//     S: rkyv::rancor::Fallible,
//     D: rkyv::rancor::Fallible,
//     Req: rkyv::Archive + rkyv::Serialize<S> + rkyv::Deserialize<Self, D>,
//     Rsp: rkyv::Archive + rkyv::Serialize<S> + rkyv::Deserialize<Self, D>,
// > OrinocoSerializedIO<Req, Rsp> for OrinocoClient
// {
//     fn request(&self, msg: Req) -> Rsp {
//         let bytes = rkyv::to_bytes::<rancor::Error>(&msg)?;
//         todo!()
//     }
// }

// impl<T: AnyRpcTemplate> RemoteAnyRpc<T> {
//     fn new() -> RemoteAnyRpc<T> {
//         let (tx, rx) = channel();
//         // RemoteAnyRpc { tx, _phantom:  }
//     }
//     fn _orinoco_init(rx: Receiver<(AnyRpcReq, Sender<AnyRpcRsp>)>) -> Self {
//         // start a fucking thread and GO TO TOWN (wait for messages generated by calls)
//         todo!()
//     }
// }

// enum AnyRpcReq {
//     hello { input: String },
//     goodbye { other: bool, input: String },
// }
// enum AnyRpcRsp {
//     hello(anyhow::Result<String>),
//     goodbye(anyhow::Result<String>),
// }
////
// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;////
////
//     #[test]//
//     fn it_works() {
//         let result = add(2, 2);//
//         assert_eq!(result, 4);
//     }
// }

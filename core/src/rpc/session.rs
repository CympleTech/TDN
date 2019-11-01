use jsonrpc_parse::httpcodec::{HTTPCodec, HTTP};
use jsonrpc_parse::{Error as ErrorResponse, Request as JSONRequest, Response as JSONResponse};
use std::io::Error;
use std::net::SocketAddr;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;

use super::request::Request;
use super::response::Response;
use super::rpc::RPCActor;

use crate::actor::prelude::*;
use crate::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use crate::traits::actor::RPCBridgeActor;

/// request message between session and rpc actor.
#[derive(Clone)]
pub(crate) struct RequestMessage(pub usize, pub Request, pub SocketAddr);

impl Message for RequestMessage {
    type Result = ();
}

/// response message between session and rpc actor.
#[derive(Clone)]
pub(crate) struct ResponseMessage(pub usize, pub Response);

impl Message for ResponseMessage {
    type Result = ();
}

/// when connect close, send close message between rpc and session actor.
#[derive(Clone)]
pub(crate) struct SessionCloseMessage(pub usize);

impl Message for SessionCloseMessage {
    type Result = ();
}

/// message when session open or new connect to, crate a new, and send addr to rpc actor.
#[derive(Clone)]
pub(crate) struct SessionOpenMessage<A: RPCBridgeActor>(pub usize, pub Addr<RPCSessionActor<A>>);

impl<A: RPCBridgeActor> Message for SessionOpenMessage<A> {
    type Result = ();
}

pub(crate) struct RPCSessionActor<A: RPCBridgeActor> {
    id: usize,
    addr: Addr<RPCActor<A>>,
    rpc_method: String,
    rpc_id: String,
    framed: FramedWrite<WriteHalf<TcpStream>, HTTPCodec>,
    socket_addr: SocketAddr,
}

impl<A: RPCBridgeActor> RPCSessionActor<A> {
    pub fn new(
        id: usize,
        addr: Addr<RPCActor<A>>,
        framed: FramedWrite<WriteHalf<TcpStream>, HTTPCodec>,
        socket_addr: SocketAddr,
    ) -> Self {
        let rpc_method = "".into();
        let rpc_id = "".into();
        Self {
            id,
            addr,
            rpc_method,
            rpc_id,
            framed,
            socket_addr,
        }
    }

    /// try send received request to rpc actor
    fn send_rpc<M: 'static>(&self, message: M)
    where
        RPCActor<A>: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <RPCActor<A> as Actor>::Context: ToEnvelope<RPCActor<A>, M>,
    {
        let _ = try_resend_times(self.addr.clone(), message, DEFAULT_TIMES).map_err(|_| {
            println!("Send request to rpc fail");
        });
    }
}

impl<A: RPCBridgeActor> Actor for RPCSessionActor<A> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_rpc(SessionOpenMessage::<A>(self.id, ctx.address()));
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.send_rpc(SessionCloseMessage(self.id));

        Running::Stop
    }
}

impl<A: RPCBridgeActor> WriteHandler<Error> for RPCSessionActor<A> {}

impl<A: RPCBridgeActor> StreamHandler<HTTP, Error> for RPCSessionActor<A> {
    fn handle(&mut self, msg: HTTP, _ctx: &mut Self::Context) {
        match msg {
            HTTP::Request(req) => {
                let operator = Request::parse(req.method(), req.params());
                match operator {
                    Request::Invalid => {
                        // invalid params
                        let response =
                            ErrorResponse::MethodNotFound(req.method().clone(), req.id().clone());
                        self.framed.write(HTTP::Error(response));
                        self.framed.close();
                    }
                    operator => {
                        self.rpc_method = req.method().clone();
                        self.rpc_id = req.id().clone();

                        self.send_rpc(RequestMessage(self.id, operator, self.socket_addr.clone()));
                    }
                }
            }
            HTTP::Response(resp) => {
                let operator = Response::parse(resp.method(), resp.result());
                match operator {
                    Response::Invalid => {}
                    _ => self.send_rpc(ResponseMessage(self.id, operator)),
                }
                self.framed.close();
            }
            _ => {
                self.framed.write(msg);
                self.framed.close();
            }
        }
    }
}

impl<A: RPCBridgeActor> Handler<RequestMessage> for RPCSessionActor<A> {
    type Result = ();

    fn handle(&mut self, msg: RequestMessage, _ctx: &mut Self::Context) {
        let (method, params) = msg.1.deparse();
        let path_option = params.get("request_path").and_then(|e| e.as_str());
        let path: String = if path_option.is_some() {
            // TODO remove request_path
            path_option.unwrap().into()
        } else {
            "/".into()
        };
        let host_option = params.get("request_host").and_then(|e| e.as_str());
        let host: String = if host_option.is_some() {
            // TODO remove request_host
            host_option.unwrap().into()
        } else {
            "/".into()
        };

        let request = JSONRequest::new(method, "0".into(), params, path, host);
        self.framed.write(HTTP::Request(request));
    }
}

impl<A: RPCBridgeActor> Handler<ResponseMessage> for RPCSessionActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ResponseMessage, _ctx: &mut Self::Context) {
        // msg packaging to response
        match msg.1 {
            Response::Invalid => {
                self.framed.write(HTTP::Error(ErrorResponse::InvalidRequest(
                    self.rpc_method.clone(),
                    self.rpc_id.clone(),
                )));
            }
            _ => {
                let (_, params) = msg.1.deparse();
                self.framed.write(HTTP::Response(JSONResponse::new(
                    self.rpc_method.clone(),
                    self.rpc_id.clone(),
                    params,
                )));
            }
        };

        self.framed.close();
    }
}

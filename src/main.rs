#[macro_use]
extern crate log;

use crate::dice::get_results;
use crate::messages::TextMessageDTO;
use std::time::{Duration, Instant};

use actix::*;
use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use messages::OutgoingMessageDTO;
use room::LeaveRoomMessage;

mod dice;
mod messages;
mod room;
mod server;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Message)]
#[rtype(result = "String")]
struct GetNameMsg;

/// Entry point for our websocket route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsChatSession {
            id: 0,
            hb: Instant::now(),
            name: None,
            server_addr: srv.get_ref().clone(),
            room_addr: None,
        },
        &req,
        stream,
    )
}

pub struct WsChatSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// peer name
    name: Option<String>,
    /// Chat server
    server_addr: Addr<server::ChatServer>,
    /// Current chat room
    room_addr: Option<Addr<room::ChatRoom>>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.server_addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.server_addr.do_send(server::Disconnect { id: self.id });
        if let Some(room) = self.room_addr.as_ref() {
            room.do_send(LeaveRoomMessage {
                name: self
                    .name
                    .as_ref()
                    .expect("Name must be provided here")
                    .to_owned(),
                id: self.id,
            });
        }
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<room::RoomMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: room::RoomMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0.to_json());
    }
}

impl Handler<GetNameMsg> for WsChatSession {
    type Result = String;

    fn handle(&mut self, _: GetNameMsg, _: &mut Self::Context) -> Self::Result {
        self.name.as_ref().expect("The name must be set if the user joined a room").clone()
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                debug!("Msg from >{:?}: '{}'", self.name, text);

                if self.name.is_none() && !text.starts_with("/name") {
                    ctx.text(
                        system_message(
                            "You need so set a name before doing anything else (i.e. /name ABC)",
                        )
                        .to_json(),
                    );
                    return;
                }

                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            self.server_addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => error!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                if let Some(room_addr) = self.room_addr.as_ref() {
                                    room_addr.do_send(LeaveRoomMessage {
                                        name: self
                                            .name
                                            .as_ref()
                                            .expect("Name must be provided here")
                                            .to_owned(),
                                        id: self.id,
                                    });
                                }

                                let room_name = v[1].to_owned();

                                self.server_addr
                                    .send(server::RequestRoom {
                                        name: room_name.clone(),
                                    })
                                    .into_actor(self)
                                    .then(move |res, this, ctx| {
                                        match res {
                                            Ok(room_addr) => {
                                                this.room_addr = Some(room_addr.clone());

                                                room_addr.do_send(room::JoinRoomMessage {
                                                    id: this.id,
                                                    name: this.name.as_ref().unwrap().to_owned(),
                                                    session_addr: ctx.address().into(),
                                                });

                                                ctx.text(
                                                    OutgoingMessageDTO::TextMessage(
                                                        TextMessageDTO::system(&format!(
                                                            "You joined room {}",
                                                            room_name
                                                        )),
                                                    )
                                                    .to_json(),
                                                );
                                            }
                                            _ => error!("Something is wrong"),
                                        }

                                        fut::ready(())
                                    })
                                    .wait(ctx)
                            } else {
                                ctx.text(
                                    OutgoingMessageDTO::TextMessage(TextMessageDTO::system(
                                        "!!! room name is required",
                                    ))
                                    .to_json(),
                                );
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_owned());
                                ctx.text(
                                    system_message(&format!(
                                        "You are now known as: {}",
                                        self.name.as_ref().unwrap()
                                    ))
                                    .to_json(),
                                );
                                if let Some(room_address) = self.room_addr.as_ref() {
                                    room_address.do_send(room::NameChangedMessage);
                                }
                            } else {
                                ctx.text(system_message("!!! name is required").to_json());
                            }
                        }
                        _ => ctx.text(
                            system_message(&format!("!!! unknown command: {:?}", m)).to_json(),
                        ),
                    }
                } else {
                    if let Some(room_address) = self.room_addr.as_ref() {
                        let sender = self.name.as_ref().unwrap();

                        let msg = if m.starts_with('!') {
                            TextMessageDTO::dice_result(m, &get_results(&m[1..]), &sender)
                        } else {
                            TextMessageDTO::chat(m, &sender)
                        };

                        room_address.do_send(room::ClientMessage {
                            id: self.id,
                            msg: msg,
                        });
                    } else {
                        ctx.text(
                            system_message(
                                "You have to join a room before sending messages (i.e. /join Main)",
                            )
                            .to_json(),
                        )
                    }
                }
            }
            ws::Message::Binary(_) => error!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl WsChatSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.server_addr.do_send(server::Disconnect { id: act.id });

                if let Some(room) = act.room_addr.as_ref() {
                    room.do_send(LeaveRoomMessage {
                        name: act
                            .name
                            .clone()
                            .unwrap_or_else(|| "<<unknown>>".to_string())
                            .to_owned(),
                        id: act.id,
                    });
                }

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

fn system_message(test: &str) -> OutgoingMessageDTO {
    OutgoingMessageDTO::TextMessage(TextMessageDTO::system(test))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Start chat server actor
    let server = server::ChatServer::new().start();

    // Create Http server with websocket support
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            // redirect to websocket.html
            .service(web::resource("/").route(web::get().to(|| {
                HttpResponse::Found()
                    .header("LOCATION", "/static/application.html")
                    .finish()
            })))
            // websocket
            .service(web::resource("/ws/").to(chat_route))
            // static resources
            .service(fs::Files::new("/static/", "static/"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

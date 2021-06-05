use std::collections::HashMap;

use crate::messages::{OutgoingMessageDTO, TextMessageDTO};
use actix::prelude::*;

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: TextMessageDTO,
}

/// ChatRoom sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomMessage(pub OutgoingMessageDTO);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinRoomMessage {
    pub id: usize,
    pub name: String,
    pub session_addr: Recipient<RoomMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveRoomMessage {
    pub name: String,
    pub id: usize,
}

pub struct ChatRoom {
    name: String,
    members: HashMap<usize, Recipient<RoomMessage>>,
}

impl ChatRoom {
    pub fn new(name: String) -> Self {
        ChatRoom {
            name,
            members: HashMap::new(),
        }
    }

    fn send_to_all(&self, message: &TextMessageDTO) {
        self.members.values().for_each(|session| {
            debug!("sending to session {:?}", session);
            let _ = session.do_send(RoomMessage(OutgoingMessageDTO::TextMessage(
                message.clone(),
            )));
        });
    }
}

impl Actor for ChatRoom {
    type Context = Context<Self>;
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        debug!("got message {:?}", msg.msg);
        self.send_to_all(&msg.msg);
    }
}

/// Handler for Message message.
impl Handler<JoinRoomMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: JoinRoomMessage, _: &mut Context<Self>) {
        self.send_to_all(&TextMessageDTO::system(&format!(
            "'{}' joined the room",
            msg.name
        )));
        self.members.insert(msg.id, msg.session_addr);
    }
}

/// Handler for Message message.
impl Handler<LeaveRoomMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoomMessage, _: &mut Context<Self>) {
        self.members.remove(&msg.id);
        self.send_to_all(&TextMessageDTO::system(&format!(
            "'{}' left the room",
            msg.name
        )));
    }
}
use std::collections::HashMap;

use crate::{messages::OutgoingMessageDTO};
use actix::prelude::*;

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: OutgoingMessageDTO,
}

/// ChatRoom sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub OutgoingMessageDTO);

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinRoomMessage {
    id: usize,
    name: String,
    session_addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveRoomMessage {
    pub name: String,
    pub id: usize,
}

pub struct ChatRoom {
    name: String,
    members: HashMap<usize, Recipient<Message>>,
}

impl ChatRoom {
    pub fn new(name: String) -> Self {
        ChatRoom {
            name,
            members: HashMap::new(),
        }
    }

    fn send_to_all(&self, message: &OutgoingMessageDTO) {
        self.members.values().for_each(|session| {
            session.do_send(Message(message.clone())).expect("sent a message to a chat session");
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
        self.send_to_all(&msg.msg);
    }
}

/// Handler for Message message.
impl Handler<JoinRoomMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: JoinRoomMessage, _: &mut Context<Self>) {
        self.send_to_all(&OutgoingMessageDTO::system(&format!("'{}' joined the room", msg.name)));
        self.members.insert(msg.id, msg.session_addr);
    }
}

/// Handler for Message message.
impl Handler<LeaveRoomMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoomMessage, _: &mut Context<Self>) {
        self.send_to_all(&OutgoingMessageDTO::system(&format!("'{}' left the room", msg.name)));
        self.members.remove(&msg.id);
    }
}
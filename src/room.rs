use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::messages::{OutgoingMessageDTO, RoomStateMessageDTO, TextMessageDTO};
use crate::{GetNameMsg, WsChatSession};
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
    pub session_addr: Addr<WsChatSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveRoomMessage {
    pub name: String,
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NameChangedMessage;

pub struct ChatRoom {
    name: String,
    members: HashMap<usize, Addr<WsChatSession>>,
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
            let _ = session.do_send(RoomMessage(OutgoingMessageDTO::TextMessage(
                message.clone(),
            )));
        });
    }

    fn send_room_state(&self, ctx: &mut Context<Self>) {
        let needed_answers = self.members.values().len();
        let answers = Arc::new(Mutex::new(vec![]));
        self.members.values().for_each(|member| {
            let answers_local = answers.clone();
            let needed_answers_local = needed_answers;
            member
                .send(GetNameMsg)
                .into_actor(self)
                .then(move |res, act, _| {
                    let name = match res {
                        Ok(name) => name,
                        // something went wrong, likely a deadlock between the actors
                        _ => "<<unknown name>>".to_string(),
                    };
                    ChatRoom::maybe_send_member_list(
                        act,
                        needed_answers_local,
                        answers_local,
                        name,
                    );
                    fut::ready(())
                })
                .wait(ctx)
        });
    }

    fn maybe_send_member_list(
        actor: &mut ChatRoom,
        needed_answers: usize,
        answers: Arc<Mutex<Vec<String>>>,
        added_name: String,
    ) {
        let mut list = answers.lock().unwrap();
        list.push(added_name);

        if list.len() == needed_answers {
            actor.members.values().for_each(|session| {
                let _ = session.do_send(RoomMessage(OutgoingMessageDTO::RoomState(
                    RoomStateMessageDTO {
                        room_name: actor.name.clone(),
                        members: list.clone(),
                    },
                )));
            });
        }
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

impl Handler<JoinRoomMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: JoinRoomMessage, ctx: &mut Context<Self>) {
        self.send_to_all(&TextMessageDTO::system(&format!(
            "'{}' joined the room",
            msg.name
        )));
        self.members.insert(msg.id, msg.session_addr);
        self.send_room_state(ctx);
    }
}

impl Handler<NameChangedMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, _: NameChangedMessage, ctx: &mut Context<Self>) {
        self.send_room_state(ctx);
    }
}

impl Handler<LeaveRoomMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoomMessage, ctx: &mut Context<Self>) {
        self.members.remove(&msg.id);
        self.send_to_all(&TextMessageDTO::system(&format!(
            "'{}' left the room",
            msg.name
        )));
        self.send_room_state(ctx);
    }
}

#![feature(try_from)]

use chrono::Local;
use futures::future::{self, Future};
use ruma_client::{
    api::r0::{
        membership::join_room_by_id_or_alias,
        room::create_room::{self, RoomPreset, Visibility},
        send::send_message_event,
    },
    Client,
};
use ruma_events::{
    room::message::{MessageEventContent, MessageType, TextMessageEventContent},
    EventType,
};
use tokio::runtime::current_thread;

use std::{
    convert::TryInto,
    io::{self, Write},
};

fn main() {
    // Get the username
    let mut user = String::new();
    print!("User: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut user)
        .expect("Could not prompt username");
    user = user.replace('\n', "");

    // Securely get the password sudo-style (with echo off)
    let pass = rpassword::prompt_password_stdout("Password: ").unwrap();

    let mut client = Client::new_https("https://matrix.org".parse().unwrap())
        .expect("Could not connect to Matrix");

    // Password is moved into and dropped right after log_in()
    client.log_in(user, pass, None).expect("Could not log in");

    println!("The registered client: {:#?}", client);

    // Try creating a room
    let fut = create_room::call(
        &client,
        create_room::Request {
            creation_content: None,
            invite: vec![client.session.clone().unwrap().user_id.clone()],
            name: None,
            preset: Some(RoomPreset::PublicChat),
            room_alias_name: Some("ruma-client-test-room".to_owned()),
            topic: Some("drozdziak1's ruma-client test room".to_owned()),
            visibility: Some(Visibility::Public),
        },
    )
    .map(|res| res.room_id)
    // Try joining if there's a problem creating
    .or_else(|err| {
        eprintln!(
            "Could not create #ruma-client-test-room: {:?}, joining...",
            err
        );
        join_room_by_id_or_alias::call(
            &client,
            join_room_by_id_or_alias::Request {
                room_id_or_alias: "#ruma-client-test-room:matrix.org".try_into().unwrap(),
                third_party_signed: None,
            },
        )
        .map(|res| res.room_id)
    })
    // Both Response structs are identical but Rust knows they're technically different types; both room_id's get
    // peeled in the map()'s to fix that.
    .and_then(|room_id| {
        send_message_event::call(
            &client,
            send_message_event::Request {
                room_id: room_id,
                event_type: EventType::RoomMessage,
                txn_id: "1".to_owned(),
                data: MessageEventContent::Text(TextMessageEventContent {
                    body: format!(
                        "Hello from the test bot! The local time is {}",
                        Local::now()
                    ),
                    msgtype: MessageType::Text,
                }),
            },
        )
    });

    println!(
        "Top-level result: {:?}",
        current_thread::block_on_all(fut).unwrap()
    );
}

extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate futures;
extern crate resolve;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_core;

mod cmd_line_args;

use byteorder::{BigEndian, ByteOrder};
use bytes::Bytes;
use chrono::prelude::*;
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use tokio::net::TcpListener;
use tokio::prelude::*;

/* Very simple binary protocol is used (it has already some support in Rust):
 *  4 bytes: message length
 *  1byte: message type: 'M' -- message, '!' -- acknowledgment
 *
 * Better protocol: Often, one byte is enough for both message type and message length:
 *   1bit: message type
 *   7 bits: 0 .. 123 -- message length
 *           124 -- message length is stored in a next byte
 *           125 -- message length is stored in next two bytes
 *           126 -- message length is stored in next three bytes
 *           127 -- message length is stored in next four bytes
 */

/*
 * RTT:
 *  - https://stackoverflow.com/questions/26593387/how-can-i-get-the-current-time-in-milliseconds
 *  - https://stackoverflow.com/questions/29445026/converting-number-primitives-i32-f64-etc-to-byte-representations
 */
#[derive(Debug)]
enum MessageType {
    Message,
    Acknowledgment,
}

#[derive(Debug)]
struct Message {
    t: MessageType,
    time_beg: i64,
    content: Bytes,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let now = Utc::now().timestamp_nanos();
        let diff = (now - self.time_beg) as f64 / 1_000_000f64;

        // FIXME: drf: Do we allocate additional memory in to_vec()?
        match String::from_utf8(self.content.to_vec()) {
            Ok(s) => write!(
                f,
                "Text-message: type: {:?}; time-diff: {:.3}millis; content: {}",
                self.t, diff, s
            ),
            Err(_) => write!(
                f,
                "Binary-message: type: {:?}; time-diff: {:.3}millis; content: {:?}",
                self.t, diff, self.content
            ),
        }
    }
}

#[derive(Debug)]
struct MessageError {}
impl Error for MessageError {}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot decode message")
    }
}

fn encode_message(msg: Message) -> Bytes {
    // FIXME: drf: Do we use the minimal number of allocations?
    let mut t_arr = [0u8; 1 + 8];
    t_arr[0] = match msg.t {
        MessageType::Acknowledgment => b'!',
        MessageType::Message => b'M',
    };
    let time_beg = if msg.time_beg > 0 {
        msg.time_beg
    } else {
        Utc::now().timestamp_nanos()
    };
    BigEndian::write_i64(&mut t_arr[1..9], time_beg);

    let mut ret = Bytes::with_capacity(1 + 8 + msg.content.len());
    ret.extend_from_slice(&t_arr);
    ret.extend_from_slice(&msg.content);
    ret
}

fn decode_message(b: Bytes) -> Result<Message, MessageError> {
    if b.len() > 1 + 8 {
        let msg_type = match b[0] {
            b'M' => MessageType::Message,
            b'!' => MessageType::Acknowledgment,
            _ => return Err(MessageError {}),
        };

        let time_beg = BigEndian::read_i64(&b[1..9]);
        let content = b.slice_from(9);

        Ok(Message {
            t: msg_type,
            time_beg: time_beg,
            content: content,
        })
    } else {
        Err(MessageError {})
    }
}

fn to_stdio_err<E: std::error::Error>(e: &E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e))
}

// FIXME: drf: It seems that send sometimes aborts.
fn main() {
    let receiver = std::sync::Arc::new(std::sync::Mutex::new(
        None::<futures::sync::mpsc::UnboundedSender<Bytes>>,
    ));
    let recv_srv = std::sync::Arc::clone(&receiver);
    let recv_close = std::sync::Arc::clone(&receiver);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lock = stdin.lock();
        futures::stream::iter_result(stdin_lock.lines())
            .for_each(|line| {
                let recv_sink_opt = receiver.lock().unwrap();
                match recv_sink_opt.as_ref() {
                    None => {
                        eprintln!("Message will be skipped :-(");
                        Ok(())
                    }
                    Some(recv_sink) => {
                        recv_sink
                            .clone()
                            .send(encode_message(Message {
                                t: MessageType::Message,
                                time_beg: 0,
                                content: Bytes::from(line.into_bytes()),
                            }))
                            .map(|_| ())
                            .map_err(|err| to_stdio_err(&err))
                            .wait()
                            .unwrap();
                        Ok(())
                    }
                }
            })
            .wait()
            .unwrap();
    });

    let handle_sock_fn = move |sock| {
        let mut tx_opt = recv_srv.lock().unwrap();
        match *tx_opt {
            Some(_) => {
                eprintln!("Another client already connected!");
            }
            None => {
                let framed_sock = tokio_codec::Framed::new(
                    sock,
                    tokio::codec::length_delimited::LengthDelimitedCodec::new(),
                );
                let (tx, rx) = framed_sock.split();
                let (ch_tx, ch_rx) = futures::sync::mpsc::unbounded::<Bytes>();

                *tx_opt = Some(ch_tx.clone());

                tokio::spawn(
                    tx.send_all(ch_rx.map_err(|_| { // Error == ()
                        std::io::ErrorKind::Other
                    })).then(|_| Ok(())),
                );

                let mtx = recv_close.clone();
                tokio::spawn(
                    rx.for_each(move |item_mut| {
                        let item = item_mut.freeze();
                        match decode_message(item) {
                            Err(e) => {
                                eprintln!("Cannot decode incoming message: {}", e);
                                futures::future::Either::B(futures::future::ok(()))
                            }
                            Ok(msg) => {
                                println!("Incoming msg: {}", msg);

                                match msg.t {
                                    MessageType::Acknowledgment => {
                                        futures::future::Either::B(futures::future::ok(()))
                                    }
                                    MessageType::Message => {
                                        // FIXME: drf: How to convert String to Bytes with the minimum number of allocations?
                                        let content = Bytes::from(
                                            format!("Message of length: {}", msg.content.len())
                                                .into_bytes(),
                                        );
                                        futures::future::Either::A(
                                            ch_tx
                                                .clone()
                                                .send(encode_message(Message {
                                                    t: MessageType::Acknowledgment,
                                                    time_beg: msg.time_beg,
                                                    content: content,
                                                }))
                                                .map(|_| ())
                                                .map_err(|err| to_stdio_err(&err)),
                                        )
                                    }
                                }
                            }
                        }
                    }).map(move |res| {
                            let mut tx_guard = mtx.lock().unwrap();
                            *tx_guard = None;
                            res
                        })
                        .map_err(|err| eprintln!("IO error {:?}", err)),
                );
            }
        };
        Ok(())
    };

    let args = match cmd_line_args::parse() {
        Err(e) => panic!("Cannot parse cmd-line: {:?}", e),
        Ok(a) => a,
    };

    // FIXME: drf: Try to connect/bind to more addresses.
    let addr = args.addrs[0];
    println!("Reolved address: {:?}", addr);

    // FIXME: drf: How to connect/bind to multiple addresses.
    if args.role == cmd_line_args::Role::Server {
        let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

        let server = listener
            .incoming()
            .map_err(|e| eprintln!("accept failed = {:?}", e))
            .for_each(move |sock| handle_sock_fn(sock));

        // Start the Tokio runtime
        tokio::run(server);
    } else {
        let connection = tokio::net::TcpStream::connect(&addr);
        let client = connection
            .map_err(|e| eprintln!("accept failed = {:?}", e))
            .and_then(move |tcp_stream| handle_sock_fn(tcp_stream));

        // Start the Tokio runtime
        tokio::run(client);
    };
}

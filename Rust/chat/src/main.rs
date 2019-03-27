extern crate futures;
extern crate tokio;
extern crate tokio_codec;

use std::io::BufRead;
use tokio::prelude::*;
use tokio::net::TcpListener;

#[derive(Debug)]
enum Error {
    Stdin(std::io::Error),
    Channel(futures::sync::mpsc::SendError<String>),
}

fn spawn_stdin_stream_unbounded() -> futures::sync::mpsc::UnboundedReceiver<String> {
    let (channel_sink, channel_stream) = futures::sync::mpsc::unbounded::<String>();
    let stdin_sink = channel_sink.sink_map_err(Error::Channel);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let stdin_lock = stdin.lock();
        futures::stream::iter(stdin_lock.lines())
            .map_err(Error::Stdin)
            .forward(stdin_sink)
            .wait()
            .unwrap();
    });

    channel_stream
}

fn main() {
    // Bind the server's socket.
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr)
        .expect("unable to bind TCP listener");

    // Pull out a stream of sockets for incoming connections
    let server = listener.incoming()
        .map_err(|e| eprintln!("accept failed = {:?}", e))
        .for_each(|sock| {
            let framed_sock = tokio_codec::Framed::new(sock, tokio_codec::LinesCodec::new());
            let (tx, rx) = framed_sock.split();
            let (ch_tx, ch_rx) = futures::sync::mpsc::unbounded::<String>();

            tokio::spawn(
                tx.send_all(
                    ch_rx.map_err(|err|{ std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)) })
                ).then(
                    |_| Err(())
                )
            );

            tokio::spawn(
                ch_tx.clone().sink_map_err(|_|())
                .send_all(
                    spawn_stdin_stream_unbounded().map_err(|_|())//.map_err(|err|{ std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)) })
                ).map_err(|_| ())
                .map(|_| ())
            );

            tokio::spawn(
                rx.for_each(move |item| {
                    println!("Received message of length: {}", item.len());
                    ch_tx.clone().send(item).map(
                        |_| ()
                   ).map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err))
                   })
                }).map_err(|err| {
                  eprintln!("IO error {:?}", err)
                })
            )

            //tokio::spawn(tx.send_all(ch_rx.map_err(|err|{ std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err)) })))
            //tokio::spawn(ch_rx.forward(tx).map_err(|err|{ std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err))}));
            // Split up the reading and writing parts of the
            // socket.
            /*
            let (reader, writer) = sock.split();
            let r = std::io::BufReader::new(reader);
            let w = std::io::BufWriter::new(writer);
            // A future that echos the data and returns how
            // many bytes were copied...
            //let bytes_copied = copy(reader, writer);
*/
            /*
            let handle_read = tokio::io::read_until(r, b'\n', vec![]).and_then(|(_, buf)| {
                println!("Read {:} bytes", buf.len());
                tokio::io::write_all(w, format!("Received message of length: {}", buf.len()));
                Ok(())
            }).map_err(|err| {
                eprintln!("IO error {:?}", err)
            });
            */
            
            /*
            // ... after which we'll print what happened.
            let handle_conn = bytes_copied.map(|amt| {
                println!("wrote {:} bytes", amt.0)
            }).map_err(|err| {
                eprintln!("IO error {:?}", err)
            });
            */

            // Spawn the future as a concurrent task.
            //tokio::spawn(handle_read)
        });

    // Start the Tokio runtime
    tokio::run(server);
}
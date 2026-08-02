use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::api_key_tools::test_openai_api_key_connectivity;
use crate::provider_endpoint::ProviderEndpointScope;

#[test]
fn credential_bearing_requests_do_not_follow_same_origin_redirects() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    let address = listener
        .local_addr()
        .expect("test listener should expose its address");
    let (followed_tx, followed_rx) = mpsc::channel();

    let server = thread::spawn(move || {
        let (mut first_stream, _) = listener
            .accept()
            .expect("initial credential-bearing request should arrive");
        let mut buffer = [0_u8; 2048];
        let _ = first_stream.read(&mut buffer);
        write!(
            first_stream,
            "HTTP/1.1 302 Found\r\nLocation: http://{address}/v1/redirected-models\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .expect("redirect response should write");
        drop(first_stream);

        listener
            .set_nonblocking(true)
            .expect("listener should become nonblocking");
        let deadline = Instant::now() + Duration::from_millis(750);
        while Instant::now() < deadline {
            match listener.accept() {
                Ok((_stream, _)) => {
                    let _ = followed_tx.send(true);
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    let _ = followed_tx.send(false);
                    return;
                }
            }
        }
        let _ = followed_tx.send(false);
    });

    let scope = ProviderEndpointScope::parse(&format!("http://{address}/v1"))
        .expect("loopback endpoint should be valid");
    let error = test_openai_api_key_connectivity(&scope, "secret", None, None, 2_000)
        .expect_err("same-origin redirect must be rejected");

    assert!(error.contains("Redirects are not allowed"));
    assert!(!followed_rx
        .recv_timeout(Duration::from_millis(900))
        .expect("server should report redirect-follow status"));
    server.join().expect("test server should exit cleanly");
}

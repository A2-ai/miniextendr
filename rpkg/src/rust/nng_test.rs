// NNG integration tests — verify NNG symbols link and the safe wrappers work.

use miniextendr_api::prelude::*;

#[cfg(feature = "nng")]
use miniextendr_api::nng::*;

/// Return NNG library version string.
#[cfg(feature = "nng")]
#[miniextendr]
pub fn nng_version_string() -> String {
    unsafe {
        let ptr = miniextendr_api::nng::ffi::nng_version();
        core::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned()
    }
}

/// Test PAIR socket echo over inproc transport using message API.
#[cfg(feature = "nng")]
#[miniextendr]
pub fn nng_pair_echo_test(message: String) -> Result<String, String> {
    let url = "inproc://miniextendr-pair-echo";

    let sock_a = NngSocket::pair().map_err(|e| format!("pair open A: {}", e))?;
    sock_a.listen(url).map_err(|e| format!("listen: {}", e))?;

    let sock_b = NngSocket::pair().map_err(|e| format!("pair open B: {}", e))?;
    sock_b.dial(url).map_err(|e| format!("dial: {}", e))?;

    // Small delay to ensure inproc connection is established
    std::thread::sleep(std::time::Duration::from_millis(10));

    let msg = NngMsg::from_bytes(message.as_bytes()).map_err(|e| format!("msg alloc: {}", e))?;
    sock_a.send_msg(msg).map_err(|e| format!("send_msg A->B: {}", e))?;

    let received = sock_b.recv_msg().map_err(|e| format!("recv_msg B: {}", e))?;
    let text = String::from_utf8_lossy(received.body()).into_owned();

    let echo_msg = NngMsg::from_bytes(format!("echo: {}", text).as_bytes())
        .map_err(|e| format!("echo msg alloc: {}", e))?;
    sock_b.send_msg(echo_msg).map_err(|e| format!("send_msg B->A: {}", e))?;

    let reply = sock_a.recv_msg().map_err(|e| format!("recv_msg A: {}", e))?;
    Ok(String::from_utf8_lossy(reply.body()).into_owned())
}

/// Test req/rep pattern over inproc transport.
#[cfg(feature = "nng")]
#[miniextendr]
pub fn nng_reqrep_test(message: String) -> Result<String, String> {
    let url = "inproc://miniextendr-reqrep";

    let rep = NngSocket::rep().map_err(|e| e.to_string())?;
    rep.listen(url).map_err(|e| e.to_string())?;

    let req = NngSocket::req().map_err(|e| e.to_string())?;
    req.dial(url).map_err(|e| e.to_string())?;

    // REQ sends, REP receives then replies
    let msg = NngMsg::from_bytes(message.as_bytes()).map_err(|e| e.to_string())?;
    req.send_msg(msg).map_err(|e| e.to_string())?;

    let received = rep.recv_msg().map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(received.body()).into_owned();

    let reply_msg =
        NngMsg::from_bytes(format!("reply: {}", text).as_bytes()).map_err(|e| e.to_string())?;
    rep.send_msg(reply_msg).map_err(|e| e.to_string())?;

    let reply = req.recv_msg().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(reply.body()).into_owned())
}

/// Test NNG message API.
#[cfg(feature = "nng")]
#[miniextendr]
pub fn nng_msg_test() -> Result<String, String> {
    let msg = NngMsg::from_bytes(b"hello world").map_err(|e| e.to_string())?;
    let len = msg.len();
    let body = String::from_utf8_lossy(msg.body()).into_owned();
    Ok(format!("len={}, body={}", len, body))
}

/// Test push/pull pipeline over inproc.
/// NOTE: Disabled because synchronous push/pull on inproc may hang
/// if the connection isn't fully established before send. Needs NngAio (Plan 1 Phase B).
#[cfg(all(feature = "nng", feature = "DISABLED"))]
#[miniextendr]
pub fn nng_pushpull_test() -> Result<String, String> {
    let url = "inproc://miniextendr-pushpull";

    let pull = NngSocket::pull().map_err(|e| format!("pull open: {}", e))?;
    pull.listen(url).map_err(|e| format!("pull listen: {}", e))?;

    let push = NngSocket::push().map_err(|e| format!("push open: {}", e))?;
    push.dial(url).map_err(|e| format!("push dial: {}", e))?;

    // Small delay for inproc connection to establish
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Set recv timeout (1 second) to avoid infinite hang
    pull.set_recv_timeout(1000).map_err(|e| format!("set timeout: {}", e))?;

    // Push 3 messages
    for i in 0..3 {
        let msg = NngMsg::from_bytes(format!("msg-{}", i).as_bytes())
            .map_err(|e| format!("push msg alloc: {}", e))?;
        push.send_msg(msg).map_err(|e| format!("push send: {}", e))?;
    }

    // Pull and concatenate
    let mut results = Vec::new();
    for i in 0..3 {
        let msg = pull.recv_msg().map_err(|e| format!("pull recv {}: {}", i, e))?;
        results.push(String::from_utf8_lossy(msg.body()).into_owned());
    }

    Ok(results.join(", "))
}

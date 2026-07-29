//! Tiny management client used only by the local development launcher.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const MANAGEMENT_TIMEOUT: Duration = Duration::from_millis(250);

pub(crate) fn status(address: SocketAddr, path: &str) -> std::io::Result<u16> {
    request(address, "GET", path)
}

pub(crate) fn drain(address: SocketAddr) -> std::io::Result<u16> {
    request(address, "POST", "/drain")
}

fn request(address: SocketAddr, method: &str, path: &str) -> std::io::Result<u16> {
    let mut stream = TcpStream::connect_timeout(&address, MANAGEMENT_TIMEOUT)?;
    stream.set_read_timeout(Some(MANAGEMENT_TIMEOUT))?;
    stream.set_write_timeout(Some(MANAGEMENT_TIMEOUT))?;
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: ferrite-dev\r\nConnection: close\r\n\r\n"
    )?;
    stream.flush()?;
    let mut response = [0_u8; 64];
    let count = stream.read(&mut response)?;
    let line = std::str::from_utf8(&response[..count])
        .ok()
        .and_then(|response| response.split_once("\r\n").map(|pair| pair.0))
        .ok_or_else(|| std::io::Error::other("management response has no status line"))?;
    let status = line
        .split(' ')
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| std::io::Error::other("management response has invalid status"))?;
    Ok(status)
}

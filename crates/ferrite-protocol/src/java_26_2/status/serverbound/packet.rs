#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusServerboundPacket {
    Request,
    Ping(i64),
}

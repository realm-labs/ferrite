#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginServerboundPacket {
    Hello(LoginHello),
    Acknowledged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginHello {
    pub name: String,
    pub supplied_profile_id: u128,
}

//! Ordered login admission gates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionRequest {
    pub user_banned: bool,
    pub whitelist_enabled: bool,
    pub whitelisted: bool,
    pub ip_banned: bool,
    pub current_players: usize,
    pub capacity: usize,
    pub bypass_capacity: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionGate {
    UserBan,
    Whitelist,
    IpBan,
    Capacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionRejection {
    UserBanned,
    NotWhitelisted,
    IpBanned,
    ServerFull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionDecision {
    pub checked: Vec<AdmissionGate>,
    pub rejection: Option<AdmissionRejection>,
}

#[must_use]
pub fn admit(request: AdmissionRequest) -> AdmissionDecision {
    let mut checked = Vec::with_capacity(4);
    checked.push(AdmissionGate::UserBan);
    if request.user_banned {
        return rejected(checked, AdmissionRejection::UserBanned);
    }
    checked.push(AdmissionGate::Whitelist);
    if request.whitelist_enabled && !request.whitelisted {
        return rejected(checked, AdmissionRejection::NotWhitelisted);
    }
    checked.push(AdmissionGate::IpBan);
    if request.ip_banned {
        return rejected(checked, AdmissionRejection::IpBanned);
    }
    checked.push(AdmissionGate::Capacity);
    if request.current_players >= request.capacity && !request.bypass_capacity {
        return rejected(checked, AdmissionRejection::ServerFull);
    }
    AdmissionDecision {
        checked,
        rejection: None,
    }
}

fn rejected(checked: Vec<AdmissionGate>, rejection: AdmissionRejection) -> AdmissionDecision {
    AdmissionDecision {
        checked,
        rejection: Some(rejection),
    }
}

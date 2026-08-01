#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetBorderCenter {
    pub center_x: f64,
    pub center_z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetBorderLerpSize {
    pub old_size: f64,
    pub new_size: f64,
    pub duration_millis: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetBorderSize {
    pub size: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetBorderWarningDelay {
    pub warning_time: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetBorderWarningDistance {
    pub warning_blocks: i32,
}

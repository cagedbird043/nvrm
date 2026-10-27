use super::super::VideoError;

/// Origin-preserving encoder errors; frame validation is distinct from channel health.
#[derive(Debug)]
pub enum EncError {
    InvalidInputPlane { plane: usize, reason: &'static str },
    InputSlotsBusy,
    InvalidUpdate { field: &'static str },
    DeviceOpen { source: VideoError },
    ChannelSubmit { source: VideoError },
    Completion { ticket: u32, source: VideoError },
    ChannelOperation { stage: &'static str, source: VideoError },
    PutReadbackMismatch { expected: u32, observed: u32 },
    ChannelTimeout { ticket: u64 },
    FencePoll { errno: i32, revents: i16 },
    CommandListTooLong { bytes: usize },
    TicketOrder { expected: u32, actual: u32 },
    FrameMicrocode { ticket: u32, status: u32 },
    EmptyBitstream { ticket: u32 },
    BitstreamRange { offset: usize, bytes: usize, capacity: usize },
    Av1TileCount { expected: usize, actual: usize },
    Av1EmptyTile { tile: usize },
    Av1TileRange { tile: usize, offset: usize, bytes: usize, capacity: usize },
    OutputAssemblyCapacity,
    InternalInvariant { reason: &'static str },
    Panic,
}
impl EncError {
    pub fn recoverable(&self) -> bool {
        matches!(self, Self::InvalidInputPlane { .. } | Self::InputSlotsBusy
            | Self::InvalidUpdate { .. } | Self::FrameMicrocode { .. }
            | Self::EmptyBitstream { .. } | Self::BitstreamRange { .. }
            | Self::Av1TileCount { .. } | Self::Av1EmptyTile { .. }
            | Self::Av1TileRange { .. } | Self::OutputAssemblyCapacity)
    }
}
impl std::fmt::Display for EncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for EncError {}
impl From<EncError> for VideoError {
    fn from(e: EncError) -> Self {
        if e.recoverable() { Self::Invalid(e.to_string()) } else { Self::DeviceLost(e.to_string()) }
    }
}

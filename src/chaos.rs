pub const DEFAULT_CHAOS_RATE: f32 = 0.1;

#[derive(Debug, Clone, Copy)]
pub enum ChaosFault {
    LatencySpike { duration_ms: u64 },
    CorruptedPayload,
    MetadataCorruption,
    ConnectionDrop,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChaosEngine {
    pub enabled: bool,
    pub rate: f32,
}

impl ChaosEngine {
    pub fn new(enabled: bool, rate: f32) -> Self {
        Self { enabled, rate }
    }

    #[inline]
    pub fn select_fault(&self) -> Option<ChaosFault> {
        if !self.enabled || fastrand::f32() >= self.rate {
            return None;
        }
        match fastrand::u8(0..4) {
            0 => Some(ChaosFault::LatencySpike { duration_ms: 150 }),
            1 => Some(ChaosFault::CorruptedPayload),
            2 => Some(ChaosFault::MetadataCorruption),
            _ => Some(ChaosFault::ConnectionDrop),
        }
    }
}

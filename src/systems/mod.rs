use map_system::MapSystem;
use time_system::TimeSystem;

use crate::events::{events::EventSystem, triggers::TriggerSystem};

pub mod map_system;
pub mod time_system;

pub struct Systems {
    pub time: TimeSystem,
    pub map: MapSystem,
    pub trigger: TriggerSystem,
    pub event: EventSystem,
}
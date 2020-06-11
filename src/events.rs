use std::io;

#[derive(Copy, Clone, Debug)]
pub enum ExtEventState {
    On,
    Off,
}

impl ExtEventState {
    pub fn from_str(string: &str) -> Result<Self, io::Error> {
        match string {
            "on" => Ok(ExtEventState::On),
            "off" => Ok(ExtEventState::Off),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Bad event state (should on/off)",
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExtEvent {
    pub name: String,
    pub state: ExtEventState,
}

#[derive(Clone, Debug)]
pub struct FocusEvent {
    pub window: String,
}

#[derive(Clone, Debug)]
pub enum AltEvent {
    AltFocusEvent(FocusEvent),
    AltExtEvent(ExtEvent),
}


#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DfrCanId {
    pub priority: u16,
    pub target: u16,
    pub command: u16,
    pub source: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum BootloaderCommand {
    Ping = 0x45,
    Erase = 0x46,
    Write = 0x47,
    Jump = 0x48,
    SetAddress = 0x49,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CanDevices {
    RaspberryPi = 0x01,
    Nuc1 = 0x06,
    Nuc2 = 0x07,
    UNKNOWN = 0x1F,
}

pub fn parse_can_id(raw_id: u32) -> DfrCanId {
    // raw_id (29 bits) = [priority 3b][target 5b][command 16b][source 5b]
    let priority = ((raw_id >> 26) & 0x07) as u16;
    let target = ((raw_id >> 21) & 0x1F) as u16;
    let command = ((raw_id >> 5) & 0xFFFF) as u16;
    let source = (raw_id & 0x1F) as u16;

    DfrCanId {
        priority,
        target,
        command,
        source,
    }
}

impl TryFrom<u16> for BootloaderCommand {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == BootloaderCommand::Ping as u16 => Ok(BootloaderCommand::Ping),
            x if x == BootloaderCommand::Erase as u16 => Ok(BootloaderCommand::Erase),
            x if x == BootloaderCommand::SetAddress as u16 => Ok(BootloaderCommand::SetAddress),
            x if x == BootloaderCommand::Write as u16 => Ok(BootloaderCommand::Write),
            x if x == BootloaderCommand::Jump as u16 => Ok(BootloaderCommand::Jump),
            _ => Err(()),
        }
    }
}

impl From<BootloaderCommand> for u16 {
    fn from(cmd: BootloaderCommand) -> Self {
        cmd as u16
    }
}

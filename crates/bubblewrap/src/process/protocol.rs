use crate::Result;
use nix::fcntl::OFlag;
use nix::unistd::pipe2;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Message {
    UserNamespaceReady = 1,
    IdMappingsInstalled = 2,
    InitReady = 3,
    Abort = 255,
}

pub(crate) struct Protocol {
    reader: File,
    writer: File,
}

impl Protocol {
    pub(crate) fn pair() -> Result<(Self, Self)> {
        let (left_reader, left_writer) = pipe2(OFlag::O_CLOEXEC)?;
        let (right_reader, right_writer) = pipe2(OFlag::O_CLOEXEC)?;

        let left = Self {
            reader: File::from(right_reader),
            writer: File::from(left_writer),
        };
        let right = Self {
            reader: File::from(left_reader),
            writer: File::from(right_writer),
        };
        Ok((left, right))
    }

    pub(crate) fn send(&mut self, message: Message) -> Result<()> {
        self.writer.write_all(&[message as u8])?;
        Ok(())
    }

    pub(crate) fn receive(&mut self) -> Result<Message> {
        let mut byte = [0];
        self.reader.read_exact(&mut byte)?;
        match byte[0] {
            1 => Ok(Message::UserNamespaceReady),
            2 => Ok(Message::IdMappingsInstalled),
            3 => Ok(Message::InitReady),
            255 => Ok(Message::Abort),
            _ => Err(crate::Error::InvalidConfig(
                "invalid process protocol message",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_round_trip() {
        let (mut sender, mut receiver) = Protocol::pair().unwrap();
        sender.send(Message::UserNamespaceReady).unwrap();
        sender.send(Message::InitReady).unwrap();

        assert_eq!(receiver.receive().unwrap(), Message::UserNamespaceReady);
        assert_eq!(receiver.receive().unwrap(), Message::InitReady);
    }
}

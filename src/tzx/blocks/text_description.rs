use binrw::{
    binrw,
};
use std::any::Any;
use std::fmt;
use crate::tzx::blocks::Block;
use crate::tzx::blocks::BlockType;

// A [Text description](https://worldofspectrum.net/TZXformat.html#TEXTDESCR) block.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct TextDescription {
    length: u8,
    #[br(count = length)]
    text: Vec<u8>
}

impl fmt::Display for TextDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = String::from_utf8_lossy(&self.text);
        write!(f, "TextDescription: {}", description)
    }
}

impl Block for TextDescription {
    fn r#type(&self) -> BlockType {
        return BlockType::TextDescription;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// A [Message block](https://worldofspectrum.net/features/TZXformat.html#MSGBLOCK).
/// Parsed, but unsupported beyond display of the message during playback / inspection. The display time is shown
/// but not respected.
#[binrw]
#[brw(little)]
#[derive(Clone, Debug)]
pub struct MessageBlock {
    display_for_secs: u8,
    length: u8,
    #[br(count = length)]
    text: Vec<u8>
}

impl fmt::Display for MessageBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = String::from_utf8_lossy(&self.text);
        write!(f, "MessageBlock: {} ({}s)", description, self.display_for_secs)
    }
}

impl Block for MessageBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::MessageBlock;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

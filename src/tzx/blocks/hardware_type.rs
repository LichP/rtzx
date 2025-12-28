pub mod adc_dac;
pub mod computer;
pub mod digitizer;
pub mod eprom_programmer;
pub mod external_storage;
pub mod graphics_device;
pub mod joystick;
pub mod keyboard;
pub mod modem;
pub mod mouse;
pub mod network_adapter;
pub mod other_controller;
pub mod parallel_port;
pub mod printer;
pub mod rom_ram;
pub mod serial_port;
pub mod sound;

pub use adc_dac::AdcDacType;
pub use computer::ComputerType;
pub use digitizer::DigitizerType;
pub use eprom_programmer::EpromProgrammerType;
pub use external_storage::ExternalStorageType;
pub use graphics_device::GraphicsDeviceType;
pub use joystick::JoystickType;
pub use keyboard::KeyboardType;
pub use modem::ModemType;
pub use mouse::MouseType;
pub use network_adapter::NetworkAdapterType;
pub use other_controller::OtherControllerType;
pub use parallel_port::ParallelPortType;
pub use printer::PrinterType;
pub use rom_ram::RomRamType;
pub use serial_port::SerialPortType;
pub use sound::SoundType;

use binrw::{
    binrw,
    BinRead, BinWrite,
};
use std::fmt;
use strum_macros::Display;

use crate::tzx::{
    ExtendedDisplayCollector,
    RecoveryEnum,
    blocks::{Block, BlockType}
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct HardwareTypeBlock {
    entry_count: u8,
    #[br(count = entry_count)]
    entries: Vec<HardwareTypeBlockEntry>
}

impl fmt::Display for HardwareTypeBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HardwareTypeBlock: {} entries", self.entry_count)
    }
}

impl Block for HardwareTypeBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::HardwareType;
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }

    fn extended_display(&self, out: &mut dyn ExtendedDisplayCollector) {
        for entry in &self.entries {
            out.push(entry);
        }
    }
}

#[derive(BinRead)]
#[brw(little)]
#[derive(Clone, Copy, Display, Debug)]
pub enum HardwareType {
    #[br(magic = 0u8)]
    #[strum(to_string = "Computer::{0}")]
    Computer(RecoveryEnum<ComputerType, u8>),

    #[br(magic = 1u8)]
    #[strum(to_string = "ExternalStorage::{0}")]
    ExternalStorage(RecoveryEnum<ExternalStorageType, u8>),

    #[br(magic = 2u8)]
    #[strum(to_string = "RomRam::{0}")]
    RomRam(RecoveryEnum<RomRamType, u8>),

    #[br(magic = 3u8)]
    #[strum(to_string = "Sound::{0}")]
    Sound(RecoveryEnum<SoundType, u8>),

    #[br(magic = 4u8)]
    #[strum(to_string = "Joystick::{0}")]
    Joystick(RecoveryEnum<JoystickType, u8>),

    #[br(magic = 5u8)]
    #[strum(to_string = "Mouse::{0}")]
    Mouse(RecoveryEnum<MouseType, u8>),

    #[br(magic = 6u8)]
    #[strum(to_string = "OtherController::{0}")]
    OtherController(RecoveryEnum<OtherControllerType, u8>),

    #[br(magic = 7u8)]
    #[strum(to_string = "SerialPort::{0}")]
    SerialPort(RecoveryEnum<SerialPortType, u8>),

    #[br(magic = 8u8)]
    #[strum(to_string = "ParallelPort::{0}")]
    ParallelPort(RecoveryEnum<ParallelPortType, u8>),

    #[br(magic = 9u8)]
    #[strum(to_string = "Printer::{0}")]
    Printer(RecoveryEnum<PrinterType, u8>),

    #[br(magic = 10u8)]
    #[strum(to_string = "Modem::{0}")]
    Modem(RecoveryEnum<ModemType, u8>),

    #[br(magic = 11u8)]
    #[strum(to_string = "Digitizer::{0}")]
    Digitizer(RecoveryEnum<DigitizerType, u8>),

    #[br(magic = 12u8)]
    #[strum(to_string = "NetworkAdapter::{0}")]
    NetworkAdapter(RecoveryEnum<NetworkAdapterType, u8>),

    #[br(magic = 13u8)]
    #[strum(to_string = "Keyboard::{0}")]
    Keyboard(RecoveryEnum<KeyboardType, u8>),

    #[br(magic = 14u8)]
    #[strum(to_string = "AdcDac::{0}")]
    AdcDac(RecoveryEnum<AdcDacType, u8>),

    #[br(magic = 15u8)]
    #[strum(to_string = "EpromProgrammer::{0}")]
    EpromProgrammer(RecoveryEnum<EpromProgrammerType, u8>),

    #[br(magic = 16u8)]
    #[strum(to_string = "GraphicsDevice::{0}")]
    GraphicsDevice(RecoveryEnum<GraphicsDeviceType, u8>),
}

impl BinWrite for HardwareType {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        match self {
            HardwareType::Computer(inner) => {
                0u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::ExternalStorage(inner) => {
                1u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::RomRam(inner) => {
                2u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::Sound(inner) => {
                3u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::Joystick(inner) => {
                4u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::Mouse(inner) => {
                5u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::OtherController(inner) => {
                6u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::SerialPort(inner) => {
                7u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::ParallelPort(inner) => {
                8u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::Printer(inner) => {
                9u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::Modem(inner) => {
                10u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::Digitizer(inner) => {
                111u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::NetworkAdapter(inner) => {
                12u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::Keyboard(inner) => {
                13u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::AdcDac(inner) => {
                14u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::EpromProgrammer(inner) => {
                15u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
            HardwareType::GraphicsDevice(inner) => {
                16u8.write_options(writer, endian, ())?;
                inner.write_options(writer, endian, ())?;
            }
        }
        Ok(())
    }
}

#[binrw]
#[brw(little, repr = u8)]
#[derive(Clone, Copy, Display, Debug)]
pub enum HardwareInformation {
    Runs = 0x00,
    Uses = 0x01,
    RunsDoesntUse = 0x02,
    DoesntRun = 0x03,
}

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct HardwareTypeBlockEntry {
    hardware_type: RecoveryEnum<HardwareType, u8>,
    #[br(if(match hardware_type { RecoveryEnum::Known(_) => false, _ => true }, 0))]
    unknown_hardware_id: u8,
    information: RecoveryEnum<HardwareInformation, u8>,
}

impl fmt::Display for HardwareTypeBlockEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.hardware_type, self.information)
    }
}

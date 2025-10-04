use binrw::{
    binrw,
};
use std::fmt;
use std::sync::Arc;

use crate::tzx::{
    Machine,
    blocks::{Block, BlockType},
    waveforms::{
        DataWaveform,
        PauseWaveform,
        PilotWaveform,
        SyncWaveform,
        Waveform,
    },
};

#[binrw]
#[brw(little)]
#[derive(Clone)]
pub struct StandardSpeedDataBlock {
    pause: u16,
    #[bw(try_calc(u16::try_from(data.len())))]
    length: u16,
    #[br(count = length)]
    data: Vec<u8>,
}

impl fmt::Display for StandardSpeedDataBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StandardSpeedDataBlock: {} bytes, pause {}ms", self.data.len(), self.pause)
    }
}

impl Block for StandardSpeedDataBlock {
    fn r#type(&self) -> BlockType {
        return BlockType::StandardSpeedDataBlock;
    }

    fn get_waveforms(&self, machine: Arc<Machine>, start_pulse_high: bool) -> Vec<Box<dyn Waveform + Send>> {
        let header = self.data[0] < 128;
        let pilot_source = PilotWaveform::new(
            machine.clone(),
            2168,
            if header { 8063 } else { 3223 },
            start_pulse_high,
        );
        let sync_pulses_source = SyncWaveform::new(
            machine.clone(),
            667,
            735,
            start_pulse_high,
        );
        let data_source = DataWaveform::new(
            machine.clone(),
            855,
            1710,
            &self.data,
            8,
            start_pulse_high,
        );
        let pause_source = PauseWaveform::new(self.pause);

        return vec![Box::new(pilot_source), Box::new(sync_pulses_source), Box::new(data_source), Box::new(pause_source)];
    }

    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

use std::sync::Arc;

use crate::gameboy::memory::GameboyMemory;

const DMA_COPY_SIZE: u16 = 0x9F;
const TRANSFER_TARGET: u16 = 0xFE00;

pub struct DmaTransfer {
    source: u16,
    current: u16,

    copied: usize,
    started_at: usize,
    gb_mem: Arc<GameboyMemory>
}

impl DmaTransfer {
    pub fn new(source: u8, started_at: usize, gb_mem: Arc<GameboyMemory>) -> DmaTransfer {
        let source = (source as u16) << 8;

        DmaTransfer {
            source,
            current: TRANSFER_TARGET,

            copied: 0,
            started_at,
            gb_mem
        }
    }

    pub fn step(&mut self, cycles: usize) -> bool {
        let elapsed = cycles - self.started_at;
        let bytes_to_copy = {
            let missing = DMA_COPY_SIZE as usize - self.copied;
            let mut amount = (elapsed / 4) - self.copied;

            if amount > missing {
                amount = missing
            }
            
            amount
        };

        for _ in 0..bytes_to_copy {
            let byte = self.gb_mem.read(self.source);
            self.gb_mem.write(self.current, byte);

            self.copied += 1;
            self.source += 1;
            self.current += 1;
        }

        self.current >= TRANSFER_TARGET + DMA_COPY_SIZE
    }
}

use std::sync::{Arc, RwLock};

pub struct IoRegister {
    value: RwLock<u8>,

    write_mask: Arc<u8>,
    unused_mask: Arc<u8>
}

impl Clone for IoRegister {
    fn clone(&self) -> Self {
        IoRegister {
            value: RwLock::new(*self.value.read().unwrap()),

            write_mask: self.write_mask.clone(),
            unused_mask: self.unused_mask.clone()
        }
    }
}

impl Default for IoRegister {
    fn default() -> IoRegister {
        IoRegister {
            value: RwLock::new(0),

            write_mask: Arc::new(0b1111_1111),
            unused_mask: Arc::new(0b0000_0000)
        }
    }
}

impl IoRegister {
    pub fn init(value: u8, write_mask: u8, unused_mask: u8) -> IoRegister {
        let value = RwLock::new(value);
        let write_mask = Arc::new(write_mask);
        let unused_mask = Arc::new(unused_mask);

        IoRegister {
            value,

            write_mask,
            unused_mask
        }
    }

    // For internal use of the component that owns the register.
    // Gets the real value, without masks.
    pub fn get(&self) -> u8 {
        *self.value.read().unwrap()
    }

    // For internal use of the component that owns the register.
    // Sets the value ignoring the write mask.
    pub fn set(&self, value: u8) {
        *self.value.write().unwrap() = value
    }

    // For external reads, gets the value | unused bits mask.
    pub fn read(&self) -> u8 {
        self.get() | *self.unused_mask
    }

    // For external writes, writes a new value AND'd against the write mask.
    // Preserves the bits protected by the mask.
    pub fn write(&self, value: u8) {
        let value = value & *self.write_mask;
        let result = (self.get() & !*self.write_mask) | value;

        self.set(result);
    }
}

pub fn init_io_regs() -> Vec<Arc<IoRegister>> {
    let mut io = {
        let mut res = Vec::with_capacity(128);

        for _ in 0..128 {
            res.push(Arc::new(IoRegister::default()));
        }

        res
    };

    // FF02 - SC.
    io[0x02] = Arc::new(IoRegister::init(0, 0b1000_0001, 0b0111_1110));

    // Unused.
    io[0x03] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));

    // FF04 - DIV.
    io[0x04] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b0000_0000));
    // FF07 - TAC.
    io[0x07] = Arc::new(IoRegister::init(0, 0b0000_0111, 0b1111_1000));

    // Unused.
    io[0x08] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x09] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x0A] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x0B] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x0C] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x0D] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x0E] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));

    // 0xF00F - IF.
    io[0x0F] = Arc::new(IoRegister::init(0, 0b0111_1111, 0b1000_0000));

    // Unused.
    io[0x15] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x1F] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));

    // 0xFF26 - NR52.
    io[0x26] = Arc::new(IoRegister::init(0, 0b1000_0000, 0b0111_0000));

    // Unused.
    io[0x27] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x28] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x29] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x2A] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x2B] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x2C] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x2D] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x2E] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x2F] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));

    // 0xFF40 - STAT.
    io[0x41] = Arc::new(IoRegister::init(0, 0b0111_1000, 0b1000_0000));
    // 0xFF44 - LY.
    io[0x44] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b0000_0000));

    // Unused.
    io[0x4D] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x4E] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x4F] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));

    // 0xFF50 - BOOT
    io[0x50] = Arc::new(IoRegister::init(0, 0b0000_0001, 0b1111_1110));

    // Unused.
    io[0x57] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x58] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x59] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x5A] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x5B] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x5C] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x5D] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x5E] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x5F] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x60] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x61] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x62] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x63] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x64] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x65] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x66] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x67] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x68] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x69] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x6A] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x6B] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x6C] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x6D] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x6E] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x6F] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x70] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x71] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x72] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x73] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x74] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x75] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x76] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x77] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x78] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x79] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x7A] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x7B] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x7C] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x7D] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x7E] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));
    io[0x7F] = Arc::new(IoRegister::init(0, 0b0000_0000, 0b1111_1111));

    io
}

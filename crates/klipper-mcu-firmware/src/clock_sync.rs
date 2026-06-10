#[cfg(target_arch = "avr")]
compile_error!("AVR architecture is explicitly not supported by this clock synchronizer.");

pub trait ClockSynchronizer {
    fn offset_nanoseconds(&self) -> i32;
    fn set_offset_nanoseconds(&mut self, offset_nanoseconds: i32);
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub struct NvicPllSynchronizer {
    pub offset_nanoseconds: i32,
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
impl NvicPllSynchronizer {
    pub const fn new(offset_nanoseconds: i32) -> Self {
        Self { offset_nanoseconds }
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
impl ClockSynchronizer for NvicPllSynchronizer {
    fn offset_nanoseconds(&self) -> i32 {
        self.offset_nanoseconds
    }

    fn set_offset_nanoseconds(&mut self, offset: i32) {
        self.offset_nanoseconds = offset;
    }
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub struct RiscvPllSynchronizer {
    pub offset_nanoseconds: i32,
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
impl RiscvPllSynchronizer {
    pub const fn new(offset_nanoseconds: i32) -> Self {
        Self { offset_nanoseconds }
    }
}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
impl ClockSynchronizer for RiscvPllSynchronizer {
    fn offset_nanoseconds(&self) -> i32 {
        self.offset_nanoseconds
    }

    fn set_offset_nanoseconds(&mut self, offset: i32) {
        self.offset_nanoseconds = offset;
    }
}

#[cfg(target_arch = "xtensa")]
pub struct XtensaPllSynchronizer {
    pub offset_nanoseconds: i32,
}

#[cfg(target_arch = "xtensa")]
impl XtensaPllSynchronizer {
    pub const fn new(offset_nanoseconds: i32) -> Self {
        Self { offset_nanoseconds }
    }
}

#[cfg(target_arch = "xtensa")]
impl ClockSynchronizer for XtensaPllSynchronizer {
    fn offset_nanoseconds(&self) -> i32 {
        self.offset_nanoseconds
    }

    fn set_offset_nanoseconds(&mut self, offset: i32) {
        self.offset_nanoseconds = offset;
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub type McuPllSynchronizer = NvicPllSynchronizer;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub type McuPllSynchronizer = RiscvPllSynchronizer;

#[cfg(target_arch = "xtensa")]
pub type McuPllSynchronizer = XtensaPllSynchronizer;
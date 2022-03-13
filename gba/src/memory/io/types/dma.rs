use util::{bitfields, primitive_enum};

#[derive(Copy, Clone, Default)]
pub struct DMARegisters {
    pub source: DMAAddress,
    pub destination: DMAAddress,
    pub count: u16,
    pub control: DMAControl,
}

bitfields! {
    pub struct DMAAddress: u32 {
        [0,15]  lo, set_lo: u16,
        [15,31] hi, set_hi: u16,
    }
}

bitfields! {
    /// 40000BAh - DMA0CNT_H - DMA 0 Control (R/W)
    /// 40000C6h - DMA1CNT_H - DMA 1 Control (R/W)
    /// 40000D2h - DMA2CNT_H - DMA 2 Control (R/W)
    /// 40000DEh - DMA3CNT_H - DMA 3 Control (R/W)
    ///   Bit   Expl.
    ///   0-4   Not used
    ///   5-6   Dest Addr Control  (0=Increment,1=Decrement,2=Fixed,3=Increment/Reload)
    ///   7-8   Source Adr Control (0=Increment,1=Decrement,2=Fixed,3=Prohibited)
    ///   9     DMA Repeat                   (0=Off, 1=On) (Must be zero if Bit 11 set)
    ///   10    DMA Transfer Type            (0=16bit, 1=32bit)
    ///   11    Game Pak DRQ  - DMA3 only -  (0=Normal, 1=DRQ <from> Game Pak, DMA3)
    ///   12-13 DMA Start Timing  (0=Immediately, 1=VBlank, 2=HBlank, 3=Special)
    ///           The 'Special' setting (Start Timing=3) depends on the DMA channel:
    ///           DMA0=Prohibited, DMA1/DMA2=Sound FIFO, DMA3=Video Capture
    ///   14    IRQ upon end of Word Count   (0=Disable, 1=Enable)
    ///   15    DMA Enable                   (0=Off, 1=On)
    pub struct DMAControl: u16 {
        [5,6]   dst_addr_control, set_dst_addr_control: AddressControl,
        [7,8]   src_addr_control, set_src_addr_control: AddressControl,
        [9]     repeat, set_repeat: bool,
        [10]    transfer_type, set_transfer_type: TransferType,
        [12]    gamepak_drq, set_gamepak_drq: bool,
        [13]    timing, set_timing: Timing,
        [14]    irq, set_irq: bool,
        [15]    enabled, set_enabled: bool,
    }
}

primitive_enum! {
    pub enum Timing: u16 {
        Immediate,
        VBlank,
        HBlank,
        Special,
    }
}

primitive_enum! {
    pub enum AddressControl: u16 {
        Increment = 0,
        Decrement,
        Fixed,
        IncrementReload,
    }
}

primitive_enum! {
    pub enum TransferType: u16 {
        Halfword = 0,
        Word = 1,
    }
}

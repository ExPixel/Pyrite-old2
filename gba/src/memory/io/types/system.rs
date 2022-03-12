use util::bitfields;

bitfields! {
    pub struct WaitstateControl: u16 {
        readonly = 0x8000
    }
}

#[macro_use]
extern crate nix;

#[macro_use]
mod macros;

mod event_bits_iter;
mod input;
mod input_sys;
mod uinput;
mod uinput_sys;
mod utils;

pub use crate::{
    input::{
        AbsoluteAxisBit,
        DeviceId,
        Device,
        EventBit,
        ForceFeedbackDuration,
        ForceFeedbackEffectKind,
        InputEvent,
        InputEventBody,
        poll_read
    },
    input_sys::{
        AbsoluteAxis,
        Bus,
        EventKind,
        ForceFeedback,
        Key,
        RawInputEvent,
        RelativeAxis,
        Timestamp
    },
    uinput::{
        DeviceCreateError,
        ForceFeedbackEffectErase,
        ForceFeedbackEffectUpload,
        ForceFeedbackRequest,
        VirtualDevice
    }
};

use {
    crate::{
        input_sys::{
            RawAbsInfo,
            RawForceFeedbackEffect,
            RawDeviceId
        }
    }
};

#[repr(C)]
pub struct RawDeviceSetup {
    pub id: RawDeviceId,
    pub name: [u8; 80],
    pub force_feedback_effects_max: u32
}

#[repr(C)]
pub struct RawAbsSetup {
    pub axis: u16,
    pub info: RawAbsInfo
}

#[repr(C)]
pub struct RawForceFeedbackUpload {
    pub request_id: u32,
    pub return_value: i32,
    pub effect: RawForceFeedbackEffect,
    pub old_effect: RawForceFeedbackEffect
}

#[repr(C)]
pub struct RawForceFeedbackErase {
    pub request_id: u32,
    pub return_value: i32,
    pub effect_id: u32
}

ioctl_write_ptr!( device_setup, b'U', 3, RawDeviceSetup );
ioctl_write_ptr!( abs_setup, b'U', 4, RawAbsSetup );
ioctl_none!( device_create, b'U', 1 );
ioctl_none!( device_destroy, b'U', 2 );

ioctl_write_int!( device_set_event_bit, b'U', 100 );
ioctl_write_int!( device_set_key_bit, b'U', 101 );
ioctl_write_int!( device_set_relative_axis_bit, b'U', 102 );
ioctl_write_int!( device_set_absolute_axis_bit, b'U', 103 );
ioctl_write_int!( device_set_misc_bit, b'U', 104 );
ioctl_write_int!( device_set_force_feedback_bit, b'U', 107 );

ioctl_readwrite!( begin_force_feedback_upload, b'U', 200, RawForceFeedbackUpload );
ioctl_write_ptr!( end_force_feedback_upload, b'U', 201, RawForceFeedbackUpload );
ioctl_readwrite!( begin_force_feedback_erase, b'U', 202, RawForceFeedbackErase );
ioctl_write_ptr!( end_force_feedback_erase, b'U', 203, RawForceFeedbackErase );

pub const EV_UINPUT: u16 = 0x0101;
pub const UI_FF_UPLOAD: u16 = 1;
pub const UI_FF_ERASE: u16 = 2;

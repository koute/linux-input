#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawDeviceId {
    pub bus: u16,
    pub vendor: u16,
    pub product: u16,
    pub version: u16
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawAbsInfo {
    /// Latest reported value for the axis.
    pub value: i32,
    /// Minimum value for the axis.
    pub minimum: i32,
    /// Maximum value for the axis.
    pub maximum: i32,
    /// Threshold for noise filtering.
    pub noise_threshold: i32,
    /// Deadzone.
    pub deadzone: i32,
    /// Resolution of the reported values.
    pub resolution: i32
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackTrigger {
    pub button: u16,
    pub interval: u16
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackReplay {
    pub length: u16,
    pub delay: u16
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackEnvelope {
    pub attack_length: u16,
    pub attack_level: u16,
    pub fade_length: u16,
    pub fade_level: u16
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackConstantEffect {
    pub level: u16,
    pub envelope: RawForceFeedbackEnvelope
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackRampEffect {
    pub start_level: i16,
    pub end_level: u16,
    pub envelope: RawForceFeedbackEnvelope
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackPeriodicEffect {
    pub waveform: u16,
    pub period: u16,
    pub magnitude: i16,
    pub offset: i16,
    pub phase: u16,
    pub envelope: RawForceFeedbackEnvelope,

    pub custom_length: u32,
    pub custom_data: *mut i16
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackConditionEffect {
    pub right_saturation: u16,
    pub left_saturation: u16,
    pub right_coefficient: i16,
    pub left_coefficient: i16,
    pub deadband: u16,
    pub center: i16
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RawForceFeedbackRumbleEffect {
    pub strong_magnitude: u16,
    pub weak_magnitude: u16
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union RawForceFeedbackBody {
    pub constant: RawForceFeedbackConstantEffect,
    pub ramp: RawForceFeedbackRampEffect,
    pub periodic: RawForceFeedbackPeriodicEffect,
    pub condition: [RawForceFeedbackConditionEffect; 2],
    pub rumble: RawForceFeedbackRumbleEffect
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RawForceFeedbackEffect {
    pub kind: u16,
    pub id: i16,
    pub direction: u16,
    pub trigger: RawForceFeedbackTrigger,
    pub replay: RawForceFeedbackReplay,
    pub body: RawForceFeedbackBody
}

pub const FF_RUMBLE: u16 = 0x50;
#[allow(dead_code)]
pub const FF_PERIODIC: u16 = 0x51;
#[allow(dead_code)]
pub const FF_CONSTANT: u16 = 0x52;
#[allow(dead_code)]
pub const FF_SPRING: u16 = 0x53;
#[allow(dead_code)]
pub const FF_FRICTION: u16 = 0x54;
#[allow(dead_code)]
pub const FF_DAMPER: u16 = 0x55;
#[allow(dead_code)]
pub const FF_INERTIA: u16 = 0x56;
#[allow(dead_code)]
pub const FF_RAMP: u16 = 0x57;

pub const FF_GAIN: u16 = 0x60;
#[allow(dead_code)]
pub const FF_AUTOCENTER: u16 = 0x61;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Timestamp {
    pub sec: libc::time_t,
    pub usec: libc::suseconds_t
}

impl Timestamp {
    /// Reads the current timestamp using the CLOCK_MONOTONIC source.
    pub fn get() -> Result< Self, std::io::Error > {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0
        };

        let result = unsafe {
            libc::clock_gettime( libc::CLOCK_MONOTONIC, &mut ts )
        };

        if result < 0 {
            Err( std::io::Error::last_os_error() )
        } else {
            Ok( Timestamp {
                sec: ts.tv_sec,
                usec: ts.tv_nsec / 1000
            })
        }
    }

    pub fn as_f64( self ) -> f64 {
        self.sec as f64 + self.usec as f64 / 1000_000.0
    }
}

impl std::ops::Sub for Timestamp {
    type Output = std::time::Duration;
    fn sub( self, rhs: Timestamp ) -> Self::Output {
        std::time::Duration::new( self.sec as _, self.usec as _ ) - std::time::Duration::new( rhs.sec as _, rhs.usec as _ )
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
#[repr(C)]
pub struct RawInputEvent {
    pub timestamp: Timestamp,
    pub kind: u16,
    pub code: u16,
    pub value: i32
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    enum EventKind {
        Other( u16 ),
        Synchronization     = 0x00,
        Key                 = 0x01,
        RelativeAxis        = 0x02,
        AbsoluteAxis        = 0x03,
        Misc                = 0x04,
        Switch              = 0x05,
        LED                 = 0x11,
        Sound               = 0x12,
        AutoRepeat          = 0x14,
        ForceFeedback       = 0x15,
        Power               = 0x16,
        ForceFeedbackStatus = 0x17
    }
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    enum Key {
        Other( u16 ),
        Escape = 1,
        Digit1 = 2,
        Digit2 = 3,
        Digit3 = 4,
        Digit4 = 5,
        Digit5 = 6,
        Digit6 = 7,
        Digit7 = 8,
        Digit8 = 9,
        Digit9 = 10,
        Digit0 = 11,
        Minus = 12,
        Equal = 13,
        Backspace = 14,
        Tab = 15,
        Q = 16,
        W = 17,
        E = 18,
        R = 19,
        T = 20,
        Y = 21,
        U = 22,
        I = 23,
        O = 24,
        P = 25,
        LeftBrace = 26,
        RightBrace = 27,
        Enter = 28,
        LeftCtrl = 29,
        A = 30,
        S = 31,
        D = 32,
        F = 33,
        G = 34,
        H = 35,
        J = 36,
        K = 37,
        L = 38,
        Semicolon = 39,
        Apostrophe = 40,
        Grave = 41,
        LeftShift = 42,
        Backslash = 43,
        Z = 44,
        X = 45,
        C = 46,
        V = 47,
        B = 48,
        N = 49,
        M = 50,
        Comma = 51,
        Dot = 52,
        Slash = 53,
        RightShift = 54,
        KeypadAsterisk = 55,
        LeftAlt = 56,
        Space = 57,
        CapsLock = 58,
        F1 = 59,
        F2 = 60,
        F3 = 61,
        F4 = 62,
        F5 = 63,
        F6 = 64,
        F7 = 65,
        F8 = 66,
        F9 = 67,
        F10 = 68,
        NumLock = 69,
        ScrollLock = 70,
        Keypad7 = 71,
        Keypad8 = 72,
        Keypad9 = 73,
        KeypadMinus = 74,
        Keypad4 = 75,
        Keypad5 = 76,
        Keypad6 = 77,
        KeypadPlus = 78,
        Keypad1 = 79,
        Keypad2 = 80,
        Keypad3 = 81,
        Keypad0 = 82,
        KeypadDot = 83,
        F11 = 87,
        F12 = 88,
        KeypadEnter = 96,
        RightCtrl = 97,
        KeypadSlash = 98,
        SysRq = 99,
        RightAlt = 100,
        Home = 102,
        Up = 103,
        PageUp = 104,
        Left = 105,
        Right = 106,
        End = 107,
        Down = 108,
        PageDown = 109,
        Insert = 110,
        Delete = 111,
        KeypadEqual = 117,
        KeypadPlusMinus = 118,
        Pause = 119,
        KeypadComma = 121,
        LeftMeta = 125,
        RightMeta = 126,
        MouseLeft = 0x110,
        MouseRight = 0x111,
        MouseMiddle = 0x112,
        MouseExtra1 = 0x113,
        MouseExtra2 = 0x114,
        MouseExtra3 = 0x115,
        MouseExtra4 = 0x116,
        MouseExtra5 = 0x117,

        // https://www.kernel.org/doc/html/v4.15/input/gamepad.html
        PadSouth = 0x130,
        PadEast = 0x131,
        PadNorth = 0x133,
        PadWest = 0x134,
        ShoulderLeft = 0x136,
        ShoulderRight = 0x137,
        ShoulderLeftLower = 0x138,
        ShoulderRightLower = 0x139,
        Select = 0x13a,
        Start = 0x13b,
        HomeButton = 0x13c,
        StickLeft = 0x13d,
        StickRight = 0x13e,
        PadUp = 0x220,
        PadDown = 0x221,
        PadLeft = 0x222,
        PadRight = 0x223,

        ButtonMisc = 0x100,
        TriggerHappy = 0x2c0
    }
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    enum RelativeAxis {
        Other( u16 ),
        X = 0,
        Y = 1,
        Z = 2,
        RX = 3,
        RY = 4,
        RZ = 5,
        Wheel = 8,
        WheelHiRes = 11
    }
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    enum AbsoluteAxis {
        Other( u16 ),
        X = 0,
        Y = 1,
        Z = 2,
        RX = 3,
        RY = 4,
        RZ = 5,
        Hat0X = 16,
        Hat0Y = 17,
        Hat1X = 18,
        Hat1Y = 19,
        Hat2X = 20,
        Hat2Y = 21,
        Misc = 40
    }
}

define_enum! {
    // Source: linux/input.h
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    enum Bus {
        Other( u16 ),
        PCI = 0x01,
        USB = 0x03,
        HIL = 0x04,
        Bluetooth = 0x05,
        Virtual = 0x06,
        ISA = 0x10,
        Host = 0x19
    }
}

define_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    enum ForceFeedback {
        Other( u16 ),
        Rumble = 0x50
    }
}

ioctl_write_int!( evdev_grab_or_release, b'E', 0x90 );
ioctl_read!( evdev_get_id, b'E', 0x02, RawDeviceId );
ioctl_write_ptr!( evdev_set_clock_id, b'E', 0xa0, libc::c_int );

ioctl_write_ptr!( evdev_start_force_feedback, b'E', 0x80, RawForceFeedbackEffect );
ioctl_write_int!( evdev_stop_force_feedback, b'E', 0x81 );
ioctl_read!( evdev_get_maximum_simultaneous_force_feedback_effect_count, b'E', 0x84, libc::c_int );

pub unsafe fn evdev_get_event_bits( fd: libc::c_int, kind: EventKind, data: *mut u8, length: usize ) -> nix::Result< libc::c_int > {
    let result = libc::ioctl( fd, request_code_read!( b'E', 0x20 + kind.raw() as usize, length ), data );
    nix::errno::Errno::result( result )
}

pub unsafe fn evdev_get_abs_info( fd: libc::c_int, axis: AbsoluteAxis ) -> nix::Result< RawAbsInfo > {
    let mut abs_info = std::mem::MaybeUninit::uninit();
    let result = libc::ioctl( fd, request_code_read!( b'E', 0x40 + axis.raw() as usize, std::mem::size_of::< RawAbsInfo >() ), abs_info.as_mut_ptr() );
    if result < 0 {
        return Err( nix::errno::Errno::last().into() );
    }

    Ok( abs_info.assume_init() )
}

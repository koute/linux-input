use {
    std::{
        fmt,
        fs::{
            self,
            File
        },
        io::{
            self
        },
        os::{
            unix::{
                io::{
                    AsRawFd
                }
            }
        },
        iter::{
            FusedIterator
        },
        mem::{
            self
        },
        path::{
            Path
        },
        slice,
        time::{
            Duration
        }
    },
    crate::{
        event_bits_iter::{
            EventBitsIter
        },
        input_sys::{
            self,
            AbsoluteAxis,
            Bus,
            EventKind,
            ForceFeedback,
            Key,
            RawAbsInfo,
            RawDeviceId,
            RawForceFeedbackBody,
            RawForceFeedbackEffect,
            RawForceFeedbackReplay,
            RawForceFeedbackRumbleEffect,
            RawForceFeedbackTrigger,
            RawInputEvent,
            RelativeAxis,
            Timestamp
        },
        utils::{
            ioctl_get_string
        }
    }
};

impl fmt::Debug for RawInputEvent {
    fn fmt( &self, fmt: &mut fmt::Formatter ) -> fmt::Result {
        fmt.debug_struct( "RawInputEvent" )
            .field( "timestamp", &self.timestamp )
            .field( "kind", &EventKind::from( self.kind ) )
            .field( "code", &self.code )
            .field( "value", &self.value )
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InputEvent {
    pub timestamp: Timestamp,
    pub body: InputEventBody
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InputEventBody {
    KeyPress( Key ),
    KeyRelease( Key ),
    RelativeMove {
        axis: RelativeAxis,
        delta: i32
    },
    AbsoluteMove {
        axis: AbsoluteAxis,
        position: i32
    },
    Flush,
    Dropped,
    Other {
        kind: EventKind,
        code: u16,
        value: i32
    }
}

impl From< RawInputEvent > for InputEvent {
    fn from( raw_event: RawInputEvent ) -> Self {
        let kind: EventKind = raw_event.kind.into();
        let body = match kind {
            EventKind::Key if raw_event.value == 1 => InputEventBody::KeyPress( raw_event.code.into() ),
            EventKind::Key if raw_event.value == 0 => InputEventBody::KeyRelease( raw_event.code.into() ),
            EventKind::RelativeAxis => InputEventBody::RelativeMove { axis: raw_event.code.into(), delta: raw_event.value },
            EventKind::AbsoluteAxis => InputEventBody::AbsoluteMove { axis: raw_event.code.into(), position: raw_event.value },
            EventKind::Synchronization if raw_event.code == 0 && raw_event.value == 0 => InputEventBody::Flush,
            EventKind::Synchronization if raw_event.code == 3 && raw_event.value == 0 => InputEventBody::Dropped,
            _ => InputEventBody::Other{
                kind: raw_event.kind.into(),
                code: raw_event.code,
                value: raw_event.value
            }
        };

        InputEvent {
            timestamp: raw_event.timestamp,
            body
        }
    }
}

impl From< InputEvent > for RawInputEvent {
    fn from( event: InputEvent ) -> Self {
        let (kind, code, value) = match event.body {
            InputEventBody::KeyPress( key ) => (EventKind::Key, key.into(), 1),
            InputEventBody::KeyRelease( key ) => (EventKind::Key, key.into(), 0),
            InputEventBody::RelativeMove { axis, delta } => (EventKind::RelativeAxis, axis.into(), delta),
            InputEventBody::AbsoluteMove { axis, position } => (EventKind::AbsoluteAxis, axis.into(), position),
            InputEventBody::Flush => (EventKind::Synchronization, 0, 0),
            InputEventBody::Dropped => (EventKind::Synchronization, 3, 0),
            InputEventBody::Other { kind, code, value } => (kind, code, value)
        };

        RawInputEvent {
            timestamp: event.timestamp,
            kind: kind.into(),
            code,
            value
        }
    }
}

impl From< InputEvent > for InputEventBody {
    fn from( event: InputEvent ) -> Self {
        event.body
    }
}

impl AsRef< InputEventBody > for InputEvent {
    fn as_ref( &self ) -> &InputEventBody {
        &self.body
    }
}

impl AsRef< InputEventBody > for InputEventBody {
    fn as_ref( &self ) -> &InputEventBody {
        self
    }
}

pub trait EventCode: From< u16 > {
    const EVENT_KIND: EventKind;
}

impl EventCode for Key {
    const EVENT_KIND: EventKind = EventKind::Key;
}

impl EventCode for RelativeAxis {
    const EVENT_KIND: EventKind = EventKind::RelativeAxis;
}

impl EventCode for AbsoluteAxis {
    const EVENT_KIND: EventKind = EventKind::AbsoluteAxis;
}

impl EventCode for ForceFeedback {
    const EVENT_KIND: EventKind = EventKind::ForceFeedback;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DeviceId {
    pub bus: Bus,
    pub vendor: u16,
    pub product: u16,
    pub version: u16
}

impl From< RawDeviceId > for DeviceId {
    fn from( id: RawDeviceId ) -> Self {
        DeviceId {
            bus: id.bus.into(),
            vendor: id.vendor,
            product: id.product,
            version: id.version
        }
    }
}

impl From< DeviceId > for RawDeviceId {
    fn from( id: DeviceId ) -> Self {
        RawDeviceId {
            bus: id.bus.into(),
            vendor: id.vendor,
            product: id.product,
            version: id.version
        }
    }
}

pub(crate) fn emit_into< T >( fp: &File, body: T ) -> Result< (), io::Error > where T: AsRef< InputEventBody > {
    let raw_event: RawInputEvent = InputEvent {
        timestamp: Timestamp {
            sec: 0,
            usec: 0
        },
        body: body.as_ref().clone()
    }.into();

    let bytes = &raw_event as *const RawInputEvent as *const u8;
    let bytes = unsafe { slice::from_raw_parts( bytes, mem::size_of::< RawInputEvent >() ) };
    let result = unsafe { libc::write( fp.as_raw_fd(), bytes.as_ptr() as *const libc::c_void, bytes.len() ) };
    if result < 0 {
        return Err( io::Error::last_os_error() );
    }

    let count = result as usize;
    assert_eq!( count, bytes.len() );

    Ok(())
}

#[derive(Clone, Debug)]
pub struct AbsoluteAxisBit {
    pub axis: AbsoluteAxis,
    pub initial_value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub noise_threshold: i32,
    pub deadzone: i32,
    pub resolution: i32
}

#[derive(Clone, Debug)]
pub enum EventBit {
    Key( Key ),
    RelativeAxis( RelativeAxis ),
    AbsoluteAxis( AbsoluteAxisBit ),
    ForceFeedback( ForceFeedback )
}

#[derive(Clone, Debug)]
pub enum ForceFeedbackEffectKind {
    Rumble {
        strong_magnitude: u16,
        weak_magnitude: u16
    }
}

impl ForceFeedbackEffectKind {
    unsafe fn from_raw( kind: u16, body: &RawForceFeedbackBody ) -> Self {
        match kind {
            crate::input_sys::FF_RUMBLE => {
                let raw_effect = &body.rumble;
                ForceFeedbackEffectKind::Rumble {
                    strong_magnitude: raw_effect.strong_magnitude,
                    weak_magnitude: raw_effect.weak_magnitude
                }
            },
            kind => unimplemented!( "unsupported force feedback effect: {}", kind )
        }
    }
}

#[derive(Clone, Debug)]
pub enum ForceFeedbackDuration {
    Finite( std::time::Duration ),
    Infinite
}

#[derive(Clone, Debug)]
pub struct ForceFeedbackEffect {
    pub id: i16,
    pub direction: u16,
    pub kind: ForceFeedbackEffectKind,
    pub duration: ForceFeedbackDuration,
    pub delay: std::time::Duration,
    pub trigger: RawForceFeedbackTrigger
}

impl ForceFeedbackEffect {
    pub(crate) unsafe fn from_raw( raw_effect: &RawForceFeedbackEffect ) -> Self {
        ForceFeedbackEffect {
            id: raw_effect.id,
            direction: raw_effect.direction,
            kind: ForceFeedbackEffectKind::from_raw( raw_effect.kind, &raw_effect.body ),
            trigger: raw_effect.trigger,
            duration: match raw_effect.replay.length {
                0 => ForceFeedbackDuration::Infinite,
                length => ForceFeedbackDuration::Finite( std::time::Duration::from_millis( length as u64 ) )
            },
            delay: std::time::Duration::from_millis( raw_effect.replay.delay as u64 )
        }
    }
}

fn convert_and_clip( duration: std::time::Duration ) -> u16 {
    let duration = duration.as_millis();
    if duration > 0x7fff {
        0x7fff
    } else {
        duration as u16
    }
}

impl From< ForceFeedbackEffect > for RawForceFeedbackEffect {
    fn from( effect: ForceFeedbackEffect ) -> Self {
        RawForceFeedbackEffect {
            id: effect.id,
            direction: effect.direction,
            trigger: effect.trigger,
            replay: RawForceFeedbackReplay {
                length: match effect.duration {
                    ForceFeedbackDuration::Finite( duration ) => convert_and_clip( duration ),
                    ForceFeedbackDuration::Infinite => 0
                },
                delay: convert_and_clip( effect.delay )
            },
            body: match effect.kind {
                ForceFeedbackEffectKind::Rumble { weak_magnitude, strong_magnitude } => {
                    RawForceFeedbackBody {
                        rumble: RawForceFeedbackRumbleEffect {
                            weak_magnitude, strong_magnitude
                        }
                    }
                }
            },
            kind: match effect.kind {
                ForceFeedbackEffectKind::Rumble { .. } => crate::input_sys::FF_RUMBLE
            }
        }
    }
}

pub struct Device {
    fp: File
}

pub fn poll_read( fd: std::os::unix::io::RawFd, timeout: Option< Duration > ) -> Result< bool, io::Error > {
    let timeout = timeout.map( |timeout| {
        libc::timespec {
            tv_sec: timeout.as_secs() as _,
            tv_nsec: timeout.subsec_nanos() as _
        }
    });

    let timeout_p = timeout.as_ref().map( |timeout: &libc::timespec| timeout as *const libc::timespec ).unwrap_or( std::ptr::null() );
    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0
    };

    let sigmask = unsafe {
        let mut sigmask = std::mem::MaybeUninit::uninit();
        let errcode = libc::sigemptyset( sigmask.as_mut_ptr() );
        assert_eq!( errcode, 0 );

        sigmask.assume_init()
    };

    // `nix`'s bindings for this are broken, so we call it manually.
    let result = unsafe {
        libc::ppoll( &mut pollfd, 1, timeout_p, &sigmask )
    };

    std::mem::drop( timeout );

    if result < 0 {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            return Ok( false );
        }
        return Err( error );
    } else if result == 0 {
        return Ok( false );
    }

    Ok( pollfd.revents & (libc::POLLIN | libc::POLLHUP) != 0 )
}

pub(crate) fn read_raw_input_event( fp: &File, timeout: Option< Duration > ) -> Result< Option< RawInputEvent >, io::Error > {
    if poll_read( fp.as_raw_fd(), timeout )? {
        let mut buffer = RawInputEvent::default();
        let raw_buffer = unsafe {
            std::slice::from_raw_parts_mut( &mut buffer as *mut RawInputEvent as *mut u8, mem::size_of::< RawInputEvent >() )
        };

        let result = unsafe { libc::read( fp.as_raw_fd(), raw_buffer.as_mut_ptr() as *mut libc::c_void, raw_buffer.len() as libc::size_t ) };
        if result < 0 {
            return Err( io::Error::last_os_error() );
        }

        let count = result as usize;
        if count == mem::size_of::< RawInputEvent >() {
            return Ok( Some( buffer ) );
        }

        assert_eq!( count, 0 );
    }

    Ok( None )
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ForceFeedbackEffectId( i16 );

impl Device {
    pub fn open< P >( path: P ) -> Result< Self, io::Error > where P: AsRef< Path > {
        let path = path.as_ref();
        let fp = fs::OpenOptions::new()
            .read( true )
            .write( true )
            .create( false )
            .open( path )?;

        let flags = unsafe { libc::fcntl( fp.as_raw_fd(), libc::F_GETFL, 0 ) };
        if flags < 0 {
            let error = io::Error::last_os_error();
            return Err( error );
        }

        if unsafe { libc::fcntl( fp.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK ) < -1 } {
            let error = io::Error::last_os_error();
            return Err( error );
        }

        let device = Device {
            fp
        };

        device.set_clock_source( libc::CLOCK_MONOTONIC )
            .map_err( |error| io::Error::new( io::ErrorKind::Other, format!( "failed to set the clock source to CLOCK_MONOTONIC: {}", error ) ) )?;

        Ok( device )
    }

    pub fn id( &self ) -> Result< DeviceId, nix::Error > {
        let mut raw_id = RawDeviceId {
            bus: 0,
            vendor: 0,
            product: 0,
            version: 0
        };

        unsafe {
            input_sys::evdev_get_id( self.fp.as_raw_fd(), &mut raw_id )?;
        }

        Ok( raw_id.into() )
    }

    pub fn name( &self ) -> Result< String, nix::Error > {
        unsafe {
            ioctl_get_string( self.fp.as_raw_fd(), b'E', 0x06 )
        }
    }

    pub fn physical_location( &self ) -> Result< String, nix::Error > {
        unsafe {
            ioctl_get_string( self.fp.as_raw_fd(), b'E', 0x07 )
        }
    }

    pub fn read( &self, timeout: Option< Duration > ) -> Result< Option< InputEvent >, io::Error > {
        read_raw_input_event( &self.fp, timeout ).map( |event| event.map( |event| event.into() ) )
    }

    pub fn get_raw_abs_info( &self, axis: AbsoluteAxis ) -> Result< RawAbsInfo, nix::Error > {
        unsafe {
            crate::input_sys::evdev_get_abs_info( self.fp.as_raw_fd(), axis )
        }
    }

    fn append_event_bits_into_buffer( &self, kind: EventKind, buffer: &mut Vec< u8 > ) -> Result< usize, nix::Error > {
        let length = buffer.len();
        buffer.resize( length + 1024, 0 );
        let count = unsafe {
            crate::input_sys::evdev_get_event_bits( self.fp.as_raw_fd(), kind, buffer[ length..length + 1024 ].as_mut_ptr(), 1024 )?
        } as usize;
        buffer.truncate( length + count );

        Ok( count )
    }

    pub fn event_bits_of_kind< T >( &self ) -> Result< impl Iterator< Item = T > + FusedIterator, nix::Error > where T: EventCode {
        let mut buffer = Vec::new();
        self.append_event_bits_into_buffer( T::EVENT_KIND, &mut buffer )?;
        let iter = EventBitsIter::< T >::new( buffer.into() );
        Ok( iter )
    }

    pub fn absolute_axis_event_bits( &self ) -> Result< impl Iterator< Item = AbsoluteAxisBit > + FusedIterator, nix::Error > {
        let mut buffer = Vec::new();
        for axis in self.event_bits_of_kind::< AbsoluteAxis >()? {
            let info = self.get_raw_abs_info( axis )?;
            buffer.push( AbsoluteAxisBit {
                axis,
                initial_value: info.value,
                minimum: info.minimum,
                maximum: info.maximum,
                noise_threshold: info.noise_threshold,
                deadzone: info.deadzone,
                resolution: info.resolution
            });
        }

        Ok( buffer.into_iter() )
    }

    pub fn event_bits( &self ) -> Result< impl Iterator< Item = EventBit > + FusedIterator, nix::Error > {
        let mut output = Vec::new();
        let mut buffer = Vec::new();

        buffer.clear();
        self.append_event_bits_into_buffer( EventKind::Key, &mut buffer )?;
        output.extend( EventBitsIter::< Key >::new( (&buffer).into() ).map( EventBit::Key ) );

        buffer.clear();
        self.append_event_bits_into_buffer( EventKind::RelativeAxis, &mut buffer )?;
        output.extend( EventBitsIter::< RelativeAxis >::new( (&buffer).into() ).map( EventBit::RelativeAxis ) );

        output.extend( self.absolute_axis_event_bits()?.map( EventBit::AbsoluteAxis ) );

        Ok( output.into_iter() )
    }

    fn set_clock_source( &self, clock_source: libc::c_int ) -> Result< (), nix::Error > {
        unsafe {
            input_sys::evdev_set_clock_id( self.fp.as_raw_fd(), &clock_source )?;
        }

        Ok(())
    }

    pub fn upload_force_feedback_effect( &self, effect: impl Into< RawForceFeedbackEffect > ) -> Result< ForceFeedbackEffectId, nix::Error > {
        let mut effect = effect.into();
        effect.id = -1; // The kernel will automatically assign an ID.

        let id = unsafe {
            input_sys::evdev_start_force_feedback( self.fp.as_raw_fd(), &effect )?
        };

        assert!( id >= 0 && id <= std::i16::MAX as _ );
        Ok( ForceFeedbackEffectId( id as i16 ) )
    }

    pub fn erase_force_feedback_effect( &self, id: ForceFeedbackEffectId ) -> Result< (), nix::Error > {
        unsafe {
            input_sys::evdev_stop_force_feedback( self.fp.as_raw_fd(), id.0 as _ )?;
        }

        Ok(())
    }

    pub fn enable_force_feedback_effect( &self, effect_id: ForceFeedbackEffectId, cycle_count: i32 ) -> Result< (), io::Error > {
        self.emit( InputEventBody::Other {
            kind: EventKind::ForceFeedback,
            code: effect_id.0 as u16,
            value: cycle_count
        })
    }

    pub fn disable_force_feedback_effect( &self, effect_id: ForceFeedbackEffectId ) -> Result< (), io::Error > {
        self.emit( InputEventBody::Other {
            kind: EventKind::ForceFeedback,
            code: effect_id.0 as u16,
            value: 0
        })
    }

    /// Grabs the device for exclusive access.
    ///
    /// No one else will receive any events from it.
    pub fn grab( &self ) -> Result< (), nix::Error > {
        unsafe {
            input_sys::evdev_grab_or_release( self.fp.as_raw_fd(), 1 )?;
        }

        Ok(())
    }

    /// Releases the device from exclusive access.
    pub fn release( &self ) -> Result< (), nix::Error > {
        unsafe {
            input_sys::evdev_grab_or_release( self.fp.as_raw_fd(), 0 )?;
        }

        Ok(())
    }

    /// Emits a given event just as if it was sent by the device itself.
    ///
    /// Makes sense only when the device is *not* grabbed for exclusive access.
    pub fn emit< T >( &self, body: T ) -> Result< (), io::Error > where T: AsRef< InputEventBody > {
        emit_into( &self.fp, body )
    }
}

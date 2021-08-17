use {
    std::{
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
        time::{
            Duration
        },
        path::{
            PathBuf
        }
    },

    crate::{
        input::{
            DeviceId,
            EventBit,
            ForceFeedbackEffect,
            InputEventBody,
            emit_into
        },
        input_sys::{
            EventKind,
            RawAbsInfo,
            RawForceFeedbackEffect
        },
        uinput_sys::{
            self,
            RawAbsSetup,
            RawForceFeedbackErase,
            RawForceFeedbackUpload,
            RawDeviceSetup
        },
        utils::{
            ioctl_get_string
        }
    }
};

#[derive(Debug)]
pub enum DeviceCreateError {
    DeviceNameTooLong,
    DeviceSetupFailed( nix::Error ),
    DeviceCreateFailed( nix::Error ),
    IoFailure( io::Error )
}

pub struct ForceFeedbackEffectUpload< 'a > {
    device: &'a VirtualDevice,
    raw: RawForceFeedbackUpload,
    is_finished: bool
}

impl< 'a > ForceFeedbackEffectUpload< 'a > {
    pub fn effect_id( &self ) -> u16 {
        self.raw.effect.id as u16
    }

    pub fn raw_effect( &self ) -> RawForceFeedbackEffect {
        self.raw.effect.clone()
    }

    pub fn effect( &self ) -> ForceFeedbackEffect {
        unsafe {
            ForceFeedbackEffect::from_raw( &self.raw.effect )
        }
    }

    pub fn complete( mut self ) -> Result< (), nix::Error > {
        self.finish()
    }

    fn finish( &mut self ) -> Result< (), nix::Error > {
        if self.is_finished {
            return Ok(());
        }
        self.is_finished = true;

        unsafe {
            uinput_sys::end_force_feedback_upload( self.device.fp.as_raw_fd(), &mut self.raw )?;
        }

        Ok(())
    }
}

impl< 'a > Drop for ForceFeedbackEffectUpload< 'a > {
    fn drop( &mut self ) {
        let _ = self.finish();
    }
}

pub struct ForceFeedbackEffectErase< 'a > {
    device: &'a VirtualDevice,
    raw: RawForceFeedbackErase,
    is_finished: bool
}

impl< 'a > ForceFeedbackEffectErase< 'a > {
    pub fn effect_id( &self ) -> u16 {
        self.raw.effect_id as u16
    }

    pub fn complete( mut self ) -> Result< (), nix::Error > {
        self.finish()
    }

    fn finish( &mut self ) -> Result< (), nix::Error > {
        if self.is_finished {
            return Ok(());
        }
        self.is_finished = true;

        unsafe {
            uinput_sys::end_force_feedback_erase( self.device.fp.as_raw_fd(), &mut self.raw )?;
        }

        Ok(())
    }
}

impl< 'a > Drop for ForceFeedbackEffectErase< 'a > {
    fn drop( &mut self ) {
        let _ = self.finish();
    }
}

pub enum ForceFeedbackRequest< 'a > {
    Upload( ForceFeedbackEffectUpload< 'a > ),
    Erase( ForceFeedbackEffectErase< 'a > ),
    Enable {
        /// The ID of the effect to enable.
        effect_id: u16,
        /// The number of times the effect should be run.
        cycle_count: i32
    },
    Disable {
        /// The ID of the effect to disable.
        effect_id: u16
    },
    Other {
        code: u16,
        value: i32
    }
}

pub struct VirtualDevice {
    fp: File
}

impl VirtualDevice {
    pub fn create< I >( id: DeviceId, name: &str, event_bits: I ) -> Result< Self, DeviceCreateError >
        where I: IntoIterator< Item = EventBit >
    {
        if name.len() >= 80 {
            return Err( DeviceCreateError::DeviceNameTooLong );
        }

        let fp = fs::OpenOptions::new()
            .read( true )
            .write( true )
            .create( false )
            .open( "/dev/uinput" ).map_err( DeviceCreateError::IoFailure )?;

        let mut has_event_key = false;
        let mut has_event_relative_axis = false;
        let mut has_event_absolute_axis = false;
        let mut has_event_force_feedback = false;

        for event_bit in event_bits {
            match event_bit {
                EventBit::Key( key ) => {
                    has_event_key = true;
                    unsafe {
                        uinput_sys::device_set_key_bit( fp.as_raw_fd(), key.raw() as _ )
                    }.unwrap();
                },
                EventBit::RelativeAxis( axis ) => {
                    has_event_relative_axis = true;
                    unsafe {
                        uinput_sys::device_set_relative_axis_bit( fp.as_raw_fd(), axis.raw() as _ )
                    }.unwrap();
                },
                EventBit::AbsoluteAxis( descriptor ) => {
                    has_event_absolute_axis = true;
                    unsafe {
                        uinput_sys::device_set_absolute_axis_bit( fp.as_raw_fd(), descriptor.axis.raw() as _ )
                    }.unwrap();

                    assert!( descriptor.maximum >= descriptor.minimum );

                    let abs_setup = RawAbsSetup {
                        axis: descriptor.axis.raw(),
                        info: RawAbsInfo {
                            value: (descriptor.maximum - descriptor.minimum) / 2 + descriptor.minimum,
                            minimum: descriptor.minimum,
                            maximum: descriptor.maximum,
                            noise_threshold: descriptor.noise_threshold,
                            deadzone: descriptor.deadzone,
                            resolution: descriptor.resolution
                        }
                    };

                    unsafe {
                        uinput_sys::abs_setup( fp.as_raw_fd(), &abs_setup )
                    }.unwrap();
                },
                EventBit::ForceFeedback( bit ) => {
                    has_event_force_feedback = true;
                    unsafe {
                        uinput_sys::device_set_force_feedback_bit( fp.as_raw_fd(), bit.raw() as _ )
                    }.unwrap();
                }
            }
        }

        if has_event_key {
            unsafe {
                uinput_sys::device_set_event_bit( fp.as_raw_fd(), EventKind::Key.raw() as _ )
            }.unwrap();
        }

        if has_event_relative_axis {
            unsafe {
                uinput_sys::device_set_event_bit( fp.as_raw_fd(), EventKind::RelativeAxis.raw() as _ )
            }.unwrap();
        }

        if has_event_absolute_axis {
            unsafe {
                uinput_sys::device_set_event_bit( fp.as_raw_fd(), EventKind::AbsoluteAxis.raw() as _ )
            }.unwrap();
        }

        if has_event_force_feedback {
            unsafe {
                uinput_sys::device_set_event_bit( fp.as_raw_fd(), EventKind::ForceFeedback.raw() as _ )
            }.unwrap();
        }

        let mut setup = RawDeviceSetup {
            id: id.into(),
            name: [0; 80],
            force_feedback_effects_max: if has_event_force_feedback { 1 } else { 0 }
        };

        setup.name[ 0..name.len() ].copy_from_slice( name.as_bytes() );

        unsafe {
            uinput_sys::device_setup( fp.as_raw_fd(), &setup )
        }.map_err( DeviceCreateError::DeviceSetupFailed )?;

        unsafe {
            uinput_sys::device_create( fp.as_raw_fd() )
        }.map_err( DeviceCreateError::DeviceCreateFailed )?;

        let device = VirtualDevice {
            fp
        };

        Ok( device )
    }

    fn sysname( &self ) -> Result< String, nix::Error > {
        unsafe {
            ioctl_get_string( self.fp.as_raw_fd(), b'U', 44 )
        }
    }

    pub fn path( &self ) -> Result< PathBuf, nix::Error > {
        let sysname = self.sysname()?;
        let dir_path = format!( "/sys/devices/virtual/input/{}", sysname );
        let dir = fs::read_dir( dir_path ).unwrap();
        for entry in dir {
            let entry = entry.unwrap();
            if !entry.path().is_dir() {
                continue;
            }

            let path = entry.path();
            if path.file_name().unwrap().to_str().unwrap().starts_with( "event" ) {
                let output: PathBuf = "/dev/input".into();
                let output = output.join( path.file_name().unwrap() );
                return Ok( output );
            }
        }

        unreachable!();
    }

    pub fn poll_force_feedback( &self, timeout: Option< Duration > ) -> Result< Option< ForceFeedbackRequest >, io::Error > {
        match crate::input::read_raw_input_event( &self.fp, timeout )? {
            Some( event ) if event.kind == uinput_sys::EV_UINPUT && event.code == uinput_sys::UI_FF_UPLOAD => {
                let upload = unsafe {
                    let mut upload = std::mem::MaybeUninit::< RawForceFeedbackUpload >::zeroed();
                    (*upload.as_mut_ptr()).request_id = event.value as u32;
                    uinput_sys::begin_force_feedback_upload( self.fp.as_raw_fd(), upload.as_mut_ptr() )
                        .map_err( |error| io::Error::new( io::ErrorKind::Other, error ) )?;
                    upload.assume_init()
                };

                let request = ForceFeedbackRequest::Upload( ForceFeedbackEffectUpload {
                    device: self,
                    raw: upload,
                    is_finished: false
                });

                Ok( Some( request ) )
            },
            Some( event ) if event.kind == uinput_sys::EV_UINPUT && event.code == uinput_sys::UI_FF_ERASE => {
                let mut erase = RawForceFeedbackErase {
                    request_id: event.value as u32,
                    return_value: 0,
                    effect_id: 0
                };

                unsafe {
                    uinput_sys::begin_force_feedback_erase( self.fp.as_raw_fd(), &mut erase )
                        .map_err( |error| io::Error::new( io::ErrorKind::Other, error ) )?;
                }

                let request = ForceFeedbackRequest::Erase( ForceFeedbackEffectErase {
                    device: self,
                    raw: erase,
                    is_finished: false
                });

                Ok( Some( request ) )
            },
            Some( event ) if event.kind == EventKind::ForceFeedback.raw() => {
                let event = if event.code < crate::input_sys::FF_GAIN {
                    if event.value > 0 {
                        ForceFeedbackRequest::Enable {
                            effect_id: event.code as _,
                            cycle_count: event.value
                        }
                    } else {
                        ForceFeedbackRequest::Disable {
                            effect_id: event.code as _
                        }
                    }
                } else {
                    ForceFeedbackRequest::Other {
                        code: event.code,
                        value: event.value
                    }
                };

                Ok( Some( event ) )
            },
            Some( event ) => unreachable!( "unknown event kind: {}", event.kind ),
            _ => Ok( None )
        }
    }

    /// Emits an event into the device.
    ///
    /// You can also pass a whole `InputEvent` here, however
    /// the timestamp will be ignored.
    ///
    /// The events are buffered and will not be sent immediately;
    /// you need to send `InputEventBody::Flush` to flush them.
    pub fn emit< T >( &self, body: T ) -> Result< (), io::Error > where T: AsRef< InputEventBody > {
        emit_into( &self.fp, body )
    }
}

impl Drop for VirtualDevice {
    fn drop( &mut self ) {
        unsafe {
            let _ = uinput_sys::device_destroy( self.fp.as_raw_fd() );
        }
    }
}

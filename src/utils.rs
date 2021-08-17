use {
    std::{
        os::{
            unix::{
                io::{
                    RawFd
                }
            }
        }
    }
};

pub unsafe fn ioctl_get_string( fd: RawFd, ioctl_id: u8, ioctl_seq: usize ) -> Result< String, nix::Error > {
    let mut buffer = [0; 1024];

    let result = libc::ioctl( fd, request_code_read!( ioctl_id, ioctl_seq, buffer.len() ), buffer.as_mut_ptr() );
    let length = nix::errno::Errno::result( result )?;

    let name = String::from_utf8_lossy( &buffer[ 0..(length as usize) - 1 ] );
    Ok( name.into_owned() )
}

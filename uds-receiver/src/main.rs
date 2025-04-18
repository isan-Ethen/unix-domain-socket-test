use libc::{bind, socket};
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::os::unix::io::{FromRawFd, RawFd};
use std::thread;

type Result<T> = std::result::Result<T, io::Error>;

fn from_syscall_error(error: syscall::Error) -> io::Error {
    io::Error::from_raw_os_error(error.errno as i32)
}

fn listen_gate(path: &str) -> Result<RawFd> {
    println!("make socket");
    let gate = unsafe { socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if gate < 0 {
        return Err(io::Error::last_os_error());
    }

    let c_path = CString::new(path)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains null bytes"))?;

    println!("initialize gate_addr");
    let mut gate_addr: libc::sockaddr_un = unsafe { mem::zeroed() };
    gate_addr.sun_family = libc::AF_UNIX as libc::sa_family_t;

    let path_bytes = c_path.as_bytes_with_nul();

    println!("check len of path");
    if path_bytes.len() > gate_addr.sun_path.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path is too long",
        ));
    }

    println!("write path to gate_addr");
    for (i, &byte) in path_bytes.iter().enumerate() {
        gate_addr.sun_path[i] = byte as libc::c_char;
    }

    println!("bind socket");
    let bind_result = unsafe {
        bind(
            gate,
            &gate_addr as *const _ as *const libc::sockaddr,
            mem::size_of::<libc::sockaddr_un>() as libc::socklen_t,
        )
    };
    println!("bind result: {}", bind_result);
    if bind_result < 0 {
        let err = io::Error::last_os_error();
        unsafe { libc::close(gate) };
        return Err(err);
    }

    Ok(gate)
}

fn main() -> Result<()> {
    let fd_path = "/tmp/uds/test";
    let scheme_path = format!("chan:{}", fd_path);
    println!("scheme path: {}", scheme_path);

    println!("listen gate");
    let receiver_fd = listen_gate(&scheme_path)?;
    println!("accept socket");
    let conn_fd = unsafe { libc::accept(receiver_fd, std::ptr::null_mut(), std::ptr::null_mut()) };
    if conn_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    println!("as raw fd");
    let mut file = unsafe { File::from_raw_fd(conn_fd as RawFd) };

    let mut contents = String::new();
    println!("read to string");
    file.read_to_string(&mut contents)?;

    println!("file contents:\n{}", contents);

    Ok(())
}

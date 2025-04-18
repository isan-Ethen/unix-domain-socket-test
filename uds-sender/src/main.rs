use libc::{connect, socket, write};
use std::ffi::CString;
use std::io::{self};
use std::mem;
use std::os::unix::io::RawFd;

type Result<T> = std::result::Result<T, io::Error>;

fn from_syscall_error(error: syscall::Error) -> io::Error {
    io::Error::from_raw_os_error(error.errno as i32)
}

fn connect_gate(path: &str) -> Result<RawFd> {
    println!("make socket");
    let gate = unsafe { socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if gate < 0 {
        return Err(io::Error::last_os_error());
    }

    let c_path = CString::new(path)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains null bytes"))?;

    println!("initialize gate_addr");
    let mut gate_addr: libc::sockaddr_un = unsafe { mem::zeroed() };
    println!("set sun_family");
    gate_addr.sun_family = libc::AF_UNIX as libc::sa_family_t;

    println!("get path bytes");
    let path_bytes = c_path.as_bytes_with_nul();
    println!("path bytes len: {}", path_bytes.len());

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

    println!("connect socket");
    let connect_result = unsafe {
        connect(
            gate,
            &gate_addr as *const _ as *const libc::sockaddr,
            mem::size_of::<libc::sockaddr_un>() as libc::socklen_t,
        )
    };
    println!("connect result: {}", connect_result);
    if connect_result < 0 {
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

    println!("connect gate");
    let sender_fd = connect_gate(&scheme_path)?;

    println!("send message");
    let message = "hello";
    let res = unsafe {
        write(
            sender_fd.try_into().expect("invalid argument"),
            message.as_ptr() as *const std::os::raw::c_void,
            message.len(),
        )
    };
    println!("res: {}", res);

    Ok(())
}

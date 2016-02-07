extern crate winapi;
extern crate user32;
extern crate kernel32;

use self::winapi::*;

use image;

use super::{HashResult, HashError};

use img::{Image, HashSettings};

use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::IntoRawHandle;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::{io, mem, ptr};

enum Message {
    Load(PathBuf),
    Loaded(PathBuf, Vec<u8>),
    Quit,
}

pub struct WorkManager {
   iocp: HANDLE,
}

impl WorkManager {
    pub fn new(num_threads: usize) -> Self {
        let iocp = unsafe {
            kernel32::CreateIoCompletionPort(INVALID_HANDLE_VALUE, ptr::null_mut(), 0, num_threads as DWORD)
        };

        assert_not_null(iocp);

        WorkManager {
            iocp: iocp
        }
    }

    pub fn enqueue_load(&self, path: PathBuf) {
        send_msg(self.iocp, Message::Load(path));
    }

    pub fn quit(&self) {
        send_msg(self.iocp, Message::Quit);
    }

    pub fn worker(&self, tx: &Sender<HashResult>, hash_cfg: HashSettings) -> Worker {
       Worker {
           iocp: self.iocp,
           tx: tx.clone(),
           hash_cfg: hash_cfg
        }
    }
}

fn send_msg(iocp: HANDLE, msg: Message) {
    let iocp_data_ptr = IocpData::with_message(msg).to_ptr();

    let res = unsafe { kernel32::PostQueuedCompletionStatus(iocp, 0, 0, iocp_data_ptr) };
    assert_nonzero(res);
}

pub struct Worker {
    iocp: HANDLE,
    tx: Sender<HashResult>,
    hash_cfg: HashSettings,
}

unsafe impl Send for Worker {}

impl Worker {
    pub fn work(self) {
        use self::Message::*;

        loop {
            match self.get_message() {
                (Load(path), _, _) => if let Err(err) = self.load(path) {
                    self.tx.send(Err(err)).unwrap();
                },
                (Loaded(path, data), read, hnd) => self.tx.send(self.loaded(path, data, read, hnd)).unwrap(),
                (Quit, _, _) => {
                    send_msg(self.iocp, Message::Quit);
                    break;
                }
            }
        }
    }

    fn load(&self, path: PathBuf) -> Result<(), HashError>  {
        let meta = try_with_path!(path; fs::metadata(&path));

        let len = meta.len();

        let mut open_opts = OpenOptions::new();
        
        open_opts.read(true)
            .flags_and_attributes(FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED);

        let file_hnd = try_with_path!(path; open_opts.open(&path))
            .into_raw_handle();

        let res = unsafe { 
            kernel32::CreateIoCompletionPort(file_hnd, self.iocp, file_hnd as usize as u64, 0)
        };

        assert_not_null(res);

        let mut buf = Vec::with_capacity(len as usize);
        
        let buf_ptr = buf.as_mut_ptr();

        let iocp_data_ptr = IocpData::with_message(Message::Loaded(path, buf)).to_ptr();

        let res = unsafe {
            kernel32::ReadFile(file_hnd, buf_ptr as *mut _, len as DWORD, ptr::null_mut(), iocp_data_ptr)
        };

        assert_nonzero(res);

        Ok(())
    }

    fn loaded(&self, path: PathBuf, mut data: Vec<u8>, read: u64, hnd: HANDLE) -> HashResult {
        let res = unsafe { 
            kernel32::CloseHandle(hnd)
        };

        assert_nonzero(res);

        unsafe {
            data.set_len(read as usize);
        }

        match image::load_from_memory(&data) {
            Ok(img) => Ok(Image::hash(path, img, self.hash_cfg, read)),
            Err(err) => Err(HashError::path_and_err(path, err)),
        }
    }

    fn get_message(&self) -> (Message, u64, HANDLE) {
        let mut bytes_read = 0;
        let mut comp_key = 0;
        let mut iocp_data_ptr: LPOVERLAPPED = ptr::null_mut();

        let iocp_data = unsafe  {
            let res = kernel32::GetQueuedCompletionStatus(self.iocp, &mut bytes_read, &mut comp_key, &mut iocp_data_ptr, INFINITE);
            assert_nonzero(res);

            IocpData::from_ptr(iocp_data_ptr)
        };

        (iocp_data.message, bytes_read as u64, comp_key as usize as HANDLE)
    }
}

fn assert_nonzero(res: BOOL) {
    assert!(res != 0, "Kernel call returned false. Error: {:?}", io::Error::last_os_error());
}

fn assert_not_null<T>(ptr: *mut T) {
    assert!(!ptr.is_null(), "Kernel call returned NULL. Error: {:?}", io::Error::last_os_error());
}


#[repr(C)]
struct IocpData {
    over: OVERLAPPED,
    message: Message,
}

impl IocpData {
    fn with_message(msg: Message) -> Self {
        IocpData {
            over: unsafe { mem::zeroed() },
            message: msg,
        }
    }

    fn to_ptr(self) -> LPOVERLAPPED {
        Box::into_raw(Box::new(self)) as LPOVERLAPPED
    }

    unsafe fn from_ptr(ptr: LPOVERLAPPED) -> Self {
        *Box::from_raw(ptr as *mut Self)
    }
}


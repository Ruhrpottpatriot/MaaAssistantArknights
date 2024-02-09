#![warn(
    clippy::missing_safety_doc,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]

use std::{
    collections::{HashMap, HashSet},
    ffi::{c_char, c_int, c_void, CStr, CString, NulError},
};

include!("./bind.rs");

#[derive(Debug)]
pub enum Error {
    Unknown,
    TooLargeAlloc,
    Null,
    Utf8,
}

impl From<NulError> for Error {
    fn from(_: NulError) -> Self {
        Self::Null
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Utf8
    }
}
#[derive(Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub type_: String,
    pub params: String,
}

#[derive(Debug)]
pub struct Maa {
    handle: AsstHandle,
    uuid: Option<String>,
    target: Option<String>,
    tasks: HashMap<i32, Task>,
}

impl Maa {
    /// Create a new Maa instance
    pub fn new() -> Self {
        let handle = unsafe { AsstCreate() };
        Maa {
            handle,
            uuid: None,
            target: None,
            tasks: HashMap::new(),
        }
    }

    pub fn get_null_size() -> u64 {
        unsafe { AsstGetNullSize() }
    }

    pub fn with_callback_and_custom_arg(
        callback: AsstApiCallback,
        custom_arg: *mut c_void,
    ) -> Self {
        let handle = unsafe { AsstCreateEx(callback, custom_arg) };
        Maa {
            handle,
            uuid: None,
            target: None,
            tasks: HashMap::new(),
        }
    }

    pub fn with_callback(callback: AsstApiCallback) -> Self {
        Self::with_callback_and_custom_arg(callback, std::ptr::null_mut::<c_void>())
    }

    /// The default callback function that just prints the message and the detail json
    pub unsafe extern "C" fn default_callback(
        msg: c_int,
        detail_json: *const c_char,
        _: *mut c_void,
    ) {
        println!(
            "msg:{}: {}",
            msg,
            CStr::from_ptr(detail_json).to_str().unwrap()
        );
    }

    pub fn load_resource(path: &str) -> Result<(), Error> {
        let path = CString::new(path.to_string())?;

        // Safety: The path pointe is guaranteed to be valid and null-terminated since it was
        // created in safe rust with no errors.
        let return_code = unsafe { AsstLoadResource(path.as_ptr()) };
        match return_code {
            1 => Ok(()),
            _ => Err(Error::Unknown),
        }
    }

    /// Gets the current version of the MAA library
    pub fn get_version() -> Result<String, Error> {
        // Safety: The version string pointer is checked to be non-null. However, no
        // guarantees can be made at this point whether the string contains a
        // null-terminator.
        // This block also contains multiple unsafe operations, which we usually want to
        // avoid. However, in this case the second operation directly depends on the first
        // and therefore should be in the same block.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        let version = unsafe {
            let version = AsstGetVersion();
            if version.is_null() {
                return Err(Error::Null);
            }

            CStr::from_ptr(version)
        };

        // Throw an error instead of replacing invalid utf8 characters
        // TODO: Return a reference instead of a cloned string
        // TODO: The returned version is should be a SemVer  struct
        Ok(version.to_str()?.to_string())
    }

    pub fn set_static_option(option: AsstStaticOptionKey, value: &str) -> Result<(), Error> {
        let c_option_value = CString::new(value)?;

        // Safety: The string is guaranteed to be null-terminated and valid since it was
        // created in safe rust with no errors.
        let return_code = unsafe { AsstSetStaticOption(option, c_option_value.as_ptr()) };
        if return_code == 1 {
            Ok(())
        } else {
            Err(Error::Unknown)
        }
    }

    pub fn set_working_directory(path: &str) -> Result<(), Error> {
        let c_path = CString::new(path)?;

        // Safety: The string is guaranteed to be null-terminated and valid since it was
        // created in safe rust with no errors.
        let return_code = unsafe { AsstSetUserDir(c_path.as_ptr()) };
        if return_code == 1 {
            Ok(())
        } else {
            Err(Error::Unknown)
        }
    }

    pub fn set_option(&mut self, option: AsstInstanceOptionKey, value: &str) -> Result<(), Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let c_option_value = CString::new(value)?;

        // Safety:
        // * The handle is never null at this point
        // * The string is guaranteed to be null-terminated and valid since it was
        let return_code =
            unsafe { AsstSetInstanceOption(self.handle, option, c_option_value.as_ptr()) };
        if return_code == 1 {
            Ok(())
        } else {
            Err(Error::Unknown)
        }
    }

    pub fn connect(
        &mut self,
        adb_path: &str,
        address: &str,
        config: Option<String>,
    ) -> Result<i32, Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let c_adb_path = CString::new(adb_path)?;
        let c_address = CString::new(address)?;
        let c_cfg_ptr = match config {
            Some(cfg) => CString::new(cfg)?.as_ptr(),
            None => std::ptr::null::<c_char>(),
        };

        // Safety:
        // * The handle is never null at this point
        // * The strings are guaranteed to be null-terminated and valid since they were
        //   created in safe rust with no errors.
        let return_code = unsafe {
            AsstAsyncConnect(
                self.handle,
                c_adb_path.as_ptr(),
                c_address.as_ptr(),
                c_cfg_ptr,
                1,
            )
        };
        if return_code != 0 {
            self.target = Some(address.to_string());
            Ok(return_code)
        } else {
            Err(Error::Unknown)
        }
    }

    #[deprecated]
    pub fn connect_legacy(
        &mut self,
        adb_path: &str,
        address: &str,
        config: Option<&str>,
    ) -> Result<(), Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let c_adb_path = CString::new(adb_path)?;
        let c_address = CString::new(address)?;
        let c_cfg_ptr = match config {
            Some(cfg) => CString::new(cfg)?.as_ptr(),
            None => std::ptr::null::<c_char>(),
        };

        // Safety:
        // * The handle is never null at this point
        // * The strings are guaranteed to be null-terminated and valid since they were
        //   created in safe rust with no errors.
        let return_code = unsafe {
            AsstConnect(
                self.handle,
                c_adb_path.as_ptr(),
                c_address.as_ptr(),
                c_cfg_ptr,
            )
        };
        if return_code == 1 {
            Ok(())
        } else {
            Err(Error::Unknown)
        }
    }

    /// Check if an instance of MAA is running
    ///
    /// # Safety
    /// This function checks if the handle stored inside `self` is not null. If the handle
    /// is null, the function will return false.
    pub fn running(&self) -> bool {
        if self.handle.is_null() {
            return false;
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstRunning(self.handle) };
        return_code == 1
    }

    /// Clicks on the screen at the given coordinates
    pub fn click(&self, x: i32, y: i32) -> Result<i32, Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstAsyncClick(self.handle, x, y, 0) };
        if return_code != 0 {
            Ok(return_code)
        } else {
            Err(Error::Unknown)
        }
    }

    pub fn screenshot(&self) -> Result<Vec<u8>, Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let mut buff_size = 2 * 1920 * 1080 * 4;
        loop {
            if buff_size > 10 * 1920 * 1080 * 4 {
                return Err(Error::TooLargeAlloc);
            }
            let mut buff: Vec<u8> = Vec::with_capacity(buff_size);

            // Safety: The handle is never null at this point and the buffer is guaranteed
            // to be valid
            let data_size = unsafe {
                AsstGetImage(
                    self.handle,
                    buff.as_mut_ptr() as *mut c_void,
                    buff_size as u64,
                )
            };
            if data_size == Self::get_null_size() {
                buff_size *= 2;
                continue;
            }

            let data_size = data_size as usize;
            if data_size > buff.capacity() {
                return Err(Error::TooLargeAlloc);
            }

            // Safety: The new length of the buffer is guaranteed to be a valid size
            unsafe { buff.set_len(data_size as usize) };
            buff.resize(data_size as usize, 0);
            return Ok(buff);
        }
    }

    pub fn take_screenshot(&mut self) -> Result<(), Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstAsyncScreencap(self.handle, 1) };
        match return_code {
            0 => Err(Error::Unknown),
            _ => Ok(()),
        }
    }

    pub fn create_task(&mut self, type_: &str, params: &str) -> Result<AsstTaskId, Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let c_type = CString::new(type_)?;
        let c_params = CString::new(params)?;

        // Safety: The handle is never null at this point and the strings are guaranteed
        // to be valid and null-terminated
        let task_id = unsafe { AsstAppendTask(self.handle, c_type.as_ptr(), c_params.as_ptr()) };
        self.tasks.insert(
            task_id,
            Task {
                id: task_id,
                type_: type_.to_string(),
                params: params.to_string(),
            },
        );

        Ok(task_id)
    }

    pub fn set_task(&self, id: i32, params: &str) -> Result<(), Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let c_params = CString::new(params)?;

        // Safety: The handle is never null at this point and the string is guaranteed to
        // be valid and null-terminated
        let return_code = unsafe { AsstSetTaskParams(self.handle, id, c_params.as_ptr()) };
        match return_code {
            1 => Ok(()),
            _ => Err(Error::Unknown),
        }
    }

    pub fn get_uuid(&mut self) -> Result<String, Error> {
        if let Some(uuid) = self.uuid.clone() {
            return Ok(uuid);
        };

        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let mut buff_size = 1024;
        loop {
            if buff_size > 1024 * 1024 {
                return Err(Error::TooLargeAlloc);
            }
            let mut buff: Vec<u8> = Vec::with_capacity(buff_size);

            // Safety:
            // * The handle is never null at this point
            // * The buffer is guaranteed to be valid
            let data_size =
                unsafe { AsstGetUUID(self.handle, buff.as_mut_ptr() as *mut i8, buff_size as u64) };
            if data_size == Self::get_null_size() {
                buff_size *= 2;
                continue;
            }

            let data_size = data_size as usize;
            if data_size > buff.capacity() {
                return Err(Error::TooLargeAlloc);
            }

            // Safety: The new length of the buffer is guaranteed to be a valid size
            unsafe { buff.set_len(data_size) };
            let ret = String::from_utf8_lossy(&buff).to_string();
            self.uuid = Some(ret.clone());
            return Ok(ret);
        }
    }

    pub fn get_target(&self) -> Option<String> {
        // TODO: Give out a reference
        self.target.clone()
    }

    /// Gets a list of tasks that are currently configured
    pub fn get_tasks(&mut self) -> Result<&HashMap<i32, Task>, Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        let mut buff_size = 1024;
        loop {
            if buff_size > 1024 * 1024 {
                return Err(Error::TooLargeAlloc);
            }
            let mut buff: Vec<i32> = Vec::with_capacity(buff_size);

            // Safety:
            // * The handle is never null at this point
            // * The buffer is guaranteed to be valid
            let data_size =
                unsafe { AsstGetTasksList(self.handle, buff.as_mut_ptr(), buff_size as u64) };

            if data_size == Self::get_null_size() {
                buff_size *= 2;
                continue;
            }

            let data_size = data_size as usize;
            if data_size > buff.capacity() {
                return Err(Error::TooLargeAlloc);
            }

            // Safety: The new length of the buffer is guaranteed to be a valid size
            unsafe { buff.set_len(data_size) };

            buff.resize(data_size as usize, 0);

            let mut task_ids = HashSet::with_capacity(buff.len());

            for i in buff {
                task_ids.insert(i);
            }

            self.tasks.retain(|k, _| task_ids.contains(k));

            return Ok(&self.tasks);
        }
    }

    /// Starts the configured tasks
    pub fn start(&self) -> Result<(), Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstStart(self.handle) };
        match return_code {
            1 => Ok(()),
            _ => Err(Error::Unknown),
        }
    }

    /// Stops the configured tasks
    pub fn stop(&self) -> Result<(), Error> {
        if self.handle.is_null() {
            return Err(Error::Null);
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstStop(self.handle) };
        match return_code {
            1 => Ok(()),
            _ => Err(Error::Unknown),
        }
    }

    /// Passes the given log message to the underlying MAA instance
    pub fn log(level_str: &str, message: &str) -> Result<(), Error> {
        let c_level_str = CString::new(level_str)?;
        let c_message = CString::new(message)?;

        // Safety: The strings are guaranteed to be null-terminated and valid since they
        // were created in safe code with no errors.
        unsafe {
            AsstLog(c_level_str.as_ptr(), c_message.as_ptr());
        }

        Ok(())
    }
}

impl Drop for Maa {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }

        // Safety: The handle is never null at this point
        unsafe { AsstDestroy(self.handle) }
    }
}

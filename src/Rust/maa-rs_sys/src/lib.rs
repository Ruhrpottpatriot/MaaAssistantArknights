//! maa-rs_sy is the main crate the provides rust bindings to the exported
//! interface provided by MAA. It aims to give users an _unsafe-free_ api they
//! can use in their Rust applications.
//!
//! This create also contains wrappers around most types to make them secure and
//! add more features to them that are interesting to Rust users, e.g. numerical
//! id's, such as being equatable, comparable and cloneable, which
//! the original FFI type might not be.

#![warn(
    clippy::missing_safety_doc,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]

/// This module just includes the bindings file, so we can control the visibility of said
/// bindings and not have them pollute the public namespace of the crate. The reason for
/// this is rust-bindgen's way of generating C-bindings, which are *pub* by default. Since
/// this could potentially leak unsafe types, we have to make sure that none of those
/// appear in the public API of this crate.
pub(crate) mod bind;
use bind::*;

use derive_more::Display;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    ffi::{c_char, c_void, CStr, CString, NulError},
    net::SocketAddr,
    path::Path,
};
use strum::EnumString;

/// Makes sure that as soon as a result without an error is used, the crate's
/// error type is used.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An Unknown error has occurred")]
    Unknown,

    #[error("Could not allocate such a large buffer")]
    TooLargeAlloc,

    #[error("The handle to the backend is invalid")]
    InvalidHandle,

    #[error(transparent)]
    CNull(#[from] NulError),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Allocation failed in the backend")]
    AllocFailed,

    #[error("The given contained non UTF-8 chars")]
    InvalidPath,

    #[error("Could not serialize object to JSON")]
    Serialize(#[from] serde_json::Error),
}

/// Represents the id of an asynchronous call
pub struct AsyncCallId(pub(crate) bind::AsstAsyncCallId);

impl AsyncCallId {
    /// Creates a new [`AsyncCallId`]
    ///
    /// # Parameters
    /// * `id`:  The numerical id of the asynchronous call
    pub fn new(id: i32) -> Self {
        Self(id)
    }
}

/// Represents the key used to identify an option value
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Display, Serialize)]
#[repr(transparent)]
pub struct OptionKey(pub(crate) bind::AsstStaticOptionKey);

impl OptionKey {
    /// Creates a new [`OptionKey`]
    ///
    /// # Parameters
    /// * `id`:  The numerical id of the option
    pub fn new(id: i32) -> Self {
        Self(id)
    }
}

/// Represents the type of a message.
///
/// TODO: Convert the message type to a Rust enum
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Display, Serialize)]
#[repr(transparent)]
pub struct MessageType(pub(crate) bind::AsstMsgId);

impl MessageType {
    /// Creates a new [`MessageId`]
    ///
    /// # Parameters
    /// * `id`:  The numerical id of the message
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn to_be_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

/// Represents the ID of a task.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Display, Serialize)]
#[repr(transparent)]
pub struct TaskId(pub(crate) bind::AsstTaskId);

impl TaskId {
    /// Creates a new [`TaskId`]
    ///
    /// # Parameters
    /// * `id`:  The numerical id of the task
    pub fn new(id: i32) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,

    /// The type of the task
    /// TODO: Convert the task type to a Rust enum
    pub task_type: String,

    /// The parameters for the task, serialized as a JSON string
    pub params: String,
}

/// The callback function that is called every time a message occurs in the backend
type Callback = fn(MessageType, &str);

#[derive(Debug)]
pub struct Assistant {
    handle: bind::AsstHandle,
    uuid: Option<String>,
    target: Option<SocketAddr>,
    tasks: HashMap<TaskId, Task>,

    /// A raw pointer to the callback function presented duting construction. This field
    /// should NEVER be used, except for passing it to the backend and when dropping the
    /// instance!
    callback_ptr: Option<*mut Callback>,
}

impl Drop for Assistant {
    fn drop(&mut self) {
        if self.handle.is_null() {
            return;
        }

        // Safety: The handle is never null at this point. The same goes for the callback
        // pointer, which we want to get back from a raw pointer so it can be sasfely
        // dropped and the memory doesn't leak.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        unsafe {
            AsstDestroy(self.handle);

            if let Some(callback_ptr) = self.callback_ptr {
                let _ = Box::from_raw(callback_ptr);
            }
        }
    }
}

impl Assistant {
    /// Create a new MAA assistant instance
    ///
    /// This function takes a callback, which moved into the function and then boxed so it
    /// can be passed to the backend via pointer. It's important to note, that internally
    /// the callback function is converted into a raw void pointer and then passed as the
    /// `custom_arg` parameter to the backend. This is done to allow any rust function,
    /// including closures, to be passed to the backend. To make sure the callback
    /// function is properly called and satisfies Rust's type system, a trampoline
    /// function is utilized that wraps the rust callback. See [`Self::trampoline`] for
    /// more details.
    ///
    /// # Parameters
    /// * `callback` - An optional function that is called every time a message occurs in
    ///   the backend.
    pub fn new(callback: Option<Callback>) -> Result<Self> {
        let (handle, callback_ptr) = match callback {
            Some(callback) => {
                let args = Box::into_raw(Box::new(callback));

                // Safety: This allocation might return a null pointer
                let handlef =
                    unsafe { AsstCreateEx(Some(Self::trampoline::<Callback>), args as *mut _) };
                (handlef, Some(args))
            }
            None => {
                // Safety: This allocation might return a null pointer
                let handle = unsafe { AsstCreate() };
                (handle, None)
            }
        };

        if handle.is_null() {
            return Err(Error::AllocFailed);
        }

        Ok(Assistant {
            handle,
            uuid: None,
            target: None,
            tasks: HashMap::new(),
            callback_ptr,
        })
    }

    /// The default callback function that just prints the message and the detail json
    pub fn default_callback(msg_id: MessageType, detail_json: &str) {
        println!("msg: {}: {}", msg_id, detail_json,);
    }

    /// Loads a resource from the given path
    ///
    /// Since paths on Windows might contain non UTF-8 characters, this function needs to
    /// covert the path into bytes first, which might fail. On the other hand convert the
    /// path to a C string. This is pretty straightworard on UNIX based systems, since
    /// there paths are just a sequence of bytes. However, on Windows paths are either
    /// passed as ASCII or UTF-16. Passing UTF-8 paths to the system is not supported.
    /// Therefore we have to make a distinction between windows and unix, which means a
    /// slight performance hit on non-UNIX systems. See:
    /// * https://stackoverflow.com/questions/38948669/
    /// * https://internals.rust-lang.org/t/pathbuf-to-cstring/12560
    pub fn load_resource<P: AsRef<Path>>(path: P) -> Result<()> {
        #[cfg(unix)]
        fn to_bytes<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
            use std::os::unix::ffi::OsStrExt;
            Some(path.as_ref().as_os_str().as_bytes().to_vec())
        }

        #[cfg(not(unix))]
        fn path_to_bytes<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
            // On Windows, could use std::os::windows::ffi::OsStrExt to encode_wide(),
            // but you end up with a Vec<u16> instead of a Vec<u8>, so that doesn't
            // really help.
            path.as_ref()
                .to_str()
                .map(|s| s.as_bytes())
                .map(|b| b.to_vec())
        }

        let path = path_to_bytes(path).ok_or(Error::InvalidPath)?;
        let path = CString::new(path)?;

        // Safety: The path pointer is guaranteed to be valid and null-terminated
        let return_code = unsafe { AsstLoadResource(path.as_ptr()) };
        match return_code {
            1 => Ok(()),
            _ => Err(Error::Unknown),
        }
    }

    pub fn set_static_option(option: OptionKey, value: &str) -> Result<()> {
        let c_option_value = CString::new(value)?;

        // Safety: The string is guaranteed to be null-terminated and valid since it was
        // created in safe rust with no errors.
        let return_code = unsafe { AsstSetStaticOption(option.0, c_option_value.as_ptr()) };
        if return_code == 1 {
            Ok(())
        } else {
            Err(Error::Unknown)
        }
    }

    pub fn set_working_directory(path: &str) -> Result<()> {
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

    pub fn set_option(&mut self, option: OptionKey, value: &str) -> Result<()> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        let value = CString::new(value)?;

        // Safety:
        // * The handle is never null at this point
        // * The string is guaranteed to be null-terminated and valid since it was
        let return_code = unsafe { AsstSetInstanceOption(self.handle, option.0, value.as_ptr()) };
        is_success(return_code)
    }

    /// Asynchronously connects to an emulator at the given address
    ///
    /// # Parameters
    /// * `adb_path` - The path to the adb executable
    /// * `address` - The address of the emulator to connect to
    /// * `config` - The configuration to use for the connection
    #[deprecated(
        since = "0.1.0",
        note = "This function is deprecated and will be removed in the next major version. Use `connect_async` instead."
    )]
    pub fn connect<S: Into<String>>(
        &mut self,
        adb_path: S,
        address: SocketAddr,
        config: Option<S>,
    ) -> Result<()> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        let c_adb_path = CString::new(adb_path.into())?;

        let c_address = CString::new(address.to_string())?;
        let c_cfg_ptr = match config {
            Some(cfg) => CString::new(cfg.into())?.as_ptr(),
            None => std::ptr::null::<c_char>(),
        };

        // Safety:
        // * The handle is never null at this point
        // * The strings are guaranteed to be null-terminated and valid since they were
        //   created in safe rust with no errors.
        let result_code = unsafe {
            AsstConnect(
                self.handle,
                c_adb_path.as_ptr(),
                c_address.as_ptr(),
                c_cfg_ptr,
            )
        };
        is_success(result_code)?;

        self.target = Some(address);
        Ok(())
    }

    /// Asynchronously connects to an emulator at the given address
    ///
    /// # Parameters
    /// * `adb_path` - The path to the adb executable
    /// * `address` - The address of the emulator to connect to
    /// * `config` - The configuration to use for the connection
    /// * `block` - If true, the function will block until the connection is established
    ///
    /// # Returns
    /// An [`AsyncCallId`] that can be used to identify the asynchronous call
    pub fn connect_async<S: Into<String>>(
        &mut self,
        adb_path: S,
        address: SocketAddr,
        config: Option<S>,
        block: bool,
    ) -> Result<AsyncCallId> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        let c_adb_path = CString::new(adb_path.into())?;

        let c_address = CString::new(address.to_string())?;
        let c_cfg_ptr = match config {
            Some(cfg) => CString::new(cfg.into())?.as_ptr(),
            None => std::ptr::null::<c_char>(),
        };

        // Safety:
        // * The handle is never null at this point
        // * The strings are guaranteed to be null-terminated and valid since they were
        //   created in safe rust with no errors.
        let async_call_id = unsafe {
            AsstAsyncConnect(
                self.handle,
                c_adb_path.as_ptr(),
                c_address.as_ptr(),
                c_cfg_ptr,
                block.into(),
            )
        };
        self.target = Some(address);
        Ok(AsyncCallId(async_call_id))
    }
    }

    /// Check if an instance of MAA is running
    pub fn running(&self) -> bool {
        if self.handle.is_null() {
            return false;
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstRunning(self.handle) };
        return_code == 1
    }

    /// Clicks on the screen at the given coordinates
    ///
    /// # Parameters
    /// * `x` - The x coordinate of the click
    /// * `y` - The y coordinate of the click
    /// * `block` - If true, the function will block until the click is done.
    ///
    /// # Returns
    /// An [`AsyncCallId`] that can be used to identify the asynchronous call
    pub fn click_async(&self, x: i32, y: i32, block: bool) -> Result<AsyncCallId> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        // Safety: The handle is never null at this point
        let async_call_id = unsafe { AsstAsyncClick(self.handle, x, y, block.into()) };
        Ok(AsyncCallId(async_call_id))
    }
    pub fn screenshot(&self) -> Result<Vec<u8>> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        let mut buff_size = 2 * 1920 * 1080 * 4;
        loop {
            // This is the maximum size of out buffer and always << 2^32, so it will never
            // overflow, even if we're using usize on x86
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
                ) as usize
            };
            if !is_valid_size(data_size) {
                buff_size *= 2;
                continue;
            }

            // Safety: The new length of the buffer is guaranteed to be a valid size
            unsafe { buff.set_len(data_size) };
            buff.resize(data_size, 0);
            return Ok(buff);
        }
    }

    /// Takes a screenshot
    ///
    /// # Parameters
    /// * `block` - If true, the function will block until the screenshot is taken
    ///
    /// # Returns
    /// An [`AsyncCallId`] that can be used to identify the asynchronous call
    pub fn take_screenshot_async(&mut self, block: bool) -> Result<AsyncCallId> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        // Safety: The handle is never null at this point
        let async_call_id = unsafe { AsstAsyncScreencap(self.handle, block.into()) };
        Ok(AsyncCallId(async_call_id))
    }

    /// Creates a new task and appends it to the list of tasks
    ///
    /// # Parameters
    /// * `task_type` - The type of the task to create
    /// * `params` - The parameters for the task
    pub fn create_task<S: Into<String>, T: Serialize>(
        &mut self,
        task_type: S,
        params: &T,
    ) -> Result<TaskId> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        let task_type = task_type.into();

        let type_ = CString::new(task_type.clone())?.as_ptr();
        let params_json = serde_json::to_string(params)?;

        // TODO: Find a better way than stupidly cloning the string
        // If the AsstAppendTask function doesn't take ownership of the string, then we
        // can get the original string back from it
        let params = CString::new(params_json.clone())?.as_ptr();

        // Safety: The handle is never null at this point and the strings are guaranteed
        // to be valid and null-terminated
        let task_id = unsafe { AsstAppendTask(self.handle, type_, params) };
        let task_id = TaskId(task_id);
        self.tasks.insert(
            task_id,
            Task {
                id: task_id,
                task_type: task_type,
                params: params_json.to_string(),
            },
        );

        Ok(task_id)
    }

    /// Sets the parameters for a task
    ///
    /// Parameters for a task can be set at any time, but some parameters are only
    /// evaluated at the start of the task. These parameters are marked with the "Editing
    /// in run-time is not supported" comment in the MAA documentation and any change to
    /// them that occurs during the runtime of a task will be ignored.
    ///
    /// # Parameters
    /// * `id` - The id of the task to set the parameters for
    /// * `params` - The parameters to set.
    pub fn set_task_parameters<T>(&self, id: TaskId, params: &T) -> Result<()>
    where
        T: Serialize,
    {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        let params = serde_json::to_string(params)?;
        let params = CString::new(params)?.as_ptr();

        // Safety: The handle is never null at this point and the string is guaranteed to
        // be valid and null-terminated
        let return_code = unsafe { AsstSetTaskParams(self.handle, id.0, params) };
        is_success(return_code)
    }
    pub fn uuid(&mut self) -> Result<&str> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        if self.uuid.is_none() {
            let mut buff_size = 1024;
            loop {
                if buff_size > 1024 * 1024 {
                    return Err(Error::TooLargeAlloc);
                }
                let mut buff: Vec<u8> = Vec::with_capacity(buff_size);

                // Safety:
                // * The handle is never null at this point
                // * The buffer is guaranteed to be valid
                let data_size = unsafe {
                    AsstGetUUID(self.handle, buff.as_mut_ptr() as *mut i8, buff_size as u64)
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
                unsafe { buff.set_len(data_size) };
                self.uuid = Some(String::from_utf8_lossy(&buff).to_string());
            }
        }

        Ok(self.uuid.as_deref().expect("The uuid was set just above and therefore is guaranteed to be valid."))
    }

    /// Get the current target address
    ///
    /// The target address is usually an IP, but can be anything
    pub fn target(&self) -> Option<SocketAddr> {
        self.target
    }

    /// Gets a list of tasks that are currently configured
    pub fn tasks(&mut self) -> Result<&HashMap<TaskId, Task>> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
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
            let data_size = unsafe {
                AsstGetTasksList(self.handle, buff.as_mut_ptr(), buff_size as u64) as usize
            };

            if !is_valid_size(data_size) {
                // TODO: Replace the growing factor with 1.5
                buff_size *= 2;
                continue;
            }

            // Safety: The new length of the buffer is guaranteed to be a valid size
            unsafe { buff.set_len(data_size) };

            buff.resize(data_size, 0);

            let mut task_ids = HashSet::with_capacity(buff.len());

            for i in buff {
                let i = TaskId(i);
                task_ids.insert(i);
            }

            // Update cached tasks
            self.tasks.retain(|k, _| task_ids.contains(k));

            return Ok(&self.tasks);
        }
    }

    /// Starts the list of configured tasks
    pub fn start(&self) -> Result<()> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstStart(self.handle) };
        is_success(return_code)
    }

    /// Stops the currently running and all following tasks
    pub fn stop(&self) -> Result<()> {
        if self.handle.is_null() {
            return Err(Error::InvalidHandle);
        }

        // Safety: The handle is never null at this point
        let return_code = unsafe { AsstStop(self.handle) };
        is_success(return_code)
    }

    /// Wraps the Rust callback so that it can be passed over C-FFI bounds to the backend
    ///
    /// This trampoline function is necessary so the rust callback function can be called
    /// without issues. It's an `extern "C"` function that takes the raw pointer of the
    /// callback as well as the arguments for said callback. Internally, it converts the
    /// raw pointer fist into a raw pointer to a function that has the signature of the
    /// callback and then in a second step converts the function pointer into a mutable
    /// reference to the callback.
    ///
    /// Since the callback function in this case expects a `&str` as second parameter, it
    /// also converts the C string into a [`CStr`] and then into a Rust `&str`. However,
    /// currently the conversion is expected to not fail, and therefore strings with
    /// invalid UTF-8 characters will cause a panic. A proper approach would either return
    /// a [`Resul<T, E>`] or set the parameter to a sane default like an empty string. A
    /// third approach could use an error flag that can be checked by the caller.
    unsafe extern "C" fn trampoline<F>(i: bind::AsstMsgId, data: *const c_char, ctx: *mut c_void)
    where
        F: FnMut(MessageType, &str) + 'static,
    {
        assert!(!ctx.is_null());

        let callback_ptr = ctx as *mut F;
        let callback = &mut *callback_ptr;

        let rust_str = CStr::from_ptr(data)
            .to_str()
            .expect("Failed to convert C string to Rust string");

        // The AsstMsgId is just a typedef, but we have a newtype
        let i = MessageType(i);

        callback(i, rust_str);
    }
}

/// Loads a resource from the given path
pub fn load_resource<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path_to_bytes(path).ok_or(Error::InvalidPath)?;
    let path = CString::new(path)?;

    // Safety: The path pointer is guaranteed to be valid and null-terminated
    let return_code = unsafe { AsstLoadResource(path.as_ptr()) };
    is_success(return_code)
}

/// Gets the current version of the MAA library
///
/// # Examples
/// ```rust, ignore
/// use maa_rs_sys::get_version;
///
/// let version = get_version().unwrap();
/// println!("The version of the MAA library is: {}", version);
/// ```
pub fn get_version<'a>() -> Result<&'a str> {
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
            return Err(Error::InvalidHandle);
        }

        CStr::from_ptr(version)
    };

    // Throw an error instead of replacing invalid utf8 characters
    // TODO: The returned version is should be a SemVer  struct
    version.to_str().map_err(Error::from)
}

/// Sets a process wide option
///
/// # Parameters
/// * `option` - The key of the option to set
/// * `value` - The value to set the option
///
/// # Examples
/// ```rust, ignore
/// use maa_rs_sys::{Assistant, OptionKey};
///
/// Assistant::set_static_option(OptionKey::new(1), "value");
/// ```
pub fn set_static_option<S: Into<String>>(option: OptionKey, value: S) -> Result<()> {
    let c_option_value = CString::new(value.into())?;

    // Safety: The string is guaranteed to be null-terminated and valid since it was
    // created in safe rust with no errors.
    let return_code = unsafe { AsstSetStaticOption(option.0, c_option_value.as_ptr()) };
    is_success(return_code)
}

/// Sets the working directory of the MAA backend
///
/// The working directory is the directory where the backend stores the log files or looks
/// up cache entries.
///
/// # Parameters
/// * `path` - The path to the new working directory
///
/// # Examples
/// ```rust, ignore
/// use maa_rs_sys::set_working_directory;
///
/// set_working_directory("/path/to/working/directory");
/// ```
pub fn set_working_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path_to_bytes(path).ok_or(Error::InvalidPath)?;
    let c_path = CString::new(path)?;

    // Safety: The string is guaranteed to be null-terminated and valid since it was
    // created in safe rust with no errors.
    let return_code = unsafe { AsstSetUserDir(c_path.as_ptr()) };
    is_success(return_code)
}

/// Enumerates the posdible log levels for the MAA backend
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, EnumString, strum::Display)]
pub enum LogLevel {
    #[strum(serialize = "TRC")]
    Trace,

    #[strum(serialize = "DBG")]
    Debug,

    #[strum(serialize = "INF")]
    Info,

    #[strum(serialize = "WRN")]
    Warn,

    #[strum(serialize = "ERR")]
    Error,
}

/// Logs a message with the backend
///
/// # Parameters
/// * `level` - The log level of the message
/// * `message` - The message to log
///
/// # Examples
/// ```rust, ignore
/// use maa_rs_sys::{log, LogLevel};
///
/// log(LogLevel::Info, "This is an info message");
/// ```
pub fn log(level: LogLevel, message: &str) -> Result<()> {
    let level = CString::new(level.to_string())?;
    let message = CString::new(message)?;

    // Safety: The strings are guaranteed to be null-terminated and valid since they
    // were created in safe code with no errors.
    unsafe {
        AsstLog(level.as_ptr(), message.as_ptr());
    }

    Ok(())
}

/// Since paths on Windows might contain non UTF-8 characters, this function needs to
/// covert the path into bytes first, which might fail. On the other hand convert the
/// path to a C string. This is pretty straightworard on UNIX based systems, since
/// there paths are just a sequence of bytes. However, on Windows paths are either
/// passed as ASCII or UTF-16. Passing UTF-8 paths to the system is not supported.
/// Therefore we have to make a distinction between windows and unix, which means a
/// slight performance hit on non-UNIX systems.
///
/// See:
/// * https://stackoverflow.com/questions/38948669/
/// * https://internals.rust-lang.org/t/pathbuf-to-cstring/12560
fn path_to_bytes<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(path.as_ref().as_os_str().as_bytes().to_vec())
    }

    #[cfg(not(unix))]
    {
        // On Windows, could use std::os::windows::ffi::OsStrExt to encode_wide(),
        // but you end up with a Vec<u16> instead of a Vec<u8>, so that doesn't
        // really help.
        path.as_ref().to_str().map(|s| s.as_bytes().to_vec())
    }
}

/// Gets the size of an object when it was not properly read
///
/// BUG: On the C/C++ side this is defifned as the number _-1_,
/// however an u64 (or an uint64_t on the FFI side) can never have that value and will
/// wrap arount to u64::MAX
fn is_valid_size(size: usize) -> bool {
    // Safety: This call should never fail, as there are no parameters passed to it.
    let null_size = unsafe { AsstGetNullSize() };

    // We can always cast an usize to an u64, as _usize <= u64_ always holds
    size as u64 != null_size
}

/// Tests if the return code of the backend call is "true"
///
/// # Parameters
/// * `code` - The return code of the backend call
///
/// # Examples
/// ```rust, ignore
/// use maa_rs_sys::check_return_code;
///
/// let return_code: AsstBool = 1;
/// let actual = check_return_code(return_code);
///
/// assert!(actual.is_ok());
/// ```
///
/// ```rust, ignore
/// use maa_rs_sys::check_return_code;
///
/// let return_code: AsstBool = 0;
/// let actual = check_return_code(return_code);
///     
/// assert!(actual.is_err())
/// ```
fn is_success(code: AsstBool) -> Result<()> {
    if code == 1 {
        Ok(())
    } else {
        Err(Error::Unknown)
    }
}

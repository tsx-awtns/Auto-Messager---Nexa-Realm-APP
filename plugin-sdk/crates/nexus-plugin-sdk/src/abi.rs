// Nexus Realm
// File: abi.rs
// Purpose: Plugin API v1 WASM export shims and bounded message protocol

use crate::plugin::{NexusPlugin, PluginContext, PluginError, PluginEvent};
use nexus_plugin_api::abi::{
    AbiDescribeResponse, AbiEvent, AbiInitRequest, Status, HOST_LOG_LEVELS, MAX_ABI_MESSAGE_BYTES,
    MAX_HOST_LOG_BYTES,
};
use nexus_plugin_api::version::{api_version_word, ApiVersion, MANIFEST_SCHEMA_VERSION};
use nexus_plugin_api::PluginId;
use std::sync::Mutex;

pub const MAX_MESSAGE_BYTES: u32 = MAX_ABI_MESSAGE_BYTES;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "nexus_host")]
extern "C" {
    fn nexus_host_log(level: u32, pointer: u32, length: u32) -> i32;
}

fn validate_log_request(level: u32, message: &str) -> Result<(), PluginError> {
    if !HOST_LOG_LEVELS.contains(&level) {
        return Err(PluginError::InvalidArgument);
    }
    if message.is_empty() {
        return Err(PluginError::InvalidArgument);
    }
    if message.len() as u32 > MAX_HOST_LOG_BYTES {
        return Err(PluginError::LimitExceeded);
    }
    if message.chars().any(|character| character.is_control()) {
        return Err(PluginError::InvalidArgument);
    }
    Ok(())
}

fn status_result(status: i32) -> Result<(), PluginError> {
    match Status::from_code(status) {
        Some(Status::Ok) => Ok(()),
        Some(status) => Err(PluginError::from(status)),
        None => Err(PluginError::Failed),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn host_log(level: u32, message: &str) -> Result<(), PluginError> {
    validate_log_request(level, message)?;
    let status = unsafe { nexus_host_log(level, message.as_ptr() as u32, message.len() as u32) };
    status_result(status)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn host_log(level: u32, message: &str) -> Result<(), PluginError> {
    validate_log_request(level, message)?;
    Err(PluginError::InvalidState)
}

pub fn api_version() -> u32 {
    api_version_word()
}

pub fn alloc(size: u32) -> u32 {
    if size == 0 || size > MAX_ABI_MESSAGE_BYTES {
        return 0;
    }
    let mut buffer = vec![0u8; size as usize];
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer as usize as u32
}

pub fn free(pointer: u32, size: u32) {
    if pointer == 0 || size == 0 || size > MAX_ABI_MESSAGE_BYTES {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(
            pointer as usize as *mut u8,
            size as usize,
            size as usize,
        ));
    }
}

pub fn init<P: NexusPlugin>(slot: &Mutex<Option<P>>, request: &[u8]) -> Status {
    if request.is_empty() || request.len() > MAX_ABI_MESSAGE_BYTES as usize {
        return Status::InvalidArgument;
    }
    let request: AbiInitRequest = match serde_json::from_slice(request) {
        Ok(request) => request,
        Err(_) => return Status::MalformedMessage,
    };
    if let Err(status) = request.validate() {
        return status;
    }
    let api_version = match ApiVersion::parse(&request.api_version) {
        Ok(version) => version,
        Err(_) => return Status::UnsupportedVersion,
    };
    let mut guard = match slot.lock() {
        Ok(guard) => guard,
        Err(_) => return Status::Internal,
    };
    if guard.is_some() {
        return Status::InvalidState;
    }
    let mut instance = P::default();
    let metadata = instance.metadata();
    let plugin_id = match metadata.plugin_id() {
        Ok(id) => id,
        Err(_) => return Status::InvalidArgument,
    };
    if plugin_id.as_str() != request.plugin_id {
        return Status::InvalidArgument;
    }
    if metadata.describe_response().is_err() {
        return Status::InvalidArgument;
    }
    let context = PluginContext::new(plugin_id, api_version);
    if let Err(error) = instance.on_load(&context) {
        return error.status();
    }
    *guard = Some(instance);
    Status::Ok
}

pub fn describe<P: NexusPlugin>(
    slot: &Mutex<Option<P>>,
) -> Result<Vec<u8>, PluginError> {
    let guard = slot.lock().map_err(|_| PluginError::InvalidState)?;
    let instance = guard.as_ref().ok_or(PluginError::InvalidState)?;
    let response: AbiDescribeResponse = instance
        .metadata()
        .describe_response()?;
    serde_json::to_vec(&response).map_err(|_| PluginError::MalformedMessage)
}

pub fn event<P: NexusPlugin>(slot: &Mutex<Option<P>>, message: &[u8]) -> Status {
    if message.is_empty() || message.len() > MAX_ABI_MESSAGE_BYTES as usize {
        return Status::InvalidArgument;
    }
    let message: AbiEvent = match serde_json::from_slice(message) {
        Ok(message) => message,
        Err(_) => return Status::MalformedMessage,
    };
    if let Err(status) = message.validate() {
        return status;
    }
    let event = match PluginEvent::from_message(message) {
        Ok(event) => event,
        Err(error) => return error.status(),
    };
    let mut guard = match slot.lock() {
        Ok(guard) => guard,
        Err(_) => return Status::Internal,
    };
    let instance = match guard.as_mut() {
        Some(instance) => instance,
        None => return Status::InvalidState,
    };
    let metadata = instance.metadata();
    let plugin_id = match metadata.plugin_id() {
        Ok(id) => id,
        Err(_) => return Status::InvalidArgument,
    };
    let context = PluginContext::new(plugin_id, ApiVersion::CURRENT);
    match instance.on_event(&context, event) {
        Ok(()) => Status::Ok,
        Err(error) => error.status(),
    }
}

pub fn shutdown<P: NexusPlugin>(slot: &Mutex<Option<P>>) -> Status {
    match slot.lock() {
        Ok(mut guard) => {
            guard.take();
            Status::Ok
        }
        Err(_) => Status::Internal,
    }
}

pub fn dispatch_init<P: NexusPlugin>(slot: &Mutex<Option<P>>, pointer: u32, length: u32) -> i32 {
    match with_buffer(pointer, length, |request| init::<P>(slot, request)) {
        Ok(status) => status.code(),
        Err(status) => status.code(),
    }
}

pub fn dispatch_describe<P: NexusPlugin>(
    slot: &Mutex<Option<P>>,
    pointer: u32,
    capacity: u32,
) -> i32 {
    let message = match describe::<P>(slot) {
        Ok(message) => message,
        Err(error) => return error.status().code(),
    };
    match write_buffer(pointer, capacity, &message) {
        Ok(written) => written as i32,
        Err(status) => status.code(),
    }
}

pub fn dispatch_event<P: NexusPlugin>(slot: &Mutex<Option<P>>, pointer: u32, length: u32) -> i32 {
    match with_buffer(pointer, length, |message| event::<P>(slot, message)) {
        Ok(status) => status.code(),
        Err(status) => status.code(),
    }
}

pub fn dispatch_shutdown<P: NexusPlugin>(slot: &Mutex<Option<P>>) -> i32 {
    shutdown::<P>(slot).code()
}

pub fn init_request(plugin_id: &PluginId) -> Result<Vec<u8>, PluginError> {
    let request = AbiInitRequest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        api_version: nexus_plugin_api::PLUGIN_API_VERSION.to_string(),
        plugin_id: plugin_id.as_str().to_string(),
    };
    serde_json::to_vec(&request).map_err(|_| PluginError::MalformedMessage)
}

pub fn event_message(sequence: u64, occurred_at_ms: u64, kind: nexus_plugin_api::EventKind) -> Vec<u8> {
    let message = AbiEvent {
        schema_version: nexus_plugin_api::EVENT_SCHEMA_VERSION,
        sequence,
        occurred_at_ms,
        kind,
    };
    serde_json::to_vec(&message).unwrap_or_default()
}

pub fn validate_pointer_args(pointer: u32, length: u32) -> Result<(), Status> {
    if pointer == 0 || length == 0 || length > MAX_ABI_MESSAGE_BYTES {
        return Err(Status::InvalidArgument);
    }
    Ok(())
}

pub fn with_buffer<T>(
    pointer: u32,
    length: u32,
    action: impl FnOnce(&[u8]) -> T,
) -> Result<T, Status> {
    validate_pointer_args(pointer, length)?;
    let bytes = unsafe { std::slice::from_raw_parts(pointer as usize as *const u8, length as usize) };
    Ok(action(bytes))
}

pub fn write_buffer(pointer: u32, capacity: u32, message: &[u8]) -> Result<u32, Status> {
    if pointer == 0 || capacity == 0 || capacity > MAX_ABI_MESSAGE_BYTES {
        return Err(Status::InvalidArgument);
    }
    if message.len() as u32 > capacity {
        return Err(Status::LimitExceeded);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            message.as_ptr(),
            pointer as usize as *mut u8,
            message.len(),
        );
    }
    Ok(message.len() as u32)
}

#[macro_export]
macro_rules! export_plugin {
    ($plugin:ty) => {
        static NEXUS_PLUGIN_INSTANCE: ::std::sync::Mutex<::std::option::Option<$plugin>> =
            ::std::sync::Mutex::new(::std::option::Option::None);

        #[no_mangle]
        pub extern "C" fn nexus_plugin_api_version() -> u32 {
            $crate::abi::api_version()
        }

        #[no_mangle]
        pub extern "C" fn nexus_plugin_alloc(size: u32) -> u32 {
            $crate::abi::alloc(size)
        }

        #[no_mangle]
        pub extern "C" fn nexus_plugin_free(pointer: u32, size: u32) {
            $crate::abi::free(pointer, size)
        }

        #[no_mangle]
        pub extern "C" fn nexus_plugin_init(pointer: u32, length: u32) -> i32 {
            $crate::abi::dispatch_init::<$plugin>(&NEXUS_PLUGIN_INSTANCE, pointer, length)
        }

        #[no_mangle]
        pub extern "C" fn nexus_plugin_describe(pointer: u32, capacity: u32) -> i32 {
            $crate::abi::dispatch_describe::<$plugin>(&NEXUS_PLUGIN_INSTANCE, pointer, capacity)
        }

        #[no_mangle]
        pub extern "C" fn nexus_plugin_event(pointer: u32, length: u32) -> i32 {
            $crate::abi::dispatch_event::<$plugin>(&NEXUS_PLUGIN_INSTANCE, pointer, length)
        }

        #[no_mangle]
        pub extern "C" fn nexus_plugin_shutdown() -> i32 {
            $crate::abi::dispatch_shutdown::<$plugin>(&NEXUS_PLUGIN_INSTANCE)
        }
    };
}

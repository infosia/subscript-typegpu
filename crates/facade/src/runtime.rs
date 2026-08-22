//! Future-slot lifetime machinery (F7/F8). This module carries only
//! numeric kind tags and raw handle values.
//! Generated code owns backend names and performs deferred releases.
#![cfg_attr(test, allow(dead_code))]

use std::collections::{BTreeMap, VecDeque};
use std::ffi::c_void;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::generated;

const STATUS_UNKNOWN_FUTURE: i32 = -100;

struct Slot {
    instance: usize,
    kind: u32,
    outcome: Option<Outcome>,
    userdata: Option<usize>,
    doomed: bool,
    device_event_id: Option<usize>,
}

struct Outcome {
    status: i32,
    handle: usize,
    record_value: i32,
    #[allow(dead_code)]
    message: String,
}

struct EventRecord {
    value: i32,
    message: String,
}

struct DeviceEvents {
    device: Option<usize>,
    uncaptured: VecDeque<EventRecord>,
    lost: Option<EventRecord>,
    filled_message: String,
}

struct DeviceEventTable {
    by_id: BTreeMap<usize, DeviceEvents>,
    by_device: BTreeMap<usize, usize>,
}

/// A record value plus a view into facade-owned string bytes.
pub(crate) struct RecordFill {
    pub(crate) value: i32,
    pub(crate) data: usize,
    pub(crate) length: usize,
}

/// A completed handle still owned by a discarded slot.
pub(crate) struct OwnedHandle {
    pub(crate) kind: u32,
    pub(crate) value: usize,
}

static FUTURES: Mutex<BTreeMap<u64, Slot>> = Mutex::new(BTreeMap::new());
static DEFERRED_HANDLES: Mutex<Vec<OwnedHandle>> = Mutex::new(Vec::new());
static HANDLE_INSTANCES: Mutex<BTreeMap<usize, usize>> = Mutex::new(BTreeMap::new());
static DEVICE_EVENTS: Mutex<DeviceEventTable> = Mutex::new(DeviceEventTable {
    by_id: BTreeMap::new(),
    by_device: BTreeMap::new(),
});
static FUTURE_FILLED_MESSAGES: Mutex<BTreeMap<usize, String>> = Mutex::new(BTreeMap::new());
static ADAPTER_INFO_STRINGS: Mutex<BTreeMap<usize, [String; 4]>> = Mutex::new(BTreeMap::new());
static NEXT_FUTURE_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_DEVICE_EVENT_ID: AtomicUsize = AtomicUsize::new(1);
static OWNED_HANDLE_RELEASE_COUNT: AtomicUsize = AtomicUsize::new(0);
static WEBGPU_TABLE: OnceLock<generated::WebgpuTable> = OnceLock::new();

pub(crate) fn table() -> Option<&'static generated::WebgpuTable> {
    WEBGPU_TABLE.get()
}

pub(crate) fn initialize_table() -> bool {
    if WEBGPU_TABLE.get().is_some() {
        return true;
    }
    let Some(path) = std::env::var_os("SUBSCRIPT_TYPEGPU_BACKEND_LIB") else {
        let _ = writeln!(
            std::io::stderr(),
            "subscript-typegpu: set SUBSCRIPT_TYPEGPU_BACKEND_LIB to the webgpu.h shared library"
        );
        return false;
    };
    let path = std::path::PathBuf::from(path);
    let loaded = match generated::WebgpuTable::load(&path) {
        Ok(table) => table,
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "subscript-typegpu: {error}");
            return false;
        }
    };
    WEBGPU_TABLE.set(loaded).is_ok() || WEBGPU_TABLE.get().is_some()
}

fn futures() -> std::sync::MutexGuard<'static, BTreeMap<u64, Slot>> {
    FUTURES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn deferred_handles() -> std::sync::MutexGuard<'static, Vec<OwnedHandle>> {
    DEFERRED_HANDLES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn handle_instances() -> std::sync::MutexGuard<'static, BTreeMap<usize, usize>> {
    HANDLE_INSTANCES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn device_events() -> std::sync::MutexGuard<'static, DeviceEventTable> {
    DEVICE_EVENTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn future_filled_messages() -> std::sync::MutexGuard<'static, BTreeMap<usize, String>> {
    FUTURE_FILLED_MESSAGES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn adapter_info_strings() -> std::sync::MutexGuard<'static, BTreeMap<usize, [String; 4]>> {
    ADAPTER_INFO_STRINGS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Stores the four copied adapter-info strings until the next fill on
/// the same parent handle and returns stable views into their bytes.
pub(crate) fn store_adapter_info_strings(parent: usize, strings: [String; 4]) -> [RecordFill; 4] {
    let mut table = adapter_info_strings();
    table.insert(parent, strings);
    let Some(stored) = table.get(&parent) else {
        return std::array::from_fn(|_| RecordFill {
            value: 0,
            data: 0,
            length: 0,
        });
    };
    std::array::from_fn(|index| RecordFill {
        value: 0,
        data: stored[index].as_ptr() as usize,
        length: stored[index].len(),
    })
}

/// Releases adapter-info string bytes owned by one handle.
pub(crate) fn release_adapter_info_strings(parent: usize) {
    adapter_info_strings().remove(&parent);
}

pub(crate) fn instance_for_handle(handle: usize) -> usize {
    handle_instances().get(&handle).copied().unwrap_or(0)
}

pub(crate) fn inherit_handle_instance(parent: usize, child: usize) {
    if child == 0 {
        return;
    }
    let mut owners = handle_instances();
    let Some(instance) = owners.get(&parent).copied() else {
        return;
    };
    owners.insert(child, instance);
}

pub(crate) fn register_instance(instance: usize) {
    if instance != 0 {
        handle_instances().insert(instance, instance);
    }
}

pub(crate) fn new_pending_slot(instance: usize, kind: u32) -> (u64, *mut c_void) {
    let id = NEXT_FUTURE_ID.fetch_add(1, Ordering::Relaxed);
    let userdata = Box::into_raw(Box::new(id)).cast::<c_void>();
    futures().insert(
        id,
        Slot {
            instance,
            kind,
            outcome: None,
            userdata: Some(userdata as usize),
            doomed: false,
            device_event_id: None,
        },
    );
    (id, userdata)
}

/// Attaches request-device callback state so instance release can
/// reclaim it even if the request is still pending.
pub(crate) fn attach_device_event_to_future(future: u64, event_id: usize) {
    if let Some(slot) = futures().get_mut(&future) {
        slot.device_event_id = Some(event_id);
    }
}

pub(crate) fn callback_guard(f: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(f)).is_err() {
        std::process::abort();
    }
}

/// Records one callback completion and reclaims its userdata box.
///
/// # Safety
///
/// `userdata1` is the unique pointer returned with this request and is
/// passed exactly once by the registered callback.
pub(crate) unsafe fn complete_from_callback(
    userdata1: *mut c_void,
    kind: u32,
    success: bool,
    raw_status: i32,
    handle: usize,
    message: String,
) {
    unsafe {
        complete_outcome_from_callback(userdata1, raw_status, success, kind, handle, 0, message);
    }
}

/// Records a callback completion carrying an F11 record.
///
/// # Safety
///
/// `userdata1` is the unique pointer returned with this request and is
/// passed exactly once by the registered callback.
pub(crate) unsafe fn complete_record_from_callback(
    userdata1: *mut c_void,
    kind: u32,
    success: bool,
    raw_status: i32,
    record_value: i32,
    message: String,
) {
    unsafe {
        complete_outcome_from_callback(
            userdata1,
            raw_status,
            success,
            kind,
            0,
            record_value,
            message,
        );
    }
}

unsafe fn complete_outcome_from_callback(
    userdata1: *mut c_void,
    raw_status: i32,
    success: bool,
    kind: u32,
    handle: usize,
    record_value: i32,
    message: String,
) {
    // SAFETY: guaranteed by the callback registration contract above.
    let future_id = unsafe { *Box::from_raw(userdata1.cast::<u64>()) };
    let owned = if success { handle } else { 0 };
    let mut table = futures();
    let Some(slot) = table.get_mut(&future_id) else {
        drop(table);
        if owned != 0 {
            deferred_handles().push(OwnedHandle { kind, value: owned });
        }
        return;
    };
    slot.userdata = None;
    if slot.doomed {
        table.remove(&future_id);
        drop(table);
        if owned != 0 {
            deferred_handles().push(OwnedHandle { kind, value: owned });
        }
        return;
    }
    slot.outcome = Some(Outcome {
        status: if success { 1 } else { -raw_status },
        handle: owned,
        record_value,
        message,
    });
}

fn belongs_to_instance(slot: &Slot, instance: usize) -> bool {
    slot.instance == instance
}

pub(crate) fn future_status(instance: usize, future: u64) -> i32 {
    let mut table = futures();
    let Some(slot) = table.get_mut(&future) else {
        return STATUS_UNKNOWN_FUTURE;
    };
    if slot.doomed || !belongs_to_instance(slot, instance) {
        return STATUS_UNKNOWN_FUTURE;
    }
    slot.outcome.as_ref().map_or(0, |outcome| outcome.status)
}

/// Successful typed take removes the slot immediately (F8).
pub(crate) fn take_handle(instance: usize, future: u64, kind: u32) -> usize {
    let mut table = futures();
    let can_take = table.get_mut(&future).is_some_and(|slot| {
        belongs_to_instance(slot, instance)
            && !slot.doomed
            && slot.kind == kind
            && slot
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.status == 1 && outcome.handle != 0)
    });
    if !can_take {
        return 0;
    }
    let handle = table
        .remove(&future)
        .and_then(|slot| slot.outcome)
        .map_or(0, |outcome| outcome.handle);
    drop(table);
    if handle != 0 {
        handle_instances().insert(handle, instance);
    }
    handle
}

/// Successful typed record take removes the slot and stores its string
/// bytes until the next record fill on the same instance.
pub(crate) fn take_record(instance: usize, future: u64, kind: u32) -> Option<RecordFill> {
    let mut table = futures();
    let can_take = table.get(&future).is_some_and(|slot| {
        belongs_to_instance(slot, instance)
            && !slot.doomed
            && slot.kind == kind
            && slot
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.status == 1)
    });
    if !can_take {
        return None;
    }
    let outcome = table.remove(&future)?.outcome?;
    drop(table);
    let mut messages = future_filled_messages();
    let storage = messages.entry(instance).or_default();
    Some(fill_record(storage, outcome.record_value, &outcome.message))
}

/// Drops a completed slot, or marks a pending slot doomed until its
/// callback reclaims userdata and discards the result.
pub(crate) fn drop_future(instance: usize, future: u64) -> Option<OwnedHandle> {
    let mut table = futures();
    let slot = table.get_mut(&future)?;
    if !belongs_to_instance(slot, instance) {
        return None;
    }
    if slot.outcome.is_none() {
        slot.doomed = true;
        return None;
    }
    let slot = table.remove(&future)?;
    let outcome = slot.outcome?;
    (outcome.handle != 0).then_some(OwnedHandle {
        kind: slot.kind,
        value: outcome.handle,
    })
}

pub(crate) fn drain_deferred_handles() -> Vec<OwnedHandle> {
    std::mem::take(&mut *deferred_handles())
}

pub(crate) fn note_owned_handle_release() {
    OWNED_HANDLE_RELEASE_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Removes the released instance's slots. Pending userdata boxes can
/// Callbacks cannot observe the removed slots after instance release.
/// The function returns completed owned handles for release.
pub(crate) fn release_all_slots(instance: usize) -> Vec<OwnedHandle> {
    let mut table = futures();
    let ids: Vec<u64> = table
        .iter()
        .filter_map(|(id, slot)| (slot.instance == instance).then_some(*id))
        .collect();
    let mut owned = Vec::new();
    for id in ids {
        let Some(slot) = table.remove(&id) else {
            continue;
        };
        if let Some(event_id) = slot.device_event_id {
            discard_device_event_slot(event_id);
        }
        if let Some(userdata) = slot.userdata {
            // SAFETY: a pending slot owns the unique userdata box and no
            // callback can run after its instance has been released.
            unsafe { drop(Box::from_raw(userdata as *mut u64)) };
        }
        if let Some(outcome) = slot.outcome {
            if outcome.handle != 0 {
                owned.push(OwnedHandle {
                    kind: slot.kind,
                    value: outcome.handle,
                });
            }
        }
    }
    drop(table);
    let mut owners = handle_instances();
    let released_handles: Vec<usize> = owners
        .iter()
        .filter_map(|(handle, owner)| (*owner == instance).then_some(*handle))
        .collect();
    owners.retain(|_, owner| *owner != instance);
    drop(owners);
    for handle in released_handles {
        release_device_events(handle);
        release_adapter_info_strings(handle);
    }
    future_filled_messages().remove(&instance);
    owned
}

fn fill_record(storage: &mut String, value: i32, message: &str) -> RecordFill {
    storage.clear();
    storage.push_str(message);
    RecordFill {
        value,
        data: storage.as_ptr() as usize,
        length: storage.len(),
    }
}

/// Allocates one stable numeric callback key before device creation.
pub(crate) fn new_device_event_slot() -> usize {
    let id = NEXT_DEVICE_EVENT_ID.fetch_add(1, Ordering::Relaxed);
    device_events().by_id.insert(
        id,
        DeviceEvents {
            device: None,
            uncaptured: VecDeque::new(),
            lost: None,
            filled_message: String::new(),
        },
    );
    id
}

/// Associates request-device callback state with its completed handle.
pub(crate) fn associate_device_events(event_id: usize, device: usize) {
    if event_id == 0 || device == 0 {
        return;
    }
    let mut table = device_events();
    let Some(events) = table.by_id.get_mut(&event_id) else {
        return;
    };
    events.device = Some(device);
    table.by_device.insert(device, event_id);
}

/// Discards callback state for a request that did not produce a device.
pub(crate) fn discard_device_event_slot(event_id: usize) {
    if event_id == 0 {
        return;
    }
    let mut table = device_events();
    if let Some(events) = table.by_id.remove(&event_id) {
        if let Some(device) = events.device {
            table.by_device.remove(&device);
        }
    }
}

/// Enqueues one immediate-style callback result from any thread.
pub(crate) fn enqueue_uncaptured_error(event_id: usize, value: i32, message: String) {
    let mut table = device_events();
    let Some(events) = table.by_id.get_mut(&event_id) else {
        return;
    };
    events.uncaptured.push_back(EventRecord { value, message });
}

/// Records the single creation-time device-lost callback result.
pub(crate) fn record_device_lost(event_id: usize, value: i32, message: String) {
    let mut table = device_events();
    let Some(events) = table.by_id.get_mut(&event_id) else {
        return;
    };
    events.lost = Some(EventRecord { value, message });
}

/// Drains one uncaptured error and returns a view valid until the next
/// fill call on this device.
pub(crate) fn next_uncaptured_error(device: usize) -> Option<RecordFill> {
    let mut table = device_events();
    let event_id = *table.by_device.get(&device)?;
    let events = table.by_id.get_mut(&event_id)?;
    let record = events.uncaptured.pop_front()?;
    Some(fill_record(
        &mut events.filled_message,
        record.value,
        &record.message,
    ))
}

/// Returns the recorded lost info and refreshes the device's shared
/// fill buffer.
pub(crate) fn device_lost_info(device: usize) -> Option<RecordFill> {
    let mut table = device_events();
    let event_id = *table.by_device.get(&device)?;
    let events = table.by_id.get_mut(&event_id)?;
    let record = events.lost.as_ref()?;
    let value = record.value;
    let message = record.message.clone();
    Some(fill_record(&mut events.filled_message, value, &message))
}

/// Test-only injection through the same per-device FIFO.
pub(crate) fn enqueue_uncaptured_for_device(device: usize, value: i32, message: String) -> bool {
    let table = device_events();
    let Some(event_id) = table.by_device.get(&device).copied() else {
        return false;
    };
    drop(table);
    enqueue_uncaptured_error(event_id, value, message);
    true
}

/// Removes the state associated with a released device. Numeric callback
/// userdata remains safe if a late backend callback arrives.
pub(crate) fn release_device_events(device: usize) {
    let mut table = device_events();
    let Some(event_id) = table.by_device.remove(&device) else {
        return;
    };
    table.by_id.remove(&event_id);
}

/// Test-only observability for the F8 leak assertion.
#[doc(hidden)]
#[cfg(test)]
pub fn subscript_typegpu_internal_slot_count() -> usize {
    futures().len()
}

/// Test-only observability for owned-handle cleanup.
#[doc(hidden)]
#[cfg(test)]
pub fn subscript_typegpu_internal_owned_handle_release_count() -> usize {
    OWNED_HANDLE_RELEASE_COUNT.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_error_and_aborted_statuses_remain_distinct() {
        const INSTANCE: usize = 41;
        let _ = release_all_slots(INSTANCE);
        let (error_id, error_userdata) = new_pending_slot(INSTANCE, 7);
        let (aborted_id, aborted_userdata) = new_pending_slot(INSTANCE, 7);
        // SAFETY: each pointer is passed exactly once, as a callback would.
        unsafe {
            complete_from_callback(error_userdata, 7, false, 3, 0, String::new());
            complete_from_callback(aborted_userdata, 7, false, 4, 0, String::new());
        }
        assert_eq!(future_status(INSTANCE, error_id), -3);
        assert_eq!(future_status(INSTANCE, aborted_id), -4);
        assert!(drop_future(INSTANCE, error_id).is_none());
        assert!(drop_future(INSTANCE, aborted_id).is_none());
        assert_eq!(subscript_typegpu_internal_slot_count(), 0);
    }

    #[test]
    fn uncaptured_queue_accepts_cross_thread_enqueue() {
        let event_id = new_device_event_slot();
        let device = event_id + 10_000;
        associate_device_events(event_id, device);
        std::thread::spawn(move || {
            enqueue_uncaptured_error(event_id, 2, "cross-thread".to_string());
        })
        .join()
        .expect("enqueue thread");
        let record = next_uncaptured_error(device).expect("queued record");
        assert_eq!(record.value, 2);
        assert_eq!(record.length, "cross-thread".len());
        release_device_events(device);
    }
}

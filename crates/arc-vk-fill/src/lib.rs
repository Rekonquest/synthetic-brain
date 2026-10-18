//! BEAT-1 (docs/BEAT-1-HEARTBEAT.md): the Arc as a heat transmitter, composed over the existing
//! driver. Through the system's Vulkan loader (found by `dlopen` at run time — no crates) and Mesa's
//! Intel driver, both untouched, it fills a device-local buffer with `vkCmdFillBuffer` again and
//! again: memory traffic, no shader. Every submission is fenced, so it stops within one submission.
//! It writes no setting and touches no power or clock control. Its own check: a host-visible buffer
//! filled on the GPU reads back as the fill word. Which device, how big and how long are the
//! caller's.

use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::{null, null_mut};

type Handle = *mut c_void;
type Nd = u64;
type Pfn = *const c_void;

extern "C" {
    fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[repr(C)]
struct AppInfo {
    s_type: u32,
    p_next: *const c_void,
    app_name: *const c_char,
    app_version: u32,
    engine_name: *const c_char,
    engine_version: u32,
    api_version: u32,
}
#[repr(C)]
struct InstanceInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
    app: *const AppInfo,
    layer_count: u32,
    layers: *const *const c_char,
    ext_count: u32,
    exts: *const *const c_char,
}
#[repr(C)]
struct QueueInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
    family: u32,
    count: u32,
    priorities: *const f32,
}
#[repr(C)]
struct DeviceInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
    queue_info_count: u32,
    queue_infos: *const QueueInfo,
    layer_count: u32,
    layers: *const *const c_char,
    ext_count: u32,
    exts: *const *const c_char,
    features: *const c_void,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct QueueFamily {
    flags: u32,
    count: u32,
    timestamp_bits: u32,
    granularity: [u32; 3],
}
#[repr(C)]
struct BufferInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
    size: u64,
    usage: u32,
    sharing: u32,
    family_count: u32,
    families: *const u32,
}
#[repr(C)]
#[derive(Default)]
struct MemReq {
    size: u64,
    alignment: u64,
    type_bits: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct MemType {
    flags: u32,
    heap: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct MemHeap {
    size: u64,
    flags: u32,
}
#[repr(C)]
struct MemProps {
    type_count: u32,
    types: [MemType; 32],
    heap_count: u32,
    heaps: [MemHeap; 16],
}
#[repr(C)]
struct AllocInfo {
    s_type: u32,
    p_next: *const c_void,
    size: u64,
    type_index: u32,
}
#[repr(C)]
struct PoolInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
    family: u32,
}
#[repr(C)]
struct CmdAllocInfo {
    s_type: u32,
    p_next: *const c_void,
    pool: Nd,
    level: u32,
    count: u32,
}
#[repr(C)]
struct BeginInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
    inheritance: *const c_void,
}
#[repr(C)]
struct SubmitInfo {
    s_type: u32,
    p_next: *const c_void,
    wait_count: u32,
    waits: *const Nd,
    wait_stages: *const u32,
    cmd_count: u32,
    cmds: *const Handle,
    signal_count: u32,
    signals: *const Nd,
}
#[repr(C)]
struct FenceInfo {
    s_type: u32,
    p_next: *const c_void,
    flags: u32,
}

const DEVICE_LOCAL: u32 = 0x1;
const HOST_VISIBLE: u32 = 0x2;
const HOST_COHERENT: u32 = 0x4;
const TRANSFER_DST: u32 = 0x2;

/// The Arc, ready to burn power on demand.
pub struct Transmitter {
    f: Fns,
    instance: Handle,
    device: Handle,
    queue: Handle,
    pool: Nd,
    cmd: Handle,
    fence: Nd,
    buffers: Vec<(Nd, Nd)>,
    /// The device's name as the driver reports it.
    pub name: String,
    pub vendor: u32,
    pub device_id: u32,
    /// The words read back by the self-check, all of which must equal the fill word.
    pub check_words: usize,
    pub check_ok: bool,
}

struct Fns {
    gipa: unsafe extern "C" fn(Handle, *const c_char) -> Pfn,
}

macro_rules! vkfn {
    ($self:expr, $inst:expr, $name:literal, fn($($a:ty),*) $(-> $r:ty)?) => {{
        let n = CString::new($name).unwrap();
        let p = unsafe { ($self.gipa)($inst, n.as_ptr()) };
        assert!(!p.is_null(), "missing Vulkan entry point {}", $name);
        unsafe { std::mem::transmute::<Pfn, unsafe extern "C" fn($($a),*) $(-> $r)?>(p) }
    }};
}

fn ok(r: i32, what: &str) -> Result<(), String> {
    if r == 0 { Ok(()) } else { Err(format!("{what} returned VkResult {r}")) }
}

impl Transmitter {
    /// Open the first device whose vendor is `vendor` and whose name contains `name_part`; allocate
    /// a device-local buffer of `bytes`; record `fills_per_submit` fills of it; run the self-check.
    pub fn open(vendor: u32, name_part: &str, bytes: u64, fills_per_submit: usize) -> Result<Transmitter, String> {
        let lib = CString::new("libvulkan.so.1").unwrap();
        // SAFETY: dlopen/dlsym on the system loader; the symbol is the loader's documented entry point.
        let h = unsafe { dlopen(lib.as_ptr(), 2) };
        if h.is_null() {
            return Err("libvulkan.so.1 not found".into());
        }
        let sym = CString::new("vkGetInstanceProcAddr").unwrap();
        let p = unsafe { dlsym(h, sym.as_ptr()) };
        if p.is_null() {
            return Err("vkGetInstanceProcAddr not found".into());
        }
        let f = Fns { gipa: unsafe { std::mem::transmute::<*mut c_void, unsafe extern "C" fn(Handle, *const c_char) -> Pfn>(p) } };
        let create_instance = vkfn!(f, null_mut(), "vkCreateInstance", fn(*const InstanceInfo, *const c_void, *mut Handle) -> i32);
        let app_name = CString::new("arc-vk-fill").unwrap();
        let app = AppInfo { s_type: 0, p_next: null(), app_name: app_name.as_ptr(), app_version: 1, engine_name: app_name.as_ptr(), engine_version: 1, api_version: (1 << 22) | (1 << 12) };
        let ii = InstanceInfo { s_type: 1, p_next: null(), flags: 0, app: &app, layer_count: 0, layers: null(), ext_count: 0, exts: null() };
        let mut instance: Handle = null_mut();
        ok(unsafe { create_instance(&ii, null(), &mut instance) }, "vkCreateInstance")?;
        let enumerate = vkfn!(f, instance, "vkEnumeratePhysicalDevices", fn(Handle, *mut u32, *mut Handle) -> i32);
        let props = vkfn!(f, instance, "vkGetPhysicalDeviceProperties", fn(Handle, *mut u8));
        let qprops = vkfn!(f, instance, "vkGetPhysicalDeviceQueueFamilyProperties", fn(Handle, *mut u32, *mut QueueFamily));
        let mprops = vkfn!(f, instance, "vkGetPhysicalDeviceMemoryProperties", fn(Handle, *mut MemProps));
        let mut n = 0u32;
        ok(unsafe { enumerate(instance, &mut n, null_mut()) }, "vkEnumeratePhysicalDevices")?;
        let mut pds = vec![null_mut(); n as usize];
        ok(unsafe { enumerate(instance, &mut n, pds.as_mut_ptr()) }, "vkEnumeratePhysicalDevices")?;
        let mut chosen = None;
        for &pd in &pds {
            let mut raw = vec![0u64; 160];
            unsafe { props(pd, raw.as_mut_ptr() as *mut u8) };
            let b = raw.as_ptr() as *const u8;
            let v = unsafe { *(b.add(8) as *const u32) };
            let did = unsafe { *(b.add(12) as *const u32) };
            let name = unsafe { CStr::from_ptr(b.add(20) as *const c_char) }.to_string_lossy().into_owned();
            if v == vendor && name.contains(name_part) {
                chosen = Some((pd, name, v, did));
                break;
            }
        }
        let (pd, name, v, did) = chosen.ok_or(format!("no device with vendor {vendor:#x} and a name containing {name_part:?}"))?;
        let mut qn = 0u32;
        unsafe { qprops(pd, &mut qn, null_mut()) };
        let mut qf = vec![QueueFamily::default(); qn as usize];
        unsafe { qprops(pd, &mut qn, qf.as_mut_ptr()) };
        let family = qf.iter().position(|q| q.flags & 0x3 != 0 && q.count > 0).ok_or("no graphics/compute queue")? as u32;
        let prio = 1.0f32;
        let qi = QueueInfo { s_type: 2, p_next: null(), flags: 0, family, count: 1, priorities: &prio };
        let di = DeviceInfo { s_type: 3, p_next: null(), flags: 0, queue_info_count: 1, queue_infos: &qi, layer_count: 0, layers: null(), ext_count: 0, exts: null(), features: null() };
        let create_device = vkfn!(f, instance, "vkCreateDevice", fn(Handle, *const DeviceInfo, *const c_void, *mut Handle) -> i32);
        let mut device: Handle = null_mut();
        ok(unsafe { create_device(pd, &di, null(), &mut device) }, "vkCreateDevice")?;
        let get_queue = vkfn!(f, instance, "vkGetDeviceQueue", fn(Handle, u32, u32, *mut Handle));
        let mut queue: Handle = null_mut();
        unsafe { get_queue(device, family, 0, &mut queue) };
        let mut mp: MemProps = unsafe { std::mem::zeroed() };
        unsafe { mprops(pd, &mut mp) };
        let mut me = Transmitter { f, instance, device, queue, pool: 0, cmd: null_mut(), fence: 0, buffers: Vec::new(), name, vendor: v, device_id: did, check_words: 0, check_ok: false };
        // The self-check: a host-visible buffer filled on the GPU, read back.
        let check_bytes = 1 << 20;
        let (cbuf, cmem) = me.buffer(check_bytes, HOST_VISIBLE | HOST_COHERENT, &mp)?;
        me.record(&[(cbuf, check_bytes, 1)], 0xA5A5_A5A5)?;
        me.submit_wait()?;
        let map = vkfn!(me.f, me.instance, "vkMapMemory", fn(Handle, Nd, u64, u64, u32, *mut *mut c_void) -> i32);
        let mut ptr: *mut c_void = null_mut();
        ok(unsafe { map(me.device, cmem, 0, check_bytes, 0, &mut ptr) }, "vkMapMemory")?;
        let words = unsafe { std::slice::from_raw_parts(ptr as *const u32, (check_bytes / 4) as usize) };
        me.check_words = words.len();
        me.check_ok = words.iter().all(|&w| w == 0xA5A5_A5A5);
        let unmap = vkfn!(me.f, me.instance, "vkUnmapMemory", fn(Handle, Nd));
        unsafe { unmap(me.device, cmem) };
        // The load: a device-local buffer, filled `fills_per_submit` times per submission.
        let (lbuf, _) = me.buffer(bytes, DEVICE_LOCAL, &mp)?;
        me.record(&[(lbuf, bytes, fills_per_submit)], 0x5A5A_5A5A)?;
        Ok(me)
    }

    fn buffer(&mut self, bytes: u64, want: u32, mp: &MemProps) -> Result<(Nd, Nd), String> {
        let create = vkfn!(self.f, self.instance, "vkCreateBuffer", fn(Handle, *const BufferInfo, *const c_void, *mut Nd) -> i32);
        let reqs = vkfn!(self.f, self.instance, "vkGetBufferMemoryRequirements", fn(Handle, Nd, *mut MemReq));
        let alloc = vkfn!(self.f, self.instance, "vkAllocateMemory", fn(Handle, *const AllocInfo, *const c_void, *mut Nd) -> i32);
        let bind = vkfn!(self.f, self.instance, "vkBindBufferMemory", fn(Handle, Nd, Nd, u64) -> i32);
        let bi = BufferInfo { s_type: 12, p_next: null(), flags: 0, size: bytes, usage: TRANSFER_DST, sharing: 0, family_count: 0, families: null() };
        let mut buf: Nd = 0;
        ok(unsafe { create(self.device, &bi, null(), &mut buf) }, "vkCreateBuffer")?;
        let mut r = MemReq::default();
        unsafe { reqs(self.device, buf, &mut r) };
        let ti = (0..mp.type_count as usize).find(|&i| r.type_bits & (1 << i) != 0 && mp.types[i].flags & want == want).ok_or(format!("no memory type with flags {want:#x}"))?;
        let ai = AllocInfo { s_type: 5, p_next: null(), size: r.size, type_index: ti as u32 };
        let mut mem: Nd = 0;
        ok(unsafe { alloc(self.device, &ai, null(), &mut mem) }, "vkAllocateMemory")?;
        ok(unsafe { bind(self.device, buf, mem, 0) }, "vkBindBufferMemory")?;
        self.buffers.push((buf, mem));
        Ok((buf, mem))
    }

    /// (Re)record the command buffer: for each (buffer, bytes, times), that many whole-buffer fills.
    fn record(&mut self, fills: &[(Nd, u64, usize)], word: u32) -> Result<(), String> {
        if self.pool == 0 {
            let create_pool = vkfn!(self.f, self.instance, "vkCreateCommandPool", fn(Handle, *const PoolInfo, *const c_void, *mut Nd) -> i32);
            let pi = PoolInfo { s_type: 39, p_next: null(), flags: 0x2, family: 0 };
            ok(unsafe { create_pool(self.device, &pi, null(), &mut self.pool) }, "vkCreateCommandPool")?;
            let alloc = vkfn!(self.f, self.instance, "vkAllocateCommandBuffers", fn(Handle, *const CmdAllocInfo, *mut Handle) -> i32);
            let ai = CmdAllocInfo { s_type: 40, p_next: null(), pool: self.pool, level: 0, count: 1 };
            ok(unsafe { alloc(self.device, &ai, &mut self.cmd) }, "vkAllocateCommandBuffers")?;
            let create_fence = vkfn!(self.f, self.instance, "vkCreateFence", fn(Handle, *const FenceInfo, *const c_void, *mut Nd) -> i32);
            let fi = FenceInfo { s_type: 8, p_next: null(), flags: 0 };
            ok(unsafe { create_fence(self.device, &fi, null(), &mut self.fence) }, "vkCreateFence")?;
        }
        let begin = vkfn!(self.f, self.instance, "vkBeginCommandBuffer", fn(Handle, *const BeginInfo) -> i32);
        let fill = vkfn!(self.f, self.instance, "vkCmdFillBuffer", fn(Handle, Nd, u64, u64, u32));
        let end = vkfn!(self.f, self.instance, "vkEndCommandBuffer", fn(Handle) -> i32);
        let bi = BeginInfo { s_type: 42, p_next: null(), flags: 0, inheritance: null() };
        ok(unsafe { begin(self.cmd, &bi) }, "vkBeginCommandBuffer")?;
        for &(buf, bytes, times) in fills {
            for _ in 0..times {
                unsafe { fill(self.cmd, buf, 0, bytes, word) };
            }
        }
        ok(unsafe { end(self.cmd) }, "vkEndCommandBuffer")
    }

    fn submit_wait(&mut self) -> Result<(), String> {
        let submit = vkfn!(self.f, self.instance, "vkQueueSubmit", fn(Handle, u32, *const SubmitInfo, Nd) -> i32);
        let wait = vkfn!(self.f, self.instance, "vkWaitForFences", fn(Handle, u32, *const Nd, u32, u64) -> i32);
        let reset = vkfn!(self.f, self.instance, "vkResetFences", fn(Handle, u32, *const Nd) -> i32);
        let si = SubmitInfo { s_type: 4, p_next: null(), wait_count: 0, waits: null(), wait_stages: null(), cmd_count: 1, cmds: &self.cmd, signal_count: 0, signals: null() };
        ok(unsafe { submit(self.queue, 1, &si, self.fence) }, "vkQueueSubmit")?;
        ok(unsafe { wait(self.device, 1, &self.fence, 1, 5_000_000_000) }, "vkWaitForFences")?;
        ok(unsafe { reset(self.device, 1, &self.fence) }, "vkResetFences")
    }

    /// Submit the recorded load, fenced, until CLOCK_MONOTONIC reaches `until_ns`; the number of
    /// submissions completed.
    pub fn run_until(&mut self, until_ns: u64) -> Result<usize, String> {
        let mut n = 0;
        while arc_sense::monotonic_ns() < until_ns {
            self.submit_wait()?;
            n += 1;
        }
        Ok(n)
    }
}

impl Drop for Transmitter {
    fn drop(&mut self) {
        let idle = vkfn!(self.f, self.instance, "vkDeviceWaitIdle", fn(Handle) -> i32);
        let dfence = vkfn!(self.f, self.instance, "vkDestroyFence", fn(Handle, Nd, *const c_void));
        let dpool = vkfn!(self.f, self.instance, "vkDestroyCommandPool", fn(Handle, Nd, *const c_void));
        let dbuf = vkfn!(self.f, self.instance, "vkDestroyBuffer", fn(Handle, Nd, *const c_void));
        let free = vkfn!(self.f, self.instance, "vkFreeMemory", fn(Handle, Nd, *const c_void));
        let ddev = vkfn!(self.f, self.instance, "vkDestroyDevice", fn(Handle, *const c_void));
        let dinst = vkfn!(self.f, self.instance, "vkDestroyInstance", fn(Handle, *const c_void));
        unsafe {
            idle(self.device);
            if self.fence != 0 {
                dfence(self.device, self.fence, null());
            }
            if self.pool != 0 {
                dpool(self.device, self.pool, null());
            }
            for &(b, m) in &self.buffers {
                dbuf(self.device, b, null());
                free(self.device, m, null());
            }
            ddev(self.device, null());
            dinst(self.instance, null());
        }
    }
}

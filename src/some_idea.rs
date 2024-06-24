use crate::{
    gl_wrappers::Gl,
    linear::Transform,
    runtime::{Error, Result},
};
use glfw::PWindow;
use serde::{Deserialize, Serialize};
use std::{
    alloc::{self, Layout},
    any::{self, TypeId},
    marker::PhantomData,
    mem::{align_of, size_of, ManuallyDrop},
    ptr::{self, addr_of, drop_in_place, NonNull},
    slice,
};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Memory,
        SystemInformation::{self, SYSTEM_INFO},
    },
};

fn aligned_addr<T>(addr: usize) -> usize {
    let align = align_of::<T>();
    let aligned_addr = (addr + align - 1) & !(align - 1);

    aligned_addr
}

fn write_backwards<T>(buff: (*mut u8, usize), value: T, count: usize) {
    unsafe {
        let ptr = buff.0.add(buff.1 - size_of::<T>() * (count + 1));
        ptr.cast::<T>().write(value);
    };
}

fn kilo(bytes: usize) -> usize {
    bytes * 1024
}

#[derive(Debug)]
struct WindowsHeap(HANDLE);

impl WindowsHeap {
    fn new(size: usize) -> Result<Self> {
        unsafe {
            match Memory::HeapCreate(
                Memory::HEAP_NO_SERIALIZE | Memory::HEAP_CREATE_ALIGN_16,
                size,
                0,
            ) {
                Ok(handle) => Ok(Self(handle)),
                Err(_) => Err(Error::MemoryError),
            }
        }
    }

    fn alloc(&self, size: usize) -> Result<NonNull<u8>> {
        unsafe {
            let ptr = Memory::HeapAlloc(self.0, Memory::HEAP_NONE, size);
            if ptr == ptr::null_mut() {
                return Err(Error::MemoryError);
            }
            return Ok(NonNull::new_unchecked(ptr).cast());
        }
    }

    fn realloc(&self, ptr: NonNull<u8>, size: usize) -> Result<NonNull<u8>> {
        unsafe {
            let ptr =
                Memory::HeapReAlloc(self.0, Memory::HEAP_NONE, Some(ptr.cast().as_ptr()), size);
            if ptr == ptr::null_mut() {
                return Err(Error::MemoryError);
            }
            return Ok(NonNull::new_unchecked(ptr).cast());
        }
    }

    fn free(&self, ptr: NonNull<u8>) -> Result<()> {
        unsafe {
            if Memory::HeapFree(self.0, Memory::HEAP_NONE, Some(ptr.cast().as_ptr())).is_ok() {
                return Ok(());
            }
            return Err(Error::MemoryError);
        }
    }

    fn page_size() -> usize {
        let mut system_info = SYSTEM_INFO::default();
        unsafe {
            SystemInformation::GetSystemInfo(&mut system_info);
        }
        system_info.dwPageSize as usize
    }

    fn allocation_granularity() -> usize {
        let mut system_info = SYSTEM_INFO::default();
        unsafe {
            SystemInformation::GetSystemInfo(&mut system_info);
        }
        system_info.dwAllocationGranularity as usize
    }
}

impl Drop for WindowsHeap {
    fn drop(&mut self) {
        unsafe {
            assert!(Memory::HeapDestroy(self.0).is_ok());
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct MemoryBlockInfo {
    offset: usize,
    size: usize,
}

impl MemoryBlockInfo {
    fn new(offset: usize, size: usize) -> Self {
        Self { offset, size }
    }

    fn reduce(&mut self, size: usize) -> Self {
        assert!(self.size >= size);
        self.size -= size;
        self.offset += size;

        Self::new(self.offset, size)
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct DataBlockInfo {
    bl: MemoryBlockInfo,
    len: usize,
}

impl DataBlockInfo {
    fn new(block: MemoryBlockInfo, len: usize) -> Self {
        Self { bl: block, len }
    }
}

#[derive(Debug)]
struct Descriptor(usize);

#[derive(Debug)]
struct Header {
    data_block_ptr: *mut DataBlockInfo,
    data_block_len: usize,
    free_block_ptr: *mut MemoryBlockInfo,
    free_block_len: usize,
    size: usize,
}

impl Header {
    fn new(ptr: NonNull<u8>, size: usize) -> Self {
        Self::assert_align(ptr);

        let offset = Self::aligned_offset(size);
        let free_block_ptr = unsafe { ptr.cast::<MemoryBlockInfo>().as_ptr().byte_add(offset) };

        Self {
            data_block_ptr: ptr.cast().as_ptr(),
            data_block_len: 0,
            free_block_ptr,
            free_block_len: 0,
            size,
        }
    }

    fn aligned_offset(size: usize) -> usize {
        aligned_addr::<MemoryBlockInfo>(size / 2)
    }

    fn upsize_on_reallocation(&mut self, new_ptr: NonNull<u8>, new_size: usize) {
        Self::assert_align(new_ptr);

        assert!(new_size >= self.size);

        // Shift forward free blocks data
        unsafe {
            let offset = self.free_block_ptr as usize - self.data_block_ptr as usize;
            let old_free_block_ptr = new_ptr.cast::<MemoryBlockInfo>().as_ptr().byte_add(offset);
            let offset = Self::aligned_offset(new_size);
            let new_free_block_ptr = new_ptr.cast::<MemoryBlockInfo>().as_ptr().byte_add(offset);

            new_free_block_ptr.copy_from(old_free_block_ptr, self.free_block_len);
            self.free_block_ptr = new_free_block_ptr;
        }

        self.data_block_ptr = new_ptr.cast().as_ptr();
        self.size = new_size;
    }

    fn assert_align(ptr: NonNull<u8>) {
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % align_of::<DataBlockInfo>(), 0);
    }

    fn is_enough_for_data_block(&self) -> bool {
        let addr_bound = unsafe { self.data_block_ptr.add(self.data_block_len + 1) as usize };
        let free_block_addr = self.free_block_ptr as usize;
        addr_bound <= free_block_addr
    }

    fn is_enough_for_free_block(&self) -> bool {
        let addr_bound = unsafe { self.free_block_ptr.add(self.free_block_len + 1) as usize };
        let data_addr = unsafe { self.data_block_ptr.byte_add(self.size) as usize };
        addr_bound <= data_addr
    }

    fn push_data_block(&mut self, value: DataBlockInfo) {
        assert!(self.is_enough_for_data_block());
        unsafe {
            self.data_block_ptr.add(self.data_block_len).write(value);
        }
        self.data_block_len += 1;
    }

    fn merge_push_free_block(&mut self, value: MemoryBlockInfo) {
        todo!("merge");
        assert!(self.is_enough_for_free_block());
        unsafe {
            self.free_block_ptr.add(self.free_block_len).write(value);
        }
        self.free_block_len += 1;
    }

    fn take_free_block(&mut self, index: usize) -> MemoryBlockInfo {
        assert!(index < self.free_block_len, "Index is out of bounds");
        unsafe {
            let ptr = self.free_block_ptr.add(index);
            let block = ptr.read();
            ptr.copy_from(ptr.add(1), self.free_block_len - index - 1);
            self.free_block_len -= 1;
            block
        }
    }

    fn free_blocks(&self) -> &[MemoryBlockInfo] {
        unsafe { slice::from_raw_parts(self.free_block_ptr, self.free_block_len) }
    }

    fn base_ptr(&self) -> *mut u8 {
        // data block pointer is the pointer returned by alloc
        self.data_block_ptr.cast()
    }
}

#[derive(Debug)]
struct Data {
    size: usize,
}

impl Data {
    fn new(size: usize) -> Self {
        Self { size }
    }
}

pub struct MemoryManager {
    heap: WindowsHeap,
    header: Header,
    data: Data,
    drops: Vec<fn(*mut u8, usize)>,
}

impl MemoryManager {
    pub const ALLOCATION_GRANULARITY: usize = 16;

    pub fn new() -> Result<Self> {
        // Choose allocation granularity for data blocks sush as all the alignments would be satisfied
        // Currently the biggest align of primitive type is 16 bytes (u128)
        // Assuming that we choose granularity of each data block as 16 bytes
        // This way all storing types would be properly aligned as their alignments is 16 bytes or less
        // And data reallocation wouldn't require any relocation of data blocks for proper alignment assurance

        assert!(WindowsHeap::allocation_granularity() >= Self::ALLOCATION_GRANULARITY);

        let page_size = WindowsHeap::page_size();
        let heap = WindowsHeap::new(page_size * 32)?;
        let size = Self::get_granular_size(kilo(100));
        let ptr = heap.alloc(size)?;

        let header_size = Self::get_granular_size(kilo(4));
        let header = Header::new(ptr, header_size);
        let data = Data::new(size - header_size);

        Ok(Self {
            heap,
            header,
            data,
            drops: Vec::new(),
        })
    }

    pub fn new_data_cell<T>(&mut self) -> Result<DataCell<T>> {
        _ = Self::assert_align::<T>()?;

        let drop = |ptr: *mut u8, len: usize| unsafe {
            let ptr: *mut [T] = slice::from_raw_parts_mut(ptr.cast(), len);
            ptr::drop_in_place(ptr);
        };

        self.drops.push(drop);

        if !self.header.is_enough_for_data_block() {
            _ = self.extend_header()?;
        }

        let descriptor = Descriptor(self.header.data_block_len);
        let data_cell = DataCell::new(descriptor);
        let data_block = DataBlockInfo::default();
        self.header.push_data_block(data_block);

        Ok(data_cell)
    }

    pub fn data_cell_with_capacity<T>(&mut self) -> Result<DataCell<T>> {
        todo!()
    }

    fn assert_align<T>() -> Result<()> {
        if Self::ALLOCATION_GRANULARITY >= align_of::<T>() {
            Ok(())
        } else {
            Err(Error::UnsupportedAlignment(any::type_name::<T>()))
        }
    }

    fn get_granular_size(size: usize) -> usize {
        size + size % Self::ALLOCATION_GRANULARITY
    }

    fn get_data_block(&self, descriptor: &Descriptor) -> &mut DataBlockInfo {
        unsafe {
            let ptr = self.header.data_block_ptr.add(descriptor.0);
            &mut *ptr
        }
    }

    fn extend_data_cell(&mut self, descriptor: &Descriptor, new_size: usize) {
        let data_block = self.get_data_block(descriptor);
        assert!(new_size >= data_block.bl.size);

        let granular_size = Self::get_granular_size(new_size);
        let opt = self.find_fit(granular_size);
        if opt.is_none() {
            self.extend_data(granular_size * 2);
        }
        let index = opt.unwrap_or(self.find_fit(granular_size).unwrap());

        let mut block = self.header.take_free_block(index);
        let occupied = block.reduce(granular_size);
        if block.size > 0 {
            self.header.merge_push_free_block(block);
        }

        todo!()
    }

    fn find_fit(&self, size: usize) -> Option<usize> {
        let mut ret = None;
        for (i, block) in self.header.free_blocks().iter().enumerate() {
            if block.size >= size {
                ret = Some(i);
                break;
            }
        }
        ret
    }

    fn extend_data(&mut self, by: usize) -> Result<()> {
        let new_total_size = self.header.size + self.data.size + by;
        let ptr = self.heap.realloc(
            unsafe { NonNull::new_unchecked(self.header.base_ptr()).cast() },
            new_total_size,
        )?;
        self.header.upsize_on_reallocation(ptr, self.header.size);
        todo!("Update new free space");
        Ok(())
    }

    fn reduce_data_cell(&mut self, descriptor: &Descriptor) {
        todo!()
    }

    fn extend_header(&mut self) -> Result<()> {
        let data_size = self.data.size;
        let new_header_size = Self::get_granular_size(self.header.size * 2);
        let new_total_size = new_header_size + data_size;
        let ptr = self.heap.realloc(
            unsafe { NonNull::new_unchecked(self.header.base_ptr()).cast() },
            new_total_size,
        )?;

        // Shift forward data
        unsafe {
            let new_data_ptr = ptr.as_ptr().add(new_header_size);
            let old_data_ptr = ptr.as_ptr().add(self.header.size);
            new_data_ptr.copy_from(old_data_ptr, data_size);
        }
        self.header.upsize_on_reallocation(ptr, new_header_size);

        Ok(())
    }

    fn optimize_fragmentation(&mut self) {
        todo!()
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        todo!("call drop functions for each data");
    }
}

#[derive(Debug)]
pub struct DataCell<T> {
    pd: PhantomData<T>,
    descriptor: Descriptor,
    len: usize,
}

impl<T> DataCell<T> {
    fn new(descriptor: Descriptor) -> Self {
        Self {
            pd: Default::default(),
            descriptor,
            len: 0,
        }
    }

    pub fn len(&self, mm: &MemoryManager) -> usize {
        self.len
    }

    pub fn slice<'a>(&self, mm: &'a MemoryManager) -> &'a [T] {
        let data_block = mm.get_data_block(&self.descriptor);
        unsafe {
            let ptr = mm
                .header
                .base_ptr()
                .cast::<T>()
                .byte_add(data_block.bl.offset);
            slice::from_raw_parts(ptr, data_block.len)
        }
    }

    pub fn slice_mut<'a>(&mut self, mm: &'a MemoryManager) -> &'a mut [T] {
        let data_block = mm.get_data_block(&self.descriptor);
        unsafe {
            let ptr = mm
                .header
                .base_ptr()
                .cast::<T>()
                .byte_add(data_block.bl.offset);
            slice::from_raw_parts_mut(ptr, data_block.len)
        }
    }

    pub fn push(&mut self, value: T, mm: &mut MemoryManager) {
        let data_block = mm.get_data_block(&self.descriptor);
        let is_enough_space = data_block.len * size_of::<T>() <= data_block.bl.size;
        if !is_enough_space {
            let new_size = size_of::<T>() * data_block.len * 2;
            mm.extend_data_cell(&self.descriptor, new_size);
        }

        unsafe {
            mm.header.base_ptr().cast::<T>().add(self.len).write(value);
        }

        mm.get_data_block(&self.descriptor).len += 1;
    }

    pub fn take_at(&mut self, index: usize, mm: &MemoryManager) -> T {
        todo!()
    }

    pub fn swap_take(&mut self, index: usize, mm: &MemoryManager) -> T {
        todo!()
    }

    pub fn shrink(&mut self, mm: &mut MemoryManager) {
        todo!()
    }
}

pub struct Engine;

impl Engine {
    pub fn run(f: impl Fn(&mut MemoryManager) -> Result<()>) -> Result<()> {
        let mm = MemoryManager::new()?;
        let rm = ResourceManager::new();

        todo!()
    }
}

struct GraphicsBackend {
    gl: Gl,
    window: PWindow,
}

pub struct Context {
    pub mm: MemoryManager,
    pub rm: ResourceManager,
    pub scene: Scene,
}

pub struct ResourceManager {
    // ...
}

impl ResourceManager {
    fn new() -> Self {
        Self {}
    }
}

pub struct Scene {
    // active systems
    // components
    // some data
}

impl Scene {
    pub fn get_component<T>() -> DataCell<T> {
        // This function has to do some magic
        // Somehow transform type T into a number and retrieve actual data cell by this number
        todo!()
    }
}

pub trait Component<T: Sized + Serialize + for<'a> Deserialize<'a> = Self> {
    fn create() -> Self;
}

enum Serialized {
    Json(String),
    Binary, // something here
}

struct RawValue(*mut u8, Layout);

impl RawValue {
    fn create<T>(value: T) -> Result<Self> {
        unsafe {
            let layout = Layout::for_value(&value);
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                return Err(Error::MemoryError);
            }
            ptr.cast::<T>().write(value);

            Ok(Self(ptr, layout))
        }
    }

    fn dealloc<T>(self) -> T {
        unsafe {
            let ret = self.0.cast::<T>().read();
            alloc::dealloc(self.0, self.1);
            ret
        }
    }
}

impl Drop for RawValue {
    fn drop(&mut self) {
        todo!()
    }
}

pub struct TypeInfo {
    id: TypeId,
    name: &'static str,
    size: usize,
    align: usize,
    create: fn() -> Result<RawValue>,
    serialize: fn() -> Serialized,
    deserialize: fn(Serialized) -> Result<RawValue>,
}

pub struct TypeRegistry {
    reg: DataCell<TypeInfo>,
}

impl TypeRegistry {
    pub fn add_type<T>(&mut self)
    where
        T: 'static + Component + Serialize + for<'a> Deserialize<'a>,
    {
        let type_name = any::type_name::<T>();
        let type_id = any::TypeId::of::<T>();

        let create = || -> Result<RawValue> {
            let component = T::create();
            RawValue::create(component)
        };
        todo!();
    }
}

pub enum SystemType {
    Update,
    Init,
    Trigger,
    Start,
    End,
}

struct SystemInfo<T> {
    name: &'static str,
    ptr: T,
}

pub struct SystemContainer {
    update: DataCell<fn()>,
    init: DataCell<fn()>,
    trigger: DataCell<fn()>,
    start: DataCell<fn()>,
    end: DataCell<fn()>,
}

impl SystemContainer {}

pub struct SystemContainerBuilder {
    update: Vec<fn()>,
    init: Vec<fn()>,
    trigger: Vec<fn()>,
    start: Vec<fn()>,
    end: Vec<fn()>,
}

struct Systems {
    sys: Vec<fn()>,
}

impl Systems {
    fn add<S>(&mut self)
    where
        S: UpdateSystem,
    {
        let a = [1, 2, 3];
        for num in a.iter().take(3) {}
        let ptr: fn() = S::update;
        self.sys.push(ptr);
    }
}

trait UpdateSystem {
    fn update();
}

trait InitSystem {
    // pass entities that has a newly created component attached (somehow mark those components)
    fn init();
}

trait TriggerSystem {
    // pass entities that are to be notified of trigger collision
    fn on_trigger();
}

trait SceneStartSystem {
    fn start();
}

trait SceneEndSystem {
    fn end();
}

#[derive(Debug)]
struct Entity {
    id: EntityId,
    is_alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntityId {
    id: u32,
    gen: u32,
}

struct ComponentWrapper<T: Component + Serialize + for<'a> Deserialize<'a> + Sized> {
    component: T,
    owner: EntityId,
}

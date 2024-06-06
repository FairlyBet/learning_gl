use crate::{
    linear::Transform,
    runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
use std::{
    alloc::{self, Layout},
    any::{self, TypeId},
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::{self, addr_of, NonNull},
    slice,
};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Memory,
        SystemInformation::{self, SYSTEM_INFO},
    },
};

fn aligned_ptr<T>(ptr: *const u8) -> *mut T {
    let addr = ptr as usize;
    let align = align_of::<T>();
    let aligned_addr = (addr + align - 1) & !(align - 1);

    aligned_addr as *mut T
}

fn write_backwards<T>(buff: (*mut u8, usize), value: T, count: usize) {
    unsafe {
        let ptr = buff.0.add(buff.1 - size_of::<T>() * (count + 1));
        ptr.cast::<T>().write(value);
    };
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
struct MemoryBlock {
    offset: usize,
    size: usize,
}

impl MemoryBlock {
    fn new(offset: usize, size: usize) -> Self {
        Self { offset, size }
    }
}

#[derive(Debug, Clone, Copy)]
struct DataBlock {
    block: MemoryBlock,
    aligned_ptr: NonNull<u8>,
}

impl DataBlock {
    fn new(block: MemoryBlock, aligned_ptr: NonNull<u8>) -> Self {
        Self { block, aligned_ptr }
    }
}

#[derive(Debug)]
struct Descriptor(usize);

#[derive(Debug)]
struct Header {
    data_block_ptr: *mut MemoryBlock,
    data_block_len: usize,
    free_block_ptr: *mut MemoryBlock,
    free_block_len: usize,
    size: usize,
}

impl Header {
    fn new(ptr: NonNull<u8>, size: usize) -> Self {
        Self::assert_align(ptr);

        let offset = size / size_of::<MemoryBlock>() / 2 * size_of::<MemoryBlock>(); // Calculating aligned offset
        let free_block_ptr = unsafe { ptr.cast::<MemoryBlock>().as_ptr().byte_add(offset) };

        Self {
            data_block_ptr: ptr.cast().as_ptr(),
            data_block_len: 0,
            free_block_ptr,
            free_block_len: 0,
            size,
        }
    }

    fn upsize_on_reallocation(&mut self, new_ptr: NonNull<u8>, new_size: usize) {
        Self::assert_align(new_ptr);

        assert!(new_size >= self.size);

        // Shift forward free blocks data
        unsafe {
            let offset = self.free_block_ptr as usize - self.data_block_ptr as usize;
            let old_free_block_ptr = new_ptr.cast::<MemoryBlock>().as_ptr().byte_add(offset);
            let offset = new_size / size_of::<MemoryBlock>() / 2 * size_of::<MemoryBlock>(); // Calculating aligned offset
            let new_free_block_ptr = new_ptr.cast::<MemoryBlock>().as_ptr().byte_add(offset);

            new_free_block_ptr.copy_from(old_free_block_ptr, self.free_block_len);
            self.free_block_ptr = new_free_block_ptr;
        }

        self.data_block_ptr = new_ptr.cast().as_ptr();
        self.size = new_size;
    }

    fn assert_align(ptr: NonNull<u8>) {
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % align_of::<MemoryBlock>(), 0);
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

    fn push_data_block(&mut self, value: MemoryBlock) {
        assert!(self.is_enough_for_data_block());
        unsafe {
            self.data_block_ptr.add(self.data_block_len).write(value);
        }
        self.data_block_len += 1;
    }

    fn push_free_block(&mut self, value: MemoryBlock) {
        assert!(self.is_enough_for_free_block());
        unsafe {
            self.free_block_ptr.add(self.free_block_len).write(value);
        }
        self.free_block_len += 1;
    }

    fn remove_free_block(&mut self, index: usize) {
        assert!(index < self.free_block_len, "Index is out of bounds");
        unsafe {
            let ptr = self.free_block_ptr.add(index);
            ptr.copy_from(ptr.add(1), self.free_block_len - index - 1);
        }
        self.free_block_len -= 1;
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

        assert_eq!(
            WindowsHeap::allocation_granularity() % Self::ALLOCATION_GRANULARITY,
            0
        );

        let page_size = WindowsHeap::page_size();
        let size = page_size * 31;
        let heap = WindowsHeap::new(size + page_size)?;
        let ptr = heap.alloc(size)?;

        let header_size = Self::ALLOCATION_GRANULARITY * 256;
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
            _ = self.upsize_header()?;
        }

        let descriptor = Descriptor(self.header.data_block_len);
        let data_cell = DataCell::new(descriptor);
        let data_block = MemoryBlock::default();
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

    fn get_data_block(&self, descriptor: &Descriptor) -> &MemoryBlock {
        unsafe {
            let ptr = self.header.data_block_ptr.add(descriptor.0);
            &*ptr
        }
    }

    fn upsize_data_cell(&mut self, descriptor: &Descriptor) {
        todo!()
    }

    fn shrink_data_cell(&mut self, descriptor: &Descriptor) {
        todo!()
    }

    fn upsize_data(&mut self) -> Result<()> {
        let new_data_size = self.data.size * 2;
        let new_total_size = new_data_size + self.header.size;
        let ptr = self.heap.realloc(
            unsafe { NonNull::new_unchecked(self.header.base_ptr()).cast() },
            new_total_size,
        )?;
        self.header.upsize_on_reallocation(ptr, self.header.size);
        todo!("Update free block data");
        Ok(())
    }

    fn upsize_header(&mut self) -> Result<()> {
        let data_size = self.data.size;
        let new_header_size = self.header.size * 2;
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
            let ptr = mm.header.base_ptr().cast::<T>().byte_add(data_block.offset);
            slice::from_raw_parts(ptr, self.len)
        }
    }

    pub fn slice_mut<'a>(&mut self, mm: &'a MemoryManager) -> &'a mut [T] {
        let data_block = mm.get_data_block(&self.descriptor);
        unsafe {
            let ptr = mm.header.base_ptr().cast::<T>().byte_add(data_block.offset);
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }

    pub fn push(&mut self, value: T, mm: &mut MemoryManager) {
        let data_block = mm.get_data_block(&self.descriptor);

        if self.len * size_of::<T>() > data_block.size {
            mm.upsize_data_cell(&self.descriptor);
        }

        unsafe {
            mm.header.base_ptr().cast::<T>().add(self.len).write(value);
        }

        self.len += 1;
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

pub struct Engine {
    mm: MemoryManager,
}

impl Engine {
    pub fn new() -> Result<Self> {
        let mm = MemoryManager::new()?;
        Ok(Self { mm })
    }

    fn run(mut self, f: impl FnOnce(&mut MemoryManager) -> Result<()>) -> Result<()> {
        f(&mut self.mm)
    }
}

pub struct Scene;

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

pub struct TypeInfo {
    id: TypeId,
    name: &'static str,
    size: usize,
    align: usize,
    create: fn() -> *mut u8,
    serialize: fn() -> Serialized,
    deserialize: fn() -> *mut u8,
}

pub struct TypeRegistry {
    reg: DataCell<TypeInfo>,
}

impl TypeRegistry {
    pub fn add_type<T: 'static + Component + Serialize + for<'a> Deserialize<'a>>(&mut self) {
        let type_name = any::type_name::<T>();
        let type_id = any::TypeId::of::<T>();

        let create = || -> *mut u8 {
            let component = T::create();
            let layout = Layout::for_value(&component);
            let ptr = unsafe { alloc::alloc(layout) };
            ptr.cast()
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

pub struct SystemRegistry {
    // arrays of systems
}

impl SystemRegistry {}

struct Systems {
    sys: Vec<fn()>,
}

impl Systems {
    fn add<S>(&mut self)
    where
        S: UpdateSystem,
    {
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

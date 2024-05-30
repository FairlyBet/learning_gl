use crate::runtime::{Error, Result};
use std::{
    cell::Cell,
    ffi::c_void,
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::{self, NonNull},
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

fn windows_page_size() -> usize {
    let mut system_info = SYSTEM_INFO::default();
    unsafe {
        SystemInformation::GetSystemInfo(&mut system_info);
    }
    system_info.dwPageSize as usize
}

#[derive(Debug)]
struct WindowsHeap(HANDLE);

impl WindowsHeap {
    const ALLOCATION_ALIGNMENT: usize = 0x10;

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

    fn alloc(&self, size: usize) -> Result<NonNull<c_void>> {
        unsafe {
            let ptr = Memory::HeapAlloc(self.0, Memory::HEAP_NONE, size);
            if ptr == ptr::null_mut() {
                return Err(Error::MemoryError);
            }
            return Ok(NonNull::new_unchecked(ptr));
        }
    }

    fn realloc(&self, ptr: NonNull<c_void>, size: usize) -> Result<NonNull<c_void>> {
        unsafe {
            let ptr = Memory::HeapReAlloc(self.0, Memory::HEAP_NONE, Some(ptr.as_ptr()), size);
            if ptr == ptr::null_mut() {
                return Err(Error::MemoryError);
            }
            return Ok(NonNull::new_unchecked(ptr));
        }
    }

    fn free(&self, ptr: NonNull<c_void>) -> Result<()> {
        unsafe {
            if Memory::HeapFree(self.0, Memory::HEAP_NONE, Some(ptr.as_ptr())).is_ok() {
                return Ok(());
            }
            return Err(Error::MemoryError);
        }
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
    fn new(offset: usize, size: usize, aligned_ptr: NonNull<u8>) -> Self {
        Self {
            block: MemoryBlock { offset, size },
            aligned_ptr,
        }
    }
}

#[derive(Debug)]
pub struct Descriptor(usize);

#[derive(Debug)]
struct HeaderZone {
    ptr: NonNull<u8>,
    size: usize,
    data_record_len: usize,
    free_block_offset: usize,
    free_block_len: usize,
}

impl HeaderZone {
    fn new(ptr: NonNull<u8>, size: usize) -> Self {
        assert_eq!(
            ptr.as_ptr() as usize % align_of::<MemoryBlock>(),
            0,
            "Invalid pointer alignment"
        );
        assert_eq!(align_of::<MemoryBlock>(), align_of::<DataBlock>());

        let free_block_offset = size / 2;

        Self {
            ptr,
            size,
            data_record_len: 0,
            free_block_offset,
            free_block_len: 0,
        }
    }

    fn is_enuogh_for_data_record(&self) -> bool {
        (self.data_record_len + 1) * size_of::<DataBlock>() <= self.free_block_offset
    }

    fn is_enough_for_free_block(&self) -> bool {
        self.free_block_offset + (self.free_block_len + 1) * size_of::<MemoryBlock>() <= self.size
    }

    fn push_data_record(&mut self, value: DataBlock) {
        assert!(self.is_enuogh_for_data_record());
        unsafe {
            self.ptr
                .as_ptr()
                .cast::<DataBlock>()
                .add(self.data_record_len)
                .write(value);
        }
        self.data_record_len += 1;
    }

    fn push_free_block(&mut self, value: MemoryBlock) {
        assert!(self.is_enough_for_free_block());
        unsafe {
            self.ptr
                .as_ptr()
                .add(self.free_block_offset)
                .cast::<MemoryBlock>()
                .add(self.free_block_len)
                .write(value);
        }
        self.free_block_len += 1;
    }

    fn free_blocks_mut(&mut self) -> &mut [MemoryBlock] {
        unsafe {
            let ptr = self
                .ptr
                .as_ptr()
                .add(self.free_block_offset)
                .cast::<MemoryBlock>();
            slice::from_raw_parts_mut(ptr, self.free_block_len)
        }
    }

    fn remove_free_block(&mut self, index: usize) {
        assert!(index < self.free_block_len, "Index is out of bounds");
        unsafe {
            let ptr = self
                .ptr
                .as_ptr()
                .add(self.free_block_offset)
                .cast::<MemoryBlock>()
                .add(index);
            ptr.copy_from(ptr.add(1), self.free_block_len - index - 1);
        }
        self.free_block_len -= 1;
    }

    fn upsize_on_reallocation(&mut self, new_ptr: NonNull<u8>, new_size: usize) {
        assert!(new_size >= self.size);

        if new_size == self.size {
            return;
        }

        unsafe {
            let old_free_block_ptr = new_ptr
                .as_ptr()
                .add(self.free_block_offset)
                .cast::<MemoryBlock>();

            let new_free_block_offset = new_size / 2;
            let new_free_block_ptr = new_ptr
                .as_ptr()
                .add(new_free_block_offset)
                .cast::<MemoryBlock>();

            new_free_block_ptr.copy_from(old_free_block_ptr, self.free_block_len);
        }

        self.ptr = new_ptr;
        self.size = new_size;
    }

    fn get_data_block(&self, descriptor: &Descriptor) -> &DataBlock {
        unsafe {
            let ptr = self.ptr.cast::<DataBlock>().as_ptr().add(descriptor.0);
            let ref_ = &*ptr;
            ref_
        }
    }
}

struct DataZone {
    ptr: NonNull<u8>,
    size: usize,
}

impl DataZone {
    fn new(ptr: NonNull<u8>, size: usize) -> Self {
        Self { ptr, size }
    }

    fn get_data_ptr<T>(&self, data_block: &DataBlock) -> *mut T {
        todo!()
    }
}

#[derive(Debug)]
pub struct MemoryManager {
    heap: WindowsHeap,
    ptr: NonNull<u8>,
    total_size: usize,
    header: HeaderZone,
}

impl MemoryManager {
    fn new() -> Result<Self> {
        let page_size = windows_page_size();
        let total_pages = 20;
        let heap = WindowsHeap::new((total_pages + 1) * page_size)?;
        let ptr = heap.alloc(total_pages * page_size)?.cast();
        let header_pages = 1;
        let mut header = HeaderZone::new(ptr, header_pages * page_size);
        let free_block = MemoryBlock::new(0, (total_pages - header_pages) * page_size);
        header.push_free_block(free_block);

        Ok(Self {
            heap,
            ptr,
            total_size: total_pages * page_size,
            header,
        })
    }

    fn data_ptr(&self) -> *mut u8 {
        unsafe { self.ptr.as_ptr().add(self.header.size) }
    }

    fn resize_header(&mut self) -> Result<()> {
        let data_size = self.total_size - self.header.size;
        let new_header_size = self.header.size * 2;
        let new_total_size = new_header_size + data_size;
        let ptr = self
            .heap
            .realloc(self.ptr.cast(), new_total_size)?
            .cast::<u8>();

        // Move data zone
        unsafe {
            let new_data_ptr = ptr.as_ptr().add(new_header_size);
            let data_ptr = ptr.as_ptr().add(self.header.size);
            new_data_ptr.copy_from(data_ptr, data_size);
        }

        self.ptr = ptr;
        self.total_size = new_header_size;
        self.header.upsize_on_reallocation(ptr, new_header_size);

        Ok(())
    }

    fn get_data<T>(&self, descriptor: &Descriptor) -> &mut [T] {
        todo!()
        // let data_block = self.header.get_data_block(descriptor);
        // unsafe {
        //     let ptr = self.ptr.as_ptr().add(data_block.block.offset).cast::<T>();
        //     slice::from_raw_parts_mut(ptr, data_block.len)
        // }
    }

    fn get_data_block(&self, descriptor: &Descriptor) -> &DataBlock {
        self.header.get_data_block(descriptor)
    }

    pub fn create_data_cell<T>(&mut self) -> Result<DataCell<T>> {
        // if !self.header.is_enuogh_for_data_record() {
        //     self.resize_header()?;
        // }

        // let descriptor = Descriptor(self.header.data_record_len);
        // let data_cell = DataCell::new(descriptor);
        // self.header.push_data_record(DataBlock::default());

        // Ok(data_cell)
        todo!()
    }

    pub fn optimize_fragmentation(&mut self) {
        todo!()
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
        // mm.get_data(&self.descriptor)
        todo!()
    }

    pub fn slice_mut<'a>(&mut self, mm: &'a MemoryManager) -> &'a mut [T] {
        // mm.get_data(&self.descriptor)
        todo!()
    }

    pub fn push(&mut self, value: T, mm: &mut MemoryManager) {
        todo!()
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

struct Engine {
    mm: MemoryManager,
}

impl Engine {
    fn run(mut self) {}
}

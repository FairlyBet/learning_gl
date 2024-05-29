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

#[derive(Debug, Default, Clone, Copy)]
struct DataBlock {
    block: MemoryBlock,
    len: usize,
}

impl DataBlock {
    fn new(offset: usize, size: usize, len: usize) -> Self {
        Self {
            block: MemoryBlock { offset, size },
            len,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Descriptor {
    index: usize,
}

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
}

#[derive(Debug)]
struct WindowsHeap {
    handle: HANDLE,
    page_size: usize,
}

impl WindowsHeap {
    const ALLOCATION_ALIGNMENT: usize = 0x10;

    fn new(pages: usize) -> Result<Self> {
        unsafe {
            let mut system_info = SYSTEM_INFO::default();
            SystemInformation::GetSystemInfo(&mut system_info);
            let page_size = system_info.dwPageSize as usize;
            match Memory::HeapCreate(
                Memory::HEAP_NO_SERIALIZE | Memory::HEAP_CREATE_ALIGN_16,
                page_size * pages,
                0,
            ) {
                Ok(handle) => Ok(Self { handle, page_size }),
                Err(_) => Err(Error::MemoryError),
            }
        }
    }

    fn alloc(&self, size: usize) -> Result<NonNull<c_void>> {
        unsafe {
            let ptr = Memory::HeapAlloc(self.handle, Memory::HEAP_NONE, size);
            if ptr == ptr::null_mut() {
                return Err(Error::MemoryError);
            }
            return Ok(NonNull::new_unchecked(ptr));
        }
    }

    fn realloc(&self, ptr: NonNull<c_void>, size: usize) -> Result<NonNull<c_void>> {
        unsafe {
            let ptr = Memory::HeapReAlloc(self.handle, Memory::HEAP_NONE, Some(ptr.as_ptr()), size);
            if ptr == ptr::null_mut() {
                return Err(Error::MemoryError);
            }
            return Ok(NonNull::new_unchecked(ptr));
        }
    }

    fn free(&self, ptr: NonNull<c_void>) -> Result<()> {
        unsafe {
            if Memory::HeapFree(self.handle, Memory::HEAP_NONE, Some(ptr.as_ptr())).is_ok() {
                return Ok(());
            }
            return Err(Error::MemoryError);
        }
    }
}

impl Drop for WindowsHeap {
    fn drop(&mut self) {
        unsafe {
            assert!(Memory::HeapDestroy(self.handle).is_ok());
        }
    }
}

#[derive(Debug)]
pub struct DataManager {
    heap: WindowsHeap,
    ptr: NonNull<u8>,
    total_pages: usize,
    header: HeaderZone,
    header_pages: usize,
}

impl DataManager {
    pub fn new() -> Result<Self> {
        let total_pages = 20;
        let heap = WindowsHeap::new(total_pages + 1)?;
        let ptr = heap.alloc(total_pages * heap.page_size)?.cast();
        let header_pages = 1;
        let mut header = HeaderZone::new(ptr, header_pages * heap.page_size);

        let free_block = MemoryBlock::new(0, (total_pages - header_pages) * heap.page_size);
        header.push_free_block(free_block);

        Ok(Self {
            heap,
            ptr,
            total_pages,
            header,
            header_pages,
        })
    }

    pub fn register_data<T>(&mut self, count: usize) -> Result<DataCell<T>> {
        // if !self.header.is_enuogh_for_data_record() {
        //     self.resize_header_zone()?;
        // }

        // let data_key = DataKey::<T>::new(self.header.data_record_len);
        // self.header.push_data_record(DataBlock::default());

        // Ok(data_key)
        todo!()
    }

    fn data_pages(&self) -> usize {
        self.total_pages - self.header_pages
    }

    fn data_ptr(&self) -> *mut u8 {
        unsafe {
            self.ptr
                .as_ptr()
                .add(self.header_pages * self.heap.page_size)
        }
    }

    fn resize_header_zone(&mut self) -> Result<()> {
        let new_header_pages = self.header_pages * 2;
        let new_size = (new_header_pages + self.data_pages()) * self.heap.page_size;
        let ptr = self.heap.realloc(self.ptr.cast(), new_size)?.cast::<u8>();

        // Move data zone
        unsafe {
            let new_data_ptr = self
                .ptr
                .as_ptr()
                .add(new_header_pages * self.heap.page_size);
            new_data_ptr.copy_from(self.data_ptr(), self.data_pages() * self.heap.page_size);
        }

        self.ptr = ptr;
        self.total_pages = self.data_pages() + new_header_pages;
        self.header_pages = new_header_pages;
        self.header
            .upsize_on_reallocation(ptr, new_header_pages * self.heap.page_size);

        Ok(())
    }

    pub unsafe fn optimize_fragmentation(&self) {
        todo!()
    }
}

// impl<T> DataKey<T> {
//     fn new(index: usize) -> Self {
//         Self {
//             pd: Default::default(),
//             record_index: index,
//         }
//     }
// }

#[derive(Debug)]
pub struct DataCell<'a, T> {
    pd: PhantomData<T>,
    descriptor: Descriptor,
    dm: &'a DataManager,
}

impl<'a, T> DataCell<'a, T> {
    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn capacity(&self) -> usize {
        todo!()
    }

    pub fn push(&mut self, value: T) {
        todo!()
    }

    pub fn take_at(&mut self, index: usize) -> T {
        todo!()
    }

    pub fn swap_take(&mut self, index: usize) -> T {
        todo!()
    }

    pub fn slice(&self) -> &[T] {
        todo!()
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        todo!()
    }
}

struct MM;

impl MM {}

struct Object {
    // pd: PhantomData<&'a ()>,
}

impl Object {
    fn read<'a>(&self, mm: &'a MM) -> &'a [u32] {
        todo!()
    }

    fn read_mut<'a>(&mut self, mm: &'a MM) -> &'a mut [u32] {
        todo!()
    }

    fn write(&mut self, mm: &MM) {}

    fn take(&mut self, mm: &MM) {}

    fn push(&mut self, mm: &mut MM) {}
}

fn foo() {
    let mut mm = MM;
    let mut obj = Object {};
    obj.push(&mut mm);
}

use crate::runtime::{Error, Result};
use std::{
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
    data_block_ptr: NonNull<DataBlock>,
    data_block_len: usize,
    free_block_ptr: NonNull<MemoryBlock>,
    free_block_len: usize,
    size: usize,
}

impl Header {
    fn new(ptr: NonNull<u8>, size: usize) -> Self {
        assert_eq!(align_of::<MemoryBlock>(), align_of::<DataBlock>()); // Just in case

        Self::assert_align(ptr);

        let offset = size / size_of::<DataBlock>() / 2 * size_of::<DataBlock>();
        let free_block_ptr =
            unsafe { NonNull::new_unchecked(ptr.cast::<MemoryBlock>().as_ptr().byte_add(offset)) };

        Self {
            data_block_ptr: ptr.cast(),
            data_block_len: 0,
            free_block_ptr,
            free_block_len: 0,
            size,
        }
    }

    fn upsize_on_reallocation(&mut self, new_ptr: NonNull<u8>, new_size: usize) {
        Self::assert_align(new_ptr);

        assert!(new_size >= self.size);

        // Shift forward free block data
        unsafe {
            let offset =
                self.free_block_ptr.as_ptr() as usize - self.data_block_ptr.as_ptr() as usize;
            let old_free_block_ptr = new_ptr.cast::<MemoryBlock>().as_ptr().byte_add(offset);
            let offset = new_size / size_of::<DataBlock>() / 2 * size_of::<DataBlock>();
            let new_free_block_ptr = new_ptr.cast::<MemoryBlock>().as_ptr().byte_add(offset);

            new_free_block_ptr.copy_from(old_free_block_ptr, self.free_block_len);
            self.free_block_ptr = NonNull::new_unchecked(new_free_block_ptr);
        }
        todo!("Update data block aligned pointers on reallocation");
        self.data_block_ptr = new_ptr.cast();
        self.size = new_size;
    }

    fn assert_align(ptr: NonNull<u8>) {
        let addr = ptr.as_ptr() as usize;
        assert_eq!(addr % align_of::<MemoryBlock>(), 0);
    }

    fn is_enough_for_data_block(&self) -> bool {
        let addr_bound =
            unsafe { self.data_block_ptr.as_ptr().add(self.data_block_len + 1) as usize };
        let free_block_addr = self.free_block_ptr.as_ptr() as usize;
        addr_bound <= free_block_addr
    }

    fn is_enough_for_free_block(&self) -> bool {
        let addr_bound =
            unsafe { self.free_block_ptr.as_ptr().add(self.free_block_len + 1) as usize };
        let data_addr = unsafe { self.data_block_ptr.as_ptr().byte_add(self.size) as usize };
        addr_bound <= data_addr
    }

    fn push_data_block(&mut self, value: DataBlock) {
        assert!(self.is_enough_for_data_block());
        unsafe {
            self.data_block_ptr
                .as_ptr()
                .add(self.data_block_len)
                .write(value);
        }
        self.data_block_len += 1;
    }

    fn push_free_block(&mut self, value: MemoryBlock) {
        assert!(self.is_enough_for_free_block());
        unsafe {
            self.free_block_ptr
                .as_ptr()
                .add(self.free_block_len)
                .write(value);
        }
        self.free_block_len += 1;
    }

    fn remove_free_block(&mut self, index: usize) {
        assert!(index < self.free_block_len, "Index is out of bounds");
        unsafe {
            let ptr = self.free_block_ptr.as_ptr().add(index);
            ptr.copy_from(ptr.add(1), self.free_block_len - index - 1);
        }
        self.free_block_len -= 1;
    }

    fn base_ptr(&self) -> NonNull<u8> {
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

#[derive(Debug)]
pub struct MemoryManager {
    heap: WindowsHeap,
    header: Header,
    data: Data,
}

impl MemoryManager {
    fn new() -> Result<Self> {
        let page_size = windows_page_size();
        let size = page_size * 31;
        let heap = WindowsHeap::new(size + page_size)?;
        let ptr = heap.alloc(size)?;
        let header_size = WindowsHeap::ALLOCATION_ALIGNMENT * 128;
        let header = Header::new(ptr, header_size);
        let data = Data::new(size - header_size);

        Ok(Self { heap, header, data })
    }

    pub fn create_data_cell<T>(&mut self) -> Result<DataCell<T>> {
        if WindowsHeap::ALLOCATION_ALIGNMENT < align_of::<T>() {
            return Err(Error::UnsupportedAlignment);
        }

        if !self.header.is_enough_for_data_block() {
            self.resize_header()?;
        }

        let descriptor = Descriptor(self.header.data_block_len);
        let data_cell = DataCell::new(descriptor);
        let data_block = DataBlock::new(Default::default(), self.header.base_ptr().cast());
        self.header.push_data_block(data_block);

        Ok(data_cell)
    }

    fn resize_header(&mut self) -> Result<()> {
        let data_size = self.data.size;
        let new_header_size = self.header.size * 2;
        let new_total_size = new_header_size + data_size;
        let ptr = self.heap.realloc(self.header.base_ptr(), new_total_size)?;

        // Shift forward data
        unsafe {
            let new_data_ptr = ptr.as_ptr().add(new_header_size);
            let old_data_ptr = ptr.as_ptr().add(self.header.size);
            new_data_ptr.copy_from(old_data_ptr, data_size);
        }
        self.header.upsize_on_reallocation(ptr, new_header_size);

        Ok(())
    }

    fn get_data_block(&self, descriptor: &Descriptor) -> &DataBlock {
        unsafe {
            let ptr = self.header.data_block_ptr.as_ptr().add(descriptor.0);
            &*ptr
        }
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
        unsafe { slice::from_raw_parts(data_block.aligned_ptr.cast().as_ptr(), self.len) }
    }

    pub fn slice_mut<'a>(&mut self, mm: &'a MemoryManager) -> &'a mut [T] {
        let data_block = mm.get_data_block(&self.descriptor);
        unsafe { slice::from_raw_parts_mut(data_block.aligned_ptr.cast().as_ptr(), self.len) }
    }

    pub fn push(&mut self, value: T, mm: &mut MemoryManager) {
        let data_block = mm.get_data_block(&self.descriptor);
        let addr_bound = unsafe {
            data_block
                .aligned_ptr
                .cast::<T>()
                .as_ptr()
                .add(self.len + 1) as usize
        };
        let block_bound = unsafe {
            mm.header
                .base_ptr()
                .as_ptr()
                .add(data_block.block.offset + data_block.block.size) as usize
        };

        if addr_bound > block_bound {
            todo!("Resize block")
        }

        unsafe {
            data_block
                .aligned_ptr
                .cast::<T>()
                .as_ptr()
                .add(self.len)
                .write(value);
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

// struct Engine {
//     mm: MemoryManager,
// }

// impl Engine {
//     fn run(mut self) {}
// }

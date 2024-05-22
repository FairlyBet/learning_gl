use crate::runtime::{Error, Result};
use std::{
    ffi::c_void,
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr::{self, NonNull},
};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Memory,
        SystemInformation::{self, SYSTEM_INFO},
    },
};

#[derive(Debug)]
pub struct PrivateHeap {
    handle: HANDLE,
    page_size: usize,
}

impl PrivateHeap {
    const ALLOCATION_ALIGNMENT: usize = 0x10;

    pub fn new(page_count: usize) -> Result<Self> {
        unsafe {
            let mut system_info = SYSTEM_INFO::default();
            SystemInformation::GetSystemInfo(&mut system_info);
            let page_size = system_info.dwPageSize as usize;
            match Memory::HeapCreate(
                Memory::HEAP_NO_SERIALIZE | Memory::HEAP_CREATE_ALIGN_16,
                page_size * page_count,
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

impl Drop for PrivateHeap {
    fn drop(&mut self) {
        unsafe {
            assert!(Memory::HeapDestroy(self.handle).is_ok());
        }
    }
}

static mut IS_INSTANTIATED: bool = false;

#[derive(Debug)]
pub struct DataMemoryManager {
    base: NonNull<c_void>,
    size: usize,
    header_zone_len: usize,
    data_zone_len: usize,
    registered_data_count: usize,
    free_blocks_count: usize,
    heap: PrivateHeap,
}

impl DataMemoryManager {
    pub fn new() -> Result<Self> {
        assert!(!unsafe { IS_INSTANTIATED }, "This type has to be singleton");

        let heap = PrivateHeap::new(21)?;
        let initial_size = 21 * heap.page_size;
        let ptr = heap.alloc(initial_size)?;
        let mut self_ = Self {
            base: ptr,
            size: initial_size,
            header_zone_len: heap.page_size,
            data_zone_len: heap.page_size * 20,
            registered_data_count: 0,
            free_blocks_count: 0,
            heap,
        };
        assert_eq!(
            PrivateHeap::ALLOCATION_ALIGNMENT % align_of::<MemoryBlock>(),
            0,
            "Unsupported alignment"
        );
        assert_eq!(
            PrivateHeap::ALLOCATION_ALIGNMENT % align_of::<DataBlock>(),
            0,
            "Unsupported alignment"
        );

        // self_.
        unsafe { IS_INSTANTIATED = true };

        Ok(self_)
    }

    fn write_backwards<T>(buff: (*mut u8, usize), value: T, count: usize) {
        unsafe {
            let ptr = buff.0.add(buff.1 - size_of::<T>() * (count + 1));
            ptr.cast::<T>().write(value);
        };
    }

    pub fn register_data<T>(&mut self, count: usize) -> DataKey<T> {
        todo!()
        // if self.header_zone_freespace() < size_of::<DataBlock>() {
        //     self.resize_header_zone();
        // }

        // let b = DataBlock::default();

        // unsafe {
        //     self.base
        //         .add(self.registered_data_count * size_of::<DataBlock>())
        //         .cast::<DataBlock>()
        //         .write(b);
        // };

        // let key = DataKey {
        //     pd: Default::default(),
        //     data_block_offset: self.registered_data_count,
        // };

        // self.registered_data_count += 1;

        // key
    }

    fn header_zone_freespace(&self) -> usize {
        self.header_zone_len
            - (self.free_blocks_count * size_of::<MemoryBlock>()
                + self.registered_data_count * size_of::<DataBlock>())
    }

    fn resize_header_zone(&mut self) {
        todo!()
    }

    pub fn get_data<T>(&self, key: &DataKey<T>) -> DataCell<T> {
        todo!()
    }

    pub fn get_data_mut<T>(&mut self, key: &DataKey<T>) -> DataCellMut<T> {
        todo!()
    }

    pub fn optimize_fragmentation() {
        todo!()
    }

    pub fn temporal_data() {
        todo!()
    }
}

#[derive(Debug)]
struct HeaderZone {
    ptr: NonNull<u8>,
    capacity: usize,
    // data_records_offset = 0
    data_records_len: usize,
    free_blocks_len: usize,
    free_blocks_offset: usize,

    // free_blocks: NonNull<MemoryBlock>,
    // data_records: NonNull<DataBlock>,
}

impl HeaderZone {
    // fn is_enuogh_for_data_record(&self) -> bool {
    //     self.data_records.as_ptr() as usize + (self.data_records_len + 1) * size_of::<DataBlock>()
    //         - 1
    //         < self.free_blocks.as_ptr() as usize
    // }

    // fn is_enough_for_free_block(&self) -> bool {
    //     self.free_blocks.as_ptr() as usize + (self.free_blocks_len + 1) * size_of::<MemoryBlock>()
    //         - 1
    //         < self.end.as_ptr() as usize
    // }

    // fn push_data_record(&mut self, value: DataBlock) {
    //     assert!(self.is_enuogh_for_data_record());
    //     unsafe {
    //         self.data_records
    //             .as_ptr()
    //             .add(self.data_records_len)
    //             .write(value);
    //     }
    //     self.data_records_len += 1;
    // }

    // fn push_free_block(&mut self, value: MemoryBlock) {
    //     assert!(self.is_enough_for_free_block());
    //     unsafe {
    //         self.free_blocks
    //             .as_ptr()
    //             .add(self.free_blocks_len)
    //             .write(value);
    //     }
    //     self.free_blocks_len += 1;
    // }

    // fn upsize(&mut self, start: NonNull<u8>, end: NonNull<u8>) {
    // Assuming that data was reallocated
    // assert!(
    //     end.as_ptr() as usize - start.as_ptr() as usize
    //         > self.end.as_ptr() as usize - self.data_records.as_ptr() as usize
    // );
    // unsafe {
    //     let old_free_blocks_offset =
    //         self.free_blocks.as_ptr() as usize - self.data_records.as_ptr() as usize;
    //     let old_free_blocks = start
    //         .as_ptr()
    //         .add(old_free_blocks_offset)
    //         .cast::<MemoryBlock>();
    //     let new_free_blocks = start
    //         .as_ptr()
    //         .add((end.as_ptr() as usize - start.as_ptr() as usize) / 2)
    //         .cast::<MemoryBlock>();
    //     new_free_blocks.copy_from(old_free_blocks, self.free_blocks_len);
    //     self.free_blocks = NonNull::new_unchecked(new_free_blocks);
    //     self.data_records = start.cast();
    // }
    // }
}

fn aligned_ptr<T>(ptr: *const c_void) -> *mut T {
    let addr = ptr as usize;
    let align = align_of::<T>();
    let aligned_addr = (addr + align - 1) & !(align - 1);

    aligned_addr as *mut T
}

#[derive(Debug, Default, Clone, Copy)]
struct MemoryBlock {
    offset: usize,
    capacity: usize,
}

#[derive(Debug, Default, Clone, Copy)]
struct DataBlock {
    block: MemoryBlock,
    len: usize,
}

impl DataBlock {
    fn new(offset: usize, capacity: usize, len: usize) -> Self {
        Self {
            block: MemoryBlock { offset, capacity },
            len,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DataKey<T> {
    pd: PhantomData<T>,
    data_block_offset: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct DataCell<'a, T> {
    key: DataKey<T>,
    dm: &'a DataMemoryManager,
}

impl<'a, T> DataCell<'a, T> {
    pub fn slice(&self) -> &[T] {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn capacity(&self) -> usize {
        todo!()
    }
}

#[derive(Debug)]
pub struct DataCellMut<'a, T> {
    key: DataKey<T>,
    dm: &'a mut DataMemoryManager,
}

impl<'a, T> DataCellMut<'a, T> {
    pub fn push(&mut self, value: T) {
        todo!()
    }

    pub fn pop(&mut self) -> T {
        todo!()
    }

    pub fn take_at(&mut self, index: usize) -> T {
        todo!()
    }

    pub fn swap_take(&mut self) -> T {
        todo!()
    }

    pub fn slice(&self) -> &[T] {
        todo!()
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn capacity(&self) -> usize {
        todo!()
    }
}

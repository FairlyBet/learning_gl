use std::{
    alloc::{self, Layout, LayoutError},
    cell::Cell,
    ffi::c_void,
    marker::PhantomData,
    mem::{align_of, size_of},
    ptr,
};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Memory,
        SystemInformation::{self, SYSTEM_INFO},
    },
};

#[cfg(target_pointer_width = "64")]
const MEMORY_ALLOCATION_ALIGNMENT: usize = 0x10;

#[cfg(not(target_pointer_width = "64"))]
const MEMORY_ALLOCATION_ALIGNMENT: usize = 0x8;

struct HeapObject {
    handle: HANDLE,
    page_size: usize,
}

impl HeapObject {
    fn new(page_count: usize) -> Self {
        unsafe {
            let mut system_info = SYSTEM_INFO::default();
            SystemInformation::GetSystemInfo(&mut system_info);
            let page_size = system_info.dwPageSize as usize;
            let handle =
                Memory::HeapCreate(Memory::HEAP_NO_SERIALIZE, page_size * page_count, 0).unwrap();

            Self { handle, page_size }
        }
    }

    fn alloc(&self, size: usize) -> *mut c_void {
        unsafe { Memory::HeapAlloc(self.handle, Memory::HEAP_NO_SERIALIZE, size) }
    }

    fn realloc(&self, ptr: *mut c_void, size: usize) -> *mut c_void {
        unsafe { Memory::HeapReAlloc(self.handle, Memory::HEAP_NO_SERIALIZE, Some(ptr), size) }
    }

    fn free(&self, ptr: *mut c_void) {
        unsafe {
            assert!(Memory::HeapFree(self.handle, Memory::HEAP_NO_SERIALIZE, Some(ptr)).is_ok());
        }
    }
}

impl Drop for HeapObject {
    fn drop(&mut self) {
        unsafe {
            assert!(Memory::HeapDestroy(self.handle).is_ok());
        }
    }
}

const PAGE_SIZE: usize = 4096;

static mut IS_INSTANTIATED: bool = false;

#[derive(Debug)]
pub struct DataManager {
    base: *mut u8,
    layout: Layout,
    header_zone_len: usize,
    data_zone_len: usize,
    registered_data_count: usize,
    free_blocks_count: usize,
}

impl DataManager {
    pub fn new() -> Self {
        assert!(!unsafe { IS_INSTANTIATED });

        let initial_size = 16 * PAGE_SIZE;
        let workspace = Block {
            offset: PAGE_SIZE,
            capacity: PAGE_SIZE * 15,
        };
        let layout = Layout::from_size_align(initial_size, align_of::<usize>()).unwrap();

        let ptr;

        unsafe {
            ptr = alloc::alloc(layout);
            assert_ne!(ptr, ptr::null_mut());
            Self::write_backwards((ptr, PAGE_SIZE), workspace, 0);
            IS_INSTANTIATED = true;
        }

        Self {
            base: ptr,
            layout,
            header_zone_len: PAGE_SIZE,
            data_zone_len: PAGE_SIZE * 15,
            registered_data_count: 0,
            free_blocks_count: 1,
        }
    }

    fn write_backwards<T>(buff: (*mut u8, usize), value: T, count: usize) {
        unsafe {
            let ptr = buff.0.add(buff.1 - size_of::<T>() * (count + 1));
            ptr.cast::<T>().write(value);
        };
    }

    pub fn register_data<T>(&mut self, count: usize) -> DataKey<T> {
        if self.header_zone_freespace() < size_of::<DataBlock>() {
            self.resize_header_zone();
        }

        let b = DataBlock::default();

        unsafe {
            self.base
                .add(self.registered_data_count * size_of::<DataBlock>())
                .cast::<DataBlock>()
                .write(b);
        };

        let key = DataKey {
            pd: Default::default(),
            data_block_offset: self.registered_data_count,
        };

        self.registered_data_count += 1;

        key
    }

    fn header_zone_freespace(&self) -> usize {
        self.header_zone_len
            - (self.free_blocks_count * size_of::<Block>()
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

    pub fn temporal_data() {}
}

#[derive(Debug, Default, Clone, Copy)]
struct Block {
    offset: usize,
    capacity: usize,
}

#[derive(Debug, Default, Clone, Copy)]
struct DataBlock {
    block: Block,
    len: usize,
}

impl DataBlock {
    fn new(offset: usize, capacity: usize, len: usize) -> Self {
        Self {
            block: Block { offset, capacity },
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
    dm: &'a DataManager,
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
    dm: &'a mut DataManager,
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

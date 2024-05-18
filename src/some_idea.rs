use std::{
    alloc::{self, Layout, LayoutError},
    cell::Cell,
    marker::PhantomData,
    mem::align_of,
};

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Default, Clone, Copy)]
struct Block {
    pub offset: usize,
    pub len: usize,
}

#[derive(Debug)]
pub struct DataManager {
    base: *mut u8,
    layout: Layout,
    header_space: Block,
    work_space: Block,
    registered_data: u32,
    free_blocks: usize,
}

impl DataManager {
    pub fn new() -> Self {
        let initial_size = 16 * PAGE_SIZE;
        let layout = Layout::from_size_align(initial_size, align_of::<usize>()).unwrap();
        let ptr = unsafe { alloc::alloc_zeroed(layout) };
        let header_space = Block {
            offset: 0,
            len: PAGE_SIZE,
        };
        let work_space = Block {
            offset: PAGE_SIZE,
            len: PAGE_SIZE * 15,
        };
        let free_blocks = 1;
        unsafe { ptr.add(work_space.offset).cast::<Block>().write(work_space) }
        todo!()
    }

    pub fn register_data<T>(&mut self, count: usize) -> DataKey<T> {
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

#[derive(Debug, Clone, Copy)]
pub struct DataKey<T> {
    pd: PhantomData<T>,
    key: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct DataCell<'a, T> {
    key: DataKey<T>,
    mm: &'a DataManager,
}

impl<'a, T> DataCell<'a, T> {
    pub fn slice(&self) -> &[T] {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug)]
pub struct DataCellMut<'a, T> {
    key: DataKey<T>,
    mm: &'a mut DataManager,
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

    pub fn size(&self) -> usize {
        todo!()
    }
}

use std::{
    alloc::{self, Layout, LayoutError},
    cell::Cell,
    marker::PhantomData,
    mem::{align_of, size_of},
};

const PAGE_SIZE: usize = 4096;

static mut IS_INSTANTIATED: bool = false;

#[derive(Debug)]
pub struct DataManager {
    base: *mut u8,
    layout: Layout,
    data_zone_offset: usize,
    registered_data_count: usize,
    free_blocks_count: usize,
}

impl DataManager {
    pub fn new() -> Self {
        assert!(!unsafe { IS_INSTANTIATED });

        let initial_size = 16 * PAGE_SIZE;
        let workspace = Block {
            offset: PAGE_SIZE,
            len: PAGE_SIZE * 15,
        };
        let layout = Layout::from_size_align(initial_size, align_of::<usize>()).unwrap();

        let ptr;

        unsafe {
            ptr = alloc::alloc(layout);
            Self::write_back((ptr, PAGE_SIZE), workspace, 0);
            IS_INSTANTIATED = true;
        }

        Self {
            base: ptr,
            layout,
            data_zone_offset: PAGE_SIZE,
            registered_data_count: 0,
            free_blocks_count: 1,
        }
    }

    fn write_back<T>(buff: (*mut u8, usize), value: T, count: usize) {
        unsafe {
            let ptr = buff.0.add(buff.1 - size_of::<T>() * (count + 1));
            ptr.cast::<T>().write(value);
        };
    }

    pub fn register_data<T>(&mut self, count: usize) -> DataKey<T> {
        if self.header_zone_freespace() < size_of::<Block>() {
            self.resize_header_zone();
        }

        let b = Block { offset: 0, len: 0 };
        let ptr;
        unsafe {
            ptr = self.base.add(
                self.data_zone_offset - 1 - size_of::<Block>() * (self.registered_data_count + 1),
            );
            ptr.cast::<Block>().write(b);
        };

        let key = DataKey {
            pd: Default::default(),
            key: self.registered_data_count,
        };
        self.registered_data_count += 1;

        key
    }

    fn header_zone_freespace(&self) -> usize {
        self.data_zone_offset
            - (self.free_blocks_count + self.registered_data_count) * size_of::<Block>()
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
    len: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct DataKey<T> {
    pd: PhantomData<T>,
    key: usize,
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

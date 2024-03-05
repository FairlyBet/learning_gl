use fxhash::FxHasher32;
use std::{
    alloc::{self, Layout},
    collections::HashMap,
    hash::BuildHasherDefault,
    marker::PhantomData,
    mem::{self, size_of, ManuallyDrop, MaybeUninit},
    ptr, slice,
};

pub type FxHashMap32<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;

/// Elements don't drop! In order to drop elements convert into Vec<T>
#[derive(Debug)]
pub struct TypelessVec {
    buf: *mut u8,
    layout: Layout,
    len: usize,
}

impl TypelessVec {
    pub fn capacity<T>(&self) -> usize {
        self.layout.size() / size_of::<T>()
    }

    pub fn len<T>(&self) -> usize {
        self.len / size_of::<T>()
    }

    pub fn init<T>(capacity: usize) -> Self {
        let (buf, layout) = Self::alloc_buf::<T>(capacity);
        Self {
            buf,
            layout,
            len: 0,
        }
    }

    const fn empty() -> Self {
        Self {
            buf: ptr::null_mut(),
            layout: Layout::new::<()>(),
            len: 0,
        }
    }

    pub fn rewrite<T>(&mut self, index: usize, value: T) -> T {
        assert!(index < self.len::<T>(), "Index is out of bounds");
        unsafe {
            let old_value = self.buf.cast::<T>().add(index).read();
            self.buf.cast::<T>().add(index).write(value);
            old_value
        }
    }

    pub fn push<T>(&mut self, value: T) -> Reallocated {
        let mut realloc = Reallocated::No;
        if self.len + size_of::<T>() > self.layout.size() {
            let new_capacity = self.len::<T>() * 2 + (self.len::<T>() == 0) as usize;
            self.resize::<T>(new_capacity);
            realloc = Reallocated::Yes;
        }
        unsafe {
            self.buf.add(self.len).cast::<T>().write(value);
        }
        self.len += size_of::<T>();
        realloc
    }

    fn resize<T>(&mut self, new_capacity: usize) {
        let (buf, layout) = Self::alloc_buf::<T>(new_capacity);
        unsafe {
            buf.copy_from_nonoverlapping(self.buf, self.len);
        }
        self.dealloc();
        self.buf = buf;
        self.layout = layout;
    }

    fn alloc_buf<T>(n: usize) -> (*mut u8, Layout) {
        let layout = Layout::array::<T>(n).unwrap();
        let buf = unsafe { alloc::alloc(layout) };
        assert_ne!(buf, ptr::null_mut::<u8>(), "Cannot allocate memory");
        (buf, layout)
    }

    fn dealloc(&mut self) {
        if self.buf != ptr::null_mut() {
            unsafe {
                alloc::dealloc(self.buf, self.layout);
            }
        }
    }

    pub fn get_mut<T>(&mut self, index: usize) -> &mut T {
        assert!(index < self.len::<T>(), "Index out of bound");
        unsafe {
            let value_ptr = self.buf.add(index * size_of::<T>()) as *mut T;
            &mut (*value_ptr)
        }
    }

    pub fn get<T>(&self, index: usize) -> &T {
        assert!(index < self.len::<T>(), "Index out of bound");
        unsafe {
            let value_ptr = self.buf.add(index * size_of::<T>()) as *const T;
            &(*value_ptr)
        }
    }

    pub fn slice<T>(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.buf as *const T, self.len::<T>()) }
    }

    pub fn slice_mut<T>(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.buf as *mut T, self.len::<T>()) }
    }

    pub fn take_at<T>(&mut self, index: usize) -> T {
        assert!(index < self.len::<T>(), "Index is out of bounds");
        unsafe {
            let current = self.buf.cast::<T>().add(index);
            let value = current.read();
            let index_of_last = self.len::<T>() - 1;
            let num_to_shift = index_of_last - index;
            if num_to_shift > 0 {
                let next = current.add(1);
                current.copy_from(next, num_to_shift);
            }
            self.len -= size_of::<T>();
            value
        }
    }

    pub fn swap_take<T>(&mut self, index: usize) -> T {
        assert!(index < self.len::<T>(), "Index is out of bounds");
        unsafe {
            let last_index = self.len::<T>() - 1;
            let ptr = self.buf.cast::<T>();
            let value = ptr.add(index).read();
            if last_index > 0 {
                ptr.add(index).copy_from(ptr.add(last_index), 1);
            }
            self.len -= size_of::<T>();
            value
        }
    }

    pub fn clear<T>(&mut self) {
        todo!()
        // test when ptr is null
        // let slice = self.slice_mut::<T>();
        // self.len = 0;
        // unsafe {
        // ptr::drop_in_place(slice);
        // }
    }
}

impl Drop for TypelessVec {
    fn drop(&mut self) {
        // println!("Drop");
        self.dealloc();
    }
}

impl Default for TypelessVec {
    fn default() -> Self {
        TypelessVec::empty()
    }
}

impl<T> Into<Vec<T>> for TypelessVec {
    fn into(self) -> Vec<T> {
        let vec =
            unsafe { Vec::from_raw_parts(self.buf.cast(), self.len::<T>(), self.layout.size()) };
        ManuallyDrop::new(self);
        vec
    }
}

#[derive(Default, Debug)]
pub enum Reallocated {
    #[default]
    No,
    Yes,
}

#[derive(Debug)]
pub struct ArrayVec<T, const SIZE: usize> {
    buf: MaybeUninit<[T; SIZE]>,
    len: usize,
}

impl<T, const SIZE: usize> ArrayVec<T, SIZE> {
    pub fn new() -> Self {
        Self {
            buf: MaybeUninit::zeroed(),
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        assert!(self.len < SIZE, "Index is out of bounds");
        unsafe { self.buf.as_mut_ptr().cast::<T>().add(self.len).write(value) }; // Need to be tested
        self.len += 1;
        // unsafe {
        //     let zeroed = mem::replace(&mut (self.buf.assume_init_mut()[self.len]), value);
        //     ManuallyDrop::new(zeroed);
        // }
    }

    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.len < SIZE {
            unsafe { self.buf.as_mut_ptr().cast::<T>().add(self.len).write(value) }; // Need to be tested
            self.len += 1;
            Ok(())
        } else {
            Err(value)
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> &T {
        assert!(index < self.len, "Index is out of bounds");
        unsafe { &self.buf.assume_init_ref().get_unchecked(index) }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.len, "Index is out of bounds");
        unsafe { &mut self.buf.assume_init_mut()[index] }
    }

    pub fn slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.buf.assume_init_ref().as_ptr(), self.len) }
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.buf.assume_init_mut().as_mut_ptr(), self.len) }
    }
}

impl<T, const SIZE: usize> Drop for ArrayVec<T, SIZE> {
    fn drop(&mut self) {
        println!("Dropping {} elements", self.len);
        for i in 0..self.len {
            unsafe {
                let zeroed = MaybeUninit::<T>::zeroed();
                _ = mem::replace(&mut (self.buf.assume_init_mut()[i]), zeroed.assume_init());
            }
        }
    }
}

pub fn into_vec_clone<From, Into>(from: &Vec<From>) -> Vec<Into>
where
    From: core::convert::Into<Into> + Clone,
{
    from.iter()
        .map(|item| item.clone().into())
        .collect::<Vec<Into>>()
}

pub fn convert_vec<From, Into>(from: Vec<From>) -> Vec<Into>
where
    From: core::convert::Into<Into>,
{
    let mut vec = Vec::with_capacity(from.len());
    for item in from {
        vec.push(item.into());
    }
    vec
}

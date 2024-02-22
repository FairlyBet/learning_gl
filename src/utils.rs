use fxhash::FxHasher32;
use std::{
    alloc::{self, Layout},
    collections::HashMap,
    hash::BuildHasherDefault,
    marker::PhantomData,
    mem::{self, size_of, MaybeUninit},
    ptr, slice,
};

pub type FxHashMap32<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;

/// Do not store impl Drop types there!!!
pub struct UntypedVec {
    buf: *mut u8,
    layout: Layout,
    len: usize,
}

impl UntypedVec {
    pub fn init<T>(capacity: usize) -> Self {
        let (buf, layout) = Self::alloc_buf::<T>(capacity * size_of::<T>());
        Self {
            buf,
            layout,
            len: 0,
        }
    }

    fn empty() -> Self {
        Self {
            buf: ptr::null_mut::<u8>(),
            layout: Layout::new::<()>(),
            len: Default::default(),
        }
    }

    pub fn rewrite<T>(&mut self, value: T, index: usize) {
        assert!(index < self.len::<T>(), "Index is out of bound");
        unsafe {
            self.buf
                .add(index * size_of::<T>())
                .copy_from_nonoverlapping(&value as *const T as *const u8, size_of::<T>());
        }
    }

    pub fn push<T>(&mut self, value: T) -> Reallocated {
        let mut realloc = Default::default();
        if self.len + size_of::<T>() > self.layout.size() {
            let new_capacity = self.layout.size() * 2 + (self.layout.size() == 0) as usize;
            self.resize::<T>(new_capacity);
            realloc = Reallocated::Yes;
        }
        unsafe {
            self.buf
                .add(self.len)
                .copy_from_nonoverlapping(&value as *const T as *const u8, size_of::<T>());
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

    pub fn len<T>(&self) -> usize {
        self.len / size_of::<T>()
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

    pub fn iter<T>(&self) -> Iter<'_, T> {
        Iter {
            data: self,
            index: 0,
            phantom: PhantomData::<T> {},
        }
    }

    pub fn slice<T>(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.buf as *const T, self.len::<T>()) }
    }

    pub fn mut_slice<T>(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.buf as *mut T, self.len::<T>()) }
    }
}

impl Drop for UntypedVec {
    fn drop(&mut self) {
        self.dealloc();
    }
}

impl Default for UntypedVec {
    fn default() -> Self {
        UntypedVec::empty()
    }
}

#[derive(Default)]
pub enum Reallocated {
    #[default]
    No,
    Yes,
}

pub struct Iter<'a, T> {
    data: &'a UntypedVec,
    index: usize,
    phantom: PhantomData<T>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len::<T>() {
            let result = Some(self.data.get(self.index));
            self.index += 1;
            result
        } else {
            None
        }
    }
}

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

pub fn into_vec<From, Into>(from: Vec<From>) -> Vec<Into>
where
    From: core::convert::Into<Into>,
{
    let mut vec = Vec::with_capacity(from.len());
    for item in from {
        vec.push(item.into());
    }
    vec
}

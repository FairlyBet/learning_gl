use std::{
    alloc,
    alloc::Layout,
    marker::PhantomData,
    mem::{self, size_of, ManuallyDrop, MaybeUninit},
    ptr, slice,
};

/// Do not store impl Drop types there!!!
pub struct ByteArray {
    buf: *mut u8,
    layout: Layout,
    len: usize,
}

impl ByteArray {
    pub fn init<T>(capacity: usize) -> Self {
        let (buf, layout) = Self::alloc_buf::<T>(capacity * size_of::<T>());
        Self {
            buf,
            layout,
            len: 0,
        }
    }

    fn uninited() -> Self {
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
            let new_capacity = self.layout.size() / size_of::<T>() * 2
                + (self.layout.size() / size_of::<T>() == 0) as usize;
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
        assert!(buf != ptr::null_mut::<u8>(), "Cannot allocate memory");
        (buf, layout)
    }

    fn dealloc(&mut self) {
        unsafe {
            alloc::dealloc(self.buf, self.layout);
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

impl Drop for ByteArray {
    fn drop(&mut self) {
        self.dealloc();
    }
}

impl Default for ByteArray {
    fn default() -> Self {
        ByteArray::uninited()
    }
}

#[derive(Default)]
pub enum Reallocated {
    #[default]
    No,
    Yes,
}

pub struct Iter<'a, T> {
    data: &'a ByteArray,
    index: usize,
    phantom: PhantomData<T>,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.index < self.data.len::<T>()) {
            let result = Some(self.data.get(self.index));
            self.index += 1;
            result
        } else {
            None
        }
    }
}

pub fn into_vec<From, To>(mut vec: Vec<From>) -> Vec<To>
where
    From: Into<To>,
{
    let mut res: Vec<To> = Vec::with_capacity(vec.len());
    for item in vec {
        res.push(item.into());
    }
    res
}

pub struct StaticVec<T, const SIZE: usize> {
    buf: MaybeUninit<[T; SIZE]>,
    len: usize,
}

impl<T, const SIZE: usize> StaticVec<T, SIZE> {
    pub fn new() -> Self {
        Self {
            buf: MaybeUninit::zeroed(),
            len: 0,
        }
    }

    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.len < SIZE {
            unsafe {
                let zeroed = mem::replace(&mut self.buf.assume_init_mut()[self.len], value);
                ManuallyDrop::new(zeroed);
            }
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
        unsafe { &self.buf.assume_init_ref()[index] }
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

impl<T, const SIZE: usize> Drop for StaticVec<T, SIZE> {
    fn drop(&mut self) {
        println!("Dropping {} elements", self.len);
        for i in 0..self.len {
            unsafe {
                let mut zeroed = MaybeUninit::<T>::zeroed();
                let actual = mem::replace(&mut self.buf.assume_init_mut()[i], zeroed.assume_init());
            }
        }
    }
}

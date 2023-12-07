use std::{alloc, alloc::Layout, default, mem::size_of, ptr};

pub struct ByteArray {
    buf: *mut u8,
    layout: Layout,
    len: usize,
}

impl ByteArray {
    pub fn init<T>(n: usize) -> Self {
        let (buf, layout) = Self::alloc_buf::<T>(n * size_of::<T>());
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

    pub fn write<T>(&mut self, value: T) -> Reallocated {
        let mut realloc = Default::default();
        if self.len + size_of::<T>() > self.layout.size() {
            let n = self.layout.size() / size_of::<T>() * 2
                + (self.layout.size() / size_of::<T>() == 0) as usize;
            self.resize::<T>(n);
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

    fn resize<T>(&mut self, n: usize) {
        let (buf, layout) = Self::alloc_buf::<T>(n);
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

    pub fn get_mut<T>(&self, index: usize) -> &mut T {
        assert!(index < self.len::<T>(), "Index out of bound");
        unsafe {
            let value_ptr = self.buf.add(index * size_of::<T>()) as *mut T;
            &mut (*value_ptr)
        }
    }

    pub fn get<T>(&self, index: usize) -> &T {
        self.get_mut(index)
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

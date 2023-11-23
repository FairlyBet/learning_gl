use std::{alloc, alloc::Layout, mem::size_of, ptr};

pub struct ByteArray {
    buf: *mut u8,
    size: usize,
    len: usize,
}

impl ByteArray {
    pub fn init<T>(n: usize) -> Self {
        let (buf, size) = Self::alloc_bytes(n * size_of::<T>());
        Self { buf, size, len: 0 }
    }

    fn uninited() -> Self {
        Self {
            buf: ptr::null_mut::<u8>(),
            size: Default::default(),
            len: Default::default(),
        }
    }

    pub fn write<T>(&mut self, value: T) {
        if self.len + size_of::<T>() > self.size {
            self.resize(self.size + size_of::<T>() * (self.size / size_of::<T>() + 1));
        }
        unsafe {
            self.buf
                .add(self.len)
                .copy_from(&value as *const T as *const u8, size_of::<T>());
        }
        self.len += size_of::<T>();
    }

    fn resize(&mut self, size: usize) {
        let (buf, size) = Self::alloc_bytes(size);
        unsafe {
            buf.copy_from(self.buf, self.len);
        }
        self.dealloc();
        self.buf = buf;
        self.size = size;
    }

    fn alloc_bytes(n_bytes: usize) -> (*mut u8, usize) {
        let layout = Layout::array::<u8>(n_bytes).unwrap();
        let buf = unsafe { alloc::alloc(layout) };
        assert!(buf != ptr::null_mut::<u8>(), "Cannot allocate memory");
        (buf, layout.size())
    }

    fn dealloc(&mut self) {
        let layout = Layout::array::<u8>(self.size).unwrap();
        unsafe {
            alloc::dealloc(self.buf, layout);
        }
    }

    pub fn len<T>(&self) -> usize {
        self.len / size_of::<T>()
    }

    pub fn get<T>(&self, index: usize) -> &T {
        assert!(index < self.len::<T>(), "Index out of bound");
        unsafe {
            let value_ptr = self.buf.add(index * size_of::<T>()) as *const u8 as *const T;
            &(*value_ptr)
        }
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

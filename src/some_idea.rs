use std::marker::PhantomData;
use crate::camera::Camera;

struct Container {

}



trait IContainer {
    // fn data(&self) -> &[T];
}

// impl<T> IContainer<T> for Container<T> {
//     fn data(&self) -> &[T] {
//         todo!()
//     }
// }

// struct Engine<C: IContainer> {
//     container: C,
// }

// impl<Data, C: IContainer> Engine<Data, C> {
//     fn new(container: C) -> Self {
//         Self { container }
//     }
// }

fn foo() {
    // let container = Container::<u32>::new();
    // let engine = Engine::new(container);
}

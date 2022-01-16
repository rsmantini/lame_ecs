# lame_ecs
Minimal ecs library that I did to learn a bit of rust

**features (or lack thereof)**
* static ecs: all supported components must known at compile time
* operations on non-existent/deleted entities panic
* components must be structs

**usage**

Call the macro `create_component_collection_struct` passing the type of the used components:
```rs
#[derive(Debug, PartialEq)]
struct Foo {
    m: i32,
}

#[derive(Debug, PartialEq)]
struct Bar {
    m: String,
}

create_component_collection_struct!(Foo, Bar);
```
Use it as following:
```rs
let mut world = create_world!();
let e0 = world.new_entity();
let e1 = world.new_entity();

let foo0 = Foo{m: 10};
world.add_component::<Foo>(e0, foo0);

let foo1 = Foo{m: 30};
world.add_component::<Foo>(e1, foo1);

let bar = Bar{m: "bar".to_owned()};
world.add_component::<Bar>(e1, bar);

let bar = world.get_component::<Bar>(e1);

world.remove_component::<Foo>(e0);
world.remove_entity(e1);
assert!(!world.is_alive(e1));
```

** component_iter macro **
TODO

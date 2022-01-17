# lame_ecs
Minimal ecs library that I did to learn a bit of rust

**design decisions/limitations**
* static ecs: all supported components must known at compile time
* operations on non-existent/deleted entities panic

**usage**

First generate the `LameEcsComponentCollection` struct by calling the macro `create_component_collection_struct`, 
passing the types of the components to be used as parameters. 
The generated struct needs to be imported to every file this library is used.

```rs
#[derive(Debug, PartialEq)]
struct Foo {
    m: i32,
}

#[derive(Debug, PartialEq)]
struct Bar {
    m: String,
}

lame_ecs::create_component_collection!(Foo, Bar);
```

After the struct `LameEcsComponentCollection` is generated the library can be used as following: 

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

**component iteration**

Iteration on components is acomplished via the `component_iter` and `component_iter_mut` macros.
They receive as parameters the world instance and a list of component types to be iterated on.

```rs
let mut iter = component_iter!(world, Foo, Bar);
```

The iterator will yield a tuple with the requested components and the entity that owns then.
It only yield entities that contains *ALL* of the requested components.
In the example above the yielded tupe will be: `(&Foo, &Bar, &lame_ecs::Entity)` 


Usage example:

```rs
let mut world = create_world!();
let e0 = world.new_entity();
let e1 = world.new_entity();
world.add_component(e0, Foo { m: 42 });
world.add_component(e0, Bar { m: "bar0".to_owned() });
world.add_component(e1, Foo { m: 84 });

let mut iter = component_iter!(world, Foo, Bar);
assert_eq!(
    iter.next().unwrap(),
    (
        &Foo { m: 42 },
        &Bar { m: "bar0".to_owned() },
        &e0
    )
);
assert_eq!(iter.next(), None);

let mut iter = component_iter!(world, Foo);
assert_eq!(iter.next().unwrap(), (&Foo { m: 42 }, &e0));
assert_eq!(iter.next().unwrap(), (&Foo { m: 84 }, &e1));
assert_eq!(iter.next(), None);

let mut iter = component_iter!(world, Bar);
assert_eq!(
    iter.next().unwrap(),
    (
        &Bar { m: "bar0".to_owned()},
        &e0
    )
);
assert_eq!(iter.next(), None);
```

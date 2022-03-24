use std::cell::{RefCell, RefMut};

fn main() {
    let mut world = World::new();
    let entity_id = world.new_entity();
    world.add_component_to_entity(entity_id, Name("Dylan"));
    world.add_component_to_entity(entity_id, Health(10));
    let entity_id_2 = world.new_entity();
    world.add_component_to_entity(entity_id_2, Name("Vicky"));
    world.add_component_to_entity(entity_id_2, Health(10));
    let entity_id_3 = world.new_entity();
    world.add_component_to_entity(entity_id_3, Name("Bruce"));

    let mut health_components = world.borrow_component_vec::<Health>().unwrap();
    let mut name_components = world.borrow_component_vec::<Name>().unwrap();
    let zip = health_components.iter_mut().zip(name_components.iter_mut());
    let iter = zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_mut()?)));
    for (health, name) in iter {
        if name.0 == "Vicky" {
            health.0 = 20;
        }
        println!("Health ({}), Name({})", health.0, name.0);
    }
}

// Our components.
struct Health(i32);
struct Name(&'static str);

// World to store component vectors and entity count.
struct World {
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
}

impl World {
    fn new() -> Self {
        Self {
            entities_count: 0,
            component_vecs: Vec::new(),
        }
    }

    fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }
        self.entities_count += 1;
        entity_id
    }

    // ComponentType must be static to support downcasting Any -> ComponentType
    fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        // Try to find existing component_vec for ComponentType. Insert component if component_vec
        // is found.
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = Some(component);
                return;
            }
        }

        // If component_vec not found, create a new vector & insert the component.
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);
        for _ in 0..self.entities_count {
            new_component_vec.push(None);
        }
        new_component_vec[entity] = Some(component);
        self.component_vecs
            .push(Box::new(RefCell::new(new_component_vec)));
    }

    fn borrow_component_vec<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }
}

// as_any lets us downcast from ComponentVec -> Any -> concrete component type
trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    // Each entity gets an index, all of their components are found at the same index
    // in each component vector. So entity 0's components are found at the 0 index in
    // each component vector. If the entity doesn't have that kind of component, then
    // at that index the vector contains None. Every ComponentVec type must support
    // push_none.
    fn push_none(&mut self);
}

// Casting as Any requires T to be static. Casting as Any supports downcasting
// Any -> concrete component type.
impl<T: 'static> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }
    fn push_none(&mut self) {
        self.get_mut().push(None)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

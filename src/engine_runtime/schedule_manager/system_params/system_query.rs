use std::{
    marker::PhantomData,
    ops::{BitAnd, Deref, DerefMut},
};

use legion::{
    internals::{iter::indexed::TrustedRandomAccessExt, query::view::IntoView},
    query::{DefaultFilter, DynamicFilter, EntityFilter, EntityFilterTuple, Passthrough},
    Query, World,
};

use super::system_param::SystemParam;

pub struct SystemQuery<
    Fetch,
    Filter = EntityFilterTuple<Passthrough, Passthrough>,
    T = EntityFilterTuple<Passthrough, Passthrough>,
> where
    Fetch: IntoView + DefaultFilter,
    Filter: EntityFilter,
    T: BitAnd<Filter> + EntityFilter,
    <T as BitAnd<Filter>>::Output: EntityFilter,
{
    _f: PhantomData<Fetch>,
    _fil: PhantomData<Filter>,
    _t: PhantomData<T>, //_fil: PhantomData<Filter>
}

impl<Fetch, Filter, T> SystemQuery<Fetch, Filter, T>
where
    Fetch: IntoView + DefaultFilter,
    Filter: EntityFilter,
    T: BitAnd<Filter> + EntityFilter,
    <T as BitAnd<Filter>>::Output: EntityFilter,
{
    pub fn query(self) -> Query<Fetch, <T as BitAnd<Filter>>::Output> {
        let query = <Query<Fetch, T>>::new().filter(Filter::default());
        return query;
    }
}

impl<Fetch, Filter, T> From<SystemParam> for SystemQuery<Fetch, Filter, T>
where
    Fetch: IntoView + DefaultFilter,
    Filter: EntityFilter,
    T: BitAnd<Filter> + EntityFilter,
    <T as BitAnd<Filter>>::Output: EntityFilter,
{
    fn from(value: SystemParam) -> Self {
        let mut world = &mut value.engine.ecs.world;

        SystemQuery::<Fetch, Filter, T> {
            _f: PhantomData, /*_fil: PhantomData*/
            _fil: PhantomData,
            _t: PhantomData,
        }
    }
}

trait QueryFilterable {}

pub struct EcsWorld {
    world: &'static mut World,
}
impl From<SystemParam> for EcsWorld {
    fn from(value: SystemParam) -> Self {
        Self {
            world: &mut value.engine.ecs.world,
        }
    }
}

impl Deref for EcsWorld {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        self.world
    }
}

impl DerefMut for EcsWorld {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.world
    }
}

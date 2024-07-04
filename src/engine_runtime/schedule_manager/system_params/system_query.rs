use std::{
    marker::PhantomData,
    ops::{BitAnd, Deref, DerefMut},
};

use legion::{
    internals::query::view::IntoView,
    query::{DefaultFilter, EntityFilter, EntityFilterTuple, Passthrough, View},
    IntoQuery, Query, World,
};

use crate::Position;

use super::system_param::{SystemParalellFilter, SystemParam};

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
    <<Fetch as IntoView>::View as DefaultFilter>::Filter: BitAnd<Filter>,
    <<<Fetch as IntoView>::View as DefaultFilter>::Filter as BitAnd<Filter>>::Output: EntityFilter,
{
    pub fn query(
        self,
    ) -> Query<
        Fetch,
        <<<Fetch as IntoView>::View as DefaultFilter>::Filter as BitAnd<Filter>>::Output,
    > {
        <Fetch>::query().filter(Filter::default())
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
impl<Fetch, Filter, T> SystemParalellFilter for SystemQuery<Fetch, Filter, T>
where
    Fetch: IntoView + DefaultFilter,
    Filter: EntityFilter,
    T: BitAnd<Filter> + EntityFilter,
    <T as BitAnd<Filter>>::Output: EntityFilter,
{
    fn filter_against_param(&self, param: Box<dyn std::any::Any>) -> bool {
        let a = Fetch::View::reads_types();
        todo!()
    }

    fn get_filterable(&self) -> Box<dyn std::any::Any> {
        todo!()
    } 
}

pub struct EcsWorld {
    world: &'static mut World,
}
impl SystemParamFilter for EcsWorld {}
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

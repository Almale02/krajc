use std::{
    marker::PhantomData,
    ops::{BitAnd, Deref, DerefMut},
};

use env_logger::WriteStyle;
use legion::{
    internals::query::view::IntoView,
    query::{DefaultFilter, EntityFilter, EntityFilterTuple, Passthrough, View},
    storage::ComponentTypeId,
    IntoQuery, Query, World,
};

use crate::Position;

use super::system_param::{SystemParalellFilter, SystemParam};

pub struct SystemQueryFilterable {
    pub reads: Vec<ComponentTypeId>,
    pub writes: Vec<ComponentTypeId>,
}

impl SystemQueryFilterable {
    pub fn new(reads: Vec<ComponentTypeId>, writes: Vec<ComponentTypeId>) -> Self {
        Self { reads, writes }
    }
}

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
        Fetch::query().filter(Filter::default())
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
    fn filter_against_param(&self, other: Box<dyn std::any::Any>) -> bool {
        match other.downcast_ref::<SystemQueryFilterable>() {
            Some(x) => {
                let reads = Fetch::View::reads_types_vec();
                let writes = Fetch::View::writes_types_vec();

                let other_reads = &x.reads;
                let other_writes = &x.writes;

                for this_read in reads {
                    for other_write in other_writes {
                        if this_read.type_id() == other_write.type_id() {
                            return false;
                        }
                    }
                }
                for this_write in writes {
                    for other_write in other_writes {
                        if this_write.type_id() == other_write.type_id() {
                            return false;
                        }
                    }
                }
                return true;
            }
            None => true,
        }
    }

    fn get_filterable(&self) -> Box<dyn std::any::Any> {
        Box::new(SystemQueryFilterable::new(
            Fetch::View::reads_types_vec(),
            Fetch::View::writes_types_vec(),
        ))
    }
}

pub struct EcsWorld {
    world: &'static mut World,
}
impl SystemParalellFilter for EcsWorld {
    fn filter_against_param(&self, param: Box<dyn std::any::Any>) -> bool {
        false
    }

    fn get_filterable(&self) -> Box<dyn std::any::Any> {
        Box::new(0)
    }
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

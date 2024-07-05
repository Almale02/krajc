use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{BitAnd, Deref, DerefMut},
};

use legion::{
    internals::query::view::IntoView,
    query::{DefaultFilter, EntityFilter, EntityFilterTuple, Passthrough, View},
    storage::ComponentTypeId,
    IntoQuery, Query, World,
};

use super::system_param::{IntoSystemParalellFilter, SystemParalellFilter, SystemParam};

pub struct SystemQueryFilterable {
    pub reads: Vec<ComponentTypeId>,
    pub writes: Vec<ComponentTypeId>,
}

impl SystemQueryFilterable {
    pub fn new(reads: Vec<ComponentTypeId>, writes: Vec<ComponentTypeId>) -> Self {
        Self { reads, writes }
    }
}

impl SystemParalellFilter for SystemQueryFilterable {
    fn filter_against_param(&self, other: &Box<(dyn SystemParalellFilter + 'static)>) -> bool {
        match other.downcast_ref::<SystemQueryFilterable>() {
            Some(x) => {
                let reads = self.reads.clone().into_iter().collect::<HashSet<_>>();
                let writes = self.writes.clone().into_iter().collect::<HashSet<_>>();

                let other_reads = &x.reads.clone().into_iter().collect::<HashSet<_>>();
                let other_writes = &x.writes.clone().into_iter().collect::<HashSet<_>>();

                reads.is_disjoint(other_writes)
                    && writes.is_disjoint(other_writes)
                    && other_reads.is_disjoint(&writes)
            }
            None => true,
        }
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
impl<Fetch, Filter, T> IntoSystemParalellFilter for SystemQuery<Fetch, Filter, T>
where
    Fetch: IntoView + DefaultFilter,
    Filter: EntityFilter,
    T: BitAnd<Filter> + EntityFilter,
    <T as BitAnd<Filter>>::Output: EntityFilter,
{
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter> {
        Box::new(SystemQueryFilterable::new(
            Fetch::View::reads_types_vec(),
            Fetch::View::writes_types_vec(),
        ))
    }
}

pub struct EcsWorldFilterable {}
impl SystemParalellFilter for EcsWorldFilterable {
    fn filter_against_param(&self, param: &Box<(dyn SystemParalellFilter + 'static)>) -> bool {
        false
    }
}

pub struct EcsWorld {
    world: &'static mut World,
}
impl IntoSystemParalellFilter for EcsWorld {
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter> {
        Box::new(EcsWorldFilterable {})
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

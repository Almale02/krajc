use std::{marker::PhantomData, ops::BitAnd};

use legion::{
    internals::{iter::indexed::TrustedRandomAccessExt, query::view::IntoView},
    query::{DefaultFilter, DynamicFilter, EntityFilter, EntityFilterTuple, Passthrough},
    Query, World,
};

use super::system_param::SystemParam;

pub struct SystemQuery<Fetch, Filter, T = EntityFilterTuple<Passthrough, Passthrough>>
where
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
    pub fn query(filter: Filter) -> Query<Fetch, <T as BitAnd<Filter>>::Output> {
        let query = <Query<Fetch, T>>::new().filter(filter);
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

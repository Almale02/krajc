use std::marker::PhantomData;


use legion::internals::query::view::IntoView;

use super::system_param::SystemParam;

pub struct SystemQuery<Fetch> 
where Fetch: IntoView,
{
    _f: PhantomData<Fetch>,
    //_fil: PhantomData<Filter>
}

impl<Fetch: IntoView, /*Filter*/> From<SystemParam> for SystemQuery<Fetch, /*Filter*/> {
    fn from(value: SystemParam) -> Self {
        let mut world = &mut value.engine.ecs.world;



        SystemQuery::<Fetch, /*Filter*/> { _f: PhantomData, /*_fil: PhantomData*/} 
    }
}

trait QueryFilterable {}


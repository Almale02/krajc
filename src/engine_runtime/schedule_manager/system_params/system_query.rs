use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{BitAnd, Deref, DerefMut},
};

use bevy_ecs::{
    component::ComponentId,
    entity::Entity,
    query::{
        FilteredAccess, QueryBuilder, QueryData, QueryEntityError, QueryFilter, QueryIter,
        QuerySingleError, QueryState, ROQueryItem,
    },
    world::World,
};

use crate::{engine_runtime::EngineRuntime, typed_addr::dupe};

use super::system_param::{IntoSystemParalellFilter, SystemParalellFilter, SystemParam};

pub struct SystemQueryFilterable {
    pub access: FilteredAccess<ComponentId>,
}

impl SystemQueryFilterable {
    pub fn new(access: FilteredAccess<ComponentId>) -> Self {
        Self { access }
    }
}

impl SystemParalellFilter for SystemQueryFilterable {
    fn filter_against_param(&self, other: &Box<(dyn SystemParalellFilter + 'static)>) -> bool {
        return match other.downcast_ref::<SystemQueryFilterable>() {
            Some(x) => {
                self.access.is_compatible(&x.access)
                /*let reads = self.reads.clone().into_iter().collect::<HashSet<_>>();
                let writes = self.writes.clone().into_iter().collect::<HashSet<_>>();

                let other_reads = &x.reads.clone().into_iter().collect::<HashSet<_>>();
                let other_writes = &x.writes.clone().into_iter().collect::<HashSet<_>>();

                reads.is_disjoint(other_writes)
                    && writes.is_disjoint(other_writes)
                    && other_reads.is_disjoint(&writes)*/
            }
            None => true,
        };
    }
}

pub struct SystemQuery<Data, Filter = ()>
where
    Data: QueryData,
    Filter: QueryFilter,
{
    _d: PhantomData<Data>,
    _f: PhantomData<Filter>,
    provider: QueryState<Data, Filter>,
    pub world: &'static mut World,
}

impl<Data: QueryData, Filter: QueryFilter> SystemQuery<Data, Filter> {
    #[inline]
    pub fn iter<'w, 's>(&'s mut self) -> QueryIter<'w, 's, Data::ReadOnly, Filter> {
        self.provider.iter(dupe(self.world))
    }

    /// Returns an [`Iterator`] over the query results for the given [`World`].
    ///
    /// This iterator is always guaranteed to return results from each matching entity once and only once.
    /// Iteration order is not guaranteed.
    #[inline]
    pub fn iter_mut<'w, 's>(&'s mut self) -> QueryIter<'w, 's, Data, Filter> {
        self.provider.iter_mut(dupe(self.world))
    }
    #[inline]
    pub fn get<'w>(&mut self, entity: Entity) -> Result<ROQueryItem<'w, Data>, QueryEntityError> {
        self.provider.get(dupe(self.world), entity)
    }
    #[inline]
    pub fn get_mut<'w>(&mut self, entity: Entity) -> Result<Data::Item<'w>, QueryEntityError> {
        self.provider.get_mut(dupe(self.world), entity)
    }

    #[track_caller]
    #[inline]
    pub fn single<'w>(&mut self) -> ROQueryItem<'w, Data> {
        self.provider.single(dupe(self.world))
    }

    /// Returns a single immutable query result when there is exactly one entity matching
    /// the query.
    ///
    /// This can only be called for read-only queries,
    /// see [`get_single_mut`](Self::get_single_mut) for write-queries.
    ///
    /// If the number of query results is not exactly one, a [`QuerySingleError`] is returned
    /// instead.
    #[inline]
    pub fn get_single<'w>(&mut self) -> Result<ROQueryItem<'w, Data>, QuerySingleError> {
        self.provider.get_single(dupe(self.world))
    }

    /// Returns a single mutable query result when there is exactly one entity matching
    /// the query.
    ///
    /// # Panics
    ///
    /// Panics if the number of query results is not exactly one. Use
    /// [`get_single_mut`](Self::get_single_mut) to return a `Result` instead of panicking.
    #[track_caller]
    #[inline]
    pub fn single_mut<'w>(&mut self) -> Data::Item<'w> {
        self.provider.single_mut(dupe(self.world))
    }

    /// Returns a single mutable query result when there is exactly one entity matching
    /// the query.
    ///
    /// If the number of query results is not exactly one, a [`QuerySingleError`] is returned
    /// instead.
    #[inline]
    pub fn get_single_mut<'w>(&mut self) -> Result<Data::Item<'w>, QuerySingleError> {
        self.provider.get_single_mut(dupe(self.world))
    }
}

impl<Data, Filter> From<SystemParam> for SystemQuery<Data, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
{
    fn from(value: SystemParam) -> Self {
        let world = &mut value.engine.ecs.world;
        let provider = QueryBuilder::<Data, Filter>::new(world).build();

        SystemQuery {
            _d: PhantomData,
            _f: PhantomData,
            world,
            provider,
        }
    }
}
impl<Data, Filter> IntoSystemParalellFilter for SystemQuery<Data, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
{
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter> {
        let access = self.provider.component_access();
        Box::new(SystemQueryFilterable::new(access.clone()))
    }
}

pub struct EcsWorldFilterable {}
impl SystemParalellFilter for EcsWorldFilterable {
    fn filter_against_param(&self, param: &Box<(dyn SystemParalellFilter + 'static)>) -> bool {
        false
    }
}

pub struct Runtime {
    runtime: &'static mut EngineRuntime,
}
impl Deref for Runtime {
    type Target = EngineRuntime;
    fn deref(&self) -> &Self::Target {
        self.runtime
    }
}

impl DerefMut for Runtime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.runtime
    }
}
impl From<SystemParam> for Runtime {
    fn from(value: SystemParam) -> Self {
        Self {
            runtime: value.engine,
        }
    }
}
pub struct RuntimeFilterable;

impl IntoSystemParalellFilter for Runtime {
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter> {
        Box::new(RuntimeFilterable)
    }
}
impl SystemParalellFilter for RuntimeFilterable {
    fn filter_against_param(&self, _param: &Box<dyn SystemParalellFilter>) -> bool {
        false
    }
}//
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


use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy_ecs::{
    component::ComponentId,
    entity::Entity,
    query::{
        FilteredAccess, QueryData, QueryEntityError, QueryFilter, QueryIter, QuerySingleError,
        QueryState, ROQueryItem,
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
    pub fn iter(&mut self) -> QueryIter<'_, '_, Data::ReadOnly, Filter> {
        self.provider.iter(dupe(self.world))
    }

    /// Returns an [`Iterator`] over the query results for the given [`World`].
    ///
    /// This iterator is always guaranteed to return results from each matching entity once and only once.
    /// Iteration order is not guaranteed.
    #[inline]
    pub fn iter_mut(&mut self) -> QueryIter<'_, '_, Data, Filter> {
        self.provider.iter_mut(self.world)
    }

    /// Gets the query result for the given [`World`] and [`Entity`].
    ///
    /// This can only be called for read-only queries, see [`Self::get_mut`] for write-queries.
    ///
    /// This is always guaranteed to run in `O(1)` time.
    #[inline]
    pub fn get(&mut self, entity: Entity) -> Result<ROQueryItem<'_, Data>, QueryEntityError> {
        self.provider.get(self.world, entity)
    }
    /// Gets the query result for the given [`World`] and [`Entity`].
    ///
    /// This is always guaranteed to run in `O(1)` time.
    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Result<Data::Item<'_>, QueryEntityError> {
        self.provider.get_mut(self.world, entity)
    }

    /// Returns a single immutable query result when there is exactly one entity matching
    /// the query.
    ///
    /// This can only be called for read-only queries,
    /// see [`single_mut`](Self::single_mut) for write-queries.
    ///
    /// # Panics
    ///
    /// Panics if the number of query results is not exactly one. Use
    /// [`get_single`](Self::get_single) to return a `Result` instead of panicking.
    #[track_caller]
    #[inline]
    pub fn single(&mut self) -> ROQueryItem<'_, Data> {
        self.provider.single(self.world)
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
    pub fn get_single(&mut self) -> Result<ROQueryItem<'_, Data>, QuerySingleError> {
        self.provider.get_single(self.world)
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
    pub fn single_mut(&mut self) -> Data::Item<'_> {
        self.provider.single_mut(self.world)
    }

    /// Returns a single mutable query result when there is exactly one entity matching
    /// the query.
    ///
    /// If the number of query results is not exactly one, a [`QuerySingleError`] is returned
    /// instead.
    #[inline]
    pub fn get_single_mut(&mut self) -> Result<Data::Item<'_>, QuerySingleError> {
        self.provider.get_single_mut(self.world)
    }

    /// Returns the read-only query results for the given array of [`Entity`].
    ///
    /// In case of a nonexisting entity or mismatched component, a [`QueryEntityError`] is
    /// returned instead.
    ///
    /// Note that the unlike [`QueryState::get_many_mut`], the entities passed in do not need to be unique.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_ecs::prelude::*;
    /// use bevy_ecs::query::QueryEntityError;
    ///
    /// #[derive(Component, PartialEq, Debug)]
    /// struct A(usize);
    ///
    /// let mut world = World::new();
    /// let entity_vec: Vec<Entity> = (0..3).map(|i|world.spawn(A(i)).id()).collect();
    /// let entities: [Entity; 3] = entity_vec.try_into().unwrap();
    ///
    /// world.spawn(A(73));
    ///
    /// let mut query_state = world.query::<&A>();
    ///
    /// let component_values = query_state.get_many(&world, entities).unwrap();
    ///
    /// assert_eq!(component_values, [&A(0), &A(1), &A(2)]);
    ///
    /// let wrong_entity = Entity::from_raw(365);
    ///
    /// assert_eq!(query_state.get_many(&world, [wrong_entity]), Err(QueryEntityError::NoSuchEntity(wrong_entity)));
    /// ```
    #[inline]
    pub fn get_many<const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> Result<[ROQueryItem<'_, Data>; N], QueryEntityError> {
        self.provider.get_many(self.world, entities)
    }
}

impl<Data, Filter> From<SystemParam> for SystemQuery<Data, Filter>
where
    Data: QueryData,
    Filter: QueryFilter,
{
    fn from(value: SystemParam) -> Self {
        let world = &mut value.engine.ecs.world;
        let provider = world.query_filtered::<Data, Filter>();

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
    fn filter_against_param(&self, _param: &Box<(dyn SystemParalellFilter + 'static)>) -> bool {
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
} //
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

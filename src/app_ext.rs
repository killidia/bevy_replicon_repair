//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::ecs::component::Tick;
use bevy::prelude::*;
use bevy_replicon::prelude::{AppReplicationExt, MapNetworkEntities};
use bevy_replicon::replicon_core::replication_rules::{
    SerializeFn, DeserializeFn, RemoveComponentFn, serialize_component, deserialize_component, remove_component,
    deserialize_mapped_component,
};
use serde::{de::DeserializeOwned, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Default component repair for [`AppReplicationRepairExt`].
///
/// The component `C` will be removed from `entity` if the component was not added/changed on the entity in the repair
/// tick.
///
/// If you manually added/changed the component on the entity in the repair tick, it may be erroneously left in place.
/// Likewise, if you are not replicating the component and instead manually inserted it, it may be erroneously removed.
///
/// You can disable this function for an entity by adding an [`Ignore<C>`](crate::Ignore) component to it on the client.
pub fn repair_component<C: Component>(entity: &mut EntityWorldMut, preinit_tick: Tick)
{
    let world_tick = unsafe { entity.world_mut().change_tick() };

    // check if the component is ignored from replication
    if entity.contains::<Ignore<C>>() { return; };

    // check if the component exists on the entity
    let Some(change_ticks) = entity.get_change_ticks::<C>() else { return; };

    // check if the component was mutated by the most recent replication message
    if change_ticks.is_changed(preinit_tick, world_tick) { return; }

    entity.remove::<C>();
}

//-------------------------------------------------------------------------------------------------------------------

pub trait AppReplicationRepairExt
{
    /// Mirrors [`AppReplicationExt::replicate`](bevy_replicon::prelude::AppReplicationExt::replicate) using the default
    /// component-removal repair function [`repair_component`].
    fn replicate_repair<C>(&mut self) -> &mut Self
    where
        C: Component + Serialize + DeserializeOwned;

    /// Mirrors [`AppReplicationExt::replicate_mapped`](bevy_replicon::prelude::AppReplicationExt::replicate_mapped) using
    /// the default component-removal repair function [`repair_component`].
    fn replicate_repair_mapped<C>(&mut self) -> &mut Self
    where
        C: Component + Serialize + DeserializeOwned + MapNetworkEntities;

    /// Mirrors [`AppReplicationExt::replicate_with`](bevy_replicon::prelude::AppReplicationExt::replicate_with) with
    /// a user-defined component-removal repair function.
    fn replicate_repair_with<C>(
        &mut self,
        serialize: SerializeFn,
        deserialize: DeserializeFn,
        remove: RemoveComponentFn,
        repair: RepairComponentFn,
    ) -> &mut Self
    where
        C: Component;
}

impl AppReplicationRepairExt for App {
    fn replicate_repair<C>(&mut self) -> &mut Self
    where
        C: Component + Serialize + DeserializeOwned,
    {
        self.replicate_repair_with::<C>(
                serialize_component::<C>,
                deserialize_component::<C>,
                remove_component::<C>,
                repair_component::<C>,
            )
    }

    fn replicate_repair_mapped<C>(&mut self) -> &mut Self
    where
        C: Component + Serialize + DeserializeOwned + MapNetworkEntities,
    {
        self.replicate_repair_with::<C>(
                serialize_component::<C>,
                deserialize_mapped_component::<C>,
                remove_component::<C>,
                repair_component::<C>,
            )
    }

    fn replicate_repair_with<C>(
        &mut self,
        serialize: SerializeFn,
        deserialize: DeserializeFn,
        remove: RemoveComponentFn,
        repair: RepairComponentFn,
    ) -> &mut Self
    where
        C: Component,
    {
        if !self.world.contains_resource::<ComponentRepairRules>()
        { self.world.init_resource::<ComponentRepairRules>(); }

        self.replicate_with::<C>(serialize, deserialize, remove);
        self.world.resource_mut::<ComponentRepairRules>().push(repair);

        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::collections::HashMap;

use syn_core::WorldState;
use syn_core::NpcId;
use syn_core::npc::NpcPrototype;
use crate::{NpcInstance, NpcLod, instantiate_simulated_npc_from_prototype};

#[derive(Debug, Default)]
pub struct NpcRegistry {
    /// All live NPC instances keyed by NpcId.
    pub instances: HashMap<NpcId, NpcInstance>,
}

impl NpcRegistry {
    pub fn get(&self, id: NpcId) -> Option<&NpcInstance> {
        self.instances.get(&id)
    }

    pub fn get_mut(&mut self, id: NpcId) -> Option<&mut NpcInstance> {
        self.instances.get_mut(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NpcId, &NpcInstance)> {
        self.instances.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&NpcId, &mut NpcInstance)> {
        self.instances.iter_mut()
    }

    /// Ensure an NPC is instantiated at the requested LOD.
    /// If already instantiated, upgrade LOD if needed.
    pub fn ensure_npc_instance(
        &mut self,
        world: &WorldState,
        id: NpcId,
        requested_lod: NpcLod,
        tick: u64,
    ) {
        if let Some(instance) = self.instances.get_mut(&id) {
            // Upgrade LOD if necessary
            if (requested_lod as u8) > (instance.lod as u8) {
                instance.lod = requested_lod;
            }
            return;
        }

        // Instantiate from prototype
        if let Some(proto) = world.npc_prototype(id) {
            let sim = instantiate_simulated_npc_from_prototype(proto, world, tick);
            let instance = NpcInstance {
                id,
                lod: requested_lod,
                sim,
                last_tick: tick,
                behavior: None,
                busy_until_tick: 0,
                last_action: None,
                current_activity: syn_core::npc::NpcActivityKind::Home,
            };
            self.instances.insert(id, instance);
        } else {
            // No prototype; nothing to do.
        }
    }

    pub fn focus_npc_for_scene(
        &mut self,
        world: &WorldState,
        id: NpcId,
        tick: u64,
    ) {
        self.ensure_npc_instance(world, id, NpcLod::Tier2Active, tick);
    }

    pub fn background_npc(&mut self, id: NpcId) {
        if let Some(inst) = self.instances.get_mut(&id) {
            inst.lod = NpcLod::Tier1Neighborhood;
        }
    }
}

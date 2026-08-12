use super::*;

impl GameWorld {
    pub(super) fn tick_gather(
        &mut self,
        unit_index: usize,
        resource_id: String,
        phase: GatherPhase,
        dt: f64,
    ) {
        match phase {
            GatherPhase::ToResource => self.tick_to_resource(unit_index, resource_id, dt),
            GatherPhase::Gathering => self.tick_at_resource(unit_index, resource_id, dt),
            GatherPhase::Returning => self.tick_returning(unit_index, resource_id, dt),
            GatherPhase::Depositing => self.tick_depositing(unit_index, resource_id),
        }
    }

    fn tick_to_resource(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        let Some(resource_index) = self
            .resources
            .iter()
            .position(|resource| resource.id == resource_id)
        else {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        };
        if self.resources[resource_index].amount <= 0.0 {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        }
        if self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.amount + f64::EPSILON >= VILLAGER_CARRY_CAPACITY)
        {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }

        let target = self.resources[resource_index].position;
        let (arrived, remaining) = self.tick_toward_interaction(unit_index, target, dt);
        if !arrived {
            return;
        }
        self.set_gather_phase(unit_index, resource_id.clone(), GatherPhase::Gathering);
        if remaining > 0.0 {
            self.tick_at_resource(unit_index, resource_id, remaining);
        }
    }

    fn tick_at_resource(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        let Some(resource_index) = self
            .resources
            .iter()
            .position(|resource| resource.id == resource_id)
        else {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        };
        if self.resources[resource_index].amount <= f64::EPSILON {
            self.resources[resource_index].amount = 0.0;
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        }

        let kind = self.resources[resource_index].kind;
        if self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.kind != kind)
        {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }
        let carried = self.units[unit_index]
            .cargo
            .as_ref()
            .map_or(0.0, |cargo| cargo.amount);
        let capacity_left = (VILLAGER_CARRY_CAPACITY - carried).max(0.0);
        let gathered = (GATHER_RATE * self.gather_multiplier(kind) * dt)
            .min(self.resources[resource_index].amount)
            .min(capacity_left);
        self.resources[resource_index].amount -= gathered;
        if gathered > 0.0 {
            let cargo = self.units[unit_index]
                .cargo
                .get_or_insert(CarriedResource { kind, amount: 0.0 });
            cargo.amount += gathered;
        }

        if self.resources[resource_index].amount <= f64::EPSILON {
            self.resources[resource_index].amount = 0.0;
        }
        let full = self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.amount + f64::EPSILON >= VILLAGER_CARRY_CAPACITY);
        if full || self.resources[resource_index].amount == 0.0 {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
        }
    }

    fn tick_returning(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        if self.units[unit_index].cargo.is_none() {
            self.resume_or_finish_gather(unit_index, resource_id);
            return;
        }
        let Some(target) = self.nearest_reachable_town_center(unit_index) else {
            return;
        };
        if self.tick_toward_interaction(unit_index, target, dt).0 {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Depositing);
        }
    }

    fn tick_depositing(&mut self, unit_index: usize, resource_id: String) {
        let Some(target) = self.nearest_reachable_town_center(unit_index) else {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        };
        if !self.adjacent_to(unit_index, target) {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }
        if let Some(cargo) = self.units[unit_index].cargo.take() {
            self.stockpile.add(cargo.kind, cargo.amount);
        }
        self.resume_or_finish_gather(unit_index, resource_id);
    }

    fn finish_or_return_with_cargo(&mut self, unit_index: usize, resource_id: String) {
        if self.units[unit_index].cargo.is_some() {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
        } else {
            self.units[unit_index].action = UnitAction::Idle;
        }
    }

    fn resume_or_finish_gather(&mut self, unit_index: usize, resource_id: String) {
        let resource_remains = self
            .resources
            .iter()
            .any(|resource| resource.id == resource_id && resource.amount > f64::EPSILON);
        if resource_remains {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::ToResource);
        } else {
            self.units[unit_index].action = UnitAction::Idle;
        }
    }

    fn set_gather_phase(&mut self, unit_index: usize, resource_id: String, phase: GatherPhase) {
        self.units[unit_index].action = UnitAction::Gather { resource_id, phase };
    }
}

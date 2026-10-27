//! Shared selection behavior for hierarchy, viewport and batch commands.
use crate::editor::Editor;
use eframe::egui;
use kvas_engine::prelude::*;

impl Editor {
    pub fn selected_entities(&self) -> Vec<Entity> {
        let Some(primary) = self.selection.filter(|&e| self.active().world.is_alive(e)) else { return Vec::new() };
        if !self.multi_selection.contains(&primary) { return vec![primary]; }
        self.multi_selection.iter().copied().filter(|&e| self.active().world.is_alive(e)).collect()
    }

    pub fn select_entity(&mut self, entity: Option<Entity>, modifiers: egui::Modifiers, order: &[Entity]) {
        let Some(entity) = entity else {
            if !modifiers.command && !modifiers.shift { self.selection = None; self.multi_selection.clear(); }
            return;
        };
        let mut selected = self.selected_entities();
        if modifiers.shift {
            if let Some((start, end)) = self.selection.and_then(|e| order.iter().position(|&x| x == e)).zip(order.iter().position(|&x| x == entity)) {
                for &e in &order[start.min(end)..=start.max(end)] { if !selected.contains(&e) { selected.push(e); } }
            } else if !selected.contains(&entity) { selected.push(entity); }
        } else if modifiers.command || modifiers.ctrl {
            if selected.contains(&entity) { selected.retain(|&e| e != entity); } else { selected.push(entity); }
        } else { selected = vec![entity]; }
        self.selection = if selected.contains(&entity) { Some(entity) } else { selected.last().copied() };
        self.multi_selection = selected;
    }

    pub fn selection_roots(&self) -> Vec<Entity> {
        let selected = self.selected_entities();
        selected.iter().copied().filter(|&e| !selected.iter().any(|&p| hierarchy::is_ancestor(&self.active().world, p, e))).collect()
    }

    pub fn delete_selection(&mut self) {
        for e in self.selection_roots() { self.delete_entity(e); }
        self.selection = None;
        self.multi_selection.clear();
    }

    pub fn duplicate_selection(&mut self) {
        let mut copies = Vec::new();
        for e in self.selection_roots() {
            self.duplicate_entity(e);
            if let Some(copy) = self.selection { copies.push(copy); }
        }
        self.selection = copies.last().copied();
        self.multi_selection = copies;
    }
}

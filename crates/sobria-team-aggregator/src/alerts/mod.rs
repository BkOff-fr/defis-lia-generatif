//! Alertes seuils (C29.4 — brief §C29.4).
//!
//! Permet à l'admin de définir des règles « si la conso de gCO₂eq d'un
//! utilisateur (ou de toute l'équipe) sur une période (daily / weekly /
//! monthly) dépasse `gco2eq_max`, notifier via webhook, email ou simple
//! log_only ».
//!
//! Wiring : appelé depuis `server::api::estimations::handle` après chaque
//! insertion réussie d'estimation. La logique est inline (timeout 5s sur
//! webhook) pour rester simple ; en cas de charge plus élevée on passera
//! à un job de fond.
//!
//! Voir ADR-0013 Phase 2 et `briefs/chantiers/C29-v0.7.1-polish-mode-equipe.md`.

pub mod checker;
pub mod notify;
pub mod periods;
pub mod store;

pub use checker::{check_thresholds_for_user, AlertEvent};
pub use periods::{period_bounds, AlertPeriod};
pub use store::{
    delete_threshold, insert_threshold, list_active_thresholds, list_thresholds_admin,
    list_triggers_admin, AlertScope, NewThreshold, NotifyKind, Threshold, TriggerRow,
};

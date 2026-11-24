// SPDX-License-Identifier: MIT
//! Core domain types for Peeper: the `Target` oracle (L2) and its cost model.

mod cost;
mod target;

pub use cost::CostModel;
pub use target::{Target, TargetKind};

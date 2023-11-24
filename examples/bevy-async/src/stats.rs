use bevy::prelude::*;

/// Attack Stat.
#[derive(Debug, Clone, Component)]
pub struct AttackStat(pub u32);

/// Defense Stat.
#[derive(Debug, Clone, Component)]
pub struct DefenseStat(pub u32);

/// Hit Points.
#[derive(Debug, Clone, Component)]
pub struct HitPoints(pub u32);

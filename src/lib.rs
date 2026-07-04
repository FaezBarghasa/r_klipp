#![no_std]

include!(concat!(env!("OUT_DIR"), "/config.rs"));

pub mod hal;
pub mod ipc;
pub mod tasks;
pub mod parser;
pub mod control;
pub mod kinematics;
pub mod motors;
pub mod actors;
pub mod math;

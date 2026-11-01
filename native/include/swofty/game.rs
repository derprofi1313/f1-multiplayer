#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    unused_imports
)]
use core::ffi::{c_char, c_int, c_uint, c_void};

pub const BUILD_SHA256: &str = "d873a1c8d2044cef2fd76c99c9d5a97649c5b815d055540995105f46619865ae";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Stable,
    Experimental,
}

#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub id: &'static str,
    pub kind: &'static str,
    pub status: Status,
}

#[repr(C)]
pub struct AiHandlingConfig {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarAssists {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarComponentBase {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarConvexHull {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarCoreAero {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarDamageBodyPart {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarDebugSystem {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarErs {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarErsMode {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarF1Plank {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarPerfVehicle {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarPhysicsRecord {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarSetupValues {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarSpline {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarSuspBrakeType {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarSuspCorner {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarSuspTyreType {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarSuspension {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarTuning {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarTyreCompound {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarTyreMfCurve {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarTyreState {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarVehicleManager {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarVtfAsset {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarWheel {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CarWheelProxies {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct DepartmentData {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct FEHomeViewModel {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct GdbTableCursor {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct GdbValueSlot {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct InputRecord {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct NeStr {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct RedFlagSystem {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct RnDDepartmentConfig {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct TodCurveBinding {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct UiLayoutStep {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct UiSignal {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct UiTemplateRecord {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct UpgradeEffectDef {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct UpgradeRecord {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct VehicleIdentity {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct VehiclePackageRow {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct Action {
    pub r#type: i32,
    _pad_0x4: [u8; 0x4],
    pub str: *mut NeStr,
}
const _: () = assert!(core::mem::offset_of!(Action, r#type) == 0x0);
const _: () = assert!(core::mem::offset_of!(Action, str) == 0x8);

#[repr(C)]
pub struct ActivityDef {
    _pad_0x0: [u8; 0x10],
    pub r#type: u8,
}
const _: () = assert!(core::mem::offset_of!(ActivityDef, r#type) == 0x10);

#[repr(C)]
pub struct AiController {
    _pad_0x0: [u8; 0x8],
    pub input: *mut InputRecord,
    pub owner: *mut c_void,
    pub vehicle: u8,
}
const _: () = assert!(core::mem::offset_of!(AiController, input) == 0x8);
const _: () = assert!(core::mem::offset_of!(AiController, owner) == 0x10);
const _: () = assert!(core::mem::offset_of!(AiController, vehicle) == 0x18);

#[repr(C)]
pub struct BankAccount {
    _pad_0x0: [u8; 0x10],
    pub subs_begin: *mut BankSubAccount,
    pub subs_end: *mut BankSubAccount,
    _pad_0x20: [u8; 0x8],
    pub key: u32,
    _pad_0x2c: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(BankAccount, subs_begin) == 0x10);
const _: () = assert!(core::mem::offset_of!(BankAccount, subs_end) == 0x18);
const _: () = assert!(core::mem::offset_of!(BankAccount, key) == 0x28);
const _: () = assert!(core::mem::size_of::<BankAccount>() == 0x30);

#[repr(C)]
pub struct BankSubAccount {
    _pad_0x0: [u8; 0xc],
    pub category: u8,
    _pad_0xd: [u8; 0x3],
    pub balance: i32,
    pub minimum: i32,
    _pad_0x18: [u8; 0x28],
    pub history_begin: *mut BankTransaction,
    pub history_end: *mut BankTransaction,
    _pad_0x50: [u8; 0x160],
}
const _: () = assert!(core::mem::offset_of!(BankSubAccount, category) == 0xc);
const _: () = assert!(core::mem::offset_of!(BankSubAccount, balance) == 0x10);
const _: () = assert!(core::mem::offset_of!(BankSubAccount, minimum) == 0x14);
const _: () = assert!(core::mem::offset_of!(BankSubAccount, history_begin) == 0x40);
const _: () = assert!(core::mem::offset_of!(BankSubAccount, history_end) == 0x48);
const _: () = assert!(core::mem::size_of::<BankSubAccount>() == 0x1b0);

#[repr(C)]
pub struct BankTransaction {
    pub r#type: u8,
    _pad_0x1: [u8; 0x3],
    pub amount: i32,
    _pad_0x8: [u8; 0x4],
    pub time: u32,
    pub flow: u8,
    _pad_0x11: [u8; 0x3],
}
const _: () = assert!(core::mem::offset_of!(BankTransaction, r#type) == 0x0);
const _: () = assert!(core::mem::offset_of!(BankTransaction, amount) == 0x4);
const _: () = assert!(core::mem::offset_of!(BankTransaction, time) == 0xc);
const _: () = assert!(core::mem::offset_of!(BankTransaction, flow) == 0x10);
const _: () = assert!(core::mem::size_of::<BankTransaction>() == 0x14);

#[repr(C)]
pub struct BankTransactionMessage {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x8],
    pub account: u16,
    _pad_0x12: [u8; 0x2],
    pub r#type: u8,
    _pad_0x15: [u8; 0x3],
    pub amount: i32,
    pub created: u32,
    pub time: u32,
    pub flow: u8,
    pub check_funds: u8,
    _pad_0x26: [u8; 0x2],
    pub category: u8,
}
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, account) == 0x10);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, r#type) == 0x14);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, amount) == 0x18);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, created) == 0x1c);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, time) == 0x20);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, flow) == 0x24);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, check_funds) == 0x25);
const _: () = assert!(core::mem::offset_of!(BankTransactionMessage, category) == 0x28);

#[repr(C)]
pub struct CalendarActivity {
    _pad_0x0: [u8; 0xa],
    pub day: u16,
    _pad_0xc: [u8; 0x4],
    pub activity: *mut ActivityDef,
    _pad_0x18: [u8; 0x8],
}
const _: () = assert!(core::mem::offset_of!(CalendarActivity, day) == 0xa);
const _: () = assert!(core::mem::offset_of!(CalendarActivity, activity) == 0x10);
const _: () = assert!(core::mem::size_of::<CalendarActivity>() == 0x20);

#[repr(C)]
pub struct CarBrakeEcu {
    _pad_0x0: [u8; 0x21d8],
    pub brake_bias: f32,
    pub brake_pressure: f32,
}
const _: () = assert!(core::mem::offset_of!(CarBrakeEcu, brake_bias) == 0x21d8);
const _: () = assert!(core::mem::offset_of!(CarBrakeEcu, brake_pressure) == 0x21dc);

#[repr(C)]
pub struct CarChassis {
    _pad_0x0: [u8; 0x118],
    pub tyre_physics_group: [u8; 40],
    _pad_0x140: [u8; 0x10],
    pub mass: f32,
}
const _: () = assert!(core::mem::offset_of!(CarChassis, tyre_physics_group) == 0x118);
const _: () = assert!(core::mem::offset_of!(CarChassis, mass) == 0x150);

#[repr(C)]
pub struct CarDamageFrontWing {
    _pad_0x0: [u8; 0x28],
    pub speed_threshold: f32,
    pub damage_scale: f32,
    _pad_0x30: [u8; 0x4],
    pub side_detach_scale: f32,
    pub full_detach_scale: f32,
    _pad_0x3c: [u8; 0x6c],
    pub parts_allocator: *mut c_void,
    pub parts_begin: *mut *mut c_void,
    pub parts_end: *mut *mut c_void,
    pub parts_capacity: u64,
}
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, speed_threshold) == 0x28);
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, damage_scale) == 0x2c);
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, side_detach_scale) == 0x34);
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, full_detach_scale) == 0x38);
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, parts_allocator) == 0xa8);
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, parts_begin) == 0xb0);
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, parts_end) == 0xb8);
const _: () = assert!(core::mem::offset_of!(CarDamageFrontWing, parts_capacity) == 0xc0);

#[repr(C)]
pub struct CarDamageSuspension {
    _pad_0x0: [u8; 0x28],
    pub speed_threshold_obf: u32,
    _pad_0x2c: [u8; 0xc],
    pub front_damage_scale_obf: u32,
    _pad_0x3c: [u8; 0xc],
    pub rear_damage_scale_obf: u32,
    _pad_0x4c: [u8; 0xc],
    pub wheel_detach_scale_obf: u32,
    _pad_0x5c: [u8; 0xc],
    pub parts_detach_scale_obf: u32,
    _pad_0x6c: [u8; 0xc],
    pub linear_fracture_limit: f32,
    pub angular_fracture_limit: f32,
}
const _: () = assert!(core::mem::offset_of!(CarDamageSuspension, speed_threshold_obf) == 0x28);
const _: () = assert!(core::mem::offset_of!(CarDamageSuspension, front_damage_scale_obf) == 0x38);
const _: () = assert!(core::mem::offset_of!(CarDamageSuspension, rear_damage_scale_obf) == 0x48);
const _: () = assert!(core::mem::offset_of!(CarDamageSuspension, wheel_detach_scale_obf) == 0x58);
const _: () = assert!(core::mem::offset_of!(CarDamageSuspension, parts_detach_scale_obf) == 0x68);
const _: () = assert!(core::mem::offset_of!(CarDamageSuspension, linear_fracture_limit) == 0x78);
const _: () = assert!(core::mem::offset_of!(CarDamageSuspension, angular_fracture_limit) == 0x7c);

#[repr(C)]
pub struct CarDifferential {
    _pad_0x0: [u8; 0x8],
    pub car: *mut CarLive,
    _pad_0x10: [u8; 0xdc],
    pub diff_off_throttle: f32,
    pub diff_on_throttle: f32,
}
const _: () = assert!(core::mem::offset_of!(CarDifferential, car) == 0x8);
const _: () = assert!(core::mem::offset_of!(CarDifferential, diff_off_throttle) == 0xec);
const _: () = assert!(core::mem::offset_of!(CarDifferential, diff_on_throttle) == 0xf0);

#[repr(C)]
pub struct CarObfFloat {
    pub a: u32,
    _pad_0x4: [u8; 0x4],
    pub b: u32,
}
const _: () = assert!(core::mem::offset_of!(CarObfFloat, a) == 0x0);
const _: () = assert!(core::mem::offset_of!(CarObfFloat, b) == 0x8);

#[repr(C)]
pub struct CarEngine {
    _pad_0x0: [u8; 0x8],
    pub car: *mut CarLive,
    _pad_0x10: [u8; 0x8a0],
    pub vtf_limiter_rotation_rate: CarObfFloat,
    _pad_0x8bc: [u8; 0x1c5c],
    pub max_torque: CarObfFloat,
    _pad_0x2524: [u8; 0x4],
    pub limiter_rotation_rate: f32,
    _pad_0x252c: [u8; 0xc],
    pub rotation_rate: f32,
    _pad_0x253c: [u8; 0x38],
    pub throttle: f32,
}
const _: () = assert!(core::mem::offset_of!(CarEngine, car) == 0x8);
const _: () = assert!(core::mem::offset_of!(CarEngine, vtf_limiter_rotation_rate) == 0x8b0);
const _: () = assert!(core::mem::offset_of!(CarEngine, max_torque) == 0x2518);
const _: () = assert!(core::mem::offset_of!(CarEngine, limiter_rotation_rate) == 0x2528);
const _: () = assert!(core::mem::offset_of!(CarEngine, rotation_rate) == 0x2538);
const _: () = assert!(core::mem::offset_of!(CarEngine, throttle) == 0x2574);

#[repr(C)]
pub struct CarF1Aero {
    _pad_0x0: [u8; 0x19a0],
    pub drag_scale: f32,
    pub lift_scale: f32,
}
const _: () = assert!(core::mem::offset_of!(CarF1Aero, drag_scale) == 0x19a0);
const _: () = assert!(core::mem::offset_of!(CarF1Aero, lift_scale) == 0x19a4);

#[repr(C)]
pub struct CarFuelEcu {
    _pad_0x0: [u8; 0x128],
    pub fuel_mode: i32,
}
const _: () = assert!(core::mem::offset_of!(CarFuelEcu, fuel_mode) == 0x128);

#[repr(C)]
pub struct CarFuelTank {
    _pad_0x0: [u8; 0x8],
    pub car: *mut CarLive,
    _pad_0x10: [u8; 0x88],
    pub fuel_capacity: CarObfFloat,
    _pad_0xa4: [u8; 0x5c],
    pub fuel_mass: f32,
    _pad_0x104: [u8; 0x4],
    pub fuel_flow: f32,
}
const _: () = assert!(core::mem::offset_of!(CarFuelTank, car) == 0x8);
const _: () = assert!(core::mem::offset_of!(CarFuelTank, fuel_capacity) == 0x98);
const _: () = assert!(core::mem::offset_of!(CarFuelTank, fuel_mass) == 0x100);
const _: () = assert!(core::mem::offset_of!(CarFuelTank, fuel_flow) == 0x108);

#[repr(C)]
pub struct CarGearBox {
    _pad_0x0: [u8; 0x8],
    pub car: *mut CarLive,
    _pad_0x10: [u8; 0x18],
    pub final_drive_ratio: CarObfFloat,
    _pad_0x34: [u8; 0x4],
    pub gear_ratio_reverse: CarObfFloat,
    _pad_0x44: [u8; 0x14],
    pub gear_ratio1st: CarObfFloat,
    _pad_0x64: [u8; 0x4],
    pub gear_ratio2nd: CarObfFloat,
    _pad_0x74: [u8; 0x4],
    pub gear_ratio3rd: CarObfFloat,
    _pad_0x84: [u8; 0x4],
    pub gear_ratio4th: CarObfFloat,
    _pad_0x94: [u8; 0x4],
    pub gear_ratio5th: CarObfFloat,
    _pad_0xa4: [u8; 0x4],
    pub gear_ratio6th: CarObfFloat,
    _pad_0xb4: [u8; 0x4],
    pub gear_ratio7th: CarObfFloat,
    _pad_0xc4: [u8; 0x4],
    pub gear_ratio8th: CarObfFloat,
    _pad_0xd4: [u8; 0x4],
    pub gear_ratio9th: CarObfFloat,
    _pad_0xe4: [u8; 0x4],
    pub max_clutch_torque: f32,
    _pad_0xec: [u8; 0x454],
    pub use_setup_ratios: u8,
    _pad_0x541: [u8; 0xb7],
    pub top_gear_index: i32,
    pub inv_total_ratio: [f32; 11],
    _pad_0x628: [u8; 0x58],
    pub gear_index: u32,
    pub target_gear_index: u32,
}
const _: () = assert!(core::mem::offset_of!(CarGearBox, car) == 0x8);
const _: () = assert!(core::mem::offset_of!(CarGearBox, final_drive_ratio) == 0x28);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio_reverse) == 0x38);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio1st) == 0x58);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio2nd) == 0x68);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio3rd) == 0x78);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio4th) == 0x88);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio5th) == 0x98);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio6th) == 0xa8);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio7th) == 0xb8);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio8th) == 0xc8);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_ratio9th) == 0xd8);
const _: () = assert!(core::mem::offset_of!(CarGearBox, max_clutch_torque) == 0xe8);
const _: () = assert!(core::mem::offset_of!(CarGearBox, use_setup_ratios) == 0x540);
const _: () = assert!(core::mem::offset_of!(CarGearBox, top_gear_index) == 0x5f8);
const _: () = assert!(core::mem::offset_of!(CarGearBox, inv_total_ratio) == 0x5fc);
const _: () = assert!(core::mem::offset_of!(CarGearBox, gear_index) == 0x680);
const _: () = assert!(core::mem::offset_of!(CarGearBox, target_gear_index) == 0x684);

#[repr(C)]
pub struct CarLive {
    pub teleport_tag: *mut c_char,
    pub teleport_priority: i32,
    _pad_0xc: [u8; 0x4],
    pub teleport_prev_mtx: [f32; 16],
    pub teleport_new_mtx: [f32; 16],
    pub teleport_dt: f32,
    pub teleport_flags: u32,
    _pad_0x98: [u8; 0x50],
    pub vehicle_id: u8,
    _pad_0xe9: [u8; 0x2af],
    pub system_count: u32,
    _pad_0x39c: [u8; 0x104],
    pub upgrades_allocator: *mut c_void,
    _pad_0x4a8: [u8; 0x88],
    pub engine: *mut c_void,
    pub gearbox: *mut c_void,
    pub fuel_tank: *mut c_void,
    pub fuel_ecu: *mut c_void,
    pub brake_ecu: *mut c_void,
    pub chassis: *mut c_void,
    pub steering: *mut c_void,
    pub core_aero: *mut c_void,
    pub f1_aero: *mut c_void,
    pub differential: *mut c_void,
    pub suspension: *mut c_void,
    _pad_0x588: [u8; 0x8],
    pub f1_tuning: *mut c_void,
    pub f1_assists: *mut c_void,
    pub turbo: *mut c_void,
    pub ers: *mut c_void,
    _pad_0x5b0: [u8; 0x18],
    pub f1_wing_damage: *mut c_void,
    pub f1_suspension_damage: *mut c_void,
    _pad_0x5d8: [u8; 0x9968],
    pub handling_version: u32,
}
const _: () = assert!(core::mem::offset_of!(CarLive, teleport_tag) == 0x0);
const _: () = assert!(core::mem::offset_of!(CarLive, teleport_priority) == 0x8);
const _: () = assert!(core::mem::offset_of!(CarLive, teleport_prev_mtx) == 0x10);
const _: () = assert!(core::mem::offset_of!(CarLive, teleport_new_mtx) == 0x50);
const _: () = assert!(core::mem::offset_of!(CarLive, teleport_dt) == 0x90);
const _: () = assert!(core::mem::offset_of!(CarLive, teleport_flags) == 0x94);
const _: () = assert!(core::mem::offset_of!(CarLive, vehicle_id) == 0xe8);
const _: () = assert!(core::mem::offset_of!(CarLive, system_count) == 0x398);
const _: () = assert!(core::mem::offset_of!(CarLive, upgrades_allocator) == 0x4a0);
const _: () = assert!(core::mem::offset_of!(CarLive, engine) == 0x530);
const _: () = assert!(core::mem::offset_of!(CarLive, gearbox) == 0x538);
const _: () = assert!(core::mem::offset_of!(CarLive, fuel_tank) == 0x540);
const _: () = assert!(core::mem::offset_of!(CarLive, fuel_ecu) == 0x548);
const _: () = assert!(core::mem::offset_of!(CarLive, brake_ecu) == 0x550);
const _: () = assert!(core::mem::offset_of!(CarLive, chassis) == 0x558);
const _: () = assert!(core::mem::offset_of!(CarLive, steering) == 0x560);
const _: () = assert!(core::mem::offset_of!(CarLive, core_aero) == 0x568);
const _: () = assert!(core::mem::offset_of!(CarLive, f1_aero) == 0x570);
const _: () = assert!(core::mem::offset_of!(CarLive, differential) == 0x578);
const _: () = assert!(core::mem::offset_of!(CarLive, suspension) == 0x580);
const _: () = assert!(core::mem::offset_of!(CarLive, f1_tuning) == 0x590);
const _: () = assert!(core::mem::offset_of!(CarLive, f1_assists) == 0x598);
const _: () = assert!(core::mem::offset_of!(CarLive, turbo) == 0x5a0);
const _: () = assert!(core::mem::offset_of!(CarLive, ers) == 0x5a8);
const _: () = assert!(core::mem::offset_of!(CarLive, f1_wing_damage) == 0x5c8);
const _: () = assert!(core::mem::offset_of!(CarLive, f1_suspension_damage) == 0x5d0);
const _: () = assert!(core::mem::offset_of!(CarLive, handling_version) == 0x9f40);

#[repr(C)]
pub struct CarNumberComponent {
    _pad_0x0: [u8; 0x278],
    pub entity: *mut CareerEntity,
    _pad_0x280: [u8; 0x38],
}
const _: () = assert!(core::mem::offset_of!(CarNumberComponent, entity) == 0x278);
const _: () = assert!(core::mem::size_of::<CarNumberComponent>() == 0x2b8);

#[repr(C)]
pub struct CarSteering {
    _pad_0x0: [u8; 0x8],
    pub car: *mut CarLive,
    _pad_0x10: [u8; 0x18],
    pub max_steering_lock: CarObfFloat,
}
const _: () = assert!(core::mem::offset_of!(CarSteering, car) == 0x8);
const _: () = assert!(core::mem::offset_of!(CarSteering, max_steering_lock) == 0x28);

#[repr(C)]
pub struct CarSuspAxle {
    _pad_0x0: [u8; 0x15e4],
    pub ride_height_offset: f32,
    _pad_0x15e8: [u8; 0x68],
    pub setup_camber: f32,
    pub setup_toe: f32,
    _pad_0x1658: [u8; 0xc],
    pub setup_anti_roll: f32,
}
const _: () = assert!(core::mem::offset_of!(CarSuspAxle, ride_height_offset) == 0x15e4);
const _: () = assert!(core::mem::offset_of!(CarSuspAxle, setup_camber) == 0x1650);
const _: () = assert!(core::mem::offset_of!(CarSuspAxle, setup_toe) == 0x1654);
const _: () = assert!(core::mem::offset_of!(CarSuspAxle, setup_anti_roll) == 0x1664);

#[repr(C)]
pub struct CarTurbo {
    _pad_0x0: [u8; 0x8],
    pub car: *mut CarLive,
    _pad_0x10: [u8; 0x2c8],
    pub max_boost: f32,
}
const _: () = assert!(core::mem::offset_of!(CarTurbo, car) == 0x8);
const _: () = assert!(core::mem::offset_of!(CarTurbo, max_boost) == 0x2d8);

#[repr(C)]
pub struct CareerBankComponent {
    _pad_0x0: [u8; 0x298],
    pub team_accounts_begin: *mut BankAccount,
    pub team_accounts_end: *mut BankAccount,
}
const _: () = assert!(core::mem::offset_of!(CareerBankComponent, team_accounts_begin) == 0x298);
const _: () = assert!(core::mem::offset_of!(CareerBankComponent, team_accounts_end) == 0x2a0);

#[repr(C)]
pub struct CareerDriverContractRenewalComponent {
    _pad_0x0: [u8; 0x3a1],
    pub disable_driver_moves: u8,
    _pad_0x3a2: [u8; 0x6],
    pub driver_moves_allocator: *mut c_void,
    pub driver_moves_begin: *mut DriverMoveRecord,
    pub driver_moves_end: *mut DriverMoveRecord,
    pub driver_moves_capacity: u64,
}
const _: () =
    assert!(core::mem::offset_of!(CareerDriverContractRenewalComponent, disable_driver_moves) == 0x3a1);
const _: () =
    assert!(core::mem::offset_of!(CareerDriverContractRenewalComponent, driver_moves_allocator) == 0x3a8);
const _: () =
    assert!(core::mem::offset_of!(CareerDriverContractRenewalComponent, driver_moves_begin) == 0x3b0);
const _: () = assert!(core::mem::offset_of!(CareerDriverContractRenewalComponent, driver_moves_end) == 0x3b8);
const _: () =
    assert!(core::mem::offset_of!(CareerDriverContractRenewalComponent, driver_moves_capacity) == 0x3c0);

#[repr(C)]
pub struct CareerEntity {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x2d8],
    pub entrants_begin: *mut EntrantRecord,
    pub entrants_end: *mut EntrantRecord,
    _pad_0x2f0: [u8; 0x8],
    pub entrant_update_busy: u8,
    _pad_0x2f9: [u8; 0xc5f],
    pub track_data: *mut TrackDataSystem,
    _pad_0xf60: [u8; 0xd0],
    pub component_registry: *mut *mut c_void,
    _pad_0x1038: [u8; 0x8],
    pub championships_begin: *mut c_void,
    pub championships_end: *mut c_void,
    _pad_0x1050: [u8; 0x8],
    pub current_championship: *mut c_void,
    _pad_0x1060: [u8; 0x8],
    pub championship_order_begin: *mut *mut c_void,
    pub championship_order_end: *mut *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(CareerEntity, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(CareerEntity, entrants_begin) == 0x2e0);
const _: () = assert!(core::mem::offset_of!(CareerEntity, entrants_end) == 0x2e8);
const _: () = assert!(core::mem::offset_of!(CareerEntity, entrant_update_busy) == 0x2f8);
const _: () = assert!(core::mem::offset_of!(CareerEntity, track_data) == 0xf58);
const _: () = assert!(core::mem::offset_of!(CareerEntity, component_registry) == 0x1030);
const _: () = assert!(core::mem::offset_of!(CareerEntity, championships_begin) == 0x1040);
const _: () = assert!(core::mem::offset_of!(CareerEntity, championships_end) == 0x1048);
const _: () = assert!(core::mem::offset_of!(CareerEntity, current_championship) == 0x1058);
const _: () = assert!(core::mem::offset_of!(CareerEntity, championship_order_begin) == 0x1068);
const _: () = assert!(core::mem::offset_of!(CareerEntity, championship_order_end) == 0x1070);

#[repr(C)]
pub struct CareerSponsorsComponent {
    _pad_0x0: [u8; 0x298],
    pub teams_begin: *mut SponsorTeamRecord,
    pub teams_end: *mut SponsorTeamRecord,
}
const _: () = assert!(core::mem::offset_of!(CareerSponsorsComponent, teams_begin) == 0x298);
const _: () = assert!(core::mem::offset_of!(CareerSponsorsComponent, teams_end) == 0x2a0);

#[repr(C)]
pub struct CareerTimeComponent {
    _pad_0x0: [u8; 0x290],
    pub season_dates: u8,
    _pad_0x291: [u8; 0x1f],
    pub current_minutes: u32,
}
const _: () = assert!(core::mem::offset_of!(CareerTimeComponent, season_dates) == 0x290);
const _: () = assert!(core::mem::offset_of!(CareerTimeComponent, current_minutes) == 0x2b0);

#[repr(C)]
pub struct Championship {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x200],
    pub list_vtable: *mut *mut c_void,
    _pad_0x210: [u8; 0x328],
    pub completion: u32,
    _pad_0x53c: [u8; 0xc],
    pub seasons_begin: *mut ChampionshipSeasonEntry,
    pub seasons_end: *mut ChampionshipSeasonEntry,
    _pad_0x558: [u8; 0x8],
    pub season_index: u32,
    _pad_0x564: [u8; 0x4c],
}
const _: () = assert!(core::mem::offset_of!(Championship, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(Championship, list_vtable) == 0x208);
const _: () = assert!(core::mem::offset_of!(Championship, completion) == 0x538);
const _: () = assert!(core::mem::offset_of!(Championship, seasons_begin) == 0x548);
const _: () = assert!(core::mem::offset_of!(Championship, seasons_end) == 0x550);
const _: () = assert!(core::mem::offset_of!(Championship, season_index) == 0x560);
const _: () = assert!(core::mem::size_of::<Championship>() == 0x5b0);

#[repr(C)]
pub struct ChampionshipHolder {
    _pad_0x0: [u8; 0x260],
    pub entity: *mut CareerEntity,
}
const _: () = assert!(core::mem::offset_of!(ChampionshipHolder, entity) == 0x260);

#[repr(C)]
pub struct ChampionshipSeasonEntry {
    _pad_0x0: [u8; 0x8],
    pub season: *mut Season,
}
const _: () = assert!(core::mem::offset_of!(ChampionshipSeasonEntry, season) == 0x8);
const _: () = assert!(core::mem::size_of::<ChampionshipSeasonEntry>() == 0x10);

#[repr(C)]
pub struct ContextStatSet {
    _pad_0x0: [u8; 0x10],
    pub stats_begin: *mut DriverStatistic,
    pub stats_end: *mut DriverStatistic,
    _pad_0x20: [u8; 0x8],
    pub mask_a: u32,
    pub mask_b: u32,
    _pad_0x30: [u8; 0x8],
}
const _: () = assert!(core::mem::offset_of!(ContextStatSet, stats_begin) == 0x10);
const _: () = assert!(core::mem::offset_of!(ContextStatSet, stats_end) == 0x18);
const _: () = assert!(core::mem::offset_of!(ContextStatSet, mask_a) == 0x28);
const _: () = assert!(core::mem::offset_of!(ContextStatSet, mask_b) == 0x2c);
const _: () = assert!(core::mem::size_of::<ContextStatSet>() == 0x38);

#[repr(C)]
pub struct ContractRecord {
    _pad_0x0: [u8; 0x28],
    pub clause_stat: u8,
    _pad_0x29: [u8; 0x3],
    pub clause_target: i32,
    _pad_0x30: [u8; 0x4],
    pub salary: i32,
    pub signing_bonus: i32,
    pub team_perf_money: i32,
    pub team_perf_race_value: i32,
    pub success_chance: f32,
    pub negotiation_bonus: i32,
    pub is_counteroffer: u8,
    _pad_0x4d: [u8; 0x3],
    pub driver: u32,
    pub old_team: u32,
    pub new_team: u32,
    pub driver_move_reason: u8,
    _pad_0x5d: [u8; 0x3],
    pub weekend_index: u32,
    pub career_day_index: u16,
    _pad_0x66: [u8; 0x2],
    pub day_contract_sent: u16,
    pub day_contract_decided: u16,
    pub accepted: u8,
    pub acknowledged: u8,
    _pad_0x6e: [u8; 0x2],
}
const _: () = assert!(core::mem::offset_of!(ContractRecord, clause_stat) == 0x28);
const _: () = assert!(core::mem::offset_of!(ContractRecord, clause_target) == 0x2c);
const _: () = assert!(core::mem::offset_of!(ContractRecord, salary) == 0x34);
const _: () = assert!(core::mem::offset_of!(ContractRecord, signing_bonus) == 0x38);
const _: () = assert!(core::mem::offset_of!(ContractRecord, team_perf_money) == 0x3c);
const _: () = assert!(core::mem::offset_of!(ContractRecord, team_perf_race_value) == 0x40);
const _: () = assert!(core::mem::offset_of!(ContractRecord, success_chance) == 0x44);
const _: () = assert!(core::mem::offset_of!(ContractRecord, negotiation_bonus) == 0x48);
const _: () = assert!(core::mem::offset_of!(ContractRecord, is_counteroffer) == 0x4c);
const _: () = assert!(core::mem::offset_of!(ContractRecord, driver) == 0x50);
const _: () = assert!(core::mem::offset_of!(ContractRecord, old_team) == 0x54);
const _: () = assert!(core::mem::offset_of!(ContractRecord, new_team) == 0x58);
const _: () = assert!(core::mem::offset_of!(ContractRecord, driver_move_reason) == 0x5c);
const _: () = assert!(core::mem::offset_of!(ContractRecord, weekend_index) == 0x60);
const _: () = assert!(core::mem::offset_of!(ContractRecord, career_day_index) == 0x64);
const _: () = assert!(core::mem::offset_of!(ContractRecord, day_contract_sent) == 0x68);
const _: () = assert!(core::mem::offset_of!(ContractRecord, day_contract_decided) == 0x6a);
const _: () = assert!(core::mem::offset_of!(ContractRecord, accepted) == 0x6c);
const _: () = assert!(core::mem::offset_of!(ContractRecord, acknowledged) == 0x6d);
const _: () = assert!(core::mem::size_of::<ContractRecord>() == 0x70);

#[repr(C)]
pub struct ControllerSet {
    _pad_0x0: [u8; 0x10],
    pub mode_masked: u32,
    _pad_0x14: [u8; 0x4],
    pub mode_key: u32,
    _pad_0x1c: [u8; 0x4],
    pub human_controller: *mut c_void,
    pub ai_controller: *mut c_void,
    pub flags: u32,
    pub vehicle: u8,
}
const _: () = assert!(core::mem::offset_of!(ControllerSet, mode_masked) == 0x10);
const _: () = assert!(core::mem::offset_of!(ControllerSet, mode_key) == 0x18);
const _: () = assert!(core::mem::offset_of!(ControllerSet, human_controller) == 0x20);
const _: () = assert!(core::mem::offset_of!(ControllerSet, ai_controller) == 0x28);
const _: () = assert!(core::mem::offset_of!(ControllerSet, flags) == 0x30);
const _: () = assert!(core::mem::offset_of!(ControllerSet, vehicle) == 0x34);

#[repr(C)]
pub struct CostCapComponent {
    _pad_0x0: [u8; 0x2d8],
    pub team_slots: *mut *mut CostCapTeam,
    pub team_ctrl: *mut u8,
}
const _: () = assert!(core::mem::offset_of!(CostCapComponent, team_slots) == 0x2d8);
const _: () = assert!(core::mem::offset_of!(CostCapComponent, team_ctrl) == 0x2e0);

#[repr(C)]
pub struct CostCapSeason {
    pub season_id: i32,
    _pad_0x4: [u8; 0x4],
    pub cached_expenses: i32,
    pub cost_cap: i32,
    pub cached_modifier: f32,
}
const _: () = assert!(core::mem::offset_of!(CostCapSeason, season_id) == 0x0);
const _: () = assert!(core::mem::offset_of!(CostCapSeason, cached_expenses) == 0x8);
const _: () = assert!(core::mem::offset_of!(CostCapSeason, cost_cap) == 0xc);
const _: () = assert!(core::mem::offset_of!(CostCapSeason, cached_modifier) == 0x10);

#[repr(C)]
pub struct CostCapTeam {
    pub account: u16,
    _pad_0x2: [u8; 0x2e],
    pub season_slots: *mut *mut CostCapSeason,
    pub season_ctrl: *mut u8,
}
const _: () = assert!(core::mem::offset_of!(CostCapTeam, account) == 0x0);
const _: () = assert!(core::mem::offset_of!(CostCapTeam, season_slots) == 0x30);
const _: () = assert!(core::mem::offset_of!(CostCapTeam, season_ctrl) == 0x38);

#[repr(C)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
}
const _: () = assert!(core::mem::offset_of!(CurvePoint, x) == 0x0);
const _: () = assert!(core::mem::offset_of!(CurvePoint, y) == 0x4);
const _: () = assert!(core::mem::size_of::<CurvePoint>() == 0x8);

#[repr(C)]
pub struct DriverCalendar {
    _pad_0x0: [u8; 0x50],
    pub activities_begin: *mut c_void,
    pub activities_end: *mut c_void,
    _pad_0x60: [u8; 0x60],
    pub driver: u32,
    _pad_0xc4: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(DriverCalendar, activities_begin) == 0x50);
const _: () = assert!(core::mem::offset_of!(DriverCalendar, activities_end) == 0x58);
const _: () = assert!(core::mem::offset_of!(DriverCalendar, driver) == 0xc0);
const _: () = assert!(core::mem::size_of::<DriverCalendar>() == 0xc8);

#[repr(C)]
pub struct DriverCareerContractOwner {
    _pad_0x0: [u8; 0x298],
    pub contracts_begin: *mut c_void,
    pub contracts_end: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(DriverCareerContractOwner, contracts_begin) == 0x298);
const _: () = assert!(core::mem::offset_of!(DriverCareerContractOwner, contracts_end) == 0x2a0);

#[repr(C)]
pub struct DriverCareerContractsManagerComponent {
    _pad_0x0: [u8; 0x270],
    pub weekend_source: *mut c_void,
    pub entity: *mut CareerEntity,
    _pad_0x280: [u8; 0xd0],
    pub contract_owner: *mut DriverCareerContractOwner,
    pub career_time: *mut CareerTimeComponent,
    _pad_0x360: [u8; 0x8],
    pub car_numbers: *mut CarNumberComponent,
    pub schedule: *mut ScheduleComponent,
    _pad_0x378: [u8; 0x30],
    pub secret_meetings_begin: *mut c_void,
    pub secret_meetings_end: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, weekend_source) == 0x270);
const _: () = assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, entity) == 0x278);
const _: () = assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, contract_owner) == 0x350);
const _: () = assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, career_time) == 0x358);
const _: () = assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, car_numbers) == 0x368);
const _: () = assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, schedule) == 0x370);
const _: () =
    assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, secret_meetings_begin) == 0x3a8);
const _: () =
    assert!(core::mem::offset_of!(DriverCareerContractsManagerComponent, secret_meetings_end) == 0x3b0);

#[repr(C)]
pub struct DriverContract {
    pub team: u32,
    _pad_0x4: [u8; 0x8],
    pub signed_day: u16,
    pub kind: u8,
    _pad_0xf: [u8; 0x1],
    pub end_day: u16,
    pub state: u8,
    _pad_0x13: [u8; 0x1],
    pub end_value: u32,
    _pad_0x18: [u8; 0x14],
    pub active: u8,
    _pad_0x2d: [u8; 0xb],
    pub history_begin: *mut c_void,
    pub history_end: *mut c_void,
    _pad_0x48: [u8; 0x8],
    pub driver: u32,
}
const _: () = assert!(core::mem::offset_of!(DriverContract, team) == 0x0);
const _: () = assert!(core::mem::offset_of!(DriverContract, signed_day) == 0xc);
const _: () = assert!(core::mem::offset_of!(DriverContract, kind) == 0xe);
const _: () = assert!(core::mem::offset_of!(DriverContract, end_day) == 0x10);
const _: () = assert!(core::mem::offset_of!(DriverContract, state) == 0x12);
const _: () = assert!(core::mem::offset_of!(DriverContract, end_value) == 0x14);
const _: () = assert!(core::mem::offset_of!(DriverContract, active) == 0x2c);
const _: () = assert!(core::mem::offset_of!(DriverContract, history_begin) == 0x38);
const _: () = assert!(core::mem::offset_of!(DriverContract, history_end) == 0x40);
const _: () = assert!(core::mem::offset_of!(DriverContract, driver) == 0x50);

#[repr(C)]
pub struct DriverContractChangedMessage {
    pub vtable: *mut *mut c_void,
    pub reserved: u64,
    pub driver: u32,
    pub before: [u8; 48],
    pub after: [u8; 44],
    pub kind: u8,
    _pad_0x71: [u8; 0x7],
}
const _: () = assert!(core::mem::offset_of!(DriverContractChangedMessage, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(DriverContractChangedMessage, reserved) == 0x8);
const _: () = assert!(core::mem::offset_of!(DriverContractChangedMessage, driver) == 0x10);
const _: () = assert!(core::mem::offset_of!(DriverContractChangedMessage, before) == 0x14);
const _: () = assert!(core::mem::offset_of!(DriverContractChangedMessage, after) == 0x44);
const _: () = assert!(core::mem::offset_of!(DriverContractChangedMessage, kind) == 0x70);
const _: () = assert!(core::mem::size_of::<DriverContractChangedMessage>() == 0x78);

#[repr(C)]
pub struct DriverContractsMyTeamComponent {
    _pad_0x0: [u8; 0x2e8],
    pub car_numbers: *mut CarNumberComponent,
    _pad_0x2f0: [u8; 0x4a8],
    pub pending_contracts_begin: *mut ContractRecord,
    _pad_0x7a0: [u8; 0x18],
    pub decided_contracts_begin: *mut ContractRecord,
    _pad_0x7c0: [u8; 0x18],
    pub pending_moves_begin: *mut ContractRecord,
    pub pending_moves_end: *mut ContractRecord,
    _pad_0x7e8: [u8; 0x10],
    pub last_season_signings_begin: *mut ContractRecord,
    pub last_season_signings_end: *mut ContractRecord,
}
const _: () = assert!(core::mem::offset_of!(DriverContractsMyTeamComponent, car_numbers) == 0x2e8);
const _: () =
    assert!(core::mem::offset_of!(DriverContractsMyTeamComponent, pending_contracts_begin) == 0x798);
const _: () =
    assert!(core::mem::offset_of!(DriverContractsMyTeamComponent, decided_contracts_begin) == 0x7b8);
const _: () = assert!(core::mem::offset_of!(DriverContractsMyTeamComponent, pending_moves_begin) == 0x7d8);
const _: () = assert!(core::mem::offset_of!(DriverContractsMyTeamComponent, pending_moves_end) == 0x7e0);
const _: () =
    assert!(core::mem::offset_of!(DriverContractsMyTeamComponent, last_season_signings_begin) == 0x7f8);
const _: () =
    assert!(core::mem::offset_of!(DriverContractsMyTeamComponent, last_season_signings_end) == 0x800);

#[repr(C)]
pub struct DriverMoveRecord {
    pub driver: u32,
    pub old_team: u32,
    pub new_team: u32,
    pub reason: u8,
    _pad_0xd: [u8; 0x3],
    pub weekend_index: u32,
    pub day_index: u16,
    _pad_0x16: [u8; 0x2],
}
const _: () = assert!(core::mem::offset_of!(DriverMoveRecord, driver) == 0x0);
const _: () = assert!(core::mem::offset_of!(DriverMoveRecord, old_team) == 0x4);
const _: () = assert!(core::mem::offset_of!(DriverMoveRecord, new_team) == 0x8);
const _: () = assert!(core::mem::offset_of!(DriverMoveRecord, reason) == 0xc);
const _: () = assert!(core::mem::offset_of!(DriverMoveRecord, weekend_index) == 0x10);
const _: () = assert!(core::mem::offset_of!(DriverMoveRecord, day_index) == 0x14);
const _: () = assert!(core::mem::size_of::<DriverMoveRecord>() == 0x18);

#[repr(C)]
pub struct DriverMoveConfirmedMessage {
    pub vtable: *mut *mut c_void,
    pub reserved: u64,
    pub r#move: DriverMoveRecord,
}
const _: () = assert!(core::mem::offset_of!(DriverMoveConfirmedMessage, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(DriverMoveConfirmedMessage, reserved) == 0x8);
const _: () = assert!(core::mem::offset_of!(DriverMoveConfirmedMessage, r#move) == 0x10);
const _: () = assert!(core::mem::size_of::<DriverMoveConfirmedMessage>() == 0x28);

#[repr(C)]
pub struct LineupChange {
    pub driver: u32,
    pub old_team: u32,
    pub new_team: u32,
    _pad_0xc: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(LineupChange, driver) == 0x0);
const _: () = assert!(core::mem::offset_of!(LineupChange, old_team) == 0x4);
const _: () = assert!(core::mem::offset_of!(LineupChange, new_team) == 0x8);
const _: () = assert!(core::mem::size_of::<LineupChange>() == 0x10);

#[repr(C)]
pub struct DriverMovesFinalisedMessage {
    pub vtable: *mut *mut c_void,
    pub reserved: u64,
    pub changes: [LineupChange; 40],
    pub count: u64,
}
const _: () = assert!(core::mem::offset_of!(DriverMovesFinalisedMessage, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(DriverMovesFinalisedMessage, reserved) == 0x8);
const _: () = assert!(core::mem::offset_of!(DriverMovesFinalisedMessage, changes) == 0x10);
const _: () = assert!(core::mem::offset_of!(DriverMovesFinalisedMessage, count) == 0x290);
const _: () = assert!(core::mem::size_of::<DriverMovesFinalisedMessage>() == 0x298);

#[repr(C)]
pub struct DriverSecretMeetings {
    _pad_0x0: [u8; 0x10],
    pub current_allocator: *mut c_void,
    pub current: *mut SecretMeeting,
    _pad_0x20: [u8; 0x8],
    pub previous_begin: *mut c_void,
    pub previous_end: *mut c_void,
    _pad_0x38: [u8; 0x8],
    pub driver: u32,
    _pad_0x44: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(DriverSecretMeetings, current_allocator) == 0x10);
const _: () = assert!(core::mem::offset_of!(DriverSecretMeetings, current) == 0x18);
const _: () = assert!(core::mem::offset_of!(DriverSecretMeetings, previous_begin) == 0x28);
const _: () = assert!(core::mem::offset_of!(DriverSecretMeetings, previous_end) == 0x30);
const _: () = assert!(core::mem::offset_of!(DriverSecretMeetings, driver) == 0x40);
const _: () = assert!(core::mem::size_of::<DriverSecretMeetings>() == 0x48);

#[repr(C)]
pub struct EvolutionCurve {
    pub vtable: *mut *mut c_void,
    pub inv_x_range: f32,
    pub min_x: f32,
    pub y_step: f32,
    pub end_value: f32,
    _pad_0x18: [u8; 0x4],
    pub min_y: f32,
    pub max_y: f32,
    pub lut: [u8; 65],
    _pad_0x65: [u8; 0x3],
    pub allocator: *mut c_void,
    pub points_begin: *mut CurvePoint,
    _pad_0x78: [u8; 0x8],
    pub points_capacity: u64,
}
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, inv_x_range) == 0x8);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, min_x) == 0xc);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, y_step) == 0x10);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, end_value) == 0x14);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, min_y) == 0x1c);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, max_y) == 0x20);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, lut) == 0x24);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, allocator) == 0x68);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, points_begin) == 0x70);
const _: () = assert!(core::mem::offset_of!(EvolutionCurve, points_capacity) == 0x80);
const _: () = assert!(core::mem::size_of::<EvolutionCurve>() == 0x88);

#[repr(C)]
pub struct DriverStatistic {
    _pad_0x0: [u8; 0x10],
    pub info_stat_id: u16,
    _pad_0x12: [u8; 0x6],
    pub evolution_graph: EvolutionCurve,
    pub patched_evolution_graph: EvolutionCurve,
    pub has_patched_graph: u8,
    _pad_0x129: [u8; 0x7],
    pub temporary_boost_value: f32,
    pub is_dynamic_stat: u8,
    _pad_0x135: [u8; 0x3],
    pub dynamic_level: f32,
    _pad_0x13c: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(DriverStatistic, info_stat_id) == 0x10);
const _: () = assert!(core::mem::offset_of!(DriverStatistic, evolution_graph) == 0x18);
const _: () = assert!(core::mem::offset_of!(DriverStatistic, patched_evolution_graph) == 0xa0);
const _: () = assert!(core::mem::offset_of!(DriverStatistic, has_patched_graph) == 0x128);
const _: () = assert!(core::mem::offset_of!(DriverStatistic, temporary_boost_value) == 0x130);
const _: () = assert!(core::mem::offset_of!(DriverStatistic, is_dynamic_stat) == 0x134);
const _: () = assert!(core::mem::offset_of!(DriverStatistic, dynamic_level) == 0x138);
const _: () = assert!(core::mem::size_of::<DriverStatistic>() == 0x140);

#[repr(C)]
pub struct DriverStatisticsComponent {
    _pad_0x0: [u8; 0x278],
    pub stat_context: *mut c_void,
    _pad_0x280: [u8; 0x10],
    pub career_time: *mut CareerTimeComponent,
    _pad_0x298: [u8; 0x10],
    pub stats_data: u8,
    pub use_story_mode_data: u8,
    _pad_0x2aa: [u8; 0x2],
    pub last_patch_applied_checksum: u32,
    pub last_patch_declined_checksum: u32,
    _pad_0x2b4: [u8; 0x1c],
    pub stat_infos_begin: *mut c_void,
    pub stat_infos_end: *mut c_void,
    _pad_0x2e0: [u8; 0x30],
    pub driver_data_begin: *mut DriverStatsEntry,
    pub driver_data_end: *mut DriverStatsEntry,
}
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, stat_context) == 0x278);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, career_time) == 0x290);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, stats_data) == 0x2a8);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, use_story_mode_data) == 0x2a9);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, last_patch_applied_checksum) == 0x2ac);
const _: () =
    assert!(core::mem::offset_of!(DriverStatisticsComponent, last_patch_declined_checksum) == 0x2b0);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, stat_infos_begin) == 0x2d0);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, stat_infos_end) == 0x2d8);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, driver_data_begin) == 0x310);
const _: () = assert!(core::mem::offset_of!(DriverStatisticsComponent, driver_data_end) == 0x318);

#[repr(C)]
pub struct DriverStatsEntry {
    pub kind: u8,
    _pad_0x1: [u8; 0x1],
    pub stat_id: u16,
    _pad_0x4: [u8; 0x14],
    pub stats_begin: *mut DriverStatistic,
    pub stats_end: *mut DriverStatistic,
    _pad_0x28: [u8; 0x10],
    pub context_sets_begin: *mut ContextStatSet,
    pub context_sets_end: *mut ContextStatSet,
    _pad_0x48: [u8; 0x8],
}
const _: () = assert!(core::mem::offset_of!(DriverStatsEntry, kind) == 0x0);
const _: () = assert!(core::mem::offset_of!(DriverStatsEntry, stat_id) == 0x2);
const _: () = assert!(core::mem::offset_of!(DriverStatsEntry, stats_begin) == 0x18);
const _: () = assert!(core::mem::offset_of!(DriverStatsEntry, stats_end) == 0x20);
const _: () = assert!(core::mem::offset_of!(DriverStatsEntry, context_sets_begin) == 0x38);
const _: () = assert!(core::mem::offset_of!(DriverStatsEntry, context_sets_end) == 0x40);
const _: () = assert!(core::mem::size_of::<DriverStatsEntry>() == 0x50);

#[repr(C)]
pub struct EntrantRecord {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x40],
    pub ordinal_flags: u32,
    _pad_0x4c: [u8; 0x4],
    pub id_lo: u8,
    _pad_0x51: [u8; 0x1],
    pub id_hi: u16,
    pub driver_customisation: u32,
    pub team_stat_id: u32,
    pub vehicle_ref: u32,
    pub driver_reg_value: u32,
    pub vehicle_customisation: u32,
    pub badge_stat_id: u32,
    _pad_0x6c: [u8; 0x4],
    pub subobj_vtable: *mut *mut c_void,
    pub const_ffff0012: u32,
    _pad_0x7c: [u8; 0x4],
    pub team_position: u16,
    _pad_0x82: [u8; 0x1],
    pub car_number: u8,
    _pad_0x84: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(EntrantRecord, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, ordinal_flags) == 0x48);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, id_lo) == 0x50);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, id_hi) == 0x52);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, driver_customisation) == 0x54);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, team_stat_id) == 0x58);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, vehicle_ref) == 0x5c);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, driver_reg_value) == 0x60);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, vehicle_customisation) == 0x64);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, badge_stat_id) == 0x68);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, subobj_vtable) == 0x70);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, const_ffff0012) == 0x78);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, team_position) == 0x80);
const _: () = assert!(core::mem::offset_of!(EntrantRecord, car_number) == 0x83);
const _: () = assert!(core::mem::size_of::<EntrantRecord>() == 0x88);

#[repr(C)]
pub struct EntrantSetupMode {
    _pad_0x0: [u8; 0x230],
    pub selectable_list_count: u64,
}
const _: () = assert!(core::mem::offset_of!(EntrantSetupMode, selectable_list_count) == 0x230);

#[repr(C)]
pub struct EntrantWriteData {
    pub driver: u32,
    pub driver_customisation: u32,
    pub team_stat_id: u32,
    pub vehicle_ref: u32,
    pub driver_reg_value: u32,
    pub vehicle_customisation: u32,
    pub badge_stat_id: u32,
    _pad_0x1c: [u8; 0xc],
    pub seat_value: u32,
}
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, driver) == 0x0);
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, driver_customisation) == 0x4);
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, team_stat_id) == 0x8);
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, vehicle_ref) == 0xc);
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, driver_reg_value) == 0x10);
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, vehicle_customisation) == 0x14);
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, badge_stat_id) == 0x18);
const _: () = assert!(core::mem::offset_of!(EntrantWriteData, seat_value) == 0x28);
const _: () = assert!(core::mem::size_of::<EntrantWriteData>() == 0x2c);

#[repr(C)]
pub struct EntrantsManager {
    _pad_0x0: [u8; 0x2b0],
    pub game_setup_mode: *mut GameSetupMode,
    _pad_0x2b8: [u8; 0x28],
    pub session_begin: *mut EntrantRecord,
    pub session_end: *mut EntrantRecord,
}
const _: () = assert!(core::mem::offset_of!(EntrantsManager, game_setup_mode) == 0x2b0);
const _: () = assert!(core::mem::offset_of!(EntrantsManager, session_begin) == 0x2e0);
const _: () = assert!(core::mem::offset_of!(EntrantsManager, session_end) == 0x2e8);

#[repr(C)]
pub struct F1SeasonStandings {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x18],
    pub points_begin: *mut StandingsRound,
    pub points_end: *mut StandingsRound,
    _pad_0x30: [u8; 0x78],
    pub extra_points_begin: *mut StandingsPoints,
    pub extra_points_end: *mut StandingsPoints,
    _pad_0xb8: [u8; 0x18],
    pub results_begin: *mut StandingsRound,
    pub results_end: *mut StandingsRound,
}
const _: () = assert!(core::mem::offset_of!(F1SeasonStandings, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(F1SeasonStandings, points_begin) == 0x20);
const _: () = assert!(core::mem::offset_of!(F1SeasonStandings, points_end) == 0x28);
const _: () = assert!(core::mem::offset_of!(F1SeasonStandings, extra_points_begin) == 0xa8);
const _: () = assert!(core::mem::offset_of!(F1SeasonStandings, extra_points_end) == 0xb0);
const _: () = assert!(core::mem::offset_of!(F1SeasonStandings, results_begin) == 0xd0);
const _: () = assert!(core::mem::offset_of!(F1SeasonStandings, results_end) == 0xd8);

#[repr(C)]
pub struct FacilityNode {
    pub vtable: *mut *mut c_void,
    pub state: u8,
    _pad_0x9: [u8; 0x3],
    pub stat_id: u32,
    _pad_0x10: [u8; 0x10],
    pub children_begin: *mut c_void,
    pub children_end: *mut c_void,
    _pad_0x30: [u8; 0x10],
    pub phase: i32,
    _pad_0x44: [u8; 0x3c],
    pub cost: i32,
    _pad_0x84: [u8; 0x4],
    pub name_loc_tag: *mut c_char,
    _pad_0x90: [u8; 0x18],
    pub days_to_complete: i32,
    pub days_remaining: i32,
    pub purchased: u8,
    _pad_0xb1: [u8; 0x2],
    pub seen_completed: u8,
    _pad_0xb4: [u8; 0x28],
    pub ui_level: i32,
    pub ui_max_level: i32,
    _pad_0xe4: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(FacilityNode, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(FacilityNode, state) == 0x8);
const _: () = assert!(core::mem::offset_of!(FacilityNode, stat_id) == 0xc);
const _: () = assert!(core::mem::offset_of!(FacilityNode, children_begin) == 0x20);
const _: () = assert!(core::mem::offset_of!(FacilityNode, children_end) == 0x28);
const _: () = assert!(core::mem::offset_of!(FacilityNode, phase) == 0x40);
const _: () = assert!(core::mem::offset_of!(FacilityNode, cost) == 0x80);
const _: () = assert!(core::mem::offset_of!(FacilityNode, name_loc_tag) == 0x88);
const _: () = assert!(core::mem::offset_of!(FacilityNode, days_to_complete) == 0xa8);
const _: () = assert!(core::mem::offset_of!(FacilityNode, days_remaining) == 0xac);
const _: () = assert!(core::mem::offset_of!(FacilityNode, purchased) == 0xb0);
const _: () = assert!(core::mem::offset_of!(FacilityNode, seen_completed) == 0xb3);
const _: () = assert!(core::mem::offset_of!(FacilityNode, ui_level) == 0xdc);
const _: () = assert!(core::mem::offset_of!(FacilityNode, ui_max_level) == 0xe0);
const _: () = assert!(core::mem::size_of::<FacilityNode>() == 0xe8);

#[repr(C)]
pub struct FacilityPerkTree {
    _pad_0x0: [u8; 0x8],
    pub nodes_begin: *mut c_void,
    pub nodes_end: *mut c_void,
    _pad_0x18: [u8; 0x10],
}
const _: () = assert!(core::mem::offset_of!(FacilityPerkTree, nodes_begin) == 0x8);
const _: () = assert!(core::mem::offset_of!(FacilityPerkTree, nodes_end) == 0x10);
const _: () = assert!(core::mem::size_of::<FacilityPerkTree>() == 0x28);

#[repr(C)]
pub struct FacilityTeamData {
    _pad_0x0: [u8; 0x10],
    pub facilities_begin: *mut c_void,
    pub facilities_end: *mut c_void,
    _pad_0x20: [u8; 0x8],
    pub team_key: u32,
    _pad_0x2c: [u8; 0x54],
}
const _: () = assert!(core::mem::offset_of!(FacilityTeamData, facilities_begin) == 0x10);
const _: () = assert!(core::mem::offset_of!(FacilityTeamData, facilities_end) == 0x18);
const _: () = assert!(core::mem::offset_of!(FacilityTeamData, team_key) == 0x28);
const _: () = assert!(core::mem::size_of::<FacilityTeamData>() == 0x80);

#[repr(C)]
pub struct FacilityTeamEntry {
    _pad_0x0: [u8; 0x8],
    pub data: *mut FacilityTeamData,
}
const _: () = assert!(core::mem::offset_of!(FacilityTeamEntry, data) == 0x8);

#[repr(C)]
pub struct FlowState {
    _pad_0x0: [u8; 0x20],
    pub name_size: u64,
    pub name: *mut c_char,
    _pad_0x30: [u8; 0x80],
    pub name_hash: u32,
    _pad_0xb4: [u8; 0x4],
    pub status: i32,
}
const _: () = assert!(core::mem::offset_of!(FlowState, name_size) == 0x20);
const _: () = assert!(core::mem::offset_of!(FlowState, name) == 0x28);
const _: () = assert!(core::mem::offset_of!(FlowState, name_hash) == 0xb0);
const _: () = assert!(core::mem::offset_of!(FlowState, status) == 0xb8);

#[repr(C)]
pub struct GameManager {
    _pad_0x0: [u8; 0xc0],
    pub mode_context: *mut GameModeContext,
}
const _: () = assert!(core::mem::offset_of!(GameManager, mode_context) == 0xc0);

#[repr(C)]
pub struct GameMessageBusHolder {
    _pad_0x0: [u8; 0x8],
    pub bus: *mut NeMessageBus,
}
const _: () = assert!(core::mem::offset_of!(GameMessageBusHolder, bus) == 0x8);

#[repr(C)]
pub struct GameModeContext {
    _pad_0x0: [u8; 0x88],
    pub championship_holder: *mut ChampionshipHolder,
}
const _: () = assert!(core::mem::offset_of!(GameModeContext, championship_holder) == 0x88);

#[repr(C)]
pub struct GameSetupMode {
    _pad_0x0: [u8; 0x1e0],
    pub drs_allowed_lap_group: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(GameSetupMode, drs_allowed_lap_group) == 0x1e0);

#[repr(C)]
pub struct GameWeatherSystem {
    _pad_0x0: [u8; 0x1d98],
    pub tod: *mut TimeOfDaySystem,
    _pad_0x1da0: [u8; 0x8],
    pub tod_owner: *mut TimeOfDayOwner,
    _pad_0x1db0: [u8; 0x152],
    pub weekend_valid: u8,
    _pad_0x1f03: [u8; 0x1],
    pub season_index: u32,
    pub weekend_index: u32,
    _pad_0x1f0c: [u8; 0x4],
    pub base_seed: u32,
    pub forced_seed: u32,
    pub weekend: u32,
    _pad_0x1f1c: [u8; 0x101e],
    pub dataset_dynamic: [u8; 8],
}
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, tod) == 0x1d98);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, tod_owner) == 0x1da8);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, weekend_valid) == 0x1f02);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, season_index) == 0x1f04);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, weekend_index) == 0x1f08);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, base_seed) == 0x1f10);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, forced_seed) == 0x1f14);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, weekend) == 0x1f18);
const _: () = assert!(core::mem::offset_of!(GameWeatherSystem, dataset_dynamic) == 0x2f3a);

#[repr(C)]
pub struct GdbDatabase {
    _pad_0x0: [u8; 0xf0],
    pub loaded: u8,
}
const _: () = assert!(core::mem::offset_of!(GdbDatabase, loaded) == 0xf0);

#[repr(C)]
pub struct GdbStringRecord {
    _pad_0x0: [u8; 0x10],
    pub enc_size: u64,
    pub enc: *mut u8,
}
const _: () = assert!(core::mem::offset_of!(GdbStringRecord, enc_size) == 0x10);
const _: () = assert!(core::mem::offset_of!(GdbStringRecord, enc) == 0x18);

#[repr(C)]
pub struct GdbValueMap {
    _pad_0x0: [u8; 0x28],
    pub num_elements: u64,
    pub mask: u64,
    pub max_num_elements_allowed: u64,
    _pad_0x40: [u8; 0x8],
}
const _: () = assert!(core::mem::offset_of!(GdbValueMap, num_elements) == 0x28);
const _: () = assert!(core::mem::offset_of!(GdbValueMap, mask) == 0x30);
const _: () = assert!(core::mem::offset_of!(GdbValueMap, max_num_elements_allowed) == 0x38);
const _: () = assert!(core::mem::size_of::<GdbValueMap>() == 0x48);

#[repr(C)]
pub struct VmPropHolder {
    _pad_0x0: [u8; 0x8],
    pub node: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(VmPropHolder, node) == 0x8);

#[repr(C)]
pub struct HomeItem {
    pub vtable: *mut ItemVtbl,
    _pad_0x8: [u8; 0x8],
    pub id: *mut NeStr,
    pub button_node: VmPropHolder,
}
const _: () = assert!(core::mem::offset_of!(HomeItem, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(HomeItem, id) == 0x10);
const _: () = assert!(core::mem::offset_of!(HomeItem, button_node) == 0x18);

#[repr(C)]
pub struct ItemVtbl {
    _pad_0x0: [u8; 0x20],
    pub populate: *mut c_void,
    pub get_action: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(ItemVtbl, populate) == 0x20);
const _: () = assert!(core::mem::offset_of!(ItemVtbl, get_action) == 0x28);

#[repr(C)]
pub struct MyTeamFacilitiesComponent {
    _pad_0x0: [u8; 0x2b8],
    pub team_slots: *mut *mut FacilityTeamEntry,
    pub team_ctrl: *mut u8,
    pub team_count: u64,
}
const _: () = assert!(core::mem::offset_of!(MyTeamFacilitiesComponent, team_slots) == 0x2b8);
const _: () = assert!(core::mem::offset_of!(MyTeamFacilitiesComponent, team_ctrl) == 0x2c0);
const _: () = assert!(core::mem::offset_of!(MyTeamFacilitiesComponent, team_count) == 0x2c8);

#[repr(C)]
pub struct MyTeamFacility {
    pub facility_type: u8,
    _pad_0x1: [u8; 0x7],
    pub tree: *mut FacilityPerkTree,
    _pad_0x10: [u8; 0x40],
    pub workforce_cap: i32,
    pub workforce_cap_set: u8,
    _pad_0x55: [u8; 0x183],
    pub highest_phase_seen: i32,
    _pad_0x1dc: [u8; 0x1c],
}
const _: () = assert!(core::mem::offset_of!(MyTeamFacility, facility_type) == 0x0);
const _: () = assert!(core::mem::offset_of!(MyTeamFacility, tree) == 0x8);
const _: () = assert!(core::mem::offset_of!(MyTeamFacility, workforce_cap) == 0x50);
const _: () = assert!(core::mem::offset_of!(MyTeamFacility, workforce_cap_set) == 0x54);
const _: () = assert!(core::mem::offset_of!(MyTeamFacility, highest_phase_seen) == 0x1d8);
const _: () = assert!(core::mem::size_of::<MyTeamFacility>() == 0x1f8);

#[repr(C)]
pub struct NeAllocator {
    pub vtable: *mut NeAllocatorVtbl,
}
const _: () = assert!(core::mem::offset_of!(NeAllocator, vtable) == 0x0);

#[repr(C)]
pub struct NeAllocatorVtbl {
    _pad_0x0: [u8; 0x18],
    pub alloc: *mut c_void,
    _pad_0x20: [u8; 0x18],
    pub free: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(NeAllocatorVtbl, alloc) == 0x18);
const _: () = assert!(core::mem::offset_of!(NeAllocatorVtbl, free) == 0x38);

#[repr(C)]
pub struct NeBagCodec {
    pub vtable: *mut *mut c_void,
    pub stream: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(NeBagCodec, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(NeBagCodec, stream) == 0x8);

#[repr(C)]
pub struct NeCountingStream {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x20],
    pub count: u64,
}
const _: () = assert!(core::mem::offset_of!(NeCountingStream, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(NeCountingStream, count) == 0x28);

#[repr(C)]
pub struct NeLogGate {
    pub channel_hashes: [u32; 373],
    pub enabled: u8,
}
const _: () = assert!(core::mem::offset_of!(NeLogGate, channel_hashes) == 0x0);
const _: () = assert!(core::mem::offset_of!(NeLogGate, enabled) == 0x5d4);

#[repr(C)]
pub struct NeMemWriteStream {
    pub vtable: *mut *mut c_void,
    pub data: *mut u8,
    pub capacity: u64,
    pub position: u64,
    pub size: u64,
}
const _: () = assert!(core::mem::offset_of!(NeMemWriteStream, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(NeMemWriteStream, data) == 0x8);
const _: () = assert!(core::mem::offset_of!(NeMemWriteStream, capacity) == 0x10);
const _: () = assert!(core::mem::offset_of!(NeMemWriteStream, position) == 0x18);
const _: () = assert!(core::mem::offset_of!(NeMemWriteStream, size) == 0x20);

#[repr(C)]
pub struct NeMessageBus {
    _pad_0x0: [u8; 0x68],
    pub active: u8,
}
const _: () = assert!(core::mem::offset_of!(NeMessageBus, active) == 0x68);

#[repr(C)]
pub struct NeOptionalVec {
    pub alloc: *mut NeAllocator,
    pub begin: *mut c_void,
    pub end: *mut c_void,
    pub count: u64,
    pub engaged: u8,
    _pad_0x21: [u8; 0x7],
}
const _: () = assert!(core::mem::offset_of!(NeOptionalVec, alloc) == 0x0);
const _: () = assert!(core::mem::offset_of!(NeOptionalVec, begin) == 0x8);
const _: () = assert!(core::mem::offset_of!(NeOptionalVec, end) == 0x10);
const _: () = assert!(core::mem::offset_of!(NeOptionalVec, count) == 0x18);
const _: () = assert!(core::mem::offset_of!(NeOptionalVec, engaged) == 0x20);
const _: () = assert!(core::mem::size_of::<NeOptionalVec>() == 0x28);

#[repr(C)]
pub struct NePropertyTypeVtbl {
    _pad_0x0: [u8; 0x10],
    pub id: *mut c_void,
    _pad_0x18: [u8; 0x8],
    pub key_offset: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(NePropertyTypeVtbl, id) == 0x10);
const _: () = assert!(core::mem::offset_of!(NePropertyTypeVtbl, key_offset) == 0x20);

#[repr(C)]
pub struct NePropertyVtbl {
    _pad_0x0: [u8; 0x8],
    pub r#type: *mut c_void,
    _pad_0x10: [u8; 0x28],
    pub value: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(NePropertyVtbl, r#type) == 0x8);
const _: () = assert!(core::mem::offset_of!(NePropertyVtbl, value) == 0x38);

#[repr(C)]
pub struct NeSharedCtrl {
    pub vtable: *mut *mut c_void,
    pub strong: i32,
    pub weak: i32,
}
const _: () = assert!(core::mem::offset_of!(NeSharedCtrl, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(NeSharedCtrl, strong) == 0x8);
const _: () = assert!(core::mem::offset_of!(NeSharedCtrl, weak) == 0xc);

#[repr(C)]
pub struct NeStreamVtbl {
    _pad_0x0: [u8; 0x48],
    pub write: *mut c_void,
    pub write_length: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(NeStreamVtbl, write) == 0x48);
const _: () = assert!(core::mem::offset_of!(NeStreamVtbl, write_length) == 0x50);

#[repr(C)]
pub struct RenownAwardedMessage {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x8],
    pub team: u32,
    pub amount: f32,
}
const _: () = assert!(core::mem::offset_of!(RenownAwardedMessage, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(RenownAwardedMessage, team) == 0x10);
const _: () = assert!(core::mem::offset_of!(RenownAwardedMessage, amount) == 0x14);

#[repr(C)]
pub struct RenownConfig {
    _pad_0x0: [u8; 0x8],
    pub tiers_begin: *mut u32,
    pub tiers_end: *mut u32,
}
const _: () = assert!(core::mem::offset_of!(RenownConfig, tiers_begin) == 0x8);
const _: () = assert!(core::mem::offset_of!(RenownConfig, tiers_end) == 0x10);

#[repr(C)]
pub struct ResourcePointsComponent {
    _pad_0x0: [u8; 0x4a8],
    pub wallet_entity: *mut CareerEntity,
    pub wallet_mode: u8,
    _pad_0x4b1: [u8; 0x7],
    pub driver_wallets: *mut c_void,
    _pad_0x4c0: [u8; 0x68],
    pub team_wallets: *mut c_void,
}
const _: () = assert!(core::mem::offset_of!(ResourcePointsComponent, wallet_entity) == 0x4a8);
const _: () = assert!(core::mem::offset_of!(ResourcePointsComponent, wallet_mode) == 0x4b0);
const _: () = assert!(core::mem::offset_of!(ResourcePointsComponent, driver_wallets) == 0x4b8);
const _: () = assert!(core::mem::offset_of!(ResourcePointsComponent, team_wallets) == 0x528);

#[repr(C)]
pub struct SaveDataContainer {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x78],
    pub name: *const c_char,
    _pad_0x88: [u8; 0x78],
    pub retain_unknown: u8,
    _pad_0x101: [u8; 0x7],
    pub retain_unknown_key: u64,
    _pad_0x110: [u8; 0xb8],
    pub retained_count: u64,
}
const _: () = assert!(core::mem::offset_of!(SaveDataContainer, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(SaveDataContainer, name) == 0x80);
const _: () = assert!(core::mem::offset_of!(SaveDataContainer, retain_unknown) == 0x100);
const _: () = assert!(core::mem::offset_of!(SaveDataContainer, retain_unknown_key) == 0x108);
const _: () = assert!(core::mem::offset_of!(SaveDataContainer, retained_count) == 0x1c8);

#[repr(C)]
pub struct SaveSlotInfo {
    pub slot_id: u64,
    pub slot_guid: u64,
    _pad_0x10: [u8; 0x10],
    pub name: *mut c_char,
    _pad_0x28: [u8; 0x80],
    pub game_setup_mode: u8,
    _pad_0xa9: [u8; 0x3],
    pub info_season: u32,
    pub info_weekend: u32,
    _pad_0xb4: [u8; 0xa],
    pub info_team: u16,
    pub info_driver: u16,
    pub info_vehicle: u16,
    _pad_0xc4: [u8; 0x4],
    pub info_date: u64,
    _pad_0xd0: [u8; 0xa1],
    pub type_flags: u8,
}
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, slot_id) == 0x0);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, slot_guid) == 0x8);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, name) == 0x20);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, game_setup_mode) == 0xa8);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, info_season) == 0xac);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, info_weekend) == 0xb0);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, info_team) == 0xbe);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, info_driver) == 0xc0);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, info_vehicle) == 0xc2);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, info_date) == 0xc8);
const _: () = assert!(core::mem::offset_of!(SaveSlotInfo, type_flags) == 0x171);

#[repr(C)]
pub struct SaveSlotManager {
    _pad_0x0: [u8; 0x268],
    pub slots_begin: *mut *mut SaveSlotInfo,
    pub slots_end: *mut *mut SaveSlotInfo,
    _pad_0x278: [u8; 0x8],
    pub current: *mut SaveSlotInfo,
}
const _: () = assert!(core::mem::offset_of!(SaveSlotManager, slots_begin) == 0x268);
const _: () = assert!(core::mem::offset_of!(SaveSlotManager, slots_end) == 0x270);
const _: () = assert!(core::mem::offset_of!(SaveSlotManager, current) == 0x280);

#[repr(C)]
pub struct ScheduleComponent {
    _pad_0x0: [u8; 0x410],
    pub seasons_begin: *mut ScheduleSeason,
    pub seasons_end: *mut ScheduleSeason,
}
const _: () = assert!(core::mem::offset_of!(ScheduleComponent, seasons_begin) == 0x410);
const _: () = assert!(core::mem::offset_of!(ScheduleComponent, seasons_end) == 0x418);

#[repr(C)]
pub struct ScheduleSeason {
    _pad_0x0: [u8; 0x40],
    pub driver_calendars_begin: *mut DriverCalendar,
    pub driver_calendars_end: *mut DriverCalendar,
    _pad_0x50: [u8; 0x30],
}
const _: () = assert!(core::mem::offset_of!(ScheduleSeason, driver_calendars_begin) == 0x40);
const _: () = assert!(core::mem::offset_of!(ScheduleSeason, driver_calendars_end) == 0x48);
const _: () = assert!(core::mem::size_of::<ScheduleSeason>() == 0x80);

#[repr(C)]
pub struct Season {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x218],
    pub name: *mut c_char,
    _pad_0x228: [u8; 0x58],
    pub allocator: *mut NeAllocator,
    pub weekends_begin: *mut *mut Weekend,
    pub weekends_end: *mut *mut Weekend,
    _pad_0x298: [u8; 0x8],
    pub current_weekend: u32,
    _pad_0x2a4: [u8; 0x4],
    pub standings_vtable: *mut *mut c_void,
    _pad_0x2b0: [u8; 0x288],
    pub completion_state: u32,
    _pad_0x53c: [u8; 0x14],
}
const _: () = assert!(core::mem::offset_of!(Season, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(Season, name) == 0x220);
const _: () = assert!(core::mem::offset_of!(Season, allocator) == 0x280);
const _: () = assert!(core::mem::offset_of!(Season, weekends_begin) == 0x288);
const _: () = assert!(core::mem::offset_of!(Season, weekends_end) == 0x290);
const _: () = assert!(core::mem::offset_of!(Season, current_weekend) == 0x2a0);
const _: () = assert!(core::mem::offset_of!(Season, standings_vtable) == 0x2a8);
const _: () = assert!(core::mem::offset_of!(Season, completion_state) == 0x538);
const _: () = assert!(core::mem::size_of::<Season>() == 0x550);

#[repr(C)]
pub struct SecretMeeting {
    pub id: u32,
    _pad_0x4: [u8; 0x10],
    pub reason: u8,
    _pad_0x15: [u8; 0xb],
    pub team: u32,
}
const _: () = assert!(core::mem::offset_of!(SecretMeeting, id) == 0x0);
const _: () = assert!(core::mem::offset_of!(SecretMeeting, reason) == 0x14);
const _: () = assert!(core::mem::offset_of!(SecretMeeting, team) == 0x20);
const _: () = assert!(core::mem::size_of::<SecretMeeting>() == 0x24);

#[repr(C)]
pub struct Session {
    _pad_0x0: [u8; 0x8],
    pub session_id: u32,
    _pad_0xc: [u8; 0x4],
    pub completion: u32,
    _pad_0x14: [u8; 0x4],
    pub participants_allocator: *mut NeAllocator,
    pub participants_begin: *mut EntrantRecord,
    pub participants_end: *mut EntrantRecord,
    pub participants_capacity: u64,
    _pad_0x38: [u8; 0x68],
}
const _: () = assert!(core::mem::offset_of!(Session, session_id) == 0x8);
const _: () = assert!(core::mem::offset_of!(Session, completion) == 0x10);
const _: () = assert!(core::mem::offset_of!(Session, participants_allocator) == 0x18);
const _: () = assert!(core::mem::offset_of!(Session, participants_begin) == 0x20);
const _: () = assert!(core::mem::offset_of!(Session, participants_end) == 0x28);
const _: () = assert!(core::mem::offset_of!(Session, participants_capacity) == 0x30);
const _: () = assert!(core::mem::size_of::<Session>() == 0xa0);

#[repr(C)]
pub struct SpectatorDirector {
    _pad_0x0: [u8; 0xd0],
    pub ui_active: u8,
    pub active: u8,
    _pad_0xd2: [u8; 0x1],
    pub should_quit: u8,
    _pad_0xd4: [u8; 0x2],
    pub current_driver: u8,
    _pad_0xd7: [u8; 0x1],
    pub current_camera: i32,
    pub requested_driver: u8,
    _pad_0xdd: [u8; 0x3],
    pub requested_camera: i32,
}
const _: () = assert!(core::mem::offset_of!(SpectatorDirector, ui_active) == 0xd0);
const _: () = assert!(core::mem::offset_of!(SpectatorDirector, active) == 0xd1);
const _: () = assert!(core::mem::offset_of!(SpectatorDirector, should_quit) == 0xd3);
const _: () = assert!(core::mem::offset_of!(SpectatorDirector, current_driver) == 0xd6);
const _: () = assert!(core::mem::offset_of!(SpectatorDirector, current_camera) == 0xd8);
const _: () = assert!(core::mem::offset_of!(SpectatorDirector, requested_driver) == 0xdc);
const _: () = assert!(core::mem::offset_of!(SpectatorDirector, requested_camera) == 0xe0);

#[repr(C)]
pub struct SponsorEntry {
    _pad_0x0: [u8; 0x10],
    pub name: *const c_char,
    _pad_0x18: [u8; 0x10],
    pub stat_id: u16,
    _pad_0x2a: [u8; 0x6e],
    pub levels_begin: *mut SponsorLoyaltyLevel,
    pub levels_end: *mut SponsorLoyaltyLevel,
    _pad_0xa8: [u8; 0x30],
    pub slot: u8,
    _pad_0xd9: [u8; 0x3b],
    pub loyalty_xp: i32,
    _pad_0x118: [u8; 0x40],
    pub active: u8,
    _pad_0x159: [u8; 0x7],
}
const _: () = assert!(core::mem::offset_of!(SponsorEntry, name) == 0x10);
const _: () = assert!(core::mem::offset_of!(SponsorEntry, stat_id) == 0x28);
const _: () = assert!(core::mem::offset_of!(SponsorEntry, levels_begin) == 0x98);
const _: () = assert!(core::mem::offset_of!(SponsorEntry, levels_end) == 0xa0);
const _: () = assert!(core::mem::offset_of!(SponsorEntry, slot) == 0xd8);
const _: () = assert!(core::mem::offset_of!(SponsorEntry, loyalty_xp) == 0x114);
const _: () = assert!(core::mem::offset_of!(SponsorEntry, active) == 0x158);
const _: () = assert!(core::mem::size_of::<SponsorEntry>() == 0x160);

#[repr(C)]
pub struct SponsorLoyaltyLevel {
    pub claimed: u8,
    _pad_0x1: [u8; 0x3],
    pub xp_threshold: i32,
    _pad_0x8: [u8; 0x70],
}
const _: () = assert!(core::mem::offset_of!(SponsorLoyaltyLevel, claimed) == 0x0);
const _: () = assert!(core::mem::offset_of!(SponsorLoyaltyLevel, xp_threshold) == 0x4);
const _: () = assert!(core::mem::size_of::<SponsorLoyaltyLevel>() == 0x78);

#[repr(C)]
pub struct SponsorLoyaltyXpMessage {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x8],
    pub team: u32,
    pub amount: i32,
    pub reason: u8,
}
const _: () = assert!(core::mem::offset_of!(SponsorLoyaltyXpMessage, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(SponsorLoyaltyXpMessage, team) == 0x10);
const _: () = assert!(core::mem::offset_of!(SponsorLoyaltyXpMessage, amount) == 0x14);
const _: () = assert!(core::mem::offset_of!(SponsorLoyaltyXpMessage, reason) == 0x18);

#[repr(C)]
pub struct SponsorTeamRecord {
    _pad_0x0: [u8; 0x8],
    pub team: u32,
    _pad_0xc: [u8; 0xc],
    pub sponsors_begin: *mut SponsorEntry,
    pub sponsors_end: *mut SponsorEntry,
    _pad_0x28: [u8; 0x60],
}
const _: () = assert!(core::mem::offset_of!(SponsorTeamRecord, team) == 0x8);
const _: () = assert!(core::mem::offset_of!(SponsorTeamRecord, sponsors_begin) == 0x18);
const _: () = assert!(core::mem::offset_of!(SponsorTeamRecord, sponsors_end) == 0x20);
const _: () = assert!(core::mem::size_of::<SponsorTeamRecord>() == 0x88);

#[repr(C)]
pub struct StandingsPoints {
    pub key: u32,
    pub points: i32,
}
const _: () = assert!(core::mem::offset_of!(StandingsPoints, key) == 0x0);
const _: () = assert!(core::mem::offset_of!(StandingsPoints, points) == 0x4);
const _: () = assert!(core::mem::size_of::<StandingsPoints>() == 0x8);

#[repr(C)]
pub struct StandingsResult {
    pub key: u32,
    pub position: i32,
    pub fastest_lap_rank: i32,
    pub status: u32,
}
const _: () = assert!(core::mem::offset_of!(StandingsResult, key) == 0x0);
const _: () = assert!(core::mem::offset_of!(StandingsResult, position) == 0x4);
const _: () = assert!(core::mem::offset_of!(StandingsResult, fastest_lap_rank) == 0x8);
const _: () = assert!(core::mem::offset_of!(StandingsResult, status) == 0xc);
const _: () = assert!(core::mem::size_of::<StandingsResult>() == 0x10);

#[repr(C)]
pub struct StandingsRound {
    _pad_0x0: [u8; 0x8],
    pub sessions_begin: *mut StandingsSession,
    pub sessions_end: *mut StandingsSession,
    _pad_0x18: [u8; 0x8],
}
const _: () = assert!(core::mem::offset_of!(StandingsRound, sessions_begin) == 0x8);
const _: () = assert!(core::mem::offset_of!(StandingsRound, sessions_end) == 0x10);
const _: () = assert!(core::mem::size_of::<StandingsRound>() == 0x20);

#[repr(C)]
pub struct StandingsSession {
    _pad_0x0: [u8; 0x8],
    pub records_begin: *mut c_void,
    pub records_end: *mut c_void,
    _pad_0x18: [u8; 0x8],
    pub session_id: u32,
    _pad_0x24: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(StandingsSession, records_begin) == 0x8);
const _: () = assert!(core::mem::offset_of!(StandingsSession, records_end) == 0x10);
const _: () = assert!(core::mem::offset_of!(StandingsSession, session_id) == 0x20);
const _: () = assert!(core::mem::size_of::<StandingsSession>() == 0x28);

#[repr(C)]
pub struct StatInfo {
    _pad_0x0: [u8; 0x8],
    pub stat_id: u16,
    _pad_0xa: [u8; 0x126],
    pub weight: f32,
    pub default_value: f32,
}
const _: () = assert!(core::mem::offset_of!(StatInfo, stat_id) == 0x8);
const _: () = assert!(core::mem::offset_of!(StatInfo, weight) == 0x130);
const _: () = assert!(core::mem::offset_of!(StatInfo, default_value) == 0x134);

#[repr(C)]
pub struct TeamRecord {
    _pad_0x0: [u8; 0xb0],
    pub display_name: *mut c_char,
    _pad_0xb8: [u8; 0x90],
    pub display_name_code: *mut c_char,
    _pad_0x150: [u8; 0x90],
    pub display_name_short: *mut c_char,
    _pad_0x1e8: [u8; 0x90],
    pub display_name_super_short: *mut c_char,
    _pad_0x280: [u8; 0x90],
    pub livery_name: *mut c_char,
    _pad_0x318: [u8; 0x90],
    pub team_base: *mut c_char,
    _pad_0x3b0: [u8; 0x128],
    pub timing_tower_texture_path: *mut c_char,
    _pad_0x4e0: [u8; 0x84],
    pub primary_colour: i32,
    pub secondary_colour: i32,
    pub tertiary_colour: i32,
    pub team_wear_primary_colour: i32,
    pub team_wear_secondary_colour: i32,
    pub team_wear_tertiary_colour: i32,
    pub ui_team_colour: i32,
    _pad_0x580: [u8; 0x1e8],
}
const _: () = assert!(core::mem::offset_of!(TeamRecord, display_name) == 0xb0);
const _: () = assert!(core::mem::offset_of!(TeamRecord, display_name_code) == 0x148);
const _: () = assert!(core::mem::offset_of!(TeamRecord, display_name_short) == 0x1e0);
const _: () = assert!(core::mem::offset_of!(TeamRecord, display_name_super_short) == 0x278);
const _: () = assert!(core::mem::offset_of!(TeamRecord, livery_name) == 0x310);
const _: () = assert!(core::mem::offset_of!(TeamRecord, team_base) == 0x3a8);
const _: () = assert!(core::mem::offset_of!(TeamRecord, timing_tower_texture_path) == 0x4d8);
const _: () = assert!(core::mem::offset_of!(TeamRecord, primary_colour) == 0x564);
const _: () = assert!(core::mem::offset_of!(TeamRecord, secondary_colour) == 0x568);
const _: () = assert!(core::mem::offset_of!(TeamRecord, tertiary_colour) == 0x56c);
const _: () = assert!(core::mem::offset_of!(TeamRecord, team_wear_primary_colour) == 0x570);
const _: () = assert!(core::mem::offset_of!(TeamRecord, team_wear_secondary_colour) == 0x574);
const _: () = assert!(core::mem::offset_of!(TeamRecord, team_wear_tertiary_colour) == 0x578);
const _: () = assert!(core::mem::offset_of!(TeamRecord, ui_team_colour) == 0x57c);
const _: () = assert!(core::mem::size_of::<TeamRecord>() == 0x768);

#[repr(C)]
pub struct TeamRenownData {
    pub team: u32,
    pub points: f32,
    _pad_0x8: [u8; 0x10],
    pub config: *mut RenownConfig,
    _pad_0x20: [u8; 0x120],
}
const _: () = assert!(core::mem::offset_of!(TeamRenownData, team) == 0x0);
const _: () = assert!(core::mem::offset_of!(TeamRenownData, points) == 0x4);
const _: () = assert!(core::mem::offset_of!(TeamRenownData, config) == 0x18);
const _: () = assert!(core::mem::size_of::<TeamRenownData>() == 0x140);

#[repr(C)]
pub struct Tile {
    _pad_0x0: [u8; 0xd0],
    pub item: *mut HomeItem,
}
const _: () = assert!(core::mem::offset_of!(Tile, item) == 0xd0);

#[repr(C)]
pub struct TileEntry {
    _pad_0x0: [u8; 0x8],
    pub tile: *mut Tile,
}
const _: () = assert!(core::mem::offset_of!(TileEntry, tile) == 0x8);

#[repr(C)]
pub struct TileList {
    _pad_0x0: [u8; 0x78],
    pub items_begin: *mut TileEntry,
    pub items_end: *mut TileEntry,
}
const _: () = assert!(core::mem::offset_of!(TileList, items_begin) == 0x78);
const _: () = assert!(core::mem::offset_of!(TileList, items_end) == 0x80);

#[repr(C)]
pub struct TimeOfDayClock {
    _pad_0x0: [u8; 0x68],
    pub time_a: f64,
    pub time_b: f64,
    _pad_0x78: [u8; 0x8],
    pub time_scale: f32,
    pub length: f32,
    _pad_0x88: [u8; 0x200],
}
const _: () = assert!(core::mem::offset_of!(TimeOfDayClock, time_a) == 0x68);
const _: () = assert!(core::mem::offset_of!(TimeOfDayClock, time_b) == 0x70);
const _: () = assert!(core::mem::offset_of!(TimeOfDayClock, time_scale) == 0x80);
const _: () = assert!(core::mem::offset_of!(TimeOfDayClock, length) == 0x84);
const _: () = assert!(core::mem::size_of::<TimeOfDayClock>() == 0x288);

#[repr(C)]
pub struct TimeOfDayOwner {
    _pad_0x0: [u8; 0x2d0],
    pub tod: *mut TimeOfDaySystem,
}
const _: () = assert!(core::mem::offset_of!(TimeOfDayOwner, tod) == 0x2d0);

#[repr(C)]
pub struct WeatherState {
    pub r#type: u32,
    pub intensity: f32,
    pub end: f32,
    pub transition: f32,
}
const _: () = assert!(core::mem::offset_of!(WeatherState, r#type) == 0x0);
const _: () = assert!(core::mem::offset_of!(WeatherState, intensity) == 0x4);
const _: () = assert!(core::mem::offset_of!(WeatherState, end) == 0x8);
const _: () = assert!(core::mem::offset_of!(WeatherState, transition) == 0xc);
const _: () = assert!(core::mem::size_of::<WeatherState>() == 0x10);

#[repr(C)]
pub struct WeatherDataset {
    _pad_0x0: [u8; 0xb90],
    pub states: [WeatherState; 38],
    pub state_total: u32,
    _pad_0xdf4: [u8; 0x4],
    pub state_count: u32,
    _pad_0xdfc: [u8; 0x4],
}
const _: () = assert!(core::mem::offset_of!(WeatherDataset, states) == 0xb90);
const _: () = assert!(core::mem::offset_of!(WeatherDataset, state_total) == 0xdf0);
const _: () = assert!(core::mem::offset_of!(WeatherDataset, state_count) == 0xdf8);
const _: () = assert!(core::mem::size_of::<WeatherDataset>() == 0xe00);

#[repr(C)]
pub struct WeatherTypeRange {
    pub weight: f32,
    _pad_0x4: [u8; 0x8],
    pub intensity_min: f32,
    pub intensity_max: f32,
}
const _: () = assert!(core::mem::offset_of!(WeatherTypeRange, weight) == 0x0);
const _: () = assert!(core::mem::offset_of!(WeatherTypeRange, intensity_min) == 0xc);
const _: () = assert!(core::mem::offset_of!(WeatherTypeRange, intensity_max) == 0x10);
const _: () = assert!(core::mem::size_of::<WeatherTypeRange>() == 0x14);

#[repr(C)]
pub struct TimeOfDaySystem {
    _pad_0x0: [u8; 0x2c8],
    pub clocks: [TimeOfDayClock; 10],
    _pad_0x1c18: [u8; 0xa3c],
    pub scripted_scenario: u8,
    _pad_0x2655: [u8; 0x6f],
    pub active_dataset: u8,
    _pad_0x26c5: [u8; 0x13],
    pub datasets: [WeatherDataset; 10],
    _pad_0xb2d8: [u8; 0x64],
    pub seed: u32,
    _pad_0xb340: [u8; 0x10],
    pub dataset_length: [f32; 10],
    pub type_ranges: [WeatherTypeRange; 6],
    _pad_0xb3f0: [u8; 0x36],
    pub exact_timing: u8,
    _pad_0xb427: [u8; 0x2],
    pub ready_a: u8,
    pub ready_b: u8,
    _pad_0xb42b: [u8; 0x2],
    pub random_override: u8,
    _pad_0xb42e: [u8; 0x4],
    pub busy: u8,
    _pad_0xb433: [u8; 0x3],
    pub override_active: u8,
}
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, clocks) == 0x2c8);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, scripted_scenario) == 0x2654);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, active_dataset) == 0x26c4);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, datasets) == 0x26d8);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, seed) == 0xb33c);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, dataset_length) == 0xb350);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, type_ranges) == 0xb378);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, exact_timing) == 0xb426);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, ready_a) == 0xb429);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, ready_b) == 0xb42a);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, random_override) == 0xb42d);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, busy) == 0xb432);
const _: () = assert!(core::mem::offset_of!(TimeOfDaySystem, override_active) == 0xb436);

#[repr(C)]
pub struct TrackDataRecord {
    _pad_0x0: [u8; 0x1],
    pub index: u8,
    _pad_0x2: [u8; 0xce],
    pub name: *mut c_char,
}
const _: () = assert!(core::mem::offset_of!(TrackDataRecord, index) == 0x1);
const _: () = assert!(core::mem::offset_of!(TrackDataRecord, name) == 0xd0);

#[repr(C)]
pub struct TrackDataSet {
    _pad_0x0: [u8; 0x8],
    pub tracks_begin: *mut c_void,
    pub tracks_end: *mut c_void,
    _pad_0x18: [u8; 0x8],
    pub mode_mask: u32,
    pub season_mask: u32,
    pub format_mask: u32,
}
const _: () = assert!(core::mem::offset_of!(TrackDataSet, tracks_begin) == 0x8);
const _: () = assert!(core::mem::offset_of!(TrackDataSet, tracks_end) == 0x10);
const _: () = assert!(core::mem::offset_of!(TrackDataSet, mode_mask) == 0x20);
const _: () = assert!(core::mem::offset_of!(TrackDataSet, season_mask) == 0x24);
const _: () = assert!(core::mem::offset_of!(TrackDataSet, format_mask) == 0x28);

#[repr(C)]
pub struct TrackDataSystem {
    _pad_0x0: [u8; 0x68],
    pub sets_begin: *mut TrackDataSet,
    pub sets_end: *mut TrackDataSet,
}
const _: () = assert!(core::mem::offset_of!(TrackDataSystem, sets_begin) == 0x68);
const _: () = assert!(core::mem::offset_of!(TrackDataSystem, sets_end) == 0x70);

#[repr(C)]
pub struct TrackDataTrack {
    pub record: *mut TrackDataRecord,
    _pad_0x8: [u8; 0xe8],
    pub fuel_burn_per_lap: f32,
    _pad_0xf4: [u8; 0xc],
    pub race_fuel: [u8; 156],
    _pad_0x19c: [u8; 0x4],
    pub race_fuel_count: u64,
    _pad_0x1a8: [u8; 0xc],
    pub qualifying_one_shot: f32,
    _pad_0x1b8: [u8; 0x4],
    pub qualifying_short: f32,
    pub qualifying_full: f32,
}
const _: () = assert!(core::mem::offset_of!(TrackDataTrack, record) == 0x0);
const _: () = assert!(core::mem::offset_of!(TrackDataTrack, fuel_burn_per_lap) == 0xf0);
const _: () = assert!(core::mem::offset_of!(TrackDataTrack, race_fuel) == 0x100);
const _: () = assert!(core::mem::offset_of!(TrackDataTrack, race_fuel_count) == 0x1a0);
const _: () = assert!(core::mem::offset_of!(TrackDataTrack, qualifying_one_shot) == 0x1b4);
const _: () = assert!(core::mem::offset_of!(TrackDataTrack, qualifying_short) == 0x1bc);
const _: () = assert!(core::mem::offset_of!(TrackDataTrack, qualifying_full) == 0x1c0);

#[repr(C)]
pub struct UpgradeInstance {
    _pad_0x0: [u8; 0x10],
    pub name: *mut c_char,
    _pad_0x18: [u8; 0x40],
    pub node_hash: u32,
    pub cost: i32,
    pub cost_to_develop: i32,
    _pad_0x64: [u8; 0x4],
    pub num_days_to_research: i32,
    pub num_days_to_develop: i32,
    _pad_0x70: [u8; 0x2],
    pub department: u8,
    _pad_0x73: [u8; 0x1],
    pub unlock_at_facility_level: i32,
    _pad_0x78: [u8; 0x4],
    pub row_flag: u8,
    _pad_0x7d: [u8; 0x1],
    pub research_start_day: u16,
    pub research_end_day: u16,
    _pad_0x82: [u8; 0x16],
    pub develop_start_day: u16,
    pub develop_end_day: u16,
    _pad_0x9c: [u8; 0x24],
    pub states_begin: *mut UpgradeStateEntry,
    pub states_end: *mut UpgradeStateEntry,
    _pad_0xd0: [u8; 0x10],
}
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, name) == 0x10);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, node_hash) == 0x58);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, cost) == 0x5c);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, cost_to_develop) == 0x60);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, num_days_to_research) == 0x68);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, num_days_to_develop) == 0x6c);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, department) == 0x72);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, unlock_at_facility_level) == 0x74);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, row_flag) == 0x7c);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, research_start_day) == 0x7e);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, research_end_day) == 0x80);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, develop_start_day) == 0x98);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, develop_end_day) == 0x9a);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, states_begin) == 0xc0);
const _: () = assert!(core::mem::offset_of!(UpgradeInstance, states_end) == 0xc8);
const _: () = assert!(core::mem::size_of::<UpgradeInstance>() == 0xe0);

#[repr(C)]
pub struct UpgradeStateEntry {
    pub owner: u32,
    pub day: u16,
    _pad_0x6: [u8; 0x2],
    pub state: u8,
    pub adaptation: u8,
    _pad_0xa: [u8; 0x2],
    pub enabled: u8,
    pub facility_locked: u8,
    _pad_0xe: [u8; 0x2],
}
const _: () = assert!(core::mem::offset_of!(UpgradeStateEntry, owner) == 0x0);
const _: () = assert!(core::mem::offset_of!(UpgradeStateEntry, day) == 0x4);
const _: () = assert!(core::mem::offset_of!(UpgradeStateEntry, state) == 0x8);
const _: () = assert!(core::mem::offset_of!(UpgradeStateEntry, adaptation) == 0x9);
const _: () = assert!(core::mem::offset_of!(UpgradeStateEntry, enabled) == 0xc);
const _: () = assert!(core::mem::offset_of!(UpgradeStateEntry, facility_locked) == 0xd);
const _: () = assert!(core::mem::size_of::<UpgradeStateEntry>() == 0x10);

#[repr(C)]
pub struct UpgradeTree {
    _pad_0x0: [u8; 0x10],
    pub instances_begin: *mut UpgradeInstance,
    pub instances_end: *mut UpgradeInstance,
    _pad_0x20: [u8; 0xc20],
}
const _: () = assert!(core::mem::offset_of!(UpgradeTree, instances_begin) == 0x10);
const _: () = assert!(core::mem::offset_of!(UpgradeTree, instances_end) == 0x18);
const _: () = assert!(core::mem::size_of::<UpgradeTree>() == 0xc40);

#[repr(C)]
pub struct Upgrades {
    _pad_0x0: [u8; 0x8],
    pub mode: u8,
    _pad_0x9: [u8; 0x3],
    pub team_key: u32,
    _pad_0x10: [u8; 0xd0],
    pub trees: [UpgradeTree; 2],
    pub tree_count: u64,
    _pad_0x1968: [u8; 0x140],
    pub owner_cache: u32,
    _pad_0x1aac: [u8; 0x64],
}
const _: () = assert!(core::mem::offset_of!(Upgrades, mode) == 0x8);
const _: () = assert!(core::mem::offset_of!(Upgrades, team_key) == 0xc);
const _: () = assert!(core::mem::offset_of!(Upgrades, trees) == 0xe0);
const _: () = assert!(core::mem::offset_of!(Upgrades, tree_count) == 0x1960);
const _: () = assert!(core::mem::offset_of!(Upgrades, owner_cache) == 0x1aa8);
const _: () = assert!(core::mem::size_of::<Upgrades>() == 0x1b10);

#[repr(C)]
pub struct UpgradesComponent {
    _pad_0x0: [u8; 0x298],
    pub upgrades_begin: *mut *mut Upgrades,
    pub upgrades_end: *mut *mut Upgrades,
    _pad_0x2a8: [u8; 0x18],
    pub career_time: *mut CareerTimeComponent,
}
const _: () = assert!(core::mem::offset_of!(UpgradesComponent, upgrades_begin) == 0x298);
const _: () = assert!(core::mem::offset_of!(UpgradesComponent, upgrades_end) == 0x2a0);
const _: () = assert!(core::mem::offset_of!(UpgradesComponent, career_time) == 0x2c0);

#[repr(C)]
pub struct WeatherAndTimeOfDayComponent {
    _pad_0x0: [u8; 0x2a0],
    pub weather_system: *mut GameWeatherSystem,
    _pad_0x2a8: [u8; 0x1c],
    pub random_seed: u32,
}
const _: () = assert!(core::mem::offset_of!(WeatherAndTimeOfDayComponent, weather_system) == 0x2a0);
const _: () = assert!(core::mem::offset_of!(WeatherAndTimeOfDayComponent, random_seed) == 0x2c4);

#[repr(C)]
pub struct WeatherSessionSettings {
    _pad_0x0: [u8; 0x4],
    pub session_type: u32,
    _pad_0x8: [u8; 0xb0c],
    pub mode: u32,
    pub exact_timing: u8,
}
const _: () = assert!(core::mem::offset_of!(WeatherSessionSettings, session_type) == 0x4);
const _: () = assert!(core::mem::offset_of!(WeatherSessionSettings, mode) == 0xb14);
const _: () = assert!(core::mem::offset_of!(WeatherSessionSettings, exact_timing) == 0xb18);

#[repr(C)]
pub struct Weekend {
    pub vtable: *mut *mut c_void,
    _pad_0x8: [u8; 0x20],
    pub name: *mut c_char,
    _pad_0x30: [u8; 0x1f0],
    pub sessions_begin: *mut Session,
    pub sessions_end: *mut Session,
    _pad_0x230: [u8; 0x8],
    pub current_session: u32,
    pub progress_state: u32,
    _pad_0x240: [u8; 0x38],
    pub track_stat_id: u8,
    _pad_0x279: [u8; 0x277],
}
const _: () = assert!(core::mem::offset_of!(Weekend, vtable) == 0x0);
const _: () = assert!(core::mem::offset_of!(Weekend, name) == 0x28);
const _: () = assert!(core::mem::offset_of!(Weekend, sessions_begin) == 0x220);
const _: () = assert!(core::mem::offset_of!(Weekend, sessions_end) == 0x228);
const _: () = assert!(core::mem::offset_of!(Weekend, current_session) == 0x238);
const _: () = assert!(core::mem::offset_of!(Weekend, progress_state) == 0x23c);
const _: () = assert!(core::mem::offset_of!(Weekend, track_stat_id) == 0x278);
const _: () = assert!(core::mem::size_of::<Weekend>() == 0x4f0);

pub type Car_Create_DetachablePart_t = unsafe extern "C" fn(
    part_manager: *mut c_void,
    cb: *mut *mut c_void,
    name_id: *mut u32,
    params: *mut i32,
    k: f32,
) -> *mut c_void;
pub type Car_DetachablePart_Update_t = unsafe extern "C" fn(part: *mut c_void, dt: f32) -> bool;
pub type Car_FrontWing_Init_t = unsafe extern "C" fn(front_wing: *mut c_void, reset: c_char);
pub type Car_Register_DetachablePart_Thunk_t = unsafe extern "C" fn(
    front_wing: *mut c_void,
    cb: *mut *mut c_void,
    name_id: *mut u32,
    params: *mut c_void,
    k: f32,
);
pub type Car_Reset_LastFittedUpgrade_t = unsafe extern "C" fn(inst: *mut c_void);
pub type Car_Tyre_SetWearFactor_t =
    unsafe extern "C" fn(tyre_component: *mut c_void, apply_set: bool, force: u16);
pub type Career_Add_SeasonWeekend_t = unsafe extern "C" fn(season: *mut c_void, weekend: *mut c_void) -> u64;
pub type Career_Add_TeamModifier_t = unsafe extern "C" fn(list: *mut c_void, key: u32, value: f32);
pub type Career_Adjust_RpDriverWallet_t =
    unsafe extern "C" fn(wallets: *mut c_void, driver_key: u32, delta: i32);
pub type Career_Adjust_RpTeamWallet_t = unsafe extern "C" fn(wallets: *mut c_void, team_key: u32, delta: i32);
pub type Career_Build_DriverContract_t = unsafe extern "C" fn(
    contracts: *mut c_void,
    contract: *mut c_void,
    driver_key: u32,
    team_key: u32,
    r#type: u8,
    target: i32,
    seasons: i32,
) -> *mut c_void;
pub type Career_Build_LineupChanges_t = unsafe extern "C" fn(
    out: *mut c_void,
    current_entrants: *const c_void,
    baseline_entrants: *const c_void,
) -> *mut c_void;
pub type Career_Calc_UpgradeCost_t =
    unsafe extern "C" fn(list: *mut c_void, inst: *mut c_void, mods: *mut c_void, option: u32) -> i32;
pub type Career_Cancel_UpgradeDevelopment_t =
    unsafe extern "C" fn(upgrades: *mut c_void, owner: u32, inst: *mut c_void, day: u32);
pub type Career_Cancel_UpgradeResearch_t =
    unsafe extern "C" fn(upgrades: *mut c_void, owner: u32, inst: *mut c_void, day: u32);
pub type Career_Complete_PerkTreeNode_t = unsafe extern "C" fn(node: *mut c_void) -> u8;
pub type Career_Construct_RnDDeptCard_t = unsafe extern "C" fn(
    card: *mut c_void,
    ctx: *mut c_void,
    builder: *mut c_void,
    allocator: *mut c_void,
    name: *const c_char,
    icon: *const c_char,
    dept: u8,
) -> *mut c_void;
pub type Career_ContractIntro_OnEnter_t = unsafe extern "C" fn(state: *mut c_void) -> u64;
pub type Career_ContractIntro_OnUpdate_t =
    unsafe extern "C" fn(state: *mut c_void, link: *mut u32) -> *mut u32;
pub type Career_ContractOutro_OnEnter_t = unsafe extern "C" fn(state: *mut c_void) -> u64;
pub type Career_ContractOutro_OnUpdate_t =
    unsafe extern "C" fn(state: *mut c_void, link: *mut u32) -> *mut u32;
pub type Career_ContractUpdate_OnEnter_t = unsafe extern "C" fn(state: *mut c_void) -> u64;
pub type Career_ContractUpdate_OnExit_t = unsafe extern "C" fn(state: *mut c_void) -> u64;
pub type Career_ContractUpdate_OnUpdate_t =
    unsafe extern "C" fn(state: *mut c_void, link: *mut u32) -> *mut u32;
pub type Career_Create_SeasonWeekend_t =
    unsafe extern "C" fn(season: *mut c_void, index: u64, track: u8, name: *const c_char) -> *mut c_void;
pub type Career_Decide_ContractNegotiation_t =
    unsafe extern "C" fn(state: *mut c_void, link: *mut u32) -> *mut u32;
pub type Career_DriverNumbers_CheckExisting_t = unsafe extern "C" fn(car_numbers: *mut c_void, driver: u32);
pub type Career_Eval_Curve_t = unsafe extern "C" fn(curve: *mut c_void, x: f32) -> f32;
pub type Career_Get_CareerLengthDays_Thunk_t = unsafe extern "C" fn(time: *mut c_void) -> u32;
pub type Career_Get_DriverOverallRating_t =
    unsafe extern "C" fn(self_: *mut c_void, data: *mut c_void, driver: *const u32, round: bool) -> f32;
pub type Career_Get_RpWallet_t = unsafe extern "C" fn(wallets: *mut c_void, out: *mut u64, driver_key: u32);
pub type Career_Get_TeamRenownData_t =
    unsafe extern "C" fn(rf: *mut c_void, out_weak: *mut c_void, team_key: u32) -> *mut c_void;
pub type Career_Needs_DriverContract_t = unsafe extern "C" fn(contracts: *mut c_void) -> bool;
pub type Career_Post_UpgradeFittedRatio_t =
    unsafe extern "C" fn(upgrades: *mut c_void, inst: *mut c_void, fitted: u8);
pub type Career_Rebuild_ChampionshipStandings_t =
    unsafe extern "C" fn(champ_list: *mut c_void, extra_points: *mut c_void, extra_e: *mut c_void);
pub type Career_Rebuild_SeasonStandings_t = unsafe extern "C" fn(
    season: *mut c_void,
    order_a: *mut c_void,
    order_b: *mut c_void,
    extra_points: *mut c_void,
    extra_e: *mut c_void,
);
pub type Career_Record_DriverMove_t =
    unsafe extern "C" fn(renewal: *mut c_void, driver: u32, old_team: u32, new_team: u32, reason: u8);
pub type Career_Refresh_RnDDeptMode_t = unsafe extern "C" fn(holder: *mut usize, mode: u32);
pub type Career_Refresh_UpgradeAvailability_t = unsafe extern "C" fn(upgrades: *mut c_void, day: u16);
pub type Career_Set_DriverContractState_t =
    unsafe extern "C" fn(contract: *mut c_void, day: u16, state: u8, value: u32);
pub type Career_Set_EntrantData_Thunk_t = unsafe extern "C" fn(
    entity: *mut c_void,
    entrant_id: *const c_void,
    data: *const u32,
    position: u16,
    flag: u8,
);
pub type Career_Set_UpgradeFacilityLocked_t = unsafe extern "C" fn(inst: *mut c_void, locked: u8, day: u16);
pub type Career_Set_UpgradesOwner_t = unsafe extern "C" fn(upgrades: *mut c_void, owner: u32);
pub type Career_Sign_DriverContract_t = unsafe extern "C" fn(
    contracts: *mut c_void,
    contract: *mut c_void,
    driver_key: u64,
    replaced_key: u32,
    notify: u8,
);
pub type Career_Start_UpgradeDevelopment_t = unsafe extern "C" fn(
    upgrades: *mut c_void,
    owner: u32,
    inst: *mut c_void,
    start_day: u16,
    end_day: u16,
    option: u8,
    dual: u8,
    roll_failure: u8,
    rng: *mut c_void,
    store_duration: u8,
);
pub type Career_Start_UpgradeResearch_t = unsafe extern "C" fn(
    upgrades: *mut c_void,
    owner: u32,
    inst: *mut c_void,
    start_day: u32,
    end_day: u16,
    option: u8,
    roll_failure: u8,
    rng: *mut c_void,
    store_duration: u8,
);
pub type Career_Teardown_Weekend_t = unsafe extern "C" fn(weekend: *mut c_void);
pub type Ent_Deserialise_EntrantVector_t =
    unsafe extern "C" fn(vector_object: *mut c_void, bag: *mut c_void) -> c_int;
pub type FeHome_Add_CareerTile_t = unsafe extern "C" fn(list: *mut TileList, index: u8);
pub type FeHome_Handle_Command_t =
    unsafe extern "C" fn(vm: *mut FEHomeViewModel, cmd: *mut *mut NeStr) -> u64;
pub type FeTile_Add_EraTile_t = unsafe extern "C" fn(
    tiles: *mut c_void,
    alloc: *mut c_void,
    index: u8,
    node: *const c_char,
    name: *const c_char,
    enabled: u8,
    title: *const c_char,
    command: *const c_char,
    image: *const c_char,
    is_dlc: u8,
);
pub type FeTile_Build_List_t = unsafe extern "C" fn(
    self_: *mut TileList,
    ctx: *mut c_void,
    a3: *mut c_void,
    a4: *mut c_void,
    menu_kind: u8,
    sub_kind: u8,
    a7: u32,
);
pub type FeTile_CareerDlcItem_Populate_t =
    unsafe extern "C" fn(item: *mut HomeItem, node: *mut c_void, a7: u32);
pub type FeTile_Check_EntrantListUsable_t =
    unsafe extern "C" fn(tile_ctx: *mut c_void, label: *const c_char) -> u64;
pub type FeTile_Clear_EraTileSet_Thunk_t = unsafe extern "C" fn(tiles: *mut c_void);
pub type FeTile_Construct_CareerItem_t =
    unsafe extern "C" fn(item: *mut HomeItem, alloc: *mut NeAllocator, id: *const c_char) -> *mut HomeItem;
pub type FeTile_Get_ListFirstEntrantId_t =
    unsafe extern "C" fn(item: *mut c_void, out: *mut c_void) -> *mut c_void;
pub type FeTile_Is_EraScenarioAvailable_Thunk_t =
    unsafe extern "C" fn(mode: *mut c_void, setup_mode: *const c_char, entrant_list: *const c_char) -> u8;
pub type FeTile_ItemBase_Populate_t = unsafe extern "C" fn(item: *mut HomeItem, node: *mut c_void, a7: u32);
pub type FeTile_Press_EraTile_t =
    unsafe extern "C" fn(view_model: *mut c_void, command: *mut c_void) -> *mut c_char;
pub type FeTile_Route_CareerItem_t =
    unsafe extern "C" fn(item: *mut HomeItem, out: *mut Action) -> *mut Action;
pub type Game_CareerActivities_AddActivity_t =
    unsafe extern "C" fn(calendar: *mut c_void, activity: *const c_void, day: u32);
pub type Game_CareerActivities_RemoveActivity_t =
    unsafe extern "C" fn(calendar: *mut c_void, activity: *const c_void, day: u16);
pub type Game_CareerContinue_HandleCommand_t =
    unsafe extern "C" fn(vm: *mut c_void, cmd: *mut *mut NeStr, value: *mut c_void) -> u64;
pub type Game_CareerSetup_ResetEntrants_t = unsafe extern "C" fn(setup: *mut c_void);
pub type Game_CareerWizard_PushNextStep_t = unsafe extern "C" fn(wizard: *mut c_void);
pub type Game_CareerWizard_StartsInF1_t = unsafe extern "C" fn(
    entrants: *mut c_void,
    wizard_data: *mut c_void,
    entrant_list: *const c_char,
) -> bool;
pub type Game_Career_IsDriverCareerMode_t = unsafe extern "C" fn(championship: *const c_void) -> bool;
pub type Game_ContractTeamSelect_OnEnter_t = unsafe extern "C" fn(state: *mut c_void) -> u64;
pub type Game_ContractTeamSelect_OnExit_t = unsafe extern "C" fn(state: *mut c_void) -> u64;
pub type Game_ContractTeamSelect_OnUpdate_t =
    unsafe extern "C" fn(state: *mut c_void, link: *mut u32) -> *mut u32;
pub type Game_CreateSessionDlcSelectTab_BuildTiles_t = unsafe extern "C" fn(view_model: *mut c_void);
pub type Game_CreateSessionDlcSelectTab_FillTile_t = unsafe extern "C" fn(
    tile: *mut c_void,
    node: *mut c_void,
    title: *const c_char,
    command: *const c_char,
    image: *const c_char,
) -> *mut c_void;
pub type Game_CreateSessionDlcSelectTab_HandleCommand_t =
    unsafe extern "C" fn(view_model: *mut c_void, command: *mut c_void) -> *mut c_char;
pub type Game_DRS_OnSessionLoad_t = unsafe extern "C" fn(drs: *mut c_void);
pub type Game_DriverContracts_AdvanceSecretMeeting_t =
    unsafe extern "C" fn(contracts: *mut c_void, driver: u32, notify: u8);
pub type Game_DriverContracts_BuildOffers_t = unsafe extern "C" fn(contracts: *mut c_void, driver: u32);
pub type Game_DriverContracts_OfferSecretMeeting_t = unsafe extern "C" fn(
    contracts: *mut c_void,
    meeting: *const c_void,
    team: u32,
    driver: u32,
    reason: u8,
    notify: u8,
);
pub type Game_DriverSelectHeader_SetTabState_t =
    unsafe extern "C" fn(header: *mut c_void, tab: u64, content_visible: u8, enabled: u8);
pub type Game_EngineMgmt_AddGridPenalty_t =
    unsafe extern "C" fn(engine_management: *mut c_void, driver: *const u32, places: u32, reason: u32);
pub type Game_EngineMgmt_ApplySessionWear_t =
    unsafe extern "C" fn(engine_management: *mut c_void, driver: u32, amount: f32, flag: u8);
pub type Game_EntrantsUi_BuildCharacterRef_t =
    unsafe extern "C" fn(out: *mut c_void, character: *const u8, node: *mut c_void) -> *mut c_void;
pub type Game_EntrantsUi_OnCharacterListLoaded_t =
    unsafe extern "C" fn(ctx: *mut c_void, message: *const c_void) -> i64;
pub type Game_F1WorldActivityContainer_SelectEntrantList_t =
    unsafe extern "C" fn(container: *mut c_void, entrant_list: *const c_char);
pub type Game_Game_SetCurrentGameMode_t =
    unsafe extern "C" fn(mode_context: *mut c_void, mode: *mut c_void) -> u64;
pub type Game_Get_ModelNodeId_t = unsafe extern "C" fn(name: *const c_char) -> u32;
pub type Game_Handle_GarageCrewEvent_Thunk_t =
    unsafe extern "C" fn(cars: *mut *mut c_void, event: *mut c_void) -> u64;
pub type Game_Handle_SpecGarageCrewEvent_Thunk_t =
    unsafe extern "C" fn(self_: *mut c_void, event: *mut c_void) -> u64;
pub type Game_Network_CategoryOptionUsing_t = unsafe extern "C" fn(self_: *mut c_void);
pub type Game_Network_SendingRedFlagVehicleStatusUpdate_t =
    unsafe extern "C" fn(red_flag_state: *mut c_void, dt: *const f64);
pub type Game_Network_SetScenario_t =
    unsafe extern "C" fn(session: *mut c_void, stat_id: u8, spec: *const c_void);
pub type Game_OptionsStore_FindOption_t =
    unsafe extern "C" fn(map: *mut c_void, name: *mut *const c_char) -> u64;
pub type Game_PitLane_MoveDownSpline_t = unsafe extern "C" fn(
    pitlane: *mut c_void,
    car: u8,
    teleport_req: *mut c_void,
    alt_spline: bool,
    dt: f32,
    p6: bool,
    blend: bool,
    p8: bool,
    p9: bool,
);
pub type Game_PitLane_PutVehicleInPitBox_t = unsafe extern "C" fn(pitlane: *mut c_void, vehicle: u8);
pub type Game_Pitlane_SetVehicleStatus_t = unsafe extern "C" fn(self_: *mut c_void, vehicle: u8, status: u32);
pub type Game_Place_Vehicle_t = unsafe extern "C" fn(
    car: *mut c_void,
    tag: *const c_char,
    priority: c_int,
    mtx44_a: *const f32,
    mtx44_b: *const f32,
    dt: f32,
    flags: u32,
);
pub type Game_PlayerSetSubsystem_HandleUpdatePresenceNameMessage_t =
    unsafe extern "C" fn(player_set: *mut c_void, msg: *mut c_void);
pub type Game_PracticeTrackAcclimatisation_RefreshState_t = unsafe extern "C" fn(self_: *mut c_void);
pub type Game_Predict_StrategyWetness_t =
    unsafe extern "C" fn(self_: *mut c_void, window3: *const f32) -> f32;
pub type Game_RaceMarshalSystem_Update_t = unsafe extern "C" fn(system: *mut c_void, dt: *mut f64);
pub type Game_RaceStrategyVm_Populate_t = unsafe extern "C" fn(vm: *mut c_void);
pub type Game_Refresh_PerformanceComparison_t = unsafe extern "C" fn(comparison: *mut c_void);
pub type Game_Rnd_CalcUpgradeTypeProgress_t =
    unsafe extern "C" fn(instances: *mut *mut c_void, r#type: u8, state: u8, meta: *const c_void) -> f32;
pub type Game_Rnd_FillVehicleUpgrades_t =
    unsafe extern "C" fn(component: *mut c_void, car: u8, a3: u8, a4: u8);
pub type Game_SaveData_DeserialiseContainer_t = unsafe extern "C" fn(
    self_: *mut SaveDataContainer,
    result: *mut c_int,
    bag: *mut c_void,
    reset_on_error: u8,
    groups: u64,
) -> *mut c_int;
pub type Game_SaveData_SerialiseContainer_t = unsafe extern "C" fn(
    self_: *mut SaveDataContainer,
    result: *mut c_int,
    bag: *mut c_void,
    groups: u64,
) -> *mut c_int;
pub type Game_SaveData_SerialiseContainer_2_t = unsafe extern "C" fn(
    self_: *mut SaveDataContainer,
    result: *mut c_int,
    stream: *mut c_void,
    a4: *mut c_void,
    groups: u64,
    a6: *mut c_void,
) -> *mut c_int;
pub type Game_SaveData_WriteBag_t = unsafe extern "C" fn(codec: *mut NeBagCodec, bag: *mut c_void) -> c_int;
pub type Game_SaveMgr_RequestProfileSave_Thunk_t =
    unsafe extern "C" fn(save_mgr: *mut c_void, callback: *mut c_void, a3: u8, a4: u8) -> bool;
pub type Game_SaveSlotManager_Deserialise_t =
    unsafe extern "C" fn(iface: *mut c_void, bag: *mut c_void, ctx: *mut c_void) -> c_int;
pub type Game_SaveSlotManager_SelectSlot_t =
    unsafe extern "C" fn(manager: *mut c_void, slot_id: u64, force: bool, a4: u32) -> c_int;
pub type Game_SceneManagerSpawn_SpawnScene_t = unsafe extern "C" fn(
    mgr: *mut c_void,
    opts: *const u8,
    scene: *mut c_void,
    mtx44: *const f32,
    ov_a: *mut c_void,
    ov_b: *mut c_void,
    p7: u64,
) -> *mut c_void;
pub type Game_SeasonLength_ApplyPreset_t = unsafe extern "C" fn(component: *mut c_void, preset: u8);
pub type Game_SeasonLength_PushCalendarEntry_t =
    unsafe extern "C" fn(calendar: *mut c_void, entry: *const c_void) -> u64;
pub type Game_Set_VehicleController_Thunk_t =
    unsafe extern "C" fn(system: *mut c_void, vehicle: u64, ai: u32, flag: i32);
pub type Game_SpeechSystem_PostLine_t =
    unsafe extern "C" fn(speech: *mut c_void, line: *mut c_void, retrigger: u8);
pub type Game_TeamHQAdvanceViewModel_OnCommandRaised_t =
    unsafe extern "C" fn(vm: *mut c_void, cmd: *mut *mut NeStr, data: *mut c_void) -> u64;
pub type Game_TeamHQAdvanceViewModel_PressAdvance_t = unsafe extern "C" fn(vm: *mut c_void);
pub type Game_TeamHQAdvanceViewModel_ShowTodayCards_t = unsafe extern "C" fn(vm: *mut c_void) -> bool;
pub type Game_TeamHQAdvanceViewModel_StopAdvance_t = unsafe extern "C" fn(vm: *mut c_void);
pub type Game_TeamHQAdvanceViewModel_Update_t = unsafe extern "C" fn(vm: *mut c_void) -> u64;
pub type Game_Teleport_Vehicle_t = unsafe extern "C" fn(
    car: *mut c_void,
    name: *const c_char,
    prev_mtx: *mut f32,
    new_mtx: *mut f32,
    dt: f32,
    flags: u32,
);
pub type Game_TrackData_GetTrack_t =
    unsafe extern "C" fn(system: *mut c_void, track_ref: *const c_void, query: *const u8) -> *mut c_void;
pub type Game_TyreAllocationComponent_SetPresetChoiceForHumanDriver_t =
    unsafe extern "C" fn(component: *mut c_void, preset: u32, local_player: c_int);
pub type Gdb_HashMap_Rehash_t = unsafe extern "C" fn(map: *mut GdbValueMap, num_buckets: u64);
pub type Gdb_Load_GameDatabase_t =
    unsafe extern "C" fn(db: *mut GdbDatabase, cfg: *mut c_void) -> *mut GdbDatabase;
pub type Gdb_Lookup_Key_t = unsafe extern "C" fn(path: *const c_char) -> *mut *mut c_void;
pub type Ne_Compare_StringNoCase_t = unsafe extern "C" fn(a: *const c_char, b: *const c_char) -> i32;
pub type Ne_Detach_ChildNode_t = unsafe extern "C" fn(parent: *mut c_void, child: *mut c_void) -> c_int;
pub type Ne_Entity_FindComponent_t =
    unsafe extern "C" fn(entity: *mut c_void, r#type: *const c_void) -> *mut c_void;
pub type Ne_Format_String_t = unsafe extern "C" fn(out: *mut *mut NeStr, fmt: *const c_char, ...) -> i64;
pub type Ne_Get_SettingBool_t =
    unsafe extern "C" fn(node: *mut c_void, name: *const c_char, out: *mut u8, def: u8) -> u64;
pub type Ne_Get_SettingU32_t =
    unsafe extern "C" fn(node: *mut c_void, name: *const c_char, out: *mut u32, def: u32) -> u64;
pub type Ne_Heap_DeallocateNonSmall_t = unsafe extern "C" fn(heap: *mut c_void, ptr: *mut u8) -> u64;
pub type Ne_Log_Thunk_t = unsafe extern "C" fn(channel: *const c_char, fmt: *const c_char, ...);
pub type Ne_Make_RandomIdBlock_t = unsafe extern "C" fn(out: *mut u8) -> *mut u8;
pub type Ne_Make_StrFromCStr_t =
    unsafe extern "C" fn(out: *mut *mut NeStr, s: *const c_char, flags: u32, len: usize) -> u32;
pub type Ne_Parse_Long_t = unsafe extern "C" fn(text: *const c_char, end: *mut *mut c_char, base: i32) -> i32;
pub type Ne_PhysBody_ApplyImpulse_t =
    unsafe extern "C" fn(body: *mut c_void, J: *const f32, worldPoint: *const f32);
pub type Ne_Play_FullscreenVideo_t = unsafe extern "C" fn(
    feature_system: *mut c_void,
    path: *mut *mut NeStr,
    on_end: *const c_void,
    frame_callbacks: *const c_void,
    layer: i32,
);
pub type Ne_Post_BusMessage_t = unsafe extern "C" fn(bus: *mut c_void, msg: *mut c_void) -> u64;
pub type Ne_PresenceNameResolverFixed_AddName_t =
    unsafe extern "C" fn(resolver: *mut c_void, user: *const c_void, name: *const c_char);
pub type Ne_SaveSystem2Win_Mount_t = unsafe extern "C" fn(
    device: *mut c_void,
    op: *mut c_void,
    save: *mut c_void,
    for_saving: u8,
    a5: u8,
    created: *mut u8,
) -> u32;
pub type Ne_SaveSystem2Win_Start_t = unsafe extern "C" fn(device: *mut c_void) -> u64;
pub type Ne_SaveSystem_InitDevice_t = unsafe extern "C" fn(save_system: *mut c_void) -> c_int;
pub type Ne_Set_FixedString_t = unsafe extern "C" fn(dst: *mut c_void, src: *const c_char);
pub type Ne_StringTable_Lookup_t = unsafe extern "C" fn(table: *mut c_void, hash: u64) -> *const c_char;
pub type Ne_TempPool_Alloc_t = unsafe extern "C" fn(pool: *mut c_void, size: u64, align: u64) -> *mut c_void;
pub type Sess_Apply_ContentAvailability_t =
    unsafe extern "C" fn(self_: *mut c_void, availability: *mut c_void);
pub type Sess_Assign_GarageSlotsByTeamOrder_t = unsafe extern "C" fn(
    out_assignments: *mut c_void,
    entrants: *mut c_void,
    garages: *mut c_void,
    team_order: *mut c_void,
    fixed_teams: *mut c_void,
);
pub type Sess_Build_GroupResult_t =
    unsafe extern "C" fn(weekend: *mut c_void, group: u8, result: *mut c_void);
pub type Sess_Find_GroupRowPosition_t =
    unsafe extern "C" fn(weekend: *mut c_void, result: *mut c_void, driver_key: u32) -> *mut i32;
pub type Sess_FormationLap_Update_t =
    unsafe extern "C" fn(state: *mut c_void, route: *mut u32, dt: f64) -> *mut u32;
pub type Sess_GameWeatherSystem_SetActiveSessionType_t =
    unsafe extern "C" fn(gws: *mut c_void, session_type: c_int, time_scale: f32);
pub type Sess_GameWeatherSystem_SetupSession_t =
    unsafe extern "C" fn(ws: *mut c_void, settings: *mut c_void, config: *mut c_void);
pub type Sess_GameWeatherSystem_UpdateRandomSeed_t =
    unsafe extern "C" fn(ws: *mut c_void, seed: u32, weekend: u32);
pub type Sess_Get_PlayerVehicleIndex_t = unsafe extern "C" fn(ctx: *mut c_void, local_player: c_int) -> u8;
pub type Sess_GridRundown_Update_t =
    unsafe extern "C" fn(state: *mut c_void, route: *mut u32, dt: f64) -> *mut u32;
pub type Sess_InGamePQ_Update_t =
    unsafe extern "C" fn(state: *mut c_void, route: *mut u32, dt: f64) -> *mut u32;
pub type Sess_InGame_UpdateBase_t =
    unsafe extern "C" fn(state: *mut c_void, route: *mut u32, dt: f64) -> *mut u32;
pub type Sess_Invalidate_Lap_Thunk_t =
    unsafe extern "C" fn(rules: *mut c_void, vehicle: u32, reason: u32, mode: c_char);
pub type Sess_Issue_DriveThrough_t = unsafe extern "C" fn(rules: *mut c_void, incident: *mut u8, reason: u32);
pub type Sess_Issue_TimePenalty_t =
    unsafe extern "C" fn(rules: *mut c_void, incident: *mut u8, reason: u32, seconds: u32);
pub type Sess_Issue_Warning_t = unsafe extern "C" fn(rules: *mut c_void, incident: *mut u8, reason: u32);
pub type Sess_Load_Teams_t = unsafe extern "C" fn(entrants_system: *mut c_void, set: u8);
pub type Sess_PitStopUpdateJob_Update_t = unsafe extern "C" fn(job: *mut c_void);
pub type Sess_PreRaceToRace_Update_t = unsafe extern "C" fn(state: *mut c_void, route: *mut u32) -> *mut u32;
pub type Sess_PreRace_Update_t =
    unsafe extern "C" fn(state: *mut c_void, route: *mut u32, dt: f64) -> *mut u32;
pub type Sess_PuPanel_FillComponent_t =
    unsafe extern "C" fn(vm: *mut c_void, a2: u64, slot: u32, a4: u8, a5: u8, entry: *mut c_void);
pub type Sess_RaceStrategy_UpdateVehicle_t =
    unsafe extern "C" fn(proc: *mut c_void, race_session_data: *mut c_void);
pub type Sess_Record_GroupTimes_t =
    unsafe extern "C" fn(weekend: *mut c_void, group: u8, entries: *mut c_void);
pub type Sess_RedFlagRaceStrategy_Enter_t = unsafe extern "C" fn(state: *mut c_void) -> u64;
pub type Sess_RedFlagRaceStrategy_Update_t =
    unsafe extern "C" fn(state: *mut c_void, route: *mut u32) -> *mut u32;
pub type Sess_Register_RaceRecordables_t = unsafe extern "C" fn(game: *mut c_void);
pub type Sess_RuleManager_Initialise_t =
    unsafe extern "C" fn(self_: *mut c_void, systems: *mut c_void, components: *mut c_void) -> u32;
pub type Sess_Rules_IsPracticeQualifying_t =
    unsafe extern "C" fn(owner_with_session_at_0x1b8: *mut c_void) -> bool;
pub type Sess_SafetyCarTether_UpdateProgress_t = unsafe extern "C" fn(tether_entry: *mut c_void);
pub type Sess_SafetyCarTether_UpdateTarget_t = unsafe extern "C" fn(tether_entry: *mut c_void, gap: f32);
pub type Sess_ScDebrisAccumulator_Update_t = unsafe extern "C" fn(acc: *mut c_void);
pub type Sess_Set_SafetyCarDeploymentType_t =
    unsafe extern "C" fn(sc_state: *mut c_void, sc_deployment_msg: *const c_void);
pub type Sess_Setup_VehicleComponents_t = unsafe extern "C" fn(module: *mut c_void);
pub type Sess_Tod_BuildSessionDataset_Thunk_t = unsafe extern "C" fn(
    tod_owner: *mut c_void,
    types: *const u8,
    hold_s: *const f32,
    trans_s: *const f32,
    random: *const u8,
    count: u32,
    len_s: f32,
    session: u16,
    use_count: u8,
);
pub type Sess_TyreAllocation_OnSessionStart_t = unsafe extern "C" fn(component: *mut c_void);
pub type Sess_TyreClass_FromName_t =
    unsafe extern "C" fn(classes: *mut c_void, name: *const c_void, check_weekend_format: u8) -> *mut c_void;
pub type Sess_Tyres_ClosestAvailableCompound_t = unsafe extern "C" fn(
    tyre_rules: *mut c_void,
    vehicle: u8,
    preference: *const u32,
    wear_scale: f32,
) -> u32;
pub type Sess_WeatherComponent_SetSystemRandomSeed_t = unsafe extern "C" fn(component: *mut c_void);
pub type Sess_Weekend_Start_t =
    unsafe extern "C" fn(weekend: *mut c_void, entrants: *const c_void, begin: bool);
pub type Sess_WinnerSequence_Update_t =
    unsafe extern "C" fn(state: *mut c_void, route: *mut u32, dt: f64) -> *mut u32;
pub type Snd_Request_JobWorker_t = unsafe extern "C" fn(worker: *mut c_void, job_type: u32, count: u32);
pub type Team_Insert_RecordByStatId_t =
    unsafe extern "C" fn(slots: *mut c_void, record: *const u32) -> *mut c_void;
pub type Trk_GrandPrixTrackSelectVm_OnCommandRaised_t =
    unsafe extern "C" fn(vm: *mut c_void, cmd: *mut *mut u32) -> *mut *mut c_void;
pub type Trk_TimeTrialTrackSelectVm_OnCommandRaised_t =
    unsafe extern "C" fn(vm: *mut c_void, cmd: *mut *mut u32) -> u64;
pub type Vm_GetOrCreate_ListChild_t =
    unsafe extern "C" fn(registry: *mut c_void, parent: *mut c_void, hash: *const u32) -> *mut *mut c_void;
pub type Vm_List_AppendChild_t = unsafe extern "C" fn(list: *mut c_void) -> *mut *mut c_void;
pub type Vm_Release_Node_t = unsafe extern "C" fn(registry: *mut c_void, node: *mut c_void);
pub type Vm_Resolve_RegistryNode_t = unsafe extern "C" fn(
    registry: *mut c_void,
    path: *const c_char,
    base: *const c_void,
    create: *mut c_void,
) -> *mut c_void;
pub type Vm_Set_Bool_t = unsafe extern "C" fn(
    registry: *mut c_void,
    parent: *mut c_void,
    name: *const c_char,
    value: *mut u8,
    overwrite: c_int,
) -> *mut c_void;
pub type Vm_Set_NodeRef_t = unsafe extern "C" fn(
    registry: *mut c_void,
    parent: *mut c_void,
    name: *const c_char,
    value: *mut c_void,
    overwrite: c_int,
) -> *mut *mut c_void;
pub type Vm_Set_String_t = unsafe extern "C" fn(
    registry: *mut c_void,
    parent: *mut c_void,
    name: *const c_char,
    value: *mut *mut NeStr,
    overwrite: c_int,
) -> *mut c_void;
pub type Vm_Set_TabEnabled_t = unsafe extern "C" fn(tabs: usize, node: usize, enabled: u32);

pub mod bindings {
    use super::{Binding, Status};
    pub const Ai_BehaviourTestRetire_Update: Binding = Binding {
        id: "Ai_BehaviourTestRetire_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ai_Calc_GripLimitScaleA: Binding = Binding {
        id: "Ai_Calc_GripLimitScaleA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ai_Calc_GripLimitScaleB: Binding = Binding {
        id: "Ai_Calc_GripLimitScaleB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ai_DriverVariationInRaceSystem_Setup: Binding = Binding {
        id: "Ai_DriverVariationInRaceSystem_Setup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ai_Job_UpdateCarsParallel: Binding = Binding {
        id: "Ai_Job_UpdateCarsParallel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ai_Select_ErsDeployMode: Binding = Binding {
        id: "Ai_Select_ErsDeployMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ai_Update_CompoundPaceCurves: Binding = Binding {
        id: "Ai_Update_CompoundPaceCurves",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Asset_Load_DevSessionWorlds: Binding = Binding {
        id: "Asset_Load_DevSessionWorlds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Asset_Load_MaterialList: Binding = Binding {
        id: "Asset_Load_MaterialList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Asset_Load_ParticleResourcePack: Binding = Binding {
        id: "Asset_Load_ParticleResourcePack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Asset_Map_EaidToGroupA: Binding = Binding {
        id: "Asset_Map_EaidToGroupA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Asset_Unk_142a982b0: Binding = Binding {
        id: "Asset_Unk_142a982b0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Asset_Unk_14379dd00: Binding = Binding {
        id: "Asset_Unk_14379dd00",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Assets_CreateSystemForPerformanceMode: Binding = Binding {
        id: "Car_Assets_CreateSystemForPerformanceMode",
        kind: "func",
        status: Status::Stable,
    };
    pub const Car_Build_VtfComponents: Binding = Binding {
        id: "Car_Build_VtfComponents",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Chassis_Build: Binding = Binding {
        id: "Car_Chassis_Build",
        kind: "func",
        status: Status::Stable,
    };
    pub const Car_Create_DetachablePart: Binding = Binding {
        id: "Car_Create_DetachablePart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Deformation_AcquireParticles: Binding = Binding {
        id: "Car_Deformation_AcquireParticles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Deformation_ClearPool: Binding = Binding {
        id: "Car_Deformation_ClearPool",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Deformation_CreateBody: Binding = Binding {
        id: "Car_Deformation_CreateBody",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Deformation_DestroyBody: Binding = Binding {
        id: "Car_Deformation_DestroyBody",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Deformation_ReleaseParticles: Binding = Binding {
        id: "Car_Deformation_ReleaseParticles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Deformation_ReservePool: Binding = Binding {
        id: "Car_Deformation_ReservePool",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_DetachablePart_Update: Binding = Binding {
        id: "Car_DetachablePart_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Engine_ComputeDerived_Thunk: Binding = Binding {
        id: "Car_Engine_ComputeDerived_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_FrontWing_Init: Binding = Binding {
        id: "Car_FrontWing_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_FrontWing_OnCollision: Binding = Binding {
        id: "Car_FrontWing_OnCollision",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_FuelEcu_SetMode_Thunk: Binding = Binding {
        id: "Car_FuelEcu_SetMode_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Load_Vtf: Binding = Binding {
        id: "Car_Load_Vtf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Model_BindCockpitNodes: Binding = Binding {
        id: "Car_Model_BindCockpitNodes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Physics_PostVehicleUpdate: Binding = Binding {
        id: "Car_Physics_PostVehicleUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Powertrain_Step: Binding = Binding {
        id: "Car_Powertrain_Step",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Read_Engine: Binding = Binding {
        id: "Car_Read_Engine",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Read_F1Aero: Binding = Binding {
        id: "Car_Read_F1Aero",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Read_F1Assists: Binding = Binding {
        id: "Car_Read_F1Assists",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Read_F1Tuning: Binding = Binding {
        id: "Car_Read_F1Tuning",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Read_GearBox: Binding = Binding {
        id: "Car_Read_GearBox",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Register_DetachablePart_Thunk: Binding = Binding {
        id: "Car_Register_DetachablePart_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Reset_LastFittedUpgrade: Binding = Binding {
        id: "Car_Reset_LastFittedUpgrade",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Setup_ApplyTuning: Binding = Binding {
        id: "Car_Setup_ApplyTuning",
        kind: "func",
        status: Status::Stable,
    };
    pub const Car_SteeringWheelLeds_Update: Binding = Binding {
        id: "Car_SteeringWheelLeds_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Systems_FindOrCreate: Binding = Binding {
        id: "Car_Systems_FindOrCreate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Systems_Reset_Thunk: Binding = Binding {
        id: "Car_Systems_Reset_Thunk",
        kind: "func",
        status: Status::Stable,
    };
    pub const Car_Tuning_SetChassisZCom: Binding = Binding {
        id: "Car_Tuning_SetChassisZCom",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tuning_SetEngineBrakingAndRadiator: Binding = Binding {
        id: "Car_Tuning_SetEngineBrakingAndRadiator",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tuning_SetFinalDrive: Binding = Binding {
        id: "Car_Tuning_SetFinalDrive",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_TyreSystem_PitstopComplete: Binding = Binding {
        id: "Car_TyreSystem_PitstopComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tyre_GripScale: Binding = Binding {
        id: "Car_Tyre_GripScale",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tyre_MagicFormula: Binding = Binding {
        id: "Car_Tyre_MagicFormula",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tyre_SetWearFactor: Binding = Binding {
        id: "Car_Tyre_SetWearFactor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tyre_UpdateWear: Binding = Binding {
        id: "Car_Tyre_UpdateWear",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_TyresPitstopComplete_ForCarPair: Binding = Binding {
        id: "Car_TyresPitstopComplete_ForCarPair",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tyres_ApplyStateToCarPair: Binding = Binding {
        id: "Car_Tyres_ApplyStateToCarPair",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Tyres_CompletePitstopForCarList: Binding = Binding {
        id: "Car_Tyres_CompletePitstopForCarList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Upgrades_ApplyToVehicle: Binding = Binding {
        id: "Car_Upgrades_ApplyToVehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Upgrades_CalcNonLinearMass: Binding = Binding {
        id: "Car_Upgrades_CalcNonLinearMass",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Upgrades_CalcTypeMass: Binding = Binding {
        id: "Car_Upgrades_CalcTypeMass",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Write_Differential: Binding = Binding {
        id: "Car_Write_Differential",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Write_EnergyRecovery: Binding = Binding {
        id: "Car_Write_EnergyRecovery",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Write_Engine: Binding = Binding {
        id: "Car_Write_Engine",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Write_FuelTank: Binding = Binding {
        id: "Car_Write_FuelTank",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Write_GearBox: Binding = Binding {
        id: "Car_Write_GearBox",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Car_Write_Turbo: Binding = Binding {
        id: "Car_Write_Turbo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Add_SeasonWeekend: Binding = Binding {
        id: "Career_Add_SeasonWeekend",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Add_TeamModifier: Binding = Binding {
        id: "Career_Add_TeamModifier",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Adjust_RpDriverWallet: Binding = Binding {
        id: "Career_Adjust_RpDriverWallet",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Adjust_RpTeamWallet: Binding = Binding {
        id: "Career_Adjust_RpTeamWallet",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Build_DriverContract: Binding = Binding {
        id: "Career_Build_DriverContract",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Build_LineupChanges: Binding = Binding {
        id: "Career_Build_LineupChanges",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Calc_UpgradeCost: Binding = Binding {
        id: "Career_Calc_UpgradeCost",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Calendar_IgnoreActivitiesInRange: Binding = Binding {
        id: "Career_Calendar_IgnoreActivitiesInRange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Calendar_IgnoreActivityById: Binding = Binding {
        id: "Career_Calendar_IgnoreActivityById",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Calendar_MoveTrackData: Binding = Binding {
        id: "Career_Calendar_MoveTrackData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Cancel_UpgradeDevelopment: Binding = Binding {
        id: "Career_Cancel_UpgradeDevelopment",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Cancel_UpgradeResearch: Binding = Binding {
        id: "Career_Cancel_UpgradeResearch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_CareerOnlineDoc_ResetAfterSync: Binding = Binding {
        id: "Career_CareerOnlineDoc_ResetAfterSync",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Complete_PerkTreeNode: Binding = Binding {
        id: "Career_Complete_PerkTreeNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Construct_RnDDeptCard: Binding = Binding {
        id: "Career_Construct_RnDDeptCard",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractHubVM_OnActivate: Binding = Binding {
        id: "Career_ContractHubVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractIntro_OnEnter: Binding = Binding {
        id: "Career_ContractIntro_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractIntro_OnUpdate: Binding = Binding {
        id: "Career_ContractIntro_OnUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractOutro_OnEnter: Binding = Binding {
        id: "Career_ContractOutro_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractOutro_OnUpdate: Binding = Binding {
        id: "Career_ContractOutro_OnUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractPerksVM_OnActivate: Binding = Binding {
        id: "Career_ContractPerksVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractUpdate_OnEnter: Binding = Binding {
        id: "Career_ContractUpdate_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractUpdate_OnExit: Binding = Binding {
        id: "Career_ContractUpdate_OnExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ContractUpdate_OnUpdate: Binding = Binding {
        id: "Career_ContractUpdate_OnUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Create_SeasonWeekend: Binding = Binding {
        id: "Career_Create_SeasonWeekend",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Decide_ContractNegotiation: Binding = Binding {
        id: "Career_Decide_ContractNegotiation",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_DriverNumbers_CheckExisting: Binding = Binding {
        id: "Career_DriverNumbers_CheckExisting",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Eval_Curve: Binding = Binding {
        id: "Career_Eval_Curve",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Get_CareerLengthDays_Thunk: Binding = Binding {
        id: "Career_Get_CareerLengthDays_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Get_DriverOverallRating: Binding = Binding {
        id: "Career_Get_DriverOverallRating",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Get_RpWallet: Binding = Binding {
        id: "Career_Get_RpWallet",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Get_TeamRenownData: Binding = Binding {
        id: "Career_Get_TeamRenownData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_MyTeamCorporateSetupVM_OnAction: Binding = Binding {
        id: "Career_MyTeamCorporateSetupVM_OnAction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_MyTeamCorporateSetupVM_OnActivate: Binding = Binding {
        id: "Career_MyTeamCorporateSetupVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_MyTeamOwnerBackgroundVM_OnAction: Binding = Binding {
        id: "Career_MyTeamOwnerBackgroundVM_OnAction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_MyTeamSetup_AdvanceFadeOut: Binding = Binding {
        id: "Career_MyTeamSetup_AdvanceFadeOut",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_MyTeamVM_RequestThumbnails: Binding = Binding {
        id: "Career_MyTeamVM_RequestThumbnails",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Needs_DriverContract: Binding = Binding {
        id: "Career_Needs_DriverContract",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiationsB_EndFlow: Binding = Binding {
        id: "Career_OnlineNegotiationsB_EndFlow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiationsB_OnNegotiationsFinished: Binding = Binding {
        id: "Career_OnlineNegotiationsB_OnNegotiationsFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiationsB_ProcessContractRecv: Binding = Binding {
        id: "Career_OnlineNegotiationsB_ProcessContractRecv",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiationsB_ProcessContractSent: Binding = Binding {
        id: "Career_OnlineNegotiationsB_ProcessContractSent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiations_EndFlow: Binding = Binding {
        id: "Career_OnlineNegotiations_EndFlow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiations_IsLeadPlayerParticipating: Binding = Binding {
        id: "Career_OnlineNegotiations_IsLeadPlayerParticipating",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiations_IsLeadPlayersTurnTeamSel: Binding = Binding {
        id: "Career_OnlineNegotiations_IsLeadPlayersTurnTeamSel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiations_OnNegotiationsFinished: Binding = Binding {
        id: "Career_OnlineNegotiations_OnNegotiationsFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiations_ProcessContractRecv: Binding = Binding {
        id: "Career_OnlineNegotiations_ProcessContractRecv",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiations_ProcessContractSent: Binding = Binding {
        id: "Career_OnlineNegotiations_ProcessContractSent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_OnlineNegotiations_ShouldShowFiredState: Binding = Binding {
        id: "Career_OnlineNegotiations_ShouldShowFiredState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Post_UpgradeFittedRatio: Binding = Binding {
        id: "Career_Post_UpgradeFittedRatio",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Rebuild_ChampionshipStandings: Binding = Binding {
        id: "Career_Rebuild_ChampionshipStandings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Rebuild_SeasonStandings: Binding = Binding {
        id: "Career_Rebuild_SeasonStandings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Record_DriverMove: Binding = Binding {
        id: "Career_Record_DriverMove",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Refresh_RnDDeptMode: Binding = Binding {
        id: "Career_Refresh_RnDDeptMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Refresh_UpgradeAvailability: Binding = Binding {
        id: "Career_Refresh_UpgradeAvailability",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ScheduleEntryDeptEvent_Start: Binding = Binding {
        id: "Career_ScheduleEntryDeptEvent_Start",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ScheduleEntry_Finish: Binding = Binding {
        id: "Career_ScheduleEntry_Finish",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_ScheduleEntry_TryStart: Binding = Binding {
        id: "Career_ScheduleEntry_TryStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_SecretMeetingsVM_OnScreenDeactivated: Binding = Binding {
        id: "Career_SecretMeetingsVM_OnScreenDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Set_DriverContractState: Binding = Binding {
        id: "Career_Set_DriverContractState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Set_EntrantData_Thunk: Binding = Binding {
        id: "Career_Set_EntrantData_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Set_UpgradeFacilityLocked: Binding = Binding {
        id: "Career_Set_UpgradeFacilityLocked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Set_UpgradesOwner: Binding = Binding {
        id: "Career_Set_UpgradesOwner",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Sign_DriverContract: Binding = Binding {
        id: "Career_Sign_DriverContract",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Start_UpgradeDevelopment: Binding = Binding {
        id: "Career_Start_UpgradeDevelopment",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Start_UpgradeResearch: Binding = Binding {
        id: "Career_Start_UpgradeResearch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_StateTeamHq_OnStateExit: Binding = Binding {
        id: "Career_StateTeamHq_OnStateExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqAdvanceVM_AllowOnlineUnready: Binding = Binding {
        id: "Career_TeamHqAdvanceVM_AllowOnlineUnready",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqAdvanceVM_EnterPressInterview: Binding = Binding {
        id: "Career_TeamHqAdvanceVM_EnterPressInterview",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqAdvanceVM_Initialise: Binding = Binding {
        id: "Career_TeamHqAdvanceVM_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqAdvanceVM_OnNotificationsAnimOff: Binding = Binding {
        id: "Career_TeamHqAdvanceVM_OnNotificationsAnimOff",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqAdvance_ClearDelayedByAnims: Binding = Binding {
        id: "Career_TeamHqAdvance_ClearDelayedByAnims",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqAdvance_RestartTimer: Binding = Binding {
        id: "Career_TeamHqAdvance_RestartTimer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqAdvance_SetAdvanceRequested: Binding = Binding {
        id: "Career_TeamHqAdvance_SetAdvanceRequested",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqContainerVM_OnAction: Binding = Binding {
        id: "Career_TeamHqContainerVM_OnAction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqContainerVM_OnScreenActivated: Binding = Binding {
        id: "Career_TeamHqContainerVM_OnScreenActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqDriverRender_Init: Binding = Binding {
        id: "Career_TeamHqDriverRender_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqFacilities_CameraCycleFadeIn: Binding = Binding {
        id: "Career_TeamHqFacilities_CameraCycleFadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqFacilities_CameraCycleFadeInNamed: Binding = Binding {
        id: "Career_TeamHqFacilities_CameraCycleFadeInNamed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqFacilities_CameraCycleFadeInOnEnter: Binding = Binding {
        id: "Career_TeamHqFacilities_CameraCycleFadeInOnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqFacilities_CameraCycleStep: Binding = Binding {
        id: "Career_TeamHqFacilities_CameraCycleStep",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqLaptopVM_OnAction: Binding = Binding {
        id: "Career_TeamHqLaptopVM_OnAction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqLaptopVM_OnDeactivate: Binding = Binding {
        id: "Career_TeamHqLaptopVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqMyTeamVM_ClearStopping: Binding = Binding {
        id: "Career_TeamHqMyTeamVM_ClearStopping",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqMyTeamVM_Initialise: Binding = Binding {
        id: "Career_TeamHqMyTeamVM_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqMyTeamVM_OnCommandRaised: Binding = Binding {
        id: "Career_TeamHqMyTeamVM_OnCommandRaised",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqMyTeamVM_OnTimer: Binding = Binding {
        id: "Career_TeamHqMyTeamVM_OnTimer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqMyTeamVM_SignalStartAdvancing: Binding = Binding {
        id: "Career_TeamHqMyTeamVM_SignalStartAdvancing",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqMyTeamVM_StopAdvancing: Binding = Binding {
        id: "Career_TeamHqMyTeamVM_StopAdvancing",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqMyTeamVM_Update: Binding = Binding {
        id: "Career_TeamHqMyTeamVM_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_TeamHqRender_ResolveServices: Binding = Binding {
        id: "Career_TeamHqRender_ResolveServices",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_Teardown_Weekend: Binding = Binding {
        id: "Career_Teardown_Weekend",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_WizardTitleSliceVM_OnActivate: Binding = Binding {
        id: "Career_WizardTitleSliceVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Career_WizardTitleSliceVM_OnDeactivate: Binding = Binding {
        id: "Career_WizardTitleSliceVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ent_Deserialise_EntrantVector: Binding = Binding {
        id: "Ent_Deserialise_EntrantVector",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeFlow_Set_StateStatus: Binding = Binding {
        id: "FeFlow_Set_StateStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeHome_Add_CareerTile: Binding = Binding {
        id: "FeHome_Add_CareerTile",
        kind: "func",
        status: Status::Stable,
    };
    pub const FeHome_Handle_Command: Binding = Binding {
        id: "FeHome_Handle_Command",
        kind: "func",
        status: Status::Stable,
    };
    pub const FeHome_StoryItem_Populate: Binding = Binding {
        id: "FeHome_StoryItem_Populate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Add_EraTile: Binding = Binding {
        id: "FeTile_Add_EraTile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Build_List: Binding = Binding {
        id: "FeTile_Build_List",
        kind: "func",
        status: Status::Stable,
    };
    pub const FeTile_CareerDlcItem_Populate: Binding = Binding {
        id: "FeTile_CareerDlcItem_Populate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Check_EntrantListUsable: Binding = Binding {
        id: "FeTile_Check_EntrantListUsable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Clear_EraTileSet_Thunk: Binding = Binding {
        id: "FeTile_Clear_EraTileSet_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Construct_CareerItem: Binding = Binding {
        id: "FeTile_Construct_CareerItem",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Get_ListFirstEntrantId: Binding = Binding {
        id: "FeTile_Get_ListFirstEntrantId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Is_EraScenarioAvailable_Thunk: Binding = Binding {
        id: "FeTile_Is_EraScenarioAvailable_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_ItemBase_Populate: Binding = Binding {
        id: "FeTile_ItemBase_Populate",
        kind: "func",
        status: Status::Stable,
    };
    pub const FeTile_ItemBase_Slot78_Thunk: Binding = Binding {
        id: "FeTile_ItemBase_Slot78_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_ItemGeneric_Populate: Binding = Binding {
        id: "FeTile_ItemGeneric_Populate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Press_EraTile: Binding = Binding {
        id: "FeTile_Press_EraTile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const FeTile_Route_CareerItem: Binding = Binding {
        id: "FeTile_Route_CareerItem",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AccelerationDecelerationForcesCameraModifier_OnEnable: Binding = Binding {
        id: "Game_AccelerationDecelerationForcesCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ActorAssetGroupManager_Update: Binding = Binding {
        id: "Game_ActorAssetGroupManager_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ActorManager_Initialise: Binding = Binding {
        id: "Game_ActorManager_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AdaptiveAi_RecordSessionFinished: Binding = Binding {
        id: "Game_AdaptiveAi_RecordSessionFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AiManager_Initialise: Binding = Binding {
        id: "Game_AiManager_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AiPitDecision_UpdateDroppedPlayerCar: Binding = Binding {
        id: "Game_AiPitDecision_UpdateDroppedPlayerCar",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AnimLayers_EvaluateBlend: Binding = Binding {
        id: "Game_AnimLayers_EvaluateBlend",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AnimLayers_EvaluateFallback: Binding = Binding {
        id: "Game_AnimLayers_EvaluateFallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AnimLayers_EvaluateSingle: Binding = Binding {
        id: "Game_AnimLayers_EvaluateSingle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AnimatedDriver_Initialise: Binding = Binding {
        id: "Game_AnimatedDriver_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AudioAutodetect_Initialise: Binding = Binding {
        id: "Game_AudioAutodetect_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_AudioEngineBundle_Initialise: Binding = Binding {
        id: "Game_AudioEngineBundle_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_BenchmarkGameMode_BuildSessionSettings: Binding = Binding {
        id: "Game_BenchmarkGameMode_BuildSessionSettings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_BetaCode_OnEntered: Binding = Binding {
        id: "Game_BetaCode_OnEntered",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Breakable_SpawnPieceRenderModel: Binding = Binding {
        id: "Game_Breakable_SpawnPieceRenderModel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_BroadcastViewVehicleVM_OnActivate: Binding = Binding {
        id: "Game_BroadcastViewVehicleVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_BroadcastViewVehicleVM_OnDeactivate: Binding = Binding {
        id: "Game_BroadcastViewVehicleVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_BusySpinner_StopAndFadeOutUi: Binding = Binding {
        id: "Game_BusySpinner_StopAndFadeOutUi",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CameraActor_ProcessPendingCameraChanges: Binding = Binding {
        id: "Game_CameraActor_ProcessPendingCameraChanges",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CameraActor_SyncClipTarget: Binding = Binding {
        id: "Game_CameraActor_SyncClipTarget",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CameraCollection_FindByName: Binding = Binding {
        id: "Game_CameraCollection_FindByName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CameraCollection_FindByUid: Binding = Binding {
        id: "Game_CameraCollection_FindByUid",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CarCustomisation_ApplyAllCustomisation: Binding = Binding {
        id: "Game_CarCustomisation_ApplyAllCustomisation",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CarCustomisation_ApplyDriverNumbers: Binding = Binding {
        id: "Game_CarCustomisation_ApplyDriverNumbers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CarCustomisation_ApplyLiveryPreview: Binding = Binding {
        id: "Game_CarCustomisation_ApplyLiveryPreview",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CarCustomisation_MarkCarDirtyOff: Binding = Binding {
        id: "Game_CarCustomisation_MarkCarDirtyOff",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CarCustomisation_SetupWritableCamera: Binding = Binding {
        id: "Game_CarCustomisation_SetupWritableCamera",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CarDecals_BuildList: Binding = Binding {
        id: "Game_CarDecals_BuildList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerActivities_AddActivity: Binding = Binding {
        id: "Game_CareerActivities_AddActivity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerActivities_RemoveActivity: Binding = Binding {
        id: "Game_CareerActivities_RemoveActivity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerContinue_HandleCommand: Binding = Binding {
        id: "Game_CareerContinue_HandleCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventResourceManager_DeactivateExtraWorld: Binding = Binding {
        id: "Game_CareerEventResourceManager_DeactivateExtraWorld",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventResourceManager_ShutdownExtraWorlds: Binding = Binding {
        id: "Game_CareerEventResourceManager_ShutdownExtraWorlds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEvent_OnEventKey: Binding = Binding {
        id: "Game_CareerEvent_OnEventKey",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEvent_OnSequenceEvent: Binding = Binding {
        id: "Game_CareerEvent_OnSequenceEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_CloseLaptop: Binding = Binding {
        id: "Game_CareerEventsSequencer_CloseLaptop",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_Destroy: Binding = Binding {
        id: "Game_CareerEventsSequencer_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_Initialise: Binding = Binding {
        id: "Game_CareerEventsSequencer_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_IsPausableA: Binding = Binding {
        id: "Game_CareerEventsSequencer_IsPausableA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_IsPausableB: Binding = Binding {
        id: "Game_CareerEventsSequencer_IsPausableB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_OnFadedOutForExit: Binding = Binding {
        id: "Game_CareerEventsSequencer_OnFadedOutForExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_OnHubClosed: Binding = Binding {
        id: "Game_CareerEventsSequencer_OnHubClosed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_OpenLaptop: Binding = Binding {
        id: "Game_CareerEventsSequencer_OpenLaptop",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_SetAllEnabled: Binding = Binding {
        id: "Game_CareerEventsSequencer_SetAllEnabled",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_SetLoadingEnabled: Binding = Binding {
        id: "Game_CareerEventsSequencer_SetLoadingEnabled",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_SpawnDriverScene: Binding = Binding {
        id: "Game_CareerEventsSequencer_SpawnDriverScene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_TriggerFadeOutForExit: Binding = Binding {
        id: "Game_CareerEventsSequencer_TriggerFadeOutForExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerEventsSequencer_UnloadAll: Binding = Binding {
        id: "Game_CareerEventsSequencer_UnloadAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerHubRndAreaScript_Initialise: Binding = Binding {
        id: "Game_CareerHubRndAreaScript_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerSetup_ResetEntrants: Binding = Binding {
        id: "Game_CareerSetup_ResetEntrants",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerWizard_PushNextStep: Binding = Binding {
        id: "Game_CareerWizard_PushNextStep",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerWizard_ResolveOrExit: Binding = Binding {
        id: "Game_CareerWizard_ResolveOrExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CareerWizard_StartsInF1: Binding = Binding {
        id: "Game_CareerWizard_StartsInF1",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Career_FadeOutForSessionRequest: Binding = Binding {
        id: "Game_Career_FadeOutForSessionRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Career_HideDriversOnExit: Binding = Binding {
        id: "Game_Career_HideDriversOnExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Career_IsDriverCareerMode: Binding = Binding {
        id: "Game_Career_IsDriverCareerMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Career_SpawnMainDriverPose: Binding = Binding {
        id: "Game_Career_SpawnMainDriverPose",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ChallengeEvent_HandleCommand: Binding = Binding {
        id: "Game_ChallengeEvent_HandleCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ChallengeEvent_OnTabActivated: Binding = Binding {
        id: "Game_ChallengeEvent_OnTabActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ChallengeEvent_OnTabDeactivated: Binding = Binding {
        id: "Game_ChallengeEvent_OnTabDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ChampSession_Reset: Binding = Binding {
        id: "Game_ChampSession_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ChampionshipGameMode_BuildSessionSettings: Binding = Binding {
        id: "Game_ChampionshipGameMode_BuildSessionSettings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Championship_AddSessionDefaultLength: Binding = Binding {
        id: "Game_Championship_AddSessionDefaultLength",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Championship_AddSessionFromDef: Binding = Binding {
        id: "Game_Championship_AddSessionFromDef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Championship_AddSessionWithFormat: Binding = Binding {
        id: "Game_Championship_AddSessionWithFormat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Championship_ResetSessionsForRestart: Binding = Binding {
        id: "Game_Championship_ResetSessionsForRestart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicDirector_SendCinematicData: Binding = Binding {
        id: "Game_CinematicDirector_SendCinematicData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicPitStop_Start: Binding = Binding {
        id: "Game_CinematicPitStop_Start",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicPitStop_Stop: Binding = Binding {
        id: "Game_CinematicPitStop_Stop",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicSafetyCar_FadeIn: Binding = Binding {
        id: "Game_CinematicSafetyCar_FadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicSafetyCar_FadeOut: Binding = Binding {
        id: "Game_CinematicSafetyCar_FadeOut",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicSafetyCar_HideHud: Binding = Binding {
        id: "Game_CinematicSafetyCar_HideHud",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicSafetyCar_RunThenFadeIn: Binding = Binding {
        id: "Game_CinematicSafetyCar_RunThenFadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CinematicSafetyCar_Stop: Binding = Binding {
        id: "Game_CinematicSafetyCar_Stop",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CollisionBasedShakeCameraModifier_OnEnable: Binding = Binding {
        id: "Game_CollisionBasedShakeCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ContractTeamSelect_OnEnter: Binding = Binding {
        id: "Game_ContractTeamSelect_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ContractTeamSelect_OnExit: Binding = Binding {
        id: "Game_ContractTeamSelect_OnExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ContractTeamSelect_OnUpdate: Binding = Binding {
        id: "Game_ContractTeamSelect_OnUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ControllerState_SyncHudHideReason13: Binding = Binding {
        id: "Game_ControllerState_SyncHudHideReason13",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CreateSessionDlcSelectTab_BuildTiles: Binding = Binding {
        id: "Game_CreateSessionDlcSelectTab_BuildTiles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CreateSessionDlcSelectTab_FillTile: Binding = Binding {
        id: "Game_CreateSessionDlcSelectTab_FillTile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CreateSessionDlcSelectTab_HandleCommand: Binding = Binding {
        id: "Game_CreateSessionDlcSelectTab_HandleCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CreditsSelectVM_OnScreenActivated: Binding = Binding {
        id: "Game_CreditsSelectVM_OnScreenActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CrewWorkingOnCar_Initialise: Binding = Binding {
        id: "Game_CrewWorkingOnCar_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomDriverNetworkMessageHandler_AddReplicatedDriver: Binding = Binding {
        id: "Game_CustomDriverNetworkMessageHandler_AddReplicatedDriver",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomDriverNetworkMessageHandler_FindOrCreateDriverStatId: Binding = Binding {
        id: "Game_CustomDriverNetworkMessageHandler_FindOrCreateDriverStatId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomDriverNetworkMessageHandler_HandleResponse: Binding = Binding {
        id: "Game_CustomDriverNetworkMessageHandler_HandleResponse",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomDriverNetworkMessageHandler_RemoveCustomDriver: Binding = Binding {
        id: "Game_CustomDriverNetworkMessageHandler_RemoveCustomDriver",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomDriverNetworkMessageHandler_RemovePlayerReplData: Binding = Binding {
        id: "Game_CustomDriverNetworkMessageHandler_RemovePlayerReplData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomDriverNetworkMessageHandler_ReplaceReplicatedDriver: Binding = Binding {
        id: "Game_CustomDriverNetworkMessageHandler_ReplaceReplicatedDriver",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomDriverNetworkMessageHandler_UpdateReplicatedDriver: Binding = Binding {
        id: "Game_CustomDriverNetworkMessageHandler_UpdateReplicatedDriver",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationDecals_DeleteFiles: Binding = Binding {
        id: "Game_CustomisationDecals_DeleteFiles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationDecals_Destroy: Binding = Binding {
        id: "Game_CustomisationDecals_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationDecals_DestroyMaterialOverride: Binding = Binding {
        id: "Game_CustomisationDecals_DestroyMaterialOverride",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationDecals_DestroySetProxy: Binding = Binding {
        id: "Game_CustomisationDecals_DestroySetProxy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationDecals_LoadConfig: Binding = Binding {
        id: "Game_CustomisationDecals_LoadConfig",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationDecals_OpenFile: Binding = Binding {
        id: "Game_CustomisationDecals_OpenFile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationDecals_UnloadAssetBatch: Binding = Binding {
        id: "Game_CustomisationDecals_UnloadAssetBatch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationOptionsVM_OnActivate: Binding = Binding {
        id: "Game_CustomisationOptionsVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CustomisationOptionsVM_OnDeactivate: Binding = Binding {
        id: "Game_CustomisationOptionsVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneParams_SetCameraIndex: Binding = Binding {
        id: "Game_CutSceneParams_SetCameraIndex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneParams_SetCutSceneName: Binding = Binding {
        id: "Game_CutSceneParams_SetCutSceneName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneParams_SetupDataFromEntrant: Binding = Binding {
        id: "Game_CutSceneParams_SetupDataFromEntrant",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneParams_SetupFromSpecificInfoFallbacks: Binding = Binding {
        id: "Game_CutSceneParams_SetupFromSpecificInfoFallbacks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneParams_SetupLeadEngineerFromTeam: Binding = Binding {
        id: "Game_CutSceneParams_SetupLeadEngineerFromTeam",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScriptCmd_SetStateLoading: Binding = Binding {
        id: "Game_CutSceneScriptCmd_SetStateLoading",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_ActivateExtraWorld: Binding = Binding {
        id: "Game_CutSceneScript_ActivateExtraWorld",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_ActivatePrimaryWorld: Binding = Binding {
        id: "Game_CutSceneScript_ActivatePrimaryWorld",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_AddDriverBodyToAssetGroupPaths: Binding = Binding {
        id: "Game_CutSceneScript_AddDriverBodyToAssetGroupPaths",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_AddDriverBodyToAssetGroupPaths_2: Binding = Binding {
        id: "Game_CutSceneScript_AddDriverBodyToAssetGroupPaths_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_AdvanceStartTimeOffset: Binding = Binding {
        id: "Game_CutSceneScript_AdvanceStartTimeOffset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_ApplyDriverPartOptions: Binding = Binding {
        id: "Game_CutSceneScript_ApplyDriverPartOptions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_DeactivatePrimaryWorld: Binding = Binding {
        id: "Game_CutSceneScript_DeactivatePrimaryWorld",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_Initialise: Binding = Binding {
        id: "Game_CutSceneScript_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_LoadAssetGroups: Binding = Binding {
        id: "Game_CutSceneScript_LoadAssetGroups",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_ResetStartTimeOffset: Binding = Binding {
        id: "Game_CutSceneScript_ResetStartTimeOffset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_ResolveDriverOutfitAssets: Binding = Binding {
        id: "Game_CutSceneScript_ResolveDriverOutfitAssets",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_RunCommand: Binding = Binding {
        id: "Game_CutSceneScript_RunCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_SetGarageLocation: Binding = Binding {
        id: "Game_CutSceneScript_SetGarageLocation",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_SetGridLocation: Binding = Binding {
        id: "Game_CutSceneScript_SetGridLocation",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_SetStateFinished: Binding = Binding {
        id: "Game_CutSceneScript_SetStateFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_SpawnDynamicDriverActor: Binding = Binding {
        id: "Game_CutSceneScript_SpawnDynamicDriverActor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_SpawnGarageGantry: Binding = Binding {
        id: "Game_CutSceneScript_SpawnGarageGantry",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_SpawnLeadEngineer: Binding = Binding {
        id: "Game_CutSceneScript_SpawnLeadEngineer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutSceneScript_StopWorldNotification: Binding = Binding {
        id: "Game_CutSceneScript_StopWorldNotification",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutsceneContext_ParseSceneContext: Binding = Binding {
        id: "Game_CutsceneContext_ParseSceneContext",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutsceneManager_IsPreparingScenes: Binding = Binding {
        id: "Game_CutsceneManager_IsPreparingScenes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutsceneManager_SyncAssetGroups: Binding = Binding {
        id: "Game_CutsceneManager_SyncAssetGroups",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_CutsceneManager_WaitTextureStreaming: Binding = Binding {
        id: "Game_CutsceneManager_WaitTextureStreaming",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DRS_OnSessionLoad: Binding = Binding {
        id: "Game_DRS_OnSessionLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_CreateComponents: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_CreateComponents",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_Destructor: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_Destructor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_FireEvent: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_FireEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_IsPreloadSeqDataComplete: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_IsPreloadSeqDataComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_IsStateUnloaded: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_IsStateUnloaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_OnShutDownComplete: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_OnShutDownComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_PostInitialise: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_PostInitialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_SetStateStop: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_SetStateStop",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataDrivenCutSceneScript_Shutdown: Binding = Binding {
        id: "Game_DataDrivenCutSceneScript_Shutdown",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DataGathererPlayers_Update: Binding = Binding {
        id: "Game_DataGathererPlayers_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DecalFileDeleteOp_DoJob: Binding = Binding {
        id: "Game_DecalFileDeleteOp_DoJob",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DecalFileLoadOp_OnLoaded: Binding = Binding {
        id: "Game_DecalFileLoadOp_OnLoaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DecalGeneration_CacheMaterialHandle: Binding = Binding {
        id: "Game_DecalGeneration_CacheMaterialHandle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DecalGeneration_ReadBackCompressed: Binding = Binding {
        id: "Game_DecalGeneration_ReadBackCompressed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DecalGeneration_SetDecalComponent: Binding = Binding {
        id: "Game_DecalGeneration_SetDecalComponent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DeptEventViewModel_Update: Binding = Binding {
        id: "Game_DeptEventViewModel_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverCareer_SpawnDriverPair: Binding = Binding {
        id: "Game_DriverCareer_SpawnDriverPair",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverCareer_TeamMateSelectSpawn: Binding = Binding {
        id: "Game_DriverCareer_TeamMateSelectSpawn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverContracts_AdvanceSecretMeeting: Binding = Binding {
        id: "Game_DriverContracts_AdvanceSecretMeeting",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverContracts_BuildOffers: Binding = Binding {
        id: "Game_DriverContracts_BuildOffers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverContracts_OfferSecretMeeting: Binding = Binding {
        id: "Game_DriverContracts_OfferSecretMeeting",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverCustomisationReplicationManager_ReplaceCustomisation: Binding = Binding {
        id: "Game_DriverCustomisationReplicationManager_ReplaceCustomisation",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverCustomisationReplicationManager_ReplaceCustomisatn_2: Binding = Binding {
        id: "Game_DriverCustomisationReplicationManager_ReplaceCustomisatn_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverHqAgentIntroTransition_Initialise: Binding = Binding {
        id: "Game_DriverHqAgentIntroTransition_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverHqContractsTransition_Initialise: Binding = Binding {
        id: "Game_DriverHqContractsTransition_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverInCarIdle03_Initialise: Binding = Binding {
        id: "Game_DriverInCarIdle03_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverInCarIdle05_Initialise: Binding = Binding {
        id: "Game_DriverInCarIdle05_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverProfileTabVM_OnTabActivated: Binding = Binding {
        id: "Game_DriverProfileTabVM_OnTabActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverProfileTabVM_OnTabDeactivated: Binding = Binding {
        id: "Game_DriverProfileTabVM_OnTabDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverSatInCar06_PreStart: Binding = Binding {
        id: "Game_DriverSatInCar06_PreStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverSelectGridVM_OnActivate: Binding = Binding {
        id: "Game_DriverSelectGridVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverSelectGridVM_OnDeactivate: Binding = Binding {
        id: "Game_DriverSelectGridVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverSelectHeader_SetTabState: Binding = Binding {
        id: "Game_DriverSelectHeader_SetTabState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverTalkToEngineer02_Initialise: Binding = Binding {
        id: "Game_DriverTalkToEngineer02_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DriverTalksToPress_Initialise: Binding = Binding {
        id: "Game_DriverTalksToPress_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjAchievePosition_Activate: Binding = Binding {
        id: "Game_DynObjAchievePosition_Activate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjAchievePosition_UpdateStatus: Binding = Binding {
        id: "Game_DynObjAchievePosition_UpdateStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjCriterion_UpdateGapAhead: Binding = Binding {
        id: "Game_DynObjCriterion_UpdateGapAhead",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjCriterion_UpdateGapBehind: Binding = Binding {
        id: "Game_DynObjCriterion_UpdateGapBehind",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjCriterion_UpdateGapCurve: Binding = Binding {
        id: "Game_DynObjCriterion_UpdateGapCurve",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjCriterion_UpdatePositionCurve: Binding = Binding {
        id: "Game_DynObjCriterion_UpdatePositionCurve",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjDefendPosition_AccumulateTime: Binding = Binding {
        id: "Game_DynObjDefendPosition_AccumulateTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjDefendPosition_CheckRivalChange: Binding = Binding {
        id: "Game_DynObjDefendPosition_CheckRivalChange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjDefendPosition_Update: Binding = Binding {
        id: "Game_DynObjDefendPosition_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjGapCurve_Evaluate: Binding = Binding {
        id: "Game_DynObjGapCurve_Evaluate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynObjGapCurve_Update: Binding = Binding {
        id: "Game_DynObjGapCurve_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynamicObjective_GetLapsLeft: Binding = Binding {
        id: "Game_DynamicObjective_GetLapsLeft",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_DynamicObjective_TryActivatePosition: Binding = Binding {
        id: "Game_DynamicObjective_TryActivatePosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EALoginTask_OnStart: Binding = Binding {
        id: "Game_EALoginTask_OnStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EALoginTask_Update: Binding = Binding {
        id: "Game_EALoginTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EngineMgmt_AddGridPenalty: Binding = Binding {
        id: "Game_EngineMgmt_AddGridPenalty",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EngineMgmt_ApplySessionWear: Binding = Binding {
        id: "Game_EngineMgmt_ApplySessionWear",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EngineMgmt_ClearRaceGridPenalties: Binding = Binding {
        id: "Game_EngineMgmt_ClearRaceGridPenalties",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EngineMgmt_OnRaceSessionEnded: Binding = Binding {
        id: "Game_EngineMgmt_OnRaceSessionEnded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EngineMgmt_SyncGridPenaltyLog: Binding = Binding {
        id: "Game_EngineMgmt_SyncGridPenaltyLog",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsManager_ClaimFreeEntrant: Binding = Binding {
        id: "Game_EntrantsManager_ClaimFreeEntrant",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsManager_HandleCustomSELiveryReplace: Binding = Binding {
        id: "Game_EntrantsManager_HandleCustomSELiveryReplace",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsManager_OnMutableRoleChanged: Binding = Binding {
        id: "Game_EntrantsManager_OnMutableRoleChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsManager_RegisterEntrant: Binding = Binding {
        id: "Game_EntrantsManager_RegisterEntrant",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsManager_ResolveAndCloseDocument: Binding = Binding {
        id: "Game_EntrantsManager_ResolveAndCloseDocument",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsManager_ResolveEntrants: Binding = Binding {
        id: "Game_EntrantsManager_ResolveEntrants",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsUi_BuildCharacterRef: Binding = Binding {
        id: "Game_EntrantsUi_BuildCharacterRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EntrantsUi_OnCharacterListLoaded: Binding = Binding {
        id: "Game_EntrantsUi_OnCharacterListLoaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EventDispatcher_SendPositionEvent: Binding = Binding {
        id: "Game_EventDispatcher_SendPositionEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EventObjectives_OnTargetPositionDone: Binding = Binding {
        id: "Game_EventObjectives_OnTargetPositionDone",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_EventObjectives_ShowTargetPosition: Binding = Binding {
        id: "Game_EventObjectives_ShowTargetPosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ExcitementEventPair_TraceLog: Binding = Binding {
        id: "Game_ExcitementEventPair_TraceLog",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1CGRuleset_ForceFinishAll: Binding = Binding {
        id: "Game_F1CGRuleset_ForceFinishAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1CGRuleset_OnVehicleStatusRequest: Binding = Binding {
        id: "Game_F1CGRuleset_OnVehicleStatusRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WLoadout_Initialise: Binding = Binding {
        id: "Game_F1WLoadout_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldActivityContainer_GoHome: Binding = Binding {
        id: "Game_F1WorldActivityContainer_GoHome",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldActivityContainer_OnExit: Binding = Binding {
        id: "Game_F1WorldActivityContainer_OnExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldActivityContainer_SelectEntrantList: Binding = Binding {
        id: "Game_F1WorldActivityContainer_SelectEntrantList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldGoals_HandleCommand: Binding = Binding {
        id: "Game_F1WorldGoals_HandleCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldGoals_OnTabActivated: Binding = Binding {
        id: "Game_F1WorldGoals_OnTabActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldGoals_OnTabDeactivated: Binding = Binding {
        id: "Game_F1WorldGoals_OnTabDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldGoals_PopulateGoalList: Binding = Binding {
        id: "Game_F1WorldGoals_PopulateGoalList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldHub_BuildPlayTiles: Binding = Binding {
        id: "Game_F1WorldHub_BuildPlayTiles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldHub_OnPopupResult: Binding = Binding {
        id: "Game_F1WorldHub_OnPopupResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldHub_RestoreLastActivity: Binding = Binding {
        id: "Game_F1WorldHub_RestoreLastActivity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldHub_RouteToActivity: Binding = Binding {
        id: "Game_F1WorldHub_RouteToActivity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldHub_SelectTab: Binding = Binding {
        id: "Game_F1WorldHub_SelectTab",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldRewards_PopulateXpScreen: Binding = Binding {
        id: "Game_F1WorldRewards_PopulateXpScreen",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldScreenVM_Update: Binding = Binding {
        id: "Game_F1WorldScreenVM_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldScreen_NavigateEp1_2: Binding = Binding {
        id: "Game_F1WorldScreen_NavigateEp1_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldScreen_NavigateEp1_4: Binding = Binding {
        id: "Game_F1WorldScreen_NavigateEp1_4",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1WorldScreen_RefreshAndNavigate: Binding = Binding {
        id: "Game_F1WorldScreen_RefreshAndNavigate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1World_NavigateWithEntryPoint1: Binding = Binding {
        id: "Game_F1World_NavigateWithEntryPoint1",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osHttpTask_Ctor_Vt14583cd40: Binding = Binding {
        id: "Game_F1osHttpTask_Ctor_Vt14583cd40",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_CollectResults_Vt145835c00: Binding = Binding {
        id: "Game_F1osTask_CollectResults_Vt145835c00",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_Ctor_Vt145835090: Binding = Binding {
        id: "Game_F1osTask_Ctor_Vt145835090",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_FinishIfNoPending_GetFriendsList: Binding = Binding {
        id: "Game_F1osTask_FinishIfNoPending_GetFriendsList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_FinishLocked_PlayerSearch: Binding = Binding {
        id: "Game_F1osTask_FinishLocked_PlayerSearch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_OnChildFinished_Vt145835c00: Binding = Binding {
        id: "Game_F1osTask_OnChildFinished_Vt145835c00",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_OnJoinFailed_Vt145832400: Binding = Binding {
        id: "Game_F1osTask_OnJoinFailed_Vt145832400",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_OnResultSetFinished_Vt145832400: Binding = Binding {
        id: "Game_F1osTask_OnResultSetFinished_Vt145832400",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_ProcessFriendsReply_Vt145834f90: Binding = Binding {
        id: "Game_F1osTask_ProcessFriendsReply_Vt145834f90",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_RegisterEntrant_Vt1458323b0: Binding = Binding {
        id: "Game_F1osTask_RegisterEntrant_Vt1458323b0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_ResolveNames_GetFriendsList: Binding = Binding {
        id: "Game_F1osTask_ResolveNames_GetFriendsList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_SetFinishedReleaseRef: Binding = Binding {
        id: "Game_F1osTask_SetFinishedReleaseRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_SetFinished_B: Binding = Binding {
        id: "Game_F1osTask_SetFinished_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_SpawnChild_Vt1458323b0: Binding = Binding {
        id: "Game_F1osTask_SpawnChild_Vt1458323b0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTask_SpawnChild_Vt145835c00: Binding = Binding {
        id: "Game_F1osTask_SpawnChild_Vt145835c00",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_FinalisePackagePTask: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_FinalisePackagePTask",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_GetPPackagesData: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_GetPPackagesData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_HoldingPageHint: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_HoldingPageHint",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_PurchaseVIP: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_PurchaseVIP",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_RequestPackagePTask: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_RequestPackagePTask",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt145834f90: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt145834f90",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt145835090: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt145835090",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt1458352e0: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt1458352e0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt145835490: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt145835490",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt145835780: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt145835780",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt145835ba0: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt145835ba0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt14583c2a0: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt14583c2a0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_CreateTask_Vt14583d000: Binding = Binding {
        id: "Game_F1osTasksystem_CreateTask_Vt14583d000",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1osTasksystem_Update: Binding = Binding {
        id: "Game_F1osTasksystem_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateHttpRequest: Binding = Binding {
        id: "Game_F1os_CreateHttpRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateOnlineServiceTask: Binding = Binding {
        id: "Game_F1os_CreateOnlineServiceTask",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateTask_GetFriendsLeaderboard: Binding = Binding {
        id: "Game_F1os_CreateTask_GetFriendsLeaderboard",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateTask_GetLeaderboard: Binding = Binding {
        id: "Game_F1os_CreateTask_GetLeaderboard",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateTask_Login: Binding = Binding {
        id: "Game_F1os_CreateTask_Login",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateTask_ReconcileUserEntitlements: Binding = Binding {
        id: "Game_F1os_CreateTask_ReconcileUserEntitlements",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateTask_RefreshSessionToken: Binding = Binding {
        id: "Game_F1os_CreateTask_RefreshSessionToken",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_CreateTask_RequestPackageP: Binding = Binding {
        id: "Game_F1os_CreateTask_RequestPackageP",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_Create_CompleteEvent: Binding = Binding {
        id: "Game_F1os_Create_CompleteEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_Create_CraftBlueprint: Binding = Binding {
        id: "Game_F1os_Create_CraftBlueprint",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_Create_GetCCLeaderboard: Binding = Binding {
        id: "Game_F1os_Create_GetCCLeaderboard",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_Create_RemoveMember: Binding = Binding {
        id: "Game_F1os_Create_RemoveMember",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_Create_SessionBrowserSearchSessions: Binding = Binding {
        id: "Game_F1os_Create_SessionBrowserSearchSessions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_Create_SetPlayerInfo: Binding = Binding {
        id: "Game_F1os_Create_SetPlayerInfo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_Create_StartEvent: Binding = Binding {
        id: "Game_F1os_Create_StartEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_DestroyOnlineServiceTaskBase: Binding = Binding {
        id: "Game_F1os_DestroyOnlineServiceTaskBase",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_DestroyTaskBody_GetFriendsLeaderboard: Binding = Binding {
        id: "Game_F1os_DestroyTaskBody_GetFriendsLeaderboard",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1CGTask_CheckLogout: Binding = Binding {
        id: "Game_F1os_F1CGTask_CheckLogout",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1CGTask_OnRequestResult: Binding = Binding {
        id: "Game_F1os_F1CGTask_OnRequestResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1CGTask_Reset: Binding = Binding {
        id: "Game_F1os_F1CGTask_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1CGTask_Update: Binding = Binding {
        id: "Game_F1os_F1CGTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1WorldTask_CheckLogout: Binding = Binding {
        id: "Game_F1os_F1WorldTask_CheckLogout",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1WorldTask_OnRequestResult: Binding = Binding {
        id: "Game_F1os_F1WorldTask_OnRequestResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1WorldTask_Reset: Binding = Binding {
        id: "Game_F1os_F1WorldTask_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_F1WorldTask_Update: Binding = Binding {
        id: "Game_F1os_F1WorldTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogF: Binding = Binding {
        id: "Game_F1os_LogF",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_ClaimRewards: Binding = Binding {
        id: "Game_F1os_LogLargePayload_ClaimRewards",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_GetActiveChallengeCareer: Binding = Binding {
        id: "Game_F1os_LogLargePayload_GetActiveChallengeCareer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_GetCalendar: Binding = Binding {
        id: "Game_F1os_LogLargePayload_GetCalendar",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_GetCareerBackupData: Binding = Binding {
        id: "Game_F1os_LogLargePayload_GetCareerBackupData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_GetCompendiumRewards: Binding = Binding {
        id: "Game_F1os_LogLargePayload_GetCompendiumRewards",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_GetInbox: Binding = Binding {
        id: "Game_F1os_LogLargePayload_GetInbox",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_GetStorefront: Binding = Binding {
        id: "Game_F1os_LogLargePayload_GetStorefront",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_SessionBrowserGetSessionByID: Binding = Binding {
        id: "Game_F1os_LogLargePayload_SessionBrowserGetSessionByID",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_SessionBrowserSearchSessions: Binding = Binding {
        id: "Game_F1os_LogLargePayload_SessionBrowserSearchSessions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1os_LogLargePayload_UpgradeCard: Binding = Binding {
        id: "Game_F1os_LogLargePayload_UpgradeCard",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_F1wAction_Destroy_EmptyAction: Binding = Binding {
        id: "Game_F1wAction_Destroy_EmptyAction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FakeOption_GetMaxIndex: Binding = Binding {
        id: "Game_FakeOption_GetMaxIndex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FakeOption_SetValue: Binding = Binding {
        id: "Game_FakeOption_SetValue",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FanZoneSeasonResults_Exit: Binding = Binding {
        id: "Game_FanZoneSeasonResults_Exit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FanZoneStandingsVM_OnScreenDeactivated: Binding = Binding {
        id: "Game_FanZoneStandingsVM_OnScreenDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FanZoneStandingsVM_OnSkip: Binding = Binding {
        id: "Game_FanZoneStandingsVM_OnSkip",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FanZone_BuildStandingsDriverList: Binding = Binding {
        id: "Game_FanZone_BuildStandingsDriverList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FanzonesOnboardingVM_FadeIn: Binding = Binding {
        id: "Game_FanzonesOnboardingVM_FadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FlowState_EnterBootLogout: Binding = Binding {
        id: "Game_FlowState_EnterBootLogout",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FormationLapPresentation_BuildTimeline: Binding = Binding {
        id: "Game_FormationLapPresentation_BuildTimeline",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FreeLookCameraModifier_Update: Binding = Binding {
        id: "Game_FreeLookCameraModifier_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FriendsList_OnGetFriendsDone: Binding = Binding {
        id: "Game_FriendsList_OnGetFriendsDone",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontEnd_CloseFadeOut: Binding = Binding {
        id: "Game_FrontEnd_CloseFadeOut",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendLobbyVM_OnScreenActivated: Binding = Binding {
        id: "Game_FrontendLobbyVM_OnScreenActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCameraManager_GetSequencePosition: Binding = Binding {
        id: "Game_FrontendSceneCameraManager_GetSequencePosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCameraManager_SetActiveCamera: Binding = Binding {
        id: "Game_FrontendSceneCameraManager_SetActiveCamera",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCameraManager_Update: Binding = Binding {
        id: "Game_FrontendSceneCameraManager_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCarManager_SetCarOverride: Binding = Binding {
        id: "Game_FrontendSceneCarManager_SetCarOverride",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCarManager_SpawnCarAtPlacement: Binding = Binding {
        id: "Game_FrontendSceneCarManager_SpawnCarAtPlacement",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCarManager_SpawnCarAtTransform: Binding = Binding {
        id: "Game_FrontendSceneCarManager_SpawnCarAtTransform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCar_SetCustomDecalNodes: Binding = Binding {
        id: "Game_FrontendSceneCar_SetCustomDecalNodes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCar_SetCustomLivery: Binding = Binding {
        id: "Game_FrontendSceneCar_SetCustomLivery",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCar_SetVisibilityInternal: Binding = Binding {
        id: "Game_FrontendSceneCar_SetVisibilityInternal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCar_SetupComponents: Binding = Binding {
        id: "Game_FrontendSceneCar_SetupComponents",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCar_SpawnFromLiveryData: Binding = Binding {
        id: "Game_FrontendSceneCar_SpawnFromLiveryData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCar_SpawnFromLiveryData2: Binding = Binding {
        id: "Game_FrontendSceneCar_SpawnFromLiveryData2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentDriver_ChangeDriver: Binding = Binding {
        id: "Game_FrontendSceneComponentDriver_ChangeDriver",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentDriver_NeedsRespawn: Binding = Binding {
        id: "Game_FrontendSceneComponentDriver_NeedsRespawn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentDriver_SetAnimOverlay: Binding = Binding {
        id: "Game_FrontendSceneComponentDriver_SetAnimOverlay",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentDriver_Spawn: Binding = Binding {
        id: "Game_FrontendSceneComponentDriver_Spawn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentDriver_UpdateDecals: Binding = Binding {
        id: "Game_FrontendSceneComponentDriver_UpdateDecals",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentObject_DespawnComplete: Binding = Binding {
        id: "Game_FrontendSceneComponentObject_DespawnComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentObject_LogState: Binding = Binding {
        id: "Game_FrontendSceneComponentObject_LogState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentObject_NeedsRespawn: Binding = Binding {
        id: "Game_FrontendSceneComponentObject_NeedsRespawn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentObject_SetVisible: Binding = Binding {
        id: "Game_FrontendSceneComponentObject_SetVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponentObject_Spawn: Binding = Binding {
        id: "Game_FrontendSceneComponentObject_Spawn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneComponent_StopDriverAnimOverlay: Binding = Binding {
        id: "Game_FrontendSceneComponent_StopDriverAnimOverlay",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCustomObject_SpawnObject: Binding = Binding {
        id: "Game_FrontendSceneCustomObject_SpawnObject",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCustomObject_Update: Binding = Binding {
        id: "Game_FrontendSceneCustomObject_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_OnSubsceneChanged: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_OnSubsceneChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_PlayCutscene: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_PlayCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_PreInitCutscene: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_PreInitCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_PreInitCutscenes: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_PreInitCutscenes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_PreloadCutscene: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_PreloadCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_PreloadSubsceneCutscenes: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_PreloadSubsceneCutscenes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_StopActiveCutscene: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_StopActiveCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_StopAllCutscenes: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_StopAllCutscenes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_StopCutsceneEntry: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_StopCutsceneEntry",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneCutsceneManager_Update: Binding = Binding {
        id: "Game_FrontendSceneCutsceneManager_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_BindActorsToSequence: Binding = Binding {
        id: "Game_FrontendSceneDriver_BindActorsToSequence",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_ChooseSequence: Binding = Binding {
        id: "Game_FrontendSceneDriver_ChooseSequence",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_Despawn: Binding = Binding {
        id: "Game_FrontendSceneDriver_Despawn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_FillHelmetPreviewData: Binding = Binding {
        id: "Game_FrontendSceneDriver_FillHelmetPreviewData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_SetPartsVisible: Binding = Binding {
        id: "Game_FrontendSceneDriver_SetPartsVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_SpawnActors: Binding = Binding {
        id: "Game_FrontendSceneDriver_SpawnActors",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_SpawnActorsAlt: Binding = Binding {
        id: "Game_FrontendSceneDriver_SpawnActorsAlt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_Update: Binding = Binding {
        id: "Game_FrontendSceneDriver_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_UpdatePfx: Binding = Binding {
        id: "Game_FrontendSceneDriver_UpdatePfx",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneDriver_UpdateTransform: Binding = Binding {
        id: "Game_FrontendSceneDriver_UpdateTransform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_GetDriverHelmetData: Binding = Binding {
        id: "Game_FrontendSceneManager_GetDriverHelmetData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_ProcessPendingCarChanges: Binding = Binding {
        id: "Game_FrontendSceneManager_ProcessPendingCarChanges",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_ProcessPendingDriverChanges: Binding = Binding {
        id: "Game_FrontendSceneManager_ProcessPendingDriverChanges",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_ResetComponents: Binding = Binding {
        id: "Game_FrontendSceneManager_ResetComponents",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetCarVisible: Binding = Binding {
        id: "Game_FrontendSceneManager_SetCarVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetDriverAnimationOverlay: Binding = Binding {
        id: "Game_FrontendSceneManager_SetDriverAnimationOverlay",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetDriverVisible: Binding = Binding {
        id: "Game_FrontendSceneManager_SetDriverVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetDriverVisibleById: Binding = Binding {
        id: "Game_FrontendSceneManager_SetDriverVisibleById",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetObjectVisible: Binding = Binding {
        id: "Game_FrontendSceneManager_SetObjectVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetPendingSwitch: Binding = Binding {
        id: "Game_FrontendSceneManager_SetPendingSwitch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetupCarInternal: Binding = Binding {
        id: "Game_FrontendSceneManager_SetupCarInternal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetupDriverInternal: Binding = Binding {
        id: "Game_FrontendSceneManager_SetupDriverInternal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SetupObjectInternal: Binding = Binding {
        id: "Game_FrontendSceneManager_SetupObjectInternal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_StopCutscene: Binding = Binding {
        id: "Game_FrontendSceneManager_StopCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SwitchScene: Binding = Binding {
        id: "Game_FrontendSceneManager_SwitchScene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneManager_SwitchSceneFromPreset: Binding = Binding {
        id: "Game_FrontendSceneManager_SwitchSceneFromPreset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneObject_RespawnAll: Binding = Binding {
        id: "Game_FrontendSceneObject_RespawnAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendScenePreview_Refresh: Binding = Binding {
        id: "Game_FrontendScenePreview_Refresh",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendScenePreview_SelectEntry: Binding = Binding {
        id: "Game_FrontendScenePreview_SelectEntry",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendScenePreview_UpdateTick: Binding = Binding {
        id: "Game_FrontendScenePreview_UpdateTick",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneScreenF_Destructor: Binding = Binding {
        id: "Game_FrontendSceneScreenF_Destructor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneScreen_Destructor: Binding = Binding {
        id: "Game_FrontendSceneScreen_Destructor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneScript_AddFzMultiActorGroup: Binding = Binding {
        id: "Game_FrontendSceneScript_AddFzMultiActorGroup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneScript_AddToMultiActorAnim: Binding = Binding {
        id: "Game_FrontendSceneScript_AddToMultiActorAnim",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneScript_RemoveFzMultiActorGroup: Binding = Binding {
        id: "Game_FrontendSceneScript_RemoveFzMultiActorGroup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneScript_SpawnSequenceForActor: Binding = Binding {
        id: "Game_FrontendSceneScript_SpawnSequenceForActor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneScript_SpawnSequences: Binding = Binding {
        id: "Game_FrontendSceneScript_SpawnSequences",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneSystem_ChangeMainCamera: Binding = Binding {
        id: "Game_FrontendSceneSystem_ChangeMainCamera",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneSystem_PreloadScene: Binding = Binding {
        id: "Game_FrontendSceneSystem_PreloadScene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneSystem_ProcessCommand: Binding = Binding {
        id: "Game_FrontendSceneSystem_ProcessCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneSystem_SetSubscene: Binding = Binding {
        id: "Game_FrontendSceneSystem_SetSubscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FrontendSceneUi_SetMappedDriverVisible: Binding = Binding {
        id: "Game_FrontendSceneUi_SetMappedDriverVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FuelDynamicObjective_Activate: Binding = Binding {
        id: "Game_FuelDynamicObjective_Activate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_FuelDynamicObjective_Activate_2: Binding = Binding {
        id: "Game_FuelDynamicObjective_Activate_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameModes_BuildMode: Binding = Binding {
        id: "Game_GameModes_BuildMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameReplaySystem_SubscribeRewindTutorial: Binding = Binding {
        id: "Game_GameReplaySystem_SubscribeRewindTutorial",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionConfig_Destroy: Binding = Binding {
        id: "Game_GameSessionConfig_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionManager_Connect: Binding = Binding {
        id: "Game_GameSessionManager_Connect",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionManager_Destroy: Binding = Binding {
        id: "Game_GameSessionManager_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionManager_ReportJoinError: Binding = Binding {
        id: "Game_GameSessionManager_ReportJoinError",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingCareer_CreateSession: Binding = Binding {
        id: "Game_GameSessionMatchingCareer_CreateSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingCareer_JoinSession: Binding = Binding {
        id: "Game_GameSessionMatchingCareer_JoinSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingCareer_OnCreateSessionComplete: Binding = Binding {
        id: "Game_GameSessionMatchingCareer_OnCreateSessionComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingCustom_SearchSession: Binding = Binding {
        id: "Game_GameSessionMatchingCustom_SearchSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingCustom_SearchSessionWithPresence: Binding = Binding {
        id: "Game_GameSessionMatchingCustom_SearchSessionWithPresence",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingF1WSocialGP_Cancel: Binding = Binding {
        id: "Game_GameSessionMatchingF1WSocialGP_Cancel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingF1WSocialGP_CreateSession: Binding = Binding {
        id: "Game_GameSessionMatchingF1WSocialGP_CreateSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingF1WSocialGP_CreateSessionForMembers: Binding = Binding {
        id: "Game_GameSessionMatchingF1WSocialGP_CreateSessionForMembers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingF1WSocialGP_JoinSession: Binding = Binding {
        id: "Game_GameSessionMatchingF1WSocialGP_JoinSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingF1WSocialGP_OnCreateSessionComplete: Binding = Binding {
        id: "Game_GameSessionMatchingF1WSocialGP_OnCreateSessionComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatchingHopperCommon_OnSessionCreateComplete: Binding = Binding {
        id: "Game_GameSessionMatchingHopperCommon_OnSessionCreateComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMatching_ApplySessionAttributes: Binding = Binding {
        id: "Game_GameSessionMatching_ApplySessionAttributes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMigrationManager_ClearHaveResults: Binding = Binding {
        id: "Game_GameSessionMigrationManager_ClearHaveResults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMigrationManager_OnMutableRoleChanged: Binding = Binding {
        id: "Game_GameSessionMigrationManager_OnMutableRoleChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMigrationManager_SendSelfState: Binding = Binding {
        id: "Game_GameSessionMigrationManager_SendSelfState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionMigrationManager_SetPeerStatus: Binding = Binding {
        id: "Game_GameSessionMigrationManager_SetPeerStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GameSessionUI_Shutdown: Binding = Binding {
        id: "Game_GameSessionUI_Shutdown",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Game_OnRetireLocalPlayersMessage: Binding = Binding {
        id: "Game_Game_OnRetireLocalPlayersMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Game_SetCurrentGameMode: Binding = Binding {
        id: "Game_Game_SetCurrentGameMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GarageCrewInstance_PlayOverlaySequence: Binding = Binding {
        id: "Game_GarageCrewInstance_PlayOverlaySequence",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GarageCrewInstance_PlaySequence: Binding = Binding {
        id: "Game_GarageCrewInstance_PlaySequence",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GarageModule_Initialise: Binding = Binding {
        id: "Game_GarageModule_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GarageMonitorTvFeedVM_OnActivate: Binding = Binding {
        id: "Game_GarageMonitorTvFeedVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GarageMonitorTvFeedVM_OnDeactivate: Binding = Binding {
        id: "Game_GarageMonitorTvFeedVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GarageReturn_ComputeTimeSkip: Binding = Binding {
        id: "Game_GarageReturn_ComputeTimeSkip",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Garage_FadeInAfterReturn: Binding = Binding {
        id: "Game_Garage_FadeInAfterReturn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Garage_FadeOutForExit: Binding = Binding {
        id: "Game_Garage_FadeOutForExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Garage_LeaveGarageShowHud: Binding = Binding {
        id: "Game_Garage_LeaveGarageShowHud",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Garage_OnFadedOutInRace: Binding = Binding {
        id: "Game_Garage_OnFadedOutInRace",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Get_ModelNodeId: Binding = Binding {
        id: "Game_Get_ModelNodeId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GettingReadyToRace_Initialise: Binding = Binding {
        id: "Game_GettingReadyToRace_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_CancelTasks: Binding = Binding {
        id: "Game_GhostLapHandler_CancelTasks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_Ctor: Binding = Binding {
        id: "Game_GhostLapHandler_Ctor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_Dtor: Binding = Binding {
        id: "Game_GhostLapHandler_Dtor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_EvaluatePerformanceState: Binding = Binding {
        id: "Game_GhostLapHandler_EvaluatePerformanceState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_FreeSessionRivalStorage: Binding = Binding {
        id: "Game_GhostLapHandler_FreeSessionRivalStorage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_OnSessionRivalSearchCompleted: Binding = Binding {
        id: "Game_GhostLapHandler_OnSessionRivalSearchCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_StartLeaderboardEntryWrite: Binding = Binding {
        id: "Game_GhostLapHandler_StartLeaderboardEntryWrite",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_StartRemoteStorageDownload: Binding = Binding {
        id: "Game_GhostLapHandler_StartRemoteStorageDownload",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_StartRemoteStorageUpload: Binding = Binding {
        id: "Game_GhostLapHandler_StartRemoteStorageUpload",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapHandler_Update: Binding = Binding {
        id: "Game_GhostLapHandler_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapsStorageSystem_Connect: Binding = Binding {
        id: "Game_GhostLapsStorageSystem_Connect",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapsStorageSystem_Dtor: Binding = Binding {
        id: "Game_GhostLapsStorageSystem_Dtor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapsStorageSystem_OnSessionEnter: Binding = Binding {
        id: "Game_GhostLapsStorageSystem_OnSessionEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapsStorageSystem_OnSessionExit: Binding = Binding {
        id: "Game_GhostLapsStorageSystem_OnSessionExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostLapsStorageSystem_OnSessionRestart: Binding = Binding {
        id: "Game_GhostLapsStorageSystem_OnSessionRestart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostRivalFinder_Finish: Binding = Binding {
        id: "Game_GhostRivalFinder_Finish",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostRivalFinder_OnEnumerateRivalsCompleted: Binding = Binding {
        id: "Game_GhostRivalFinder_OnEnumerateRivalsCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GhostRivalFinder_SetState: Binding = Binding {
        id: "Game_GhostRivalFinder_SetState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridCutSceneScript_SetupDriverTalkToPress: Binding = Binding {
        id: "Game_GridCutSceneScript_SetupDriverTalkToPress",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridDriverAdjustGloves_Initialise: Binding = Binding {
        id: "Game_GridDriverAdjustGloves_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridDriverTalkToEngineer_Initialise: Binding = Binding {
        id: "Game_GridDriverTalkToEngineer_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridLeadEngineer_Initialise: Binding = Binding {
        id: "Game_GridLeadEngineer_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridPlayerShot_Initialise: Binding = Binding {
        id: "Game_GridPlayerShot_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridReveal_SetStateLoading: Binding = Binding {
        id: "Game_GridReveal_SetStateLoading",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridSafetyCarShotGscs_Initialise: Binding = Binding {
        id: "Game_GridSafetyCarShotGscs_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_GridSafetyCarShot_Initialise: Binding = Binding {
        id: "Game_GridSafetyCarShot_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_HandleUserEntitlementsUpdateTask_OnChildDone: Binding = Binding {
        id: "Game_HandleUserEntitlementsUpdateTask_OnChildDone",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Handle_GarageCrewEvent_Thunk: Binding = Binding {
        id: "Game_Handle_GarageCrewEvent_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Handle_SpecGarageCrewEvent_Thunk: Binding = Binding {
        id: "Game_Handle_SpecGarageCrewEvent_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_HomeScreen_OnHomeScreenInfoResult: Binding = Binding {
        id: "Game_HomeScreen_OnHomeScreenInfoResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_HomeScreen_RequestHomeScreenInfo: Binding = Binding {
        id: "Game_HomeScreen_RequestHomeScreenInfo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_HudDriverCardVM_OnBound: Binding = Binding {
        id: "Game_HudDriverCardVM_OnBound",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_HudDriverListVM_OnBound: Binding = Binding {
        id: "Game_HudDriverListVM_OnBound",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_HudFeaturedDriverVM_OnBound: Binding = Binding {
        id: "Game_HudFeaturedDriverVM_OnBound",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_HudFeaturedDriverVM_OnUnbound: Binding = Binding {
        id: "Game_HudFeaturedDriverVM_OnUnbound",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InCarDriver_EvaluateAnimLayers: Binding = Binding {
        id: "Game_InCarDriver_EvaluateAnimLayers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InGameMenuVM_OnReturnToGarage: Binding = Binding {
        id: "Game_InGameMenuVM_OnReturnToGarage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InboxTabVM_OnTabActivated: Binding = Binding {
        id: "Game_InboxTabVM_OnTabActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InboxTabVM_OnTabDeactivated: Binding = Binding {
        id: "Game_InboxTabVM_OnTabDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Input_OnControlSchemeAdded: Binding = Binding {
        id: "Game_Input_OnControlSchemeAdded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InviteManager_ChangeState: Binding = Binding {
        id: "Game_InviteManager_ChangeState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InviteManager_Join: Binding = Binding {
        id: "Game_InviteManager_Join",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InviteManager_OnCareerLoadedSoC: Binding = Binding {
        id: "Game_InviteManager_OnCareerLoadedSoC",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InviteManager_OnJoinCompleted: Binding = Binding {
        id: "Game_InviteManager_OnJoinCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_InviteManager_OnJoinCompleted_2: Binding = Binding {
        id: "Game_InviteManager_OnJoinCompleted_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_LapSectors_InitFromTrack: Binding = Binding {
        id: "Game_LapSectors_InitFromTrack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_LeaderboardRowVM_Populate: Binding = Binding {
        id: "Game_LeaderboardRowVM_Populate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_LiveryAssignVM_OnActivate: Binding = Binding {
        id: "Game_LiveryAssignVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_LiveryAssignVM_OnDeactivate: Binding = Binding {
        id: "Game_LiveryAssignVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_LoadingScreenVM_OnScreenActivated: Binding = Binding {
        id: "Game_LoadingScreenVM_OnScreenActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MatchingContext_OnPostRaceResultSyncEnter: Binding = Binding {
        id: "Game_MatchingContext_OnPostRaceResultSyncEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MatchingContext_OnSessionEnter: Binding = Binding {
        id: "Game_MatchingContext_OnSessionEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MatchingContext_OnSessionLobby: Binding = Binding {
        id: "Game_MatchingContext_OnSessionLobby",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MatchingContext_OnSessionRestart: Binding = Binding {
        id: "Game_MatchingContext_OnSessionRestart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MatchingContext_OnStartRematch: Binding = Binding {
        id: "Game_MatchingContext_OnStartRematch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MatchingContext_SetRankedSearchParams: Binding = Binding {
        id: "Game_MatchingContext_SetRankedSearchParams",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MatchingContext_SetupRankedGameSession: Binding = Binding {
        id: "Game_MatchingContext_SetupRankedGameSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MidSession_SaveReplaySnapshot: Binding = Binding {
        id: "Game_MidSession_SaveReplaySnapshot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ModelProxyList_Resolve: Binding = Binding {
        id: "Game_ModelProxyList_Resolve",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MpSeasonCompleteVM_OnActivate: Binding = Binding {
        id: "Game_MpSeasonCompleteVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MultiVotingManager_CastVote: Binding = Binding {
        id: "Game_MultiVotingManager_CastVote",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MultiVotingManager_HandleBallot: Binding = Binding {
        id: "Game_MultiVotingManager_HandleBallot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MultiVotingManager_HandleStatus: Binding = Binding {
        id: "Game_MultiVotingManager_HandleStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamContract_SetDriversVisible: Binding = Binding {
        id: "Game_MyTeamContract_SetDriversVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamContract_ShowDriverComparison: Binding = Binding {
        id: "Game_MyTeamContract_ShowDriverComparison",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamCustomisation_SpawnScene: Binding = Binding {
        id: "Game_MyTeamCustomisation_SpawnScene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamDriverMarket_Close: Binding = Binding {
        id: "Game_MyTeamDriverMarket_Close",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamDriverMarket_Open: Binding = Binding {
        id: "Game_MyTeamDriverMarket_Open",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamHq_SetupSceneForState: Binding = Binding {
        id: "Game_MyTeamHq_SetupSceneForState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamHq_SpawnFoyerCar: Binding = Binding {
        id: "Game_MyTeamHq_SpawnFoyerCar",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamHq_SpawnFoyerDrivers: Binding = Binding {
        id: "Game_MyTeamHq_SpawnFoyerDrivers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamIntroScript_PollAssetsLoaded: Binding = Binding {
        id: "Game_MyTeamIntroScript_PollAssetsLoaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_MyTeamIntroScript_PreStart: Binding = Binding {
        id: "Game_MyTeamIntroScript_PreStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetFormationLap_BroadcastAll: Binding = Binding {
        id: "Game_NetFormationLap_BroadcastAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetFormationLap_HandleSessionState: Binding = Binding {
        id: "Game_NetFormationLap_HandleSessionState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetFormationLap_SendToConnection: Binding = Binding {
        id: "Game_NetFormationLap_SendToConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetFormationLap_SetVehicleLocal: Binding = Binding {
        id: "Game_NetFormationLap_SetVehicleLocal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetFormationLap_Update: Binding = Binding {
        id: "Game_NetFormationLap_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetMarshalZones_HandleMessage: Binding = Binding {
        id: "Game_NetMarshalZones_HandleMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetMarshalZones_HostBroadcast: Binding = Binding {
        id: "Game_NetMarshalZones_HostBroadcast",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetMarshalZones_OnHostMigrated: Binding = Binding {
        id: "Game_NetMarshalZones_OnHostMigrated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetMarshalZones_Update: Binding = Binding {
        id: "Game_NetMarshalZones_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetPlayerSync_OnMutableRoleChanged: Binding = Binding {
        id: "Game_NetPlayerSync_OnMutableRoleChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetPlayerSync_OnPlayerSetEntryDiscarded: Binding = Binding {
        id: "Game_NetPlayerSync_OnPlayerSetEntryDiscarded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_SendCameraSelect: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_SendCameraSelect",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_SendCarsPassedPits: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_SendCarsPassedPits",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_SendIntroEnd: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_SendIntroEnd",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_SendLapsRemaining: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_SendLapsRemaining",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_SendNamedState: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_SendNamedState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_SendStart: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_SendStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_UpdateCameraSelect: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_UpdateCameraSelect",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_UpdateCarsPassedPits: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_UpdateCarsPassedPits",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_UpdateIntroEnd: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_UpdateIntroEnd",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_UpdateLapsRemaining: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_UpdateLapsRemaining",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_UpdateNamedState: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_UpdateNamedState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCarCinematic_UpdateStart: Binding = Binding {
        id: "Game_NetSafetyCarCinematic_UpdateStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetSafetyCar_UnregisterStale: Binding = Binding {
        id: "Game_NetSafetyCar_UnregisterStale",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetStartLights_HandleStateMessage: Binding = Binding {
        id: "Game_NetStartLights_HandleStateMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetStartLights_HandleTimeOffset: Binding = Binding {
        id: "Game_NetStartLights_HandleTimeOffset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetStartLights_OnRoleChanged: Binding = Binding {
        id: "Game_NetStartLights_OnRoleChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkReady_OnPauseSendSuperReady: Binding = Binding {
        id: "Game_NetworkReady_OnPauseSendSuperReady",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkReady_OnUnpauseSendSuperReady: Binding = Binding {
        id: "Game_NetworkReady_OnUnpauseSendSuperReady",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkReady_OnUnreadyMessage: Binding = Binding {
        id: "Game_NetworkReady_OnUnreadyMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkReady_SendSuperReady: Binding = Binding {
        id: "Game_NetworkReady_SendSuperReady",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkReady_SetRelevantSet: Binding = Binding {
        id: "Game_NetworkReady_SetRelevantSet",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkSubsystem_ChangeMPMode: Binding = Binding {
        id: "Game_NetworkSubsystem_ChangeMPMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkSubsystem_SelectTitleConfig: Binding = Binding {
        id: "Game_NetworkSubsystem_SelectTitleConfig",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkUI_StartRankedDivisionSearch: Binding = Binding {
        id: "Game_NetworkUI_StartRankedDivisionSearch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_NetworkUI_StartRankedRematching: Binding = Binding {
        id: "Game_NetworkUI_StartRankedRematching",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Network_ApplyInitialPitAndGridState: Binding = Binding {
        id: "Game_Network_ApplyInitialPitAndGridState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Network_CategoryOptionUsing: Binding = Binding {
        id: "Game_Network_CategoryOptionUsing",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Network_SendingRedFlagVehicleStatusUpdate: Binding = Binding {
        id: "Game_Network_SendingRedFlagVehicleStatusUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Network_SetScenario: Binding = Binding {
        id: "Game_Network_SetScenario",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Objective_CheckFinishPosition: Binding = Binding {
        id: "Game_Objective_CheckFinishPosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Objective_CheckOvertakeTarget: Binding = Binding {
        id: "Game_Objective_CheckOvertakeTarget",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Objective_CheckPositionTarget: Binding = Binding {
        id: "Game_Objective_CheckPositionTarget",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnMessageDriverPlayerChanged: Binding = Binding {
        id: "Game_OnMessageDriverPlayerChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineCareer_FadeOutForSessionRequest: Binding = Binding {
        id: "Game_OnlineCareer_FadeOutForSessionRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineCareer_OnFirstRequestResult: Binding = Binding {
        id: "Game_OnlineCareer_OnFirstRequestResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineCareer_OnSecondRequestResult: Binding = Binding {
        id: "Game_OnlineCareer_OnSecondRequestResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineDocumentComponent_Create: Binding = Binding {
        id: "Game_OnlineDocumentComponent_Create",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineDocumentSubComponent_Ctor: Binding = Binding {
        id: "Game_OnlineDocumentSubComponent_Ctor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineRole_HandleSessionDataRequest: Binding = Binding {
        id: "Game_OnlineRole_HandleSessionDataRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineRole_OnConnectionVerified: Binding = Binding {
        id: "Game_OnlineRole_OnConnectionVerified",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineServerLogInTask_FinishWithState: Binding = Binding {
        id: "Game_OnlineServerLogInTask_FinishWithState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineServerLogInTask_OnError: Binding = Binding {
        id: "Game_OnlineServerLogInTask_OnError",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineServerLogInTask_OnStart: Binding = Binding {
        id: "Game_OnlineServerLogInTask_OnStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineServerLogInTask_SetFinished: Binding = Binding {
        id: "Game_OnlineServerLogInTask_SetFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineServerLogInTask_SetFinishedReleaseRef: Binding = Binding {
        id: "Game_OnlineServerLogInTask_SetFinishedReleaseRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineServer_TriggerLogout: Binding = Binding {
        id: "Game_OnlineServer_TriggerLogout",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineSubsystemTaskOnServiceBase_Ctor: Binding = Binding {
        id: "Game_OnlineSubsystemTaskOnServiceBase_Ctor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineSystemEADPAsyncTask_Update: Binding = Binding {
        id: "Game_OnlineSystemEADPAsyncTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineSystemGetFriendRequestListTask_OnPlayerCards: Binding = Binding {
        id: "Game_OnlineSystemGetFriendRequestListTask_OnPlayerCards",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OnlineSystemGetFriendRequestListTask_Update: Binding = Binding {
        id: "Game_OnlineSystemGetFriendRequestListTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OptionsColourCorrectionVM_OnScreenActivated: Binding = Binding {
        id: "Game_OptionsColourCorrectionVM_OnScreenActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OptionsCrossplayVM_OnScreenActivated: Binding = Binding {
        id: "Game_OptionsCrossplayVM_OnScreenActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OptionsStore_FindOption: Binding = Binding {
        id: "Game_OptionsStore_FindOption",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OptionsSystem_ResolveOptionValue: Binding = Binding {
        id: "Game_OptionsSystem_ResolveOptionValue",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_OptionsUI_ApplyFilterLevelCollectChanges: Binding = Binding {
        id: "Game_OptionsUI_ApplyFilterLevelCollectChanges",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PagedBuffer_Transfer: Binding = Binding {
        id: "Game_PagedBuffer_Transfer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PagedBuffer_UsePageAt: Binding = Binding {
        id: "Game_PagedBuffer_UsePageAt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ParcFermeCutScene_Initialise: Binding = Binding {
        id: "Game_ParcFermeCutScene_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_BindMeshModel: Binding = Binding {
        id: "Game_Particles_BindMeshModel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_CreateEffectNode: Binding = Binding {
        id: "Game_Particles_CreateEffectNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_CreateStoredEffect: Binding = Binding {
        id: "Game_Particles_CreateStoredEffect",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_FindVehicleIndex: Binding = Binding {
        id: "Game_Particles_FindVehicleIndex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_RemoveEffect: Binding = Binding {
        id: "Game_Particles_RemoveEffect",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_RestoreLongLivedParticles: Binding = Binding {
        id: "Game_Particles_RestoreLongLivedParticles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_RestoreParticleSlot: Binding = Binding {
        id: "Game_Particles_RestoreParticleSlot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_StoreParticleSlot: Binding = Binding {
        id: "Game_Particles_StoreParticleSlot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Particles_UpdateEmitters: Binding = Binding {
        id: "Game_Particles_UpdateEmitters",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PelvisHeightModifier_Init: Binding = Binding {
        id: "Game_PelvisHeightModifier_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PhotoModeHelper_CaptureFrame: Binding = Binding {
        id: "Game_PhotoModeHelper_CaptureFrame",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PitBoxCarActor_Update: Binding = Binding {
        id: "Game_PitBoxCarActor_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PitCrewActor_Update: Binding = Binding {
        id: "Game_PitCrewActor_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PitDriveThrough_OnNoAnimFadeOut: Binding = Binding {
        id: "Game_PitDriveThrough_OnNoAnimFadeOut",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PitLane_MoveDownSpline: Binding = Binding {
        id: "Game_PitLane_MoveDownSpline",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PitLane_PutVehicleInPitBox: Binding = Binding {
        id: "Game_PitLane_PutVehicleInPitBox",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PitLane_VehicleEnteringInternal: Binding = Binding {
        id: "Game_PitLane_VehicleEnteringInternal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Pitlane_Multiplayer_teleportToGarage: Binding = Binding {
        id: "Game_Pitlane_Multiplayer_teleportToGarage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Pitlane_SetVehicleStatus: Binding = Binding {
        id: "Game_Pitlane_SetVehicleStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PitstopCutScene_Initialise: Binding = Binding {
        id: "Game_PitstopCutScene_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Place_Vehicle: Binding = Binding {
        id: "Game_Place_Vehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlatformAuthorisationTask_OnStart: Binding = Binding {
        id: "Game_PlatformAuthorisationTask_OnStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlatformAuthorisationTask_Update: Binding = Binding {
        id: "Game_PlatformAuthorisationTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerManager_HandleControllerUnplugged: Binding = Binding {
        id: "Game_PlayerManager_HandleControllerUnplugged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSetSubsystem_CreateHumanPlayers: Binding = Binding {
        id: "Game_PlayerSetSubsystem_CreateHumanPlayers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSetSubsystem_HandleUpdatePresenceNameMessage: Binding = Binding {
        id: "Game_PlayerSetSubsystem_HandleUpdatePresenceNameMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSetSubsystem_Init: Binding = Binding {
        id: "Game_PlayerSetSubsystem_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSetSubsystem_OnNewConnection: Binding = Binding {
        id: "Game_PlayerSetSubsystem_OnNewConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSetSubsystem_UpdatePresence: Binding = Binding {
        id: "Game_PlayerSetSubsystem_UpdatePresence",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSet_ChangePlayerConnectionId: Binding = Binding {
        id: "Game_PlayerSet_ChangePlayerConnectionId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSet_ChangePlayerIsWatching: Binding = Binding {
        id: "Game_PlayerSet_ChangePlayerIsWatching",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSet_SetEntriesPlaying: Binding = Binding {
        id: "Game_PlayerSet_SetEntriesPlaying",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PlayerSet_SetEntryLocked: Binding = Binding {
        id: "Game_PlayerSet_SetEntryLocked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Pod2019FirstScript_Initialise: Binding = Binding {
        id: "Game_Pod2019FirstScript_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Pod2019FirstScript_OnShutDownComplete: Binding = Binding {
        id: "Game_Pod2019FirstScript_OnShutDownComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Pod2019FirstScript_Shutdown: Binding = Binding {
        id: "Game_Pod2019FirstScript_Shutdown",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Pod2019First_ReleasePodiumThumbnails: Binding = Binding {
        id: "Game_Pod2019First_ReleasePodiumThumbnails",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Pod2019First_RequestPodiumThumbnails: Binding = Binding {
        id: "Game_Pod2019First_RequestPodiumThumbnails",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PodiumCarFiller_Initialise: Binding = Binding {
        id: "Game_PodiumCarFiller_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PodiumParcFermeScript_SetStateLoading: Binding = Binding {
        id: "Game_PodiumParcFermeScript_SetStateLoading",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PodiumParcFerme_Initialise: Binding = Binding {
        id: "Game_PodiumParcFerme_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PostraceReactionsD01Fail_Initialise: Binding = Binding {
        id: "Game_PostraceReactionsD01Fail_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PostraceReactionsD02Success_Initialise: Binding = Binding {
        id: "Game_PostraceReactionsD02Success_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PostraceReactionsD03Success_Initialise: Binding = Binding {
        id: "Game_PostraceReactionsD03Success_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PracticeProgrammeSystem_IsolatePlayer: Binding = Binding {
        id: "Game_PracticeProgrammeSystem_IsolatePlayer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PracticeProgrammeSystem_OnCompareVehicleChanged: Binding = Binding {
        id: "Game_PracticeProgrammeSystem_OnCompareVehicleChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PracticeProgrammeSystem_OnCompareVehicleChanged_2: Binding = Binding {
        id: "Game_PracticeProgrammeSystem_OnCompareVehicleChanged_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PracticeProgrammes_Init: Binding = Binding {
        id: "Game_PracticeProgrammes_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_PracticeTrackAcclimatisation_RefreshState: Binding = Binding {
        id: "Game_PracticeTrackAcclimatisation_RefreshState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Predict_StrategyWetness: Binding = Binding {
        id: "Game_Predict_StrategyWetness",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProLeaguesVM_OnExit: Binding = Binding {
        id: "Game_ProLeaguesVM_OnExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProLeaguesVM_OnLeaguesResult: Binding = Binding {
        id: "Game_ProLeaguesVM_OnLeaguesResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProfanityFilter_CheckWord: Binding = Binding {
        id: "Game_ProfanityFilter_CheckWord",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProfanityFilter_FilterText: Binding = Binding {
        id: "Game_ProfanityFilter_FilterText",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProfanityFilter_MatchList: Binding = Binding {
        id: "Game_ProfanityFilter_MatchList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProgressBasedFocusTowardsTrackCentreCameraModifier_OnEnable: Binding = Binding {
        id: "Game_ProgressBasedFocusTowardsTrackCentreCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProgressBasedRollCameraModifier_OnEnable: Binding = Binding {
        id: "Game_ProgressBasedRollCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProgressBasedStaticZoomCameraModifier_OnEnable: Binding = Binding {
        id: "Game_ProgressBasedStaticZoomCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProgressBasedTargetInhibitorCameraBehaviour_OnEnable: Binding = Binding {
        id: "Game_ProgressBasedTargetInhibitorCameraBehaviour_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProgressBasedTargetOffsetCameraModifier_OnEnable: Binding = Binding {
        id: "Game_ProgressBasedTargetOffsetCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProgressFloatInterpolator_Evaluate: Binding = Binding {
        id: "Game_ProgressFloatInterpolator_Evaluate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ProximityBasedShakeCameraModifier_OnEnable: Binding = Binding {
        id: "Game_ProximityBasedShakeCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_QualObjReaction01_Initialise: Binding = Binding {
        id: "Game_QualObjReaction01_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RaceHelper_GetVehicleByPosition: Binding = Binding {
        id: "Game_RaceHelper_GetVehicleByPosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RaceHelper_GetVehicleIdByChampPos: Binding = Binding {
        id: "Game_RaceHelper_GetVehicleIdByChampPos",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RaceHelper_GetVehicleIndexByChampPos: Binding = Binding {
        id: "Game_RaceHelper_GetVehicleIndexByChampPos",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RaceMarshalSystem_Update: Binding = Binding {
        id: "Game_RaceMarshalSystem_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RaceSession_FinaliseAndSetLapsPct: Binding = Binding {
        id: "Game_RaceSession_FinaliseAndSetLapsPct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RaceStrategyVm_Populate: Binding = Binding {
        id: "Game_RaceStrategyVm_Populate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RacingVehicles_Initialise: Binding = Binding {
        id: "Game_RacingVehicles_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RainfallSystem_Initialise: Binding = Binding {
        id: "Game_RainfallSystem_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RankedDivision_StartSearch: Binding = Binding {
        id: "Game_RankedDivision_StartSearch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RankedDivision_UpdateSearchAvailability: Binding = Binding {
        id: "Game_RankedDivision_UpdateSearchAvailability",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RankedMatchingProcessor_CreateSession: Binding = Binding {
        id: "Game_RankedMatchingProcessor_CreateSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RankedMatchingProcessor_Search: Binding = Binding {
        id: "Game_RankedMatchingProcessor_Search",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Refresh_PerformanceComparison: Binding = Binding {
        id: "Game_Refresh_PerformanceComparison",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Renderer_CreateRenderPasses: Binding = Binding {
        id: "Game_Renderer_CreateRenderPasses",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RepairTimes_FadeIn: Binding = Binding {
        id: "Game_RepairTimes_FadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RepairTimes_FadeOut: Binding = Binding {
        id: "Game_RepairTimes_FadeOut",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RepairTimes_FinishFadeIn: Binding = Binding {
        id: "Game_RepairTimes_FinishFadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RepairTimes_OnVehicleStatusFadeIn: Binding = Binding {
        id: "Game_RepairTimes_OnVehicleStatusFadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ReplayBuffer_WriteStats: Binding = Binding {
        id: "Game_ReplayBuffer_WriteStats",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ReplayFile_ReadFooter: Binding = Binding {
        id: "Game_ReplayFile_ReadFooter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ReplaySystem_LoadMidSessionData: Binding = Binding {
        id: "Game_ReplaySystem_LoadMidSessionData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ReplaySystem_LoadSnapshotFile: Binding = Binding {
        id: "Game_ReplaySystem_LoadSnapshotFile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ReplaySystem_SaveMidSessionData: Binding = Binding {
        id: "Game_ReplaySystem_SaveMidSessionData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Replay_LoadSavedReplayJob: Binding = Binding {
        id: "Game_Replay_LoadSavedReplayJob",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Replay_RestoreMainModuleBuffers: Binding = Binding {
        id: "Game_Replay_RestoreMainModuleBuffers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ResultsContainerVM_OnActivate: Binding = Binding {
        id: "Game_ResultsContainerVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ResultsContainerVM_OnDeactivate: Binding = Binding {
        id: "Game_ResultsContainerVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RetrieveConsumableEntitlementsTask_Ctor: Binding = Binding {
        id: "Game_RetrieveConsumableEntitlementsTask_Ctor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RetrieveConsumableEntitlementsTask_RequestSteamPrices: Binding = Binding {
        id: "Game_RetrieveConsumableEntitlementsTask_RequestSteamPrices",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RewardSequence_PlayCutscene: Binding = Binding {
        id: "Game_RewardSequence_PlayCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Rnd_CalcUpgradeTypeProgress: Binding = Binding {
        id: "Game_Rnd_CalcUpgradeTypeProgress",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Rnd_FillVehicleUpgrades: Binding = Binding {
        id: "Game_Rnd_FillVehicleUpgrades",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RttTaskManager_CleanupTask: Binding = Binding {
        id: "Game_RttTaskManager_CleanupTask",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RttTaskManager_OnTaskComplete: Binding = Binding {
        id: "Game_RttTaskManager_OnTaskComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RttTaskManager_ProcessCompletedTasks: Binding = Binding {
        id: "Game_RttTaskManager_ProcessCompletedTasks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RttTaskManager_QueueTask: Binding = Binding {
        id: "Game_RttTaskManager_QueueTask",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_RttTaskManager_StartTask: Binding = Binding {
        id: "Game_RttTaskManager_StartTask",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SaveData_DeserialiseContainer: Binding = Binding {
        id: "Game_SaveData_DeserialiseContainer",
        kind: "func",
        status: Status::Stable,
    };
    pub const Game_SaveData_SerialiseContainer: Binding = Binding {
        id: "Game_SaveData_SerialiseContainer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SaveData_SerialiseContainer_2: Binding = Binding {
        id: "Game_SaveData_SerialiseContainer_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SaveData_WriteBag: Binding = Binding {
        id: "Game_SaveData_WriteBag",
        kind: "func",
        status: Status::Stable,
    };
    pub const Game_SaveMgr_RequestProfileSave_Thunk: Binding = Binding {
        id: "Game_SaveMgr_RequestProfileSave_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SaveSlotManager_Deserialise: Binding = Binding {
        id: "Game_SaveSlotManager_Deserialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SaveSlotManager_SelectSlot: Binding = Binding {
        id: "Game_SaveSlotManager_SelectSlot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SceneManagerSpawn_MergeScene: Binding = Binding {
        id: "Game_SceneManagerSpawn_MergeScene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SceneManagerSpawn_SpawnScene: Binding = Binding {
        id: "Game_SceneManagerSpawn_SpawnScene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SceneManager_DestroyScene: Binding = Binding {
        id: "Game_SceneManager_DestroyScene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ScoreBreakdownVM_OnActivate: Binding = Binding {
        id: "Game_ScoreBreakdownVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ScoreBreakdownVM_OnDeactivate: Binding = Binding {
        id: "Game_ScoreBreakdownVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonDocument_CloseDocument: Binding = Binding {
        id: "Game_SeasonDocument_CloseDocument",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonDocument_HandleRebuildSeason: Binding = Binding {
        id: "Game_SeasonDocument_HandleRebuildSeason",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonDocument_SetTrackSetNameCustom: Binding = Binding {
        id: "Game_SeasonDocument_SetTrackSetNameCustom",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonLength_ApplyPreset: Binding = Binding {
        id: "Game_SeasonLength_ApplyPreset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonLength_PushCalendarEntry: Binding = Binding {
        id: "Game_SeasonLength_PushCalendarEntry",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonLobbyUi_SetTrackSetByMode: Binding = Binding {
        id: "Game_SeasonLobbyUi_SetTrackSetByMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonLobby_ResetTrackSet: Binding = Binding {
        id: "Game_SeasonLobby_ResetTrackSet",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SeasonUI_SetupChampionshipVariables: Binding = Binding {
        id: "Game_SeasonUI_SetupChampionshipVariables",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SecondWorld_RequestLoad: Binding = Binding {
        id: "Game_SecondWorld_RequestLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SecretMeetings_Initialise: Binding = Binding {
        id: "Game_SecretMeetings_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionHud_OnPlayerDroppedIn: Binding = Binding {
        id: "Game_SessionHud_OnPlayerDroppedIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionHud_OnSessionEnter: Binding = Binding {
        id: "Game_SessionHud_OnSessionEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionHud_OnSessionExit: Binding = Binding {
        id: "Game_SessionHud_OnSessionExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionResults_PopulateTable: Binding = Binding {
        id: "Game_SessionResults_PopulateTable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionResults_ReleaseThumbnails: Binding = Binding {
        id: "Game_SessionResults_ReleaseThumbnails",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionSettings_Reset: Binding = Binding {
        id: "Game_SessionSettings_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionSettings_SetSessionId: Binding = Binding {
        id: "Game_SessionSettings_SetSessionId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionState_EnableOnDemandTextureStreaming: Binding = Binding {
        id: "Game_SessionState_EnableOnDemandTextureStreaming",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionStatsTracker_OnPlayerDroppedIn: Binding = Binding {
        id: "Game_SessionStatsTracker_OnPlayerDroppedIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionStatsTracker_ResetForSession: Binding = Binding {
        id: "Game_SessionStatsTracker_ResetForSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SessionVector_EmplaceBack: Binding = Binding {
        id: "Game_SessionVector_EmplaceBack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Set_VehicleController_Thunk: Binding = Binding {
        id: "Game_Set_VehicleController_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SettingsControlsScreenViewModel_ApplyFilterRebuildList: Binding = Binding {
        id: "Game_SettingsControlsScreenViewModel_ApplyFilterRebuildList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SettingsControlsScreenViewModel_RebindAxis: Binding = Binding {
        id: "Game_SettingsControlsScreenViewModel_RebindAxis",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SettingsTabbingBaseViewModel_ApplyFilterRebuildCategories: Binding = Binding {
        id: "Game_SettingsTabbingBaseViewModel_ApplyFilterRebuildCategories",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SettingsTabbingBaseViewModel_ApplyFilterRefresh: Binding = Binding {
        id: "Game_SettingsTabbingBaseViewModel_ApplyFilterRefresh",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Showroom_HideInfoPanel: Binding = Binding {
        id: "Game_Showroom_HideInfoPanel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Showroom_ShowInfoPanel: Binding = Binding {
        id: "Game_Showroom_ShowInfoPanel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SillySeasonScript_PollAssetsLoaded: Binding = Binding {
        id: "Game_SillySeasonScript_PollAssetsLoaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SillySeason_Initialise: Binding = Binding {
        id: "Game_SillySeason_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SkinnedActor_PopulateSlotArray: Binding = Binding {
        id: "Game_SkinnedActor_PopulateSlotArray",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SocialHubFriendsVM_OnFriendSelected: Binding = Binding {
        id: "Game_SocialHubFriendsVM_OnFriendSelected",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SocialHubFriendsVM_OnInviteResult: Binding = Binding {
        id: "Game_SocialHubFriendsVM_OnInviteResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SocialNotifications_Update: Binding = Binding {
        id: "Game_SocialNotifications_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Social_OnFriendSearchDone: Binding = Binding {
        id: "Game_Social_OnFriendSearchDone",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SpecialistsCutSceneScript_Initialise: Binding = Binding {
        id: "Game_SpecialistsCutSceneScript_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SpecialistsRewards_Initialise: Binding = Binding {
        id: "Game_SpecialistsRewards_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SpeechSystem_PostLine: Binding = Binding {
        id: "Game_SpeechSystem_PostLine",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SpeedBasedFoVZoomCameraModifier_OnEnable: Binding = Binding {
        id: "Game_SpeedBasedFoVZoomCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_SponsorLoadout_BuildList: Binding = Binding {
        id: "Game_SponsorLoadout_BuildList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StabilisationCameraModifier_OnEnable: Binding = Binding {
        id: "Game_StabilisationCameraModifier_OnEnable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_OnLapCompleted: Binding = Binding {
        id: "Game_StatEvent_OnLapCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_OnRaceFinished: Binding = Binding {
        id: "Game_StatEvent_OnRaceFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_OnSectorCompleted: Binding = Binding {
        id: "Game_StatEvent_OnSectorCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteCompleteF1WRace: Binding = Binding {
        id: "Game_StatEvent_WriteCompleteF1WRace",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteF1WeekendFinished: Binding = Binding {
        id: "Game_StatEvent_WriteF1WeekendFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteFacilityUpgraded: Binding = Binding {
        id: "Game_StatEvent_WriteFacilityUpgraded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteFeaturedEventsCompleted: Binding = Binding {
        id: "Game_StatEvent_WriteFeaturedEventsCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteFreeTimeBetweenRaces: Binding = Binding {
        id: "Game_StatEvent_WriteFreeTimeBetweenRaces",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteRacesFinished: Binding = Binding {
        id: "Game_StatEvent_WriteRacesFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteResourcesPoints: Binding = Binding {
        id: "Game_StatEvent_WriteResourcesPoints",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StatEvent_WriteRnDAndFacilityUpgrades: Binding = Binding {
        id: "Game_StatEvent_WriteRnDAndFacilityUpgrades",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StateSession_OnStateExit: Binding = Binding {
        id: "Game_StateSession_OnStateExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubCall_HandleInput: Binding = Binding {
        id: "Game_StoryHubCall_HandleInput",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubCall_HangUp: Binding = Binding {
        id: "Game_StoryHubCall_HangUp",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubCall_SetPaused: Binding = Binding {
        id: "Game_StoryHubCall_SetPaused",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubCall_Update: Binding = Binding {
        id: "Game_StoryHubCall_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_OnAlwaysOnChanged: Binding = Binding {
        id: "Game_StoryHubObjectives_OnAlwaysOnChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_OnAnimEvent: Binding = Binding {
        id: "Game_StoryHubObjectives_OnAnimEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_PlayBonusCloseAnim: Binding = Binding {
        id: "Game_StoryHubObjectives_PlayBonusCloseAnim",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_PlayCloseAnim: Binding = Binding {
        id: "Game_StoryHubObjectives_PlayCloseAnim",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_PlayOpenAnim: Binding = Binding {
        id: "Game_StoryHubObjectives_PlayOpenAnim",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_Refresh: Binding = Binding {
        id: "Game_StoryHubObjectives_Refresh",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_StartFadeTimer: Binding = Binding {
        id: "Game_StoryHubObjectives_StartFadeTimer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHubObjectives_Update: Binding = Binding {
        id: "Game_StoryHubObjectives_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHub_BuildCinematicsList: Binding = Binding {
        id: "Game_StoryHub_BuildCinematicsList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryHub_BuildConversationList: Binding = Binding {
        id: "Game_StoryHub_BuildConversationList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryModeObjective_Update: Binding = Binding {
        id: "Game_StoryModeObjective_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryModeSelect_OnBeginFadeOut: Binding = Binding {
        id: "Game_StoryModeSelect_OnBeginFadeOut",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryPressInterview_Initialise: Binding = Binding {
        id: "Game_StoryPressInterview_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StoryTeamHub_Initialise: Binding = Binding {
        id: "Game_StoryTeamHub_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_StreamingAssetLoadTask_Update: Binding = Binding {
        id: "Game_StreamingAssetLoadTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TabHostVM_OnCommand: Binding = Binding {
        id: "Game_TabHostVM_OnCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TabHostVM_OnTabActivated: Binding = Binding {
        id: "Game_TabHostVM_OnTabActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TabHostVM_OnTabDeactivated: Binding = Binding {
        id: "Game_TabHostVM_OnTabDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TalkingPoints_UpdateBroadcastFades: Binding = Binding {
        id: "Game_TalkingPoints_UpdateBroadcastFades",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TeamHQAdvanceViewModel_OnCommandRaised: Binding = Binding {
        id: "Game_TeamHQAdvanceViewModel_OnCommandRaised",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TeamHQAdvanceViewModel_PressAdvance: Binding = Binding {
        id: "Game_TeamHQAdvanceViewModel_PressAdvance",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TeamHQAdvanceViewModel_ShowTodayCards: Binding = Binding {
        id: "Game_TeamHQAdvanceViewModel_ShowTodayCards",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TeamHQAdvanceViewModel_StopAdvance: Binding = Binding {
        id: "Game_TeamHQAdvanceViewModel_StopAdvance",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TeamHQAdvanceViewModel_Update: Binding = Binding {
        id: "Game_TeamHQAdvanceViewModel_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TeamHQTimeline_AnimateDeptEvent: Binding = Binding {
        id: "Game_TeamHQTimeline_AnimateDeptEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Teleport_Vehicle: Binding = Binding {
        id: "Game_Teleport_Vehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TerminalDamage_Initialise: Binding = Binding {
        id: "Game_TerminalDamage_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_ThrustmasterLcd_UpdateSectors: Binding = Binding {
        id: "Game_ThrustmasterLcd_UpdateSectors",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_Thrustmaster_SetWheelRange: Binding = Binding {
        id: "Game_Thrustmaster_SetWheelRange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TimeAttackGates_CreateGateModel: Binding = Binding {
        id: "Game_TimeAttackGates_CreateGateModel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TimeOfDayDebug_Update: Binding = Binding {
        id: "Game_TimeOfDayDebug_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TimeTrialMode_Shutdown: Binding = Binding {
        id: "Game_TimeTrialMode_Shutdown",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TimingBoard_FormatPositionText: Binding = Binding {
        id: "Game_TimingBoard_FormatPositionText",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TodData_FindParam: Binding = Binding {
        id: "Game_TodData_FindParam",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TodData_FindScenarioParam: Binding = Binding {
        id: "Game_TodData_FindScenarioParam",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TodParamBlock_SetScenario: Binding = Binding {
        id: "Game_TodParamBlock_SetScenario",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackAcclimatisation_CalcTyreTarget: Binding = Binding {
        id: "Game_TrackAcclimatisation_CalcTyreTarget",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackCameraShots_StopShot: Binding = Binding {
        id: "Game_TrackCameraShots_StopShot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackData_GetTrack: Binding = Binding {
        id: "Game_TrackData_GetTrack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackExtraYacht_Initialise: Binding = Binding {
        id: "Game_TrackExtraYacht_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackModelComponent_Initialise: Binding = Binding {
        id: "Game_TrackModelComponent_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackSelectVM_OnScreenActivated: Binding = Binding {
        id: "Game_TrackSelectVM_OnScreenActivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackSpaceDataResource_Load: Binding = Binding {
        id: "Game_TrackSpaceDataResource_Load",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackSpaceTracker_Reset: Binding = Binding {
        id: "Game_TrackSpaceTracker_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackSpace_GetDistanceAtPos: Binding = Binding {
        id: "Game_TrackSpace_GetDistanceAtPos",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackSpace_LoadXml: Binding = Binding {
        id: "Game_TrackSpace_LoadXml",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackSpace_RangeOverlaps: Binding = Binding {
        id: "Game_TrackSpace_RangeOverlaps",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrackSpline_ResolveRouteEnds: Binding = Binding {
        id: "Game_TrackSpline_ResolveRouteEnds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TrophyCabinetTabVM_OnActivate: Binding = Binding {
        id: "Game_TrophyCabinetTabVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TyreAllocationComponent_SetPresetChoiceForHumanDriver: Binding = Binding {
        id: "Game_TyreAllocationComponent_SetPresetChoiceForHumanDriver",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TyreSystem_SwapTyreModel: Binding = Binding {
        id: "Game_TyreSystem_SwapTyreModel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TyreWearDynamicObjective_Activate: Binding = Binding {
        id: "Game_TyreWearDynamicObjective_Activate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_TyreWearRelevanceCriterion_Update: Binding = Binding {
        id: "Game_TyreWearRelevanceCriterion_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_UDPPreferences_OnLocalPreferenceChange: Binding = Binding {
        id: "Game_UDPPreferences_OnLocalPreferenceChange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_UDPPreferences_OnLocalShowPlayerNameChange: Binding = Binding {
        id: "Game_UDPPreferences_OnLocalShowPlayerNameChange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_UIConnectionManager_ShowConnectionLostPopup: Binding = Binding {
        id: "Game_UIConnectionManager_ShowConnectionLostPopup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_UIThumbnailSystem_FindDriverCustomisationWill: Binding = Binding {
        id: "Game_UIThumbnailSystem_FindDriverCustomisationWill",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_UiMessageScreen_Destructor: Binding = Binding {
        id: "Game_UiMessageScreen_Destructor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleAntiStallSystem_GetStateByPlayerId: Binding = Binding {
        id: "Game_VehicleAntiStallSystem_GetStateByPlayerId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleAntiStallSystem_IsLaunchPending: Binding = Binding {
        id: "Game_VehicleAntiStallSystem_IsLaunchPending",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleAntiStallSystem_IsStalled: Binding = Binding {
        id: "Game_VehicleAntiStallSystem_IsStalled",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleAntiStallSystem_IsStartPromptActive: Binding = Binding {
        id: "Game_VehicleAntiStallSystem_IsStartPromptActive",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleCustomisationReplicationManager_AddReplicatedPlayer: Binding = Binding {
        id: "Game_VehicleCustomisationReplicationManager_AddReplicatedPlayer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleLapProgress_CommitNetLapDelta: Binding = Binding {
        id: "Game_VehicleLapProgress_CommitNetLapDelta",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressManager_AddVehicleSync: Binding = Binding {
        id: "Game_VehicleProgressManager_AddVehicleSync",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressManager_RefreshAuthority: Binding = Binding {
        id: "Game_VehicleProgressManager_RefreshAuthority",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressSystem_EraseNetProgressRecords: Binding = Binding {
        id: "Game_VehicleProgressSystem_EraseNetProgressRecords",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressSystem_GetDistanceData: Binding = Binding {
        id: "Game_VehicleProgressSystem_GetDistanceData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressSystem_GetLapProgress: Binding = Binding {
        id: "Game_VehicleProgressSystem_GetLapProgress",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressSystem_OnVehicleRemoved: Binding = Binding {
        id: "Game_VehicleProgressSystem_OnVehicleRemoved",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressSystem_SendOnHostMigration: Binding = Binding {
        id: "Game_VehicleProgressSystem_SendOnHostMigration",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressSystem_SendProgressToConnection: Binding = Binding {
        id: "Game_VehicleProgressSystem_SendProgressToConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleProgressSystem_SubscribeAndClearNet: Binding = Binding {
        id: "Game_VehicleProgressSystem_SubscribeAndClearNet",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VehicleRaceStart_SendRaceStartMessages: Binding = Binding {
        id: "Game_VehicleRaceStart_SendRaceStartMessages",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VendorA_Initialise: Binding = Binding {
        id: "Game_VendorA_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VfxManager_Initialise: Binding = Binding {
        id: "Game_VfxManager_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VictoryVO_DetectWinnerAndNotify: Binding = Binding {
        id: "Game_VictoryVO_DetectWinnerAndNotify",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VictoryVO_OnSpeechFinished: Binding = Binding {
        id: "Game_VictoryVO_OnSpeechFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VictoryVO_PlayVictoryVO: Binding = Binding {
        id: "Game_VictoryVO_PlayVictoryVO",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VirtualHelpPageVM_OnActivate: Binding = Binding {
        id: "Game_VirtualHelpPageVM_OnActivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VirtualHelpPageVM_OnDeactivate: Binding = Binding {
        id: "Game_VirtualHelpPageVM_OnDeactivate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VirtualPerformance_Initialise: Binding = Binding {
        id: "Game_VirtualPerformance_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VirtualSession_Simulate: Binding = Binding {
        id: "Game_VirtualSession_Simulate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VoPlaySoundEffectNode_Execute: Binding = Binding {
        id: "Game_VoPlaySoundEffectNode_Execute",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VotingManager_CastVote: Binding = Binding {
        id: "Game_VotingManager_CastVote",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VotingManager_Destroy: Binding = Binding {
        id: "Game_VotingManager_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VotingManager_HandleStatus: Binding = Binding {
        id: "Game_VotingManager_HandleStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_VotingManager_SetStatus: Binding = Binding {
        id: "Game_VotingManager_SetStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_WonDriverChampionship_Initialise: Binding = Binding {
        id: "Game_WonDriverChampionship_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_WonDriverChampionship_Initialise_2: Binding = Binding {
        id: "Game_WonDriverChampionship_Initialise_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Game_WrinkleModifierMorphRig_Parse: Binding = Binding {
        id: "Game_WrinkleModifierMorphRig_Parse",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Gdb_FindMetadata: Binding = Binding {
        id: "Gdb_FindMetadata",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Gdb_HashMap_Rehash: Binding = Binding {
        id: "Gdb_HashMap_Rehash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Gdb_Load_GameDatabase: Binding = Binding {
        id: "Gdb_Load_GameDatabase",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Gdb_Lookup_Key: Binding = Binding {
        id: "Gdb_Lookup_Key",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Hud_HideReasons_Add: Binding = Binding {
        id: "Hud_HideReasons_Add",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Hud_PitStartClutchPrompt_Update: Binding = Binding {
        id: "Hud_PitStartClutchPrompt_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Hud_PushProIncidentPointsMessage: Binding = Binding {
        id: "Hud_PushProIncidentPointsMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Hud_RaceStartClutchPrompt_Update: Binding = Binding {
        id: "Hud_RaceStartClutchPrompt_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Hud_ReplayHudVM_OnScreenDeactivated: Binding = Binding {
        id: "Hud_ReplayHudVM_OnScreenDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Hud_SpectatorScreenVM_OnScreenDeactivated: Binding = Binding {
        id: "Hud_SpectatorScreenVM_OnScreenDeactivated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Hud_StoryRaceEvent_Begin: Binding = Binding {
        id: "Hud_StoryRaceEvent_Begin",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ActorManager_InsertEntryNode: Binding = Binding {
        id: "Ne_ActorManager_InsertEntryNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ActorManager_RegisterEntry: Binding = Binding {
        id: "Ne_ActorManager_RegisterEntry",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ActorSceneDataResourceFactory_DestroyResource: Binding = Binding {
        id: "Ne_ActorSceneDataResourceFactory_DestroyResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ActorSceneDataResource_DestroyScenes: Binding = Binding {
        id: "Ne_ActorSceneDataResource_DestroyScenes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AnimClipResourceFactory_CreateResource: Binding = Binding {
        id: "Ne_AnimClipResourceFactory_CreateResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AnimLayer_ApplyChannels: Binding = Binding {
        id: "Ne_AnimLayer_ApplyChannels",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AnimSetResourceFactory_CreateResource: Binding = Binding {
        id: "Ne_AnimSetResourceFactory_CreateResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Application2_OnFocusChanged: Binding = Binding {
        id: "Ne_Application2_OnFocusChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Application2_OnVisibilityChanged: Binding = Binding {
        id: "Ne_Application2_OnVisibilityChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Array_InsertionSortPtrs: Binding = Binding {
        id: "Ne_Array_InsertionSortPtrs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AssetGroupSubTask_OnComplete: Binding = Binding {
        id: "Ne_AssetGroupSubTask_OnComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AssetLoader_RequestGroupDescs: Binding = Binding {
        id: "Ne_AssetLoader_RequestGroupDescs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AssetLoader_RequestGroupHandles: Binding = Binding {
        id: "Ne_AssetLoader_RequestGroupHandles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AsyncJob_StopAndNotify: Binding = Binding {
        id: "Ne_AsyncJob_StopAndNotify",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_AudioIo_OpenFileRequest: Binding = Binding {
        id: "Ne_AudioIo_OpenFileRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BindingTable_AddOrUpdate: Binding = Binding {
        id: "Ne_BindingTable_AddOrUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BindingTable_FindChannel: Binding = Binding {
        id: "Ne_BindingTable_FindChannel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitAllocator_AllocBit: Binding = Binding {
        id: "Ne_BitAllocator_AllocBit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_ReadQuantisedFloat: Binding = Binding {
        id: "Ne_BitStream_ReadQuantisedFloat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_ReadQuantizedFloat: Binding = Binding {
        id: "Ne_BitStream_ReadQuantizedFloat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_ReadRangedInt: Binding = Binding {
        id: "Ne_BitStream_ReadRangedInt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_ReadRangedUInt: Binding = Binding {
        id: "Ne_BitStream_ReadRangedUInt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_WriteBits: Binding = Binding {
        id: "Ne_BitStream_WriteBits",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_WriteFloatByPrecision: Binding = Binding {
        id: "Ne_BitStream_WriteFloatByPrecision",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_WriteQuantizedFloat: Binding = Binding {
        id: "Ne_BitStream_WriteQuantizedFloat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_WriteRangedInt: Binding = Binding {
        id: "Ne_BitStream_WriteRangedInt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BitStream_WriteZeroBits: Binding = Binding {
        id: "Ne_BitStream_WriteZeroBits",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Bitset_ThrowInvalidPosition_A: Binding = Binding {
        id: "Ne_Bitset_ThrowInvalidPosition_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Bitset_ThrowInvalidPosition_B: Binding = Binding {
        id: "Ne_Bitset_ThrowInvalidPosition_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Bitset_ThrowInvalidPosition_C: Binding = Binding {
        id: "Ne_Bitset_ThrowInvalidPosition_C",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BufferedAssetGroupLoader_GroupSwapAssetGroups: Binding = Binding {
        id: "Ne_BufferedAssetGroupLoader_GroupSwapAssetGroups",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_BusMessage_SubscribeHandlerUnique: Binding = Binding {
        id: "Ne_BusMessage_SubscribeHandlerUnique",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ByteVector_InsertUnique: Binding = Binding {
        id: "Ne_ByteVector_InsertUnique",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CallbackList_InvokeReverse: Binding = Binding {
        id: "Ne_CallbackList_InvokeReverse",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CloudGenerator_Destroy: Binding = Binding {
        id: "Ne_CloudGenerator_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CloudUploader_Upload: Binding = Binding {
        id: "Ne_CloudUploader_Upload",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Compare_StringNoCase: Binding = Binding {
        id: "Ne_Compare_StringNoCase",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ComponentDefinitions_CreateComponent: Binding = Binding {
        id: "Ne_ComponentDefinitions_CreateComponent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ComponentDefinitions_CreateComponent_2: Binding = Binding {
        id: "Ne_ComponentDefinitions_CreateComponent_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Component_ResolveProviderChain: Binding = Binding {
        id: "Ne_Component_ResolveProviderChain",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Component_ResolveProviderChain2: Binding = Binding {
        id: "Ne_Component_ResolveProviderChain2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ConcurrentHashMap_FatalStub: Binding = Binding {
        id: "Ne_ConcurrentHashMap_FatalStub",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ConcurrentHashMap_FindOrInsert: Binding = Binding {
        id: "Ne_ConcurrentHashMap_FindOrInsert",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ConcurrentHashMap_Rehash: Binding = Binding {
        id: "Ne_ConcurrentHashMap_Rehash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ConfigDict_FindStringIndex: Binding = Binding {
        id: "Ne_ConfigDict_FindStringIndex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ConfigValue_ReadInt: Binding = Binding {
        id: "Ne_ConfigValue_ReadInt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Config_GetFloatOrDefault: Binding = Binding {
        id: "Ne_Config_GetFloatOrDefault",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Config_GetString: Binding = Binding {
        id: "Ne_Config_GetString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Config_ReadSize: Binding = Binding {
        id: "Ne_Config_ReadSize",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Container_AbortNoReturn_142b52980: Binding = Binding {
        id: "Ne_Container_AbortNoReturn_142b52980",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Container_AbortNoReturn_142cc4e50: Binding = Binding {
        id: "Ne_Container_AbortNoReturn_142cc4e50",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Container_AbortNoReturn_142ee16c0: Binding = Binding {
        id: "Ne_Container_AbortNoReturn_142ee16c0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Container_AbortNoReturn_1435d82f0: Binding = Binding {
        id: "Ne_Container_AbortNoReturn_1435d82f0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Container_AbortNoReturn_1443908d0: Binding = Binding {
        id: "Ne_Container_AbortNoReturn_1443908d0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Container_GrowVectorChecked: Binding = Binding {
        id: "Ne_Container_GrowVectorChecked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdAnimationLoader_LoadBehavioursInStage: Binding = Binding {
        id: "Ne_CrowdAnimationLoader_LoadBehavioursInStage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdAnimationLoader_OnAnimClipChanged: Binding = Binding {
        id: "Ne_CrowdAnimationLoader_OnAnimClipChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdPluginConnection_ProcessMessage: Binding = Binding {
        id: "Ne_CrowdPluginConnection_ProcessMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdPluginProxy_Destructor: Binding = Binding {
        id: "Ne_CrowdPluginProxy_Destructor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdPluginProxy_Update: Binding = Binding {
        id: "Ne_CrowdPluginProxy_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdPlugin_FindZoneIndex: Binding = Binding {
        id: "Ne_CrowdPlugin_FindZoneIndex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdPlugin_LoadXml: Binding = Binding {
        id: "Ne_CrowdPlugin_LoadXml",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CrowdZoneInstance_Initialise: Binding = Binding {
        id: "Ne_CrowdZoneInstance_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_AbortNoReturn_A: Binding = Binding {
        id: "Ne_Crt_AbortNoReturn_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_AbortNoReturn_B: Binding = Binding {
        id: "Ne_Crt_AbortNoReturn_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Acosf: Binding = Binding {
        id: "Ne_Crt_Acosf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_AsinfImpl: Binding = Binding {
        id: "Ne_Crt_AsinfImpl",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Atan2f: Binding = Binding {
        id: "Ne_Crt_Atan2f",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Atanf: Binding = Binding {
        id: "Ne_Crt_Atanf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Atexit: Binding = Binding {
        id: "Ne_Crt_Atexit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Chkstk: Binding = Binding {
        id: "Ne_Crt_Chkstk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Cosf: Binding = Binding {
        id: "Ne_Crt_Cosf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_DiffClock: Binding = Binding {
        id: "Ne_Crt_DiffClock",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Exp2f: Binding = Binding {
        id: "Ne_Crt_Exp2f",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_FastFailTerminate: Binding = Binding {
        id: "Ne_Crt_FastFailTerminate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_FatalNoReturn: Binding = Binding {
        id: "Ne_Crt_FatalNoReturn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_FatalTrapB: Binding = Binding {
        id: "Ne_Crt_FatalTrapB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_FatalTrapC: Binding = Binding {
        id: "Ne_Crt_FatalTrapC",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Fmodf: Binding = Binding {
        id: "Ne_Crt_Fmodf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_ForwardTo14534a050: Binding = Binding {
        id: "Ne_Crt_ForwardTo14534a050",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_InitThreadHeader: Binding = Binding {
        id: "Ne_Crt_InitThreadHeader",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_InvalidParameterAbort: Binding = Binding {
        id: "Ne_Crt_InvalidParameterAbort",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_InvalidParameterDispatch: Binding = Binding {
        id: "Ne_Crt_InvalidParameterDispatch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_InvalidParameterNoReturn: Binding = Binding {
        id: "Ne_Crt_InvalidParameterNoReturn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_LazyInitPreserveLastError: Binding = Binding {
        id: "Ne_Crt_LazyInitPreserveLastError",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Localtime64: Binding = Binding {
        id: "Ne_Crt_Localtime64",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_LogFloatAvx: Binding = Binding {
        id: "Ne_Crt_LogFloatAvx",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_MathErrorHandlerF: Binding = Binding {
        id: "Ne_Crt_MathErrorHandlerF",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Memcmp: Binding = Binding {
        id: "Ne_Crt_Memcmp",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Memcpy: Binding = Binding {
        id: "Ne_Crt_Memcpy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Memset: Binding = Binding {
        id: "Ne_Crt_Memset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_ParseNumber_L: Binding = Binding {
        id: "Ne_Crt_ParseNumber_L",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Powf: Binding = Binding {
        id: "Ne_Crt_Powf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Qsort_Impl: Binding = Binding {
        id: "Ne_Crt_Qsort_Impl",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_ReportGsFailure: Binding = Binding {
        id: "Ne_Crt_ReportGsFailure",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_SecurityCookieFailure: Binding = Binding {
        id: "Ne_Crt_SecurityCookieFailure",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_SetMxcsrExceptionFlags: Binding = Binding {
        id: "Ne_Crt_SetMxcsrExceptionFlags",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Setjmp: Binding = Binding {
        id: "Ne_Crt_Setjmp",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Sinf: Binding = Binding {
        id: "Ne_Crt_Sinf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Snprintf: Binding = Binding {
        id: "Ne_Crt_Snprintf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Sqrtf: Binding = Binding {
        id: "Ne_Crt_Sqrtf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Strcmp: Binding = Binding {
        id: "Ne_Crt_Strcmp",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Strncmp: Binding = Binding {
        id: "Ne_Crt_Strncmp",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Strnicmp: Binding = Binding {
        id: "Ne_Crt_Strnicmp",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Strstr: Binding = Binding {
        id: "Ne_Crt_Strstr",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_StrtolCore: Binding = Binding {
        id: "Ne_Crt_StrtolCore",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Swprintf: Binding = Binding {
        id: "Ne_Crt_Swprintf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_ThrowOverflow_A: Binding = Binding {
        id: "Ne_Crt_ThrowOverflow_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_ThrowOverflow_B: Binding = Binding {
        id: "Ne_Crt_ThrowOverflow_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_ThrowOverflow_C: Binding = Binding {
        id: "Ne_Crt_ThrowOverflow_C",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_Time64: Binding = Binding {
        id: "Ne_Crt_Time64",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_ToLower: Binding = Binding {
        id: "Ne_Crt_ToLower",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_TrigDouble: Binding = Binding {
        id: "Ne_Crt_TrigDouble",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Crt_VsnprintfWrapper: Binding = Binding {
        id: "Ne_Crt_VsnprintfWrapper",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CustomisationDecalRenderer_BakePasses: Binding = Binding {
        id: "Ne_CustomisationDecalRenderer_BakePasses",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CustomisationDecalRenderer_Render: Binding = Binding {
        id: "Ne_CustomisationDecalRenderer_Render",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CustomisationDecalsProjectionAreasSetPlugin_CloseConnection: Binding = Binding {
        id: "Ne_CustomisationDecalsProjectionAreasSetPlugin_CloseConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CustomisationDecalsSetPlugin_CloseConnection: Binding = Binding {
        id: "Ne_CustomisationDecalsSetPlugin_CloseConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CutsceneScriptResourceFactory_CreateResource: Binding = Binding {
        id: "Ne_CutsceneScriptResourceFactory_CreateResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CutsceneScriptResourceFactory_CreateResourceFinished: Binding = Binding {
        id: "Ne_CutsceneScriptResourceFactory_CreateResourceFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CutsceneScriptResourceFactory_DestroyResource: Binding = Binding {
        id: "Ne_CutsceneScriptResourceFactory_DestroyResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_CutsceneScriptResourceFactory_StartDestroyResource: Binding = Binding {
        id: "Ne_CutsceneScriptResourceFactory_StartDestroyResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_DataReader_DetectFormat: Binding = Binding {
        id: "Ne_DataReader_DetectFormat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Debug_GetCallerAddress: Binding = Binding {
        id: "Ne_Debug_GetCallerAddress",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Debug_RecordRenderError: Binding = Binding {
        id: "Ne_Debug_RecordRenderError",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Debug_RecordTraceEvent3: Binding = Binding {
        id: "Ne_Debug_RecordTraceEvent3",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Debug_RecordTraceEvent5: Binding = Binding {
        id: "Ne_Debug_RecordTraceEvent5",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_DecalsProjectionAreaProxy_Ctor: Binding = Binding {
        id: "Ne_DecalsProjectionAreaProxy_Ctor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_DecalsProjectionAreaProxy_Dtor: Binding = Binding {
        id: "Ne_DecalsProjectionAreaProxy_Dtor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_DecalsSetProxy_Dtor: Binding = Binding {
        id: "Ne_DecalsSetProxy_Dtor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Delegate_InvokeAll: Binding = Binding {
        id: "Ne_Delegate_InvokeAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Deque_Grow_Qword: Binding = Binding {
        id: "Ne_Deque_Grow_Qword",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Detach_ChildNode: Binding = Binding {
        id: "Ne_Detach_ChildNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EadpErrorsDataService_UninstallExceptionHandler: Binding = Binding {
        id: "Ne_EadpErrorsDataService_UninstallExceptionHandler",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EadpErrorsDataService_UploadAttachment_OnComplete: Binding = Binding {
        id: "Ne_EadpErrorsDataService_UploadAttachment_OnComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EadpGameInviteList_DeleteSessionInvite: Binding = Binding {
        id: "Ne_EadpGameInviteList_DeleteSessionInvite",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EadpGameInviteList_OnInviteValidated: Binding = Binding {
        id: "Ne_EadpGameInviteList_OnInviteValidated",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EadpGameInviteList_UpdateInviteLifetime: Binding = Binding {
        id: "Ne_EadpGameInviteList_UpdateInviteLifetime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EadpPresenceService_UnsubscribeFriends: Binding = Binding {
        id: "Ne_EadpPresenceService_UnsubscribeFriends",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Entity_FindComponent: Binding = Binding {
        id: "Ne_Entity_FindComponent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ErrorContext_SetCode: Binding = Binding {
        id: "Ne_ErrorContext_SetCode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ErrorTrace_PushRing: Binding = Binding {
        id: "Ne_ErrorTrace_PushRing",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EventBus_Unsubscribe: Binding = Binding {
        id: "Ne_EventBus_Unsubscribe",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EventDispatcher_ProcessQueue: Binding = Binding {
        id: "Ne_EventDispatcher_ProcessQueue",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EventDispatcher_SendEvent: Binding = Binding {
        id: "Ne_EventDispatcher_SendEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EventDispatcher_TrackQueuePeak: Binding = Binding {
        id: "Ne_EventDispatcher_TrackQueuePeak",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_EventQueue_DispatchPending: Binding = Binding {
        id: "Ne_EventQueue_DispatchPending",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Event_ResetWithErrorTrace: Binding = Binding {
        id: "Ne_Event_ResetWithErrorTrace",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FatalFormat: Binding = Binding {
        id: "Ne_FatalFormat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FatalFormat_Trap: Binding = Binding {
        id: "Ne_FatalFormat_Trap",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FileSystem_OpenStream: Binding = Binding {
        id: "Ne_FileSystem_OpenStream",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FlagMaskSet_SetBit: Binding = Binding {
        id: "Ne_FlagMaskSet_SetBit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FlatHashMap_FindSlotU64_A: Binding = Binding {
        id: "Ne_FlatHashMap_FindSlotU64_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FlatHashMap_FindSlotU64_B: Binding = Binding {
        id: "Ne_FlatHashMap_FindSlotU64_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FlatHashMap_FindSlotU64_C: Binding = Binding {
        id: "Ne_FlatHashMap_FindSlotU64_C",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FlowStateInterruptManager_ConsumeInterrupt: Binding = Binding {
        id: "Ne_FlowStateInterruptManager_ConsumeInterrupt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_FogSet_SetFogData: Binding = Binding {
        id: "Ne_FogSet_SetFogData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Format_String: Binding = Binding {
        id: "Ne_Format_String",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Get_CStrLen: Binding = Binding {
        id: "Ne_Get_CStrLen",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Get_SettingBool: Binding = Binding {
        id: "Ne_Get_SettingBool",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Get_SettingU32: Binding = Binding {
        id: "Ne_Get_SettingU32",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2CommandQueue_FlushResourceBarriers: Binding = Binding {
        id: "Ne_Gfx2CommandQueue_FlushResourceBarriers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2CommandQueue_PopProfileMarker: Binding = Binding {
        id: "Ne_Gfx2CommandQueue_PopProfileMarker",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2CommandQueue_PushProfileMarker: Binding = Binding {
        id: "Ne_Gfx2CommandQueue_PushProfileMarker",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_BuildRfgSubmitJob: Binding = Binding {
        id: "Ne_Gfx2Renderer_BuildRfgSubmitJob",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_ConstructDevice: Binding = Binding {
        id: "Ne_Gfx2Renderer_ConstructDevice",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_ConstructQueueState: Binding = Binding {
        id: "Ne_Gfx2Renderer_ConstructQueueState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_DestroyDeferredEntryB: Binding = Binding {
        id: "Ne_Gfx2Renderer_DestroyDeferredEntryB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_EnqueueJobLocked: Binding = Binding {
        id: "Ne_Gfx2Renderer_EnqueueJobLocked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_ForwardEventToListener: Binding = Binding {
        id: "Ne_Gfx2Renderer_ForwardEventToListener",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_FrameEntryNameMap_FatalStub: Binding = Binding {
        id: "Ne_Gfx2Renderer_FrameEntryNameMap_FatalStub",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_FrameEntryNameMap_Rehash: Binding = Binding {
        id: "Ne_Gfx2Renderer_FrameEntryNameMap_Rehash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_FrameStartEnd: Binding = Binding {
        id: "Ne_Gfx2Renderer_FrameStartEnd",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_GetCommandList: Binding = Binding {
        id: "Ne_Gfx2Renderer_GetCommandList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_Initialise: Binding = Binding {
        id: "Ne_Gfx2Renderer_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_PostEvent_AllocateResource: Binding = Binding {
        id: "Ne_Gfx2Renderer_PostEvent_AllocateResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_PostEvent_ErpLoaded: Binding = Binding {
        id: "Ne_Gfx2Renderer_PostEvent_ErpLoaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_PostEvent_ErpRead: Binding = Binding {
        id: "Ne_Gfx2Renderer_PostEvent_ErpRead",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_PostEvent_ErpUnloaded: Binding = Binding {
        id: "Ne_Gfx2Renderer_PostEvent_ErpUnloaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_ProcessResourceEvent: Binding = Binding {
        id: "Ne_Gfx2Renderer_ProcessResourceEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_RecordCommandNodes: Binding = Binding {
        id: "Ne_Gfx2Renderer_RecordCommandNodes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_ReleaseDeferredObjects: Binding = Binding {
        id: "Ne_Gfx2Renderer_ReleaseDeferredObjects",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_RenderFrame: Binding = Binding {
        id: "Ne_Gfx2Renderer_RenderFrame",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2Renderer_UploadTransientBatch: Binding = Binding {
        id: "Ne_Gfx2Renderer_UploadTransientBatch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2TextureManager_SetInRaceOff: Binding = Binding {
        id: "Ne_Gfx2TextureManager_SetInRaceOff",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2TextureManager_SetInRaceOn: Binding = Binding {
        id: "Ne_Gfx2TextureManager_SetInRaceOn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2_BeginTripleBufferedScope: Binding = Binding {
        id: "Ne_Gfx2_BeginTripleBufferedScope",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2_TrackResourceInsert_A: Binding = Binding {
        id: "Ne_Gfx2_TrackResourceInsert_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2_TrackResourceInsert_B: Binding = Binding {
        id: "Ne_Gfx2_TrackResourceInsert_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Gfx2_TrackResourceInsert_C: Binding = Binding {
        id: "Ne_Gfx2_TrackResourceInsert_C",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_GlobalHandle_Release: Binding = Binding {
        id: "Ne_GlobalHandle_Release",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_EmplaceNode: Binding = Binding {
        id: "Ne_HashMap_EmplaceNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FatalStub_GroundCover: Binding = Binding {
        id: "Ne_HashMap_FatalStub_GroundCover",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FatalStub_Hbcn: Binding = Binding {
        id: "Ne_HashMap_FatalStub_Hbcn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FatalStub_ModelMaterial: Binding = Binding {
        id: "Ne_HashMap_FatalStub_ModelMaterial",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FatalStub_ParticleResourcePack: Binding = Binding {
        id: "Ne_HashMap_FatalStub_ParticleResourcePack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FatalStub_Particles: Binding = Binding {
        id: "Ne_HashMap_FatalStub_Particles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FatalStub_SkidmarksA: Binding = Binding {
        id: "Ne_HashMap_FatalStub_SkidmarksA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FatalStub_SkidmarksB: Binding = Binding {
        id: "Ne_HashMap_FatalStub_SkidmarksB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FindOrInsert_Float: Binding = Binding {
        id: "Ne_HashMap_FindOrInsert_Float",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_FindSlot_Thumbnail: Binding = Binding {
        id: "Ne_HashMap_FindSlot_Thumbnail",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Find_141b732f0: Binding = Binding {
        id: "Ne_HashMap_Find_141b732f0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_GrowRehash_DecalLinks: Binding = Binding {
        id: "Ne_HashMap_GrowRehash_DecalLinks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_GrowRehash_Decals: Binding = Binding {
        id: "Ne_HashMap_GrowRehash_Decals",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Grow_Gfx2ResourceObject: Binding = Binding {
        id: "Ne_HashMap_Grow_Gfx2ResourceObject",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Grow_GroundCoverPlugin: Binding = Binding {
        id: "Ne_HashMap_Grow_GroundCoverPlugin",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Grow_Hbcn: Binding = Binding {
        id: "Ne_HashMap_Grow_Hbcn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Grow_MaterialActorId: Binding = Binding {
        id: "Ne_HashMap_Grow_MaterialActorId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Grow_ParticleResourcePack: Binding = Binding {
        id: "Ne_HashMap_Grow_ParticleResourcePack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Grow_SkidmarksA: Binding = Binding {
        id: "Ne_HashMap_Grow_SkidmarksA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Grow_SkidmarksB: Binding = Binding {
        id: "Ne_HashMap_Grow_SkidmarksB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Guid128_Grow: Binding = Binding {
        id: "Ne_HashMap_Guid128_Grow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Insert_DecalLinks: Binding = Binding {
        id: "Ne_HashMap_Insert_DecalLinks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_MoveAssign_Decals: Binding = Binding {
        id: "Ne_HashMap_MoveAssign_Decals",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Rehash: Binding = Binding {
        id: "Ne_HashMap_Rehash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Rehash_142b52990: Binding = Binding {
        id: "Ne_HashMap_Rehash_142b52990",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Rehash_142ee16d0: Binding = Binding {
        id: "Ne_HashMap_Rehash_142ee16d0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Rehash_1435d8440: Binding = Binding {
        id: "Ne_HashMap_Rehash_1435d8440",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Rehash_1443908e0: Binding = Binding {
        id: "Ne_HashMap_Rehash_1443908e0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_Rehash_Gfx2Renderer: Binding = Binding {
        id: "Ne_HashMap_Rehash_Gfx2Renderer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ReserveBuckets_Gfx: Binding = Binding {
        id: "Ne_HashMap_ReserveBuckets_Gfx",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_DecalFiles: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_DecalFiles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_DecalLinks: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_DecalLinks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_DecalLinks2: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_DecalLinks2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_Decals: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_Decals",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_Decals2: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_Decals2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_Gfx2ResourceObject: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_Gfx2ResourceObject",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_MaterialActorId: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_MaterialActorId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_ThrowLengthError_Thumbnail: Binding = Binding {
        id: "Ne_HashMap_ThrowLengthError_Thumbnail",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_U64_FindOrInsertSlot: Binding = Binding {
        id: "Ne_HashMap_U64_FindOrInsertSlot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_U64_Grow: Binding = Binding {
        id: "Ne_HashMap_U64_Grow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_U64_Insert: Binding = Binding {
        id: "Ne_HashMap_U64_Insert",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashMap_U64_Insert_Fnv: Binding = Binding {
        id: "Ne_HashMap_U64_Insert_Fnv",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Hash_Djb2Xor: Binding = Binding {
        id: "Ne_Hash_Djb2Xor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Hash_Md5Digest: Binding = Binding {
        id: "Ne_Hash_Md5Digest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Hash_Md5Finalize: Binding = Binding {
        id: "Ne_Hash_Md5Finalize",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Hash_Md5PadFinalBlock: Binding = Binding {
        id: "Ne_Hash_Md5PadFinalBlock",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Hash_Md5Transform: Binding = Binding {
        id: "Ne_Hash_Md5Transform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Hash_Md5Transform_Alt: Binding = Binding {
        id: "Ne_Hash_Md5Transform_Alt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Hash_Md5U64: Binding = Binding {
        id: "Ne_Hash_Md5U64",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashedName_FromString: Binding = Binding {
        id: "Ne_HashedName_FromString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_HashedString_Assign: Binding = Binding {
        id: "Ne_HashedString_Assign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Heap_AllocUnknownTag: Binding = Binding {
        id: "Ne_Heap_AllocUnknownTag",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Heap_DeallocateNonSmall: Binding = Binding {
        id: "Ne_Heap_DeallocateNonSmall",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Heap_OnCorruptionNotification: Binding = Binding {
        id: "Ne_Heap_OnCorruptionNotification",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_IBLCubeProbe_FreeInterpolatedArrays: Binding = Binding {
        id: "Ne_IBLCubeProbe_FreeInterpolatedArrays",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Import_ResolveSlotWithRetry: Binding = Binding {
        id: "Ne_Import_ResolveSlotWithRetry",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputDeviceDirectInput_ResetForceFeedback: Binding = Binding {
        id: "Ne_InputDeviceDirectInput_ResetForceFeedback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputDevicePadPS_Cleanup: Binding = Binding {
        id: "Ne_InputDevicePadPS_Cleanup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputDevicePadPS_Initialise: Binding = Binding {
        id: "Ne_InputDevicePadPS_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputLibraryDirectInput_EnumDevicesCallback: Binding = Binding {
        id: "Ne_InputLibraryDirectInput_EnumDevicesCallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputLibraryDirectInput_IsXInputDevice: Binding = Binding {
        id: "Ne_InputLibraryDirectInput_IsXInputDevice",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputMapper_FindBindingByHash: Binding = Binding {
        id: "Ne_InputMapper_FindBindingByHash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputMapper_HasAction: Binding = Binding {
        id: "Ne_InputMapper_HasAction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputMapper_IsActionPressed: Binding = Binding {
        id: "Ne_InputMapper_IsActionPressed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputMapper_IsActionReleased: Binding = Binding {
        id: "Ne_InputMapper_IsActionReleased",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputSystem_DeviceConnection: Binding = Binding {
        id: "Ne_InputSystem_DeviceConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputSystem_Init: Binding = Binding {
        id: "Ne_InputSystem_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputSystem_MergePreset: Binding = Binding {
        id: "Ne_InputSystem_MergePreset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InputSystem_UnbindDevice: Binding = Binding {
        id: "Ne_InputSystem_UnbindDevice",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Input_InternActionName: Binding = Binding {
        id: "Ne_Input_InternActionName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Input_IsActionDownAnyDevice: Binding = Binding {
        id: "Ne_Input_IsActionDownAnyDevice",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Input_IsActionEnabled: Binding = Binding {
        id: "Ne_Input_IsActionEnabled",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InternedString_Manage: Binding = Binding {
        id: "Ne_InternedString_Manage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_InternedString_ManageB: Binding = Binding {
        id: "Ne_InternedString_ManageB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_IntrusiveList_ClearAndRelease: Binding = Binding {
        id: "Ne_IntrusiveList_ClearAndRelease",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_IntrusivePtr_Assign: Binding = Binding {
        id: "Ne_IntrusivePtr_Assign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_JobQueue_FindBestJob: Binding = Binding {
        id: "Ne_JobQueue_FindBestJob",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_JobQueue_RunPendingJobs: Binding = Binding {
        id: "Ne_JobQueue_RunPendingJobs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_JobSignal_WaitWithTimeout: Binding = Binding {
        id: "Ne_JobSignal_WaitWithTimeout",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_JobSystem_CreateJobGroup: Binding = Binding {
        id: "Ne_JobSystem_CreateJobGroup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_JsonObject_FindMember: Binding = Binding {
        id: "Ne_JsonObject_FindMember",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_JsonObject_FindMember_2: Binding = Binding {
        id: "Ne_JsonObject_FindMember_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_JsonWriter_WriteBool: Binding = Binding {
        id: "Ne_JsonWriter_WriteBool",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_CheckIhdr: Binding = Binding {
        id: "Ne_Libpng_CheckIhdr",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_DeflateEndChecked: Binding = Binding {
        id: "Ne_Libpng_DeflateEndChecked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_ErrorNoReturn: Binding = Binding {
        id: "Ne_Libpng_ErrorNoReturn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_PrepareWrite: Binding = Binding {
        id: "Ne_Libpng_PrepareWrite",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_SetIhdr: Binding = Binding {
        id: "Ne_Libpng_SetIhdr",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_SetjmpCleanup_1447a0cf0: Binding = Binding {
        id: "Ne_Libpng_SetjmpCleanup_1447a0cf0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_Warning: Binding = Binding {
        id: "Ne_Libpng_Warning",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkBkgd: Binding = Binding {
        id: "Ne_Libpng_WriteChunkBkgd",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkHeader: Binding = Binding {
        id: "Ne_Libpng_WriteChunkHeader",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkHist: Binding = Binding {
        id: "Ne_Libpng_WriteChunkHist",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkIccp: Binding = Binding {
        id: "Ne_Libpng_WriteChunkIccp",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkPcal: Binding = Binding {
        id: "Ne_Libpng_WriteChunkPcal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkPlte: Binding = Binding {
        id: "Ne_Libpng_WriteChunkPlte",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkSbit: Binding = Binding {
        id: "Ne_Libpng_WriteChunkSbit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkSplt: Binding = Binding {
        id: "Ne_Libpng_WriteChunkSplt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteChunkTrns: Binding = Binding {
        id: "Ne_Libpng_WriteChunkTrns",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteEXIf: Binding = Binding {
        id: "Ne_Libpng_WriteEXIf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteEnd: Binding = Binding {
        id: "Ne_Libpng_WriteEnd",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteITXt: Binding = Binding {
        id: "Ne_Libpng_WriteITXt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteIhdr: Binding = Binding {
        id: "Ne_Libpng_WriteIhdr",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteImage: Binding = Binding {
        id: "Ne_Libpng_WriteImage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteInfo: Binding = Binding {
        id: "Ne_Libpng_WriteInfo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteRow: Binding = Binding {
        id: "Ne_Libpng_WriteRow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteTEXt: Binding = Binding {
        id: "Ne_Libpng_WriteTEXt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Libpng_WriteZTXt: Binding = Binding {
        id: "Ne_Libpng_WriteZTXt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_LinkResolver_PushPending: Binding = Binding {
        id: "Ne_LinkResolver_PushPending",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_LinkResolver_RegisterLinkable: Binding = Binding {
        id: "Ne_LinkResolver_RegisterLinkable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_LinkResolver_ResolveLinks: Binding = Binding {
        id: "Ne_LinkResolver_ResolveLinks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_List_PushEntryOrFail: Binding = Binding {
        id: "Ne_List_PushEntryOrFail",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ListenerHandle_Assign: Binding = Binding {
        id: "Ne_ListenerHandle_Assign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ListenerHandle_Unregister: Binding = Binding {
        id: "Ne_ListenerHandle_Unregister",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Listener_DetachFromOwner: Binding = Binding {
        id: "Ne_Listener_DetachFromOwner",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Loader_RegisterRequest: Binding = Binding {
        id: "Ne_Loader_RegisterRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Log_Thunk: Binding = Binding {
        id: "Ne_Log_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Make_RandomIdBlock: Binding = Binding {
        id: "Ne_Make_RandomIdBlock",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Make_StrFromBuffer: Binding = Binding {
        id: "Ne_Make_StrFromBuffer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Make_StrFromCStr: Binding = Binding {
        id: "Ne_Make_StrFromCStr",
        kind: "func",
        status: Status::Stable,
    };
    pub const Ne_Map_DestroySubtree: Binding = Binding {
        id: "Ne_Map_DestroySubtree",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Map_DestroySubtree_Decals: Binding = Binding {
        id: "Ne_Map_DestroySubtree_Decals",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Map_FindOrDefaultWithErrorContext: Binding = Binding {
        id: "Ne_Map_FindOrDefaultWithErrorContext",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Map_LowerBoundPairIntInt: Binding = Binding {
        id: "Ne_Map_LowerBoundPairIntInt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Math_BasisFromNormal: Binding = Binding {
        id: "Ne_Math_BasisFromNormal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Math_Mat4Invert: Binding = Binding {
        id: "Ne_Math_Mat4Invert",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Math_Mat4Multiply: Binding = Binding {
        id: "Ne_Math_Mat4Multiply",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Math_Matrix4x4_Invert: Binding = Binding {
        id: "Ne_Math_Matrix4x4_Invert",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Math_ObbOverlapTest: Binding = Binding {
        id: "Ne_Math_ObbOverlapTest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Math_QuatFromEulerXYZ: Binding = Binding {
        id: "Ne_Math_QuatFromEulerXYZ",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MessageQueue_Alloc: Binding = Binding {
        id: "Ne_MessageQueue_Alloc",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MipTailGeneratorFactory_ObtainGeneratorResource: Binding = Binding {
        id: "Ne_MipTailGeneratorFactory_ObtainGeneratorResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MipTailGenerator_Construct: Binding = Binding {
        id: "Ne_MipTailGenerator_Construct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MipTailGenerator_CreateResources: Binding = Binding {
        id: "Ne_MipTailGenerator_CreateResources",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MipTailGenerator_Destroy: Binding = Binding {
        id: "Ne_MipTailGenerator_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MipTailGenerator_Record: Binding = Binding {
        id: "Ne_MipTailGenerator_Record",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ModelInstance_SetNodeMatrices: Binding = Binding {
        id: "Ne_ModelInstance_SetNodeMatrices",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ModelNodeBinding_Clear: Binding = Binding {
        id: "Ne_ModelNodeBinding_Clear",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ModelNodeBinding_Init: Binding = Binding {
        id: "Ne_ModelNodeBinding_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ModuleSystem_CheckConfig: Binding = Binding {
        id: "Ne_ModuleSystem_CheckConfig",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ModuleSystem_CheckPlatform: Binding = Binding {
        id: "Ne_ModuleSystem_CheckPlatform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ModuleSystem_Destroy: Binding = Binding {
        id: "Ne_ModuleSystem_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ModuleSystem_StartModules: Binding = Binding {
        id: "Ne_ModuleSystem_StartModules",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MoverCollision_BuildMoverShapes: Binding = Binding {
        id: "Ne_MoverCollision_BuildMoverShapes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MoverCollision_BuildQueryContext: Binding = Binding {
        id: "Ne_MoverCollision_BuildQueryContext",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_MoverCollision_CreateResource: Binding = Binding {
        id: "Ne_MoverCollision_CreateResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NameRegistry_FindId: Binding = Binding {
        id: "Ne_NameRegistry_FindId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NameTable_FindByFnv1a_B: Binding = Binding {
        id: "Ne_NameTable_FindByFnv1a_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_ConnectionArrived: Binding = Binding {
        id: "Ne_NetConnectionQuality_ConnectionArrived",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_ConnectionLeft: Binding = Binding {
        id: "Ne_NetConnectionQuality_ConnectionLeft",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_OnBandwidthEvalPayload: Binding = Binding {
        id: "Ne_NetConnectionQuality_OnBandwidthEvalPayload",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_OnBandwidthEvalResultMessage: Binding = Binding {
        id: "Ne_NetConnectionQuality_OnBandwidthEvalResultMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_OnBandwidthEvalStartMessage: Binding = Binding {
        id: "Ne_NetConnectionQuality_OnBandwidthEvalStartMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_OnBandwidthEvalStartReplyMessage: Binding = Binding {
        id: "Ne_NetConnectionQuality_OnBandwidthEvalStartReplyMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_OnBandwidthTesterStateRequestMessage: Binding = Binding {
        id: "Ne_NetConnectionQuality_OnBandwidthTesterStateRequestMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_OnBandwidthTesterStateUpdateMessage: Binding = Binding {
        id: "Ne_NetConnectionQuality_OnBandwidthTesterStateUpdateMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_OnStatsUpdateMessage: Binding = Binding {
        id: "Ne_NetConnectionQuality_OnStatsUpdateMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_SendAllStatsUpdates: Binding = Binding {
        id: "Ne_NetConnectionQuality_SendAllStatsUpdates",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_SendBandwidthTesterStateUpdateMessage: Binding = Binding {
        id: "Ne_NetConnectionQuality_SendBandwidthTesterStateUpdateMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_SendSingleStatsUpdate: Binding = Binding {
        id: "Ne_NetConnectionQuality_SendSingleStatsUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionQuality_SetTesterState: Binding = Binding {
        id: "Ne_NetConnectionQuality_SetTesterState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionStore_Deinitialise: Binding = Binding {
        id: "Ne_NetConnectionStore_Deinitialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionStore_Dtor: Binding = Binding {
        id: "Ne_NetConnectionStore_Dtor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionStore_Initialise: Binding = Binding {
        id: "Ne_NetConnectionStore_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetConnectionStore_RemoveConnection: Binding = Binding {
        id: "Ne_NetConnectionStore_RemoveConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetHttpRequestHandler_CreateRequest: Binding = Binding {
        id: "Ne_NetHttpRequestHandler_CreateRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetNullSSLPeer_Update: Binding = Binding {
        id: "Ne_NetNullSSLPeer_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetReliableAsyncSocket_DeletingDtor: Binding = Binding {
        id: "Ne_NetReliableAsyncSocket_DeletingDtor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetReliableAsyncSocket_DisconnectAll: Binding = Binding {
        id: "Ne_NetReliableAsyncSocket_DisconnectAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetReliableAsyncSocket_OnHandshakeCompleted: Binding = Binding {
        id: "Ne_NetReliableAsyncSocket_OnHandshakeCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetReliableAsyncSocket_OnMessageDecrypted: Binding = Binding {
        id: "Ne_NetReliableAsyncSocket_OnMessageDecrypted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetReliableAsyncSocket_OnMessageEncrypted: Binding = Binding {
        id: "Ne_NetReliableAsyncSocket_OnMessageEncrypted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetReliableAsyncSocket_Restart: Binding = Binding {
        id: "Ne_NetReliableAsyncSocket_Restart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetReliableAsyncSocket_SetKeyMaterial: Binding = Binding {
        id: "Ne_NetReliableAsyncSocket_SetKeyMaterial",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_CallSessionBrowserCreateSession: Binding = Binding {
        id: "Ne_NetSessionF1OS_CallSessionBrowserCreateSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_JoinSession: Binding = Binding {
        id: "Ne_NetSessionF1OS_JoinSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_MigrateSessionClient: Binding = Binding {
        id: "Ne_NetSessionF1OS_MigrateSessionClient",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_MigrateSessionHost: Binding = Binding {
        id: "Ne_NetSessionF1OS_MigrateSessionHost",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_Reset: Binding = Binding {
        id: "Ne_NetSessionF1OS_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_SetCrossplay: Binding = Binding {
        id: "Ne_NetSessionF1OS_SetCrossplay",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_SetSessionInfo: Binding = Binding {
        id: "Ne_NetSessionF1OS_SetSessionInfo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_SetSessionName: Binding = Binding {
        id: "Ne_NetSessionF1OS_SetSessionName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_SetStateCreating: Binding = Binding {
        id: "Ne_NetSessionF1OS_SetStateCreating",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionF1OS_UpdateAttributes: Binding = Binding {
        id: "Ne_NetSessionF1OS_UpdateAttributes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionLAN_MigrateSessionClient: Binding = Binding {
        id: "Ne_NetSessionLAN_MigrateSessionClient",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionLAN_MigrateSessionHost: Binding = Binding {
        id: "Ne_NetSessionLAN_MigrateSessionHost",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionLAN_OnCreateMigratedSessionFinished: Binding = Binding {
        id: "Ne_NetSessionLAN_OnCreateMigratedSessionFinished",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionSearcherF1OS_OnSessionBrowserResults: Binding = Binding {
        id: "Ne_NetSessionSearcherF1OS_OnSessionBrowserResults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSessionSearcherLAN_SearchForSessions: Binding = Binding {
        id: "Ne_NetSessionSearcherLAN_SearchForSessions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSyncClock_IntegrateRemoteHostClock: Binding = Binding {
        id: "Ne_NetSyncClock_IntegrateRemoteHostClock",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSyncClock_RecalculateClockOffset: Binding = Binding {
        id: "Ne_NetSyncClock_RecalculateClockOffset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_CreateSession: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_CreateSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_JoinSession: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_JoinSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_MigrateSession: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_MigrateSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_NotifyPeersMigrationResult: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_NotifyPeersMigrationResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_OnCreateSessionComplete: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_OnCreateSessionComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_OnHandshakingComplete: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_OnHandshakingComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_OnJoinSessionComplete: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_OnJoinSessionComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_OnMigrateSessionClient: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_OnMigrateSessionClient",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_OnSessionError: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_OnSessionError",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_OnSessionSearchComplete: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_OnSessionSearchComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_ResetNetworkSystem: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_ResetNetworkSystem",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_ResetNetworkSystemCompleted: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_ResetNetworkSystemCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemClassicMatchmaking_SearchForSessions: Binding = Binding {
        id: "Ne_NetSystemClassicMatchmaking_SearchForSessions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemCommon_HandleSendToConnection: Binding = Binding {
        id: "Ne_NetSystemCommon_HandleSendToConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemCommon_LeaveSession: Binding = Binding {
        id: "Ne_NetSystemCommon_LeaveSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemCommon_Migrate: Binding = Binding {
        id: "Ne_NetSystemCommon_Migrate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemCommon_OnConnectionArrived: Binding = Binding {
        id: "Ne_NetSystemCommon_OnConnectionArrived",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemCommon_ResetNetworkSystem: Binding = Binding {
        id: "Ne_NetSystemCommon_ResetNetworkSystem",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemCommon_ResetNetworkSystemCompleted: Binding = Binding {
        id: "Ne_NetSystemCommon_ResetNetworkSystemCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemCommon_SendKeepAliveToConnections: Binding = Binding {
        id: "Ne_NetSystemCommon_SendKeepAliveToConnections",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemHelloMessage_Deserialise: Binding = Binding {
        id: "Ne_NetSystemHelloMessage_Deserialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemHelloMessage_Serialise: Binding = Binding {
        id: "Ne_NetSystemHelloMessage_Serialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemServerHelloResponseMessage_Deserialise: Binding = Binding {
        id: "Ne_NetSystemServerHelloResponseMessage_Deserialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetSystemServerHelloResponseMessage_Serialise: Binding = Binding {
        id: "Ne_NetSystemServerHelloResponseMessage_Serialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_AddHostConnectionInfoToConnectionStore: Binding = Binding {
        id: "Ne_NetTransportCommon_AddHostConnectionInfoToConnectionStore",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_BroadcastNewConnectionToClients: Binding = Binding {
        id: "Ne_NetTransportCommon_BroadcastNewConnectionToClients",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_BroadcastRemovedConnectionToClients: Binding = Binding {
        id: "Ne_NetTransportCommon_BroadcastRemovedConnectionToClients",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_FreeConnectionID: Binding = Binding {
        id: "Ne_NetTransportCommon_FreeConnectionID",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_HandleClientHello: Binding = Binding {
        id: "Ne_NetTransportCommon_HandleClientHello",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_HandleServerDisconnection: Binding = Binding {
        id: "Ne_NetTransportCommon_HandleServerDisconnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_InitialiseClient: Binding = Binding {
        id: "Ne_NetTransportCommon_InitialiseClient",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_Migrate: Binding = Binding {
        id: "Ne_NetTransportCommon_Migrate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_ReportClientInitialConnections: Binding = Binding {
        id: "Ne_NetTransportCommon_ReportClientInitialConnections",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_SetLocalConnectivity: Binding = Binding {
        id: "Ne_NetTransportCommon_SetLocalConnectivity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_SetMyConnectionId: Binding = Binding {
        id: "Ne_NetTransportCommon_SetMyConnectionId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportCommon_StartHandshaking: Binding = Binding {
        id: "Ne_NetTransportCommon_StartHandshaking",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportLAN_OnConnectionLost: Binding = Binding {
        id: "Ne_NetTransportLAN_OnConnectionLost",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportLAN_RegisterArrivedConnection: Binding = Binding {
        id: "Ne_NetTransportLAN_RegisterArrivedConnection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportPlayFab_HandlePartyStateChange: Binding = Binding {
        id: "Ne_NetTransportPlayFab_HandlePartyStateChange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportPlayFab_MigrateConnections: Binding = Binding {
        id: "Ne_NetTransportPlayFab_MigrateConnections",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportPlayFab_SendTo: Binding = Binding {
        id: "Ne_NetTransportPlayFab_SendTo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransportPlayFab_SendToConnectionInfo: Binding = Binding {
        id: "Ne_NetTransportPlayFab_SendToConnectionInfo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetTransport_MigrateConnections: Binding = Binding {
        id: "Ne_NetTransport_MigrateConnections",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NetVoiceNotificationClient_WaveInAddBuffer: Binding = Binding {
        id: "Ne_NetVoiceNotificationClient_WaveInAddBuffer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Network_Init: Binding = Binding {
        id: "Ne_Network_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Network_StartAsync: Binding = Binding {
        id: "Ne_Network_StartAsync",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_NodeTable_FindIndexByHash: Binding = Binding {
        id: "Ne_NodeTable_FindIndexByHash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Node_DestroyTreeRecursive: Binding = Binding {
        id: "Ne_Node_DestroyTreeRecursive",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Noise_Fbm1D: Binding = Binding {
        id: "Ne_Noise_Fbm1D",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_OSHTTPRequestAndResponseResponseHandler_CbHttpResponse: Binding = Binding {
        id: "Ne_OSHTTPRequestAndResponseResponseHandler_CbHttpResponse",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_OSHttpOperation_OnStart: Binding = Binding {
        id: "Ne_OSHttpOperation_OnStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_OSHttpRequest_Send: Binding = Binding {
        id: "Ne_OSHttpRequest_Send",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_OSOperationHandle_UnregisterSelf: Binding = Binding {
        id: "Ne_OSOperationHandle_UnregisterSelf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_OSOperationQueue_StartPendingOperation: Binding = Binding {
        id: "Ne_OSOperationQueue_StartPendingOperation",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_OptionalSharedPtr_Reset: Binding = Binding {
        id: "Ne_OptionalSharedPtr_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_OptionalVector56_CopyAssign: Binding = Binding {
        id: "Ne_OptionalVector56_CopyAssign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ParameterStudio_ReserveInstancePools: Binding = Binding {
        id: "Ne_ParameterStudio_ReserveInstancePools",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Parse_BoolCString: Binding = Binding {
        id: "Ne_Parse_BoolCString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Parse_BoolValue: Binding = Binding {
        id: "Ne_Parse_BoolValue",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Parse_Long: Binding = Binding {
        id: "Ne_Parse_Long",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PathBuf_Construct: Binding = Binding {
        id: "Ne_PathBuf_Construct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysBody_ApplyImpulse: Binding = Binding {
        id: "Ne_PhysBody_ApplyImpulse",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysBody_SetAngularVelocity: Binding = Binding {
        id: "Ne_PhysBody_SetAngularVelocity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysBody_SetLinearVelocity: Binding = Binding {
        id: "Ne_PhysBody_SetLinearVelocity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysBody_SetTransform: Binding = Binding {
        id: "Ne_PhysBody_SetTransform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysCollision_FreeIntersectionUnits: Binding = Binding {
        id: "Ne_PhysCollision_FreeIntersectionUnits",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysDeform_BuildTyreWallSoftBody: Binding = Binding {
        id: "Ne_PhysDeform_BuildTyreWallSoftBody",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysMesh_RaycastTriangles: Binding = Binding {
        id: "Ne_PhysMesh_RaycastTriangles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysPool_AllocZeroed: Binding = Binding {
        id: "Ne_PhysPool_AllocZeroed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysShape_BuildConvexFromPoints: Binding = Binding {
        id: "Ne_PhysShape_BuildConvexFromPoints",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysShape_CreateConvex: Binding = Binding {
        id: "Ne_PhysShape_CreateConvex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysShape_InitConvexDefaults: Binding = Binding {
        id: "Ne_PhysShape_InitConvexDefaults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysUpdateStage_FinishStepNotifyBodies: Binding = Binding {
        id: "Ne_PhysUpdateStage_FinishStepNotifyBodies",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysUpdateStage_FlushSubstepRecords: Binding = Binding {
        id: "Ne_PhysUpdateStage_FlushSubstepRecords",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysUpdateStage_InterpolateBodyStates: Binding = Binding {
        id: "Ne_PhysUpdateStage_InterpolateBodyStates",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysUpdateStage_RunSubsteps: Binding = Binding {
        id: "Ne_PhysUpdateStage_RunSubsteps",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_AdvanceStepState: Binding = Binding {
        id: "Ne_PhysWorld_AdvanceStepState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_ApplyBroadPhaseCcdCollisions: Binding = Binding {
        id: "Ne_PhysWorld_ApplyBroadPhaseCcdCollisions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_ApplyBroadPhaseCollisions: Binding = Binding {
        id: "Ne_PhysWorld_ApplyBroadPhaseCollisions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_BuildBroadphaseGrid: Binding = Binding {
        id: "Ne_PhysWorld_BuildBroadphaseGrid",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_CollectBroadphasePairsForBody: Binding = Binding {
        id: "Ne_PhysWorld_CollectBroadphasePairsForBody",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_CommitBody: Binding = Binding {
        id: "Ne_PhysWorld_CommitBody",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_CreateConstrainedBody: Binding = Binding {
        id: "Ne_PhysWorld_CreateConstrainedBody",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_CreateRigidBody: Binding = Binding {
        id: "Ne_PhysWorld_CreateRigidBody",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_InitFromConfig: Binding = Binding {
        id: "Ne_PhysWorld_InitFromConfig",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_NotifyBodyStepCallbacks: Binding = Binding {
        id: "Ne_PhysWorld_NotifyBodyStepCallbacks",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_PostSolveIslands: Binding = Binding {
        id: "Ne_PhysWorld_PostSolveIslands",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_QueryAabbAnyHit: Binding = Binding {
        id: "Ne_PhysWorld_QueryAabbAnyHit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_RaycastClosest: Binding = Binding {
        id: "Ne_PhysWorld_RaycastClosest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_ReleaseContainers: Binding = Binding {
        id: "Ne_PhysWorld_ReleaseContainers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_ResetFrameState: Binding = Binding {
        id: "Ne_PhysWorld_ResetFrameState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_ResizeWorkerContexts: Binding = Binding {
        id: "Ne_PhysWorld_ResizeWorkerContexts",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_RunIntegratorJobs: Binding = Binding {
        id: "Ne_PhysWorld_RunIntegratorJobs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_RunSolveJobs: Binding = Binding {
        id: "Ne_PhysWorld_RunSolveJobs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_RunStepPipeline: Binding = Binding {
        id: "Ne_PhysWorld_RunStepPipeline",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PhysWorld_UpdateBodyBroadphase: Binding = Binding {
        id: "Ne_PhysWorld_UpdateBodyBroadphase",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Phys_BuildConvexHullShapeBlob: Binding = Binding {
        id: "Ne_Phys_BuildConvexHullShapeBlob",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Phys_EnsureReflectionRegistered: Binding = Binding {
        id: "Ne_Phys_EnsureReflectionRegistered",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Phys_GridSegmentTraverse: Binding = Binding {
        id: "Ne_Phys_GridSegmentTraverse",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Phys_ObbOverlapTest: Binding = Binding {
        id: "Ne_Phys_ObbOverlapTest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Phys_ParseMaterials: Binding = Binding {
        id: "Ne_Phys_ParseMaterials",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Phys_PrepareConstraintRowsBatch: Binding = Binding {
        id: "Ne_Phys_PrepareConstraintRowsBatch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Phys_SolveRowsIterate: Binding = Binding {
        id: "Ne_Phys_SolveRowsIterate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Play_FullscreenVideo: Binding = Binding {
        id: "Ne_Play_FullscreenVideo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PlayerCustomInputProvider_BindToDevices: Binding = Binding {
        id: "Ne_PlayerCustomInputProvider_BindToDevices",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PlayerInputProvider_BindToDeviceNode: Binding = Binding {
        id: "Ne_PlayerInputProvider_BindToDeviceNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PlayerInputProvider_SetDeviceBound: Binding = Binding {
        id: "Ne_PlayerInputProvider_SetDeviceBound",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PlayerManager_DisplayUnsupportedDeviceWarning: Binding = Binding {
        id: "Ne_PlayerManager_DisplayUnsupportedDeviceWarning",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PlayerManager_HandleLeadPlayerLogoutOrChangeCallback: Binding = Binding {
        id: "Ne_PlayerManager_HandleLeadPlayerLogoutOrChangeCallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PlayerManager_PlatformProcessAllInputDevices: Binding = Binding {
        id: "Ne_PlayerManager_PlatformProcessAllInputDevices",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PlayerManager_PlatformTryAssignInputToPlayer: Binding = Binding {
        id: "Ne_PlayerManager_PlatformTryAssignInputToPlayer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Player_AddPlayerInputProvider: Binding = Binding {
        id: "Ne_Player_AddPlayerInputProvider",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Player_DestroyCustomInputProvider: Binding = Binding {
        id: "Ne_Player_DestroyCustomInputProvider",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Player_DisableCustomControls: Binding = Binding {
        id: "Ne_Player_DisableCustomControls",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Player_RemovePlayerInputProvider: Binding = Binding {
        id: "Ne_Player_RemovePlayerInputProvider",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Player_SetFFBInputDeviceId: Binding = Binding {
        id: "Ne_Player_SetFFBInputDeviceId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolAllocator_GrowChunk112: Binding = Binding {
        id: "Ne_PoolAllocator_GrowChunk112",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolAllocator_GrowChunk24: Binding = Binding {
        id: "Ne_PoolAllocator_GrowChunk24",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolAllocator_GrowChunk80: Binding = Binding {
        id: "Ne_PoolAllocator_GrowChunk80",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolList_DestroyNodeChain_A: Binding = Binding {
        id: "Ne_PoolList_DestroyNodeChain_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolList_DestroyNodeChain_B: Binding = Binding {
        id: "Ne_PoolList_DestroyNodeChain_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolList_DestroyNodes: Binding = Binding {
        id: "Ne_PoolList_DestroyNodes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolList_Resize: Binding = Binding {
        id: "Ne_PoolList_Resize",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PoolList_Resize_B: Binding = Binding {
        id: "Ne_PoolList_Resize_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Pool_ComputeBlockRange: Binding = Binding {
        id: "Ne_Pool_ComputeBlockRange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Pool_ReleaseVirtualBlock: Binding = Binding {
        id: "Ne_Pool_ReleaseVirtualBlock",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PostProcessXGenerator_Copy_Render: Binding = Binding {
        id: "Ne_PostProcessXGenerator_Copy_Render",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Post_BusMessage: Binding = Binding {
        id: "Ne_Post_BusMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Post_BusMessageQueued_FlashbackUsed: Binding = Binding {
        id: "Ne_Post_BusMessageQueued_FlashbackUsed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceEadp_ProcessPlayerCardList: Binding = Binding {
        id: "Ne_PresenceEadp_ProcessPlayerCardList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverEADP_OnBatchPlayerCardsRequestComplete: Binding = Binding {
        id: "Ne_PresenceNameResolverEADP_OnBatchPlayerCardsRequestComplete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverEADP_UpdateToResolveList: Binding = Binding {
        id: "Ne_PresenceNameResolverEADP_UpdateToResolveList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverEADP_UpdateToResolveList_Lambda: Binding = Binding {
        id: "Ne_PresenceNameResolverEADP_UpdateToResolveList_Lambda",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverEADP_VectorGrow: Binding = Binding {
        id: "Ne_PresenceNameResolverEADP_VectorGrow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverFixed_AddName: Binding = Binding {
        id: "Ne_PresenceNameResolverFixed_AddName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverFixed_InsertName: Binding = Binding {
        id: "Ne_PresenceNameResolverFixed_InsertName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverFixed_MapFatalStub: Binding = Binding {
        id: "Ne_PresenceNameResolverFixed_MapFatalStub",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceNameResolverFixed_StartResolving: Binding = Binding {
        id: "Ne_PresenceNameResolverFixed_StartResolving",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceQueue_Enqueue: Binding = Binding {
        id: "Ne_PresenceQueue_Enqueue",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PresenceQueue_VectorPush: Binding = Binding {
        id: "Ne_PresenceQueue_VectorPush",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Presence_OnSubscribeResult: Binding = Binding {
        id: "Ne_Presence_OnSubscribeResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Property_ResolveBindingChain: Binding = Binding {
        id: "Ne_Property_ResolveBindingChain",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Property_SetS32IfChanged: Binding = Binding {
        id: "Ne_Property_SetS32IfChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Property_SetS64IfChanged: Binding = Binding {
        id: "Ne_Property_SetS64IfChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_PtrList_RemoveLocked: Binding = Binding {
        id: "Ne_PtrList_RemoveLocked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Queue_EnqueueEventRecord: Binding = Binding {
        id: "Ne_Queue_EnqueueEventRecord",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RbTree_EraseSubtree: Binding = Binding {
        id: "Ne_RbTree_EraseSubtree",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RbTree_EraseSubtree_RefValues: Binding = Binding {
        id: "Ne_RbTree_EraseSubtree_RefValues",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RefCounted_Release: Binding = Binding {
        id: "Ne_RefCounted_Release",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RefPtr_MoveAssign: Binding = Binding {
        id: "Ne_RefPtr_MoveAssign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RegisteredHandle_Assign: Binding = Binding {
        id: "Ne_RegisteredHandle_Assign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderGraph_ApplyBindingsByType: Binding = Binding {
        id: "Ne_RenderGraph_ApplyBindingsByType",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderGraph_BuildPassFlagsDesc: Binding = Binding {
        id: "Ne_RenderGraph_BuildPassFlagsDesc",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderGraph_ResolveHandleFlag: Binding = Binding {
        id: "Ne_RenderGraph_ResolveHandleFlag",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderParams_ApplyGridPreset: Binding = Binding {
        id: "Ne_RenderParams_ApplyGridPreset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderParams_ApplyLoadingPreset: Binding = Binding {
        id: "Ne_RenderParams_ApplyLoadingPreset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderParams_ApplyRacePreset: Binding = Binding {
        id: "Ne_RenderParams_ApplyRacePreset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderParams_ApplyStartLightsPreset: Binding = Binding {
        id: "Ne_RenderParams_ApplyStartLightsPreset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RenderViewParams_InitDefaults: Binding = Binding {
        id: "Ne_RenderViewParams_InitDefaults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ReplayBuffer_GetOldestValidPosition: Binding = Binding {
        id: "Ne_ReplayBuffer_GetOldestValidPosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ReplayRecordable_ReadFrom: Binding = Binding {
        id: "Ne_ReplayRecordable_ReadFrom",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ReplaySystem_ForceReset: Binding = Binding {
        id: "Ne_ReplaySystem_ForceReset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ReplaySystem_ResetSlot24: Binding = Binding {
        id: "Ne_ReplaySystem_ResetSlot24",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Replay_SetState: Binding = Binding {
        id: "Ne_Replay_SetState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ResourceHandle_Init: Binding = Binding {
        id: "Ne_ResourceHandle_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ResourceMgr_ReleaseIfOwned: Binding = Binding {
        id: "Ne_ResourceMgr_ReleaseIfOwned",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ResourceRefGroup_Destruct: Binding = Binding {
        id: "Ne_ResourceRefGroup_Destruct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RichText_ConstructMixedString: Binding = Binding {
        id: "Ne_RichText_ConstructMixedString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RingDeque_PushBack_F1os: Binding = Binding {
        id: "Ne_RingDeque_PushBack_F1os",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RobinHoodMap12_Rehash: Binding = Binding {
        id: "Ne_RobinHoodMap12_Rehash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RobinHoodMap_ClearDestroy: Binding = Binding {
        id: "Ne_RobinHoodMap_ClearDestroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RobinHoodMap_Rehash_A: Binding = Binding {
        id: "Ne_RobinHoodMap_Rehash_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RobinHoodMap_Str_FindIndex8: Binding = Binding {
        id: "Ne_RobinHoodMap_Str_FindIndex8",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RobinHoodMap_Str_FindIndex_2: Binding = Binding {
        id: "Ne_RobinHoodMap_Str_FindIndex_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RobinHoodMap_U8F_Insert: Binding = Binding {
        id: "Ne_RobinHoodMap_U8F_Insert",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RttRenderer_CreateSceneJob: Binding = Binding {
        id: "Ne_RttRenderer_CreateSceneJob",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RttRenderer_FlushJobs: Binding = Binding {
        id: "Ne_RttRenderer_FlushJobs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_RttRenderer_SubmitJob: Binding = Binding {
        id: "Ne_RttRenderer_SubmitJob",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Rtt_DefaultCallback: Binding = Binding {
        id: "Ne_Rtt_DefaultCallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Rtt_RegisterCameras: Binding = Binding {
        id: "Ne_Rtt_RegisterCameras",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Rtt_UnregisterGenerators: Binding = Binding {
        id: "Ne_Rtt_UnregisterGenerators",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_CorruptFmtSUU: Binding = Binding {
        id: "Ne_SaveError_CorruptFmtSUU",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_EncryptionSourceFailed: Binding = Binding {
        id: "Ne_SaveError_EncryptionSourceFailed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_NotDirectoryFmtSS: Binding = Binding {
        id: "Ne_SaveError_NotDirectoryFmtSS",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_SelectedIndexOutOfRange: Binding = Binding {
        id: "Ne_SaveError_SelectedIndexOutOfRange",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_SetFmt: Binding = Binding {
        id: "Ne_SaveError_SetFmt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_SetFmtSSS: Binding = Binding {
        id: "Ne_SaveError_SetFmtSSS",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_SetFmtWS: Binding = Binding {
        id: "Ne_SaveError_SetFmtWS",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_ThumbnailTooLarge: Binding = Binding {
        id: "Ne_SaveError_ThumbnailTooLarge",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_TransformFailed: Binding = Binding {
        id: "Ne_SaveError_TransformFailed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_Win32FmtSSSU: Binding = Binding {
        id: "Ne_SaveError_Win32FmtSSSU",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_Win32FmtSSU: Binding = Binding {
        id: "Ne_SaveError_Win32FmtSSU",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_Win32FmtSU: Binding = Binding {
        id: "Ne_SaveError_Win32FmtSU",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveError_Win32FmtU: Binding = Binding {
        id: "Ne_SaveError_Win32FmtU",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem2Win_EmptyDirectory: Binding = Binding {
        id: "Ne_SaveSystem2Win_EmptyDirectory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem2Win_Mount: Binding = Binding {
        id: "Ne_SaveSystem2Win_Mount",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem2Win_ProcessSerialiserActions: Binding = Binding {
        id: "Ne_SaveSystem2Win_ProcessSerialiserActions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem2Win_RenameDirectory: Binding = Binding {
        id: "Ne_SaveSystem2Win_RenameDirectory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem2Win_SaveSelected: Binding = Binding {
        id: "Ne_SaveSystem2Win_SaveSelected",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem2Win_Start: Binding = Binding {
        id: "Ne_SaveSystem2Win_Start",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem2Win_WriteInfoXml: Binding = Binding {
        id: "Ne_SaveSystem2Win_WriteInfoXml",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_AutoSave: Binding = Binding {
        id: "Ne_SaveSystem_AutoSave",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_BuildSaveInfoList: Binding = Binding {
        id: "Ne_SaveSystem_BuildSaveInfoList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_CreateTempDirectory: Binding = Binding {
        id: "Ne_SaveSystem_CreateTempDirectory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_Delete: Binding = Binding {
        id: "Ne_SaveSystem_Delete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_DeleteDirectory: Binding = Binding {
        id: "Ne_SaveSystem_DeleteDirectory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_Enumerate: Binding = Binding {
        id: "Ne_SaveSystem_Enumerate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_InitDevice: Binding = Binding {
        id: "Ne_SaveSystem_InitDevice",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_ListDelete: Binding = Binding {
        id: "Ne_SaveSystem_ListDelete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_ListLoad: Binding = Binding {
        id: "Ne_SaveSystem_ListLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_ListSave: Binding = Binding {
        id: "Ne_SaveSystem_ListSave",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_SelectAndDelete: Binding = Binding {
        id: "Ne_SaveSystem_SelectAndDelete",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_SelectAndLoad: Binding = Binding {
        id: "Ne_SaveSystem_SelectAndLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_SelectAndSave: Binding = Binding {
        id: "Ne_SaveSystem_SelectAndSave",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_Terminate: Binding = Binding {
        id: "Ne_SaveSystem_Terminate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SaveSystem_ThreadProcessRequest: Binding = Binding {
        id: "Ne_SaveSystem_ThreadProcessRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SceneNode_SetTransform: Binding = Binding {
        id: "Ne_SceneNode_SetTransform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Schema_ParseFieldsA: Binding = Binding {
        id: "Ne_Schema_ParseFieldsA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Schema_ParseFieldsB: Binding = Binding {
        id: "Ne_Schema_ParseFieldsB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ScriptValue_GetFloat: Binding = Binding {
        id: "Ne_ScriptValue_GetFloat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_ScriptValue_GetString: Binding = Binding {
        id: "Ne_ScriptValue_GetString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SequenceAudioStreamPacket_PreloadAudioStream: Binding = Binding {
        id: "Ne_SequenceAudioStreamPacket_PreloadAudioStream",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SequenceClipInstance_BindOptional: Binding = Binding {
        id: "Ne_SequenceClipInstance_BindOptional",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SequenceInstance_Construct: Binding = Binding {
        id: "Ne_SequenceInstance_Construct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SequenceResource_CreateResource: Binding = Binding {
        id: "Ne_SequenceResource_CreateResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sequence_Deserialize: Binding = Binding {
        id: "Ne_Sequence_Deserialize",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Set_FixedString: Binding = Binding {
        id: "Ne_Set_FixedString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Settings_BindNamedEntries: Binding = Binding {
        id: "Ne_Settings_BindNamedEntries",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SharedPtr_CopyAssign: Binding = Binding {
        id: "Ne_SharedPtr_CopyAssign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SharedPtr_MoveAssign: Binding = Binding {
        id: "Ne_SharedPtr_MoveAssign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SharedPtr_Release: Binding = Binding {
        id: "Ne_SharedPtr_Release",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SharedPtr_ReleaseA: Binding = Binding {
        id: "Ne_SharedPtr_ReleaseA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SharedPtr_ReleaseB: Binding = Binding {
        id: "Ne_SharedPtr_ReleaseB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Skeleton_CollectDescendants: Binding = Binding {
        id: "Ne_Skeleton_CollectDescendants",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Skeleton_ComputeWorldMatrices: Binding = Binding {
        id: "Ne_Skeleton_ComputeWorldMatrices",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_Elem24: Binding = Binding {
        id: "Ne_Sort_Elem24",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_Elem32ByU64: Binding = Binding {
        id: "Ne_Sort_Elem32ByU64",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_Elem40WithPred: Binding = Binding {
        id: "Ne_Sort_Elem40WithPred",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_Elem48_Median3Order: Binding = Binding {
        id: "Ne_Sort_Elem48_Median3Order",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_IntroSort16: Binding = Binding {
        id: "Ne_Sort_IntroSort16",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_Introsort_Elem48: Binding = Binding {
        id: "Ne_Sort_Introsort_Elem48",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_Introsort_Pair16_A: Binding = Binding {
        id: "Ne_Sort_Introsort_Pair16_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sort_Introsort_Pair16_B: Binding = Binding {
        id: "Ne_Sort_Introsort_Pair16_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StdMap_At_141107280: Binding = Binding {
        id: "Ne_StdMap_At_141107280",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StdMap_At_141403b30: Binding = Binding {
        id: "Ne_StdMap_At_141403b30",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_FatalExceptionTerminate: Binding = Binding {
        id: "Ne_Std_FatalExceptionTerminate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowArraySubscript: Binding = Binding {
        id: "Ne_Std_ThrowArraySubscript",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowBadArrayNewLength: Binding = Binding {
        id: "Ne_Std_ThrowBadArrayNewLength",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowBadFunctionCall: Binding = Binding {
        id: "Ne_Std_ThrowBadFunctionCall",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowDequeTooLong: Binding = Binding {
        id: "Ne_Std_ThrowDequeTooLong",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowException: Binding = Binding {
        id: "Ne_Std_ThrowException",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowExceptionNoReturn: Binding = Binding {
        id: "Ne_Std_ThrowExceptionNoReturn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowInvalidBitsetPosition: Binding = Binding {
        id: "Ne_Std_ThrowInvalidBitsetPosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowMapSetTooLong: Binding = Binding {
        id: "Ne_Std_ThrowMapSetTooLong",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowStdException: Binding = Binding {
        id: "Ne_Std_ThrowStdException",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowStringTooLong: Binding = Binding {
        id: "Ne_Std_ThrowStringTooLong",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowVectorSubscriptError: Binding = Binding {
        id: "Ne_Std_ThrowVectorSubscriptError",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowVectorTooLong: Binding = Binding {
        id: "Ne_Std_ThrowVectorTooLong",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowVectorTooLongC: Binding = Binding {
        id: "Ne_Std_ThrowVectorTooLongC",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Std_ThrowVectorTooLong_14029b1a0: Binding = Binding {
        id: "Ne_Std_ThrowVectorTooLong_14029b1a0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SteamTransport_FlushPeerBuffers: Binding = Binding {
        id: "Ne_SteamTransport_FlushPeerBuffers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_SteamTransport_QueuePacket: Binding = Binding {
        id: "Ne_SteamTransport_QueuePacket",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Steam_OnEncryptedAppTicket: Binding = Binding {
        id: "Ne_Steam_OnEncryptedAppTicket",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StrBuf_CopyConstruct: Binding = Binding {
        id: "Ne_StrBuf_CopyConstruct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StrBuf_Printf: Binding = Binding {
        id: "Ne_StrBuf_Printf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StringMap_EmplaceNode: Binding = Binding {
        id: "Ne_StringMap_EmplaceNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StringPool_InternByHash: Binding = Binding {
        id: "Ne_StringPool_InternByHash",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StringPool_InternCStr: Binding = Binding {
        id: "Ne_StringPool_InternCStr",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StringPool_LogMemoryStats: Binding = Binding {
        id: "Ne_StringPool_LogMemoryStats",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_StringTable_Lookup: Binding = Binding {
        id: "Ne_StringTable_Lookup",
        kind: "func",
        status: Status::Stable,
    };
    pub const Ne_StringTable_LookupWithFallback: Binding = Binding {
        id: "Ne_StringTable_LookupWithFallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_String_AssignWithMode: Binding = Binding {
        id: "Ne_String_AssignWithMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_String_CloneAppendCStr: Binding = Binding {
        id: "Ne_String_CloneAppendCStr",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_String_ConstructFromCStr: Binding = Binding {
        id: "Ne_String_ConstructFromCStr",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_String_FormatToPool: Binding = Binding {
        id: "Ne_String_FormatToPool",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_String_ParseTokens: Binding = Binding {
        id: "Ne_String_ParseTokens",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_String_Reserve: Binding = Binding {
        id: "Ne_String_Reserve",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_String_Utf8ToWide: Binding = Binding {
        id: "Ne_String_Utf8ToWide",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Struct_CopyConstructInlineStrings: Binding = Binding {
        id: "Ne_Struct_CopyConstructInlineStrings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Sync_WaitObject: Binding = Binding {
        id: "Ne_Sync_WaitObject",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_TempPool_Alloc: Binding = Binding {
        id: "Ne_TempPool_Alloc",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_TerrainResource_GetMaterialResourcel: Binding = Binding {
        id: "Ne_TerrainResource_GetMaterialResourcel",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_TextRingBuffer_FlushPending: Binding = Binding {
        id: "Ne_TextRingBuffer_FlushPending",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_TextureDecompressionGeneratorFactory_ObtainGeneratorResource: Binding = Binding {
        id: "Ne_TextureDecompressionGeneratorFactory_ObtainGeneratorResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Tga_ConvertToRgba8Buffer: Binding = Binding {
        id: "Ne_Tga_ConvertToRgba8Buffer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Thread_Destroy: Binding = Binding {
        id: "Ne_Thread_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Thread_SetPriority: Binding = Binding {
        id: "Ne_Thread_SetPriority",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Thunk_ExternalCall: Binding = Binding {
        id: "Ne_Thunk_ExternalCall",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_TimeManager_OnSetFixedTimeStep: Binding = Binding {
        id: "Ne_TimeManager_OnSetFixedTimeStep",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Time_BreakDownMicroseconds: Binding = Binding {
        id: "Ne_Time_BreakDownMicroseconds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Time_FormatBrokenDownUnits: Binding = Binding {
        id: "Ne_Time_FormatBrokenDownUnits",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Timer_ConstructQpcDeadline: Binding = Binding {
        id: "Ne_Timer_ConstructQpcDeadline",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Timer_GetElapsedSeconds: Binding = Binding {
        id: "Ne_Timer_GetElapsedSeconds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Tls_GetSlotB5f0Value: Binding = Binding {
        id: "Ne_Tls_GetSlotB5f0Value",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Tls_GetThreadContext: Binding = Binding {
        id: "Ne_Tls_GetThreadContext",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Tls_GetValue: Binding = Binding {
        id: "Ne_Tls_GetValue",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Tweak_FindByName: Binding = Binding {
        id: "Ne_Tweak_FindByName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_TypedArray2_LogStats_8db0: Binding = Binding {
        id: "Ne_TypedArray2_LogStats_8db0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_U64Map_FindValue_At580: Binding = Binding {
        id: "Ne_U64Map_FindValue_At580",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIAction_C_closeScreenId: Binding = Binding {
        id: "Ne_UIAction_C_closeScreenId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UICrashDumpInfo_OnScreenEvent: Binding = Binding {
        id: "Ne_UICrashDumpInfo_OnScreenEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UICrashDumpInfo_SettingsScreenClearSelectedCategory: Binding = Binding {
        id: "Ne_UICrashDumpInfo_SettingsScreenClearSelectedCategory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIElementBase_HandleCommunicationMessage: Binding = Binding {
        id: "Ne_UIElementBase_HandleCommunicationMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIFlow_AssetGroupPreUnloaded: Binding = Binding {
        id: "Ne_UIFlow_AssetGroupPreUnloaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIFlow_AssetGroupPreloaded: Binding = Binding {
        id: "Ne_UIFlow_AssetGroupPreloaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIFlow_FlowAssetsUnloaded: Binding = Binding {
        id: "Ne_UIFlow_FlowAssetsUnloaded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIFlow_UnloadAssetsGroups: Binding = Binding {
        id: "Ne_UIFlow_UnloadAssetsGroups",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIInterrupt_HandleSignal: Binding = Binding {
        id: "Ne_UIInterrupt_HandleSignal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIMessageSystem_OnMetadataAssetCleanup: Binding = Binding {
        id: "Ne_UIMessageSystem_OnMetadataAssetCleanup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UINodeBase_OnElementNodeBind: Binding = Binding {
        id: "Ne_UINodeBase_OnElementNodeBind",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UINodeBase_OnElementNodeUnBind: Binding = Binding {
        id: "Ne_UINodeBase_OnElementNodeUnBind",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UINodeBase_OnVMSetDataContext: Binding = Binding {
        id: "Ne_UINodeBase_OnVMSetDataContext",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UINodeBase_ParentVisible: Binding = Binding {
        id: "Ne_UINodeBase_ParentVisible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UINodeBase_SetElementVisibility: Binding = Binding {
        id: "Ne_UINodeBase_SetElementVisibility",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UINodeBase_Visible: Binding = Binding {
        id: "Ne_UINodeBase_Visible",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIPCOptions_PopulateOutputMonitors: Binding = Binding {
        id: "Ne_UIPCOptions_PopulateOutputMonitors",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UITextElement_SetText: Binding = Binding {
        id: "Ne_UITextElement_SetText",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UITextElement_UpdateDocument: Binding = Binding {
        id: "Ne_UITextElement_UpdateDocument",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIVideo_SendSamplerMessage: Binding = Binding {
        id: "Ne_UIVideo_SendSamplerMessage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIVideo_SetLayer: Binding = Binding {
        id: "Ne_UIVideo_SetLayer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UIVideo_SetSource: Binding = Binding {
        id: "Ne_UIVideo_SetSource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UgcSearcher_SearchCallback: Binding = Binding {
        id: "Ne_UgcSearcher_SearchCallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UgcSearcher_StartSearch: Binding = Binding {
        id: "Ne_UgcSearcher_StartSearch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UgcUtil_BrowseWorkshopInSteamOverlay: Binding = Binding {
        id: "Ne_UgcUtil_BrowseWorkshopInSteamOverlay",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UgcUtil_OpenWorkshopFileInSteamOverlay: Binding = Binding {
        id: "Ne_UgcUtil_OpenWorkshopFileInSteamOverlay",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Ugc_DeleteTemporaryFiles: Binding = Binding {
        id: "Ne_Ugc_DeleteTemporaryFiles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Ugc_GameOverlayActivatedCallback: Binding = Binding {
        id: "Ne_Ugc_GameOverlayActivatedCallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Ugc_ReadInstalledItem: Binding = Binding {
        id: "Ne_Ugc_ReadInstalledItem",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Ugc_UpdateItemCallback: Binding = Binding {
        id: "Ne_Ugc_UpdateItemCallback",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UiEvent_DispatchByName: Binding = Binding {
        id: "Ne_UiEvent_DispatchByName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UniquePtr_MoveAssign: Binding = Binding {
        id: "Ne_UniquePtr_MoveAssign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_FindOrEmplace_Fnv8_A: Binding = Binding {
        id: "Ne_UnorderedMap_FindOrEmplace_Fnv8_A",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_FindOrEmplace_Fnv8_B: Binding = Binding {
        id: "Ne_UnorderedMap_FindOrEmplace_Fnv8_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_FindOrEmplace_Fnv8_C: Binding = Binding {
        id: "Ne_UnorderedMap_FindOrEmplace_Fnv8_C",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_FindOrEmplace_Fnv8_D: Binding = Binding {
        id: "Ne_UnorderedMap_FindOrEmplace_Fnv8_D",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_FindOrEmplace_Fnv_E: Binding = Binding {
        id: "Ne_UnorderedMap_FindOrEmplace_Fnv_E",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_RehashBuckets: Binding = Binding {
        id: "Ne_UnorderedMap_RehashBuckets",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_Rehash_Gfx2Buffer: Binding = Binding {
        id: "Ne_UnorderedMap_Rehash_Gfx2Buffer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UnorderedMap_Rehash_Gfx2DeferredCommand: Binding = Binding {
        id: "Ne_UnorderedMap_Rehash_Gfx2DeferredCommand",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UserId_Equals: Binding = Binding {
        id: "Ne_UserId_Equals",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_UserId_ToDisplayString: Binding = Binding {
        id: "Ne_UserId_ToDisplayString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Variant_FormatToString: Binding = Binding {
        id: "Ne_Variant_FormatToString",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector16_InsertGrow: Binding = Binding {
        id: "Ne_Vector16_InsertGrow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector16_PushBack: Binding = Binding {
        id: "Ne_Vector16_PushBack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector24_PushBackMove: Binding = Binding {
        id: "Ne_Vector24_PushBackMove",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector24_PushBackMove_B: Binding = Binding {
        id: "Ne_Vector24_PushBackMove_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector2A8_PushBackGrow: Binding = Binding {
        id: "Ne_Vector2A8_PushBackGrow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector80_EmplaceBack: Binding = Binding {
        id: "Ne_Vector80_EmplaceBack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector88_CopyAssign: Binding = Binding {
        id: "Ne_Vector88_CopyAssign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector96_Resize: Binding = Binding {
        id: "Ne_Vector96_Resize",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VectorOfFloatVec_Assign: Binding = Binding {
        id: "Ne_VectorOfFloatVec_Assign",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector_GrowEmplace_Elem384: Binding = Binding {
        id: "Ne_Vector_GrowEmplace_Elem384",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector_InsertGrow_Elem40: Binding = Binding {
        id: "Ne_Vector_InsertGrow_Elem40",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector_PushBack_Elem16: Binding = Binding {
        id: "Ne_Vector_PushBack_Elem16",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector_PushBack_Ptr_Locked: Binding = Binding {
        id: "Ne_Vector_PushBack_Ptr_Locked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vector_ResizeIntrusivePtrs: Binding = Binding {
        id: "Ne_Vector_ResizeIntrusivePtrs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoInstance_Close: Binding = Binding {
        id: "Ne_VideoInstance_Close",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_OpenVideo: Binding = Binding {
        id: "Ne_VideoManager_OpenVideo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_PauseVideo: Binding = Binding {
        id: "Ne_VideoManager_PauseVideo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_PlayIntervalFrames: Binding = Binding {
        id: "Ne_VideoManager_PlayIntervalFrames",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_PlayIntervalSeconds: Binding = Binding {
        id: "Ne_VideoManager_PlayIntervalSeconds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_PlayOneFrame: Binding = Binding {
        id: "Ne_VideoManager_PlayOneFrame",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_PlayVideo: Binding = Binding {
        id: "Ne_VideoManager_PlayVideo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_ResumeVideo: Binding = Binding {
        id: "Ne_VideoManager_ResumeVideo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoManager_StopVideo: Binding = Binding {
        id: "Ne_VideoManager_StopVideo",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoPlayer_OpenFile: Binding = Binding {
        id: "Ne_VideoPlayer_OpenFile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VideoResource_Destroy: Binding = Binding {
        id: "Ne_VideoResource_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Video_InitBinkThreads: Binding = Binding {
        id: "Ne_Video_InitBinkThreads",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_VmNode_SetFloat: Binding = Binding {
        id: "Ne_VmNode_SetFloat",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_Vsnprintf: Binding = Binding {
        id: "Ne_Vsnprintf",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_BuildResolutionOptions: Binding = Binding {
        id: "Ne_WindowWin_BuildResolutionOptions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_MonitorEnumProc: Binding = Binding {
        id: "Ne_WindowWin_MonitorEnumProc",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_ResizeTarget: Binding = Binding {
        id: "Ne_WindowWin_ResizeTarget",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_SaveWindowPosition: Binding = Binding {
        id: "Ne_WindowWin_SaveWindowPosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_SetAspectRatio: Binding = Binding {
        id: "Ne_WindowWin_SetAspectRatio",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_SetOnMinimised: Binding = Binding {
        id: "Ne_WindowWin_SetOnMinimised",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_Startup: Binding = Binding {
        id: "Ne_WindowWin_Startup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_ToggleFullScreen: Binding = Binding {
        id: "Ne_WindowWin_ToggleFullScreen",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_Update: Binding = Binding {
        id: "Ne_WindowWin_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WindowWin_WinProc: Binding = Binding {
        id: "Ne_WindowWin_WinProc",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorkQueue_PushAndSignal: Binding = Binding {
        id: "Ne_WorkQueue_PushAndSignal",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorldInstance_ActivateAllWorldObjects: Binding = Binding {
        id: "Ne_WorldInstance_ActivateAllWorldObjects",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorldInstance_DeactivateAllWorldObjects: Binding = Binding {
        id: "Ne_WorldInstance_DeactivateAllWorldObjects",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorldInstance_DestroyAllWorldObjectsInLayer: Binding = Binding {
        id: "Ne_WorldInstance_DestroyAllWorldObjectsInLayer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorldInstance_PreWarmTextures: Binding = Binding {
        id: "Ne_WorldInstance_PreWarmTextures",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorldInstance_ReleaseWarmedTextures: Binding = Binding {
        id: "Ne_WorldInstance_ReleaseWarmedTextures",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorldLayerModule_DeregisterLayer: Binding = Binding {
        id: "Ne_WorldLayerModule_DeregisterLayer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Ne_WorldLoadTask_Update: Binding = Binding {
        id: "Ne_WorldLoadTask_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_BackBuffer_CopyToCpu: Binding = Binding {
        id: "Render_BackBuffer_CopyToCpu",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Batch_Submit: Binding = Binding {
        id: "Render_Batch_Submit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_CloudGenerator_Update: Binding = Binding {
        id: "Render_CloudGenerator_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_DDGISystem_Render: Binding = Binding {
        id: "Render_DDGISystem_Render",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_DamageMaskGenerator_Render: Binding = Binding {
        id: "Render_DamageMaskGenerator_Render",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Gfx2Buffer_Construct: Binding = Binding {
        id: "Render_Gfx2Buffer_Construct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Gfx2Buffer_Destroy: Binding = Binding {
        id: "Render_Gfx2Buffer_Destroy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Gfx2Device_Initialise: Binding = Binding {
        id: "Render_Gfx2Device_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_GroundCoverPlugin_Render: Binding = Binding {
        id: "Render_GroundCoverPlugin_Render",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Hbcn_OnCompressRequest: Binding = Binding {
        id: "Render_Hbcn_OnCompressRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_LightListResource_Create: Binding = Binding {
        id: "Render_LightListResource_Create",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Model_ValidateMaterialParams: Binding = Binding {
        id: "Render_Model_ValidateMaterialParams",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_OnRaytracingValidationMsg: Binding = Binding {
        id: "Render_OnRaytracingValidationMsg",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_ParticleManager_DrawMeshParticles: Binding = Binding {
        id: "Render_ParticleManager_DrawMeshParticles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Particles_ReadbackVertexBuffer: Binding = Binding {
        id: "Render_Particles_ReadbackVertexBuffer",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_RainFile_Load: Binding = Binding {
        id: "Render_RainFile_Load",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_RainFile_ParseMaterials: Binding = Binding {
        id: "Render_RainFile_ParseMaterials",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_SceneView_SubmitCamera: Binding = Binding {
        id: "Render_SceneView_SubmitCamera",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Screenshot_CaptureAndSave: Binding = Binding {
        id: "Render_Screenshot_CaptureAndSave",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_TerrainPlugin_Render: Binding = Binding {
        id: "Render_TerrainPlugin_Render",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Texture_CreateFromResource: Binding = Binding {
        id: "Render_Texture_CreateFromResource",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_Write_PngFile: Binding = Binding {
        id: "Render_Write_PngFile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_XeSS_Evaluate: Binding = Binding {
        id: "Render_XeSS_Evaluate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Render_XeSS_PrepareInputs: Binding = Binding {
        id: "Render_XeSS_PrepareInputs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Apply_ContentAvailability: Binding = Binding {
        id: "Sess_Apply_ContentAvailability",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Apply_DecidedPenalties: Binding = Binding {
        id: "Sess_Apply_DecidedPenalties",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Assign_GarageSlotsByTeamOrder: Binding = Binding {
        id: "Sess_Assign_GarageSlotsByTeamOrder",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_AutoCrossRuleset_PlaceVehicles: Binding = Binding {
        id: "Sess_AutoCrossRuleset_PlaceVehicles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Build_ChampionshipEntrantSelection: Binding = Binding {
        id: "Sess_Build_ChampionshipEntrantSelection",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Build_GroupResult: Binding = Binding {
        id: "Sess_Build_GroupResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Build_SafetyCarLogicGraph: Binding = Binding {
        id: "Sess_Build_SafetyCarLogicGraph",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ChallengeRuleset_UpdateLapDelta: Binding = Binding {
        id: "Sess_ChallengeRuleset_UpdateLapDelta",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Classification_CompareEntries: Binding = Binding {
        id: "Sess_Classification_CompareEntries",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Collect_CareerSetupEntrantLists: Binding = Binding {
        id: "Sess_Collect_CareerSetupEntrantLists",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Collect_EntrantsForSetupMode: Binding = Binding {
        id: "Sess_Collect_EntrantsForSetupMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Collect_FilteredEntrants: Binding = Binding {
        id: "Sess_Collect_FilteredEntrants",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Complete_Update: Binding = Binding {
        id: "Sess_Complete_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ContentAvailability_Rebuild: Binding = Binding {
        id: "Sess_ContentAvailability_Rebuild",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ContentList_AddUniqueByte: Binding = Binding {
        id: "Sess_ContentList_AddUniqueByte",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ContentList_AddUniqueTriple: Binding = Binding {
        id: "Sess_ContentList_AddUniqueTriple",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ControllerSet_ClearSessionNotStarted: Binding = Binding {
        id: "Sess_ControllerSet_ClearSessionNotStarted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ControllerSystem_ApplyInputs: Binding = Binding {
        id: "Sess_ControllerSystem_ApplyInputs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ControllerSystem_BuildSets: Binding = Binding {
        id: "Sess_ControllerSystem_BuildSets",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ControllerSystem_DestroySets: Binding = Binding {
        id: "Sess_ControllerSystem_DestroySets",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Controllers_ReleaseForRaceStart: Binding = Binding {
        id: "Sess_Controllers_ReleaseForRaceStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_CopyOptionsToState: Binding = Binding {
        id: "Sess_CopyOptionsToState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_DelayOnLoadingScreen_Update: Binding = Binding {
        id: "Sess_DelayOnLoadingScreen_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_DoughnutSequence_Enter: Binding = Binding {
        id: "Sess_DoughnutSequence_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_DoughnutSequence_Update: Binding = Binding {
        id: "Sess_DoughnutSequence_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Doughnut_SetWinnerText: Binding = Binding {
        id: "Sess_Doughnut_SetWinnerText",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_DriveThroughPenalty_ReceiveData: Binding = Binding {
        id: "Sess_DriveThroughPenalty_ReceiveData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_DriveThroughPenalty_SendData: Binding = Binding {
        id: "Sess_DriveThroughPenalty_SendData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_DriveThroughPenalty_SendData_2: Binding = Binding {
        id: "Sess_DriveThroughPenalty_SendData_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_EditMode_OnStateEnter: Binding = Binding {
        id: "Sess_EditMode_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_EditMode_OnStateExit: Binding = Binding {
        id: "Sess_EditMode_OnStateExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ElimRuleset_FinaliseResults: Binding = Binding {
        id: "Sess_ElimRuleset_FinaliseResults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ElimRuleset_ForceFinishAll: Binding = Binding {
        id: "Sess_ElimRuleset_ForceFinishAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ElimRuleset_RetireVehicle: Binding = Binding {
        id: "Sess_ElimRuleset_RetireVehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ElimTracker_AddVehicle: Binding = Binding {
        id: "Sess_ElimTracker_AddVehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_EntrantFilterSet_CopyCtor: Binding = Binding {
        id: "Sess_EntrantFilterSet_CopyCtor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_EntrantVec_PushBack: Binding = Binding {
        id: "Sess_EntrantVec_PushBack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Entrant_BuildCandidate: Binding = Binding {
        id: "Sess_Entrant_BuildCandidate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Entrant_IsTeamDriverMatch: Binding = Binding {
        id: "Sess_Entrant_IsTeamDriverMatch",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Entrant_PassesFilter: Binding = Binding {
        id: "Sess_Entrant_PassesFilter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Entrants_RemoveDuplicates: Binding = Binding {
        id: "Sess_Entrants_RemoveDuplicates",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Find_GroupRowPosition: Binding = Binding {
        id: "Sess_Find_GroupRowPosition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Flags_ApplyVehicleYellow: Binding = Binding {
        id: "Sess_Flags_ApplyVehicleYellow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Flags_GetSecondsSinceEvent: Binding = Binding {
        id: "Sess_Flags_GetSecondsSinceEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Flags_PlayZoneFlagEvent: Binding = Binding {
        id: "Sess_Flags_PlayZoneFlagEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Flags_ResetSessionTypeIndex: Binding = Binding {
        id: "Sess_Flags_ResetSessionTypeIndex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Flags_SetVehicleYellow: Binding = Binding {
        id: "Sess_Flags_SetVehicleYellow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Flags_ShowBlueFlag: Binding = Binding {
        id: "Sess_Flags_ShowBlueFlag",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Flow_ParseXml: Binding = Binding {
        id: "Sess_Flow_ParseXml",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_CheckAllParked: Binding = Binding {
        id: "Sess_FormationLapLogic_CheckAllParked",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_Construct: Binding = Binding {
        id: "Sess_FormationLapLogic_Construct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_IsActiveOtherType: Binding = Binding {
        id: "Sess_FormationLapLogic_IsActiveOtherType",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_IsActiveType0: Binding = Binding {
        id: "Sess_FormationLapLogic_IsActiveType0",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_IsRunning: Binding = Binding {
        id: "Sess_FormationLapLogic_IsRunning",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_Reset: Binding = Binding {
        id: "Sess_FormationLapLogic_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_SubscribePhotoModeDisabled: Binding = Binding {
        id: "Sess_FormationLapLogic_SubscribePhotoModeDisabled",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_SubscribePhotoModeEnabled: Binding = Binding {
        id: "Sess_FormationLapLogic_SubscribePhotoModeEnabled",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_SubscribeSafetyCarDeployment: Binding = Binding {
        id: "Sess_FormationLapLogic_SubscribeSafetyCarDeployment",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapLogic_Update: Binding = Binding {
        id: "Sess_FormationLapLogic_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapStartLights_OnStateEnter: Binding = Binding {
        id: "Sess_FormationLapStartLights_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLapType_Update: Binding = Binding {
        id: "Sess_FormationLapType_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLap_OnStateExit: Binding = Binding {
        id: "Sess_FormationLap_OnStateExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FormationLap_Update: Binding = Binding {
        id: "Sess_FormationLap_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_FreezeAllVehicles: Binding = Binding {
        id: "Sess_FreezeAllVehicles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_GameWeatherSystem_SetActiveSessionType: Binding = Binding {
        id: "Sess_GameWeatherSystem_SetActiveSessionType",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_GameWeatherSystem_SetupSession: Binding = Binding {
        id: "Sess_GameWeatherSystem_SetupSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_GameWeatherSystem_UpdateRandomSeed: Binding = Binding {
        id: "Sess_GameWeatherSystem_UpdateRandomSeed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_GarageSlotVec_PushBack: Binding = Binding {
        id: "Sess_GarageSlotVec_PushBack",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_GarageState_OnStateExit: Binding = Binding {
        id: "Sess_GarageState_OnStateExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_GarageState_RefreshHudVisibility: Binding = Binding {
        id: "Sess_GarageState_RefreshHudVisibility",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Garage_HasPendingName: Binding = Binding {
        id: "Sess_Garage_HasPendingName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Get_CarDriverName: Binding = Binding {
        id: "Sess_Get_CarDriverName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Get_PlayerVehicleIndex: Binding = Binding {
        id: "Sess_Get_PlayerVehicleIndex",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_GridRundown_Update: Binding = Binding {
        id: "Sess_GridRundown_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_InGamePQ_Update: Binding = Binding {
        id: "Sess_InGamePQ_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_InGameRace_OnStateExit: Binding = Binding {
        id: "Sess_InGameRace_OnStateExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_InGameRace_Update: Binding = Binding {
        id: "Sess_InGameRace_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_InGame_OnStateExitHideHud: Binding = Binding {
        id: "Sess_InGame_OnStateExitHideHud",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_InGame_SubscribeMessages: Binding = Binding {
        id: "Sess_InGame_SubscribeMessages",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_InGame_UnsubscribeMessages: Binding = Binding {
        id: "Sess_InGame_UnsubscribeMessages",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_InGame_UpdateBase: Binding = Binding {
        id: "Sess_InGame_UpdateBase",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_IntroVideo_CanFinish: Binding = Binding {
        id: "Sess_IntroVideo_CanFinish",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_IntroVideo_OnStateEnter: Binding = Binding {
        id: "Sess_IntroVideo_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_IntroVideo_OnVideoDoneFadeIn: Binding = Binding {
        id: "Sess_IntroVideo_OnVideoDoneFadeIn",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_IntroVideo_PushAllocRequest: Binding = Binding {
        id: "Sess_IntroVideo_PushAllocRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Invalidate_Lap_Thunk: Binding = Binding {
        id: "Sess_Invalidate_Lap_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Issue_DriveThrough: Binding = Binding {
        id: "Sess_Issue_DriveThrough",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Issue_GridPlacePenalty: Binding = Binding {
        id: "Sess_Issue_GridPlacePenalty",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Issue_PostRaceTimePenalty: Binding = Binding {
        id: "Sess_Issue_PostRaceTimePenalty",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Issue_TimePenalty: Binding = Binding {
        id: "Sess_Issue_TimePenalty",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Issue_Warning: Binding = Binding {
        id: "Sess_Issue_Warning",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapRecord_Construct: Binding = Binding {
        id: "Sess_LapRecord_Construct",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapRecords_SortByTime: Binding = Binding {
        id: "Sess_LapRecords_SortByTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapSim_PrepareVehicleParams: Binding = Binding {
        id: "Sess_LapSim_PrepareVehicleParams",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_BuildSectorComparison: Binding = Binding {
        id: "Sess_LapTimes_BuildSectorComparison",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_CheckPersonalBestSector: Binding = Binding {
        id: "Sess_LapTimes_CheckPersonalBestSector",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_GetCarData: Binding = Binding {
        id: "Sess_LapTimes_GetCarData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_IsNewFastestLap: Binding = Binding {
        id: "Sess_LapTimes_IsNewFastestLap",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_OnNewSector: Binding = Binding {
        id: "Sess_LapTimes_OnNewSector",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_ReportNewLapTime: Binding = Binding {
        id: "Sess_LapTimes_ReportNewLapTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_ReportSplit: Binding = Binding {
        id: "Sess_LapTimes_ReportSplit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_Reset: Binding = Binding {
        id: "Sess_LapTimes_Reset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LapTimes_SendLapHistory: Binding = Binding {
        id: "Sess_LapTimes_SendLapHistory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Load_GameSetupBranding: Binding = Binding {
        id: "Sess_Load_GameSetupBranding",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Load_PhysicalTyreCompounds: Binding = Binding {
        id: "Sess_Load_PhysicalTyreCompounds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Load_RuleSettingsProfiles: Binding = Binding {
        id: "Sess_Load_RuleSettingsProfiles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Load_Teams: Binding = Binding {
        id: "Sess_Load_Teams",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Load_TrackDatabase: Binding = Binding {
        id: "Sess_Load_TrackDatabase",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Load_TyreCompoundClasses: Binding = Binding {
        id: "Sess_Load_TyreCompoundClasses",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Load_VisualTyreCompounds: Binding = Binding {
        id: "Sess_Load_VisualTyreCompounds",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_BuildTrackRegionList: Binding = Binding {
        id: "Sess_Loader_BuildTrackRegionList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_PlayGridPlayerShotCutscene: Binding = Binding {
        id: "Sess_Loader_PlayGridPlayerShotCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_PrepareGridThumbnails: Binding = Binding {
        id: "Sess_Loader_PrepareGridThumbnails",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RemoveListener: Binding = Binding {
        id: "Sess_Loader_RemoveListener",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RequestCareerHubLoad: Binding = Binding {
        id: "Sess_Loader_RequestCareerHubLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RequestCareerHubUnload: Binding = Binding {
        id: "Sess_Loader_RequestCareerHubUnload",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RequestLoadThumbnails: Binding = Binding {
        id: "Sess_Loader_RequestLoadThumbnails",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RequestSessionLoad: Binding = Binding {
        id: "Sess_Loader_RequestSessionLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RequestSessionUnload: Binding = Binding {
        id: "Sess_Loader_RequestSessionUnload",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RequestTrackLoad: Binding = Binding {
        id: "Sess_Loader_RequestTrackLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_RequestTrackUnload: Binding = Binding {
        id: "Sess_Loader_RequestTrackUnload",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_SetupTrackAssets: Binding = Binding {
        id: "Sess_Loader_SetupTrackAssets",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_StartFrontendAssetLoad: Binding = Binding {
        id: "Sess_Loader_StartFrontendAssetLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_UnloadSession: Binding = Binding {
        id: "Sess_Loader_UnloadSession",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Loader_Update: Binding = Binding {
        id: "Sess_Loader_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LocationCutscene_HandleStoryEvent: Binding = Binding {
        id: "Sess_LocationCutscene_HandleStoryEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LocationCutscene_SetInputBlockBit: Binding = Binding {
        id: "Sess_LocationCutscene_SetInputBlockBit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_LocationLoadCutscene_OnStateUpdate: Binding = Binding {
        id: "Sess_LocationLoadCutscene_OnStateUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Mark_EntrantListAvailable: Binding = Binding {
        id: "Sess_Mark_EntrantListAvailable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MarshalLights_Set: Binding = Binding {
        id: "Sess_MarshalLights_Set",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MarshalLights_UpdateZoneShader: Binding = Binding {
        id: "Sess_MarshalLights_UpdateZoneShader",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MarshalZoneMonitor_Init: Binding = Binding {
        id: "Sess_MarshalZoneMonitor_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MarshalZone_CheckFlags: Binding = Binding {
        id: "Sess_MarshalZone_CheckFlags",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MarshalZone_UpdateZoneVehicles: Binding = Binding {
        id: "Sess_MarshalZone_UpdateZoneVehicles",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MisconductManager_Ctor: Binding = Binding {
        id: "Sess_MisconductManager_Ctor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MisconductTable_Ctor: Binding = Binding {
        id: "Sess_MisconductTable_Ctor",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_AddTableRow: Binding = Binding {
        id: "Sess_Misconduct_AddTableRow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_CheckEnvironmentContact: Binding = Binding {
        id: "Sess_Misconduct_CheckEnvironmentContact",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_CheckWallRiding: Binding = Binding {
        id: "Sess_Misconduct_CheckWallRiding",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_DumpTable: Binding = Binding {
        id: "Sess_Misconduct_DumpTable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_EvaluateInfringement: Binding = Binding {
        id: "Sess_Misconduct_EvaluateInfringement",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_GetDriverState: Binding = Binding {
        id: "Sess_Misconduct_GetDriverState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_GetWatchState: Binding = Binding {
        id: "Sess_Misconduct_GetWatchState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_HandBackIllegalOvertake: Binding = Binding {
        id: "Sess_Misconduct_HandBackIllegalOvertake",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_IsWatchedVehicleActive: Binding = Binding {
        id: "Sess_Misconduct_IsWatchedVehicleActive",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_OnCarToCarCollision: Binding = Binding {
        id: "Sess_Misconduct_OnCarToCarCollision",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_RegisterReporter: Binding = Binding {
        id: "Sess_Misconduct_RegisterReporter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Misconduct_RemoveWatchedIncident: Binding = Binding {
        id: "Sess_Misconduct_RemoveWatchedIncident",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MultipleWarningsPenalty_ReceiveData: Binding = Binding {
        id: "Sess_MultipleWarningsPenalty_ReceiveData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MultipleWarningsPenalty_SendData: Binding = Binding {
        id: "Sess_MultipleWarningsPenalty_SendData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_MultipleWarningsPenalty_SendData_2: Binding = Binding {
        id: "Sess_MultipleWarningsPenalty_SendData_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_NetJournalMonitorState_OnExit: Binding = Binding {
        id: "Sess_NetJournalMonitorState_OnExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ObfValue_SetWithHistory: Binding = Binding {
        id: "Sess_ObfValue_SetWithHistory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_OnTrack_SetMessageHandlers: Binding = Binding {
        id: "Sess_OnTrack_SetMessageHandlers",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_OsqTeleportToStart: Binding = Binding {
        id: "Sess_OsqTeleportToStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_OvertakeLog_SendAllRecords: Binding = Binding {
        id: "Sess_OvertakeLog_SendAllRecords",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PQRuleset_FinaliseResults: Binding = Binding {
        id: "Sess_PQRuleset_FinaliseResults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Parse_CarStateTransform: Binding = Binding {
        id: "Sess_Parse_CarStateTransform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Parse_EntrantTemplateRef: Binding = Binding {
        id: "Sess_Parse_EntrantTemplateRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Parse_PlayerDescription: Binding = Binding {
        id: "Sess_Parse_PlayerDescription",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Parse_PlayerDriverRef: Binding = Binding {
        id: "Sess_Parse_PlayerDriverRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_FormatInfringementText: Binding = Binding {
        id: "Sess_PenaltyLog_FormatInfringementText",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_FormatPenaltyText: Binding = Binding {
        id: "Sess_PenaltyLog_FormatPenaltyText",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_Print: Binding = Binding {
        id: "Sess_PenaltyLog_Print",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_ReceiveAddRecord: Binding = Binding {
        id: "Sess_PenaltyLog_ReceiveAddRecord",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_RemoveVehicle: Binding = Binding {
        id: "Sess_PenaltyLog_RemoveVehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_SendAllRecords: Binding = Binding {
        id: "Sess_PenaltyLog_SendAllRecords",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_SendRecord: Binding = Binding {
        id: "Sess_PenaltyLog_SendRecord",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PenaltyLog_SendRecord_2: Binding = Binding {
        id: "Sess_PenaltyLog_SendRecord_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Penalty_ApplyBlackFlagDsq: Binding = Binding {
        id: "Sess_Penalty_ApplyBlackFlagDsq",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PersistStream_LoadFromFile: Binding = Binding {
        id: "Sess_PersistStream_LoadFromFile",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PitStopUpdateJob_Update: Binding = Binding {
        id: "Sess_PitStopUpdateJob_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PlayCutScene_OnStateEnter: Binding = Binding {
        id: "Sess_PlayCutScene_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PlayerDescList_Append: Binding = Binding {
        id: "Sess_PlayerDescList_Append",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PlayerTerminalDamage_Enter: Binding = Binding {
        id: "Sess_PlayerTerminalDamage_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PostRaceSceneSetup_OnStateEnter: Binding = Binding {
        id: "Sess_PostRaceSceneSetup_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PreLights_OnStateEnter: Binding = Binding {
        id: "Sess_PreLights_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PreLights_Update: Binding = Binding {
        id: "Sess_PreLights_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PreRaceGridState_OnEnter: Binding = Binding {
        id: "Sess_PreRaceGridState_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PreRaceToRace_Update: Binding = Binding {
        id: "Sess_PreRaceToRace_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PreRace_OnStateExit: Binding = Binding {
        id: "Sess_PreRace_OnStateExit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PreRace_PlayGridPlayerShotCutscene: Binding = Binding {
        id: "Sess_PreRace_PlayGridPlayerShotCutscene",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PreRace_Update: Binding = Binding {
        id: "Sess_PreRace_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Progress_ComputeTimeGap: Binding = Binding {
        id: "Sess_Progress_ComputeTimeGap",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Progress_SyncResultRecord: Binding = Binding {
        id: "Sess_Progress_SyncResultRecord",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PuPanel_FillComponent: Binding = Binding {
        id: "Sess_PuPanel_FillComponent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PursuitRuleset_FinaliseResults: Binding = Binding {
        id: "Sess_PursuitRuleset_FinaliseResults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_PursuitRuleset_ResetState: Binding = Binding {
        id: "Sess_PursuitRuleset_ResetState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RC_AheadOfLeadUpdate: Binding = Binding {
        id: "Sess_RC_AheadOfLeadUpdate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RC_HandleAheadOfLeadData: Binding = Binding {
        id: "Sess_RC_HandleAheadOfLeadData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RC_SendAheadOfLeadData: Binding = Binding {
        id: "Sess_RC_SendAheadOfLeadData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceRuleset_GetTimeRemaining: Binding = Binding {
        id: "Sess_RaceRuleset_GetTimeRemaining",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceRuleset_OnLapCompleted: Binding = Binding {
        id: "Sess_RaceRuleset_OnLapCompleted",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceRuleset_SetTimeLimit: Binding = Binding {
        id: "Sess_RaceRuleset_SetTimeLimit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceState_ResetTimingAndTowState: Binding = Binding {
        id: "Sess_RaceState_ResetTimingAndTowState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceState_VehicleHasNamedEntry: Binding = Binding {
        id: "Sess_RaceState_VehicleHasNamedEntry",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_BuildPitPlan: Binding = Binding {
        id: "Sess_RaceStrategy_BuildPitPlan",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_BuildRedFlagStints: Binding = Binding {
        id: "Sess_RaceStrategy_BuildRedFlagStints",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_CopyPlanStints: Binding = Binding {
        id: "Sess_RaceStrategy_CopyPlanStints",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_EvaluateStintTyre: Binding = Binding {
        id: "Sess_RaceStrategy_EvaluateStintTyre",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_FinalisePlan: Binding = Binding {
        id: "Sess_RaceStrategy_FinalisePlan",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_GetCurrentTyreSet: Binding = Binding {
        id: "Sess_RaceStrategy_GetCurrentTyreSet",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_Initialise: Binding = Binding {
        id: "Sess_RaceStrategy_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_IsPlanValid: Binding = Binding {
        id: "Sess_RaceStrategy_IsPlanValid",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_SetPlanType: Binding = Binding {
        id: "Sess_RaceStrategy_SetPlanType",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_SortCandidatesA: Binding = Binding {
        id: "Sess_RaceStrategy_SortCandidatesA",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_SortCandidatesB: Binding = Binding {
        id: "Sess_RaceStrategy_SortCandidatesB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_SortCandidatesC: Binding = Binding {
        id: "Sess_RaceStrategy_SortCandidatesC",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RaceStrategy_UpdateVehicle: Binding = Binding {
        id: "Sess_RaceStrategy_UpdateVehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Race_ComputeUnlapLapState: Binding = Binding {
        id: "Sess_Race_ComputeUnlapLapState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Read_DeploymentPoints: Binding = Binding {
        id: "Sess_Read_DeploymentPoints",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Read_LoosePartsRules: Binding = Binding {
        id: "Sess_Read_LoosePartsRules",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Record_GroupTimes: Binding = Binding {
        id: "Sess_Record_GroupTimes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Recordable_Register: Binding = Binding {
        id: "Sess_Recordable_Register",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Recordables_AddUnique: Binding = Binding {
        id: "Sess_Recordables_AddUnique",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagFreezeVehicles_Enter: Binding = Binding {
        id: "Sess_RedFlagFreezeVehicles_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagPerformRepairs_Update: Binding = Binding {
        id: "Sess_RedFlagPerformRepairs_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagRaceStrategy_Enter: Binding = Binding {
        id: "Sess_RedFlagRaceStrategy_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagRaceStrategy_Update: Binding = Binding {
        id: "Sess_RedFlagRaceStrategy_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSimulate_Enter: Binding = Binding {
        id: "Sess_RedFlagSimulate_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSimulate_ResetEntries: Binding = Binding {
        id: "Sess_RedFlagSimulate_ResetEntries",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_Deploy: Binding = Binding {
        id: "Sess_RedFlagSystem_Deploy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_OnSessionBeginGameplay: Binding = Binding {
        id: "Sess_RedFlagSystem_OnSessionBeginGameplay",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_OnStartLightsBegin: Binding = Binding {
        id: "Sess_RedFlagSystem_OnStartLightsBegin",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_PerformRepairs: Binding = Binding {
        id: "Sess_RedFlagSystem_PerformRepairs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_PrepareRestart: Binding = Binding {
        id: "Sess_RedFlagSystem_PrepareRestart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_ResetCarStatus: Binding = Binding {
        id: "Sess_RedFlagSystem_ResetCarStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_SimulateSessionTime: Binding = Binding {
        id: "Sess_RedFlagSystem_SimulateSessionTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagSystem_SnapshotDamage: Binding = Binding {
        id: "Sess_RedFlagSystem_SnapshotDamage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagToRestart_Enter: Binding = Binding {
        id: "Sess_RedFlagToRestart_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagToRestart_Exit: Binding = Binding {
        id: "Sess_RedFlagToRestart_Exit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlagToRestart_Update: Binding = Binding {
        id: "Sess_RedFlagToRestart_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlag_InitSimVehicleState: Binding = Binding {
        id: "Sess_RedFlag_InitSimVehicleState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlag_ResetVehicleTyreState: Binding = Binding {
        id: "Sess_RedFlag_ResetVehicleTyreState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RedFlag_SimulateUnlapLaps: Binding = Binding {
        id: "Sess_RedFlag_SimulateUnlapLaps",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Register_EntrantSetupMode: Binding = Binding {
        id: "Sess_Register_EntrantSetupMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Register_RaceRecordables: Binding = Binding {
        id: "Sess_Register_RaceRecordables",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Registry_GetFormationLapLogic: Binding = Binding {
        id: "Sess_Registry_GetFormationLapLogic",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ResetReplay_OnStateEnter: Binding = Binding {
        id: "Sess_ResetReplay_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ResetReplay_Update: Binding = Binding {
        id: "Sess_ResetReplay_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Reset_OnTrackRuntimeState: Binding = Binding {
        id: "Sess_Reset_OnTrackRuntimeState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ResolveEntityHandle: Binding = Binding {
        id: "Sess_ResolveEntityHandle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Resolve_DriverRef: Binding = Binding {
        id: "Sess_Resolve_DriverRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Resolve_PlayerVehicleRef: Binding = Binding {
        id: "Sess_Resolve_PlayerVehicleRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Resolve_SelectionDomainId: Binding = Binding {
        id: "Sess_Resolve_SelectionDomainId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Resolve_TrackLayerId: Binding = Binding {
        id: "Sess_Resolve_TrackLayerId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Restart_ResetGridLineup: Binding = Binding {
        id: "Sess_Restart_ResetGridLineup",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ResultRecord_FillFromLapHistory: Binding = Binding {
        id: "Sess_ResultRecord_FillFromLapHistory",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Result_ChangeVehicleStatus: Binding = Binding {
        id: "Sess_Result_ChangeVehicleStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Result_DecodeVehicleResult: Binding = Binding {
        id: "Sess_Result_DecodeVehicleResult",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Result_RequestDNF: Binding = Binding {
        id: "Sess_Result_RequestDNF",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Result_RequestStatus: Binding = Binding {
        id: "Sess_Result_RequestStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Result_RequestVehicleStatus: Binding = Binding {
        id: "Sess_Result_RequestVehicleStatus",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RetirementSequence_OnEnter: Binding = Binding {
        id: "Sess_RetirementSequence_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RetirementSequence_PlayTerminalDamage: Binding = Binding {
        id: "Sess_RetirementSequence_PlayTerminalDamage",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RetirementSequence_ShouldEnd: Binding = Binding {
        id: "Sess_RetirementSequence_ShouldEnd",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RetirementSequence_Update: Binding = Binding {
        id: "Sess_RetirementSequence_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RuleManager_HandleSessionEvents: Binding = Binding {
        id: "Sess_RuleManager_HandleSessionEvents",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RuleManager_Initialise: Binding = Binding {
        id: "Sess_RuleManager_Initialise",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_RuleManager_ResolveSubsystems: Binding = Binding {
        id: "Sess_RuleManager_ResolveSubsystems",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ApplyBlackFlag: Binding = Binding {
        id: "Sess_Rules_ApplyBlackFlag",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ApplyPostSessionPenalties: Binding = Binding {
        id: "Sess_Rules_ApplyPostSessionPenalties",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ClearVehicleRuleFlags: Binding = Binding {
        id: "Sess_Rules_ClearVehicleRuleFlags",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_CornerCuttingTimeGained_2: Binding = Binding {
        id: "Sess_Rules_CornerCuttingTimeGained_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_Disqualify: Binding = Binding {
        id: "Sess_Rules_Disqualify",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_DisqualifyForPenaltyPoints: Binding = Binding {
        id: "Sess_Rules_DisqualifyForPenaltyPoints",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_GetBlackFlagSecondsLeft: Binding = Binding {
        id: "Sess_Rules_GetBlackFlagSecondsLeft",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_IsPracticeQualifying: Binding = Binding {
        id: "Sess_Rules_IsPracticeQualifying",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_IssueDisqualification: Binding = Binding {
        id: "Sess_Rules_IssueDisqualification",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_LookupPenaltyValue: Binding = Binding {
        id: "Sess_Rules_LookupPenaltyValue",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_OnPaceCarLapForVehicle: Binding = Binding {
        id: "Sess_Rules_OnPaceCarLapForVehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_PaceCarNewLap: Binding = Binding {
        id: "Sess_Rules_PaceCarNewLap",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_QueueInfringementEvent: Binding = Binding {
        id: "Sess_Rules_QueueInfringementEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadBlueFlagRules: Binding = Binding {
        id: "Sess_Rules_ReadBlueFlagRules",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadCollisionExpectedSpeed: Binding = Binding {
        id: "Sess_Rules_ReadCollisionExpectedSpeed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadCollisionSettings: Binding = Binding {
        id: "Sess_Rules_ReadCollisionSettings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadCornerCuttingSettings: Binding = Binding {
        id: "Sess_Rules_ReadCornerCuttingSettings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadFormationLapMiniGame: Binding = Binding {
        id: "Sess_Rules_ReadFormationLapMiniGame",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadOffTrackTimeSettings: Binding = Binding {
        id: "Sess_Rules_ReadOffTrackTimeSettings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadPerClassSettings: Binding = Binding {
        id: "Sess_Rules_ReadPerClassSettings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadPitWhiteLine: Binding = Binding {
        id: "Sess_Rules_ReadPitWhiteLine",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadRedFlagRepairTimes: Binding = Binding {
        id: "Sess_Rules_ReadRedFlagRepairTimes",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadSafetyCarPacePenalties: Binding = Binding {
        id: "Sess_Rules_ReadSafetyCarPacePenalties",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_ReadYellowFlagOvertaking: Binding = Binding {
        id: "Sess_Rules_ReadYellowFlagOvertaking",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_RemoveInvalidOvertake: Binding = Binding {
        id: "Sess_Rules_RemoveInvalidOvertake",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_SendBlackFlagTimerToOsd: Binding = Binding {
        id: "Sess_Rules_SendBlackFlagTimerToOsd",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Rules_SetCarGhostReason: Binding = Binding {
        id: "Sess_Rules_SetCarGhostReason",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Ruleset_FinaliseResults: Binding = Binding {
        id: "Sess_Ruleset_FinaliseResults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Ruleset_FinaliseResultsDefault: Binding = Binding {
        id: "Sess_Ruleset_FinaliseResultsDefault",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Ruleset_ForceFinishAllDefault: Binding = Binding {
        id: "Sess_Ruleset_ForceFinishAllDefault",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Ruleset_SetupExtraTime: Binding = Binding {
        id: "Sess_Ruleset_SetupExtraTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Ruleset_UpdateRetiredCarFade: Binding = Binding {
        id: "Sess_Ruleset_UpdateRetiredCarFade",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarDeployment_OnSessionRestart: Binding = Binding {
        id: "Sess_SafetyCarDeployment_OnSessionRestart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarDeployment_PreSessionRestart: Binding = Binding {
        id: "Sess_SafetyCarDeployment_PreSessionRestart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarEmergencyGhosting_Init: Binding = Binding {
        id: "Sess_SafetyCarEmergencyGhosting_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarLogic_Subscribe: Binding = Binding {
        id: "Sess_SafetyCarLogic_Subscribe",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarTether_ApplyAssistOverrides: Binding = Binding {
        id: "Sess_SafetyCarTether_ApplyAssistOverrides",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarTether_IsVehicleAttached: Binding = Binding {
        id: "Sess_SafetyCarTether_IsVehicleAttached",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarTether_UpdateProgress: Binding = Binding {
        id: "Sess_SafetyCarTether_UpdateProgress",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarTether_UpdateTarget: Binding = Binding {
        id: "Sess_SafetyCarTether_UpdateTarget",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCarTether_UpdateVehicle: Binding = Binding {
        id: "Sess_SafetyCarTether_UpdateVehicle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_BuildGroupedCarList: Binding = Binding {
        id: "Sess_SafetyCar_BuildGroupedCarList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_CanVehicleRejoin: Binding = Binding {
        id: "Sess_SafetyCar_CanVehicleRejoin",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_ChangeState: Binding = Binding {
        id: "Sess_SafetyCar_ChangeState",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_ComputeTargetSpeed: Binding = Binding {
        id: "Sess_SafetyCar_ComputeTargetSpeed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_IntersectionTime: Binding = Binding {
        id: "Sess_SafetyCar_IntersectionTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_IsVehicleInDeltaWindow: Binding = Binding {
        id: "Sess_SafetyCar_IsVehicleInDeltaWindow",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_OnLapChanged: Binding = Binding {
        id: "Sess_SafetyCar_OnLapChanged",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_ReadDeploymentFrequency: Binding = Binding {
        id: "Sess_SafetyCar_ReadDeploymentFrequency",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_ReadDynamicObstruction: Binding = Binding {
        id: "Sess_SafetyCar_ReadDynamicObstruction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_ReadMaxRedFlags: Binding = Binding {
        id: "Sess_SafetyCar_ReadMaxRedFlags",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_UnlapGate: Binding = Binding {
        id: "Sess_SafetyCar_UnlapGate",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SafetyCar_VehicleTransition: Binding = Binding {
        id: "Sess_SafetyCar_VehicleTransition",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScCond_HazardDetected_Update: Binding = Binding {
        id: "Sess_ScCond_HazardDetected_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScCond_MinLapsOut_Update: Binding = Binding {
        id: "Sess_ScCond_MinLapsOut_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScCond_OffTrack_Update: Binding = Binding {
        id: "Sess_ScCond_OffTrack_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScCond_PitAreaClear_Update: Binding = Binding {
        id: "Sess_ScCond_PitAreaClear_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScCond_TriggerVsc_Update: Binding = Binding {
        id: "Sess_ScCond_TriggerVsc_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScDebrisAccumulator_Update: Binding = Binding {
        id: "Sess_ScDebrisAccumulator_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScDebris_SumPartVolumes_B: Binding = Binding {
        id: "Sess_ScDebris_SumPartVolumes_B",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScRedFlagDeploy_Update: Binding = Binding {
        id: "Sess_ScRedFlagDeploy_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_Deploy_Enter: Binding = Binding {
        id: "Sess_ScState_Deploy_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_Deploy_ReadTunables: Binding = Binding {
        id: "Sess_ScState_Deploy_ReadTunables",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_Return_Enter: Binding = Binding {
        id: "Sess_ScState_Return_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_ReturnedToPit_Enter: Binding = Binding {
        id: "Sess_ScState_ReturnedToPit_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_ReturnedToPit_Exit: Binding = Binding {
        id: "Sess_ScState_ReturnedToPit_Exit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_VscDeploy_Enter: Binding = Binding {
        id: "Sess_ScState_VscDeploy_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_VscEnding_Enter: Binding = Binding {
        id: "Sess_ScState_VscEnding_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_VscEnding_Exit: Binding = Binding {
        id: "Sess_ScState_VscEnding_Exit",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScState_Vsc_Enter: Binding = Binding {
        id: "Sess_ScState_Vsc_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScStoppedCar_MapEmplace: Binding = Binding {
        id: "Sess_ScStoppedCar_MapEmplace",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ScriptHooks_FireSessionEvent: Binding = Binding {
        id: "Sess_ScriptHooks_FireSessionEvent",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SessionClock_SetTime: Binding = Binding {
        id: "Sess_SessionClock_SetTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SessionExit_OnStateEnter: Binding = Binding {
        id: "Sess_SessionExit_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SessionStateIntroCameras_OnEnter: Binding = Binding {
        id: "Sess_SessionStateIntroCameras_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SessionStateMidSessionSaveCutscene_OnEnter: Binding = Binding {
        id: "Sess_SessionStateMidSessionSaveCutscene_OnEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Session_ResetForRestart: Binding = Binding {
        id: "Sess_Session_ResetForRestart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Set_RowsAvailable: Binding = Binding {
        id: "Sess_Set_RowsAvailable",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Set_SafetyCarDeploymentType: Binding = Binding {
        id: "Sess_Set_SafetyCarDeploymentType",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Setup_VehicleComponents: Binding = Binding {
        id: "Sess_Setup_VehicleComponents",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SortAssignGarages_Thunk: Binding = Binding {
        id: "Sess_SortAssignGarages_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SortEntrantsByGarageOrder: Binding = Binding {
        id: "Sess_SortEntrantsByGarageOrder",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SpectatorDirector_SetActive: Binding = Binding {
        id: "Sess_SpectatorDirector_SetActive",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SpectatorDirector_Update: Binding = Binding {
        id: "Sess_SpectatorDirector_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SpectatorHud_Hide: Binding = Binding {
        id: "Sess_SpectatorHud_Hide",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SpectatorHud_Show: Binding = Binding {
        id: "Sess_SpectatorHud_Show",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SpeedLimitPenalty_ReceiveData: Binding = Binding {
        id: "Sess_SpeedLimitPenalty_ReceiveData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_SpeedLimitPenalty_SendData: Binding = Binding {
        id: "Sess_SpeedLimitPenalty_SendData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StartLightsMachine_SetProxy: Binding = Binding {
        id: "Sess_StartLightsMachine_SetProxy",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StartLightsSync_MustAdvance: Binding = Binding {
        id: "Sess_StartLightsSync_MustAdvance",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StartLights_Enter: Binding = Binding {
        id: "Sess_StartLights_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StartLights_Init: Binding = Binding {
        id: "Sess_StartLights_Init",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Start_VehiclesInGarages: Binding = Binding {
        id: "Sess_Start_VehiclesInGarages",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_State_EnterOnTrackCommon: Binding = Binding {
        id: "Sess_State_EnterOnTrackCommon",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_State_HandleFlowInterrupt: Binding = Binding {
        id: "Sess_State_HandleFlowInterrupt",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_State_OnPauseRequested: Binding = Binding {
        id: "Sess_State_OnPauseRequested",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_State_Pause: Binding = Binding {
        id: "Sess_State_Pause",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_State_Unpause: Binding = Binding {
        id: "Sess_State_Unpause",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StopGoPenalty_ReceiveData: Binding = Binding {
        id: "Sess_StopGoPenalty_ReceiveData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StopGoPenalty_SendData: Binding = Binding {
        id: "Sess_StopGoPenalty_SendData",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StopGoPenalty_SendData_2: Binding = Binding {
        id: "Sess_StopGoPenalty_SendData_2",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StoryPreLights_Update: Binding = Binding {
        id: "Sess_StoryPreLights_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StoryRollingStart_OnStateEnter: Binding = Binding {
        id: "Sess_StoryRollingStart_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_StoryStartLights_OnStateEnter: Binding = Binding {
        id: "Sess_StoryStartLights_OnStateEnter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Strategy_TyreClassTimeLoss: Binding = Binding {
        id: "Sess_Strategy_TyreClassTimeLoss",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Strategy_UpdateTeammatePairs: Binding = Binding {
        id: "Sess_Strategy_UpdateTeammatePairs",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TimeOfDayOwner_EvalRainIntensity: Binding = Binding {
        id: "Sess_TimeOfDayOwner_EvalRainIntensity",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TodStitch_BuildDefaultInstance: Binding = Binding {
        id: "Sess_TodStitch_BuildDefaultInstance",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TodStitch_GetWetDeltaAtPos: Binding = Binding {
        id: "Sess_TodStitch_GetWetDeltaAtPos",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TodStitch_MergeInstanceSingle: Binding = Binding {
        id: "Sess_TodStitch_MergeInstanceSingle",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TodStitch_MergeInstancesMulti: Binding = Binding {
        id: "Sess_TodStitch_MergeInstancesMulti",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Tod_BuildSessionDataset_Thunk: Binding = Binding {
        id: "Sess_Tod_BuildSessionDataset_Thunk",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Tod_BuildWeatherScenario: Binding = Binding {
        id: "Sess_Tod_BuildWeatherScenario",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Tod_RandomiseTransitions: Binding = Binding {
        id: "Sess_Tod_RandomiseTransitions",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Tod_ResetDatasetStates: Binding = Binding {
        id: "Sess_Tod_ResetDatasetStates",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Tod_StitchWeatherScenario: Binding = Binding {
        id: "Sess_Tod_StitchWeatherScenario",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TyreAllocation_OnSessionStart: Binding = Binding {
        id: "Sess_TyreAllocation_OnSessionStart",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TyreAllocation_SwapSets: Binding = Binding {
        id: "Sess_TyreAllocation_SwapSets",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TyreClass_FromName: Binding = Binding {
        id: "Sess_TyreClass_FromName",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TyreCompoundClasses_SortEntries: Binding = Binding {
        id: "Sess_TyreCompoundClasses_SortEntries",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_TyreCompoundClasses_SortEntriesB: Binding = Binding {
        id: "Sess_TyreCompoundClasses_SortEntriesB",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Tyres_ClosestAvailableCompound: Binding = Binding {
        id: "Sess_Tyres_ClosestAvailableCompound",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Veh_UpdateWheelMap: Binding = Binding {
        id: "Sess_Veh_UpdateWheelMap",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VehiclePlacement_BroadcastGrid: Binding = Binding {
        id: "Sess_VehiclePlacement_BroadcastGrid",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VehiclePlacement_FormatFlags: Binding = Binding {
        id: "Sess_VehiclePlacement_FormatFlags",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VehiclePlacement_HandleGridRequest: Binding = Binding {
        id: "Sess_VehiclePlacement_HandleGridRequest",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VehiclePlacement_PlaceOnSlot: Binding = Binding {
        id: "Sess_VehiclePlacement_PlaceOnSlot",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VehiclePlacement_ResolveTransform: Binding = Binding {
        id: "Sess_VehiclePlacement_ResolveTransform",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VehicleProgress_IsPastLapFraction: Binding = Binding {
        id: "Sess_VehicleProgress_IsPastLapFraction",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_ViewReplay_EnterPhotoMode: Binding = Binding {
        id: "Sess_ViewReplay_EnterPhotoMode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualLapsRuleset_FinaliseResults: Binding = Binding {
        id: "Sess_VirtualLapsRuleset_FinaliseResults",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualLapsRuleset_ForceFinishAll: Binding = Binding {
        id: "Sess_VirtualLapsRuleset_ForceFinishAll",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualLaps_BuildOrderLists: Binding = Binding {
        id: "Sess_VirtualLaps_BuildOrderLists",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualLaps_EraseLocalEntries: Binding = Binding {
        id: "Sess_VirtualLaps_EraseLocalEntries",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualLaps_UpdateVehicleScore: Binding = Binding {
        id: "Sess_VirtualLaps_UpdateVehicleScore",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualPerformance_CalcPace: Binding = Binding {
        id: "Sess_VirtualPerformance_CalcPace",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualSession_EstimateLapTime: Binding = Binding {
        id: "Sess_VirtualSession_EstimateLapTime",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualSession_FinishVirtualising: Binding = Binding {
        id: "Sess_VirtualSession_FinishVirtualising",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualSession_SimulateEntrantStep: Binding = Binding {
        id: "Sess_VirtualSession_SimulateEntrantStep",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_VirtualSession_Update: Binding = Binding {
        id: "Sess_VirtualSession_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_WeatherComponent_SetSystemRandomSeed: Binding = Binding {
        id: "Sess_WeatherComponent_SetSystemRandomSeed",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Weather_CheckTrackWorldComponents: Binding = Binding {
        id: "Sess_Weather_CheckTrackWorldComponents",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Weather_GetQualiSegmentStartOffset: Binding = Binding {
        id: "Sess_Weather_GetQualiSegmentStartOffset",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Weather_PickRandomType: Binding = Binding {
        id: "Sess_Weather_PickRandomType",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_WeekendManager_PrepareTrackLoad: Binding = Binding {
        id: "Sess_WeekendManager_PrepareTrackLoad",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Weekend_RebuildUniqueEntrantList: Binding = Binding {
        id: "Sess_Weekend_RebuildUniqueEntrantList",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_Weekend_Start: Binding = Binding {
        id: "Sess_Weekend_Start",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_WinnerSequence_Enter: Binding = Binding {
        id: "Sess_WinnerSequence_Enter",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_WinnerSequence_IsCarExcluded: Binding = Binding {
        id: "Sess_WinnerSequence_IsCarExcluded",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_WinnerSequence_IsPlayerWinner: Binding = Binding {
        id: "Sess_WinnerSequence_IsPlayerWinner",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Sess_WinnerSequence_Update: Binding = Binding {
        id: "Sess_WinnerSequence_Update",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Snd_Request_JobWorker: Binding = Binding {
        id: "Snd_Request_JobWorker",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Team_Build_FanZoneStandings: Binding = Binding {
        id: "Team_Build_FanZoneStandings",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Team_Insert_RecordByStatId: Binding = Binding {
        id: "Team_Insert_RecordByStatId",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Trk_GrandPrixTrackSelectVm_OnCommandRaised: Binding = Binding {
        id: "Trk_GrandPrixTrackSelectVm_OnCommandRaised",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Trk_TimeTrialTrackSelectVm_OnCommandRaised: Binding = Binding {
        id: "Trk_TimeTrialTrackSelectVm_OnCommandRaised",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_GetOrCreate_ListChild: Binding = Binding {
        id: "Vm_GetOrCreate_ListChild",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_List_AppendChild: Binding = Binding {
        id: "Vm_List_AppendChild",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_Release_Node: Binding = Binding {
        id: "Vm_Release_Node",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_Resolve_RegistryNode: Binding = Binding {
        id: "Vm_Resolve_RegistryNode",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_Set_Bool: Binding = Binding {
        id: "Vm_Set_Bool",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_Set_NodeRef: Binding = Binding {
        id: "Vm_Set_NodeRef",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_Set_String: Binding = Binding {
        id: "Vm_Set_String",
        kind: "func",
        status: Status::Experimental,
    };
    pub const Vm_Set_TabEnabled: Binding = Binding {
        id: "Vm_Set_TabEnabled",
        kind: "func",
        status: Status::Experimental,
    };
    pub const g_Career_FlowLinkContinue: Binding = Binding {
        id: "g_Career_FlowLinkContinue",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Career_RnDOverviewSelectedDept: Binding = Binding {
        id: "g_Career_RnDOverviewSelectedDept",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Career_UpgradeLifecycle: Binding = Binding {
        id: "g_Career_UpgradeLifecycle",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_FeTile_CustomisationSystem: Binding = Binding {
        id: "g_FeTile_CustomisationSystem",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_FeTile_SelectedCareerContent: Binding = Binding {
        id: "g_FeTile_SelectedCareerContent",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_FlowLinkStay: Binding = Binding {
        id: "g_Game_FlowLinkStay",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_Manager: Binding = Binding {
        id: "g_Game_Manager",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_MessageBus: Binding = Binding {
        id: "g_Game_MessageBus",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_OptionsStore: Binding = Binding {
        id: "g_Game_OptionsStore",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_RaceStartSystem: Binding = Binding {
        id: "g_Game_RaceStartSystem",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_Root: Binding = Binding {
        id: "g_Game_Root",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_Scenarios: Binding = Binding {
        id: "g_Game_Scenarios",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_TrackLength: Binding = Binding {
        id: "g_Game_TrackLength",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_VehicleRecordManager: Binding = Binding {
        id: "g_Game_VehicleRecordManager",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Game_VehicleStateRoot: Binding = Binding {
        id: "g_Game_VehicleStateRoot",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Gdb_Database: Binding = Binding {
        id: "g_Gdb_Database",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Gdb_RecordAllocator: Binding = Binding {
        id: "g_Gdb_RecordAllocator",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Ne_DefaultAllocator: Binding = Binding {
        id: "g_Ne_DefaultAllocator",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Ne_LogGate: Binding = Binding {
        id: "g_Ne_LogGate",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Ne_NameTable: Binding = Binding {
        id: "g_Ne_NameTable",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Ne_StringTable: Binding = Binding {
        id: "g_Ne_StringTable",
        kind: "data",
        status: Status::Stable,
    };
    pub const g_Sess_InfringementTypeNames: Binding = Binding {
        id: "g_Sess_InfringementTypeNames",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Sess_RedFlagCarStatusNames: Binding = Binding {
        id: "g_Sess_RedFlagCarStatusNames",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Sess_ResultStatusNames: Binding = Binding {
        id: "g_Sess_ResultStatusNames",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Sess_TyreRulesActive: Binding = Binding {
        id: "g_Sess_TyreRulesActive",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Sess_TyreRulesMode: Binding = Binding {
        id: "g_Sess_TyreRulesMode",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Sess_TyreRulesSessionType: Binding = Binding {
        id: "g_Sess_TyreRulesSessionType",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Sess_VehicleCount: Binding = Binding {
        id: "g_Sess_VehicleCount",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Sess_VehicleTable: Binding = Binding {
        id: "g_Sess_VehicleTable",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Snd_JobWorkerLock: Binding = Binding {
        id: "g_Snd_JobWorkerLock",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Snd_JobWorkerOverflow: Binding = Binding {
        id: "g_Snd_JobWorkerOverflow",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Snd_JobWorkerPending: Binding = Binding {
        id: "g_Snd_JobWorkerPending",
        kind: "data",
        status: Status::Experimental,
    };
    pub const g_Vm_Registry: Binding = Binding {
        id: "g_Vm_Registry",
        kind: "data",
        status: Status::Stable,
    };
    pub const k_Career_CareerBankComponentDesc: Binding = Binding {
        id: "k_Career_CareerBankComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CareerComponentDesc: Binding = Binding {
        id: "k_Career_CareerComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CareerDriverContractRenewalComponentDesc: Binding = Binding {
        id: "k_Career_CareerDriverContractRenewalComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CareerDriverMarketComponentDesc: Binding = Binding {
        id: "k_Career_CareerDriverMarketComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CareerFacilitiesComponentDesc: Binding = Binding {
        id: "k_Career_CareerFacilitiesComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CareerSetupComponentDesc: Binding = Binding {
        id: "k_Career_CareerSetupComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CareerSponsorsComponentDesc: Binding = Binding {
        id: "k_Career_CareerSponsorsComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CareerTimeComponentDesc: Binding = Binding {
        id: "k_Career_CareerTimeComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_ChampionshipComponentDesc: Binding = Binding {
        id: "k_Career_ChampionshipComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_ChampionshipDesc: Binding = Binding {
        id: "k_Career_ChampionshipDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_CostCapComponentDesc: Binding = Binding {
        id: "k_Career_CostCapComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_DriverCareerContractsManagerComponentDesc: Binding = Binding {
        id: "k_Career_DriverCareerContractsManagerComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_DriverCareerPlayerDriverStatisticsComponentDesc: Binding = Binding {
        id: "k_Career_DriverCareerPlayerDriverStatisticsComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_DriverContractsMyTeamComponentDesc: Binding = Binding {
        id: "k_Career_DriverContractsMyTeamComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_DriverStatisticsComponentDesc: Binding = Binding {
        id: "k_Career_DriverStatisticsComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_EngineSuppliersComponentDesc: Binding = Binding {
        id: "k_Career_EngineSuppliersComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_MyTeamAttributesComponentDesc: Binding = Binding {
        id: "k_Career_MyTeamAttributesComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_MyTeamFacilitiesComponentDesc: Binding = Binding {
        id: "k_Career_MyTeamFacilitiesComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_MyTeamRFComponentDesc: Binding = Binding {
        id: "k_Career_MyTeamRFComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_MyTeamStatsComponentDesc: Binding = Binding {
        id: "k_Career_MyTeamStatsComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_OwnerPerksComponentDesc: Binding = Binding {
        id: "k_Career_OwnerPerksComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_OwnerSetupComponentDesc: Binding = Binding {
        id: "k_Career_OwnerSetupComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_ResourcePointsComponentDesc: Binding = Binding {
        id: "k_Career_ResourcePointsComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_RnDCardColourTable: Binding = Binding {
        id: "k_Career_RnDCardColourTable",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_RnDDeptColourTable: Binding = Binding {
        id: "k_Career_RnDDeptColourTable",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_RnDErrorDeptName: Binding = Binding {
        id: "k_Career_RnDErrorDeptName",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_ScheduleComponentDesc: Binding = Binding {
        id: "k_Career_ScheduleComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_TeamHQControllerComponentDesc: Binding = Binding {
        id: "k_Career_TeamHQControllerComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_UpgradeBlockNameTable: Binding = Binding {
        id: "k_Career_UpgradeBlockNameTable",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_UpgradeTypeOrder: Binding = Binding {
        id: "k_Career_UpgradeTypeOrder",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_UpgradesComponentDesc: Binding = Binding {
        id: "k_Career_UpgradesComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Career_WeatherAndTimeOfDayComponentDesc: Binding = Binding {
        id: "k_Career_WeatherAndTimeOfDayComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Game_TrackListComponentDesc: Binding = Binding {
        id: "k_Game_TrackListComponentDesc",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Gdb_StringXorKey: Binding = Binding {
        id: "k_Gdb_StringXorKey",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Ne_BagCodecVtbl: Binding = Binding {
        id: "k_Ne_BagCodecVtbl",
        kind: "vtable",
        status: Status::Stable,
    };
    pub const k_Ne_CountingStreamVtbl: Binding = Binding {
        id: "k_Ne_CountingStreamVtbl",
        kind: "vtable",
        status: Status::Stable,
    };
    pub const k_Ne_MemWriteStreamVtbl: Binding = Binding {
        id: "k_Ne_MemWriteStreamVtbl",
        kind: "vtable",
        status: Status::Stable,
    };
    pub const k_Ne_PropertyType_Bag: Binding = Binding {
        id: "k_Ne_PropertyType_Bag",
        kind: "data",
        status: Status::Stable,
    };
    pub const k_Ne_TypeChainEnd: Binding = Binding {
        id: "k_Ne_TypeChainEnd",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Ne_TypeRoot: Binding = Binding {
        id: "k_Ne_TypeRoot",
        kind: "data",
        status: Status::Experimental,
    };
    pub const k_Sess_TyreComponentType: Binding = Binding {
        id: "k_Sess_TyreComponentType",
        kind: "data",
        status: Status::Experimental,
    };
    pub const site_Ai_TableGripInput: Binding = Binding {
        id: "site_Ai_TableGripInput",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ai_UpdateCars_CornerLineLoad: Binding = Binding {
        id: "site_Ai_UpdateCars_CornerLineLoad",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ai_UpdateCars_NoCornerLine: Binding = Binding {
        id: "site_Ai_UpdateCars_NoCornerLine",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_AeroStoreRearDownforce: Binding = Binding {
        id: "site_Car_AeroStoreRearDownforce",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_AeroStoreTotalDrag: Binding = Binding {
        id: "site_Car_AeroStoreTotalDrag",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_AeroStoreTotalLift: Binding = Binding {
        id: "site_Car_AeroStoreTotalLift",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_ChassisRegistryCount: Binding = Binding {
        id: "site_Car_ChassisRegistryCount",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_EngineDriveTorque: Binding = Binding {
        id: "site_Car_EngineDriveTorque",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_ErsGearModifierLoad: Binding = Binding {
        id: "site_Car_ErsGearModifierLoad",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_FuelFlowStore: Binding = Binding {
        id: "site_Car_FuelFlowStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_FuelMassStore: Binding = Binding {
        id: "site_Car_FuelMassStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_LiveCarBuilt: Binding = Binding {
        id: "site_Car_LiveCarBuilt",
        kind: "site",
        status: Status::Stable,
    };
    pub const site_Car_RegisterVehicleId: Binding = Binding {
        id: "site_Car_RegisterVehicleId",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_SetupTuningApplied: Binding = Binding {
        id: "site_Car_SetupTuningApplied",
        kind: "site",
        status: Status::Stable,
    };
    pub const site_Car_UpgradeSectionApplyCall: Binding = Binding {
        id: "site_Car_UpgradeSectionApplyCall",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Car_VehicleTableRemove: Binding = Binding {
        id: "site_Car_VehicleTableRemove",
        kind: "site",
        status: Status::Stable,
    };
    pub const site_Career_ApplyUpgradeBlockNameDefault: Binding = Binding {
        id: "site_Career_ApplyUpgradeBlockNameDefault",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_ApplyUpgradeBlockNameJoin: Binding = Binding {
        id: "site_Career_ApplyUpgradeBlockNameJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_ApplyUpgradeKeyCall: Binding = Binding {
        id: "site_Career_ApplyUpgradeKeyCall",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_ApplyUpgradeKeyFilter: Binding = Binding {
        id: "site_Career_ApplyUpgradeKeyFilter",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_DeserialiseUpgradesEnd: Binding = Binding {
        id: "site_Career_DeserialiseUpgradesEnd",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_LoadUpgradeRows_DeptPath: Binding = Binding {
        id: "site_Career_LoadUpgradeRows_DeptPath",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDBoxGridFirstTab: Binding = Binding {
        id: "site_Career_RnDBoxGridFirstTab",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDBoxGridNameClamp: Binding = Binding {
        id: "site_Career_RnDBoxGridNameClamp",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDBoxGridNameIndex: Binding = Binding {
        id: "site_Career_RnDBoxGridNameIndex",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDBoxGridNameRead: Binding = Binding {
        id: "site_Career_RnDBoxGridNameRead",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardArtImage0: Binding = Binding {
        id: "site_Career_RnDCardArtImage0",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardArtImage1: Binding = Binding {
        id: "site_Career_RnDCardArtImage1",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardArtImage2: Binding = Binding {
        id: "site_Career_RnDCardArtImage2",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardArtMask0: Binding = Binding {
        id: "site_Career_RnDCardArtMask0",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardArtMask1: Binding = Binding {
        id: "site_Career_RnDCardArtMask1",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardArtMask2: Binding = Binding {
        id: "site_Career_RnDCardArtMask2",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardColourPick0: Binding = Binding {
        id: "site_Career_RnDCardColourPick0",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardColourPick1: Binding = Binding {
        id: "site_Career_RnDCardColourPick1",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardColourPick2: Binding = Binding {
        id: "site_Career_RnDCardColourPick2",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardColourPick3: Binding = Binding {
        id: "site_Career_RnDCardColourPick3",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardColourPick4: Binding = Binding {
        id: "site_Career_RnDCardColourPick4",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardDeptIcon0: Binding = Binding {
        id: "site_Career_RnDCardDeptIcon0",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardDeptIcon1: Binding = Binding {
        id: "site_Career_RnDCardDeptIcon1",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCardDeptIcon2: Binding = Binding {
        id: "site_Career_RnDCardDeptIcon2",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDCompareDept: Binding = Binding {
        id: "site_Career_RnDCompareDept",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDContainerOpenMaint: Binding = Binding {
        id: "site_Career_RnDContainerOpenMaint",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDContainerOpenTail: Binding = Binding {
        id: "site_Career_RnDContainerOpenTail",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptBoxGridEntry: Binding = Binding {
        id: "site_Career_RnDDeptBoxGridEntry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptBoxGridInject: Binding = Binding {
        id: "site_Career_RnDDeptBoxGridInject",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptBoxLayoutProbe: Binding = Binding {
        id: "site_Career_RnDDeptBoxLayoutProbe",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptPagePreQuery: Binding = Binding {
        id: "site_Career_RnDDeptPagePreQuery",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptPageQueryDone: Binding = Binding {
        id: "site_Career_RnDDeptPageQueryDone",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptPageTypeLoop: Binding = Binding {
        id: "site_Career_RnDDeptPageTypeLoop",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptRowCategoryIcon: Binding = Binding {
        id: "site_Career_RnDDeptRowCategoryIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptRowTypeIcon: Binding = Binding {
        id: "site_Career_RnDDeptRowTypeIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptRowTypeIconJoin: Binding = Binding {
        id: "site_Career_RnDDeptRowTypeIconJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptRowTypeLabel: Binding = Binding {
        id: "site_Career_RnDDeptRowTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDDeptRowTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDDeptRowTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDInProgressColourStore: Binding = Binding {
        id: "site_Career_RnDInProgressColourStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDInProgressPartIcon: Binding = Binding {
        id: "site_Career_RnDInProgressPartIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDInProgressStrip: Binding = Binding {
        id: "site_Career_RnDInProgressStrip",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDInProgressTypeIcon: Binding = Binding {
        id: "site_Career_RnDInProgressTypeIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDInProgressTypeIconJoin: Binding = Binding {
        id: "site_Career_RnDInProgressTypeIconJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDInProgressTypeLabel: Binding = Binding {
        id: "site_Career_RnDInProgressTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDInProgressTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDInProgressTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDListRowColourStore: Binding = Binding {
        id: "site_Career_RnDListRowColourStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDListRowDeptIcon: Binding = Binding {
        id: "site_Career_RnDListRowDeptIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDListRowLightColourStore: Binding = Binding {
        id: "site_Career_RnDListRowLightColourStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPageModifierList: Binding = Binding {
        id: "site_Career_RnDPageModifierList",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPanelModifierList: Binding = Binding {
        id: "site_Career_RnDPanelModifierList",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPanelResearchLimit: Binding = Binding {
        id: "site_Career_RnDPanelResearchLimit",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPerfChildName: Binding = Binding {
        id: "site_Career_RnDPerfChildName",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPerfDeptColour: Binding = Binding {
        id: "site_Career_RnDPerfDeptColour",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPerfDeptLoopEnd: Binding = Binding {
        id: "site_Career_RnDPerfDeptLoopEnd",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPerfDeptName: Binding = Binding {
        id: "site_Career_RnDPerfDeptName",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPerfGotoName: Binding = Binding {
        id: "site_Career_RnDPerfGotoName",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPerfViewModelSlotDtor: Binding = Binding {
        id: "site_Career_RnDPerfViewModelSlotDtor",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPerksTitleDeptIcon: Binding = Binding {
        id: "site_Career_RnDPerksTitleDeptIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieBufSize: Binding = Binding {
        id: "site_Career_RnDPieBufSize",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieCtorRet: Binding = Binding {
        id: "site_Career_RnDPieCtorRet",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieFocusJoin: Binding = Binding {
        id: "site_Career_RnDPieFocusJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieFocusNameCmp: Binding = Binding {
        id: "site_Career_RnDPieFocusNameCmp",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieFocusRestore: Binding = Binding {
        id: "site_Career_RnDPieFocusRestore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieMoveAssignSlotDtor: Binding = Binding {
        id: "site_Career_RnDPieMoveAssignSlotDtor",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieMovedFromSlotDtor: Binding = Binding {
        id: "site_Career_RnDPieMovedFromSlotDtor",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPiePopulateDeptGuard: Binding = Binding {
        id: "site_Career_RnDPiePopulateDeptGuard",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPiePopulateGate: Binding = Binding {
        id: "site_Career_RnDPiePopulateGate",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPiePopulateNextCode: Binding = Binding {
        id: "site_Career_RnDPiePopulateNextCode",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPiePopulateSlotCap: Binding = Binding {
        id: "site_Career_RnDPiePopulateSlotCap",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieSliceDefaultColour: Binding = Binding {
        id: "site_Career_RnDPieSliceDefaultColour",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieSliceDescFormat: Binding = Binding {
        id: "site_Career_RnDPieSliceDescFormat",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieSliceIconArgs: Binding = Binding {
        id: "site_Career_RnDPieSliceIconArgs",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieSliceNameJoin: Binding = Binding {
        id: "site_Career_RnDPieSliceNameJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPieTeardownSlotDtor: Binding = Binding {
        id: "site_Career_RnDPieTeardownSlotDtor",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPopupCategoryIcon: Binding = Binding {
        id: "site_Career_RnDPopupCategoryIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPopupColourStore: Binding = Binding {
        id: "site_Career_RnDPopupColourStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPotentialPieEntry: Binding = Binding {
        id: "site_Career_RnDPotentialPieEntry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPurchaseBackOpen: Binding = Binding {
        id: "site_Career_RnDPurchaseBackOpen",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPurchaseOptionsEntry: Binding = Binding {
        id: "site_Career_RnDPurchaseOptionsEntry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPurchasePanelTypeLabel: Binding = Binding {
        id: "site_Career_RnDPurchasePanelTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPurchasePanelTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDPurchasePanelTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPurchaseRequestEntry: Binding = Binding {
        id: "site_Career_RnDPurchaseRequestEntry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPurchaseScreenTypeLabel: Binding = Binding {
        id: "site_Career_RnDPurchaseScreenTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDPurchaseScreenTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDPurchaseScreenTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResearchFocusNameCmp: Binding = Binding {
        id: "site_Career_RnDResearchFocusNameCmp",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResearchRowCategoryIcon: Binding = Binding {
        id: "site_Career_RnDResearchRowCategoryIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResearchRowTypeIcon: Binding = Binding {
        id: "site_Career_RnDResearchRowTypeIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResearchRowTypeIconJoin: Binding = Binding {
        id: "site_Career_RnDResearchRowTypeIconJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResearchRowTypeLabel: Binding = Binding {
        id: "site_Career_RnDResearchRowTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResearchRowTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDResearchRowTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResearchTypeLoop: Binding = Binding {
        id: "site_Career_RnDResearchTypeLoop",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResultTypeIcon: Binding = Binding {
        id: "site_Career_RnDResultTypeIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResultTypeIconJoin: Binding = Binding {
        id: "site_Career_RnDResultTypeIconJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResultTypeLabel: Binding = Binding {
        id: "site_Career_RnDResultTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDResultTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDResultTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDTileCountBody: Binding = Binding {
        id: "site_Career_RnDTileCountBody",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDTileCountExit: Binding = Binding {
        id: "site_Career_RnDTileCountExit",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDTileCountGuards: Binding = Binding {
        id: "site_Career_RnDTileCountGuards",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDTileCountIndexGuard: Binding = Binding {
        id: "site_Career_RnDTileCountIndexGuard",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDTodoTypeLabel: Binding = Binding {
        id: "site_Career_RnDTodoTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDTodoTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDTodoTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDTreeRowCountProbe: Binding = Binding {
        id: "site_Career_RnDTreeRowCountProbe",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDUpgradeInfoTypeLabel: Binding = Binding {
        id: "site_Career_RnDUpgradeInfoTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDUpgradeInfoTypeLabelJoin: Binding = Binding {
        id: "site_Career_RnDUpgradeInfoTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_RnDUpgradeListDeptIcon: Binding = Binding {
        id: "site_Career_RnDUpgradeListDeptIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_StandingsWeekendTrackDone: Binding = Binding {
        id: "site_Career_StandingsWeekendTrackDone",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_StandingsWeekendTrackLoad: Binding = Binding {
        id: "site_Career_StandingsWeekendTrackLoad",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeamDeptEnumStore: Binding = Binding {
        id: "site_Career_TeamDeptEnumStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeamHqIsMyTeamResult: Binding = Binding {
        id: "site_Career_TeamHqIsMyTeamResult",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeamHqRnDPopupTypeIcon: Binding = Binding {
        id: "site_Career_TeamHqRnDPopupTypeIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeamHqRnDPopupTypeIconJoin: Binding = Binding {
        id: "site_Career_TeamHqRnDPopupTypeIconJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeleFailedTypeName: Binding = Binding {
        id: "site_Career_TeleFailedTypeName",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeleFailedTypeNameJoin: Binding = Binding {
        id: "site_Career_TeleFailedTypeNameJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeleRecordTypeName: Binding = Binding {
        id: "site_Career_TeleRecordTypeName",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeleRecordTypeNameCall: Binding = Binding {
        id: "site_Career_TeleRecordTypeNameCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Career_TeleResearchTypeName: Binding = Binding {
        id: "site_Career_TeleResearchTypeName",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_TeleResearchTypeNameCall: Binding = Binding {
        id: "site_Career_TeleResearchTypeNameCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Career_Unk141814320TypeLabel: Binding = Binding {
        id: "site_Career_Unk141814320TypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_Unk141814320TypeLabelJoin: Binding = Binding {
        id: "site_Career_Unk141814320TypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_Unk1418169f0TypeLabel: Binding = Binding {
        id: "site_Career_Unk1418169f0TypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_Unk1418169f0TypeLabelJoin: Binding = Binding {
        id: "site_Career_Unk1418169f0TypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeAutoFitBody: Binding = Binding {
        id: "site_Career_UpgradeAutoFitBody",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeAutoFitDeptGate: Binding = Binding {
        id: "site_Career_UpgradeAutoFitDeptGate",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeBlockReplayEntry: Binding = Binding {
        id: "site_Career_UpgradeBlockReplayEntry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeCardDeptIcon: Binding = Binding {
        id: "site_Career_UpgradeCardDeptIcon",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeCardTypeLabel: Binding = Binding {
        id: "site_Career_UpgradeCardTypeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeCardTypeLabelJoin: Binding = Binding {
        id: "site_Career_UpgradeCardTypeLabelJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeGraphHubTest: Binding = Binding {
        id: "site_Career_UpgradeGraphHubTest",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeGraphRegularNode: Binding = Binding {
        id: "site_Career_UpgradeGraphRegularNode",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeInitFakeDeptBranch: Binding = Binding {
        id: "site_Career_UpgradeInitFakeDeptBranch",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeInitNormal: Binding = Binding {
        id: "site_Career_UpgradeInitNormal",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeMetaTypeJoin: Binding = Binding {
        id: "site_Career_UpgradeMetaTypeJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradePushStateBody: Binding = Binding {
        id: "site_Career_UpgradePushStateBody",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradePushStateDeptGate: Binding = Binding {
        id: "site_Career_UpgradePushStateDeptGate",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeRowTypeJoin: Binding = Binding {
        id: "site_Career_UpgradeRowTypeJoin",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeSetEnabledBody: Binding = Binding {
        id: "site_Career_UpgradeSetEnabledBody",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeSetEnabledDeptGate: Binding = Binding {
        id: "site_Career_UpgradeSetEnabledDeptGate",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeSetFittedBody: Binding = Binding {
        id: "site_Career_UpgradeSetFittedBody",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeSetFittedDeptGate: Binding = Binding {
        id: "site_Career_UpgradeSetFittedDeptGate",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeSetLockedBody: Binding = Binding {
        id: "site_Career_UpgradeSetLockedBody",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_UpgradeSetLockedDeptGate: Binding = Binding {
        id: "site_Career_UpgradeSetLockedDeptGate",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_VehicleFillOrderEnd: Binding = Binding {
        id: "site_Career_VehicleFillOrderEnd",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_VehicleFillOrderStart: Binding = Binding {
        id: "site_Career_VehicleFillOrderStart",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_WeekendObjectiveNext: Binding = Binding {
        id: "site_Career_WeekendObjectiveNext",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_WeekendObjectiveType: Binding = Binding {
        id: "site_Career_WeekendObjectiveType",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Career_WeekendsTrackRecordLoad: Binding = Binding {
        id: "site_Career_WeekendsTrackRecordLoad",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeFlow_SetStateStatus: Binding = Binding {
        id: "site_FeFlow_SetStateStatus",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeHome_AfterCareerCall: Binding = Binding {
        id: "site_FeHome_AfterCareerCall",
        kind: "site",
        status: Status::Stable,
    };
    pub const site_FeTile_CareerContentEnd: Binding = Binding {
        id: "site_FeTile_CareerContentEnd",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeTile_EraTilesBuilt: Binding = Binding {
        id: "site_FeTile_EraTilesBuilt",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeTile_ListRowEntry: Binding = Binding {
        id: "site_FeTile_ListRowEntry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeTile_ListRowFieldLoad: Binding = Binding {
        id: "site_FeTile_ListRowFieldLoad",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeTile_ListRowNone: Binding = Binding {
        id: "site_FeTile_ListRowNone",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeTile_ListScanNone: Binding = Binding {
        id: "site_FeTile_ListScanNone",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_FeTile_ListScanStart: Binding = Binding {
        id: "site_FeTile_ListScanStart",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_AcclimPenalty_Entry: Binding = Binding {
        id: "site_Game_AcclimPenalty_Entry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_AcclimPenalty_Return: Binding = Binding {
        id: "site_Game_AcclimPenalty_Return",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_CareerContinue_DriverSlotTitle: Binding = Binding {
        id: "site_Game_CareerContinue_DriverSlotTitle",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Game_CareerWizardConfirm_NextRoute: Binding = Binding {
        id: "site_Game_CareerWizardConfirm_NextRoute",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_CareerWizard_BackRoute: Binding = Binding {
        id: "site_Game_CareerWizard_BackRoute",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_DriverSelectHeader_BuildTabs: Binding = Binding {
        id: "site_Game_DriverSelectHeader_BuildTabs",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Game_DriverSelect_BackRoute: Binding = Binding {
        id: "site_Game_DriverSelect_BackRoute",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_DriverSelect_BadgeFormat: Binding = Binding {
        id: "site_Game_DriverSelect_BadgeFormat",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Game_DriverSelect_NumberResult: Binding = Binding {
        id: "site_Game_DriverSelect_NumberResult",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_DriverSelect_TeamLogoFormat: Binding = Binding {
        id: "site_Game_DriverSelect_TeamLogoFormat",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Game_ErsNewLap_NextVehicle: Binding = Binding {
        id: "site_Game_ErsNewLap_NextVehicle",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_ErsNewLap_VehicleLookup: Binding = Binding {
        id: "site_Game_ErsNewLap_VehicleLookup",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_GarageCrew_SceneRead: Binding = Binding {
        id: "site_Game_GarageCrew_SceneRead",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_GarageCrew_SceneTest: Binding = Binding {
        id: "site_Game_GarageCrew_SceneTest",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_GarageCrew_SpawnVector: Binding = Binding {
        id: "site_Game_GarageCrew_SpawnVector",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_GarageSetups_EntryStep: Binding = Binding {
        id: "site_Game_GarageSetups_EntryStep",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_GarageSetups_NotFound: Binding = Binding {
        id: "site_Game_GarageSetups_NotFound",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_PitLane_BoxPoseFalse: Binding = Binding {
        id: "site_Game_PitLane_BoxPoseFalse",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_PitLane_BoxSlotRead: Binding = Binding {
        id: "site_Game_PitLane_BoxSlotRead",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_PitStops_CrewTemplateRead: Binding = Binding {
        id: "site_Game_PitStops_CrewTemplateRead",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_PitStops_CrewTemplateSkip: Binding = Binding {
        id: "site_Game_PitStops_CrewTemplateSkip",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_SaveData_ContainersChildren: Binding = Binding {
        id: "site_Game_SaveData_ContainersChildren",
        kind: "site",
        status: Status::Stable,
    };
    pub const site_Game_SaveData_ContainersCount: Binding = Binding {
        id: "site_Game_SaveData_ContainersCount",
        kind: "site",
        status: Status::Stable,
    };
    pub const site_Game_SaveData_ContainersDone: Binding = Binding {
        id: "site_Game_SaveData_ContainersDone",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_SaveData_UnclaimedChild: Binding = Binding {
        id: "site_Game_SaveData_UnclaimedChild",
        kind: "site",
        status: Status::Stable,
    };
    pub const site_Game_SaveSlot_Selected: Binding = Binding {
        id: "site_Game_SaveSlot_Selected",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_TeamSelect_NumberResult: Binding = Binding {
        id: "site_Game_TeamSelect_NumberResult",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_TeamSelect_TeamLogoFormat: Binding = Binding {
        id: "site_Game_TeamSelect_TeamLogoFormat",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Game_ThrustmasterLcd_EntrantRecord: Binding = Binding {
        id: "site_Game_ThrustmasterLcd_EntrantRecord",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_ThrustmasterLcd_SectorTail: Binding = Binding {
        id: "site_Game_ThrustmasterLcd_SectorTail",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_TimeTrialLeaderboard_BoardRead: Binding = Binding {
        id: "site_Game_TimeTrialLeaderboard_BoardRead",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Game_TimeTrialLeaderboard_BoardsDone: Binding = Binding {
        id: "site_Game_TimeTrialLeaderboard_BoardsDone",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Gdb_DeptCountStore: Binding = Binding {
        id: "site_Gdb_DeptCountStore",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Gdb_InsertGrowCall: Binding = Binding {
        id: "site_Gdb_InsertGrowCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Gdb_ReserveTarget: Binding = Binding {
        id: "site_Gdb_ReserveTarget",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ne_Format_String_Entry: Binding = Binding {
        id: "site_Ne_Format_String_Entry",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_CrowdInShadows: Binding = Binding {
        id: "site_Ne_HwBool_CrowdInShadows",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_GodRaysPost: Binding = Binding {
        id: "site_Ne_HwBool_GodRaysPost",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_GrassFlattening: Binding = Binding {
        id: "site_Ne_HwBool_GrassFlattening",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_GroundCoverEnabled: Binding = Binding {
        id: "site_Ne_HwBool_GroundCoverEnabled",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_GroundCoverMeshes: Binding = Binding {
        id: "site_Ne_HwBool_GroundCoverMeshes",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_LensStreak: Binding = Binding {
        id: "site_Ne_HwBool_LensStreak",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_MotionBlurV2: Binding = Binding {
        id: "site_Ne_HwBool_MotionBlurV2",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_NightShadowCharCast: Binding = Binding {
        id: "site_Ne_HwBool_NightShadowCharCast",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_RainBeads: Binding = Binding {
        id: "site_Ne_HwBool_RainBeads",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_RainHigh: Binding = Binding {
        id: "site_Ne_HwBool_RainHigh",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_RainSheets: Binding = Binding {
        id: "site_Ne_HwBool_RainSheets",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_RainSplashes: Binding = Binding {
        id: "site_Ne_HwBool_RainSplashes",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_SkyShadowLowLod: Binding = Binding {
        id: "site_Ne_HwBool_SkyShadowLowLod",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_SkyShadowMap32Bits: Binding = Binding {
        id: "site_Ne_HwBool_SkyShadowMap32Bits",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_TrackHeatHaze: Binding = Binding {
        id: "site_Ne_HwBool_TrackHeatHaze",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwBool_VolumetricFog: Binding = Binding {
        id: "site_Ne_HwBool_VolumetricFog",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwConfigParseCall: Binding = Binding {
        id: "site_Ne_HwConfigParseCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_CrowdBillboardDistance: Binding = Binding {
        id: "site_Ne_HwInt_CrowdBillboardDistance",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_NightShadowMapSize: Binding = Binding {
        id: "site_Ne_HwInt_NightShadowMapSize",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_NightShadowSourceCount: Binding = Binding {
        id: "site_Ne_HwInt_NightShadowSourceCount",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_ShadowSampling: Binding = Binding {
        id: "site_Ne_HwInt_ShadowSampling",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_SkyShadowCascadeCount: Binding = Binding {
        id: "site_Ne_HwInt_SkyShadowCascadeCount",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_SkyShadowMapSize: Binding = Binding {
        id: "site_Ne_HwInt_SkyShadowMapSize",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_SpotShadowMapSize: Binding = Binding {
        id: "site_Ne_HwInt_SpotShadowMapSize",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_SpotShadowSampling: Binding = Binding {
        id: "site_Ne_HwInt_SpotShadowSampling",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_TexStreamSizeInMiB: Binding = Binding {
        id: "site_Ne_HwInt_TexStreamSizeInMiB",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_VolFogMapHeight: Binding = Binding {
        id: "site_Ne_HwInt_VolFogMapHeight",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_VolFogMapWidth: Binding = Binding {
        id: "site_Ne_HwInt_VolFogMapWidth",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwInt_WorldShadowMapSize: Binding = Binding {
        id: "site_Ne_HwInt_WorldShadowMapSize",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwXmlParseCall: Binding = Binding {
        id: "site_Ne_HwXmlParseCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_HwXmlWriteCall: Binding = Binding {
        id: "site_Ne_HwXmlWriteCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Ne_MemoryMapHeapSize: Binding = Binding {
        id: "site_Ne_MemoryMapHeapSize",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ne_MemoryMapInternalHeapSize: Binding = Binding {
        id: "site_Ne_MemoryMapInternalHeapSize",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ne_ReplayAfterRecord: Binding = Binding {
        id: "site_Ne_ReplayAfterRecord",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ne_WatchdogKillPath: Binding = Binding {
        id: "site_Ne_WatchdogKillPath",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Ne_WatchdogKillSkip: Binding = Binding {
        id: "site_Ne_WatchdogKillSkip",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_AiRecords_NameHash: Binding = Binding {
        id: "site_Sess_AiRecords_NameHash",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_AiVehicleRegisterCall: Binding = Binding {
        id: "site_Sess_AiVehicleRegisterCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Sess_ControllerSystem_ApplyRecordCall: Binding = Binding {
        id: "site_Sess_ControllerSystem_ApplyRecordCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Sess_GarageSlots_Done: Binding = Binding {
        id: "site_Sess_GarageSlots_Done",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_GarageSlots_GarageLoad: Binding = Binding {
        id: "site_Sess_GarageSlots_GarageLoad",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_InGamePlayerFinishedAiCall: Binding = Binding {
        id: "site_Sess_InGamePlayerFinishedAiCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Sess_InGameSessionCompleteAiCall: Binding = Binding {
        id: "site_Sess_InGameSessionCompleteAiCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Sess_PitEvalListNext: Binding = Binding {
        id: "site_Sess_PitEvalListNext",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PitEvalListSlot: Binding = Binding {
        id: "site_Sess_PitEvalListSlot",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PitEvalOwnSlot: Binding = Binding {
        id: "site_Sess_PitEvalOwnSlot",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PitEvalReturnFalse: Binding = Binding {
        id: "site_Sess_PitEvalReturnFalse",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PitRankEpilogue: Binding = Binding {
        id: "site_Sess_PitRankEpilogue",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PitRankListNext: Binding = Binding {
        id: "site_Sess_PitRankListNext",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PitRankListSlot: Binding = Binding {
        id: "site_Sess_PitRankListSlot",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PitRankOwnSlot: Binding = Binding {
        id: "site_Sess_PitRankOwnSlot",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_PuPanel_SlotHolder: Binding = Binding {
        id: "site_Sess_PuPanel_SlotHolder",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_TimeTrialLeaderboard_TabChild: Binding = Binding {
        id: "site_Sess_TimeTrialLeaderboard_TabChild",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_TimeTrialLeaderboard_TabDone: Binding = Binding {
        id: "site_Sess_TimeTrialLeaderboard_TabDone",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_TyreVisSlotLookup: Binding = Binding {
        id: "site_Sess_TyreVisSlotLookup",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_TyreVisTailCall: Binding = Binding {
        id: "site_Sess_TyreVisTailCall",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_TyreWearReturn: Binding = Binding {
        id: "site_Sess_TyreWearReturn",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_TyreWearSlotLookup: Binding = Binding {
        id: "site_Sess_TyreWearSlotLookup",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Sess_WheelMapInsertCall: Binding = Binding {
        id: "site_Sess_WheelMapInsertCall",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Team_LoadInsertCall: Binding = Binding {
        id: "site_Team_LoadInsertCall",
        kind: "call",
        status: Status::Experimental,
    };
    pub const site_Team_LoadInserted: Binding = Binding {
        id: "site_Team_LoadInserted",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Team_SessionCarBadgeLabel: Binding = Binding {
        id: "site_Team_SessionCarBadgeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Team_TimingPillBadgeLabel: Binding = Binding {
        id: "site_Team_TimingPillBadgeLabel",
        kind: "site",
        status: Status::Experimental,
    };
    pub const site_Trk_TrackData_SetMatched: Binding = Binding {
        id: "site_Trk_TrackData_SetMatched",
        kind: "site",
        status: Status::Experimental,
    };
    pub const vt_Car_VehicleEnvironment: Binding = Binding {
        id: "vt_Car_VehicleEnvironment",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_BankTransactionMessage: Binding = Binding {
        id: "vt_Career_BankTransactionMessage",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_Championship: Binding = Binding {
        id: "vt_Career_Championship",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_FacilityNode: Binding = Binding {
        id: "vt_Career_FacilityNode",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_MessageDriverContractChanged: Binding = Binding {
        id: "vt_Career_MessageDriverContractChanged",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_MessageDriverMoveConfirmed: Binding = Binding {
        id: "vt_Career_MessageDriverMoveConfirmed",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_MessageDriverMovesFinalised: Binding = Binding {
        id: "vt_Career_MessageDriverMovesFinalised",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_MessageDriverMovesFinalisedRecognition: Binding = Binding {
        id: "vt_Career_MessageDriverMovesFinalisedRecognition",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_MessageDriverMovesFinalisedUpgrades: Binding = Binding {
        id: "vt_Career_MessageDriverMovesFinalisedUpgrades",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_RenownAwardedMessage: Binding = Binding {
        id: "vt_Career_RenownAwardedMessage",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_RnDDeptCard: Binding = Binding {
        id: "vt_Career_RnDDeptCard",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_Season: Binding = Binding {
        id: "vt_Career_Season",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_SponsorLoyaltyXpMessage: Binding = Binding {
        id: "vt_Career_SponsorLoyaltyXpMessage",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Career_Weekend: Binding = Binding {
        id: "vt_Career_Weekend",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Game_CareerDLCSelectViewModel: Binding = Binding {
        id: "vt_Game_CareerDLCSelectViewModel",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Game_CareerWizardSettingsSelectViewModel: Binding = Binding {
        id: "vt_Game_CareerWizardSettingsSelectViewModel",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Game_DriverCareerDriverSelectViewModel: Binding = Binding {
        id: "vt_Game_DriverCareerDriverSelectViewModel",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Game_DriverCareerSlotPredicate: Binding = Binding {
        id: "vt_Game_DriverCareerSlotPredicate",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Game_FECareerViewModel: Binding = Binding {
        id: "vt_Game_FECareerViewModel",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Game_SaveDataRootContainer: Binding = Binding {
        id: "vt_Game_SaveDataRootContainer",
        kind: "vtable",
        status: Status::Stable,
    };
    pub const vt_Game_TrackInformationModule: Binding = Binding {
        id: "vt_Game_TrackInformationModule",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Game_TrackListComponent: Binding = Binding {
        id: "vt_Game_TrackListComponent",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Ne_ParticleEffectComponent: Binding = Binding {
        id: "vt_Ne_ParticleEffectComponent",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Ne_SaveSystem2Win: Binding = Binding {
        id: "vt_Ne_SaveSystem2Win",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_AiController: Binding = Binding {
        id: "vt_Sess_AiController",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_CareerHubLocation: Binding = Binding {
        id: "vt_Sess_CareerHubLocation",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ContractNegotiations: Binding = Binding {
        id: "vt_Sess_ContractNegotiations",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_CutsceneLocation: Binding = Binding {
        id: "vt_Sess_CutsceneLocation",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_CutsceneLocationLoad: Binding = Binding {
        id: "vt_Sess_CutsceneLocationLoad",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_CutsceneLocationUnload: Binding = Binding {
        id: "vt_Sess_CutsceneLocationUnload",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_DelayOnLoadingScreen: Binding = Binding {
        id: "vt_Sess_DelayOnLoadingScreen",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_DriverCareerContractNegotiations: Binding = Binding {
        id: "vt_Sess_DriverCareerContractNegotiations",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_DriverCareerContractNegotiationsIntro: Binding = Binding {
        id: "vt_Sess_DriverCareerContractNegotiationsIntro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_DriverCareerContractNegotiationsOutro: Binding = Binding {
        id: "vt_Sess_DriverCareerContractNegotiationsOutro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_DriverCareerContractNegotiationsTeamSelect: Binding = Binding {
        id: "vt_Sess_DriverCareerContractNegotiationsTeamSelect",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_DriverHQIntro: Binding = Binding {
        id: "vt_Sess_DriverHQIntro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_EndOfSeasonCutscene: Binding = Binding {
        id: "vt_Sess_EndOfSeasonCutscene",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_F1CGIntro: Binding = Binding {
        id: "vt_Sess_F1CGIntro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_F1CGOutro: Binding = Binding {
        id: "vt_Sess_F1CGOutro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_F1CGUpdate: Binding = Binding {
        id: "vt_Sess_F1CGUpdate",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_Frontend: Binding = Binding {
        id: "vt_Sess_Frontend",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_FrontendWorld: Binding = Binding {
        id: "vt_Sess_FrontendWorld",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_GetEventToken: Binding = Binding {
        id: "vt_Sess_GetEventToken",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_InGamePlaying: Binding = Binding {
        id: "vt_Sess_InGamePlaying",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_Intro: Binding = Binding {
        id: "vt_Sess_Intro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_Launch: Binding = Binding {
        id: "vt_Sess_Launch",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_LoadSession: Binding = Binding {
        id: "vt_Sess_LoadSession",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_LocationLoad: Binding = Binding {
        id: "vt_Sess_LocationLoad",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_LocationUnload: Binding = Binding {
        id: "vt_Sess_LocationUnload",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_OSQSyncReady: Binding = Binding {
        id: "vt_Sess_OSQSyncReady",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_OnlineConnect: Binding = Binding {
        id: "vt_Sess_OnlineConnect",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_OnlineFrontend: Binding = Binding {
        id: "vt_Sess_OnlineFrontend",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_OnlineLobby: Binding = Binding {
        id: "vt_Sess_OnlineLobby",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_Persistent: Binding = Binding {
        id: "vt_Sess_Persistent",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_PlayCutScene: Binding = Binding {
        id: "vt_Sess_PlayCutScene",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_PostLoadDataSync: Binding = Binding {
        id: "vt_Sess_PostLoadDataSync",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_PreRace: Binding = Binding {
        id: "vt_Sess_PreRace",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_PressInterview: Binding = Binding {
        id: "vt_Sess_PressInterview",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_PressInterviewBase: Binding = Binding {
        id: "vt_Sess_PressInterviewBase",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_QualifyingSyncReady: Binding = Binding {
        id: "vt_Sess_QualifyingSyncReady",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_RaceRuleset: Binding = Binding {
        id: "vt_Sess_RaceRuleset",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ReadyMonitor: Binding = Binding {
        id: "vt_Sess_ReadyMonitor",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_RedFlagSystem: Binding = Binding {
        id: "vt_Sess_RedFlagSystem",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ReplaySessionLoaded: Binding = Binding {
        id: "vt_Sess_ReplaySessionLoaded",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScAheadOfLead: Binding = Binding {
        id: "vt_Sess_ScAheadOfLead",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScCanReturn: Binding = Binding {
        id: "vt_Sess_ScCanReturn",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScCarsGrouped: Binding = Binding {
        id: "vt_Sess_ScCarsGrouped",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScDebugDeploy: Binding = Binding {
        id: "vt_Sess_ScDebugDeploy",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScForceReturn: Binding = Binding {
        id: "vt_Sess_ScForceReturn",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScRaceLapsRemaining: Binding = Binding {
        id: "vt_Sess_ScRaceLapsRemaining",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScTriggerRedFlag: Binding = Binding {
        id: "vt_Sess_ScTriggerRedFlag",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_ScTriggerVsc: Binding = Binding {
        id: "vt_Sess_ScTriggerVsc",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SecretMeeting: Binding = Binding {
        id: "vt_Sess_SecretMeeting",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SecretMeetingOutro: Binding = Binding {
        id: "vt_Sess_SecretMeetingOutro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_Session: Binding = Binding {
        id: "vt_Sess_Session",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SessionBaseState: Binding = Binding {
        id: "vt_Sess_SessionBaseState",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SessionTransition: Binding = Binding {
        id: "vt_Sess_SessionTransition",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SillySeasonCutscene: Binding = Binding {
        id: "vt_Sess_SillySeasonCutscene",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SmokeTestSkipFEPreLaunchHold: Binding = Binding {
        id: "vt_Sess_SmokeTestSkipFEPreLaunchHold",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SpectatorReadyMonitor: Binding = Binding {
        id: "vt_Sess_SpectatorReadyMonitor",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_StartLoadingFlow: Binding = Binding {
        id: "vt_Sess_StartLoadingFlow",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_StoryHub: Binding = Binding {
        id: "vt_Sess_StoryHub",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_StoryHubDepartmentEvents: Binding = Binding {
        id: "vt_Sess_StoryHubDepartmentEvents",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_StoryHubDepartmentEventsBase: Binding = Binding {
        id: "vt_Sess_StoryHubDepartmentEventsBase",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_StoryHubDepartmentEventsIntro: Binding = Binding {
        id: "vt_Sess_StoryHubDepartmentEventsIntro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_StoryHubDepartmentEventsOutro: Binding = Binding {
        id: "vt_Sess_StoryHubDepartmentEventsOutro",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_SynchroniseCloseDocument: Binding = Binding {
        id: "vt_Sess_SynchroniseCloseDocument",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_TeamHQ: Binding = Binding {
        id: "vt_Sess_TeamHQ",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_TeamHQToSession: Binding = Binding {
        id: "vt_Sess_TeamHQToSession",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_TeamHQUpdate: Binding = Binding {
        id: "vt_Sess_TeamHQUpdate",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_TyreAllocation: Binding = Binding {
        id: "vt_Sess_TyreAllocation",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_UnloadRewardSequence: Binding = Binding {
        id: "vt_Sess_UnloadRewardSequence",
        kind: "vtable",
        status: Status::Experimental,
    };
    pub const vt_Sess_Video: Binding = Binding {
        id: "vt_Sess_Video",
        kind: "vtable",
        status: Status::Experimental,
    };
}

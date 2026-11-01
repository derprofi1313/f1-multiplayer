#pragma once
#include <atomic>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <algorithm>
#include <functional>
#include <mutex>
#include <string>
#include <vector>

#include <swofty_api.h>

// The core's live car operations (SwoftyCoreCarOps, standard library 1.6): cars::live reads and writes a
// car's handling values while it is on track. A car is a slot of g_Sess_VehicleTable (its CarLive, whose
// vehicle_id is the slot); each value is a field of one of the systems the car caches by section name at
// CarLive+0x530..+0x5e0. Every offset below is a KB field (re/db/structs/{CarLive,CarSteering,CarGearBox,
// CarDifferential,CarTurbo,CarEngine,CarFuelTank,CarDamageFrontWing,CarDamageSuspension,CarObfFloat}.h, each
// exported by bindgen: L2 with an agree verify row, or L3), and "obf" fields are CarObfFloat pairs: the float's
// bits XOR the key 8 bytes on. The offsets are the KB build's, so car_live.cpp hands over no table on any
// other exe. Ops<Tag> keeps the state per tag so the host test runs the same code on fake cars.

namespace swofty_core::hooks::car_live {

constexpr uintptr_t CAR_VEHICLE_ID = 0xe8, CAR_IS_AI = 0x124, CAR_LOADER = 0x4a0, SYSTEM_CAR = 0x08;
constexpr uintptr_t FUEL_CAPACITY = 0x98;
constexpr uint32_t MAX_CARS = 32;

constexpr uintptr_t ENGINE = 0x530, STEERING = 0x560, GEARBOX = 0x538,
                    DIFFERENTIAL = 0x578, TURBO = 0x5a0, FUEL_TANK = 0x540,
                    WING_DAMAGE = 0x5c8, SUSPENSION_DAMAGE = 0x5d0;

enum class Kind { plain, obf, virt };

// What set re-runs after the write so the game's derived copy follows: the engine's derived values
// (Car_Engine_ComputeDerived_Thunk), the gearbox's ratio table (Car_Tuning_SetFinalDrive with 0x20|0x2000, which
// leaves the current gear alone; 0x200 would shift to neutral) or the chassis' mass (sqrt_mass, then the system's
// OnReset with 0x208, flag 0x8 being the slot's gate).
enum class Rederive { none, engine, gearbox, chassis };
// Where the field's storage is: in the system at Field::system, in the suspension's tyre record of an axle (found by
// its name), or in the suspension's axle object (the rear one has the lower local rest position).
enum class Via { system, tyre_front, tyre_rear, axle_front, axle_rear };
constexpr uintptr_t GEARBOX_USE_SETUP_RATIOS = 0x540;
constexpr uint32_t GEARBOX_REBUILD = 0x2020;
constexpr uint32_t CHASSIS_REBUILD = 0x208;
constexpr uintptr_t SUSPENSION = 0x580, SUSP_TYRE_TYPES = 0x38, SUSP_AXLE0 = 0x90, SUSP_AXLE1 = 0x98;
constexpr uintptr_t AXLE_LOCAL_REST_Z = 0x1598, TYRE_TYPE_NAME = 0x00;
constexpr uintptr_t CHASSIS_SQRT_MASS = 0x98;
constexpr uintptr_t CHASSIS_GROUP = 0x118, STRING_CAPACITY = 0x08, STRING_BUFFER = 0x10;
constexpr uint32_t NO_VIRTUAL = 0xff;
constexpr uint32_t VIRTUALS = 3;

struct Field {
    uint32_t id;
    const char *name;
    uintptr_t system, offset;
    Kind kind;
    float lo, hi;
    Rederive rederive = Rederive::none;
    Via via = Via::system;
    const char *needs = nullptr;
    float neutral = 0;
    bool relative = false;
    uint32_t vslot = NO_VIRTUAL;
};

inline constexpr float BIG = 1e9f;
constexpr uintptr_t F1_AERO = 0x570, BRAKE_ECU = 0x550, CHASSIS = 0x558;

inline constexpr Field FIELDS[] = {
    {SWOFTY_CAR_STEERING_LOCK, "steering.max_steering_lock", STEERING, 0x28, Kind::obf, 0.01f, 1.5708f},
    {SWOFTY_CAR_CLUTCH_TORQUE, "gearbox.max_clutch_torque", GEARBOX, 0xe8, Kind::plain, 0, 100},
    {SWOFTY_CAR_DIFF_ON_THROTTLE, "differential.diff_on_throttle", DIFFERENTIAL, 0xf0, Kind::plain, 0, 1},
    {SWOFTY_CAR_DIFF_OFF_THROTTLE, "differential.diff_off_throttle", DIFFERENTIAL, 0xec, Kind::plain, 0, 1},
    {SWOFTY_CAR_TURBO_MAX_BOOST, "turbo.max_boost", TURBO, 0x2d8, Kind::plain, 0, 100},
    {SWOFTY_CAR_FUEL_MASS, "fuel_tank.fuel_mass", FUEL_TANK, 0x100, Kind::plain, 0, BIG},
    {SWOFTY_CAR_WING_SPEED_THRESHOLD, "f1_wing_damage.speed_threshold", WING_DAMAGE, 0x28, Kind::plain, 0, BIG},
    {SWOFTY_CAR_WING_DAMAGE_SCALE, "f1_wing_damage.damage_scale", WING_DAMAGE, 0x2c, Kind::plain, 0, BIG},
    {SWOFTY_CAR_WING_CHASSIS_DAMAGE_MULTIPLE, "f1_wing_damage.chassis_damage_multiple", WING_DAMAGE, 0x30, Kind::plain,
     0, BIG},
    {SWOFTY_CAR_WING_SIDE_DETACH_SCALE, "f1_wing_damage.side_detach_scale", WING_DAMAGE, 0x34, Kind::plain, 0, BIG},
    {SWOFTY_CAR_WING_FULL_DETACH_SCALE, "f1_wing_damage.full_detach_scale", WING_DAMAGE, 0x38, Kind::plain, 0, BIG},
    {SWOFTY_CAR_SUSP_SPEED_THRESHOLD, "f1_suspension_damage.speed_threshold", SUSPENSION_DAMAGE, 0x28, Kind::obf, 0,
     BIG},
    {SWOFTY_CAR_SUSP_FRONT_DAMAGE_SCALE, "f1_suspension_damage.front_damage_scale", SUSPENSION_DAMAGE, 0x38, Kind::obf,
     0, BIG},
    {SWOFTY_CAR_SUSP_REAR_DAMAGE_SCALE, "f1_suspension_damage.rear_damage_scale", SUSPENSION_DAMAGE, 0x48, Kind::obf, 0,
     BIG},
    {SWOFTY_CAR_SUSP_WHEEL_DETACH_SCALE, "f1_suspension_damage.wheel_detach_scale", SUSPENSION_DAMAGE, 0x58, Kind::obf,
     0, BIG},
    {SWOFTY_CAR_SUSP_PARTS_DETACH_SCALE, "f1_suspension_damage.parts_detach_scale", SUSPENSION_DAMAGE, 0x68, Kind::obf,
     0, BIG},
    {SWOFTY_CAR_SUSP_LINEAR_FRACTURE_LIMIT, "f1_suspension_damage.linear_fracture_limit", SUSPENSION_DAMAGE, 0x78,
     Kind::plain, 0, BIG},
    {SWOFTY_CAR_SUSP_ANGULAR_FRACTURE_LIMIT, "f1_suspension_damage.angular_fracture_limit", SUSPENSION_DAMAGE, 0x7c,
     Kind::plain, 0, BIG},
    {SWOFTY_CAR_ENGINE_LIMITER, "engine.vtf_limiter_rotation_rate", ENGINE, 0x8b0, Kind::obf, 100, 2500,
     Rederive::engine},
    {SWOFTY_CAR_FINAL_DRIVE, "gearbox.final_drive_ratio", GEARBOX, 0x28, Kind::obf, 1, 20, Rederive::gearbox},
    // Ratios R, N, 1st..9th are CarObfFloat pairs at +0x38 stride 0x10: 1st is +0x58.
    {SWOFTY_CAR_GEAR_RATIO_1, "gearbox.gear_ratio_1", GEARBOX, 0x58, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 1, "gearbox.gear_ratio_2", GEARBOX, 0x68, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 2, "gearbox.gear_ratio_3", GEARBOX, 0x78, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 3, "gearbox.gear_ratio_4", GEARBOX, 0x88, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 4, "gearbox.gear_ratio_5", GEARBOX, 0x98, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 5, "gearbox.gear_ratio_6", GEARBOX, 0xa8, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 6, "gearbox.gear_ratio_7", GEARBOX, 0xb8, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 7, "gearbox.gear_ratio_8", GEARBOX, 0xc8, Kind::obf, 0.1f, 20, Rederive::gearbox},
    {SWOFTY_CAR_GEAR_RATIO_1 + 8, "gearbox.gear_ratio_9", GEARBOX, 0xd8, Kind::obf, 0.1f, 20, Rederive::gearbox},
    // Version 2. needs names the KB row that must have an independent agree verify row before the field acts.
    {SWOFTY_CAR_AERO_LIFT_SCALE, "aero.lift_scale", F1_AERO, 0x19a4, Kind::plain, 0, 10, Rederive::none, Via::system,
     "CarF1Aero.lift_scale"},
    {SWOFTY_CAR_AERO_DRAG_SCALE, "aero.drag_scale", F1_AERO, 0x19a0, Kind::plain, 0, 10, Rederive::none, Via::system,
     "CarF1Aero.drag_scale"},
    {SWOFTY_CAR_ENGINE_POWER_SCALE, "engine.power_scale", 0, 0, Kind::virt, 0, 4, Rederive::none, Via::system,
     "site_Car_EngineDriveTorque", 1, false, 0},
    {SWOFTY_CAR_ENGINE_LIMITER_OFFSET_RPM, "engine.limiter_offset_rpm", 0, 0, Kind::virt, -5000, 5000, Rederive::none,
     Via::system, "site_Car_EngineDriveTorque", 0, false, 1},
    {SWOFTY_CAR_AI_GRIP_SCALE, "ai.grip_scale", 0, 0, Kind::virt, 0, 4, Rederive::none, Via::system,
     "site_Ai_TableGripInput", 1, false, 2},
    {SWOFTY_CAR_TYRES_FRONT_GRIP_SCALE, "tyres.front.grip_scale", 0, 0x2c, Kind::plain, 0, 10, Rederive::none,
     Via::tyre_front, "CarSuspTyreType.grip_multiplier"},
    {SWOFTY_CAR_TYRES_REAR_GRIP_SCALE, "tyres.rear.grip_scale", 0, 0x2c, Kind::plain, 0, 10, Rederive::none,
     Via::tyre_rear, "CarSuspTyreType.grip_multiplier"},
    {SWOFTY_CAR_CHASSIS_BALLAST_KG, "chassis.ballast_kg", CHASSIS, 0x28, Kind::obf, -300, 600, Rederive::chassis,
     Via::system, "Car_Tuning_SetChassisZCom", 0, true},
    {SWOFTY_CAR_BRAKES_BIAS, "brakes.bias", BRAKE_ECU, 0x21d8, Kind::plain, 0, 1, Rederive::none, Via::system,
     "CarBrakeEcu.brake_bias"},
    {SWOFTY_CAR_BRAKES_PRESSURE, "brakes.pressure", BRAKE_ECU, 0x21dc, Kind::plain, 0, 4, Rederive::none, Via::system,
     "CarBrakeEcu.brake_pressure"},
    {SWOFTY_CAR_SUSP_FRONT_CAMBER, "suspension.front.camber", 0, 0x1650, Kind::plain, -1, 1, Rederive::none,
     Via::axle_front, "CarSuspAxle.setup_camber"},
    {SWOFTY_CAR_SUSP_FRONT_TOE, "suspension.front.toe", 0, 0x1654, Kind::plain, -1, 1, Rederive::none, Via::axle_front,
     "CarSuspAxle.setup_toe"},
    {SWOFTY_CAR_SUSP_FRONT_ANTI_ROLL, "suspension.front.anti_roll", 0, 0x1664, Kind::plain, 0, BIG, Rederive::none,
     Via::axle_front, "CarSuspAxle.setup_anti_roll"},
    {SWOFTY_CAR_SUSP_FRONT_RIDE_HEIGHT_OFFSET, "suspension.front.ride_height_offset", 0, 0x15e4, Kind::plain, -1, 1,
     Rederive::none, Via::axle_front, "CarSuspAxle.ride_height_offset"},
    {SWOFTY_CAR_SUSP_REAR_CAMBER, "suspension.rear.camber", 0, 0x1650, Kind::plain, -1, 1, Rederive::none,
     Via::axle_rear, "CarSuspAxle.setup_camber"},
    {SWOFTY_CAR_SUSP_REAR_TOE, "suspension.rear.toe", 0, 0x1654, Kind::plain, -1, 1, Rederive::none, Via::axle_rear,
     "CarSuspAxle.setup_toe"},
    {SWOFTY_CAR_SUSP_REAR_ANTI_ROLL, "suspension.rear.anti_roll", 0, 0x1664, Kind::plain, 0, BIG, Rederive::none,
     Via::axle_rear, "CarSuspAxle.setup_anti_roll"},
    {SWOFTY_CAR_SUSP_REAR_RIDE_HEIGHT_OFFSET, "suspension.rear.ride_height_offset", 0, 0x15e4, Kind::plain, -1, 1,
     Rederive::none, Via::axle_rear, "CarSuspAxle.ride_height_offset"},
};

inline constexpr size_t FIELD_COUNT = sizeof FIELDS / sizeof FIELDS[0];

inline int field_index(uint32_t id)
{
    for (size_t i = 0; i < FIELD_COUNT; i++)
        if (FIELDS[i].id == id)
            return int(i);
    return -1;
}

inline const Field *find_field_path(const char *path)
{
    if (!path)
        return nullptr;
    for (auto &f : FIELDS)
        if (!std::strcmp(f.name, path))
            return &f;
    return nullptr;
}

inline const Field *field(uint32_t id)
{
    int i = field_index(id);
    return i < 0 ? nullptr : &FIELDS[i];
}

struct Env {
    std::function<bool(uintptr_t addr, void *out, size_t n)> read;
    std::function<bool(uintptr_t addr, const void *in, size_t n)> write;
    std::function<bool()> flow_thread;
    std::function<bool()> enabled;
    std::function<void(uintptr_t engine)> engine_derived;
    std::function<void(uintptr_t gearbox, uint32_t flags)> gearbox_reset;
    std::function<void(uintptr_t chassis, uint32_t flags)> chassis_reset;
    std::function<uintptr_t(uint32_t index, uintptr_t car)> tyre_component;
    std::function<void(uintptr_t tyre_component)> tyre_refit;
    // Whether the KB row a field needs has its independent agree verify row; null means every row is ready.
    std::function<bool(const char *row)> row_ready;
    uintptr_t table = 0, count = 0;
};

constexpr float RPM_TO_RAD_S = 0.10471976f;
constexpr float LIMITER_RAMP_RPM = 60.0f;

// The engine drive torque per full throttle after the car's power factor and limiter offset (the cut starts
// LIMITER_RAMP_RPM under the lowered limiter and reaches zero torque at it). A positive offset changes nothing.
inline float shaped_torque(float torque, float rad_s, float limiter_rad_s, float power_scale, float limiter_offset_rpm)
{
    float k = power_scale;
    if (limiter_offset_rpm < 0 && limiter_rad_s > 0) {
        float cut = limiter_rad_s + limiter_offset_rpm * RPM_TO_RAD_S;
        float ramp = LIMITER_RAMP_RPM * RPM_TO_RAD_S;
        if (cut > 0 && rad_s > cut - ramp) {
            float f = (cut - rad_s) / ramp;
            k *= f > 0.f ? f : 0.f;
        }
    }
    return torque * k;
}

template <class Tag> struct Ops {
    static inline Env env;
    static inline std::atomic<bool> bound{false};

    struct LayerRec {
        uint32_t token;
        std::string owner;
        uint32_t index, field;
        uint32_t op;
        float value;
        int32_t priority;
        uint64_t order;
    };
    struct Slot {
        bool valid = false;
        float base = 0, written = 0;
    };
    static inline std::mutex lay_lock;
    static inline std::vector<LayerRec> layers;
    static inline uint32_t next_token = 1;
    static inline uint64_t next_order = 0;
    static inline Slot slots[MAX_CARS][FIELD_COUNT];
    static inline std::atomic<float> virt[MAX_CARS][VIRTUALS];
    static inline std::atomic<bool> dirty{false}, power_active{false}, ai_active{false};
    static inline std::mutex slot_lock;

    static void reset_virtuals()
    {
        for (uint32_t c = 0; c < MAX_CARS; c++)
            for (auto &f : FIELDS)
                if (f.kind == Kind::virt)
                    virt[c][f.vslot] = f.neutral;
        power_active = false;
        ai_active = false;
    }

    static void bind(Env e)
    {
        bound = false;
        env = std::move(e);
        {
            std::lock_guard<std::mutex> g(lay_lock);
            layers.clear();
            next_token = 1;
            next_order = 0;
        }
        {
            std::lock_guard<std::mutex> g(slot_lock);
            for (auto &row : slots)
                for (auto &s : row)
                    s = Slot{};
        }
        reset_virtuals();
        dirty = false;
        std::lock_guard<std::mutex> g(pending_lock);
        pending.clear();
        bound = true;
    }

    template <class T> static bool rd(uintptr_t a, T &v) { return a && env.read && env.read(a, &v, sizeof v); }

    static int available() { return bound.load() && env.table && env.count && (!env.enabled || env.enabled()) ? 1 : 0; }

    static bool ready(const Field &f) { return !f.needs || !env.row_ready || env.row_ready(f.needs); }

    static int cars(uint32_t &n)
    {
        uint32_t c = 0;
        if (!available() || !rd(env.count, c))
            return SWOFTY_ASSET_UNAVAILABLE;
        n = c < MAX_CARS ? c : MAX_CARS;
        return SWOFTY_ASSET_OK;
    }

    static int car_at(uint32_t index, const void *handle, uintptr_t &car)
    {
        uint32_t n = 0;
        if (int r = cars(n); r != SWOFTY_ASSET_OK)
            return r;
        if (index >= MAX_CARS)
            return SWOFTY_ASSET_INVALID;
        uint8_t id = 0;
        if (index >= n || !rd(env.table + index * 8, car) || !car || !rd(car + CAR_VEHICLE_ID, id) || id != index)
            return SWOFTY_ASSET_LATE;
        if (handle && car != reinterpret_cast<uintptr_t>(handle))
            return SWOFTY_ASSET_LATE;
        return SWOFTY_ASSET_OK;
    }

    static int system_at(uintptr_t car, uintptr_t offset, uintptr_t &sys)
    {
        uintptr_t owner = 0;
        if (!rd(car + offset, sys))
            return SWOFTY_ASSET_LATE;
        if (!sys)
            return SWOFTY_ASSET_UNAVAILABLE;
        return rd(sys + SYSTEM_CAR, owner) && owner == car ? SWOFTY_ASSET_OK : SWOFTY_ASSET_LATE;
    }

    // The address of a field's storage on one car; sys is the system the field belongs to (for a rederive).
    static int locate(uintptr_t car, const Field &f, uintptr_t &sys, uintptr_t &at)
    {
        if (f.kind == Kind::virt)
            return SWOFTY_ASSET_UNAVAILABLE;
        if (f.via == Via::system) {
            if (int r = system_at(car, f.system, sys); r != SWOFTY_ASSET_OK)
                return r;
            at = sys + f.offset;
            return SWOFTY_ASSET_OK;
        }
        if (int r = system_at(car, SUSPENSION, sys); r != SWOFTY_ASSET_OK)
            return r;
        if (f.via == Via::tyre_front || f.via == Via::tyre_rear) {
            const char *want = f.via == Via::tyre_front ? "Front" : "Rear";
            for (uintptr_t i = 0; i < 2; i++) {
                uintptr_t rec = 0;
                char name[16] = {};
                if (!rd(sys + SUSP_TYRE_TYPES + i * 8, rec) || !rec || !rd(rec + TYRE_TYPE_NAME, name))
                    continue;
                name[15] = 0;
                if (!std::strcmp(name, want)) {
                    at = rec + f.offset;
                    return SWOFTY_ASSET_OK;
                }
            }
            return SWOFTY_ASSET_UNAVAILABLE;
        }
        uintptr_t a0 = 0, a1 = 0;
        float z0 = 0, z1 = 0;
        if (!rd(sys + SUSP_AXLE0, a0) || !rd(sys + SUSP_AXLE1, a1) || !a0 || !a1 || !rd(a0 + AXLE_LOCAL_REST_Z, z0) ||
            !rd(a1 + AXLE_LOCAL_REST_Z, z1) || (z0 <= 0) == (z1 <= 0))
            return SWOFTY_ASSET_UNAVAILABLE;
        uintptr_t rear = z0 <= 0 ? a0 : a1, front = z0 <= 0 ? a1 : a0;
        at = (f.via == Via::axle_rear ? rear : front) + f.offset;
        return SWOFTY_ASSET_OK;
    }

    static bool load(uintptr_t at, Kind kind, float &v)
    {
        uint32_t a = 0, key = 0;
        if (!rd(at, a) || (kind == Kind::obf && !rd(at + 8, key)))
            return false;
        a ^= key;
        std::memcpy(&v, &a, 4);
        return true;
    }

    static bool store(uintptr_t at, Kind kind, float value)
    {
        uint32_t bits = 0, key = 0;
        std::memcpy(&bits, &value, 4);
        if (kind == Kind::obf && !rd(at + 8, key))
            return false;
        bits ^= key;
        return env.write && env.write(at, &bits, 4);
    }

    static int op_available() { return available(); }

    static int op_count(uint32_t *n)
    {
        uint32_t c = 0;
        int r = cars(c);
        if (r == SWOFTY_ASSET_OK && n)
            *n = c;
        return r;
    }

    static int op_car(uint32_t index, void **handle, uint32_t *is_ai)
    {
        uintptr_t car = 0;
        if (int r = car_at(index, nullptr, car); r != SWOFTY_ASSET_OK)
            return r;
        uint32_t ai = 0;
        if (is_ai && !rd(car + CAR_IS_AI, ai))
            return SWOFTY_ASSET_LATE;
        if (handle)
            *handle = reinterpret_cast<void *>(car);
        if (is_ai)
            *is_ai = ai;
        return SWOFTY_ASSET_OK;
    }

    static float composed(uint32_t index, const Field &f, float start)
    {
        std::vector<LayerRec> mine;
        {
            std::lock_guard<std::mutex> g(lay_lock);
            for (auto &l : layers)
                if (l.field == f.id && (l.index == SWOFTY_CAR_ALL || l.index == index))
                    mine.push_back(l);
        }
        std::stable_sort(mine.begin(), mine.end(), [](const LayerRec &a, const LayerRec &b) {
            return a.priority != b.priority ? a.priority < b.priority : a.order < b.order;
        });
        float v = start;
        for (auto &l : mine)
            v = l.op == SWOFTY_CAR_OP_SET ? l.value : l.op == SWOFTY_CAR_OP_SCALE ? v * l.value : v + l.value;
        return v;
    }

    static bool has_layer(uint32_t index, uint32_t field)
    {
        std::lock_guard<std::mutex> g(lay_lock);
        for (auto &l : layers)
            if (l.field == field && (l.index == SWOFTY_CAR_ALL || l.index == index))
                return true;
        return false;
    }

    static float clamp(const Field &f, float v) { return std::min(std::max(v, f.lo), f.hi); }

    static void refresh_virtuals()
    {
        bool power = false, ai = false;
        for (auto &f : FIELDS) {
            if (f.kind != Kind::virt)
                continue;
            for (uint32_t c = 0; c < MAX_CARS; c++) {
                float v = clamp(f, composed(c, f, f.neutral));
                virt[c][f.vslot] = v;
                if (v != f.neutral) {
                    if (f.vslot == 2)
                        ai = true;
                    else
                        power = true;
                }
            }
        }
        power_active = power;
        ai_active = ai;
    }

    static int op_get(uint32_t index, const void *handle, uint32_t id, float *value)
    {
        const Field *f = field(id);
        if (!f || !value)
            return SWOFTY_ASSET_INVALID;
        if (!ready(*f))
            return SWOFTY_ASSET_UNAVAILABLE;
        if (f->kind == Kind::virt) {
            if (index >= MAX_CARS)
                return SWOFTY_ASSET_INVALID;
            *value = virt[index][f->vslot].load();
            return SWOFTY_ASSET_OK;
        }
        uintptr_t car = 0, sys = 0, at = 0;
        if (int r = car_at(index, handle, car); r != SWOFTY_ASSET_OK)
            return r;
        if (int r = locate(car, *f, sys, at); r != SWOFTY_ASSET_OK)
            return r;
        float v = 0;
        if (!load(at, f->kind, v))
            return SWOFTY_ASSET_LATE;
        if (f->relative) {
            std::lock_guard<std::mutex> g(slot_lock);
            Slot &s = slots[index][field_index(id)];
            v -= s.valid ? s.base : v;
        }
        *value = v;
        return SWOFTY_ASSET_OK;
    }

    // Writes one value into a located field and re-runs what the game derives from it.
    static int write_value(uintptr_t car, uint32_t index, const Field &f, uintptr_t sys, uintptr_t at, float value)
    {
        if ((f.rederive == Rederive::engine && !env.engine_derived) ||
            (f.rederive == Rederive::gearbox && !env.gearbox_reset) ||
            (f.rederive == Rederive::chassis && !env.chassis_reset))
            return SWOFTY_ASSET_UNAVAILABLE;
        uint8_t setup_ratios = 0;
        if (f.rederive == Rederive::gearbox && (!rd(sys + GEARBOX_USE_SETUP_RATIOS, setup_ratios) || setup_ratios))
            return SWOFTY_ASSET_UNAVAILABLE;
        if (f.id == SWOFTY_CAR_FUEL_MASS) {
            float capacity = 0;
            if (load(sys + FUEL_CAPACITY, Kind::obf, capacity) && capacity > 0 && value > capacity)
                return SWOFTY_ASSET_INVALID;
        }
        if (!store(at, f.kind, value))
            return SWOFTY_ASSET_LATE;
        if (f.rederive == Rederive::engine)
            env.engine_derived(sys);
        else if (f.rederive == Rederive::gearbox)
            env.gearbox_reset(sys, GEARBOX_REBUILD);
        else if (f.rederive == Rederive::chassis) {
            if (!store(sys + CHASSIS_SQRT_MASS, Kind::obf, std::sqrt(std::max(value, 0.f))))
                return SWOFTY_ASSET_LATE;
            env.chassis_reset(sys, CHASSIS_REBUILD);
        }
        (void)car;
        (void)index;
        return SWOFTY_ASSET_OK;
    }

    static int op_set(uint32_t index, const void *handle, uint32_t id, float value)
    {
        const Field *f = field(id);
        if (!f || !std::isfinite(value) || value < f->lo || value > f->hi)
            return SWOFTY_ASSET_INVALID;
        if (!ready(*f))
            return SWOFTY_ASSET_UNAVAILABLE;
        if (f->kind == Kind::virt) {
            if (index >= MAX_CARS)
                return SWOFTY_ASSET_INVALID;
            virt[index][f->vslot] = value;
            if (value != f->neutral)
                (f->vslot == 2 ? ai_active : power_active).store(true);
            return SWOFTY_ASSET_OK;
        }
        if (env.flow_thread && !env.flow_thread())
            return SWOFTY_ASSET_LATE;
        uintptr_t car = 0, sys = 0, at = 0;
        if (int r = car_at(index, handle, car); r != SWOFTY_ASSET_OK)
            return r;
        if (int r = locate(car, *f, sys, at); r != SWOFTY_ASSET_OK)
            return r;
        float memory = value;
        if (f->relative) {
            float current = 0;
            if (!load(at, f->kind, current))
                return SWOFTY_ASSET_LATE;
            std::lock_guard<std::mutex> g(slot_lock);
            Slot &s = slots[index][field_index(id)];
            float base = s.valid && bitwise_equal(current, s.written) ? s.base : current;
            memory = base + value;
            if (!(memory > 0))
                return SWOFTY_ASSET_INVALID;
            if (int r = write_value(car, index, *f, sys, at, memory); r != SWOFTY_ASSET_OK)
                return r;
            float written = memory;
            load(at, f->kind, written);
            s = Slot{true, base, written};
            return SWOFTY_ASSET_OK;
        }
        return write_value(car, index, *f, sys, at, memory);
    }

    static bool bitwise_equal(float a, float b)
    {
        uint32_t x, y;
        std::memcpy(&x, &a, 4);
        std::memcpy(&y, &b, 4);
        return x == y;
    }

    // Applies every layer that matches one car now. A memory field's base is the value the game last wrote: the
    // value read is the game's when it differs from what this wrote, so a reset (which restores the game's value)
    // is picked up as the new base, and a layer applied twice does not compound.
    static void apply_car(uint32_t index)
    {
        uintptr_t car = 0;
        if (car_at(index, nullptr, car) != SWOFTY_ASSET_OK)
            return;
        for (size_t i = 0; i < FIELD_COUNT; i++) {
            const Field &f = FIELDS[i];
            if (f.kind == Kind::virt || !ready(f))
                continue;
            bool want = has_layer(index, f.id);
            std::lock_guard<std::mutex> g(slot_lock);
            Slot &s = slots[index][i];
            if (!want && !s.valid)
                continue;
            uintptr_t sys = 0, at = 0;
            float current = 0;
            if (locate(car, f, sys, at) != SWOFTY_ASSET_OK || !load(at, f.kind, current)) {
                s = Slot{};
                continue;
            }
            float base = s.valid && bitwise_equal(current, s.written) ? s.base : current;
            if (!want) {
                if (s.valid && bitwise_equal(current, s.written))
                    write_value(car, index, f, sys, at, base);
                s = Slot{};
                continue;
            }
            float target = f.relative ? base + clamp(f, composed(index, f, 0)) : clamp(f, composed(index, f, base));
            if (!std::isfinite(target) || write_value(car, index, f, sys, at, target) != SWOFTY_ASSET_OK) {
                s = Slot{};
                continue;
            }
            float written = target;
            load(at, f.kind, written);
            s = Slot{true, base, written};
        }
    }

    static void apply_all()
    {
        uint32_t n = 0;
        if (cars(n) != SWOFTY_ASSET_OK)
            return;
        for (uint32_t i = 0; i < n; i++)
            apply_car(i);
    }

    static int op_field(const char *path, uint32_t *id)
    {
        const Field *f = find_field_path(path);
        if (!f || !id)
            return SWOFTY_ASSET_INVALID;
        *id = f->id;
        return SWOFTY_ASSET_OK;
    }

    static int add_layer(const std::string &owner, uint32_t index, uint32_t id, uint32_t op, float value, int32_t priority,
                         uint32_t *token)
    {
        const Field *f = field(id);
        if (!f || op > SWOFTY_CAR_OP_ADD || !std::isfinite(value) || (index != SWOFTY_CAR_ALL && index >= MAX_CARS))
            return SWOFTY_ASSET_INVALID;
        if (!ready(*f))
            return SWOFTY_ASSET_UNAVAILABLE;
        uint32_t t = 0;
        {
            std::lock_guard<std::mutex> g(lay_lock);
            t = next_token++;
            layers.push_back({t, owner, index, id, op, value, priority, next_order++});
        }
        if (token)
            *token = t;
        refresh_virtuals();
        dirty = true;
        return SWOFTY_ASSET_OK;
    }

    static int op_layer(const char *owner, uint32_t index, uint32_t id, uint32_t op, float value, int32_t priority,
                        uint32_t *token)
    {
        int r = add_layer(owner ? owner : "", index, id, op, value, priority, token);
        if (r == SWOFTY_ASSET_OK && (!env.flow_thread || env.flow_thread()))
            flush();
        return r;
    }

    static int op_unlayer(uint32_t token)
    {
        {
            std::lock_guard<std::mutex> g(lay_lock);
            auto it = std::find_if(layers.begin(), layers.end(), [&](const LayerRec &l) { return l.token == token; });
            if (it == layers.end())
                return SWOFTY_ASSET_INVALID;
            layers.erase(it);
        }
        refresh_virtuals();
        dirty = true;
        if (!env.flow_thread || env.flow_thread())
            flush();
        return SWOFTY_ASSET_OK;
    }

    // Removes every layer of one owner for one car (or all, SWOFTY_CAR_ALL): a car.profile re-evaluation.
    static void drop_owner(const std::string &owner, uint32_t index)
    {
        {
            std::lock_guard<std::mutex> g(lay_lock);
            std::erase_if(layers, [&](const LayerRec &l) {
                return l.owner == owner && (index == SWOFTY_CAR_ALL || l.index == index);
            });
        }
        refresh_virtuals();
        dirty = true;
    }

    static void flush()
    {
        if (!dirty.exchange(false))
            return;
        apply_all();
    }

    static int op_refit_tyres(uint32_t index, const char *group)
    {
        if (!group || !*group || std::strlen(group) > 31)
            return SWOFTY_ASSET_INVALID;
        for (const char *c = group; *c; c++)
            if (!((*c >= 'A' && *c <= 'Z') || (*c >= 'a' && *c <= 'z') || (*c >= '0' && *c <= '9') || *c == '_'))
                return SWOFTY_ASSET_INVALID;
        if (!env.tyre_component || !env.tyre_refit || (env.row_ready && !env.row_ready("CarChassis.tyre_physics_group")))
            return SWOFTY_ASSET_UNAVAILABLE;
        if (env.flow_thread && !env.flow_thread())
            return SWOFTY_ASSET_LATE;
        uintptr_t car = 0, chassis = 0;
        if (int r = car_at(index, nullptr, car); r != SWOFTY_ASSET_OK)
            return r;
        if (int r = system_at(car, CHASSIS, chassis); r != SWOFTY_ASSET_OK)
            return r;
        uintptr_t component = env.tyre_component(index, car);
        if (!component)
            return SWOFTY_ASSET_UNAVAILABLE;
        size_t capacity = 0;
        uintptr_t buffer = 0;
        size_t len = std::strlen(group);
        if (!rd(chassis + CHASSIS_GROUP + STRING_CAPACITY, capacity) || !rd(chassis + CHASSIS_GROUP + STRING_BUFFER, buffer) ||
            !buffer || capacity <= len || capacity > 0x1000)
            return SWOFTY_ASSET_UNAVAILABLE;
        if (!env.write || !env.write(buffer, group, len + 1))
            return SWOFTY_ASSET_LATE;
        env.tyre_refit(component);
        return SWOFTY_ASSET_OK;
    }

    // The car of an engine, AI-state or other system pointer (system+8 is its car), for the core's site handlers.
    static bool car_index_of_system(uintptr_t sys, uint32_t &index)
    {
        uintptr_t car = 0;
        uint8_t id = 0;
        if (!rd(sys + SYSTEM_CAR, car) || !car || !rd(car + CAR_VEHICLE_ID, id) || id >= MAX_CARS)
            return false;
        index = id;
        return true;
    }

    static float drive_torque(uintptr_t engine, float torque, float rad_s, float limiter_rad_s)
    {
        uint32_t index = 0;
        if (!power_active.load(std::memory_order_relaxed) || !car_index_of_system(engine, index))
            return torque;
        return shaped_torque(torque, rad_s, limiter_rad_s, virt[index][0].load(std::memory_order_relaxed),
                             virt[index][1].load(std::memory_order_relaxed));
    }

    static float ai_grip(uint32_t index, uint8_t weather, float grip)
    {
        if (!ai_active.load(std::memory_order_relaxed) || index >= MAX_CARS || weather != 0)
            return grip;
        return grip * virt[index][2].load(std::memory_order_relaxed);
    }

    // car.built, car.reset, car.upgrades.replayed and car.setup.applied: the game builds each car on a loader thread
    // (T119) and re-reads its parts through Car_Systems_Reset_Thunk (T120), so the hooks only queue the car and the
    // next frame takes the queue on the flow thread, where layers apply and set() may write, keeping the cars still
    // in their slot.
    enum class What : uint8_t { built, reset, replayed, setup, damage };
    struct Pending {
        uint32_t index;
        uintptr_t car;
        What what;
        float progress;
        uint32_t part;
    };
    static inline std::mutex pending_lock;
    static inline std::vector<Pending> pending;

    static void queue(uintptr_t car, What what, float progress = 0, uint32_t part = 0)
    {
        uint8_t id = 0;
        if (!bound.load() || !car || !rd(car + CAR_VEHICLE_ID, id) || id >= MAX_CARS)
            return;
        std::lock_guard<std::mutex> g(pending_lock);
        if (pending.size() < MAX_CARS * 8)
            pending.push_back({id, car, what, progress, part});
    }

    static void built(uintptr_t car) { queue(car, What::built); }
    static void reset(uintptr_t loader) { queue(loader >= CAR_LOADER ? loader - CAR_LOADER : 0, What::reset); }
    static void replayed(uintptr_t loader, float progress)
    {
        queue(loader >= CAR_LOADER ? loader - CAR_LOADER : 0, What::replayed, progress);
    }
    static void damaged(uintptr_t wing, uint32_t part, float amount)
    {
        uintptr_t car = 0;
        if (amount > 0 && rd(wing + SYSTEM_CAR, car))
            queue(car, What::damage, amount, part);
    }
    static void setup_applied(uintptr_t f1_tuning)
    {
        uintptr_t car = 0;
        if (rd(f1_tuning + SYSTEM_CAR, car))
            queue(car, What::setup);
    }

    static std::vector<Pending> take()
    {
        std::vector<Pending> out;
        {
            std::lock_guard<std::mutex> g(pending_lock);
            out.swap(pending);
        }
        std::erase_if(out, [](const Pending &p) {
            uintptr_t car = 0;
            return car_at(p.index, reinterpret_cast<const void *>(p.car), car) != SWOFTY_ASSET_OK;
        });
        return out;
    }

    static inline const SwoftyCarOps table = {
        SWOFTY_CAR_OPS_VERSION, op_available, op_count, op_car, op_get, op_set, op_field, op_layer, op_unlayer,
        op_refit_tyres,
    };
};

}

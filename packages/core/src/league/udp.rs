//! Race results from the game's own UDP telemetry (Settings > Telemetry in F1 25, stored in
//! `hardware_settings_config.xml`).
//!
//! The game sends its packets to one address; the launcher listens there and keeps a few of them
//! per session: the session packet (track and session type), the participants (names, race
//! numbers, which cars are AI) and the final classification at the end of a race or qualifying.
//! Together they make a [`RaceReport`]. A practice session is reported from each car's laps (the
//! session history packets) when it ends (the `SEND` event, its classification, or the next
//! session starting), with the player's own laps for the practice programmes. The layout is EA's published UDP specification; packet formats 2024
//! and later share the parts read here, and the per-car record sizes are taken from the packet
//! length, so a format that grew a field at the end of a record still reads. Format 2026 (the
//! game's 2026 season update) sends 24 cars instead of 22 (its lap data is 29 + 24 × 57 + 2
//! bytes, `Game_TelemetryUdp_Send2026` in `re/db`), and its participant records grew three bytes
//! before the name (checked against packets the game sent, `tests/udp-2026/`).
//!
//! During a session the collector also follows the player's car lap by lap (lap data, car status
//! and car damage packets): the fuel, the ERS energy and the tyre wear each lap used, for the
//! practice programmes ([`LapTelemetry`]).

use std::collections::{HashMap, HashSet};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use super::{GameSettings, Lap, LapTelemetry, RaceReport, ReportCar, SessionKind};

pub const HEADER_LEN: usize = 29;
/// Cars per packet in formats 2024 and 2025.
pub const MAX_CARS: usize = 22;
/// Cars per packet from format 2026 on.
pub const MAX_CARS_2026: usize = 24;
pub const PACKET_SESSION: u8 = 1;
pub const PACKET_LAP_DATA: u8 = 2;
pub const PACKET_EVENT: u8 = 3;
pub const PACKET_PARTICIPANTS: u8 = 4;
pub const PACKET_CAR_STATUS: u8 = 7;
pub const PACKET_FINAL_CLASSIFICATION: u8 = 8;
pub const PACKET_CAR_DAMAGE: u8 = 10;
pub const PACKET_SESSION_HISTORY: u8 = 11;
/// Bytes a lap data packet sends after its car records: the time trial's personal-best and
/// rival car indices.
const LAP_DATA_TAIL: usize = 2;
/// Laps of telemetry kept per session (a lap number is one byte).
const TELEMETRY_LAPS: usize = 255;
/// Laps a session history packet holds.
const HISTORY_LAPS: usize = 100;
/// Tyre stints a session history packet holds after the laps, three bytes each.
const HISTORY_STINTS: usize = 8 * 3;
/// Session types of a race (race, race 2, race 3); a sprint weekend's sprint is a race too.
pub const RACE_SESSIONS: std::ops::RangeInclusive<u8> = 15..=17;
/// The game's default telemetry port.
pub const DEFAULT_PORT: u16 = 20777;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub format: u16,
    pub packet_id: u8,
    pub session: u64,
    pub player_car: u8,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

fn f64_at(b: &[u8], at: usize) -> Option<f64> {
    Some(f64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

/// A finite `f32` (the game's floats; anything else reads as missing).
fn f32_at(b: &[u8], at: usize) -> Option<f32> {
    Some(f32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?)).filter(|x| x.is_finite())
}

pub fn header(b: &[u8]) -> Option<Header> {
    if b.len() < HEADER_LEN {
        return None;
    }
    let format = u16_at(b, 0)?;
    if !(2024..2100).contains(&format) {
        return None;
    }
    Some(Header {
        format,
        packet_id: b[6],
        session: u64_at(b, 7)?,
        player_car: b[27],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionInfo {
    pub track: Option<u8>,
    pub session_type: u8,
    pub total_laps: u8,
    /// The weekend's session types (`weekend_len` of them), see [`SessionInfo::weekend`].
    pub weekend_types: [u8; WEEKEND_MAX],
    pub weekend_len: u8,
    /// The game's settings, when the packet is long enough to hold them.
    pub settings: Option<GameSettings>,
}

impl SessionInfo {
    /// The weekend structure the game sent: its session types in order. Empty when the game sent
    /// none (a Grand Prix may send a count of 0).
    pub fn weekend(&self) -> Vec<u8> {
        self.weekend_types[..usize::from(self.weekend_len)].to_vec()
    }
}

/// Session types a weekend structure holds.
pub const WEEKEND_MAX: usize = 12;
/// Where the session body (after the header) keeps the weekend structure: a count, then
/// [`WEEKEND_MAX`] session types (the F1 24 layout, which F1 25 keeps).
const WEEKEND_AT: usize = 703;

pub fn session(b: &[u8]) -> Option<SessionInfo> {
    let at = HEADER_LEN;
    let total_laps = *b.get(at + 3)?;
    let session_type = *b.get(at + 6)?;
    let track = *b.get(at + 7)? as i8;
    let mut weekend_types = [0u8; WEEKEND_MAX];
    let mut weekend_len = 0;
    if let Some(&count) = b.get(at + WEEKEND_AT)
        && let Some(types) = b.get(at + WEEKEND_AT + 1..at + WEEKEND_AT + 1 + WEEKEND_MAX)
        && (1..=WEEKEND_MAX as u8).contains(&count)
        && types[..usize::from(count)].iter().all(|&t| (1..=18).contains(&t))
    {
        weekend_types.copy_from_slice(types);
        weekend_len = count;
    }
    // The settings bytes, at their F1 24 body offsets (0 can mean the game didn't send one).
    let byte = |off: usize| b.get(at + off).copied();
    let settings = (|| {
        Some(GameSettings {
            weather: byte(0)?,
            network_game: byte(125)?,
            ai_difficulty: byte(640)?,
            game_mode: byte(665)?,
            rule_set: byte(666)?,
            session_length: byte(671)?,
            equal_performance: byte(679)?,
            car_damage: byte(687)?,
            parc_ferme: byte(694)?,
            safety_car: byte(696)?,
            formation_lap: byte(698)?,
            red_flags: byte(700)?,
        })
    })();
    Some(SessionInfo {
        track: u8::try_from(track).ok(),
        session_type,
        total_laps,
        weekend_types,
        weekend_len,
        settings,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participant {
    pub ai: bool,
    pub team: u8,
    pub number: u8,
    pub name: String,
}

/// The number of per-car records and their size, from the packet's format and length after a
/// one-byte count.
fn records(b: &[u8]) -> Option<(usize, usize)> {
    let body = b.len().checked_sub(HEADER_LEN + 1)?;
    let first = if header(b).is_some_and(|h| h.format >= 2026) {
        [MAX_CARS_2026, MAX_CARS]
    } else {
        [MAX_CARS, MAX_CARS_2026]
    };
    first
        .into_iter()
        .find(|&cars| body % cars == 0)
        .map(|cars| (cars, body / cars))
}

/// Where a participant record keeps the team, the race number and the name, and the name's size.
#[derive(Clone, Copy)]
struct ParticipantLayout {
    team: usize,
    number: usize,
    name: usize,
    name_len: usize,
}

pub fn participants(b: &[u8]) -> Option<Vec<Participant>> {
    let (cars, stride) = records(b)?;
    let format = header(b)?.format;
    let layout = if format >= 2026 {
        // 2026 (seen from the game, 60-byte records): three more bytes before the name; the team
        // at 5, the race number at 8, a 32-byte name at 10.
        ParticipantLayout {
            team: 5,
            number: 8,
            name: 10,
            name_len: 32,
        }
    } else {
        // 2025: 32-byte names (57-byte records); 2024: 48-byte names (60-byte records).
        ParticipantLayout {
            team: 3,
            number: 5,
            name: 7,
            name_len: if stride >= 60 { 48 } else { 32 },
        }
    };
    if stride < layout.name + layout.name_len {
        return None;
    }
    // Every record, not the packet's car count: the game counted 21 active cars in a 22-car
    // session, leaving out the player's car at index 21. Empty records at the end are dropped.
    let mut list: Vec<Participant> = (0..cars)
        .map(|i| {
            let r = &b[HEADER_LEN + 1 + i * stride..][..stride];
            let raw = &r[layout.name..layout.name + layout.name_len];
            let end = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
            Participant {
                ai: r[0] != 0,
                team: r[layout.team],
                number: r[layout.number],
                name: name_text(&raw[..end]),
            }
        })
        .collect();
    let used = (b[HEADER_LEN] as usize).min(cars);
    while list.len() > used && list.last().is_some_and(|p| p.name.is_empty() && p.number == 0) {
        list.pop();
    }
    Some(list)
}

/// A name as the game sends it: UTF-8 by the specification, but the game sends accented names in
/// Latin-1 (`H\xdcLKENBERG`).
fn name_text(raw: &[u8]) -> String {
    match std::str::from_utf8(raw) {
        Ok(s) => s.trim().to_owned(),
        Err(_) => raw
            .iter()
            .map(|&c| char::from(c))
            .collect::<String>()
            .trim()
            .to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinalCar {
    pub position: u8,
    pub laps: u8,
    pub grid: u8,
    pub points: u8,
    pub status: u8,
    pub best_lap_ms: u32,
    pub time_s: f64,
    pub penalty_s: u8,
}

pub fn final_classification(b: &[u8]) -> Option<Vec<FinalCar>> {
    let (cars, stride) = records(b)?;
    // 2025 added a result-reason byte after the status (46-byte records; 45 in 2024).
    let shift = usize::from(stride >= 46);
    if stride < 19 + shift {
        return None;
    }
    let count = (b[HEADER_LEN] as usize).min(cars);
    (0..count)
        .map(|i| {
            let r = &b[HEADER_LEN + 1 + i * stride..][..stride];
            Some(FinalCar {
                position: r[0],
                laps: r[1],
                grid: r[2],
                points: r[3],
                status: r[5],
                best_lap_ms: u32_at(r, 6 + shift)?,
                time_s: f64_at(r, 10 + shift)?,
                penalty_s: r[18 + shift],
            })
        })
        .collect()
}

/// One car's laps from a session history packet: the car's index and its laps so far (time and
/// whether it was within track limits). Each lap record starts with its time and ends with its
/// validity flags (bit 0: valid); the record size comes from the packet length.
pub fn session_history(b: &[u8]) -> Option<(u8, Vec<Lap>)> {
    let body = b.len().checked_sub(HEADER_LEN + 7 + HISTORY_STINTS)?;
    if body % HISTORY_LAPS != 0 {
        return None;
    }
    let stride = body / HISTORY_LAPS;
    if stride < 5 {
        return None;
    }
    let car = *b.get(HEADER_LEN)?;
    let laps = (*b.get(HEADER_LEN + 1)? as usize).min(HISTORY_LAPS);
    let first = HEADER_LEN + 7;
    let list = (0..laps)
        .filter_map(|i| {
            let r = b.get(first + i * stride..first + (i + 1) * stride)?;
            Some(Lap {
                ms: u32_at(r, 0)?,
                valid: r[stride - 1] & 1 != 0,
            })
        })
        // The lap being driven has no time yet.
        .filter(|l| l.ms > 0)
        .collect();
    Some((car, list))
}

/// The player's record of a packet whose car records follow the header directly (lap data, car
/// status, car damage), with `tail` bytes after them: 22 or 24 records, their size from the
/// packet length, at least `min` bytes each. `None` for a spectator (player car 255).
fn player_record(b: &[u8], tail: usize, min: usize) -> Option<&[u8]> {
    let h = header(b)?;
    let body = b.len().checked_sub(HEADER_LEN + tail)?;
    let first = if h.format >= 2026 {
        [MAX_CARS_2026, MAX_CARS]
    } else {
        [MAX_CARS, MAX_CARS_2026]
    };
    let (cars, stride) = first
        .into_iter()
        .find(|&cars| body % cars == 0 && body / cars >= min)
        .map(|cars| (cars, body / cars))?;
    let car = usize::from(h.player_car);
    if car >= cars {
        return None;
    }
    b.get(HEADER_LEN + car * stride..HEADER_LEN + (car + 1) * stride)
}

/// The player's car in a lap data packet (packet 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LapState {
    /// The lap being driven (`m_currentLapNum`), from 1.
    pub lap: u8,
    /// The last completed lap's time (`m_lastLapTimeInMS`).
    pub last_lap_ms: u32,
    /// The lap being driven left the track limits (`m_currentLapInvalid`).
    pub invalid: bool,
    /// In the pit lane or garage, or on an out or in lap (`m_pitStatus`, `m_driverStatus`).
    pub pit: bool,
}

/// The player's car in a lap data packet. Records of 57 bytes in formats 2024 to 2026 (22 or 24
/// of them, then two bytes): the last lap's time at 0, the lap number at 33, the pit status at
/// 34, the lap's invalid flag at 37 and the driver status at 44.
pub fn lap_data(b: &[u8]) -> Option<LapState> {
    let r = player_record(b, LAP_DATA_TAIL, 45)?;
    // Driver status: 0 garage, 1 flying lap, 2 in lap, 3 out lap, 4 on track.
    let driver = r[44];
    Some(LapState {
        lap: r[33],
        last_lap_ms: u32_at(r, 0)?,
        invalid: r[37] != 0,
        pit: r[34] != 0 || matches!(driver, 0 | 2 | 3),
    })
}

/// The player's car in a car status packet (packet 7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarStatus {
    /// `m_fuelInTank`, kg.
    pub fuel_kg: f32,
    /// `m_ersStoreEnergy`, joules.
    pub ers_store_j: f32,
    /// `m_ersDeployedThisLap`, joules.
    pub ers_deployed_j: f32,
}

/// The player's car in a car status packet. Records of 55 bytes (formats 2024 and 2025): the fuel
/// in the tank at 5, the ERS store at 37, the ERS deployed this lap at 50.
pub fn car_status(b: &[u8]) -> Option<CarStatus> {
    let r = player_record(b, 0, 54)?;
    Some(CarStatus {
        fuel_kg: f32_at(r, 5)?,
        ers_store_j: f32_at(r, 37)?,
        ers_deployed_j: f32_at(r, 50)?,
    })
}

/// The player's tyre wear in percent (`m_tyresWear`: rear left, rear right, front left, front
/// right), from a car damage packet (packet 10; records of 46 bytes in 2025, 42 in 2024, the wear
/// first).
pub fn car_damage(b: &[u8]) -> Option<[f32; 4]> {
    let r = player_record(b, 0, 16)?;
    Some([f32_at(r, 0)?, f32_at(r, 4)?, f32_at(r, 8)?, f32_at(r, 12)?])
}

/// Follows the player's car lap by lap: at each lap completed, what it used.
#[derive(Debug, Default, Clone)]
struct LapTracker {
    /// The lap being driven, 0 before the first lap data.
    lap: u8,
    /// The lap being driven was seen from its start, with the car's state there.
    whole: bool,
    invalid: bool,
    pit: bool,
    fuel_start: f32,
    wear_start: Option<f32>,
    /// The most ERS energy the lap's counter showed.
    ers_deployed: f32,
    /// The latest fuel, ERS store and mean tyre wear.
    fuel: Option<f32>,
    ers_store: Option<f32>,
    wear: Option<f32>,
    laps: Vec<LapTelemetry>,
}

impl LapTracker {
    fn status(&mut self, s: CarStatus) {
        self.fuel = Some(s.fuel_kg);
        self.ers_store = Some(s.ers_store_j);
        self.ers_deployed = self.ers_deployed.max(s.ers_deployed_j);
    }

    fn damage(&mut self, wear: [f32; 4]) {
        self.wear = Some(wear.iter().sum::<f32>() / 4.0);
    }

    fn lap_data(&mut self, s: LapState) {
        if s.lap == self.lap {
            self.invalid |= s.invalid;
            self.pit |= s.pit;
            return;
        }
        let crossed = self.lap > 0 && s.lap == self.lap + 1;
        if crossed
            && self.whole
            && let Some(fuel) = self.fuel
            && self.laps.len() < TELEMETRY_LAPS
        {
            let wear = match (self.wear_start, self.wear) {
                (Some(start), Some(end)) => (end - start).max(0.0),
                _ => 0.0,
            };
            self.laps.push(LapTelemetry {
                lap: self.lap,
                ms: s.last_lap_ms,
                fuel_used_kg: (self.fuel_start - fuel).max(0.0),
                ers_deployed_mj: self.ers_deployed / 1e6,
                ers_store_end_mj: self.ers_store.unwrap_or(0.0) / 1e6,
                tyre_wear_pct: wear,
                valid: !self.invalid && !self.pit,
            });
        }
        if s.lap < self.lap {
            // A flashback (or a restart) across the line: the laps from there on are driven again.
            self.laps.retain(|l| l.lap < s.lap);
        }
        self.lap = s.lap;
        self.whole = crossed && self.fuel.is_some();
        self.fuel_start = self.fuel.unwrap_or(0.0);
        self.wear_start = self.wear;
        self.ers_deployed = 0.0;
        self.invalid = s.invalid;
        self.pit = s.pit;
    }
}

/// An event packet's four-letter code (`SEND` when a session ends).
pub fn event_code(b: &[u8]) -> Option<[u8; 4]> {
    b.get(HEADER_LEN..HEADER_LEN + 4)?.try_into().ok()
}

#[derive(Default, Clone)]
struct SessionState {
    info: Option<SessionInfo>,
    participants: Vec<Participant>,
    /// Each car's laps (practice sessions), by car index.
    history: HashMap<u8, Vec<Lap>>,
    player_car: u8,
    /// The player's laps with what the car used on them.
    telemetry: LapTracker,
}

/// A stand-in session uid for a classification the game sent without one (a race it simulated
/// is classified with session uid 0): a hash of the results, so the copies the results screen
/// keeps sending count once and two simulated races don't collide.
fn result_uid(cars: &[FinalCar]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for c in cars {
        for x in [c.position, c.laps, c.grid, c.status]
            .into_iter()
            .chain(c.best_lap_ms.to_le_bytes())
            .chain(c.time_s.to_le_bytes())
        {
            h = (h ^ u64::from(x)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    h | 1 << 63
}

/// Whether a classification is a race's: points were given or more than a lap was driven
/// (a qualifying classification has neither).
fn looks_like_race(cars: &[FinalCar]) -> bool {
    cars.iter().any(|c| c.points > 0 || c.laps > 1)
}

fn is_practice(info: Option<SessionInfo>) -> bool {
    info.is_some_and(|i| super::session_kind(i.session_type) == Some(SessionKind::Practice))
}

/// The cars of a classification with their participants.
fn classified_cars(cars: &[FinalCar], state: &SessionState, player_car: u8) -> Vec<ReportCar> {
    cars.iter()
        .enumerate()
        .filter(|(_, c)| c.position > 0)
        .map(|(i, c)| {
            let p = state.participants.get(i);
            ReportCar {
                name: p.map(|p| p.name.clone()).unwrap_or_default(),
                number: p.map_or(0, |p| p.number),
                team: p.map_or(255, |p| p.team),
                ai: p.is_none_or(|p| p.ai),
                mine: i == player_car as usize,
                position: c.position,
                grid: c.grid,
                laps: c.laps,
                status: c.status,
                best_lap_ms: c.best_lap_ms,
                time_ms: (c.time_s.max(0.0) * 1000.0) as u64,
                penalty_s: u16::from(c.penalty_s),
            }
        })
        .collect()
}

/// The cars of a practice session from their laps: ordered by best clean lap, the cars that drove
/// none left out.
fn practice_cars(state: &SessionState) -> Vec<ReportCar> {
    let mut cars: Vec<(usize, u32, u8)> = state
        .participants
        .iter()
        .enumerate()
        .filter_map(|(i, _)| {
            let laps = state.history.get(&(i as u8))?;
            let best = laps.iter().filter(|l| l.valid).map(|l| l.ms).min().unwrap_or(0);
            (!laps.is_empty()).then_some((i, best, laps.len().min(255) as u8))
        })
        .collect();
    cars.sort_by_key(|&(i, best, _)| (best == 0, best, i));
    cars.iter()
        .enumerate()
        .map(|(pos, &(i, best, laps))| {
            let p = &state.participants[i];
            ReportCar {
                name: p.name.clone(),
                number: p.number,
                team: p.team,
                ai: p.ai,
                mine: i == state.player_car as usize,
                position: pos as u8 + 1,
                grid: 0,
                laps,
                status: 3,
                best_lap_ms: best,
                time_ms: 0,
                penalty_s: 0,
            }
        })
        .collect()
}

/// A report's fields that come from the session packet: the race distance, the weekend
/// structure and the game's settings.
fn report_of_session(info: &SessionInfo) -> RaceReport {
    RaceReport {
        total_laps: info.total_laps,
        weekend: info.weekend(),
        settings: info.settings,
        ..RaceReport::default()
    }
}

/// Turns the game's packets into a [`RaceReport`] at the end of each race, qualifying and
/// practice session.
#[derive(Default)]
pub struct Collector {
    sessions: HashMap<u64, SessionState>,
    /// The session the last participants packet belonged to.
    last: Option<u64>,
    /// The session the last session packet belonged to.
    current: Option<u64>,
    reported: HashSet<u64>,
    /// Packets seen, by id, for diagnostics.
    pub seen: HashMap<u8, u64>,
    /// The last session, participants or classification packet that didn't read, as (id, size).
    pub unread: Option<(u8, usize)>,
}

impl Collector {
    pub fn new() -> Collector {
        Collector::default()
    }

    /// The report of practice session `uid` from the laps driven (and its classification, when the
    /// game sent one), unless it's reported already or the player drove no lap.
    fn practice(
        &mut self,
        uid: u64,
        classification: Option<&[FinalCar]>,
        now: u64,
        report_as: u64,
    ) -> Option<RaceReport> {
        if self.reported.contains(&report_as) {
            return None;
        }
        let state = self.sessions.get(&uid)?.clone();
        let info = state.info?;
        let cars = match classification {
            Some(c) => classified_cars(c, &state, state.player_car),
            None => practice_cars(&state),
        };
        let laps = state.history.get(&state.player_car).cloned().unwrap_or_default();
        let lap_telemetry = state.telemetry.laps;
        if laps.is_empty()
            && lap_telemetry.is_empty()
            && !cars.iter().any(|c| c.mine && (c.laps > 0 || c.best_lap_ms > 0))
        {
            return None;
        }
        self.reported.insert(report_as);
        Some(RaceReport {
            session: report_as,
            track: info.track,
            session_type: info.session_type,
            cars,
            at: now,
            laps,
            lap_telemetry,
            ..report_of_session(&info)
        })
    }

    /// Feed one datagram; returns the session's report when this ended a race, qualifying or
    /// practice session.
    pub fn feed(&mut self, b: &[u8], now: u64) -> Option<RaceReport> {
        let h = header(b)?;
        *self.seen.entry(h.packet_id).or_default() += 1;
        let unread = Some((h.packet_id, b.len()));
        match h.packet_id {
            PACKET_SESSION => {
                let Some(info) = session(b) else {
                    self.unread = unread;
                    return None;
                };
                let state = self.sessions.entry(h.session).or_default();
                state.info = Some(info);
                state.player_car = h.player_car;
                // A practice session left for the next one ends with what was driven.
                let left = self.current.filter(|&c| c != h.session && h.session != 0);
                if h.session != 0 {
                    self.current = Some(h.session);
                }
                let left = left.filter(|c| is_practice(self.sessions.get(c).and_then(|s| s.info)))?;
                self.practice(left, None, now, left)
            }
            PACKET_SESSION_HISTORY => {
                let Some((car, laps)) = session_history(b) else {
                    self.unread = unread;
                    return None;
                };
                let state = self.sessions.entry(h.session).or_default();
                state.player_car = h.player_car;
                state.history.insert(car, laps);
                None
            }
            // The player's car lap by lap, in a session of its own (the practice programmes).
            PACKET_LAP_DATA | PACKET_CAR_STATUS | PACKET_CAR_DAMAGE => {
                if h.session == 0 || self.reported.contains(&h.session) {
                    return None;
                }
                let tracker = &mut self.sessions.entry(h.session).or_default().telemetry;
                let read = match h.packet_id {
                    PACKET_LAP_DATA => lap_data(b).map(|l| tracker.lap_data(l)),
                    PACKET_CAR_STATUS => car_status(b).map(|c| tracker.status(c)),
                    _ => car_damage(b).map(|w| tracker.damage(w)),
                };
                // A spectator's packets (player car 255) have no car of the player's.
                if read.is_none() && h.player_car != 255 {
                    self.unread = unread;
                }
                None
            }
            PACKET_EVENT => {
                if event_code(b)? != *b"SEND" {
                    return None;
                }
                let info = self.sessions.get(&h.session)?.info;
                if !is_practice(info) {
                    return None;
                }
                self.practice(h.session, None, now, h.session)
            }
            PACKET_PARTICIPANTS => {
                let Some(list) = participants(b) else {
                    self.unread = unread;
                    return None;
                };
                self.sessions.entry(h.session).or_default().participants = list;
                if h.session != 0 {
                    self.last = Some(h.session);
                }
                None
            }
            PACKET_FINAL_CLASSIFICATION => {
                if self.reported.contains(&h.session) {
                    return None;
                }
                let Some(cars) = final_classification(b) else {
                    self.unread = unread;
                    return None;
                };
                let known = self
                    .sessions
                    .get(&h.session)
                    .filter(|s| h.session != 0 && s.info.is_some());
                let (uid, state, info) = match known {
                    Some(state) => {
                        let info = state.info?;
                        // Races, qualifying and practice count; sprint shootouts and time trial
                        // don't.
                        if super::session_kind(info.session_type)? == SessionKind::Practice {
                            return self.practice(h.session, Some(&cars), now, h.session);
                        }
                        (h.session, state.clone(), info)
                    }
                    // A session the game simulated comes without a session of its own: it is
                    // the weekend of the last session seen, with that session's cars. After a
                    // practice session, a classification without points is that practice's.
                    None => {
                        let uid = result_uid(&cars);
                        if self.reported.contains(&uid) {
                            return None;
                        }
                        let last = self.last?;
                        let state = self.sessions.get(&last)?.clone();
                        let points = cars.iter().any(|c| c.points > 0);
                        if !points && is_practice(state.info) {
                            return self.practice(last, Some(&cars), now, uid);
                        }
                        if !looks_like_race(&cars) {
                            return None;
                        }
                        // A sprint weekend's race after its sprint is race 2.
                        let seen = state.info?;
                        let weekend = seen.weekend();
                        let session_type = if weekend.contains(&16) && seen.session_type == 15 {
                            16
                        } else {
                            *RACE_SESSIONS.start()
                        };
                        let info = SessionInfo { session_type, ..seen };
                        (uid, state, info)
                    }
                };
                self.reported.insert(uid);
                Some(RaceReport {
                    session: uid,
                    track: info.track,
                    session_type: info.session_type,
                    cars: classified_cars(&cars, &state, h.player_car),
                    at: now,
                    laps: Vec::new(),
                    ..report_of_session(&info)
                })
            }
            _ => None,
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Where the launcher keeps the races the game reported (`<data>/multiplayer/races`).
pub const RACES_DIR: &str = "multiplayer/races";
/// Races kept, the newest included.
const RACES_KEPT: usize = 40;

/// Listens for the game's telemetry and keeps every race report as `<dir>/<session>.json`, so a
/// race is never lost while nobody looks at the career page.
pub struct Listener {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    status: Arc<Mutex<ListenStatus>>,
}

#[derive(Debug, Clone, Default)]
pub struct ListenStatus {
    pub datagrams: u64,
    pub races: u64,
    /// The newest race reported while listening.
    pub last: Option<RaceReport>,
    /// A packet this launcher couldn't read, as (packet id, size): a telemetry format it doesn't
    /// know.
    pub unread: Option<(u8, usize)>,
}

impl Listener {
    pub fn start(addr: SocketAddr, dir: PathBuf) -> io::Result<Listener> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(400)))?;
        std::fs::create_dir_all(&dir)?;
        let stop = Arc::new(AtomicBool::new(false));
        let status = Arc::new(Mutex::new(ListenStatus::default()));
        let thread = {
            let (stop, status) = (stop.clone(), status.clone());
            std::thread::Builder::new()
                .name("race-telemetry".into())
                .spawn(move || {
                    let mut collector = Collector::new();
                    let mut buf = [0u8; 2048];
                    while !stop.load(Ordering::Relaxed) {
                        let Ok(n) = socket.recv(&mut buf) else { continue };
                        let report = collector.feed(&buf[..n], now_secs());
                        let mut s = status.lock().unwrap_or_else(|e| e.into_inner());
                        s.datagrams += 1;
                        s.unread = collector.unread;
                        if let Some(report) = report {
                            let _ = save(&dir, &report);
                            s.races += 1;
                            s.last = Some(report);
                        }
                    }
                })?
        };
        Ok(Listener {
            addr,
            stop,
            thread: Some(thread),
            status,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn status(&self) -> ListenStatus {
        self.status.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

pub fn save(dir: &Path, report: &RaceReport) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{:016x}.json", report.session));
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(report).unwrap_or_default())?;
    std::fs::rename(&tmp, &path)?;
    let mut all = saved(dir);
    while all.len() > RACES_KEPT {
        let old = all.remove(0);
        let _ = std::fs::remove_file(dir.join(format!("{:016x}.json", old.session)));
    }
    Ok(())
}

/// Forget a kept race.
pub fn discard(dir: &Path, session: u64) -> io::Result<()> {
    std::fs::remove_file(dir.join(format!("{session:016x}.json")))
}

/// The races kept in `dir`, oldest first.
pub fn saved(dir: &Path) -> Vec<RaceReport> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut all: Vec<RaceReport> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| serde_json::from_slice(&std::fs::read(e.path()).ok()?).ok())
        .collect();
    all.sort_by_key(|r| (r.at, r.session));
    all
}

// ---------------------------------------------------------------------------------------------
// The game's telemetry setting

/// The UDP telemetry setting in `hardware_settings_config.xml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub enabled: bool,
    pub ip: String,
    pub port: u16,
    pub format: String,
}

impl Settings {
    /// The address the launcher listens on to receive what the game sends.
    pub fn listen_addr(&self) -> SocketAddr {
        let ip = if self.ip == "127.0.0.1" || self.ip.is_empty() {
            [127, 0, 0, 1]
        } else {
            [0, 0, 0, 0]
        };
        SocketAddr::from((ip, self.port))
    }

    /// Whether the game sends packets this launcher reads.
    pub fn usable(&self) -> bool {
        self.enabled && self.format.parse::<u16>().is_ok_and(|f| f >= 2024)
    }
}

/// `Documents\My Games\F1 25\hardwaresettings\hardware_settings_config.xml`.
pub fn settings_path() -> Option<PathBuf> {
    let documents = std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join("Documents"))?;
    Some(
        documents
            .join("My Games")
            .join("F1 25")
            .join("hardwaresettings")
            .join("hardware_settings_config.xml"),
    )
}

fn udp_element(xml: &str) -> Option<(usize, usize)> {
    let start = xml.find("<udp ")?;
    let end = start + xml[start..].find("/>")? + 2;
    Some((start, end))
}

fn attribute<'a>(element: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let at = element.find(&key)? + key.len();
    let len = element[at..].find('"')?;
    Some(&element[at..at + len])
}

fn set_attribute(element: &str, name: &str, value: &str) -> String {
    let key = format!(" {name}=\"");
    match element.find(&key) {
        Some(at) => {
            let start = at + key.len();
            let len = element[start..].find('"').unwrap_or(0);
            format!("{}{value}{}", &element[..start], &element[start + len..])
        }
        None => element
            .replacen("/>", &format!("{key}{value}\" />"), 1)
            .replace("\"  />", "\" />"),
    }
}

pub fn parse_settings(xml: &str) -> Option<Settings> {
    let (start, end) = udp_element(xml)?;
    let e = &xml[start..end];
    Some(Settings {
        enabled: attribute(e, "enabled") == Some("true"),
        ip: attribute(e, "ip").unwrap_or("127.0.0.1").to_owned(),
        port: attribute(e, "port")
            .and_then(|p| p.parse().ok())
            .unwrap_or(DEFAULT_PORT),
        format: attribute(e, "format").unwrap_or("").to_owned(),
    })
}

pub fn read_settings(path: &Path) -> Option<Settings> {
    parse_settings(&std::fs::read_to_string(path).ok()?)
}

/// `xml` with UDP telemetry turned on: the address and port stay, broadcast stays as it was, the
/// format becomes 2025 unless it's already 2024 or later.
pub fn enable_in(xml: &str) -> Option<String> {
    let (start, end) = udp_element(xml)?;
    let mut e = xml[start..end].to_owned();
    e = set_attribute(&e, "enabled", "true");
    if attribute(&e, "port").is_none() {
        e = set_attribute(&e, "port", &DEFAULT_PORT.to_string());
    }
    if attribute(&e, "ip").is_none_or(str::is_empty) {
        e = set_attribute(&e, "ip", "127.0.0.1");
    }
    if attribute(&e, "format")
        .and_then(|f| f.parse::<u16>().ok())
        .is_none_or(|f| f < 2024)
    {
        e = set_attribute(&e, "format", "2025");
    }
    Some(format!("{}{e}{}", &xml[..start], &xml[end..]))
}

/// Turn the game's UDP telemetry on (the game must be closed: it writes the file when it quits).
pub fn enable(path: &Path) -> io::Result<Settings> {
    let xml = std::fs::read_to_string(path)?;
    let changed = enable_in(&xml).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "the game's settings file has no telemetry entry",
        )
    })?;
    if changed != xml {
        std::fs::copy(path, path.with_extension("xml.f1mp-backup"))?;
        std::fs::write(path, &changed)?;
    }
    parse_settings(&changed).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unreadable settings"))
}

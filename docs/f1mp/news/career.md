---
title: Your own career, 2016 to 2026
date: 2026-11-01
description: The new Career page runs a whole career, alone or with your friends. Qualifying and races count from the game by themselves, with poles, team goals, a rival, your team-mate's duel and contracts that move you between teams each winter.
---

# Your own career, 2016 to 2026

The **Career** page in the sidebar runs a career that belongs to you, not to the game. Drive it **alone**, kept on your PC, or **with friends** on the host's server, so everyone always sees the same standings. It needs nothing from EA.

## How a career works

1. **Start it.** On the Career page, choose **Alone** or **With friends** (for that you need to be on a host server, see the **Multiplayer** page), name the career and choose the years (2016 to 2026 by default), the race distance and the rules. Friends join with the career's code, like `K-7D19D0`.
2. **Pick your seat.** You take a real driver's seat for the first season, for example Kimi Räikkönen's at Ferrari in 2016. Friends can drive for the same team. With **Rookie start**, you begin in the slower half of the grid.
3. **Drive the rounds.** The page shows the next round. Alone, open **F1 World > Grand Prix** in the game, choose that year's season box, the round's track and your team, and pick the driver whose seat you have. With friends, open a LAN lobby. Qualifying and the race count by themselves when they end, also when you let the game simulate a session.
4. **Sign for next season.** When a season ends, you get contract offers of one to three years. They depend on how you did against your car and your team-mate. Under contract, you stay with your team unless a faster team buys you out. The next season starts with that year's real grid, with you in your new seat.

## Like the game's career

- **Qualifying counts:** poles in the standings, and simulated rounds have a qualifying of their own, so the grid matters.
- **Team goals:** each race, your team expects a finish (P2 or better at the fastest team, further back in slower cars), and the news says whether you made it.
- **Your team-mate and a rival:** each season, a rival from another team with a car and rating like yours. *Your career* shows both duels, qualifying and race, all season long.
- **Your record:** starts, wins, podiums, poles, fastest laps, points, titles and every season, under *Your career*.
- **News:** first points, first podium, first win and first pole, every result, the champions and the contracts.

Rounds at circuits F1 25 doesn't have, like Hockenheim, Sepang or Madrid, are simulated (or left out, if you prefer). You can also simulate or leave out any round yourself, and take back the last one.

## Results from the game

The launcher reads qualifying and race results from F1 25's **UDP telemetry**, in the formats 2024, 2025 and 2026. If it's off, the Career page turns it on for you (with the game closed), or switch it on in the game under *Settings > Telemetry*. The page lists any session the game reported that wasn't counted, so nothing gets lost: count it, count it as the sprint, or discard it.

## From the command line

Everything on the page also works with `f1mp league` (or `f1mp mp career`): `new` (add `--solo` to drive alone), `join`, `seat`, `begin`, `show`, `listen`, `report`, `simulate`, `skip`, `undo`, `sign`, `next-season` and `delete`.

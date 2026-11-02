---
name: cotw-save-knowledge
description: Everything known about theHunter Call of the Wild save files and game archives (formats, hashes, offsets, safe edits) to build or debug tools that read/edit COTW saves on Windows.
---

# theHunter: Call of the Wild — save & game file knowledge

Use this whenever you work on a tool that reads or edits COTW data (the Windows companion app, scripts, artifacts). Everything here was verified on a real level-60 PC save (Steam, game build of late 2026) unless marked **unverified**.

## 0. Golden rules (non-negotiable)

1. **Backup before any write.** Copy the whole save folder to a timestamped folder first. Keep every backup; never delete one automatically.
2. **Game must be closed** before writing (check the `theHunterCotW_F` process). The game rewrites saves on exit and will overwrite or corrupt edits.
3. **Single-player data only.** Never touch multiplayer/online state, never automate the game client.
4. **Map before modifying.** Read and show the user what exists, propose the change, apply only after they decide. Prefer re-adjusting/moving over deleting; never delete user items by default.
5. **Same-size edits are the safe class.** Changing values inside existing records (animal bytes, skill slots, zone fields) is safe. Anything that grows a file must rebuild offsets **and the ADF relocation chain** (§2.3) — the old "+20% population" edit crashed the game because the chain was not rebuilt.
6. **Never redistribute game assets.** The app reads the user's own installation at runtime. Do not commit extracted game files, textures, maps or text tables to any repo. Code, formats and small hand-written facts are fine.
7. **Verify every write by re-reading it** with the same parser (counts, sizes, only intended bytes changed) before handing the file to the user.

## 1. Locations

- Saves: `%USERPROFILE%\Documents\Avalanche Studios\COTW\Saves\<steamid64>\` (often under OneDrive\Documentos). One folder per Steam account.
- Game: `<Steam>\steamapps\common\theHunterCotW\archives_win64\gameN.arc` + `gameN.tab` (N = 0..143), exe `theHunterCotW_F.exe`.
- Steam achievements cache: `<Steam>\appcache\stats\UserGameStats_<accountid32>_518790.bin` and `UserGameStatsSchema_518790.bin` (app id 518790).

### Reserve ids (save file suffix `_N`)
0 Hirschfelden · 1 Layton Lake · 2 Medved-Taiga · 3 Vurhonga Savanna · 4 Parque Fernando · 6 Yukon Valley · 8 Cuatro Colinas · 9 Silver Ridge Peaks · 11 Rancho del Arroyo · 12 Mississippi Acres · 13 Revontuli Coast · 14 New England Mountains · 16 Emerald Coast · 17 Sundarpatan (later DLC reserves follow the same pattern; discover ids from the files present).

## 2. Container formats

### 2.1 Save file wrapper (all `*_adf` / `animal_population_N` saves)
- 32-byte header: magic `SAVE`… then `COMP`; **u32 @8 and u32 @24 = decompressed size**. Update both when the size changes.
- Bytes 32.. = zlib stream (`zlib.compress`, standard header).
- Decompressed = 5-byte prefix (`01 01 00 00 00` in profile) + an **ADF** file. Keep the prefix verbatim.

### 2.2 ADF (Avalanche Data Format)
- Magic `" FDA"` (bytes `20 46 44 41`). Header u32s from offset 4: version, instanceCount, instanceOffset, typedefCount, typedefOffset, stringHashCount, stringHashOffset, nameCount, nameOffset, totalSize.
- Name table: `nameCount` length bytes, then the strings each NUL-terminated.
- Typedef: `meta,size,align,typeHash,nameIdx(u64),flags,elemTypeHash,elemLen,memberCount`. meta 0 primitive, 1 struct (members: nameIdx u64, typeHash, size, offset (low 24 bits; high 8 = bit), flags, default u64 — 32 bytes each), 3 array (heap: u32 offset rel. to instance data, u32 chain, u64 count), 4 inline array (fixed `elemLen`), 7 bit field, 8 enum, 9 string hash.
- Instance header: `nameHash, typeHash, dataOffset, dataSize, nameIdx(u64)`.
- Primitive type hashes: u8 0x0CA2821D, s8 0x580D0A62, u16 0x86D152BD, s16 0xD13FCF93, u32 0x075E4E4F, s32 0x192FE633, u64 0xA139E01F, s64 0xAF41354F, f32 0x7515A207, f64 0xC609F663, String 0x8955583E.
- Reference parser: `research/reference/adf.py` (dump to JSON), `core.js` (browser, read/write).

### 2.3 ADF relocation chain (critical for any resize)
Every non-null array/pointer field is a 16-byte header `{u32 offset, u32 next, u64 count}`. **`next` = distance in bytes from this pointer field to the next pointer field in the instance** (sorted by position); last one keeps its original value (0). After inserting/removing bytes you must: shift every `offset` that points past the edit, update counts, rewrite `next` for all pointers, bump instance `dataSize`, and shift header offsets (instance/typedef/stringhash/name/total) that lie after the instance data. Empty arrays (offset 0, count 0) are not in the chain. Verified: original files have 0 chain mismatches; the old insert code produced 1 → game crash on load.

### 2.4 Archives `.tab` / `.arc`
- `.tab`: 12-byte header then 12-byte entries `{u32 nameHash, u32 offset, u32 size}` into the matching `.arc`.
- **nameHash = Jenkins `hashlittle(path)`** (lookup3, init 0) of the lowercase path, e.g. `missions/achievements.hpmissionsc` → `b0e88e20`, `settings/hp_settings/player_skills.bin`, `settings/hp_settings/player_skill_trees.bin`, `ui/skill_tree.gfx`. Brute-forcing candidate paths against all tabs is how files are found; beware 32-bit collisions (validate the magic of the hit).
- Entries are raw ADF/RTPC, or **AAF** bundles: `AAF` header, u32 chunkCount @44, chunks from 48: `{compSize, uncompSize, chunkSize, magic}` + raw-deflate data (`zlib -15`); if comp==uncomp the chunk is stored. Concatenate → **SARC**: `@4 'SARC'`, u32 dataSize @12, entries from 16: `{u32 nameLen, name (padded), u32 offset, u32 size}` (offset 0 = external file).
- **RTPC** (`.blo`, `settings/player.bin`): `RTPC`, v2; node `{hash, dataOff, u16 propCount, u16 childCount}`, props 9 bytes `{hash, 4-byte raw, type}`; types 1 u32, 2 f32, 3 string, 4–8 vec2/vec3/vec4/mat3/mat4, 9 u32 array, 10 f32 array, 11 bytes, 13 u64, 14 objectid array. Parser: `research/reference/rtpc.py`.
- **XLS books** (`player_skills.bin`, `player_skill_trees.bin`): ADF instance `XLSBook{Sheet[{Cols,Rows,CellIndex[],Name}], Cell[{Type(1 string,2 value),DataIndex,AttributeIndex}], StringData[], ValueData[], BoolData…}`. Cell (r,c) = `Cell[Sheet.CellIndex[r*Cols+c]]`.
- `.gfx` = Scaleform: `CFX` + version + zlib → SWF (UI layout/strings only).
- Localized UI text (PT-BR etc.) was **not** found uncompressed; mission text lives in `missions/missions.stringlookup` (language index 7 = Portuguese). UI names below are the official English ones.

## 3. Save files

### 3.1 `animal_population_N` (per reserve)
- Root `AnimalPopulationReserveData{AlreadyConverted, Populations[], HuntingPressureMap, ReserveSeed, AtypicalsCooldownTime}`.
- `AnimalPopulationData` (96 B): `NameHashId@0, Groups[]@8, RepopulateData[]@24, InitialTotalScore@40, InitialNumAnimals@44, Revision@48, GroupToWarrenId@56, …`.
- `AnimalGroupData` (48 B): `SpawnAreadId@0, NeedZonePathGuids[]@8 (u32 area keys = the group's daily route, in order, cyclic), Animals[]@24, Id@40`. UI group number = index+1.
- `AnimalData` (32 B): `Gender u8@0 (1 male, 2 female), Weight f32@4, Score f32@8, FeatureModifiers u8@12 (bit0 = Great One), IsScripted u8@13, VisualVariationSeed u32@16 (fur), Id u32@20 (0 in current saves), MapPosition vec2@24 (always 0 — positions are NOT stored; use routes)`.
- Species of a population: majority species of its route area keys (spawn maps, §4.1). `NameHashId` alone is incomplete.
- **Safe edits (same size):** gender, weight, score, GO bit, fur seed. Diamond = male with score ≥ species diamond `score_low` (see §4.4). One diamond per group without touching population worked in game.
- **Creating animals** = insert 32-byte records in a group's `Animals` array (clone one from the same group) + full §2.3 rebuild + cap group size at the species' max observed group size. Bigger groups / broken chain crashed the game. Treat as experimental and verify.

### 3.2 `found_need_zones_adf` (discovered need zones, all reserves)
- Instance: `@8 array of reserves {u64 ReserveId, array zones}`; zone = 64 bytes: `x,y,z f32 @0; areaKey u32 @12; type u32 @16 (1 eat, 2 drink, 3 rest); uniqueId 8 bytes @24 (random, must be unique); start f32 @32, end f32 @36 (hours 0–24, can wrap); u32 @40 = species-specific constant (copy the most common value of that species in the save); u32 @44 = 0; DiscoveredGroups array @48 (u32 list)`.
- Writer must rebuild the whole instance (`research/reference/nzwrite.py` + `nzfile.py`): pointers, alignment 8, relocation chain, instance size, header offsets.
- Zone position for a not-yet-discovered zone = center of the spawn-map area AABB (x,z); y = `AabbMin.y + 128.7` (median terrain height above AabbMin observed on game-written zones). Never write y = 1.0 (old bug). The game accepted injected zones and later rewrote the file itself.
- Zone times: take the species' real slots from zones the game already wrote (per species and type), choose the slot that fits between the group's known neighbour stops (votes). If nothing known, use the species/reserve schedule table.

### 3.3 `worlditemsdata_adf` (placed structures)
- `WorldItem` 112 B: `SpawnTag@4, EquipmentHash@8, Position vec3@12, Quaternion@24, ReserveId@40, SaveLinkId@44 (unique, increment max), arrays @48 Cosmetics(4B), @64 BaitSiteData(16B), @80 TreestandData(16B)` + a returned-items u32 array. Writer: `research/reference/wiwrite.py`.
- `EquipmentHash = hashlittle("equipment_<name>")`. Known: tent_01 1536931832, tent_02 2431759753, tent_03 1302347349, tent_04 2381084796, tent_05 1906456911, tripod_01 2614406862, tripod_02 3977902675, tripod_03 2716977701, duck decoys 1821735882/2620602711, turkey decoys 3312710950/1846331269, goose decoys 3597600564/3484985562/644708169, feeder_ribcage 3276263588, feeder_bear_barrel 680407533 (BaitSiteData amount u32 @4), remote caller (Quist) 1857381197.
- Only place items on known ground heights (game-written zone y, existing items). Users disliked mass removal: re-adjust (move useless items near zones/lakes, add mixed sets around lakes), never delete.

### 3.4 `found_icons_N`
`IconData{u64 Id, u8 State}`; State 2 = discovered. Setting all to 2 reveals map icons.

### 3.5 `thp_player_profile_adf` (player)
`PlayerProfileData` (inline arrays, same-size edits): `Level@12, Xp, SkillPoints@20, PerkPoints@24, SkillPointsSpent u32[16]@28 ([0] Stalker, [1] Ambusher), PerkPointsSpent u32[16]@92 ([0] Rifles, [1] Handguns, [2] Shotguns, [3] Archery), Cash, WeaponLevel[4], WeaponScore[4], Equipment*[1024], Skills u32[128]@16576 (hashlittle(id)), SkillLevel u32[128]@17088 (rank-1), ActiveSkill@17600 (hash, 0 if not owned)`. Offsets are relative to instance data — always resolve them from the typedef. Level 60 = 22 skill + 22 perk points.

### 3.6 Missions & achievements
- `missions_missiondata_adf`: `Campaigns_Memory.ChainMemory[{Key = reserve id, Value}]`; children = missions (`CompletionState` 4 done, 2/3 active, 0 locked), grandchildren = objectives (`CompletionsDone`).
- Definitions: `missions/missions*.hpmissionsc` (game34; per reserve file map `{0:9499bacb,1:e85b2f1a,2:0374d3d7,3:e70461a3,4:c1e69aed,6:f4bf4737,8:76eced24,9:7433b547,11:0fe39d5c,12:3a9ef0a1,13:1af7e409,14:b6d11bc2,16:a85a98d5,17:d83f6a67}`). Text via `missions/missions.stringlookup` (hash → per-language strings). Mission giver: `Character = hashlittle("first_last")`.
- `achievements_progress_missiondata_adf` + definitions `missions/achievements.hpmissionsc` (b0e88e20). **Achievement ↔ mission: mission `Name` = hashlittle(Steam api name, e.g. "ach_012")** (155/160 match). Progress = objective `CompletionsDone/CountRequired`; multi-objective achievements = count of objectives done.
- Which item is missing: objective conditions point (offset into the definition ADF, +0x60 base) to strings such as species (`wild_boar`, `eurasian_lynx`) or regions (`sib_d_02`). Counter-only achievements keep `ValueOccurrences` hash tables in the save: e.g. Wildlife Paparazzi stores `hashlittle(species)` of every photographed species.
- Steam unlock state: binary KeyValues cache (type bytes 0 nested, 1 string, 2/4/6 int, 3 float, 7 u64, 8 end); bits per stat id map to schema achievements. Steam web profiles do not expose this without auth.

## 4. Game data

### 4.1 Spawn maps (need-zone areas)
`atlas_aaf` SARC entries `settings/hp_settings/hp_ai_textures/spawn_maps/spawn_<species>.bmp_datac` (ADF): `Layers[{TypeHash, Areas[{key, AabbMin, AabbMax}]}]`; TypeHash 0x0407CDFC eat, 0x48D18EBE drink, 0xAE825F0D rest. Area key (masked u32) = the key used in group routes and zones. Normalize species names (strip `spawn_`, `e_NN_`, `reserve_NN_`; alias table e.g. bear→black_bear).

### 4.2 Fur
- `global/global_animal_types.blo` (RTPC, in game34 bundle 22257a35): per species `SAnimalTypeVisualVariation{0x73ead679 index, 0x69e88c57 gender 0/1/2, 0xc82af7b4 rarity, 0xd8a03db0 weight, 0xf336b29f name, 0xeb13fcd3 is_great_one}` → real fur odds.
- **seed → fur is computed in the exe and unknown** (10 RNG hypotheses failed). Known working seeds per fur come from the community APC config (`animal_details.json`: `diamonds{score_low/high, weight_low/high, furs{male|female}{fur:seed}}`, `go{…, furs}`); aliases brown_bear→eurasian_brown_bear, lynx→eurasian_lynx, musk_deer→siberian_musk_deer, roosevelt_elk, rockymountain_elk, prong_horn, eu_bison, eu_hare, eu_rabbit, eg_kangaroo, sw_crocodile, sidestriped_jackal, southeastern_ibex, tundra_bean_goose. Great One for species missing from APC: copy weight/score/seed of a real GO already in the user's save.

### 4.3 Skills & perks
- `settings/hp_settings/player_skills.bin` (XLS): sheets `skills_active` (=Stalker ids `skill_01_xx`), `skills_passive` (=Ambusher `skill_02_xx`), `skills_strategic` (`skill_03_xx`, some used in trees), `perks_rifles/handguns/shotguns/bows` (`perk_0T_xx`). Columns: id, name key, description key, icon, type (active/passive), cooldown, then `Effects Level n`/`Cost Lvl n` pairs (ranks = number of cost cells). Save hash = `hashlittle(id)`.
- `settings/hp_settings/player_skill_trees.bin`: skills sheets give `Tier Cost` (0,1,5,9,13 = points needed in the rows above, same tree); perk sheets are a 3×4 grid of cells `"perk_id, dependency"` where dependency may be `A OR B` (need ≥1 rank in one). Cross placement: `perk_04_04` (Windage) sits in Rifles, `perk_04_02` (Ranger) in Handguns.
- Official names (EN): Stalker — Locate Tracks(01_02,3) · Track Knowledge(01_05,3), I'm Only Happy When It Rains(01_03,2), Hardened(03_07,1) · Connect The Dots(01_08,1), Soft Feet(01_06,2), Weather Prediction(01_07,2 active) · Innate Triangulation(01_11,2), Improvised Blind(01_09,1), Endurance(01_10,1) · Disturbed Vegetation(01_14,1), Wind Prediction(01_12,2 active), Startle Call(01_16,1 active). Ambusher — Scent Tinkerer(02_03,4) · Spotting Knowledge(02_02,3), Dazed And Confused(02_09,1), The More The Merrier(02_10,1) · Sight Spotting(02_11,1), Fatal Attraction(02_06,2), Impact Resistant(02_04,1) · Tag(02_08,2), Who's Deer?(02_12,2), Pack Mule(03_10,1) · Keen Eye(03_08,2 active), Hill Caller(02_15,1), Haggle(02_13,1). Rifles — Muscle Memory(01_01,2) → Breath Control(01_04,3) → Focused Shot(01_02,1); Zeroing(01_05,2); Steady Hands(01_03,2) → Windage(04_04,1). Handguns — Sprint & Load(02_01,1) → Quick Feet(02_02,2) / Survival Instinct(02_05,1) → Ranger(04_02,1) → Lightning Hands(02_06,3) / Quick Draw(02_07,3). Shotguns — Both Eyes Open(03_01,2) → Fast Shouldering(03_03,2) → Tracershot(03_04,1); Body Control(03_05,2) → Recoil Management(03_07,3); Boomstick(03_08,1) needs Tracershot or Recoil Management. Archery — Full Draw(04_01,3) → Move n' Shoot(04_05,1) → Increased Confidence(04_03,2) → Like a Pro(04_07,1); Recycle(04_06,2) → Pumping Iron(04_08,2).
- Validation rule used: a skill of tier T is valid if points in lower tiers of its tree ≥ T; a perk is valid if it has no dependency or one dependency has ≥1 rank. The game also offers a paid in-game tree reset (UI strings `reset_skill_tree`, `m_ResetSkillTreeCost`).

### 4.4 Medals & ranges
Diamond thresholds/ranges per species from APC `diamonds`. Group-route schedule per species/reserve = slots seen in game-written zones (keep slots seen ≥2 times and ≥20% of the max).

## 5. Proven algorithms (port, don't reinvent)
- Group tracker: route = `NeedZonePathGuids` deduped, cyclic; zone position from save if discovered else spawn-map center; times from save else estimated (§3.2). "Where is the group at hour h" = stops whose slot contains h.
- Unlock zones (free tier feature): add every area key used by current group routes that is missing in `found_need_zones_adf`, with position/height/time rules of §3.2.
- Diamonds per group, Great One placement (spread groups > 2.5 km by route centroid), rare furs per species — all same-size population edits.
- Structure re-adjustment: lake clusters from drink zones (350 m chaining), mixed stand+tent(+feeder if predators) per lake, move items that have < 3 zones within 250 m, decoys only near their species' zones.

## 6. Reference code in the repo
`research/reference/`: `adf.py`, `aaf.py`, `sarc.py`, `rtpc.py`, `hash.py` (hashlittle), `nzio.py`, `nzwrite.py`, `nzfile.py`, `wiwrite.py`, `pop.py`, `core.js` (browser ADF + population read/write incl. chain rebuild, zip writer). These are the proven implementations; the app should port them with tests that round-trip real files byte-for-byte when nothing changes.

# Shader and script catalogue

This page is generated (`python tools/gen_catalog.py`). Every new project gets **426 shaders** (`assets/shaders/library/`) and **236 scripts** (`scripts/src/<folder>/`), plus the materials ([catalogue](./nodes/materials)).

::: info How to use
Object shader: drag the `.wgsl` onto an object or pick it in the `shader` field of the MeshRenderer. Camera shader: drop it into the scene view or add it to the camera effects (`params.x` is the strength). The constants at the top of a file are its settings. Script: drag the file from the Project panel onto an object; its fields show in the Inspector.
:::

::: warning Shader instead of material
An object shader currently **replaces** the material on an object; it is not layered on top. To combine them use a node graph (`.kshader` / `.kmat`) or export the graph with **Export as WGSL shader**. Shader layers over a material are planned.
:::

## Shaders (426)

### Camera / general (20)

`black_white`, `chromatic_pop`, `cinematic_bars`, `crt`, `damage_flash`, `dream_glow`, `drunk`, `edge_outline`, `film_grain`, `glitch`, `night_vision`, `pixelate`, `posterize`, `sepia`, `sharpen`, `teal_orange`, `thermal`, `tilt_shift`, `underwater`, `vhs`

### Camera / grade (36)

`Contrast12`, `Contrast15`, `Contrast20`, `Contrast7`, `GradeBleach`, `GradeCool`, `GradeCross`, `GradeDusk`, `GradeFade`, `GradeGolden`, `GradeMatrix`, `GradeMoon`, `GradeNoir`, `GradeToxic`, `GradeVivid`, `GradeWarm`, `HueShift120`, `HueShift180`, `HueShift240`, `HueShift60`, `Saturation0`, `Saturation14`, `Saturation20`, `Saturation4`, `TintBlue`, `TintCyan`, `TintGreen`, `TintLime`, `TintMagenta`, `TintOrange`, `TintPink`, `TintRed`, `TintTeal`, `TintViolet`, `TintWhite`, `TintYellow`

### Camera / lens (13)

`Chromatic1`, `Chromatic2`, `Chromatic4`, `Chromatic8`, `Grain15`, `Grain4`, `Grain8`, `Sharpen`, `SoftBlur`, `WaveDistort12`, `WaveDistort20`, `WaveDistort5`, `WaveDistort8`

### Camera / pixel (62)

`Pixel128Autumn`, `Pixel128Candy`, `Pixel128Cga`, `Pixel128GameBoy`, `Pixel128GameBoyPocket`, `Pixel128Ice`, `Pixel128Neon`, `Pixel128Sepia`, `Pixel160Autumn`, `Pixel160Candy`, `Pixel160Cga`, `Pixel160GameBoy`, `Pixel160GameBoyPocket`, `Pixel160Ice`, `Pixel160Neon`, `Pixel160Sepia`, `Pixel240Autumn`, `Pixel240Candy`, `Pixel240Cga`, `Pixel240GameBoy`, `Pixel240GameBoyPocket`, `Pixel240Ice`, `Pixel240Neon`, `Pixel240Sepia`, `Pixel320Autumn`, `Pixel320Candy`, `Pixel320Cga`, `Pixel320GameBoy`, `Pixel320GameBoyPocket`, `Pixel320Ice`, `Pixel320Neon`, `Pixel320Sepia`, `Pixel64Autumn`, `Pixel64Candy`, `Pixel64Cga`, `Pixel64GameBoy`, `Pixel64GameBoyPocket`, `Pixel64Ice`, `Pixel64Neon`, `Pixel64Sepia`, `Pixel96Autumn`, `Pixel96Candy`, `Pixel96Cga`, `Pixel96GameBoy`, `Pixel96GameBoyPocket`, `Pixel96Ice`, `Pixel96Neon`, `Pixel96Sepia`, `PixelColour120x3`, `PixelColour120x4`, `PixelColour120x6`, `PixelColour160x3`, `PixelColour160x4`, `PixelColour160x6`, `PixelColour240x3`, `PixelColour240x4`, `PixelColour240x6`, `PixelColour80x3`, `PixelColour80x4`, `PixelColour80x6`, `PixelOutline120`, `PixelOutline200`

### Camera / retro (15)

`Pixelate120`, `Pixelate180`, `Pixelate240`, `Pixelate60`, `Pixelate90`, `Posterize12`, `Posterize2`, `Posterize3`, `Posterize4`, `Posterize6`, `Posterize8`, `Scanlines120`, `Scanlines200`, `Scanlines300`, `Scanlines480`

### Camera / stylize (6)

`Duotone`, `Heat`, `Invert`, `Sepia`, `Solarize`, `Threshold`

### Camera / vignette (6)

`Vignette120`, `Vignette20`, `Vignette35`, `Vignette50`, `Vignette70`, `Vignette90`

### Object / fx3d (28)

`CoverBlue`, `CoverGreen`, `CoverOrange`, `CoverPink`, `CoverViolet`, `CoverWhite`, `DissolveCyan`, `DissolveGreen`, `DissolveOrange`, `DissolveRed`, `DissolveViolet`, `DissolveWhite`, `FlameBlue`, `FlameCyan`, `FlameGreen`, `FlameOrange`, `FlameRed`, `FlameViolet`, `GlitterCyan`, `GlitterPink`, `GlitterWhite`, `GlitterYellow`, `MatCapCandy`, `MatCapFire`, `MatCapForest`, `MatCapOcean`, `MatCapSunset`, `MatCapToxic`

### Object / general (26)

`bricks`, `checker_floor`, `dissolve`, `fire`, `force_field`, `glass`, `gold_metal`, `height_gradient`, `hologram`, `ice`, `lava`, `marble`, `moss_on_top`, `neon_glow`, `outline_toon`, `pearl`, `rainbow_height`, `rim_light`, `sand_ripples`, `snow_cover`, `toon`, `triplanar_blend`, `tron_grid`, `unlit_color`, `wind_sway`, `wood_planks`

### Object / glow (48)

`ForceFieldCyan`, `ForceFieldGreen`, `ForceFieldMagenta`, `ForceFieldOrange`, `ForceFieldViolet`, `ForceFieldWhite`, `GhostBlue`, `GhostCyan`, `GhostGreen`, `GhostMagenta`, `GhostOrange`, `GhostRed`, `GhostViolet`, `GhostWhite`, `HologramBlue`, `HologramCyan`, `HologramGreen`, `HologramMagenta`, `HologramOrange`, `HologramRed`, `HologramWhite`, `HologramYellow`, `IridescentGreen`, `IridescentLime`, `IridescentOrange`, `IridescentRed`, `IridescentTeal`, `IridescentYellow`, `PulseBlue`, `PulseCyan`, `PulseGreen`, `PulseMagenta`, `PulseOrange`, `PulseRed`, `PulseViolet`, `PulseYellow`, `RimGlowBlue`, `RimGlowCyan`, `RimGlowGreen`, `RimGlowLime`, `RimGlowMagenta`, `RimGlowOrange`, `RimGlowPink`, `RimGlowRed`, `RimGlowTeal`, `RimGlowViolet`, `RimGlowWhite`, `RimGlowYellow`

### Object / noise (24)

`FlowCandy`, `FlowDesert`, `FlowFire`, `FlowForest`, `FlowGold`, `FlowIce`, `FlowLava`, `FlowMono`, `FlowNight`, `FlowOcean`, `FlowSunset`, `FlowToxic`, `NoiseCandy`, `NoiseDesert`, `NoiseFire`, `NoiseForest`, `NoiseGold`, `NoiseIce`, `NoiseLava`, `NoiseMono`, `NoiseNight`, `NoiseOcean`, `NoiseSunset`, `NoiseToxic`

### Object / pattern (53)

`Checker3DCandy`, `Checker3DFire`, `Checker3DForest`, `Checker3DIce`, `Checker3DNight`, `Checker3DOcean`, `Checker3DSunset`, `Checker3DToxic`, `GridBlue`, `GridCyan`, `GridGreen`, `GridMagenta`, `GridOrange`, `GridWhite`, `HeightGradientCandy`, `HeightGradientDesert`, `HeightGradientFire`, `HeightGradientForest`, `HeightGradientGold`, `HeightGradientIce`, `HeightGradientLava`, `HeightGradientMono`, `HeightGradientNight`, `HeightGradientOcean`, `HeightGradientSunset`, `HeightGradientToxic`, `PolkaDotsCandy`, `PolkaDotsFire`, `PolkaDotsForest`, `PolkaDotsOcean`, `PolkaDotsSunset`, `PolkaDotsToxic`, `RadialGradientCandy`, `RadialGradientDesert`, `RadialGradientFire`, `RadialGradientForest`, `RadialGradientGold`, `RadialGradientIce`, `RadialGradientLava`, `RadialGradientMono`, `RadialGradientNight`, `RadialGradientOcean`, `RadialGradientSunset`, `RadialGradientToxic`, `StripesXForest`, `StripesXOcean`, `StripesXSunset`, `StripesYForest`, `StripesYOcean`, `StripesYSunset`, `StripesZForest`, `StripesZOcean`, `StripesZSunset`

### Object / pixel (25)

`PixelNoise10`, `PixelNoise16`, `PixelNoise6`, `PixelTexture128`, `PixelTexture16`, `PixelTexture32`, `PixelTexture64`, `VoxelLook16Cga`, `VoxelLook16GameBoy`, `VoxelLook16GameBoyPocket`, `VoxelLook16Ice`, `VoxelLook16Neon`, `VoxelLook16Sepia`, `VoxelLook32Cga`, `VoxelLook32GameBoy`, `VoxelLook32GameBoyPocket`, `VoxelLook32Ice`, `VoxelLook32Neon`, `VoxelLook32Sepia`, `VoxelLook8Cga`, `VoxelLook8GameBoy`, `VoxelLook8GameBoyPocket`, `VoxelLook8Ice`, `VoxelLook8Neon`, `VoxelLook8Sepia`

### Object / texture (6)

`ConveyorDiagonal`, `ConveyorFast`, `ConveyorScrollDown`, `ConveyorScrollLeft`, `ConveyorScrollRight`, `ConveyorScrollUp`

### Object / toon (16)

`OutlineHeavy`, `OutlineInk`, `OutlineThick`, `OutlineThin`, `Toon2BandsCyanRim`, `Toon2BandsOrangeRim`, `Toon2BandsWhiteRim`, `Toon3BandsCyanRim`, `Toon3BandsOrangeRim`, `Toon3BandsWhiteRim`, `Toon4BandsCyanRim`, `Toon4BandsOrangeRim`, `Toon4BandsWhiteRim`, `Toon5BandsCyanRim`, `Toon5BandsOrangeRim`, `Toon5BandsWhiteRim`

### Object / vertex (16)

`BounceSoft`, `BounceStrong`, `BreatheSoft`, `BreatheStrong`, `FlagSoft`, `FlagStrong`, `JitterSoft`, `JitterStrong`, `RippleSoft`, `RippleStrong`, `TwistSoft`, `TwistStrong`, `WaveSoft`, `WaveStrong`, `WindSoft`, `WindStrong`

### Object / water (26)

`ToonWaterInk`, `ToonWaterLake`, `ToonWaterLava`, `ToonWaterMilk`, `ToonWaterOcean`, `ToonWaterPool`, `ToonWaterRiver`, `ToonWaterSwamp`, `ToonWaterToxic`, `ToonWaterTropical`, `WaterInk`, `WaterLake`, `WaterLava`, `WaterMilk`, `WaterOcean`, `WaterPool`, `WaterRiver`, `WaterSwamp`, `WaterToxic`, `WaterTropical`, `WavySurfaceLake`, `WavySurfaceOcean`, `WavySurfacePool`, `WavySurfaceRiver`, `WavySurfaceSwamp`, `WavySurfaceTropical`

## Scripts (236)

### animate (67)

| Script | Description |
|---|---|
| `blink_alarm` | Blinks between the original colour and a flash colour. |
| `blink_cyan` | Blinks between the original colour and a flash colour. |
| `blink_green` | Blinks between the original colour and a flash colour. |
| `blink_red` | Blinks between the original colour and a flash colour. |
| `blink_warning` | Blinks between the original colour and a flash colour. |
| `bob_x` | Moves up and down along X like a buoy. |
| `bob_xbig` | Moves up and down along X like a buoy. |
| `bob_xsmall` | Moves up and down along X like a buoy. |
| `bob_y` | Moves up and down along Y like a buoy. |
| `bob_ybig` | Moves up and down along Y like a buoy. |
| `bob_ysmall` | Moves up and down along Y like a buoy. |
| `bob_z` | Moves up and down along Z like a buoy. |
| `bob_zbig` | Moves up and down along Z like a buoy. |
| `bob_zsmall` | Moves up and down along Z like a buoy. |
| `color_cycle` | Cycles the object colour through the rainbow. |
| `color_cycle_disco` | Cycles the object colour through the rainbow. |
| `color_cycle_fast` | Cycles the object colour through the rainbow. |
| `color_cycle_slow` | Cycles the object colour through the rainbow. |
| `drive` | Moves forward (local -Z) at a constant speed in metres per second. |
| `drive_fast` | Moves forward (local -Z) at a constant speed in metres per second. |
| `drive_slow` | Moves forward (local -Z) at a constant speed in metres per second. |
| `flicker` | Switches the object on and off irregularly like a broken lamp. |
| `flicker_fast` | Switches the object on and off irregularly like a broken lamp. |
| `flicker_slow` | Switches the object on and off irregularly like a broken lamp. |
| `hover_float` | Bobs up and down while slowly turning: pickups, crystals, floating signs. |
| `hover_hover` | Bobs up and down while slowly turning: pickups, crystals, floating signs. |
| `hover_idle` | Bobs up and down while slowly turning: pickups, crystals, floating signs. |
| `hover_levitate` | Bobs up and down while slowly turning: pickups, crystals, floating signs. |
| `orbit_centre` | Circles around the point where the object started (or an object named in `center`). |
| `orbit_centre_close` | Circles around the point where the object started (or an object named in `center`). |
| `orbit_centre_fast` | Circles around the point where the object started (or an object named in `center`). |
| `orbit_centre_slow` | Circles around the point where the object started (or an object named in `center`). |
| `orbit_centre_tiny` | Circles around the point where the object started (or an object named in `center`). |
| `orbit_centre_wide` | Circles around the point where the object started (or an object named in `center`). |
| `pulse_scale` | Scales the object smoothly between a minimum and a maximum. |
| `pulse_scale_breathing` | Scales the object smoothly between a minimum and a maximum. |
| `pulse_scale_gentle` | Scales the object smoothly between a minimum and a maximum. |
| `pulse_scale_heartbeat` | Scales the object smoothly between a minimum and a maximum. |
| `pulse_scale_pop` | Scales the object smoothly between a minimum and a maximum. |
| `pulse_scale_strong` | Scales the object smoothly between a minimum and a maximum. |
| `slide_x` | Slides back and forth along X. |
| `slide_xlong` | Slides back and forth along X. |
| `slide_xshort` | Slides back and forth along X. |
| `slide_y` | Slides back and forth along Y. |
| `slide_ylong` | Slides back and forth along Y. |
| `slide_yshort` | Slides back and forth along Y. |
| `slide_z` | Slides back and forth along Z. |
| `slide_zlong` | Slides back and forth along Z. |
| `slide_zshort` | Slides back and forth along Z. |
| `spin_x` | Rotates around the X axis (90 degrees per second by default). |
| `spin_xfast` | Rotates around the X axis (360 degrees per second by default). |
| `spin_xslow` | Rotates around the X axis (20 degrees per second by default). |
| `spin_y` | Rotates around the Y axis (90 degrees per second by default). |
| `spin_yfast` | Rotates around the Y axis (360 degrees per second by default). |
| `spin_yslow` | Rotates around the Y axis (20 degrees per second by default). |
| `spin_z` | Rotates around the Z axis (90 degrees per second by default). |
| `spin_zfast` | Rotates around the Z axis (360 degrees per second by default). |
| `spin_zslow` | Rotates around the Z axis (20 degrees per second by default). |
| `wobble_xrock` | Swings around X by +-10 degrees. |
| `wobble_xshake` | Swings around X by +-4 degrees. |
| `wobble_xsway` | Swings around X by +-25 degrees. |
| `wobble_yrock` | Swings around Y by +-10 degrees. |
| `wobble_yshake` | Swings around Y by +-4 degrees. |
| `wobble_ysway` | Swings around Y by +-25 degrees. |
| `wobble_zrock` | Swings around Z by +-10 degrees. |
| `wobble_zshake` | Swings around Z by +-4 degrees. |
| `wobble_zsway` | Swings around Z by +-25 degrees. |

### audio (7)

| Script | Description |
|---|---|
| `ambient_loop` | Starts a music or ambience loop when the game starts. |
| `master_volume_full` | Sets the master volume to 1.0. |
| `master_volume_half` | Sets the master volume to 0.5. |
| `master_volume_quiet` | Sets the master volume to 0.2. |
| `stereo_loop_centre` | Plays a looping 2D sound panned to the centre. |
| `stereo_loop_left` | Plays a looping 2D sound panned to the left. |
| `stereo_loop_right` | Plays a looping 2D sound panned to the right. |

### camera (24)

| Script | Description |
|---|---|
| `camera_shake` | Shakes the camera when a "shake" message arrives (send it with ctx.emit("shake", json!({}))), |
| `zoom_on_key` | Hold the right mouse button to zoom the camera in (a field-of-view change), release to return. |
| `camera_pan` | Turns the camera around the vertical axis for trailers and menus. |
| `camera_pan_fast` | Turns the camera around the vertical axis for trailers and menus. |
| `camera_pan_slow` | Turns the camera around the vertical axis for trailers and menus. |
| `follow_behind_smooth` | Follows the object named `target` with an offset (0.0, 3.0, 7.0), smoothing 5.0. |
| `follow_behind_tight` | Follows the object named `target` with an offset (0.0, 3.0, 7.0), smoothing 14.0. |
| `follow_close_smooth` | Follows the object named `target` with an offset (0.0, 1.8, 3.5), smoothing 5.0. |
| `follow_close_tight` | Follows the object named `target` with an offset (0.0, 1.8, 3.5), smoothing 14.0. |
| `follow_high_smooth` | Follows the object named `target` with an offset (0.0, 9.0, 6.0), smoothing 5.0. |
| `follow_high_tight` | Follows the object named `target` with an offset (0.0, 9.0, 6.0), smoothing 14.0. |
| `follow_shoulder_smooth` | Follows the object named `target` with an offset (1.2, 1.9, 3.0), smoothing 5.0. |
| `follow_shoulder_tight` | Follows the object named `target` with an offset (1.2, 1.9, 3.0), smoothing 14.0. |
| `follow_side_smooth` | Follows the object named `target` with an offset (9.0, 2.5, 0.0), smoothing 5.0. |
| `follow_side_tight` | Follows the object named `target` with an offset (9.0, 2.5, 0.0), smoothing 14.0. |
| `follow_top_down_smooth` | Follows the object named `target` with an offset (0.0, 14.0, 0.5), smoothing 5.0. |
| `follow_top_down_tight` | Follows the object named `target` with an offset (0.0, 14.0, 0.5), smoothing 14.0. |
| `fov_breath_cinematic` | Slowly changes the field of view between two values. |
| `fov_breath_sprint` | Slowly changes the field of view between two values. |
| `fov_breath_wide` | Slowly changes the field of view between two values. |
| `shake_on_key` | Shakes the camera for a moment when you press C (impact feeling). |
| `shake_on_key_hard` | Shakes the camera for a moment when you press C (impact feeling). |
| `shake_on_key_quake` | Shakes the camera for a moment when you press C (impact feeling). |
| `shake_on_key_soft` | Shakes the camera for a moment when you press C (impact feeling). |

### control (31)

| Script | Description |
|---|---|
| `drive_arrows` | Car-style control with the ARROWS keys: forward / back and steering. |
| `drive_ijkl` | Car-style control with the IJKL keys: forward / back and steering. |
| `drive_wasd` | Car-style control with the WASD keys: forward / back and steering. |
| `jump` | Jumps with Space when the body is close to the ground (a physics RigidBody is required). |
| `jump_high` | Jumps with Space when the body is close to the ground (a physics RigidBody is required). |
| `jump_low` | Jumps with Space when the body is close to the ground (a physics RigidBody is required). |
| `jump_moon` | Jumps with Space when the body is close to the ground (a physics RigidBody is required). |
| `move_arrows_run` | Moves the object on the ground with the ARROWS keys (7.0 m/s). Not physics based. |
| `move_arrows_sprint` | Moves the object on the ground with the ARROWS keys (12.0 m/s). Not physics based. |
| `move_arrows_walk` | Moves the object on the ground with the ARROWS keys (3.0 m/s). Not physics based. |
| `move_ijkl_run` | Moves the object on the ground with the IJKL keys (7.0 m/s). Not physics based. |
| `move_ijkl_sprint` | Moves the object on the ground with the IJKL keys (12.0 m/s). Not physics based. |
| `move_ijkl_walk` | Moves the object on the ground with the IJKL keys (3.0 m/s). Not physics based. |
| `move_wasd_run` | Moves the object on the ground with the WASD keys (7.0 m/s). Not physics based. |
| `move_wasd_sprint` | Moves the object on the ground with the WASD keys (12.0 m/s). Not physics based. |
| `move_wasd_walk` | Moves the object on the ground with the WASD keys (3.0 m/s). Not physics based. |
| `shoot_ball_e` | Throws a physics ball forward with E. |
| `shoot_ball_f` | Throws a physics ball forward with F. |
| `shoot_ball_mouse` | Throws a physics ball forward with the left mouse button. |
| `sound_on_e` | Plays a sound when E is pressed. |
| `sound_on_f` | Plays a sound when F is pressed. |
| `sound_on_num1` | Plays a sound when Num1 is pressed. |
| `sound_on_num2` | Plays a sound when Num2 is pressed. |
| `sound_on_space` | Plays a sound when Space is pressed. |
| `toggle_e` | Shows or hides the object when you press E. |
| `toggle_enter` | Shows or hides the object when you press Enter. |
| `toggle_f` | Shows or hides the object when you press F. |
| `toggle_space` | Shows or hides the object when you press Space. |
| `turn_arrows` | Turns left and right with the ARROWS keys. |
| `turn_ijkl` | Turns left and right with the IJKL keys. |
| `turn_wasd` | Turns left and right with the WASD keys. |

### debug (7)

| Script | Description |
|---|---|
| `log_fps_five` | Prints the frame rate every 5.0 s. |
| `log_fps_second` | Prints the frame rate every 1.0 s. |
| `log_globals` | Prints the shared variable `score` every 2 s. |
| `log_position_five` | Prints the object position every 5.0 s. |
| `log_position_second` | Prints the object position every 1.0 s. |
| `log_position_tenth` | Prints the object position every 0.1 s. |
| `log_tags` | Prints the tags of the object once at start. |

### gameplay (65)

| Script | Description |
|---|---|
| `collectible` | A pickup: spins, bobs, and is collected when the player (an object named "Player") gets close. |
| `look_at` | Turns the object towards another object (turrets, eyes, signs). |
| `respawn` | Puts the object back at its start (or a chosen point) when it falls out of the world. |
| `score_counter` | Counts "collected" messages and shows the score; announces the win. |
| `spawner` | Drops a new physics primitive at this object every few seconds, and removes it later. |
| `button_click` | Sends the signal `open` when the Player is near and presses E. |
| `button_enter` | Sends the signal `open` when the Player is near and presses Enter. |
| `count_clicks` | Increments the shared variable `clicks` each time Space is pressed (a test counter). |
| `count_deaths` | Increments the shared variable `deaths` each time Space is pressed (a test counter). |
| `count_kills` | Increments the shared variable `kills` each time Space is pressed (a test counter). |
| `count_score` | Increments the shared variable `score` each time Space is pressed (a test counter). |
| `despawn_after10s` | Removes the object 10 seconds after the game starts. |
| `despawn_after1s` | Removes the object 1 seconds after the game starts. |
| `despawn_after2s` | Removes the object 2 seconds after the game starts. |
| `despawn_after30s` | Removes the object 30 seconds after the game starts. |
| `despawn_after3s` | Removes the object 3 seconds after the game starts. |
| `despawn_after5s` | Removes the object 5 seconds after the game starts. |
| `heartbeat_five` | Broadcasts the signal `tick_5` every 5.0 s. |
| `heartbeat_minute` | Broadcasts the signal `tick_60` every 60.0 s. |
| `heartbeat_second` | Broadcasts the signal `tick_1` every 1.0 s. |
| `heartbeat_ten` | Broadcasts the signal `tick_10` every 10.0 s. |
| `kill_zone_deep` | Teleports the object back to where it started when it falls below y = -100.0. |
| `kill_zone_floor` | Teleports the object back to where it started when it falls below y = -20.0. |
| `kill_zone_shallow` | Teleports the object back to where it started when it falls below y = -5.0. |
| `message_hint` | Shows the message "Press E to interact" once when the game starts. |
| `message_objective` | Shows the message "Find the exit" once when the game starts. |
| `message_warning` | Shows the message "Careful!" once when the game starts. |
| `message_welcome` | Shows the message "Welcome!" once when the game starts. |
| `message_win` | Shows the message "You win!" once when the game starts. |
| `pickup_ammo` | Adds 10 to the shared variable `ammo` when the Player touches it. |
| `pickup_coin` | A pickup worth 1 point(s): spins, and is collected by the object named Player. Adds to the shared variable `score`. |
| `pickup_coins` | Adds 5 to the shared variable `coins` when the Player touches it. |
| `pickup_crystal` | A pickup worth 25 point(s): spins, and is collected by the object named Player. Adds to the shared variable `score`. |
| `pickup_energy` | Adds 50 to the shared variable `energy` when the Player touches it. |
| `pickup_gem` | A pickup worth 10 point(s): spins, and is collected by the object named Player. Adds to the shared variable `score`. |
| `pickup_health` | Adds 25 to the shared variable `health` when the Player touches it. |
| `pickup_lives` | Adds 1 to the shared variable `lives` when the Player touches it. |
| `pickup_orb` | A pickup worth 2 point(s): spins, and is collected by the object named Player. Adds to the shared variable `score`. |
| `pickup_star` | A pickup worth 5 point(s): spins, and is collected by the object named Player. Adds to the shared variable `score`. |
| `proximity_signal` | Sends a signal once when the Player comes close. |
| `proximity_signal_far` | Sends a signal once when the Player comes close. |
| `proximity_signal_near` | Sends a signal once when the Player comes close. |
| `proximity_signal_wide` | Sends a signal once when the Player comes close. |
| `respawn` | Puts the object back at its start after 3.0 s without moving (stuck objects). |
| `respawn_fast` | Puts the object back at its start after 1.0 s without moving (stuck objects). |
| `respawn_slow` | Puts the object back at its start after 8.0 s without moving (stuck objects). |
| `slide_door_x` | Slides 3.5 m along X when it receives the signal `open` and back on `close`. |
| `slide_door_xshort` | Slides 2.0 m along X when it receives the signal `open` and back on `close`. |
| `slide_door_y` | Slides 3.5 m along Y when it receives the signal `open` and back on `close`. |
| `slide_door_yshort` | Slides 2.0 m along Y when it receives the signal `open` and back on `close`. |
| `slide_door_z` | Slides 3.5 m along Z when it receives the signal `open` and back on `close`. |
| `slide_door_zshort` | Slides 2.0 m along Z when it receives the signal `open` and back on `close`. |
| `spawn_capsule_fast` | Drops a physics capsule every 0.4 s above the object (at most `max_alive` at a time). |
| `spawn_capsule_slow` | Drops a physics capsule every 2.0 s above the object (at most `max_alive` at a time). |
| `spawn_cone_fast` | Drops a physics cone every 0.4 s above the object (at most `max_alive` at a time). |
| `spawn_cone_slow` | Drops a physics cone every 2.0 s above the object (at most `max_alive` at a time). |
| `spawn_cube_fast` | Drops a physics cube every 0.4 s above the object (at most `max_alive` at a time). |
| `spawn_cube_slow` | Drops a physics cube every 2.0 s above the object (at most `max_alive` at a time). |
| `spawn_cylinder_fast` | Drops a physics cylinder every 0.4 s above the object (at most `max_alive` at a time). |
| `spawn_cylinder_slow` | Drops a physics cylinder every 2.0 s above the object (at most `max_alive` at a time). |
| `spawn_sphere_fast` | Drops a physics sphere every 0.4 s above the object (at most `max_alive` at a time). |
| `spawn_sphere_slow` | Drops a physics sphere every 2.0 s above the object (at most `max_alive` at a time). |
| `teleport_camera` | Moves the object to the object named Main Camera when T is pressed. |
| `teleport_player` | Moves the object to the object named Player when T is pressed. |
| `teleport_target` | Moves the object to the object named Target when T is pressed. |

### movement (9)

| Script | Description |
|---|---|
| `bobber` | Floats up and down like a pickup or a buoy. |
| `fly_camera` | Free-fly camera: WASD, mouse look, Space / Ctrl for up and down, Shift to go faster. |
| `follow_camera` | Smoothly follows an object from behind and above, always looking at it. |
| `fps_player` | First-person player on a capsule body: WASD, mouse look, Space to jump, Shift to sprint, |
| `jumper` | Adds a jump (Space) to a physics body: only when something is under its feet. |
| `orbit` | Circles around a point and keeps looking at it: product shots, menus, cutscenes. |
| `patrol` | Moves back and forth along an axis (a moving platform, a patrolling enemy). |
| `spinner` | Rotates the object continuously (degrees per second around each axis). |
| `top_down_mover` | Moves a body on the ground plane with WASD (top-down and isometric games). |

### ui (1)

| Script | Description |
|---|---|
| `welcome_message` | Shows a message and an objective when the game starts. |

### world (25)

| Script | Description |
|---|---|
| `clouds_clear` | Sets the cloud cover to 0.0. |
| `clouds_cloudy` | Sets the cloud cover to 0.6. |
| `clouds_few` | Sets the cloud cover to 0.25. |
| `clouds_overcast` | Sets the cloud cover to 1.0. |
| `day_night` | Sets the day length: 0.2 game hours per second. |
| `day_night_fast` | Sets the day length: 1.0 game hours per second. |
| `day_night_frozen` | Sets the day length: 0.0 game hours per second. |
| `day_night_slow` | Sets the day length: 0.05 game hours per second. |
| `day_night_timelapse` | Sets the day length: 4.0 game hours per second. |
| `exposure_blinding` | Sets the exposure to 3.0. |
| `exposure_bright` | Sets the exposure to 1.5. |
| `exposure_dark` | Sets the exposure to 0.5. |
| `look_cinematic` | Applies the `cinematic` post-processing look when the game starts. |
| `look_crisp` | Applies the `crisp` post-processing look when the game starts. |
| `look_dreamy` | Applies the `dreamy` post-processing look when the game starts. |
| `look_plain` | Applies the `none` post-processing look when the game starts. |
| `look_retro` | Applies the `retro` post-processing look when the game starts. |
| `set_time_dawn` | Sets the time of day to 6.0 h and freezes it. |
| `set_time_dusk` | Sets the time of day to 18.5 h and freezes it. |
| `set_time_evening` | Sets the time of day to 20.0 h and freezes it. |
| `set_time_midnight` | Sets the time of day to 0.0 h and freezes it. |
| `set_time_morning` | Sets the time of day to 9.0 h and freezes it. |
| `set_time_noon` | Sets the time of day to 12.0 h and freezes it. |
| `day_night_cycle` | Runs the sky clock. Clouds thicken towards evening. Keys T / Y change the hour by hand. |
| `night_light` | A lamp that switches on at dusk, flickers like a flame and goes out at dawn. |


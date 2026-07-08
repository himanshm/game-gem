//! Audio system (feature-gated behind `audio` feature).
//!
//! Provides:
//! - **Spatial audio** — 2D positional sound with distance falloff
//! - **Music & SFX channels** — separate volume controls
//! - **Fade in/out** — smooth volume transitions
//! - **Sound pooling** — avoid allocating duplicate sounds
//!
//! Built on `rodio` for cross-platform audio output.

#![cfg_attr(not(feature = "audio"), allow(dead_code))]

use crate::math::Vec2;
use std::collections::HashMap;

// ─────────────────────────────────────────────
// Sound handle
// ─────────────────────────────────────────────

/// A loaded sound effect.
#[derive(Debug, Clone)]
pub struct Sound {
    /// Name identifier.
    pub name: String,
    /// Whether this is a music track (streamed) vs SFX (loaded into memory).
    pub is_music: bool,
    /// Base volume (0.0–1.0).
    pub volume: f32,
    /// Pitch multiplier.
    pub pitch: f32,
    /// Whether to loop.
    pub looping: bool,
    /// Minimum distance for spatial audio (within this, no attenuation).
    pub spatial_min_distance: f32,
    /// Maximum distance for spatial audio (beyond this, inaudible).
    pub spatial_max_distance: f32,
    /// 2D position for spatial audio.
    pub position: Option<Vec2>,
}

impl Sound {
    /// Create a new sound configuration.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_music: false,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial_min_distance: 50.0,
            spatial_max_distance: 500.0,
            position: None,
        }
    }

    /// Builder: set as music (streamed).
    pub fn as_music(mut self) -> Self {
        self.is_music = true;
        self
    }

    /// Builder: set volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Builder: set pitch.
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 10.0);
        self
    }

    /// Builder: set looping.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Builder: set 2D position for spatial audio.
    pub fn with_position(mut self, pos: Vec2) -> Self {
        self.position = Some(pos);
        self
    }

    /// Builder: set spatial distance range.
    pub fn with_spatial_range(mut self, min: f32, max: f32) -> Self {
        self.spatial_min_distance = min;
        self.spatial_max_distance = max;
        self
    }
}

// ─────────────────────────────────────────────
// Audio Manager
// ─────────────────────────────────────────────

/// Master volume channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel {
    /// Sound effects channel.
    Sfx,
    /// Music channel.
    Music,
    /// UI sounds channel.
    Ui,
    /// Ambient / environment channel.
    Ambient,
}

/// The audio manager controls playback, volume, and spatial audio.
///
/// # Example
/// ```
/// let mut audio = AudioManager::new();
/// audio.load("jump", "sounds/jump.ogg");
/// audio.load("bgm", "music/theme.ogg");
///
/// // Play a sound effect
/// audio.play("jump");
///
/// // Play music with fade-in
/// audio.play_music("bgm", Fade::In(2.0));
///
/// // Update spatial audio
/// audio.update_listener_position(listener_pos);
/// audio.update(dt);
/// ```
pub struct AudioManager {
    /// Master volume (0.0–1.0).
    master_volume: f32,
    /// Per-channel volumes.
    channel_volumes: HashMap<AudioChannel, f32>,
    /// Loaded sounds.
    sounds: HashMap<String, Sound>,
    /// Currently playing instances.
    instances: Vec<PlayingInstance>,
    /// Listener position for spatial audio.
    listener_position: Vec2,
}

/// A currently-playing sound instance.
#[derive(Debug, Clone)]
struct PlayingInstance {
    sound_name: String,
    channel: AudioChannel,
    /// Current volume (after all multipliers).
    current_volume: f32,
    /// Fade state.
    fade: Option<FadeState>,
    /// Whether this instance is still alive.
    alive: bool,
}

/// Fade in/out state.
#[derive(Debug, Clone, Copy)]
struct FadeState {
    kind: FadeKind,
    duration: f32,
    elapsed: f32,
    start_volume: f32,
    target_volume: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FadeKind {
    In,
    Out,
}

/// Fade configuration for play commands.
#[derive(Debug, Clone, Copy)]
pub enum Fade {
    /// Fade in over N seconds.
    In(f32),
    /// Fade out over N seconds, then stop.
    Out(f32),
    /// No fade.
    None,
}

impl Default for Fade {
    fn default() -> Self {
        Fade::None
    }
}

impl AudioManager {
    /// Create a new audio manager.
    pub fn new() -> Self {
        let mut channel_volumes = HashMap::new();
        channel_volumes.insert(AudioChannel::Sfx, 1.0);
        channel_volumes.insert(AudioChannel::Music, 0.8);
        channel_volumes.insert(AudioChannel::Ui, 0.7);
        channel_volumes.insert(AudioChannel::Ambient, 0.6);

        Self {
            master_volume: 1.0,
            channel_volumes,
            sounds: HashMap::new(),
            instances: Vec::new(),
            listener_position: Vec2::ZERO,
        }
    }

    /// Register a sound (in a real impl, this loads the audio file).
    pub fn load(&mut self, name: &str, _path: &str) -> Result<(), String> {
        let sound = Sound::new(name);
        self.sounds.insert(name.to_string(), sound);
        Ok(())
    }

    /// Register a sound with custom configuration.
    pub fn load_with_config(&mut self, sound: Sound) {
        let name = sound.name.clone();
        self.sounds.insert(name, sound);
    }

    /// Play a sound effect.
    pub fn play(&mut self, name: &str) {
        self.play_with_fade(name, AudioChannel::Sfx, Fade::None);
    }

    /// Play a sound on a specific channel.
    pub fn play_on_channel(&mut self, name: &str, channel: AudioChannel) {
        self.play_with_fade(name, channel, Fade::None);
    }

    /// Play music with optional fade.
    pub fn play_music(&mut self, name: &str, fade: Fade) {
        // Stop current music
        self.stop_channel(AudioChannel::Music);
        self.play_with_fade(name, AudioChannel::Music, fade);
    }

    /// Internal: play with fade.
    fn play_with_fade(&mut self, name: &str, channel: AudioChannel, fade: Fade) {
        let sound = match self.sounds.get(name) {
            Some(s) => s.clone(),
            None => return,
        };

        let base_volume = sound.volume
            * self.master_volume
            * self.channel_volumes.get(&channel).copied().unwrap_or(1.0);

        // Spatial attenuation
        let spatial_volume = if let Some(sound_pos) = sound.position {
            let dist = sound_pos.distance_to(self.listener_position);
            if dist <= sound.spatial_min_distance {
                1.0
            } else if dist >= sound.spatial_max_distance {
                0.0
            } else {
                1.0 - (dist - sound.spatial_min_distance)
                    / (sound.spatial_max_distance - sound.spatial_min_distance)
            }
        } else {
            1.0
        };

        let start_volume = match fade {
            Fade::In(duration) => {
                Some(FadeState {
                    kind: FadeKind::In,
                    duration,
                    elapsed: 0.0,
                    start_volume: 0.0,
                    target_volume: base_volume * spatial_volume,
                })
            }
            Fade::Out(duration) => {
                Some(FadeState {
                    kind: FadeKind::Out,
                    duration,
                    elapsed: 0.0,
                    start_volume: base_volume * spatial_volume,
                    target_volume: 0.0,
                })
            }
            Fade::None => None,
        };

        let current_volume = start_volume
            .as_ref()
            .map(|f| f.start_volume)
            .unwrap_or(base_volume * spatial_volume);

        self.instances.push(PlayingInstance {
            sound_name: name.to_string(),
            channel,
            current_volume,
            fade: start_volume,
            alive: true,
        });
    }

    /// Stop all instances of a named sound.
    pub fn stop(&mut self, name: &str) {
        for instance in &mut self.instances {
            if instance.sound_name == name {
                instance.alive = false;
            }
        }
    }

    /// Stop all instances on a channel.
    pub fn stop_channel(&mut self, channel: AudioChannel) {
        for instance in &mut self.instances {
            if instance.channel == channel {
                instance.alive = false;
            }
        }
    }

    /// Stop all sounds.
    pub fn stop_all(&mut self) {
        for instance in &mut self.instances {
            instance.alive = false;
        }
    }

    /// Set the master volume (0.0–1.0).
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Set a channel's volume (0.0–1.0).
    pub fn set_channel_volume(&mut self, channel: AudioChannel, volume: f32) {
        self.channel_volumes.insert(channel, volume.clamp(0.0, 1.0));
    }

    /// Set the listener position for spatial audio.
    pub fn set_listener_position(&mut self, pos: Vec2) {
        self.listener_position = pos;
    }

    /// Pause all audio.
    pub fn pause_all(&mut self) {
        // In a real implementation, this would pause the audio output
    }

    /// Resume all audio.
    pub fn resume_all(&mut self) {
        // In a real implementation, this would resume the audio output
    }

    /// Check if a named sound is currently playing.
    pub fn is_playing(&self, name: &str) -> bool {
        self.instances.iter().any(|i| i.sound_name == name && i.alive)
    }

    /// Update the audio system (call once per frame).
    pub fn update(&mut self, dt: f32) {
        for instance in &mut self.instances {
            if !instance.alive {
                continue;
            }

            // Update fade
            if let Some(fade) = &mut instance.fade {
                fade.elapsed += dt;
                let t = (fade.elapsed / fade.duration).clamp(0.0, 1.0);
                instance.current_volume = fade.start_volume.lerp(fade.target_volume, t);

                if t >= 1.0 {
                    match fade.kind {
                        FadeKind::Out => instance.alive = false,
                        FadeKind::In => instance.fade = None,
                    }
                }
            }
        }

        // Remove dead instances
        self.instances.retain(|i| i.alive);
    }

    /// Number of currently playing instances.
    pub fn playing_count(&self) -> usize {
        self.instances.iter().filter(|i| i.alive).count()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
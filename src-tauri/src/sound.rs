use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum SoundKind {
    Done,
    Urgent,
}

pub fn play(kind: SoundKind) {
    std::thread::spawn(move || {
        play_blocking(kind);
    });
}

#[cfg(target_os = "macos")]
fn play_blocking(kind: SoundKind) {
    // macOS ships these in /System/Library/Sounds/
    let name = match kind {
        SoundKind::Done => "Glass",
        SoundKind::Urgent => "Basso",
    };
    Command::new("afplay")
        .arg(format!("/System/Library/Sounds/{name}.aiff"))
        .output()
        .ok();
}

#[cfg(target_os = "linux")]
fn play_blocking(kind: SoundKind) {
    // Freedesktop sound theme (pacman -S sound-theme-freedesktop on Arch)
    let name = match kind {
        SoundKind::Done => "complete",
        SoundKind::Urgent => "dialog-warning",
    };
    let candidates = [
        format!("/usr/share/sounds/freedesktop/stereo/{name}.oga"),
        format!("/usr/share/sounds/freedesktop/stereo/{name}.ogg"),
        format!("/usr/share/sounds/ubuntu/stereo/{name}.ogg"),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            Command::new("paplay").arg(path).output().ok();
            return;
        }
    }
    // Last resort: system bell via paplay
    Command::new("paplay")
        .arg("--volume=32768")
        .arg("/dev/null")
        .output()
        .ok();
}

// Stub for any other platform (shouldn't happen per README, but keeps it compiling)
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn play_blocking(_kind: SoundKind) {}

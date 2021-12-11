
# Todo
* Font loading
    * include crossfont
    * load a font
* Atlas generation
    * Research alacritty

# Crates Used

```
structopt = "0.3.22"
log = { version = "0.4", features = ["std", "serde"] }
time = "0.1.40"
fnv = "1"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.8"
serde_json = "1"
glutin = { version = "0.26.0", default-features = false, features = ["serde"] }
notify = "4"
parking_lot = "0.11.0"
crossfont = { version = "0.3.1", features = ["force_system_fontconfig"] }
copypasta = { version = "0.7.0", default-features = false }
libc = "0.2"
unicode-width = "0.1"
bitflags = "1"
dirs = "3.0.1"

```

* copypasta
    * system clipboard library
* fnv
    * fast hashing for short key hashes / sets
* parking_lot
    * faster thread synchronisation
* crossfont
    * cross platform font loading
* dirs
    * cross platform directory finding for config storage
* unicode-width
    * gets width of unicode strings in CHARS not bytes

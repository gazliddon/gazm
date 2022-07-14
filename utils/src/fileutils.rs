use std::path::{Path, PathBuf};
use path_clean::PathClean;

fn rel_path<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, base: P2) -> Option<PathBuf> {
    pathdiff::diff_paths(&path,&base)
}

fn abs_path<P1: AsRef<Path>, P2: AsRef<Path>>(path: P1, base: P2) -> PathBuf {
    let path = path.as_ref();
    let base = base.as_ref().to_path_buf();

    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
    .clean();

    abs
}




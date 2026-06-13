use vergen_gix::{Build, Emitter, Gix, Rustc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = Build::all_build();
    let git = Gix::all_git();
    let rustc = Rustc::all_rustc();

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&git)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}

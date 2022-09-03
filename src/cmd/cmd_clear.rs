use crate::{ Deps };

pub fn clear(deps: &Deps) {
    let response = deps.input.confirm(
        format!(
            "Are you sure you want to delete {} and {}?",
            &deps.mem.get_all_file_path(),
            &deps.mem.get_used_file_path()
        )
    );

    if response == true {
        deps.mem.clear_files();
    }
}
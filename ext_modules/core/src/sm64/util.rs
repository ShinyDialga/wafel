use super::{ObjectBehavior, ObjectSlot, SM64ErrorCause, SurfaceSlot};
use crate::{data_path::GlobalDataPath, error::Error, memory::Memory, timeline::State};

/// Get the data path for an object.
pub fn object_path<M: Memory>(
    state: &impl State<Memory = M>,
    object: ObjectSlot,
) -> Result<GlobalDataPath<M>, Error> {
    let active_flags = state
        .read(format!("gObjectPool[{}].activeFlags", object.0))?
        .as_int()?;

    if active_flags == 0 {
        Err(SM64ErrorCause::InactiveObject { object })?;
    }

    let object_path = state
        .memory()
        .global_path(&format!("gObjectPool[{}]", object.0))?;

    Ok(object_path)
}

/// Get the behavior address for an object.
pub fn object_behavior<M: Memory>(
    state: &impl State<Memory = M>,
    object_path: &GlobalDataPath<M>,
) -> Result<ObjectBehavior, Error> {
    let behavior_path =
        object_path.concat(&state.memory().local_path("struct Object.behavior")?)?;
    let behavior_address = state.read(&behavior_path)?.as_address()?;
    Ok(ObjectBehavior::new(behavior_address))
}

/// Get the data path for a surface.
pub fn surface_path<M: Memory>(
    state: &impl State<Memory = M>,
    surface: SurfaceSlot,
) -> Result<GlobalDataPath<M>, Error> {
    let num_surfaces = state.read("gSurfacesAllocated")?.as_int()? as usize;
    if surface.0 >= num_surfaces {
        Err(SM64ErrorCause::InactiveSurface { surface })?;
    }
    let surface_path = state
        .memory()
        .global_path(&format!("sSurfacePool[{}]", surface))?;
    Ok(surface_path)
}

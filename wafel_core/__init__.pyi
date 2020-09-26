from __future__ import annotations

from typing import *


class Pipeline:
  @staticmethod
  def load(dll_path: str) -> Pipeline: ...
  @staticmethod
  def load_reusing_edits(dll_path: str, prev_pipeline: Pipeline) -> Pipeline: ...
  def dump_layout(self) -> str: ...

  def read(self, variable: Variable) -> object: ...
  def write(self, variable: Variable, value: object) -> None: ...
  def reset(self, variable: Variable) -> None: ...

  def path_address(self, frame: int, path: str) -> Optional[Address]: ...
  def path_read(self, frame: int, path: str) -> object: ...

  def insert_frame(self, frame: int) -> None: ...
  def delete_frame(self, frame: int) -> None: ...

  def begin_drag(self, source_variable: Variable, source_value: object) -> None: ...
  def update_drag(self, target_frame: int) -> None: ...
  def release_drag(self) -> None: ...
  def find_edit_range(self, variable: Variable) -> Optional[EditRange]: ...

  def set_hotspot(self, name: str, frame: int) -> None: ...
  def balance_distribution(self, max_run_time_seconds: float) -> None: ...

  def cached_frames(self) -> List[int]: ...
  def num_advances(self) -> int: ...
  def num_copies(self) -> int: ...
  def data_cache_size(self) -> int: ...

  def label(self, variable: Variable) -> Optional[str]: ...
  def is_int(self, variable: Variable) -> bool: ...
  def is_float(self, variable: Variable) -> bool: ...
  def is_bit_flag(self, variable: Variable) -> bool: ...

  def variable_group(self, group: str) -> List[Variable]: ...
  def address_to_base_pointer(self, frame: int, address: Address) -> int: ...
  def field_offset(self, path: str) -> int: ...
  def pointer_or_array_stride(self, path: str) -> Optional[int]: ...
  def action_names(self) -> Dict[int, str]: ...
  def object_behavior(self, frame: int, object: int) -> Optional[ObjectBehavior]: ...
  def object_behavior_name(self, behavior: ObjectBehavior) -> str: ...

  def frame_log(self, frame: int) -> List[Dict[str, Any]]: ...

  def trace_ray_to_surface(
    self, frame: int, ray: Tuple[Iterable[float], Iterable[float]]
  ) -> Optional[int]: ...
  def read_surfaces_to_scene(self, scene: Scene, frame: int) -> None: ...
  def read_objects_to_scene(self, scene: Scene, frame: int) -> None: ...
  def read_mario_path(self, frame_start: int, frame_end: int) -> ObjectPath: ...


class Variable:
  def __init__(self, name: str) -> None: ...
  @staticmethod
  def from_bytes(src: bytes) -> Variable: ...
  def to_bytes(self) -> bytes: ...

  @property
  def name(self) -> str: ...
  @property
  def frame(self) -> Optional[int]: ...
  @property
  def object(self) -> Optional[int]: ...
  @property
  def object_behavior(self) -> Optional[ObjectBehavior]: ...
  @property
  def surface(self) -> Optional[int]: ...

  def with_frame(self, frame: int) -> Variable: ...
  def without_frame(self) -> Variable: ...
  def with_object(self, object: int) -> Variable: ...
  def without_object(self) -> Variable: ...
  def with_object_behavior(self, behavior: ObjectBehavior) -> Variable: ...
  def without_object_behavior(self) -> Variable: ...
  def with_surface(self, surface: int) -> Variable: ...
  def without_surface(self) -> Variable: ...


class ObjectBehavior:
  pass


class Address:
  pass


class EditRange:
  @property
  def id(self) -> int: ...
  @property
  def start(self) -> int: ...
  @property
  def end(self) -> int: ...
  @property
  def value(self) -> object: ...


class Scene:
  viewport: Viewport
  camera: Union[RotateCamera, BirdsEyeCamera]
  show_camera_target: bool
  # surfaces: List[Surface]
  wall_hitbox_radius: float
  hovered_surface: Optional[int]
  hidden_surfaces: Set[int]
  # objects: List[Object]
  object_paths: List[ObjectPath]

class Viewport:
  x: float
  y: float
  width: float
  height: float

class RotateCamera:
  pos: Tuple[float, float, float]
  target: Tuple[float, float, float]
  fov_y: float

  @property
  def pitch(self) -> float: ...
  @property
  def yaw(self) -> float: ...

class BirdsEyeCamera:
  pos: Tuple[float, float, float]
  span_y: float

class ObjectPath:
  # nodes: List[ObjectPathNode]
  root_index: int

  def set_quarter_steps(self, index: int, quarter_steps: List[QuarterStep]) -> None: ...

class QuarterStep:
  intended_pos: Tuple[float, float, float]
  result_pos: Tuple[float, float, float]


def open_window_and_run(
  title: str,
  update_fn: Callable[[], Tuple[object, List[Scene]]],
) -> None:
  ...


class AdjustedStick:
  x: float
  y: float
  mag: float

  def __init__(self, x: float, y: float, mag: float) -> None: ...

class IntendedStick:
  yaw: int
  mag: float

  def __init__(self, yaw: int, mag: float) -> None: ...

def stick_raw_to_adjusted(raw_stick_x: int, raw_stick_y: int) -> AdjustedStick: ...

def stick_adjusted_to_intended(
  adjusted: AdjustedStick,
  face_yaw: int,
  camera_yaw: int,
  squished: bool,
) -> IntendedStick: ...

def stick_adjusted_to_raw_euclidean(
  target_adjusted_x: float,
  target_adjusted_y: float,
) -> Tuple[int, int]: ...

def stick_intended_to_raw_heuristic(
  intended: IntendedStick,
  face_yaw: int,
  camera_yaw: int,
  squished: bool,
  relative_to: int,
) -> Tuple[int, int]:
  ...

from typing import *

from wafel.core.cell_manager import StateSequence, GenericTimeline
from wafel.core.game_state import GameState
from wafel.core.game_lib import GameLib
from wafel.core.edit import Edits
from wafel.core.variable import Variables
from wafel.core.variable_param import VariableParam


class _GameStateSequence(StateSequence[GameState, int]):
  def __init__(
    self,
    lib: GameLib,
    variables: Variables,
    edits: Edits,
  ) -> None:
    self.lib = lib
    self.variables = variables
    self.edits = edits

  def alloc_state(self) -> int:
    return self.lib.state_new()

  def dealloc_state(self, addr: int) -> None:
    self.lib.state_delete(addr)

  def raw_copy_state(self, dst: int, src: int) -> None:
    self.lib.state_raw_copy(dst, src)

  def execute_frame(self, addr: int) -> None:
    self.lib.state_update(addr)

  def to_owned(self, base_addr: int, frame: int, addr: int) -> GameState:
    return GameState(self.lib, base_addr, frame, addr)

  def apply_edits(self, state: GameState) -> None:
    for edit in self.edits.get_edits(state.frame):
      variable = self.variables[edit.variable_id]
      variable.set(edit.value, { VariableParam.STATE: state })

  def get_num_frames(self) -> int:
    return len(self.edits)

  def on_invalidation(self, callback: Callable[[int], None]) -> None:
    self.edits.on_edit(callback)


class Timeline(GenericTimeline[GameState, int]):
  def __init__(
    self,
    lib: GameLib,
    variables: Variables,
    edits: Edits,
  ) -> None:
    super().__init__(_GameStateSequence(lib, variables, edits))

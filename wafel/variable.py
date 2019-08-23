from typing import *
from enum import Enum, auto

from wafel.util import *


class VariableParam(Enum):
  STATE = auto()


class VariableSemantics(Enum):
  RAW = auto()
  POSITION = auto()
  VELOCITY = auto()
  ANGLE = auto()
  FLAG = auto()
  MARIO_ACTION = auto()


class VariableDataType(Enum):
  BOOL = auto()
  S8 = auto()
  U8 = auto()
  S16 = auto()
  U16 = auto()
  S32 = auto()
  U32 = auto()
  S64 = auto()
  U64 = auto()
  F32 = auto()
  F64 = auto()
  VEC3F = auto()

  @staticmethod
  def from_spec(spec: dict, type_: dict) -> 'VariableDataType':
    type_ = concrete_type(spec, type_)
    if type_['kind'] == 'primitive':
      return {
        's8': VariableDataType.S8,
        'u8': VariableDataType.U8,
        's16': VariableDataType.S16,
        'u16': VariableDataType.U16,
        's32': VariableDataType.S32,
        'u32': VariableDataType.U32,
        's64': VariableDataType.S64,
        'u64': VariableDataType.U64,
        'f32': VariableDataType.F32,
        'f64': VariableDataType.F64,
      }[type_['name']]
    elif type_['kind'] == 'array':
      elem_type = VariableDataType.from_spec(spec, type_['base'])
      if type_['length'] == 3 and elem_type == VariableDataType.F32:
        return VariableDataType.VEC3F
      else:
        raise NotImplementedError(type_)
    else:
      raise NotImplementedError(type_['kind'])


class Variable:
  def __init__(
    self,
    name: str,
    params: List[VariableParam],
    semantics: VariableSemantics,
    read_only: bool,
    data_type: VariableDataType,
  ) -> None:
    self.name = name
    self.params = params
    self.semantics = semantics
    self.read_only = read_only
    self.data_type = data_type

  def get(self, *args: Any) -> Any:
    raise NotImplementedError

  def set(self, value: Any, *args: Any) -> None:
    raise NotImplementedError


class Variables:
  def __init__(self, variables: List[Variable]) -> None:
    # TODO: Formatting settings
    self.variables = variables
    self.by_name = {var.name: var for var in self.variables}
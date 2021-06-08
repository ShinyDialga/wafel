use std::collections::HashMap;

use wafel_data_type::{
    Address, DataType, DataTypeRef, FloatType, FloatValue, IntType, IntValue, TypeName, Value,
};

use crate::MemoryError::{self, *};

pub trait SymbolLookup {
    fn symbol_address(&self, symbol: &str) -> Option<Address>;
}

pub trait MemoryRead {
    fn read_int(&self, address: Address, int_type: IntType) -> Result<IntValue, MemoryError>;
    fn read_float(
        &self,
        address: Address,
        float_type: FloatType,
    ) -> Result<FloatValue, MemoryError>;
    fn read_address(&self, address: Address) -> Result<Address, MemoryError>;

    fn read_value(
        &self,
        address: Address,
        data_type: &DataTypeRef,
        mut resolve_type: impl FnMut(&TypeName) -> Option<DataTypeRef>,
    ) -> Result<Value, MemoryError> {
        read_value_impl(self, address, data_type, &mut resolve_type)
    }
}

fn read_value_impl<M: MemoryRead + ?Sized>(
    memory: &M,
    address: Address,
    data_type: &DataTypeRef,
    resolve_type: &mut impl FnMut(&TypeName) -> Option<DataTypeRef>,
) -> Result<Value, MemoryError> {
    let value = match data_type.as_ref() {
        DataType::Void => Value::Null,
        DataType::Int(int_type) => Value::Int(memory.read_int(address, *int_type)?),
        DataType::Float(float_type) => Value::Float(memory.read_float(address, *float_type)?),
        DataType::Pointer { .. } => Value::Address(memory.read_address(address)?),
        DataType::Array {
            base,
            length,
            stride,
        } => match *length {
            Some(length) => {
                let values: Vec<Value> = (0..length)
                    .map(|index| {
                        read_value_impl(memory, address + index * *stride, base, resolve_type)
                    })
                    .collect::<Result<_, MemoryError>>()?;
                Value::Array(values)
            }
            None => return Err(ReadUnsizedArray),
        },
        DataType::Struct { fields } => {
            let mut field_values: HashMap<String, Value> = HashMap::new();
            for (name, field) in fields {
                let field_value = read_value_impl(
                    memory,
                    address + field.offset,
                    &field.data_type,
                    resolve_type,
                )?;
                field_values.insert(name.clone(), field_value);
            }
            Value::Struct {
                fields: Box::new(field_values),
            }
        }
        DataType::Union { .. } => return Err(ReadUnion),
        DataType::Name(type_name) => {
            let resolved_type =
                resolve_type(type_name).ok_or_else(|| UndefinedTypeName(type_name.clone()))?;
            read_value_impl(memory, address, &resolved_type, resolve_type)?
        }
    };
    Ok(value)
}

pub trait MemoryWrite {
    fn write_int(
        &mut self,
        address: Address,
        int_type: IntType,
        value: IntValue,
    ) -> Result<(), MemoryError>;
    fn write_float(
        &mut self,
        address: Address,
        float_type: FloatType,
        value: FloatValue,
    ) -> Result<(), MemoryError>;
    fn write_address(&mut self, address: Address, value: Address) -> Result<(), MemoryError>;

    fn write_value(
        &mut self,
        address: Address,
        data_type: &DataTypeRef,
        value: Value,
        mut resolve_type: impl FnMut(&TypeName) -> Option<DataTypeRef>,
    ) -> Result<(), MemoryError> {
        write_value_impl(self, address, data_type, value, &mut resolve_type)
    }
}

fn write_value_impl<M: MemoryWrite + ?Sized>(
    memory: &mut M,
    address: Address,
    data_type: &std::sync::Arc<DataType>,
    value: Value,
    resolve_type: &mut impl FnMut(&TypeName) -> Option<std::sync::Arc<DataType>>,
) -> Result<(), MemoryError> {
    match data_type.as_ref() {
        DataType::Void => value.as_null()?,
        DataType::Int(int_type) => memory.write_int(address, *int_type, value.as_int()?)?,
        DataType::Float(float_type) => {
            memory.write_float(address, *float_type, value.as_float()?)?
        }
        DataType::Pointer { .. } => memory.write_address(address, value.as_address()?)?,
        DataType::Array {
            base,
            length,
            stride,
        } => {
            let elements = match *length {
                Some(length) => value.as_array_with_len(length)?,
                None => value.as_array()?,
            };
            for (i, element) in elements.iter().enumerate() {
                write_value_impl(
                    memory,
                    address + i * *stride,
                    base,
                    element.clone(),
                    resolve_type,
                )?;
            }
        }
        DataType::Struct { fields } => {
            let field_values = value.as_struct()?;
            for name in field_values.keys() {
                if !fields.contains_key(name) {
                    return Err(WriteExtraField(name.clone()));
                }
            }
            for name in fields.keys() {
                if !field_values.contains_key(name) {
                    return Err(WriteExtraField(name.clone()));
                }
            }
            for (field_name, field) in fields {
                let field_value = field_values[field_name].clone();
                write_value_impl(
                    memory,
                    address + field.offset,
                    &field.data_type,
                    field_value,
                    resolve_type,
                )?;
            }
        }
        DataType::Union { fields: _ } => return Err(WriteUnion),
        DataType::Name(type_name) => {
            let resolved_type =
                resolve_type(type_name).ok_or_else(|| UndefinedTypeName(type_name.clone()))?;
            write_value_impl(memory, address, &resolved_type, value, resolve_type)?
        }
    }
    Ok(())
}

pub trait SlottedMemory {
    type Slot;

    type StaticView<'a>: MemoryRead;
    type SlotView<'a>: MemoryRead;
    type SlotViewMut<'a>: MemoryRead + MemoryWrite;

    fn static_view(&self) -> Self::StaticView<'_>;
    fn with_slot<'a>(&'a self, slot: &'a Self::Slot) -> Self::SlotView<'a>;
    fn with_slot_mut<'a>(&'a self, slot: &'a mut Self::Slot) -> Self::SlotViewMut<'a>;

    fn create_backup_slot(&self) -> Self::Slot;
    fn copy_slot(&self, dst: &mut Self::Slot, src: &Self::Slot);
    fn advance_base_slot(&self, base_slot: &mut Self::Slot);
}

pub trait MemoryReadPrimitive {
    /// Read a primitive value from memory.
    ///
    /// # Safety
    ///
    /// T must be an integral, float, or raw pointer type.
    unsafe fn read_primitive<T: Copy>(&self, address: Address) -> Result<T, MemoryError>;
}

pub trait MemoryWritePrimitive {
    /// Write a primitive value to memory.
    ///
    /// # Safety
    ///
    /// T must be an integral, float, or raw pointer type.
    unsafe fn write_primitive<T: Copy>(
        &mut self,
        address: Address,
        value: T,
    ) -> Result<(), MemoryError>;
}

impl<M: MemoryReadPrimitive> MemoryRead for M {
    fn read_int(&self, address: Address, int_type: IntType) -> Result<IntValue, MemoryError> {
        unsafe {
            Ok(match int_type {
                IntType::U8 => (self.read_primitive::<u8>(address)?).into(),
                IntType::S8 => (self.read_primitive::<i8>(address)?).into(),
                IntType::U16 => (self.read_primitive::<u16>(address)?).into(),
                IntType::S16 => (self.read_primitive::<i16>(address)?).into(),
                IntType::U32 => (self.read_primitive::<u32>(address)?).into(),
                IntType::S32 => (self.read_primitive::<i32>(address)?).into(),
                IntType::U64 => (self.read_primitive::<u64>(address)?).into(),
                IntType::S64 => (self.read_primitive::<i64>(address)?).into(),
            })
        }
    }

    fn read_float(
        &self,
        address: Address,
        float_type: FloatType,
    ) -> Result<FloatValue, MemoryError> {
        unsafe {
            Ok(match float_type {
                FloatType::F32 => (self.read_primitive::<f32>(address)?).into(),
                FloatType::F64 => (self.read_primitive::<f64>(address)?),
            })
        }
    }

    fn read_address(&self, address: Address) -> Result<Address, MemoryError> {
        unsafe {
            let pointer = self.read_primitive::<*const u8>(address)?;
            Ok(Address(pointer as usize))
        }
    }
}

impl<M: MemoryWritePrimitive> MemoryWrite for M {
    fn write_int(
        &mut self,
        address: Address,
        int_type: IntType,
        value: IntValue,
    ) -> Result<(), MemoryError> {
        unsafe {
            match int_type {
                IntType::U8 => self.write_primitive(address, value as u8),
                IntType::S8 => self.write_primitive(address, value as i8),
                IntType::U16 => self.write_primitive(address, value as u16),
                IntType::S16 => self.write_primitive(address, value as i16),
                IntType::U32 => self.write_primitive(address, value as u32),
                IntType::S32 => self.write_primitive(address, value as i32),
                IntType::U64 => self.write_primitive(address, value as u64),
                IntType::S64 => self.write_primitive(address, value as i64),
            }
        }
    }

    fn write_float(
        &mut self,
        address: Address,
        float_type: FloatType,
        value: FloatValue,
    ) -> Result<(), MemoryError> {
        unsafe {
            match float_type {
                FloatType::F32 => self.write_primitive(address, value as f32),
                FloatType::F64 => self.write_primitive(address, value as f64),
            }
        }
    }

    fn write_address(&mut self, address: Address, value: Address) -> Result<(), MemoryError> {
        unsafe { self.write_primitive(address, value.0 as *const u8) }
    }
}

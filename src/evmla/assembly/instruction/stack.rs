//!
//! Translates the stack memory operations.
//!

use inkwell::values::BasicValue;

///
/// Translates the ordinar value push.
///
pub fn push<'ctx, D>(
    context: &mut compiler_llvm_context::Context<'ctx, D>,
    value: String,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: compiler_llvm_context::Dependency,
{
    let result = context
        .field_type()
        .const_int_from_string(value.as_str(), inkwell::types::StringRadix::Hexadecimal)
        .expect("Always valid")
        .as_basic_value_enum();
    Ok(result)
}

///
/// Translates the block tag label push.
///
pub fn push_tag<'ctx, D>(
    context: &mut compiler_llvm_context::Context<'ctx, D>,
    value: String,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: compiler_llvm_context::Dependency,
{
    let result = context
        .field_type()
        .const_int_from_string(value.as_str(), inkwell::types::StringRadix::Decimal)
        .expect("Always valid");
    Ok(result.as_basic_value_enum())
}

///
/// Translates the stack memory duplicate.
///
pub fn dup<'ctx, D>(
    context: &mut compiler_llvm_context::Context<'ctx, D>,
    offset: usize,
    height: usize,
    original: &mut Option<String>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: compiler_llvm_context::Dependency,
{
    let element = &context.evmla().stack[height - offset - 1];
    let value = context.build_load(
        compiler_llvm_context::Pointer::new_stack_field(
            context,
            element.to_llvm().into_pointer_value(),
        ),
        format!("dup{offset}").as_str(),
    );

    *original = element.original.to_owned();

    Ok(value)
}

///
/// Translates the stack memory swap.
///
pub fn swap<D>(
    context: &mut compiler_llvm_context::Context<D>,
    offset: usize,
    height: usize,
) -> anyhow::Result<()>
where
    D: compiler_llvm_context::Dependency,
{
    let top_element = context.evmla().stack[height - 1].to_owned();
    let top_pointer = compiler_llvm_context::Pointer::new_stack_field(
        context,
        top_element.to_llvm().into_pointer_value(),
    );
    let top_value = context.build_load(top_pointer, format!("swap{offset}_top_value").as_str());

    let swap_element = context.evmla().stack[height - offset - 1].to_owned();
    let swap_pointer = compiler_llvm_context::Pointer::new_stack_field(
        context,
        swap_element.to_llvm().into_pointer_value(),
    );
    let swap_value = context.build_load(swap_pointer, format!("swap{offset}_swap_value").as_str());

    context.evmla_mut().stack[height - 1].original = swap_element.original.to_owned();
    context.evmla_mut().stack[height - offset - 1].original = top_element.original.to_owned();

    context.build_store(top_pointer, swap_value);
    context.build_store(swap_pointer, top_value);

    Ok(())
}

///
/// Translates the stack memory pop.
///
pub fn pop<D>(_context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()>
where
    D: compiler_llvm_context::Dependency,
{
    Ok(())
}

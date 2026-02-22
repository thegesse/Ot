use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{BasicValueEnum, PointerValue, FunctionValue};
use inkwell::IntPredicate;
use std::collections::HashMap;
use std::cell::RefCell;
use crate::ast::{Statement, Expression, Op};

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub variables: RefCell<HashMap<String, PointerValue<'ctx>>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn compile(&self, ast: Vec<Statement>) {
        let i32_type = self.context.i32_type();
        let main_fn_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, None);

        let entry = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        // Standard library setup
        let i8_ptr = self.context.i8_type().ptr_type(inkwell::AddressSpace::from(0));
        let printf_type = i32_type.fn_type(&[i8_ptr.into()], true);
        self.module.add_function("printf", printf_type, None);

        for stmt in ast {
            self.compile_statement(stmt, main_fn);
        }

        self.builder.build_return(Some(&i32_type.const_int(0, false))).unwrap();
    }

    fn compile_statement(&self, stmt: Statement, current_fn: FunctionValue<'ctx>) {
        match stmt {
            Statement::Manifestation { name, value, .. } => {
                let val = self.compile_expression(value);

                let mut vars = self.variables.borrow_mut();
                let ptr = if let Some(&existing_ptr) = vars.get(&name) {
                    existing_ptr // Use the old spot in memory
                } else {
                    let new_ptr = self.builder.build_alloca(val.get_type(), &name).unwrap();
                    vars.insert(name.clone(), new_ptr);
                    new_ptr
                };

                self.builder.build_store(ptr, val).unwrap();
            }

            Statement::ManifestKnowledge(expr) => {
                let val = self.compile_expression(expr);
                let printf = self.module.get_function("printf").unwrap();

                // 1. Detect if we are printing a String (Pointer) or an Integer
                let fmt = if val.is_pointer_value() { "%s\n" } else { "%d\n" };
                
                // 2. Create the appropriate format string
                let format_str = self.builder.build_global_string_ptr(fmt, "fmt").unwrap();

                // 3. Call printf with the format string and the value
                self.builder.build_call(
                    printf,
                    &[format_str.as_pointer_value().into(), val.into()], 
                    "tmp"
                ).unwrap();
            }

            //If/Elif/Else
            Statement::Conditional { if_block, elifs, else_block } => {
                let (condition, body) = if_block;
                let cond_val = self.compile_expression(condition).into_int_value();
                
                let then_bb = self.context.append_basic_block(current_fn, "then");
                let else_bb = self.context.append_basic_block(current_fn, "else");
                let merge_bb = self.context.append_basic_block(current_fn, "if_merge");

                let zero = self.context.i32_type().const_int(0, false);
                let is_nonzero = self.builder.build_int_compare(IntPredicate::NE, cond_val, zero, "ifcond").unwrap();
                self.builder.build_conditional_branch(is_nonzero, then_bb, else_bb).unwrap();

                // Build 'Then'
                self.builder.position_at_end(then_bb);
                for s in body { self.compile_statement(s, current_fn); }
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                // Build 'Else'
                self.builder.position_at_end(else_bb);
                if let Some(eb) = else_block {
                    for s in eb { self.compile_statement(s, current_fn); }
                }
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                self.builder.position_at_end(merge_bb);
            }

            //For Loop)
            Statement::RecursiveProtocol { init, condition, step, body } => {
                // 1. Initializer
                self.compile_statement(*init, current_fn);

                let header_bb = self.context.append_basic_block(current_fn, "loop_header");
                let body_bb = self.context.append_basic_block(current_fn, "loop_body");
                let after_bb = self.context.append_basic_block(current_fn, "after_loop");

                self.builder.build_unconditional_branch(header_bb).unwrap();
                self.builder.position_at_end(header_bb);

                //Condition Check
                let cond_val = self.compile_expression(condition).into_int_value();
                let zero = self.context.i32_type().const_int(0, false);
                let is_truthy = self.builder.build_int_compare(IntPredicate::NE, cond_val, zero, "loopcond").unwrap();
                self.builder.build_conditional_branch(is_truthy, body_bb, after_bb).unwrap();

                //Loop Body
                self.builder.position_at_end(body_bb);
                for s in body { self.compile_statement(s, current_fn); }
                
                //Step
                self.compile_statement(*step, current_fn);
                self.builder.build_unconditional_branch(header_bb).unwrap();

                self.builder.position_at_end(after_bb);
            }
        }
    }

    fn compile_expression(&self, expr: Expression) -> BasicValueEnum<'ctx> {
        match expr {
            Expression::LiteralInt(v) => self.context.i32_type().const_int(v as u64, false).into(),
            Expression::LiteralString(s) => {
            // This creates the string in memory and returns a pointer to the first character
            self.builder.build_global_string_ptr(&s, "str_lit")
                .unwrap()
                .as_pointer_value()
                .into()
        }
            Expression::Variable(name) => {
                let vars = self.variables.borrow();
                let ptr = vars.get(&name).expect("IDENTIFIER UNKNOWN");
                

                let ptr_type = ptr.get_type();
                let load_type = self.builder.get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap()
                    .get_type();
                let val = self.builder.build_load(self.context.i32_type(), *ptr, &name).unwrap();
                val.into()
            }
            Expression::BinaryOp(left, op, right) => {
                let lhs = self.compile_expression(*left).into_int_value();
                let rhs = self.compile_expression(*right).into_int_value();
                match op {
                    Op::Add => self.builder.build_int_add(lhs, rhs, "tmp").unwrap().into(),
                    Op::Sub => self.builder.build_int_sub(lhs, rhs, "tmp").unwrap().into(),
                    Op::Mul => self.builder.build_int_mul(lhs, rhs, "tmp").unwrap().into(),
                    Op::Div => self.builder.build_int_signed_div(lhs, rhs, "tmp").unwrap().into(),
                }
            }
            _ => todo!(),
        }
    }
}
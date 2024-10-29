#[macro_export]
macro_rules! impl_binary_op {
    ($registers:expr, $dest: expr, $lhs:expr, $x:tt, $rhs:expr) => {
        match (&$registers[*$lhs as usize], &$registers[*$rhs as usize]) {
            (RegisterValue::Literal(lhs), RegisterValue::Literal(rhs)) => {
                let lhs = lhs.as_ref();
                let rhs = rhs.as_ref();

                match (lhs, rhs) {
                    (ast::Literal::Float(lhs), ast::Literal::Float(rhs)) => {
                        $registers[*$dest as usize] =
                            RegisterValue::Literal(Cow::Owned(ast::Literal::Float(lhs $x rhs)))
                    }
                    (ast::Literal::Float(lhs), ast::Literal::Integer(rhs)) => {
                        $registers[*$dest as usize] = RegisterValue::Literal(Cow::Owned(
                            ast::Literal::Float(*lhs $x *rhs as f64),
                        ))
                    }
                    (ast::Literal::Integer(lhs), ast::Literal::Float(rhs)) => {
                        $registers[*$dest as usize] = RegisterValue::Literal(Cow::Owned(
                            ast::Literal::Float(*lhs as f64 $x *rhs),
                        ))
                    }
                    (ast::Literal::Integer(lhs), ast::Literal::Integer(rhs)) => {
                        $registers[*$dest as usize] =
                            RegisterValue::Literal(Cow::Owned(ast::Literal::Integer(lhs $x rhs)))
                    }

                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    };
}

#[macro_export]
macro_rules! impl_binary_comparator {
    ($registers:expr, $dest: expr, $lhs:expr, $x:tt, $rhs:expr) => {
        {
            let lhs = &$registers[*$lhs as usize];
            let rhs = &$registers[*$rhs as usize];

            let is_equal = lhs $x rhs;

            $registers[*$dest as usize] =
                RegisterValue::Literal(Cow::Owned(ast::Literal::Boolean(is_equal)))
        }
    }
}

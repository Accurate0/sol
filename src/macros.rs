#[macro_export]
macro_rules! impl_binary_op {
    ($registers:expr, $dest: expr, $lhs:expr, $x:tt, $rhs:expr) => {
        match (&$registers[$lhs], &$registers[$rhs]) {
            (VMValue::Literal(lhs), VMValue::Literal(rhs)) => {
                let lhs = lhs.as_ref();
                let rhs = rhs.as_ref();

                match (lhs, rhs) {
                    (types::Literal::Float(lhs), types::Literal::Float(rhs)) => {
                        $registers[$dest] =
                            VMValue::Literal(Cow::Owned(types::Literal::Float(lhs $x rhs)))
                    }
                    (types::Literal::Float(lhs), types::Literal::Integer(rhs)) => {
                        $registers[$dest] = VMValue::Literal(Cow::Owned(
                            types::Literal::Float(*lhs $x *rhs as f64),
                        ))
                    }
                    (types::Literal::Integer(lhs), types::Literal::Float(rhs)) => {
                        $registers[$dest] = VMValue::Literal(Cow::Owned(
                            types::Literal::Float(*lhs as f64 $x *rhs),
                        ))
                    }
                    (types::Literal::Integer(lhs), types::Literal::Integer(rhs)) => {
                        $registers[$dest] =
                            VMValue::Literal(Cow::Owned(types::Literal::Integer(lhs $x rhs)))
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
            let lhs = &$registers[$lhs];
            let rhs = &$registers[$rhs];

            let is_equal = lhs $x rhs;

            $registers[$dest] =
                VMValue::Literal(Cow::Owned(types::Literal::Boolean(is_equal)))
        }
    }
}

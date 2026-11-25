#![allow(unused)]

use std::any::Any;


pub trait RypustType {
    fn as_any(&self) -> &dyn Any;
    fn __add__(&self, other: &Self) -> Self where Self: Sized;
    fn __subtract__(&self, other: &Self) -> Self where Self: Sized;
    fn __multiple__(&self, other: &Self) -> Self where Self: Sized;
    fn __division__(&self, other: &Self) -> Self where Self: Sized;
    fn __mod__(&self, other: &Self) -> Self where Self: Sized;
    fn __eq__(&self, other: &dyn RypustType) -> bool;
    fn __noteq__(&self, other: &dyn RypustType) -> bool;
    fn __lessth__(&self, other: &Self) -> bool where Self: Sized;
    fn __lesseq__(&self, other: &Self) -> bool where Self: Sized;
    fn __moreth__(&self, other: &Self) -> bool where Self: Sized;
    fn __moreeq__(&self, other: &Self) -> bool where Self: Sized;
    fn to_str(&self) -> String;
}
/// int type for RYPUST language
pub struct intType {
    value: i64
}

impl RypustType for intType {
    fn as_any(&self) -> &dyn Any {
        self
    }
    /// Return self + value
    /// # example
    /// ```rust
    /// let first = intType { value: 14 }
    /// let second = intType { value: 2 }
    /// println!(first.__add__(second))
    /// ```
    /// ==> 16
    fn __add__(&self, other: &intType) -> intType {
        return intType { value: self.value + other.value };
    }

    /// Return self - value
    /// # example
    /// ```rust
    /// let first = intType { value: 12 }
    /// let second = intType { value: 10 }
    /// println!(first.__subtract__(second))
    /// ```
    /// ==> 2
    fn __subtract__(&self, other: &intType) -> intType {
        return intType { value: self.value - other.value };
    }

    /// return self * value
    /// # example
    /// ```rust
    /// let first = intType { value: 3 }
    /// let second = intType { value: 8 }
    /// println!(first.__multiple__(second))
    /// ```
    /// ==> 24
    fn __multiple__(&self, other: &intType) -> intType {
        return intType { value: self.value * other.value };
    }

    /// return self / value
    /// # example
    /// ```rust
    /// let first = intType { value: 14 }
    /// let second = intType { value: 2 }
    /// println!(first.__division__(second))
    /// ```
    /// ==> 7
    fn __division__(&self, other: &intType) -> intType {
        return intType { value: self.value / other.value };
    }

    /// return self % other
    /// # example
    /// ```rust
    /// let first = intType { value: 14 }
    /// let second = intType { value: 2 }
    /// println!(first.__mod__(second))
    /// ```
    /// ==> 0
    fn __mod__(&self, other: &intType) -> intType {
        return intType { value: self.value % other.value };
    }

    /// return string representation of number
    fn to_str(&self) -> String {
        return self.value.to_string();
    }

    // * Region operation

    /// return result of equivalent
    fn __eq__(&self, other: &dyn RypustType) -> bool {
        if let Some(other_int) = other.as_any().downcast_ref::<intType>() {
            return self.value == other_int.value;
        } else {
            return false;
        }
    }

    /// return result of negative equivalent
    fn __noteq__(&self, other: &dyn RypustType) -> bool {
        if let Some(other_int) = other.as_any().downcast_ref::<intType>() {
            return self.value != other_int.value;
        } else {
            return false;
        }
    }

    /// return result of self < other
    fn __lessth__(&self, other: &intType) -> bool {
        return self.value < other.value;
    }

    /// return result of self <= other
    fn __lesseq__(&self, other: &intType) -> bool {
        return self.value <= other.value;
    }

    /// return result of self > other
    fn __moreth__(&self, other: &intType) -> bool {
        return self.value > other.value;
    }

    /// return result of self >= other
    fn __moreeq__(&self, other: &intType) -> bool {
        return self.value >= other.value;
    }
}
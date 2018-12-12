use super::super::pyobject::{
    AttributeProtocol, PyContext, PyFuncArgs, PyObjectKind, PyObjectRef, PyResult, TypeProtocol,
};
use super::super::vm::VirtualMachine;
use super::objint;
use super::objsequence::PySliceableSequence;
use super::objtype;
use num_bigint::ToBigInt;
use num_traits::ToPrimitive;
use std::hash::{Hash, Hasher};

// functions like isdigit also checks if exponents are digits
// exponents from 0-9
const VALID_UNICODES: &'static [&'static u16; 10] = &[
    &0x2070, &0x00B9, &0x00B2, &0x00B3, &0x2074, &0x2075, &0x2076, &0x2077, &0x2078, &0x2079,
];

pub fn init(context: &PyContext) {
    let ref str_type = context.str_type;
    str_type.set_attr("__add__", context.new_rustfunc(str_add));
    str_type.set_attr("__eq__", context.new_rustfunc(str_eq));
    str_type.set_attr("__contains__", context.new_rustfunc(str_contains));
    str_type.set_attr("__getitem__", context.new_rustfunc(str_getitem));
    str_type.set_attr("__gt__", context.new_rustfunc(str_gt));
    str_type.set_attr("__hash__", context.new_rustfunc(str_hash));
    str_type.set_attr("__len__", context.new_rustfunc(str_len));
    str_type.set_attr("__mul__", context.new_rustfunc(str_mul));
    str_type.set_attr("__new__", context.new_rustfunc(str_new));
    str_type.set_attr("__str__", context.new_rustfunc(str_str));
    str_type.set_attr("__repr__", context.new_rustfunc(str_repr));
    str_type.set_attr("lower", context.new_rustfunc(str_lower));
    str_type.set_attr("upper", context.new_rustfunc(str_upper));
    str_type.set_attr("capitalize", context.new_rustfunc(str_capitalize));
    str_type.set_attr("split", context.new_rustfunc(str_split));
    str_type.set_attr("strip", context.new_rustfunc(str_strip));
    str_type.set_attr("lstrip", context.new_rustfunc(str_lstrip));
    str_type.set_attr("rstrip", context.new_rustfunc(str_rstrip));
    str_type.set_attr("endswith", context.new_rustfunc(str_endswith));
    str_type.set_attr("startswith", context.new_rustfunc(str_startswith));
    str_type.set_attr("title", context.new_rustfunc(str_title));
    str_type.set_attr("swapcase", context.new_rustfunc(str_swapcase));
    str_type.set_attr("isalnum", context.new_rustfunc(str_isalnum));
    str_type.set_attr("isalpha", context.new_rustfunc(str_isalpha));
    str_type.set_attr("isdigit", context.new_rustfunc(str_isdigit));

    // str_type.set_attr("center", context.new_rustfunc(str_center));
}

pub fn get_value(obj: &PyObjectRef) -> String {
    if let PyObjectKind::String { value } = &obj.borrow().kind {
        value.to_string()
    } else {
        panic!("Inner error getting str");
    }
}

fn str_eq(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(a, Some(vm.ctx.str_type())), (b, None)]
    );

    let result = if objtype::isinstance(b, &vm.ctx.str_type()) {
        get_value(a) == get_value(b)
    } else {
        false
    };
    Ok(vm.ctx.new_bool(result))
}

fn str_gt(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [
            (zelf, Some(vm.ctx.str_type())),
            (other, Some(vm.ctx.str_type()))
        ]
    );
    let zelf = get_value(zelf);
    let other = get_value(other);
    let result = zelf > other;
    Ok(vm.ctx.new_bool(result))
}

fn str_str(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    Ok(s.clone())
}

fn count_char(s: &str, c: char) -> usize {
    s.chars().filter(|x| *x == c).count()
}

fn str_repr(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(s);
    let quote_char = if count_char(&value, '\'') > count_char(&value, '"') {
        '"'
    } else {
        '\''
    };
    let mut formatted = String::new();
    formatted.push(quote_char);
    for c in value.chars() {
        if c == quote_char || c == '\\' {
            formatted.push('\\');
            formatted.push(c);
        } else if c == '\n' {
            formatted.push('\\');
            formatted.push('n');
        } else if c == '\t' {
            formatted.push('\\');
            formatted.push('t');
        } else if c == '\r' {
            formatted.push('\\');
            formatted.push('r');
        } else {
            formatted.push(c);
        }
    }
    formatted.push(quote_char);
    Ok(vm.ctx.new_str(formatted))
}

fn str_add(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(s, Some(vm.ctx.str_type())), (s2, None)]
    );
    if objtype::isinstance(s2, &vm.ctx.str_type()) {
        Ok(vm
            .ctx
            .new_str(format!("{}{}", get_value(&s), get_value(&s2))))
    } else {
        Err(vm.new_type_error(format!("Cannot add {:?} and {:?}", s, s2)))
    }
}

fn str_hash(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(zelf, Some(vm.ctx.str_type()))]);
    let value = get_value(zelf);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(vm.ctx.new_int(hash.to_bigint().unwrap()))
}

fn str_len(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let sv = get_value(s);
    Ok(vm.ctx.new_int(sv.len().to_bigint().unwrap()))
}

fn str_mul(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(s, Some(vm.ctx.str_type())), (s2, None)]
    );
    if objtype::isinstance(s2, &vm.ctx.int_type()) {
        let value1 = get_value(&s);
        let value2 = objint::get_value(s2).to_i32().unwrap();
        let mut result = String::new();
        for _x in 0..value2 {
            result.push_str(value1.as_str());
        }
        Ok(vm.ctx.new_str(result))
    } else {
        Err(vm.new_type_error(format!("Cannot multiply {:?} and {:?}", s, s2)))
    }
}

fn str_upper(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s).to_uppercase();
    Ok(vm.ctx.new_str(value))
}

fn str_lower(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s).to_lowercase();
    Ok(vm.ctx.new_str(value))
}

fn str_capitalize(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s);
    let (first_part, lower_str) = value.split_at(1);
    let capitalized = format!("{}{}", first_part.to_uppercase().to_string(), lower_str);
    Ok(vm.ctx.new_str(capitalized))
}

fn str_split(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(s, Some(vm.ctx.str_type())), (pat, Some(vm.ctx.str_type()))]
    );
    let value = get_value(&s);
    // if some
    let pat = get_value(&pat);
    let str_pat = pat.as_str();
    let elements = value
        .split(str_pat)
        .map(|o| vm.ctx.new_str(o.to_string()))
        .collect();
    Ok(vm.ctx.new_list(elements))
}

fn str_strip(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s).trim().to_string();
    Ok(vm.ctx.new_str(value))
}

fn str_lstrip(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s).trim_left().to_string();
    Ok(vm.ctx.new_str(value))
}

fn str_rstrip(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s).trim_right().to_string();
    Ok(vm.ctx.new_str(value))
}

fn str_endswith(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(s, Some(vm.ctx.str_type())), (pat, Some(vm.ctx.str_type()))]
    );
    let value = get_value(&s);
    let pat = get_value(&pat);
    Ok(vm.ctx.new_bool(value.ends_with(pat.as_str())))
}

fn str_swapcase(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s);
    let mut swapped_str = String::with_capacity(value.len());
    for c in value.chars() {
        if c.is_lowercase() {
            swapped_str.push(c.to_ascii_uppercase());
        } else if c.is_uppercase() {
            swapped_str.push(c.to_ascii_lowercase());
        } else {
            swapped_str.push(c);
        }
    }
    Ok(vm.ctx.new_str(swapped_str))
}

fn str_title(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s);
    let titled_str = value
        .split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f
                    .to_uppercase()
                    .chain(c.flat_map(|t| t.to_lowercase()))
                    .collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok(vm.ctx.new_str(titled_str))
}

// fn str_center(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
//     arg_check!(
//         vm,
//         args,
//         required = [(s, Some(vm.ctx.str_type())), (len, Some(vm.ctx.int_type()))],
//         optional = [(chars, None)]
//     );
//     let value = get_value(&s);
//     let len = get_value(&len).parse::<usize>();
//     let chars = args.get_kwargs
//     Ok(vm.ctx.new_str(value))
// }

fn str_startswith(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(s, Some(vm.ctx.str_type())), (pat, Some(vm.ctx.str_type()))]
    );
    let value = get_value(&s);
    let pat = get_value(&pat);
    Ok(vm.ctx.new_bool(value.starts_with(pat.as_str())))
}

fn str_contains(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [
            (s, Some(vm.ctx.str_type())),
            (needle, Some(vm.ctx.str_type()))
        ]
    );
    let value = get_value(&s);
    let needle = get_value(&needle);
    Ok(vm.ctx.new_bool(value.contains(needle.as_str())))
}

fn str_isalnum(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s);
    let mut is_alnum: bool = true;
    for c in value.chars() {
        if !c.is_alphanumeric() {
            is_alnum = false;
            break;
        }
    }
    Ok(vm.ctx.new_bool(is_alnum))
}

fn str_isalpha(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s);
    let mut is_alpha: bool = true;
    for c in value.chars() {
        if !c.is_alphabetic() {
            is_alpha = false;
            break;
        }
    }
    Ok(vm.ctx.new_bool(is_alpha))
}

fn str_isdigit(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(s, Some(vm.ctx.str_type()))]);
    let value = get_value(&s);
    let mut is_digit: bool = true;
    for c in value.chars() {
        if !c.is_digit(10) {
            // checking if char is exponent
            let char_as_uni: u16 = c as u16;
            if VALID_UNICODES.contains(&&char_as_uni) {
                continue;
            } else {
                is_digit = false;
                break;
            }
        }
    }
    Ok(vm.ctx.new_bool(is_digit))
}

fn str_getitem(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(s, Some(vm.ctx.str_type())), (needle, None)]
    );
    let value = get_value(&s);
    subscript(vm, &value, needle.clone())
}

// TODO: should with following format
// class str(object='')
// class str(object=b'', encoding='utf-8', errors='strict')
fn str_new(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    if args.args.len() == 1 {
        return Ok(vm.new_str("".to_string()));
    }

    if args.args.len() > 2 {
        panic!("str expects exactly one parameter");
    };

    vm.to_str(&args.args[1])
}

impl PySliceableSequence for String {
    fn do_slice(&self, start: usize, stop: usize) -> Self {
        self[start..stop].to_string()
    }
    fn do_stepped_slice(&self, start: usize, stop: usize, step: usize) -> Self {
        self[start..stop].chars().step_by(step).collect()
    }
    fn len(&self) -> usize {
        self.len()
    }
}

pub fn subscript(vm: &mut VirtualMachine, value: &str, b: PyObjectRef) -> PyResult {
    // let value = a
    if objtype::isinstance(&b, &vm.ctx.int_type()) {
        let pos = objint::get_value(&b).to_i32().unwrap();
        let idx = value.to_string().get_pos(pos);
        Ok(vm.new_str(value[idx..idx + 1].to_string()))
    } else {
        match &(*b.borrow()).kind {
            &PyObjectKind::Slice {
                start: _,
                stop: _,
                step: _,
            } => Ok(vm.new_str(value.to_string().get_slice_items(&b).to_string())),
            _ => panic!(
                "TypeError: indexing type {:?} with index {:?} is not supported (yet?)",
                value, b
            ),
        }
    }
}

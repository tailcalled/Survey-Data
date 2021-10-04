use serde_json::Value;

pub fn contains(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (&a, &b) {
        (Null, _) => true,
        (Bool(b), Bool(c)) => b == c,
        (Number(n), Number(m)) => n == m,
        (String(s), String(t)) => s == t,
        (Array(arr), Array(lst)) => {
            let mut each = true;
            for el in arr {
                let mut some = false;
                for it in lst {
                    if contains(el, it) {
                        some = true;
                    }
                }
                if !some {
                    each = false;
                }
            }
            each
        },
        (Object(obj), Object(val)) => {
            let mut each = true;
            for (elk, elv) in obj {
                println!("Searching {:?}:{:?}", elk, elv);
                let mut some = false;
                for (itk, itv) in val {
                    println!("Testing {:?}:{:?} vs {:?}:{:?}", elk, elv, itk, itv);
                    if elk == itk && contains(elv, itv) {
                        println!("Found!");
                        some = true;
                    }
                    else {
                        println!("Nope.");
                    }
                }
                if !some {
                    println!("Failed search");
                    each = false;
                }
            }
            each
        }
        _ => false
    }
}